#!/usr/bin/env bash
# Run a bounded X11 Linux runtime smoke for the shared showcase example.
#
# The script intentionally writes all screenshots, logs, window metadata, and
# timing samples under ARTIFACT_DIR so CI/local runs are easy to inspect without
# modifying the repository. It uses Xvfb for an isolated display and Openbox when
# available for focus/resize behavior; if Openbox is missing, focus is recorded
# as skipped instead of being claimed.
set -euo pipefail

ROOT_DIR="${ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
ARTIFACT_DIR="${ARTIFACT_DIR:-/tmp/egui-expressive-smoke/linux-x11-$(date -u +%Y%m%dT%H%M%SZ)}"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/egui-linux-smoke-target}"
EXAMPLE_BIN="$TARGET_DIR/debug/examples/cross_platform_showcase"
TITLE="Cross-Platform Showcase"

mkdir -p "$ARTIFACT_DIR"

# Fail early for tools that are required to build, launch, inspect, and capture
# the virtual X11 desktop. Optional tools are probed later and recorded in the
# environment artifact.
need_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required tool: $1" | tee -a "$ARTIFACT_DIR/smoke.log" >&2
    exit 2
  fi
}

for tool in cargo rustc Xvfb xdotool xwininfo xdpyinfo; do
  need_tool "$tool"
done

if ! command -v scrot >/dev/null 2>&1 && ! command -v import >/dev/null 2>&1; then
  echo "missing screenshot tool: need scrot or ImageMagick import" | tee -a "$ARTIFACT_DIR/smoke.log" >&2
  exit 2
fi

if ! command -v identify >/dev/null 2>&1; then
  echo "missing ImageMagick identify for screenshot statistics" | tee -a "$ARTIFACT_DIR/smoke.log" >&2
  exit 2
fi

{
  echo "root=$ROOT_DIR"
  echo "artifact_dir=$ARTIFACT_DIR"
  echo "target_dir=$TARGET_DIR"
  uname -a
  rustc --version
  cargo --version
  echo "tools:"
  for tool in Xvfb xdotool xwininfo xdpyinfo scrot import identify wmctrl openbox; do
    printf '  %s=' "$tool"
    command -v "$tool" || true
  done
} > "$ARTIFACT_DIR/environment.txt"

printf '=== cargo build ===\n' | tee "$ARTIFACT_DIR/smoke.log"
(
  cd "$ROOT_DIR"
  CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TARGET_DIR" cargo build --example cross_platform_showcase
) > "$ARTIFACT_DIR/cargo-build.log" 2>&1

if [[ ! -x "$EXAMPLE_BIN" ]]; then
  echo "missing built example: $EXAMPLE_BIN" | tee -a "$ARTIFACT_DIR/smoke.log" >&2
  exit 1
fi

# Pick a high display number to avoid colliding with an existing local desktop
# or another concurrent smoke run.
pick_display() {
  local candidate
  for _ in $(seq 1 50); do
    candidate=$(( 90 + (RANDOM % 80) ))
    if [[ ! -e "/tmp/.X11-unix/X${candidate}" ]]; then
      printf ':%s' "$candidate"
      return 0
    fi
  done
  echo "could not find free X display" >&2
  return 1
}

# Prefer scrot for simple root-window captures, with ImageMagick import as a
# fallback on distributions that do not install scrot.
capture_screen() {
  local image_path="$1"
  if command -v scrot >/dev/null 2>&1; then
    scrot "$image_path"
  else
    import -window root "$image_path"
  fi
}

# Store basic image statistics and fail closed when a capture is fully black or
# flat. This catches renderer/window/capture failures without relying on a golden
# image for this broad compatibility smoke.
write_image_stats() {
  local image_path="$1"
  local stats_path="$2"
  identify "$image_path" > "$stats_path"
  identify -format 'mean=%[fx:mean]\nstandard_deviation=%[fx:standard_deviation]\nwidth=%w\nheight=%h\n' "$image_path" >> "$stats_path"
  awk -F= '
    $1 == "mean" { mean = $2 + 0 }
    $1 == "standard_deviation" { std = $2 + 0 }
    END {
      if (mean <= 0 || std <= 0) {
        printf("non-black check failed: mean=%s std=%s\n", mean, std) > "/dev/stderr";
        exit 1;
      }
    }
  ' "$stats_path"
}

