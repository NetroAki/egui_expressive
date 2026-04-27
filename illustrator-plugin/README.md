# egui_expressive Illustrator Exporter

This is a **CEP extension** (not UXP) that exports Illustrator artboards to egui Rust code. It installs as a `.zxp` file.
This plugin uses CEP (Common Extensibility Platform). The `manifest.json` file is included for reference but the active runtime is CEP via the `.zxp` installer.
The packaged extension includes the Rust `ai-parser` binary under `bin/<platform>/`; the CEP panel probes that bundled binary and uses it to enrich DOM extraction with project-file transforms, paths, artboards, and appearance data when the host permits local process execution.

## Installation

1. Build the platform-specific `.zxp` file by running `cd installer && bash build_zxp.sh` (macOS/Linux) or `cd installer && build_zxp.bat` (Windows). The output is named like `egui_expressive_export-1.0.0-darwin.zxp`, `egui_expressive_export-1.0.0-linux.zxp`, or `egui_expressive_export-1.0.0-win32.zxp`.
   - **Note**: You can set the `ZXP_SIGN_PASSWORD` environment variable to specify a custom password for the self-signed certificate. If not set, an ephemeral password is used.
2. Install the generated `.zxp` file:
   - **Windows**: Double-click `install.bat` from the Windows installer bundle (requires `egui_expressive_export-1.0.0.zxp` next to the script or in `..\dist\`)
   - **macOS**: Run `chmod +x install.sh && ./install.sh` (requires `*-darwin.zxp`, next to the script or in `../dist/`)
   - **Linux CEP test host**: Install the `*-linux.zxp` with your CEP extension manager; the macOS/Windows helper scripts intentionally refuse cross-platform ZXP installs.
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
5. Click **Export Selected Artboards**
6. Copy the generated code or save to your project

If project-file analysis is unavailable, the panel still exports from the Illustrator DOM and shows an `ai-parser` diagnostic in the status/warnings area instead of silently ignoring the missing parser.

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

Each artboard generates a `draw_<name>(ui: &mut Ui, state: &mut <Name>State) -> Option<<Name>Action>` function. Add it to your project and call it from your egui update loop.
