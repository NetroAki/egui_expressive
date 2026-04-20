#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────────────────
# build_zxp.sh — Package egui_expressive Exporter as a signed .zxp extension
#
# Prerequisites:
#   - ZXPSignCmd (download from https://github.com/Adobe-CEP/CEP-Resources)
#     Place it somewhere in $PATH or set ZXPSIGNCMD env var.
#   - Or: npm install -g zxp-sign-cmd  (Node.js wrapper)
#
# Usage:
#   ./build_zxp.sh                    # Build with self-signed cert
#   ./build_zxp.sh /path/to/cert.p12  # Build with existing cert
# ──────────────────────────────────────────────────────────────────────────────
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$PLUGIN_DIR")"
OUTPUT_DIR="$PROJECT_ROOT/dist"

EXTENSION_ID="com.egui-expressive.illustrator-exporter"
VERSION="1.0.0"
ZXP_NAME="egui_expressive_export-${VERSION}.zxp"

# Certificate defaults (for self-signed)
CERT_COUNTRY="US"
CERT_STATE="NA"
CERT_ORG="egui_expressive"
CERT_NAME="egui_expressive Exporter"
CERT_PASSWORD="${ZXP_SIGN_PASSWORD:-}"
CERT_FILE="$OUTPUT_DIR/cert.p12"

# Timestamp authority
TSA_URL="http://timestamp.digicert.com"

# ─── Colors ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

# Ephemeral password warning (must be after warn() is defined)
if [ -z "$CERT_PASSWORD" ]; then
    CERT_PASSWORD="selfsign_$(date +%s)"
    warn "No ZXP_SIGN_PASSWORD env var set — using ephemeral password. Set ZXP_SIGN_PASSWORD for reproducible builds."
fi

# ─── Find ZXPSignCmd ────────────────────────────────────────────────────────
find_signer() {
    if [ -n "${ZXPSIGNCMD:-}" ] && command -v "$ZXPSIGNCMD" &>/dev/null; then
        echo "$ZXPSIGNCMD"
        return
    fi

    if command -v ZXPSignCmd &>/dev/null; then
        echo "ZXPSignCmd"
        return
    fi

    # Check common locations
    local candidates=(
        "$HOME/.local/bin/ZXPSignCmd"
        "/usr/local/bin/ZXPSignCmd"
        "$PROJECT_ROOT/tools/ZXPSignCmd"
        "$PLUGIN_DIR/installer/ZXPSignCmd"
    )
    for c in "${candidates[@]}"; do
        if [ -x "$c" ]; then
            echo "$c"
            return
        fi
    done

    # Check for Windows .exe via Wine (Linux)
    if command -v wine &>/dev/null; then
        local exe_candidates=(
            "$PROJECT_ROOT/tools/ZXPSignCmd.exe"
            "$PLUGIN_DIR/installer/ZXPSignCmd.exe"
            "$HOME/.local/bin/ZXPSignCmd.exe"
        )
        for c in "${exe_candidates[@]}"; do
            if [ -f "$c" ]; then
                echo "wine:$c"
                return
            fi
        done
    fi

    # Check for npm wrapper
    if command -v npx &>/dev/null; then
        echo "npx zxp-sign-cmd"
        return
    fi

    echo "UNSIGNED"
}

# ─── Prepare staging directory ──────────────────────────────────────────────
prepare_staging() {
    local stage="$OUTPUT_DIR/staging"
    rm -rf "$stage"
    mkdir -p "$stage/CSXS"

    info "Staging extension files..." >&2

    # CEP manifest
    cp "$PLUGIN_DIR/CSXS/manifest.xml" "$stage/CSXS/manifest.xml"

    # Plugin files
    cp "$PLUGIN_DIR/index.html"  "$stage/index.html"
    cp "$PLUGIN_DIR/plugin.js"   "$stage/plugin.js"

    # ExtendScript host for CEP mode
    if [ -f "$PLUGIN_DIR/host.jsx" ]; then
        cp "$PLUGIN_DIR/host.jsx" "$stage/host.jsx"
    fi

    # NOTE: .debug file is excluded from production builds.
    # For development, create it manually:
    #   cat > .debug << 'EOF'
    #   <?xml version="1.0"?><ExtensionList><Extension Id="com.egui-expressive.illustrator-exporter.panel"><HostList><Host Name="ILST" Port="8088"/></HostList></Extension></ExtensionList>
    #   EOF

    echo "$stage"
}

