#!/bin/bash
# egui_expressive Illustrator Plugin Installer (macOS)

PLUGIN_NAME="egui_expressive_export"
INSTALLED=0

# Try multiple Illustrator versions
for VERSION in 28 27 26 25; do
    PLUGIN_BASE="$HOME/Library/Application Support/Adobe/UXP/PluginsStorage/ILST/$VERSION/develop"
    if [ -d "$PLUGIN_BASE" ]; then
        PLUGIN_DIR="$PLUGIN_BASE/$PLUGIN_NAME"
        mkdir -p "$PLUGIN_DIR"
        cp manifest.json plugin.js index.html "$PLUGIN_DIR/"
        echo "✓ Plugin installed to: $PLUGIN_DIR"
        INSTALLED=1
    fi
done

if [ $INSTALLED -eq 0 ]; then
    echo "⚠️  Could not find Illustrator UXP plugin directory."
    echo ""
    echo "Please manually copy these files:"
    echo "  manifest.json, plugin.js, index.html"
    echo ""
    echo "To: ~/Library/Application Support/Adobe/UXP/PluginsStorage/ILST/28/develop/$PLUGIN_NAME/"
fi

echo ""
echo "After installation:"
echo "  1. Restart Adobe Illustrator"
echo "  2. Go to Plugins > Plugin Manager"
echo "  3. Enable 'egui_expressive Export'"
