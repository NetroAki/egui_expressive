# Tokens, Typography, Icons, and Visual Styling

Stage 7 centralizes the visual vocabulary for `egui_expressive`: design tokens,
Tailwind-style utilities, typography scales, icons, Material 3 families, and
bounded effect recipes.

## Token Sources

| Source | Purpose | Use when |
| --- | --- | --- |
| `src/style/tokens.rs` / `DesignTokens` | Product-level spacing, surfaces, accents, widget-state colors | Building custom widgets or design-system primitives. |
| `src/theme/mod.rs` / `Theme`, `SemanticColors`, `Elevation` | Runtime app theme, semantic colors, elevation shadows | Styling app surfaces and loading dark/light variants. |
| `src/tailwind/theme_tokens.rs` / `ColorToken` | Tailwind-style token references inside `Tw` | Authoring `Tw` styles that follow the active theme. |
| `src/m3/*` | Material 3 color, elevation, typography, and component family tokens | Building M3-inspired UI while preserving egui-native rendering. |

Figma REST style exports do not include actual token color values in the styles
metadata response. `src/figma/codegen.rs` therefore emits explicit placeholder
colors and replacement comments for that input shape; treat those generated
values as handoff scaffolding, not design-token parity, unless a Figma Tokens JSON
export supplies concrete values.

## Tailwind-Style Recipes

```rust
use egui_expressive::{AccentKind, SurfaceLevel, Tw};

Tw::new()
    .p(16.0)
    .rounded_lg()
    .bg_surface(SurfaceLevel::Surface2)
    .text_accent(AccentKind::Primary)
    .shadow(egui_expressive::Elevation::Level2);
```

Use `docs/ui-framework/tw-render-contract.md` as the source of truth for which
utilities are rendered, bounded approximations, or unsupported.

## Typography Guidance

- `Tw::text_xs` through `Tw::text_3xl` are convenient utility sizes.
- `Tw::font_thin` through `font_black`, plus `font_weight(100..900)`, cover
  the Tailwind-style weight scale. R100-005A propagates that numeric
  weight-intent into `TypeSpec`; egui-native `RichText` and
  `TypeSpec::to_rich_text` still collapse weights to bounded weak / normal /
  strong text emphasis unless a widget consumes the recorded weight more
  precisely.
- Phase 6 adds `Tw::to_type_spec` for the exact-capable ASCII/default-font subset:
  utility size, tracking, foreground color, and R100-005A numeric weight intent
  can be converted into `TypeSpec` and rendered through `render_text_block`
  without claiming bundled font parity, weight-specific font selection, or full
  browser text layout.
- Phase 7 adds exact proof for ASCII/default-font decoration and overflow only:
  underline, strikethrough, tracking, foreground color, and clip/ellipsis can be
  validated for fixed fixtures, while font weight and browser text layout remain bounded.
- Phase 8 adds exact proof for built-in family aliases only: `font_mono`,
  `font_sans`, and `TypeSpec::font_family("mono" | "monospace" | "sans" |
  "proportional")` map to egui's built-in `Monospace` or `Proportional` family
  for fixed ASCII fixtures. Custom names still require app-registered fonts;
  weight-specific font selection remains bounded.
- R100-005A also adds an M3 `M3TextStyle::to_type_spec()` bridge so Material 3
  Regular, Medium, and Bold values are preserved as numeric `TypeSpec.weight`
  intent. `M3TextStyle::to_font_id()` remains size/family-only and does not
  select weight-specific font faces.
- `src/typography/*` owns richer `TypeSpec`, `TypeScale`, text block, shaping,
  transform, overflow, and decoration concepts. Use those lower-level primitives
  when CSS-like text behavior exceeds the `Tw` contract.
- `src/m3/typography.rs` provides Material 3 type scale names for M3-style apps.

## Font Registration Guidance

This stage does not add a native font installer or bundled font loader. Register
fonts through egui's existing font APIs in your app setup, then use `Tw`,
`TextStyles`, `TypeSpec`, or M3 typography tokens to select size/weight/tracking.
Keep font files and licenses in the application, not hidden inside the core library.

Roadmap follow-up: the dense-tooling gap analysis in
`Tests/egui_expressive_gap_analysis.md` adds planned ownership for optional
library-side symbol fallback helpers, icon-font registration helpers by family,
and missing-glyph diagnostics. Until that lands, app code still owns concrete
font registration.

## Icon Guidance

- `src/icons/mod.rs` exposes Material and Phosphor icon families plus `Icon` and
  `IconButton` widgets.
- App code must ensure the corresponding icon font is registered with egui before
  expecting glyph parity.
- Accessibility labels, keyboard semantics, and screen-reader guidance remain a
  Stage 8 responsibility; Stage 7 only documents visual packaging and theme use.

Planned follow-up: Stage 7/8 roadmap work now explicitly tracks icon/font
fallback infrastructure so apps can opt into library-owned helpers instead of
repeating the same registration boilerplate.

## Visual Effects Guidance

- Prefer `Tw::shadow(Elevation)` for design-system elevation.
- Use `Tw::drop_shadow`, `bg_gradient`, `ring`, and `backdrop_blur` only within the
  bounded behavior documented in `tw-render-contract.md`.
- For richer vector gradients or image operations, use `src/draw/*` and
  `src/blur/*` primitives directly; `Tw` intentionally remains a small utility DSL.

Planned follow-up: the roadmap now explicitly tracks higher-level generic visual
recipes such as glass/glow/inner-stroke/state-layer/accent-pulse helpers when
they can stay composable and domain-neutral.

## Material 3 Visual Families

Material 3 components remain Beta. Stage 7 documents their token relationship but
does not stabilize every component family or complete release proof; Stage 9 owns
release readiness and broad regression coverage. Phase 6 adds a narrow exact
fixture for `M3TopAppBar` sizing, centered-title alignment, and scrolled-state
surface proof; that row does not promote the full M3 family to Stable.
Phase 7 adds a second narrow exact fixture for fixed button/card token surfaces;
it does not stabilize the whole M3 family or its interaction/accessibility surface.
Phase 8 adds exact endpoint fixtures for input controls, text fields, navigation,
and list items, but does not promote animated states, indeterminate progress,
dialogs/snackbars, or accessibility breadth to Stable.
