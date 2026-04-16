# egui_expressive

An authoring-layer helper library on top of egui 0.31. If you've found yourself rewriting the same widgets, state machines, and easing curves across every egui project, this is for you.

## Installation

```toml
[dependencies]
egui_expressive = "0.1"
```

## What's inside

The crate is organized into focused modules. Not everything may work on your target (some parts need a GPU for blur), but the split makes it easy to import only what you need.

| Module | What it gives you |
|--------|-------------------|
| `draw` | LayeredPainter, ShapeBuilder, box shadows, gradients, icon helpers |
| `style` | VisualState\<T\>, DesignTokens, SurfacePalette, AccentColors, Lerp, TextStyles |
| `state` | StateSlot\<T\>, StateMachine\<S\>, InteractionState |
| `interaction` | DragDelta, PanZoom, drag_to_value_delta |
| `animation` | Easing (10+ variants), Tween, Spring, AnimSequence, Transition |
| `surface` | LargeCanvas, ViewportCuller — for virtual canvases larger than 50k px |
| `widgets` | Knob, Fader, Meter, StepGrid, ChannelStrip, TimelineClip, Ruler, Waveform, ResizableSplit, TabBar, TreeView, FloatingPanel, ContextMenuBuilder, ToggleDot, TransportButton |
| `debug` | DebugOverlay, debug_label, debug_interaction |
| `blur` | Gaussian soft shadows and software blur (CPU-side) |
| `devtools` | Runtime visual property editor, no-ops in release builds |
| `figma` | Figma Tokens JSON to Rust DesignTokens CLI exporter |
| `layout` | vstack!, hstack!, zstack! macros |
| `m3` | Full Material Design 3 — 21 components |
| `tailwind` | Tw style builder with Tailwind utility methods and SwiftUI aliases |
| `swiftui` | Navigator, ScrollList, GeometryProxy, ViewModifier |

## Example

```rust
use egui_expressive::{vstack, Knob, KnobStyle, DesignTokens};

// Store gain somewhere in your app state.
let mut gain: f64 = 0.75;

vstack!(ui, gap: 12.0, {
    ui.add(
        Knob::new(&mut gain, 0.0..=1.0)
            .style(KnobStyle::Default)
            .label("Gain"),
    );
    ui.label(format!("{:.2}", gain));
});
```

## Figma Token Exporter

```bash
cargo run --bin figma-export -- tokens.json
```

Feed it the JSON export from the Figma Tokens plugin. It prints Rust code with your colors, spacing, and rounding values — pipe it wherever you want.

## M3 Components

The m3 module covers most of MD3. Here's a button:

```rust
use egui_expressive::{M3Button, M3Theme};

// Apply the theme once at startup.
M3Theme::dark().store(ctx);

// Then use components anywhere.
ui.add(M3Button::new("Save").tonal());
```

The components don't force you into the full MD3 system. Use one, use five, ignore the rest.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option. Both licenses permit commercial use.