# Run the same showcase lifecycle at one DPI/screen-size combination:
# build artifact already exists, launch, discover window, focus if possible,
# capture, resize, capture again, and verify the process survives.
run_case() {
  local name="$1"
  local dpi="$2"
  local screen="$3"
  local case_dir="$ARTIFACT_DIR/$name"
  mkdir -p "$case_dir"

  local display
  display="$(pick_display)"
  printf '=== %s display=%s dpi=%s screen=%s ===\n' "$name" "$display" "$dpi" "$screen" | tee -a "$ARTIFACT_DIR/smoke.log"

  Xvfb "$display" -screen 0 "${screen}x24" -dpi "$dpi" > "$case_dir/xvfb.log" 2>&1 &
  local xvfb_pid=$!
  sleep 0.8

  local wm_pid=""
  if command -v openbox >/dev/null 2>&1; then
    DISPLAY="$display" openbox > "$case_dir/openbox.log" 2>&1 &
    wm_pid=$!
    sleep 0.8
  fi

  local start_ms
  start_ms=$(date +%s%3N)
  DISPLAY="$display" LIBGL_ALWAYS_SOFTWARE="${LIBGL_ALWAYS_SOFTWARE:-1}" "$EXAMPLE_BIN" \
    > "$case_dir/app.stdout.log" 2> "$case_dir/app.stderr.log" &
  local app_pid=$!
  echo "$app_pid" > "$case_dir/app.pid"

  local window_id=""
  for _ in $(seq 1 80); do
    if ! kill -0 "$app_pid" >/dev/null 2>&1; then
      echo "app exited before window discovery" | tee -a "$ARTIFACT_DIR/smoke.log" >&2
      wait "$app_pid" || true
      [[ -n "$wm_pid" ]] && kill "$wm_pid" >/dev/null 2>&1 || true
      kill "$xvfb_pid" >/dev/null 2>&1 || true
      return 1
    fi
    window_id=$(DISPLAY="$display" xdotool search --name "$TITLE" 2>/dev/null | head -n 1 || true)
    [[ -n "$window_id" ]] && break
    sleep 0.25
  done

  if [[ -z "$window_id" ]]; then
    echo "could not find showcase window" | tee -a "$ARTIFACT_DIR/smoke.log" >&2
    kill "$app_pid" >/dev/null 2>&1 || true
    [[ -n "$wm_pid" ]] && kill "$wm_pid" >/dev/null 2>&1 || true
    kill "$xvfb_pid" >/dev/null 2>&1 || true
    return 1
  fi

  local found_ms
  found_ms=$(date +%s%3N)
  printf 'window_discovery_ms=%s\n' "$((found_ms - start_ms))" > "$case_dir/performance.txt"
  echo "$window_id" > "$case_dir/window-id.txt"

  DISPLAY="$display" xdpyinfo > "$case_dir/xdpyinfo.txt"
  DISPLAY="$display" xwininfo -id "$window_id" > "$case_dir/window-before.txt"

  if DISPLAY="$display" xdotool windowactivate "$window_id" >/dev/null 2> "$case_dir/focus.err"; then
    echo "focus=passed" > "$case_dir/focus.txt"
  else
    echo "focus=skipped-or-blocked" > "$case_dir/focus.txt"
  fi

  sleep 1.5
  DISPLAY="$display" capture_screen "$case_dir/screenshot-before.png"
  write_image_stats "$case_dir/screenshot-before.png" "$case_dir/screenshot-before.stats"

  DISPLAY="$display" xdotool windowsize "$window_id" 640 480
  sleep 1.0
  DISPLAY="$display" xwininfo -id "$window_id" > "$case_dir/window-after-resize.txt"
  DISPLAY="$display" capture_screen "$case_dir/screenshot-after-resize.png"
  write_image_stats "$case_dir/screenshot-after-resize.png" "$case_dir/screenshot-after-resize.stats"

  if kill -0 "$app_pid" >/dev/null 2>&1; then
    echo "process_alive_after_resize=true" > "$case_dir/lifecycle.txt"
  else
    echo "process_alive_after_resize=false" > "$case_dir/lifecycle.txt"
    wait "$app_pid" || true
    [[ -n "$wm_pid" ]] && kill "$wm_pid" >/dev/null 2>&1 || true
    kill "$xvfb_pid" >/dev/null 2>&1 || true
    return 1
  fi

  kill "$app_pid" >/dev/null 2>&1 || true
  wait "$app_pid" >/dev/null 2>&1 || true
  echo "terminated_after_capture=true" >> "$case_dir/lifecycle.txt"

  [[ -n "$wm_pid" ]] && kill "$wm_pid" >/dev/null 2>&1 || true
  kill "$xvfb_pid" >/dev/null 2>&1 || true
  wait "$xvfb_pid" >/dev/null 2>&1 || true
}

run_case normal-dpi 96 1280x800
run_case high-dpi 192 1440x900

# Scan only app stdout/stderr for runtime failure markers. Compositor/window
# manager logs can contain unrelated diagnostics and are preserved separately.
if grep -RniE 'panic|panicked|segmentation fault|device lost|wgpu error|gl error' "$ARTIFACT_DIR"/*/app.stdout.log "$ARTIFACT_DIR"/*/app.stderr.log > "$ARTIFACT_DIR/error-scan.txt" 2>/dev/null; then
  echo "runtime app error markers found; see $ARTIFACT_DIR/error-scan.txt" | tee -a "$ARTIFACT_DIR/smoke.log" >&2
  exit 1
fi

echo "no runtime app error markers found" > "$ARTIFACT_DIR/error-scan.txt"
find "$ARTIFACT_DIR" -maxdepth 2 -type f | sort > "$ARTIFACT_DIR/artifact-list.txt"
cat > "$ARTIFACT_DIR/summary.txt" <<SUMMARY
result=passed
scope=Linux X11/Xvfb bounded runtime smoke for examples/cross_platform_showcase.rs
normal_dpi=96
high_dpi=192
artifact_dir=$ARTIFACT_DIR
SUMMARY

printf 'Linux smoke passed; artifacts: %s\n' "$ARTIFACT_DIR" | tee -a "$ARTIFACT_DIR/smoke.log"
