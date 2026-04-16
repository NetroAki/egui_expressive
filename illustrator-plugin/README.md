# egui_expressive Illustrator Exporter

## Installation

### Windows (Easy)
1. Download or clone this repository
2. Navigate to the `illustrator-plugin/` folder
3. Double-click `install.bat`
4. Restart Adobe Illustrator
5. Go to **Plugins → Plugin Manager** and enable **egui_expressive Export**

### Windows (Installer .exe)
Build the installer using NSIS:
1. Install [NSIS](https://nsis.sourceforge.io/)
2. Run `installer/build_installer.bat`
3. Run the generated `egui_expressive_plugin_installer.exe`

### macOS
1. Open Terminal in the `illustrator-plugin/` folder
2. Run: `chmod +x install.sh && ./install.sh`
3. Restart Adobe Illustrator
4. Go to **Plugins → Plugin Manager** and enable **egui_expressive Export**

### Manual Installation
Copy `manifest.json`, `plugin.js`, and `index.html` to:
- **Windows**: `%APPDATA%\Adobe\UXP\PluginsStorage\ILST\28\develop\egui_expressive_export\`
- **macOS**: `~/Library/Application Support/Adobe/UXP/PluginsStorage/ILST/28/develop/egui_expressive_export/`

Replace `28` with your Illustrator version number (28=2024, 27=2023, 26=2022).

## Usage

1. Open your Illustrator document
2. Open the plugin panel: **Window → Plugins → egui_expressive**
3. Select the artboards you want to export
4. Configure options:
   - **Use naming conventions** — Name layers `row-toolbar`, `btn-save`, `col-sidebar` for better output
   - **Infer gaps** — Automatically detect spacing between elements
   - **Include JSON sidecar** — Export element data for manual inspection
5. Click **Export**
6. Copy the generated code or save to your project

## Naming Conventions

Name your Illustrator layers to get better code output:

| Layer name prefix | Generated code |
|---|---|
| `row-*` | `ui.horizontal(\|ui\| { ... })` |
| `col-*` | `ui.vertical(\|ui\| { ... })` |
| `btn-*` | `ui.button("...")` |
| `label-*` | `ui.label("...")` |
| `card-*` | `egui::Frame::NONE.fill(...).show(...)` |
| `scroll-*` | `egui::ScrollArea::vertical().show(...)` |
| `divider` | `ui.separator()` |
| `spacer` | `ui.add_space(8.0)` |
| `badge-*` | `egui_expressive::Badge::new("...")` |
| `gap-N` | Sets item_spacing to N px |

## Output format

Each artboard generates a `draw_<name>(ui: &mut egui::Ui)` function. Add it to your project and call it from your egui update loop.
