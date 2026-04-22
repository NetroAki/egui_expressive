# Generated Files Directory

Copy your exported `.rs` files from Illustrator here before running `cargo run`.

## Required files

The exporter generates these files per artboard:

- `mod.rs` — module declarations
- `tokens.rs` — color constants
- `state.rs` — state structs + action enums
- `components.rs` — reusable component functions
- `<artboard_name>.rs` — one per artboard (e.g. `login_screen.rs`)

## Workflow

1. Export from the Illustrator panel using **Save to Folder**
2. Copy all `.rs` files into this `generated/` directory (overwriting placeholders)
3. Run `cargo run` from the `preview/` directory
4. Select an artboard from the sidebar to see it rendered

## Example

```bash
cd preview
cp ~/Downloads/exported/*.rs generated/
cargo run
```
