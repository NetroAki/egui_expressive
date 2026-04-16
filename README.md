# egui_expressive

**A batteries-included extension crate for [egui](https://github.com/emilk/egui) 0.34** — design tokens, Material Design 3 widgets, animation primitives, blur effects, DAW-style controls, layout macros, and more.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![egui: 0.34](https://img.shields.io/badge/egui-0.34-orange.svg)](https://github.com/emilk/egui)

## Installation

```toml
[dependencies]
egui_expressive = { git = "https://github.com/NetroAki/egui_expressive" }
egui = "0.34"

[dev-dependencies]
eframe = "0.34"
```

## Modules

| Module | Description |
|--------|-------------|
| `animation` | `Tween`, `Spring`, `Transition` — frame-rate-independent animation with 10+ easing curves |
| `blur` | Soft shadows, glow, inner shadows, and CPU-side image blur approximations |
| `debug` | Debug overlays, `debug_label`, `debug_interaction` (enabled by default via `debug` feature) |
| `devtools` | Live-tweakable `Prop` system with `DevToolsPanel` inspector — no-ops in release builds |
| `draw` | `ShapeBuilder`, `LayeredPainter`, gradients (linear + radial), box shadows, icons, scan-lines, vignette |
| `figma` | Figma design-token import and `figma-export` CLI binary |
| `interaction` | `DragDelta`, `DragAxis`, `PanZoom` — pointer and gesture helpers |
| `layout` | `vstack!`/`hstack!`/`zstack!` macros, `auto_layout`, `styled_frame`, dividers |
| `m3` | Full Material Design 3 component set — buttons, cards, navigation, dialogs, FABs, and more |
| `state` | `StateSlot<T>`, `StateMachine<S>`, `InteractionState` |
| `style` | `DesignTokens`, `SurfacePalette`, `AccentColors`, `TextStyles`, `VisualState<T>`, theming utilities |
| `surface` | `LargeCanvas` and `ViewportCuller` for virtual canvases larger than 50k px |
| `swiftui` | SwiftUI-inspired `ViewModifier`, `GeometryProxy`, `Navigator`, `ScrollList` |
| `tailwind` | `Tw` style builder with Tailwind-like spacing, sizing, and layout DSL |
| `widgets` | DAW controls (Knob, Fader, Meter, StepGrid, DragNumber), layout widgets (ResizableSplit, TabBar, TreeView, CollapsePanel, DragReorder), timeline (TimelineClip, Ruler, Waveform), and more |

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `debug` | ✅ | Enables `DebugOverlay`, `debug_label`, `debug_interaction`, and `DevToolsPanel` |

## Quick Example

```rust
use egui_expressive::{Knob, KnobStyle, DragNumber, vstack};

struct MyApp {
    gain: f64,
    bpm: f64,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            vstack!(ui, gap: 12.0, {
                ui.add(
                    Knob::new(&mut self.gain, 0.0..=1.0)
                        .style(KnobStyle::Default)
                        .label("GAIN"),
                );
                ui.add(
                    DragNumber::new(&mut self.bpm, 60.0..=300.0)
                        .label("BPM")
                        .default_value(120.0)
                        .decimals(1),
                );
            });
        });
    }
}
```

## Material Design 3

```rust
use egui_expressive::{M3Button, M3Theme};

// Apply theme once at startup
M3Theme::dark().store(ctx);

// Use components anywhere
ui.add(M3Button::new("Save").tonal());
ui.add(M3Button::new("Cancel").outlined());
```

The `m3` module covers buttons, cards, switches, checkboxes, chips, progress indicators, sliders, text fields, navigation bars/rails, top app bars, list items, dialogs, snackbars, FABs, and dropdown menus.

## Figma Integration

Export design tokens from Figma using the included plugin (`figma-plugin/`) or the CLI:

```bash
cargo run --bin figma-export -- tokens.json > src/design_tokens.rs
```

## Examples

```bash
cargo run --example daw_strip       # DAW channel strip: Knob + Fader + Meter + StepGrid
cargo run --example step_sequencer  # BPM-driven step sequencer
cargo run --example timeline        # Large timeline canvas with viewport culling
```

## Documentation

📖 **[Wiki](../../wiki)** — full guides for every module:

[Getting Started](../../wiki/Getting-Started) · [Animation](../../wiki/Animation) · [Blur Effects](../../wiki/Blur-Effects) · [Drawing & Shapes](../../wiki/Drawing-and-Shapes) · [Interaction](../../wiki/Interaction) · [Layout Macros](../../wiki/Layout-Macros) · [Material Design 3](../../wiki/Material-Design-3) · [State Management](../../wiki/State-Management) · [Style & Theming](../../wiki/Style-and-Theming) · [SwiftUI Patterns](../../wiki/SwiftUI-Patterns) · [Large Surfaces](../../wiki/Large-Surfaces) · [Widgets](../../wiki/Widgets) · [Debug & DevTools](../../wiki/Debug-and-DevTools) · [Figma Integration](../../wiki/Figma-Integration) · [Cookbook](../../wiki/Cookbook)

## License

MIT
