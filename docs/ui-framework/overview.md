# egui_expressive UI Framework Overview

`egui_expressive` is being shaped into a full Rust UI framework that feels familiar to Tailwind/CSS/HTML designers while staying native to egui.

## Mental Model for Web Designers

| Web concept | egui_expressive concept |
|---|---|
| Tailwind utilities | `Tw` builder (`p`, `mx`, `rounded_lg`, `shadow`) |
| CSS variables / design tokens | `DesignTokens`, `Theme`, `SemanticColors` |
| Responsive breakpoints | `Breakpoints`, `BreakpointName`, `Responsive<T>` |
| Flex layouts | `FlexContainer`, `vstack!`, `hstack!` |
| CSS grid | `GridLayout`, `GridSpan`, `Tw::grid_cols`, `Tw::col_span` |
| CSS position/inset/translate/z-index | `PositionStyle`, `PositionMode`, `Tw::absolute`, `Tw::inset`, `Tw::translate_x`, `Tw::z` |
| Forms | `TextField`, `TextAreaField`, `SelectField`, `CheckboxField`, `SwitchField`, `ValidationMessage` |
| Canvas/editor surfaces | `EditorCanvas`, `SnapGrid`, `Axis`, `CanvasItem` |
| Component state variants | `TwVariants`, `TwVariant`, `VisualState`, `VisualVariant` |
| CSS transitions | `Transition`, `Tween`, `Spring` |
| Accessibility metadata | `AccessibilityMeta`, `AccessibilityRole`, `FocusRing`, `MotionPolicy` |

## Practical Parity Targets

The framework targets the UI patterns designers use most:

- Box model: margin, padding, border, radius, size, shadow.
- Utility coverage: grid, position, directional borders, overflow, cursor, slash-alpha colors, percentage/viewport sizing.
- Breakpoints: `sm`, `md`, `lg`, `xl`, `2xl`.
- States: hover, pressed, focused, selected, disabled.
- Dark/light themes and design tokens.
- Reduced-motion-aware animations.
- Accessibility metadata and visible focus affordances for custom widgets.
- Forms, dashboards, editors, pro-app panels, command palettes, and custom controls.

Explicit non-goals for the core framework:

- Full CSS selector syntax such as `:nth-child`.
- Browser cascade/inheritance semantics.
- Full media-query grammar.
- Arbitrary CSS `@keyframes` syntax.
- App-specific widgets hardcoded into core modules.

## Neutraudio Mockup Capability Matrix

| Mockup region | Generic primitives |
|---|---|
| Top bar / transport | `ToolbarStrip`, `ToolButton`, `TransportButton`, `Tw`, `FlexContainer` |
| Browser/sidebar | `TreeView`, `SearchField`, `TabBar`, `ResizableSplit`, `Tw` |
| Playlist / arrangement | `EditorCanvas`, `Axis`, `SnapGrid`, `CanvasItem`, `SelectionModel`, `Ruler` |
| Piano roll | `EditorCanvas`, `Axis::indexed`, `CanvasItem`, `MarqueeSelection`, `SnapGrid` |
| Mixer | `ChannelStrip`, `Fader`, `Meter`, `Knob`, `ResizableSplit` |
| Channel rack | `StepGrid`, `StepCellGrid`, `DragReorder`, `ControlGroup` |
| Devices / FX rack | `ControlGroup`, `Knob`, `DragReorder`, `FloatingPanel` |
| Command palette | `CommandPalette`, `ActionRegistry`, `ShortcutRegistry` |
| Modals/preferences | `ModalOverlay`, `TextField`, `SelectField`, `CheckboxField`, `SwitchField`, `Tw` |

The Neutraudio UI should be implemented as app code composed from these primitives, not as core-library DAW screens.

## Current Known Gaps

The framework now represents the main layout utilities used by the mockup: grid columns/spans, absolute/relative/fixed positioning, inset/translate/z-index values, directional borders, cursor intent, overflow intent, slash-alpha colors, `%`/`vw` sizing, native form wrappers, dark/theme token shortcuts, responsive `Tw` variants, editor lane-stack/value-lane submodules, and a compiled `neutraudio_shell` proof example. Remaining broad-approval work is continued migration out of the legacy widget monolith and deeper feature parity such as gradients/filter/backdrop utilities.
