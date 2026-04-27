# egui_expressive

**Expressive, beautiful UI for [egui](https://github.com/emilk/egui) 0.34 — without giving up immediate-mode simplicity or performance.** Design tokens, Material Design 3 widgets, animation primitives, blur and glow effects, blend modes, DAW-style controls, design-tool code generation, layout macros, and more.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![egui: 0.34](https://img.shields.io/badge/egui-0.34-orange.svg)](https://github.com/emilk/egui)

## Why `egui_expressive`?

`egui` is fast, portable, immediate-mode, and deliberately minimal. That is its strength—but building visually rich, production-polished interfaces directly from raw `Painter` calls can become repetitive: layered effects, state-driven styling, gradients, shadows, masks, complex strokes, large canvases, design tokens, and design-tool imports all require boilerplate.

`egui_expressive` is the expressive design layer on top of egui. The goal is not to replace egui’s renderer, layout model, or interaction system. The goal is to make egui capable of highly designed interfaces while preserving the benefits that make egui valuable:

- **Immediate-mode ergonomics** — APIs compose with `egui::Ui`, `egui::Painter`, `egui::Shape`, and `egui::Response`.
- **Performance-first rendering** — use CPU/epaint meshes for cheap vector primitives, viewport culling for huge canvases, and GPU/WGPU-backed callbacks only when an effect genuinely needs shader or render-target support.
- **Beautiful by default, escape hatches always available** — higher-level builders terminate into egui-native concepts, and you can drop down to raw egui at any point.
- **Code output over screenshots** — design-tool exports aim to generate editable Rust primitives and effect code. Current exports cover an incremental subset of features (vector/primitive export target). Limitations include embedded rasters, unsupported live effects, and complex blend/mask cases.

In short: **egui is the floor, not the ceiling.** This crate exists so applications can stay immediate-mode and lightweight while still looking expressive, polished, and aiming for design-tool fidelity.

## Architecture

```text
┌─────────────────────────────────────────────┐
│              Your Application                │
├─────────────────────────────────────────────┤
│           egui_expressive                    │
│  ┌─────┐ ┌─────┐ ┌───────┐ ┌─────────────┐ │
│  │draw │ │style│ │state  │ │interaction  │ │
│  └──┬──┘ └──┬──┘ └───┬───┘ └──────┬──────┘ │
│  ┌──┴──┐ ┌──┴──────┐ │  ┌─────────┴──────┐ │
│  │surf.│ │animation│ │  │   widgets      │ │
│  └─────┘ └─────────┘ │  └────────────────┘ │
│                    ┌──┴──┐                   │
│                    │tools│                   │
│                    └─────┘                   │
├─────────────────────────────────────────────┤
│          egui (Painter, Ui, Response)        │
├─────────────────────────────────────────────┤
│          epaint / egui-wgpu where needed     │
└─────────────────────────────────────────────┘
```

Most primitives render through egui’s existing `epaint` mesh and shape pipeline. More demanding Illustrator-style effects—true blurs, advanced blend/composite modes, stencil-like masks, gradient meshes, warps, or 3D-style effects—can be implemented as opt-in GPU-backed primitives using egui’s custom paint callback path while retaining immediate-mode call sites.

Core philosophy:

1. **egui is the floor, not the ceiling** — every API composes with raw egui.
2. **Low overhead when unused** — features should be gated or pay-as-you-call.
3. **State machines over boolean soup** — richer controls use explicit state models.
4. **Builders terminate into egui types** — no hidden retained UI framework.
5. **Culling is first-class** — timelines, node graphs, and large design surfaces should stay fast.

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
| `codegen` | SVG layout inference and Rust scaffold code generation. Naming convention parser (`row-*`, `col-*`, `btn-*`, etc.), gap inference, layout tree builder, multi-file artboard output |
| `debug` | Debug overlays, `debug_label`, `debug_interaction` (enabled by default via `debug` feature) |
| `devtools` | Live-tweakable `Prop` system with `DevToolsPanel` inspector (panel is no-op outside debug_assertions) |
| `draw` | `ShapeBuilder`, `LayeredPainter`, gradients (linear + radial), box shadows, icons, scan-lines, vignette, **2D affine transforms, ZStack** |
| `figma` | Figma design-token import and `figma-export` CLI binary |
| `icons` | Icon font rendering with `Icon`, `IconButton`, `IconSize` types and built-in icon constants |
| `interaction` | `DragDelta`, `DragAxis`, `PanZoom` — pointer and gesture helpers |
| `layout` | `vstack!`/`hstack!`/`zstack!` macros, `auto_layout`, `styled_frame`, dividers |
| `m3` | Full Material Design 3 component set — buttons, cards, navigation, dialogs, FABs, and more |
| `state` | `StateSlot<T>`, `StateMachine<S>`, `InteractionState` |
| `style` | `DesignTokens`, `SurfacePalette`, `AccentColors`, `TextStyles`, `VisualState<T>`, theming utilities |
| `surface` | `LargeCanvas` and `ViewportCuller` for virtual canvases larger than 50k px |
| `svg` | SVG path parsing (`svg_path_to_shape`, `svg_to_shapes`), CSS/SVG color parser, Adobe Swatch Exchange (ASE) binary parser |
| `swiftui` | SwiftUI-inspired `ViewModifier`, `GeometryProxy`, `Navigator`, `ScrollList` |
| `tailwind` | `Tw` style builder with Tailwind-like spacing, sizing, and layout DSL |
| `theme` | `Border`, `Elevation`, `SemanticColors`, `Theme` — light/dark theme management, border utilities, semantic color tokens |
| `typography` | Rich text rendering with `TypeScale`, `TypeSpec`, text overflow/decoration/transform support |
| `widgets` | DAW controls (Knob, Fader, Meter, StepGrid, DragNumber), layout widgets (ResizableSplit, TabBar, TreeView, CollapsePanel, DragReorder), timeline (TimelineClip, Ruler, Waveform), and more |
| `daw` | *(feature-gated)* — Convenience re-export module for DAW-oriented widgets and utilities (gated behind `daw` feature) |
| `gpu` | *(feature-gated)* — GPU-accelerated effects pipeline using `wgpu` (gated behind `wgpu` feature) |
| `scene` | Retained-mode scene graph for complex vector rendering and effect compositing |

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `debug` | [yes] | Enables DebugOverlay (methods become no-ops when off), debug_label, debug_interaction (removed from exports when off) |
| `daw` | [no] | Enables the daw convenience re-export module (widgets are always available in the widgets module) |
| `clip-mask` | [no] | Enables `clipped_shape_cpu` for CPU-side polygon mask overlay approximation (background-dependent; requires `tiny-skia`) |
| `wgpu` | [no] | Enables the `gpu` module and `wgpu` dependency for hardware-accelerated effects |
| `gpu-effects` | [no] | Enables advanced GPU effects (currently an alias for `wgpu`) |

## Quick Example

```rust
use egui_expressive::{Knob, KnobStyle, DragNumber, vstack};

struct MyApp {
    gain: f64,
    bpm: f64,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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

## Adobe Illustrator Parser

Parse `.ai` files (PDF-wrapped Illustrator files) and extract visual properties:

```bash
# Output JSON with all elements, effects, transforms, and path geometry
cargo run --bin ai-parser -- design.ai --pretty

# Generate per-artboard Rust scaffold code
cargo run --bin ai-parser -- design.ai --per-artboard
```

The parser currently extracts an incremental subset of features: artboards, AIPrivateData streams, CTM transforms, Bezier path geometry, corner radius detection, and fill/stroke appearance. Limitations include embedded rasters, unsupported live effects, and complex blend/mask cases.

## SVG Parsing

```rust
use egui_expressive::svg::{svg_path_to_shape, svg_to_shapes, parse_svg_color};

// Parse an SVG path `d` attribute into an egui Shape
let shape = svg_path_to_shape("M10 10 L90 90 Z", egui::Color32::WHITE, egui::Stroke::new(1.0, egui::Color32::BLACK));

// Parse a full SVG string with multiple <path> elements
let shapes: Vec<(egui::Shape, egui::Rect)> = svg_to_shapes(svg_string);

// Parse CSS/SVG color strings
let color = parse_svg_color("#ff8040"); // Also supports rgb(), rgba(), named colors

// Parse Adobe Swatch Exchange (.ase) files
let colors = egui_expressive::svg::parse_ase(&ase_bytes)?;
```

Supports all SVG path commands: M, L, H, V, C, S, Q, T, A, Z (absolute and relative).

## Code Generation

Convert SVG exports from design tools into egui layout code:

```rust
use egui_expressive::codegen::{infer_layout, generate_rust, InferenceOptions};

// Parse SVG elements and infer layout structure
let nodes = infer_layout(&elements, &InferenceOptions::default());

// Generate Rust code
let code = generate_rust("my_screen", 375.0, 812.0, &nodes, Some(bg_color), None, None);
```

Supports naming convention hints (`row-*`, `col-*`, `btn-*`, `card-*`, `panel-*`, `scroll-*`, `badge-*`, etc.), automatic gap inference, and multi-file artboard output.

## Examples

```bash
cargo run --example daw_strip       # DAW channel strip: Knob + Fader + Meter + StepGrid
cargo run --example step_sequencer  # BPM-driven step sequencer
cargo run --example timeline        # Large timeline canvas with viewport culling
```

## Documentation

[docs] **[Wiki](../../wiki)** — full guides for every module:

[Getting Started](../../wiki/Getting-Started) · [Animation](../../wiki/Animation) · [Blur Effects](../../wiki/Blur-Effects) · [Drawing & Shapes](../../wiki/Drawing-and-Shapes) · [Interaction](../../wiki/Interaction) · [Layout Macros](../../wiki/Layout-Macros) · [Material Design 3](../../wiki/Material-Design-3) · [State Management](../../wiki/State-Management) · [Style & Theming](../../wiki/Style-and-Theming) · [SwiftUI Patterns](../../wiki/SwiftUI-Patterns) · [Large Surfaces](../../wiki/Large-Surfaces) · [Widgets](../../wiki/Widgets) · [Debug & DevTools](../../wiki/Debug-and-DevTools) · [Figma Integration](../../wiki/Figma-Integration) · [Cookbook](../../wiki/Cookbook)

## License

MIT
