# egui_expressive Illustrator Exporter

This is a **CEP extension** (not UXP) that exports Illustrator artboards to egui Rust code. It installs as a `.zxp` file.
This plugin uses CEP (Common Extensibility Platform). The `manifest.json` file is included for reference but the active runtime is CEP via the `.zxp` installer.

## Installation

1. Build the `.zxp` file by running `installer/build_zxp.sh` (macOS) or `installer\build_zxp.bat` (Windows).
2. Install the generated `.zxp` file:
   - **Windows**: Double-click `install.bat`
   - **macOS**: Run `chmod +x install.sh && ./install.sh`
3. Restart Adobe Illustrator.
4. Go to **Window → Extensions → egui_expressive Export** to open the panel.

## Usage

1. Open your Illustrator document
2. Open the plugin panel: **Window → Extensions → egui_expressive Export**
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
