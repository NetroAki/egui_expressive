# egui_expressive Preview Runner

A GUI app that lets you preview exported Illustrator artboards as live egui UIs.

## Quick Start

1. **Export from Illustrator**
   - Open the egui_expressive panel in Illustrator
   - Select artboards and click **Save to Folder**
   - Choose a destination folder

2. **Run the preview**
   ```bash
   cd preview
   cargo run
   ```

3. **Select your export folder**
   - Click **Select Export Folder** in the launcher
   - Pick the folder where you saved the `.rs` files
   - The app copies files, rebuilds, and launches the preview automatically

4. **View your artboards**
   - Select an artboard from the left sidebar
   - See it rendered live in the main area

## Workflow

```
Illustrator → Save to Folder → Select in Preview → See rendered UI
```

To load a different export, click **Load Different Folder** in the top bar.

## Files

The preview app looks for these files in the selected folder:

- `<artboard_name>.rs` — one per artboard (e.g. `login_screen.rs`)

If `mod.rs`, `tokens.rs`, `state.rs`, or `components.rs` are missing, the app auto-generates minimal placeholders so the preview still compiles. For the best experience, export all files from Illustrator.

## How it works

The preview uses a `build.rs` script that:
1. Copies `.rs` files from your selected folder into `generated/`
2. Auto-generates module declarations for artboard files
3. Generates dispatch code to call each artboard's `draw_*` function
4. The eframe app renders them in a scrollable viewer with a sidebar picker

## Requirements

- Rust + Cargo
- The `egui_expressive` library (sibling directory)

## Troubleshooting

**"No artboards loaded"**
→ Make sure the folder contains `.rs` files exported from Illustrator.

**Build errors after selecting folder**
→ The exported code may have issues. Check the terminal output for compilation errors.

**Artboard not showing**
→ Make sure the artboard `.rs` file is in the folder and `mod.rs` declares it.
