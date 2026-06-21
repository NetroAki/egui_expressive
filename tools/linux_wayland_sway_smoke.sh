#!/usr/bin/env bash
# Run a bounded Wayland Linux runtime smoke for the shared showcase example.
#
# The script starts isolated headless Sway/wlroots sessions, forces winit to use
# Wayland, captures screenshots with grim, records Sway tree metadata, and fails
# closed on blank captures or app runtime error markers. It is intended for local
# validation and CI-style Linux runners; it does not require a physical monitor.
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
ARTIFACT_DIR="${ARTIFACT_DIR:-/tmp/egui-expressive-smoke/linux-wayland-sway-$(date -u +%Y%m%dT%H%M%SZ)}"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/egui-local-virtual-wayland-sway-target}"
EXAMPLE_BIN="$TARGET_DIR/debug/examples/cross_platform_showcase"
mkdir -p "$ARTIFACT_DIR"
{
  echo "root=$ROOT_DIR"
  echo "artifact_dir=$ARTIFACT_DIR"
  echo "target_dir=$TARGET_DIR"
  uname -a
  rustc --version
  cargo --version
  for t in sway swaymsg grim wayland-info identify; do printf '%s=' "$t"; command -v "$t" || true; done
} > "$ARTIFACT_DIR/environment.txt"
(
  cd "$ROOT_DIR"
  CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TARGET_DIR" cargo build --example cross_platform_showcase
) > "$ARTIFACT_DIR/cargo-build.log" 2>&1
# Run the same lifecycle at one Wayland output scale: launch Sway, launch the
# showcase, verify the toplevel exists, capture, try a floating resize, capture
# again, and shut down both app and compositor.
run_case() {
  local name="$1" width="$2" height="$3" scale="$4"
  local case_dir="$ARTIFACT_DIR/$name"
  # Keep XDG_RUNTIME_DIR short; Wayland socket paths have strict length limits.
  local runtime_dir
  runtime_dir="$(mktemp -d /tmp/egui-wl-${name}.XXXXXX)"
  local socket="egui-sway-$RANDOM"
  mkdir -p "$case_dir" "$runtime_dir"
  chmod 700 "$runtime_dir"
  cat > "$case_dir/sway.config" <<CFG
xwayland disable
font pango:monospace 8
output * resolution ${width}x${height} scale ${scale}
default_border none
focus_follows_mouse no
seat * hide_cursor 100
CFG
  # Use wlroots' headless backend and pixman renderer so the proof is independent
  # of a real display server or GPU adapter.
  XDG_RUNTIME_DIR="$runtime_dir" WAYLAND_DISPLAY="$socket" WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 WLR_RENDERER=pixman sway -c "$case_dir/sway.config" -d > "$case_dir/sway.log" 2>&1 &
  local sway_pid=$!
  local app_pid=""
  cleanup_case() {
    if [[ -n "${app_pid:-}" ]]; then kill "$app_pid" >/dev/null 2>&1 || true; fi
    XDG_RUNTIME_DIR="$runtime_dir" WAYLAND_DISPLAY="$socket" SWAYSOCK="$sway_sock" swaymsg exit >/dev/null 2>&1 || true
    kill "$sway_pid" >/dev/null 2>&1 || true
    wait "$sway_pid" >/dev/null 2>&1 || true
    rm -rf "$runtime_dir" >/dev/null 2>&1 || true
  }
  # Sway may choose a concrete wayland-N socket name even when WAYLAND_DISPLAY is
  # seeded. Discover the actual Wayland and IPC sockets before launching clients.
  local sway_sock=""
  for _ in $(seq 1 100); do
    local found_socket
    found_socket=$(find "$runtime_dir" -maxdepth 1 -type s -name 'wayland-*' -printf '%f\n' 2>/dev/null | head -n 1 || true)
    if [[ -n "$found_socket" ]]; then
      socket="$found_socket"
      sway_sock=$(find "$runtime_dir" -maxdepth 1 -type s -name 'sway-ipc*.sock' -print 2>/dev/null | head -n 1 || true)
      break
    fi
    sleep 0.1
  done
  if [[ ! -S "$runtime_dir/$socket" ]]; then
    echo "sway socket not created" >&2
    tail -n 140 "$case_dir/sway.log" >&2 || true
    cleanup_case
    return 1
  fi
  if [[ -z "$sway_sock" ]]; then
    sway_sock=$(find "$runtime_dir" -maxdepth 1 -type s -name 'sway-ipc*.sock' -print 2>/dev/null | head -n 1 || true)
  fi
  echo "WAYLAND_DISPLAY=$socket" > "$case_dir/socket.env"
  echo "SWAYSOCK=$sway_sock" >> "$case_dir/socket.env"
  XDG_RUNTIME_DIR="$runtime_dir" WAYLAND_DISPLAY="$socket" wayland-info > "$case_dir/wayland-info.txt" 2> "$case_dir/wayland-info.err" || true
  local start_ms
  start_ms=$(date +%s%3N)
  XDG_RUNTIME_DIR="$runtime_dir" WAYLAND_DISPLAY="$socket" WINIT_UNIX_BACKEND=wayland LIBGL_ALWAYS_SOFTWARE=1 "$EXAMPLE_BIN" > "$case_dir/app.stdout.log" 2> "$case_dir/app.stderr.log" &
  app_pid=$!
  echo "$app_pid" > "$case_dir/app.pid"
  for _ in $(seq 1 80); do
    if ! kill -0 "$app_pid" >/dev/null 2>&1; then
      echo "app exited before capture" >&2
      wait "$app_pid" || true
      cleanup_case
      return 1
    fi
    if XDG_RUNTIME_DIR="$runtime_dir" WAYLAND_DISPLAY="$socket" SWAYSOCK="$sway_sock" swaymsg -t get_tree > "$case_dir/tree.json" 2> "$case_dir/tree.err" && grep -q 'Cross-Platform Showcase' "$case_dir/tree.json"; then
      break
    fi
    sleep 0.25
  done
  if ! grep -q 'Cross-Platform Showcase' "$case_dir/tree.json" 2>/dev/null; then
    echo "Wayland toplevel not found in sway tree" >&2
    cleanup_case
    return 1
  fi
  local found_ms
  found_ms=$(date +%s%3N)
  echo "window_discovery_ms=$((found_ms - start_ms))" > "$case_dir/performance.txt"
  sleep 1.0
  XDG_RUNTIME_DIR="$runtime_dir" WAYLAND_DISPLAY="$socket" grim "$case_dir/screenshot-before.png"
  identify "$case_dir/screenshot-before.png" > "$case_dir/screenshot-before.stats"
  identify -format 'mean=%[fx:mean]\nstandard_deviation=%[fx:standard_deviation]\nwidth=%w\nheight=%h\n' "$case_dir/screenshot-before.png" >> "$case_dir/screenshot-before.stats"
  awk -F= '$1=="mean"{m=$2+0}$1=="standard_deviation"{s=$2+0}END{if(m<=0 || s<=0){printf("blank screenshot mean=%s std=%s\n",m,s)>"/dev/stderr"; exit 1}}' "$case_dir/screenshot-before.stats"
  # Floating resize is best-effort because the app/compositor can enforce minimum
  # sizes. The Sway tree after resize is recorded so docs can state the actual
  # resulting size instead of assuming the requested size was accepted.
  XDG_RUNTIME_DIR="$runtime_dir" WAYLAND_DISPLAY="$socket" SWAYSOCK="$sway_sock" swaymsg '[title="Cross-Platform Showcase"] floating enable' > "$case_dir/floating.log" 2>&1 || true
  XDG_RUNTIME_DIR="$runtime_dir" WAYLAND_DISPLAY="$socket" SWAYSOCK="$sway_sock" swaymsg '[title="Cross-Platform Showcase"] resize set width 640 height 480' > "$case_dir/resize.log" 2>&1 || true
  sleep 1.0
  XDG_RUNTIME_DIR="$runtime_dir" WAYLAND_DISPLAY="$socket" SWAYSOCK="$sway_sock" swaymsg -t get_tree > "$case_dir/tree-after-resize.json" 2> "$case_dir/tree-after-resize.err" || true
  XDG_RUNTIME_DIR="$runtime_dir" WAYLAND_DISPLAY="$socket" grim "$case_dir/screenshot-after-resize.png"
  identify "$case_dir/screenshot-after-resize.png" > "$case_dir/screenshot-after-resize.stats"
  identify -format 'mean=%[fx:mean]\nstandard_deviation=%[fx:standard_deviation]\nwidth=%w\nheight=%h\n' "$case_dir/screenshot-after-resize.png" >> "$case_dir/screenshot-after-resize.stats"
  awk -F= '$1=="mean"{m=$2+0}$1=="standard_deviation"{s=$2+0}END{if(m<=0 || s<=0){printf("blank resize screenshot mean=%s std=%s\n",m,s)>"/dev/stderr"; exit 1}}' "$case_dir/screenshot-after-resize.stats"
  if kill -0 "$app_pid" >/dev/null 2>&1; then
    echo "process_alive_after_resize=true" > "$case_dir/lifecycle.txt"
  else
    echo "process_alive_after_resize=false" > "$case_dir/lifecycle.txt"
    cleanup_case
    return 1
  fi
  kill "$app_pid" >/dev/null 2>&1 || true
  wait "$app_pid" >/dev/null 2>&1 || true
  app_pid=""
  echo "terminated_after_capture=true" >> "$case_dir/lifecycle.txt"
  cleanup_case
}
run_case normal-dpi 1280 800 1
run_case high-dpi 1440 900 2
# Scan app logs for clear runtime failure markers after both DPI/scale cases
# complete. Compositor logs are kept for debugging but not treated as app errors.
if grep -RniE 'panic|panicked|segmentation fault|device lost|wgpu error|gl error' "$ARTIFACT_DIR"/*/app.stdout.log "$ARTIFACT_DIR"/*/app.stderr.log > "$ARTIFACT_DIR/error-scan.txt" 2>/dev/null; then
  echo "runtime app error markers found" >&2
  exit 1
fi
echo "no runtime app error markers found" > "$ARTIFACT_DIR/error-scan.txt"
find "$ARTIFACT_DIR" -maxdepth 2 -type f | sort > "$ARTIFACT_DIR/artifact-list.txt"
cat > "$ARTIFACT_DIR/summary.txt" <<SUMMARY
result=passed
scope=Local Linux Wayland Sway headless smoke for examples/cross_platform_showcase.rs
backend=wayland/sway-headless/wlroots
normal_scale=1
high_dpi_scale=2
artifact_dir=$ARTIFACT_DIR
SUMMARY
echo "Wayland/Sway smoke passed; artifacts: $ARTIFACT_DIR"
