#!/bin/bash
# install.sh — Per-user CEP extension installer for macOS (no admin required)
# Extracts .zxp to ~/Library/Application Support/Adobe/CEP/extensions/

EXT_ID="com.egui-expressive.illustrator-exporter"
EXT_NAME="egui_expressive Exporter"

echo "============================================"
echo " $EXT_NAME Installer"
echo "============================================"
echo ""

# --- Resolve ZXP path ---
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ZXP_FILE="$SCRIPT_DIR/egui_expressive_export-1.0.0.zxp"
if [ ! -f "$ZXP_FILE" ]; then
  ZXP_FILE="$SCRIPT_DIR/../dist/egui_expressive_export-1.0.0.zxp"
fi
if [ ! -f "$ZXP_FILE" ]; then
  echo "ERROR: egui_expressive_export-1.0.0.zxp not found."
  echo "Place the .zxp next to this script, or run build_zxp.sh first."
  exit 1
fi

echo "[INFO] Found: $ZXP_FILE"

# --- Uninstall: discover and remove from ALL known locations ---
echo "[INFO] Removing previous version..."
FOUND_OLD=0

for BASE in \
  "$HOME/Library/Application Support/Adobe/CEP/extensions" \
  "/Library/Application Support/Adobe/CEP/extensions" \
  "/Library/Application Support/Adobe/CEPServiceManager4/extensions"
do
  OLD_DIR="$BASE/$EXT_ID"
  if [ -d "$OLD_DIR" ]; then
    echo "[INFO]   Found old install at: $OLD_DIR"
    rm -rf "$OLD_DIR"
    if [ $? -ne 0 ]; then
      echo "ERROR:   Failed to delete $OLD_DIR"
      echo "ERROR:   Close Illustrator and retry."
      exit 1
    fi
    FOUND_OLD=1
    echo "[INFO]   Deleted."
  fi
done

if [ $FOUND_OLD -eq 0 ]; then
  echo "[INFO] No previous installation found."
fi

# Also try UPIA unregister if available (optional cleanup)
UPIA="/Library/Application Support/Adobe/Adobe Desktop Common/RemoteComponents/UPI/UnifiedPluginInstallerAgent/UnifiedPluginInstallerAgent.app/Contents/MacOS/UnifiedPluginInstallerAgent"
if [ -f "$UPIA" ]; then
  echo "[INFO] Unregistering from Adobe extension database..."
  if "$UPIA" /remove "$EXT_ID"; then
    echo "[INFO] Unregister done."
  else
    echo "[WARN] UPIA unregister failed (non-critical, extension folder already deleted)."
  fi
fi

# --- Install: per-user location ---
EXT_DIR="$HOME/Library/Application Support/Adobe/CEP/extensions/$EXT_ID"
echo "[INFO] Installing to: $EXT_DIR"
mkdir -p "$EXT_DIR"

# Extract .zxp (it's a signed zip)
unzip -q -o "$ZXP_FILE" -d "$EXT_DIR"
if [ $? -ne 0 ]; then
  echo "ERROR: Extraction failed. The .zxp may be corrupt."
  exit 1
fi
echo "[INFO] Extraction complete."

# --- Enable CEP debug mode (per-user, no admin) ---
echo "[INFO] Enabling CEP debug mode for self-signed extensions..."
for V in 10 11 12 13 14 15; do
  if defaults write "com.adobe.CSXS.$V" PlayerDebugMode 1 2>/dev/null; then
    echo "[INFO]   CSXS.$V debug mode enabled."
  else
    echo "[WARN]   CSXS.$V registry write failed."
  fi
done

echo ""
echo "============================================"
echo " Installation complete!"
echo ""
echo " Restart Illustrator and open:"
echo " Window → Extensions → egui_expressive Export"
echo "============================================"
