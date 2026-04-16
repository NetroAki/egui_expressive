# egui_expressive

An authoring-layer helper library on top of egui 0.31. If you've found yourself rewriting the same widgets, state machines, and easing curves across every egui project, this is for you.

## Installation

```toml
[dependencies]
egui_expressive = "0.1"
```

## What's inside

The crate is organized into focused modules. Not everything may work on your target (some parts need a GPU for blur), but the split makes it easy to import only what you need.

**`draw/`** — LayeredPainter, ShapeBuilder, box shadows, gradients, and a small icon library. Builds up complex shapes from simpler primitives without fighting egui's paint API.

**`style/`** — VisualState<T>, DesignTokens, SurfacePalette, AccentColors, Lerp, TextStyles. Centralizes the gap between "design tokens from Figma" and "what gets painted".

**`state/`** — StateSlot<T>, StateMachine<S>, InteractionState. Three different takes on UI state that show up constantly: a single value slot, a proper state machine, and interaction tracking.

**`interaction/`** — DragDelta, PanZoom, drag_to_value_delta. Mouse and touch handling that goes beyond egui's built-in input.

**`animation/`** — Easing (10+ variants), Tween, Spring, AnimSequence, Transition. Tween between values with easing, drive springs for physics-based feel, sequence animations together.

**`surface/`** — LargeCanvas, ViewportCuller. For virtual canvases larger than 50k pixels. Culls what's off-screen so egui doesn't choke.

**`widgets/`** — Knob (4 styles), Fader (stereo meter), Meter, StepGrid, ChannelStrip, TimelineClip, Ruler, Waveform, ResizableSplit, TabBar, TreeView, FloatingPanel, ContextMenuBuilder, ToggleDot, TransportButton. Audio-plugin-style widgets you'd normally pull from a separate crate.

**`debug/`** — DebugOverlay, debug_label, debug_interaction. Always-on debug visualization for layout and interaction without scattering print statements.

**`blur/`** — Gaussian soft shadows and software blur. Not GPU-accelerated, but useful for small radii or offscreen buffers.

**`devtools/`** — Runtime visual property editor. No-ops in release builds, wired up in debug so you can tweak design tokens live.

**`figma/`** — Figma Tokens JSON → Rust DesignTokens CLI exporter. Point it at your tokens JSON, get generated Rust code.

**`layout/`** — vstack!, hstack!, zstack! macros. The three layout primitives that come up every time egui's layouts feel too verbose.

**`m3/`** — Full Material Design 3. 21 components: Button, Card, Switch, Checkbox, Radio, Chip, Progress, Badge, Slider, TextField, NavigationBar, NavigationRail, TopAppBar, ListItem, Dialog, Snackbar, FAB, DropdownMenu, Divider, Tooltip, CircularProgress. Not a design system you have to commit to — just components if MD3 is your thing.

**`tailwind/`** — Tw style builder with Tailwind utility methods and SwiftUI aliases. Build styles chain-style instead of struct-style.

**`swiftui/`** — Navigator, ScrollList, GeometryProxy, ViewModifier. SwiftUI-inspired APIs for egui.

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

MIT or Apache-2.0, your choice.
