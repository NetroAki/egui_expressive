# Tokens, Typography, Icons, and Visual Styling

the current release candidate centralizes the visual vocabulary for `egui_expressive`: design tokens,
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
  the Tailwind-style weight scale. fidelity item propagates that numeric
  weight-intent into `TypeSpec`; egui-native `RichText` and
  `TypeSpec::to_rich_text` still collapse weights to bounded weak / normal /
  strong text emphasis unless a widget consumes the recorded weight more
  precisely.
- Phase 6 adds `Tw::to_type_spec` for the exact-capable ASCII/default-font subset:
  utility size, tracking, foreground color, and fidelity item numeric weight intent
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
- the current release candidate adds opt-in runtime/manual registry helpers: `FontRegistry`,
  `FontFaceRecord`, `FontFamilyRecord`, and `TypeSpec::resolve_font` select an
  app-provided face deterministically and return a `FontSelectionReport`. Exact
  reports mean registry metadata selected a registered face with approved license
  ID membership, non-empty provenance identity/ledger metadata, and app-provided bytes; they do not prove egui raster output,
  browser layout, shaping completeness, or codegen/plugin font emission.
- fidelity item also adds an M3 `M3TextStyle::to_type_spec()` bridge so Material 3
  Regular, Medium, and Bold values are preserved as numeric `TypeSpec.weight`
  intent. the current release candidate adds `M3TextStyle::resolve_font` for app-provided registry
  selection; `M3TextStyle::to_font_id()` remains size/family-only and does not
  select weight-specific font faces.
- `src/typography/*` owns richer `TypeSpec`, `TypeScale`, text block, shaping,
  transform, overflow, and decoration concepts. Use those lower-level primitives
  when CSS-like text behavior exceeds the `Tw` contract.
- `src/m3/typography.rs` provides Material 3 type scale names for M3-style apps.

## Font Registration Guidance

This stage does not add a native font installer, OS font enumerator, network font
fetcher, or bundled font loader. Applications still own font files, license
review, and egui font registration. Use `FontRegistry` to describe app-provided
faces and approved license IDs, then call `TypeSpec::resolve_font`,
`Tw::resolve_font`, or `M3TextStyle::resolve_font` when the app needs a typed
selection report for size/weight/fallback decisions.

Roadmap follow-up: the dense-tooling gap analysis in
`Tests/egui_expressive_gap_analysis.md` adds planned ownership for optional
library-side symbol fallback helpers, icon-font registration helpers by family,
and broader missing-glyph diagnostics. the current release candidate narrows that gap with explicit
registry reports, but app code still owns concrete font assets and registration.

## Icon Guidance

- `src/icons/mod.rs` exposes Material and Phosphor icon families plus `Icon` and
  `IconButton` widgets.
- App code must ensure the corresponding icon font is registered with egui before
  expecting glyph parity.
- Accessibility labels, keyboard semantics, and screen-reader guidance remain a
  the current release candidate responsibility; the current release candidate only documents visual packaging and theme use.

Planned follow-up: the current release candidate roadmap work now explicitly tracks icon/font
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

Material 3 components remain Beta. the current release candidate documents their token relationship but
does not stabilize every component family or complete release proof; the current release candidate owns
release readiness and broad regression coverage. Phase 6 adds a narrow exact
fixture for `M3TopAppBar` sizing, centered-title alignment, and scrolled-state
surface proof; that row does not promote the full M3 family to Stable.
Phase 7 adds a second narrow exact fixture for fixed button/card token surfaces;
it does not stabilize the whole M3 family or its interaction/accessibility surface.
Phase 8 adds exact endpoint fixtures for input controls, text fields, navigation,
and list items, but does not promote animated states, indeterminate progress,
dialogs/snackbars, or accessibility breadth to Stable.
