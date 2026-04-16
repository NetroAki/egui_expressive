# egui_expressive Illustrator Exporter

A UXP plugin for Adobe Illustrator 2021+ that exports artboards as self-contained Rust code using `egui_expressive`.

## Installation

1. Open Illustrator
2. Go to **Plugins → Development → Load Plugin**
3. Select this folder (containing `manifest.json`)

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
