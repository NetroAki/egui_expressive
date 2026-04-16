# egui_expressive Token Exporter — Figma Plugin

Exports Figma paint styles, text styles, effect styles, and variables as JSON compatible with the `figma_tokens_to_rust` CLI tool.

## Installation

1. In Figma, go to **Plugins → Development → Import plugin from manifest...**
2. Select `manifest.json` from this directory.

## Usage

1. Open the plugin from **Plugins → egui_expressive Token Exporter**
2. Click **Re-export** to refresh
3. Click **Copy JSON** or **Download tokens.json**
4. Run the converter:

```bash
cargo run --bin figma-export -- tokens.json > src/design_tokens.rs
```

5. Use in your app:

```rust
mod design_tokens;
design_tokens::design_tokens().store(ctx);
```

## What gets exported

| Figma | JSON type | Rust |
|-------|-----------|------|
| Paint styles (solid colors) | `color` | `SurfacePalette` / `AccentColors` |
| Text styles | `typography` | (informational) |
| Effect styles (shadows) | `boxShadow` | (informational) |
| Float variables | `spacing` | `SpacingScale` |
| Color variables | `color` | `SurfacePalette` / `AccentColors` |

## License

MIT
