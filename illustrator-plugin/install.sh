#!/bin/bash
# egui_expressive Illustrator Plugin Installer (macOS)

echo "============================================"
echo " egui_expressive Illustrator Plugin Installer"
echo "============================================"
echo ""

UPIA="/Library/Application Support/Adobe/Adobe Desktop Common/RemoteComponents/UPI/UnifiedPluginInstallerAgent/UnifiedPluginInstallerAgent.app/Contents/MacOS/UnifiedPluginInstallerAgent"

ZXP_FILE="$(dirname "$0")/egui_expressive_export-1.0.0.zxp"
if [ ! -f "$ZXP_FILE" ]; then
  echo "ERROR: egui_expressive_export-1.0.0.zxp not found in this folder."
  exit 1
fi

echo "Removing any previous version..."
if [ -f "$UPIA" ]; then
    "$UPIA" /remove "com.egui-expressive.illustrator-exporter" >/dev/null 2>&1 || true
    echo "Previous version removed (if present)."
else
    rm -rf "$HOME/Library/Application Support/Adobe/CEP/extensions/com.egui-expressive.illustrator-exporter"
    echo "Previous version removed (if present)."
fi

if [ -f "$UPIA" ]; then
    echo "Found UPIA. Installing ZXP..."
    "$UPIA" /install "$ZXP_FILE"
else
    echo "ERROR: UPIA not found."
    echo "To install manually on macOS, use UPIA:"
    echo "  /Library/Application\ Support/Adobe/Adobe\ Desktop\ Common/RemoteComponents/UPI/UnifiedPluginInstallerAgent/UnifiedPluginInstallerAgent /install \"\$ZXP_FILE\""
fi

echo "Enabling CEP debug mode for self-signed extensions..."
defaults write com.adobe.CSXS.10 PlayerDebugMode 1 2>/dev/null || true
defaults write com.adobe.CSXS.11 PlayerDebugMode 1 2>/dev/null || true
defaults write com.adobe.CSXS.12 PlayerDebugMode 1 2>/dev/null || true
defaults write com.adobe.CSXS.13 PlayerDebugMode 1 2>/dev/null || true
defaults write com.adobe.CSXS.14 PlayerDebugMode 1 2>/dev/null || true
defaults write com.adobe.CSXS.15 PlayerDebugMode 1 2>/dev/null || true
echo "CEP debug mode enabled (CSXS.10 through CSXS.15)."