# ─── Run signer command (handles wine: prefix) ───────────────────────────────
run_signer() {
    local signer="$1"; shift
    if [[ "$signer" == wine:* ]]; then
        local exe="${signer#wine:}"
        local win_args=()
        for arg in "$@"; do
            # Convert absolute paths to Windows format for Wine
            if [[ "$arg" == /* ]]; then
                win_args+=("$(winepath -w "$arg" 2>/dev/null || echo "$arg")")
            else
                win_args+=("$arg")
            fi
        done
        wine "$exe" "${win_args[@]}" 2>&1 | grep -v "^0.*fixme:" | grep -v "^$"
    elif [[ "$signer" == "npx zxp-sign-cmd" ]]; then
        npx zxp-sign-cmd "$@"
    else
        "$signer" "$@"
    fi
}

# ─── Generate self-signed certificate ───────────────────────────────────────
generate_cert() {
    local signer="$1"
    local cert_path="$2"

    info "Generating self-signed certificate..."
    mkdir -p "$(dirname "$cert_path")"

    run_signer "$signer" -selfSignedCert \
        "$CERT_COUNTRY" "$CERT_STATE" "$CERT_ORG" "$CERT_NAME" \
        "$CERT_PASSWORD" "$cert_path"

    if [ $? -ne 0 ]; then
        error "Failed to generate self-signed certificate"
    fi
    info "Certificate created: $cert_path"
}

# ─── Sign and package ───────────────────────────────────────────────────────
sign_and_package() {
    local signer="$1"
    local stage="$2"
    local cert="$3"
    local output="$OUTPUT_DIR/$ZXP_NAME"

    mkdir -p "$OUTPUT_DIR"

    # Remove old ZXP before signing to avoid Wine file-locking issues
    rm -f "$output"

    info "Signing and packaging..."
    info "  Input:  $stage"
    info "  Output: $output"
    info "  Cert:   $cert"

    if ! run_signer "$signer" -sign \
        "$stage" "$output" "$cert" "$CERT_PASSWORD" \
        -tsa "$TSA_URL" 2>/dev/null; then
        warn "TSA unavailable, signing without timestamp"
        run_signer "$signer" -sign \
            "$stage" "$output" "$cert" "$CERT_PASSWORD" || { error "Failed to sign package"; }
    fi

    # Verify
    info "Verifying package..."
    if ! run_signer "$signer" -verify "$output" 2>/dev/null; then
        warn "Signature verification failed — package may not be accepted by all Creative Cloud versions"
        warn "This is common with self-signed certificates. The package will still install manually."
    else
        info "Signature verified successfully"
    fi

    local size
    size=$(du -sh "$output" | cut -f1)
    info "Package created: $output ($size)"
}

# ─── Unsigned packaging fallback ────────────────────────────────────────────
package_unsigned() {
    local stage="$1"
    local output="$OUTPUT_DIR/${ZXP_NAME%.zxp}-unsigned.zip"

    mkdir -p "$OUTPUT_DIR"
    info "Creating unsigned package (no ZXPSignCmd found)..."
    (cd "$stage" && zip -r "$output" .)
    mv "$output" "${output%.zip}.zxp"

    local size
    size=$(du -sh "${output%.zip}.zxp" | cut -f1)
    warn "Package is UNSIGNED — will not pass Creative Cloud verification."
    info "Unsigned package: ${output%.zip}.zxp ($size)"
    info "Install manually by extracting to:"
    echo "  macOS:   ~/Library/Application Support/Adobe/CEP/extensions/"
    echo "  Windows: %APPDATA%\\Adobe\\CEP\\extensions\\"
}

# ─── Main ────────────────────────────────────────────────────────────────────
main() {
    echo "╔══════════════════════════════════════════════════════╗"
    echo "║  egui_expressive Exporter — .zxp Package Builder    ║"
    echo "╚══════════════════════════════════════════════════════╝"
    echo ""

    # Stage files
    local stage
    stage=$(prepare_staging)

    # Find signing tool
    local signer
    signer=$(find_signer)

    if [ "$signer" = "UNSIGNED" ]; then
        warn "ZXPSignCmd not found — creating unsigned package."
        echo "  Download from: https://github.com/Adobe-CEP/CEP-Resources/tree/master/ZXPSignCMD"
        echo "  Or install: npm install -g zxp-sign-cmd"
        echo ""
        package_unsigned "$stage"
    else
        info "Using signer: $signer"

        # Determine cert path
        local cert="${1:-$CERT_FILE}"

        if [ -n "${1:-}" ]; then
            # User provided explicit cert path — use as-is
            if [ ! -f "$cert" ]; then
                error "Specified certificate not found: $cert"
            fi
            info "Using provided certificate: $cert"
        else
            # Default self-signed cert — always regenerate to avoid password mismatch
            if [ -f "$cert" ]; then
                info "Removing old self-signed cert to regenerate with current password..."
                rm -f "$cert"
            fi
            generate_cert "$signer" "$cert"
        fi

        # Sign and package
        sign_and_package "$signer" "$stage" "$cert"

        echo ""
        info "Done! Install with:"
        echo ""
        echo "  macOS:"
        echo "    \"/Library/Application Support/Adobe/Adobe Desktop Common/RemoteComponents/UPI/UnifiedPluginInstallerAgent/UnifiedPluginInstallerAgent\" /install \"$OUTPUT_DIR/$ZXP_NAME\""
        echo ""
        echo "  Windows:"
        echo "    \"C:\\Program Files\\Common Files\\Adobe\\Adobe Desktop Common\\RemoteComponents\\UPI\\UnifiedPluginInstallerAgent\\UnifiedPluginInstallerAgent.exe\" /install \"$OUTPUT_DIR\\$ZXP_NAME\""
        echo ""
        echo "  Or use Anastasiy's Extension Manager: https://install.anastasiy.com"
    fi

    # Cleanup staging
    rm -rf "$OUTPUT_DIR/staging"
}

main "$@"
