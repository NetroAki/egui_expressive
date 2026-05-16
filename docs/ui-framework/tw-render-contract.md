# Tw Render Contract

`Tw` methods must either affect rendered egui output, record style intent that a
renderer consumes, or be explicitly listed here as bounded/approximate/unsupported.
This file is the Stage 7 support matrix for DEBT-009.

## Support Levels

- **Rendered** — visible egui output changes in `Tw::show` or `Tw::to_frame`.
- **Recorded intent** — values are stored and consumed by a resolver, widget, or
  post-paint path, but may not be equivalent to browser CSS.
- **Bounded approximation** — intentionally smaller than CSS/Tailwind; limits are
  documented and tested so apps do not over-assume.
- **Unsupported** — no public `Tw` method exists or the behavior is deferred.

## Matrix

| Area | Methods / types | Status | Contract |
| --- | --- | --- | --- |
| Box model | `m`, `mx`, `my`, directional margin, `p`, `px`, `py`, directional padding | Rendered | Maps to `egui::Frame` outer/inner margins. |
| Sizing | `w`, `h`, `w_full`, `h_full`, percent/viewport width-height, min/max helpers, `aspect_ratio` | Rendered / bounded | Sets egui min/max dimensions. Percent/viewport values resolve against current available/content rect, not CSS layout tree. |
| Display/layout | `block`, `flex`, `flex_col`, `flex_wrap`, `grid`, `grid_cols`, `grid_rows`, `gap`, `gap_x`, `gap_y`, `space_x`, `space_y` | Rendered / recorded intent | Uses egui layout/grid primitives. `space_*` and `gap_*` set item spacing; CSS flexbox parity is not claimed. |
| Divide | `divide_x`, `divide_y` | Bounded approximation | Paints a center-line hint in the container, not browser-style per-child dividers. |
| Position | `absolute`, `fixed`, `sticky`, `relative`, `inset`, `translate_x`, `translate_y`, `z` | Rendered / bounded | Uses `egui::Area` for positioned modes; no browser containing-block model. |
| Colors | `bg`, `bg_alpha`, `text_color`, `border_color`, aliases `background`, `foreground_color` | Rendered | Maps to frame fill, override text color, and border stroke where applicable. |
| Opacity | `opacity`, `opacity_50`, `opacity_75` | Rendered / bounded | Multiplies alpha for Tw-owned frame fill, stroke, text color, elevation shadow, gradients, drop shadow, ring, and backdrop overlay. It does not globally fade arbitrary child widgets. |
| Theme tokens | `bg_surface`, `text_surface`, `bg_accent`, `bg_accent_alpha`, `bg_surface_alpha`, `text_accent`, `TwThemeVariants`, `ResponsiveTw` | Rendered / recorded intent | Tokens resolve through `Theme::load(ctx)` before rendering. Responsive/theme variants pick a concrete `Tw` before render. |
| Borders/radius | `border_*`, directional borders, `rounded_*`, grouped corner helpers | Rendered / bounded | Uniform border maps to frame stroke; directional borders are post-painted edge strokes. Corner radius maps to egui `CornerRadius` and clamps to `u8`. |
| Typography | `text_xs`..`text_3xl`, `font_thin`..`font_black`, `font_weight(100..900)`, `font_mono`, `font_sans`, `tracking_*`, `rich_text`, `label` | Rendered / recorded intent | Tailwind's discrete 100–900 weight steps are recorded directly. `font_weight(u16)` maps off-step values to the nearest Tailwind step, then records that step for widgets. R100-005A propagates that numeric weight intent into `TypeSpec` and keeps egui-native `rich_text` / `TypeSpec::to_rich_text` rendering bounded to weak / normal / strong emphasis. Phase 8 exact family selection is limited to egui built-in `Monospace`/`Proportional` aliases; R100-005A does not select weight-specific font faces. |
| Elevation/shadow | `shadow(Elevation)` | Rendered | Maps design elevation to `egui::Frame::shadow`. |
| Drop shadow | `drop_shadow(offset, blur, color)` | Bounded default / source-qualified exact subset | Default and rejected cases use deterministic Gaussian-weighted soft-shadow layers. R100-002 may use an initialized egui-wgpu source-layer callback only for solid non-rounded rectangular frames with an opaque solid fill, no border/ring/gradient/directional-border/divide mismatch, blur at least `1.0`, finite in-budget geometry, and a parent-painter slot reserved before frame/content paint. This is not CSS-complete/browser shadow compositing. |
| Gradient | `bg_gradient`, `bg_gradient_stops`, `bg_gradient_to_r`, `bg_gradient_to_b`, `bg_gradient_angle`, `bg_gradient_angle_stops` | Bounded approximation | Paints linear meshes into the `Tw` frame background using the existing draw helper, including arbitrary-angle and multi-stop linear gradients. Radial, conic, and path-clipped gradients remain outside `Tw`; lower-level draw helpers may support richer cases. |
| Blur/backdrop blur | `backdrop_blur(radius)`, `backdrop_blur_app_provided(radius)`, `TwBackdropSource` | Bounded default / explicit source-qualified exact subset | Default `Tw::backdrop_blur` paints a translucent overlay proportional to radius. `Tw::backdrop_blur_app_provided` explicitly selects the R100-001A app-provided snapshot helper and uses an exact callback only when that helper reports exact support; every non-exact report falls back to the bounded overlay. Neither path samples native/host/browser pixels implicitly. |
| Ring | `ring(width, color)` | Rendered | Post-paints an outside stroke expanded by ring width. |
| Selection | `selection(bg, fg)` | Rendered | Temporarily updates egui selection visuals for child UI. |
| Cursor/pointer events | `cursor_*`, `pointer_events_none`, `pointer_events_auto` | Rendered / bounded | Cursor uses `Response::on_hover_cursor`; pointer-events-none renders children disabled. |
| State variants | `hover`, `pressed`, `focus`, `selected`, `disabled`, `TwVariants::resolve`, `show_variant`, `show_response` | Rendered / recorded intent | Resolves one concrete `Tw` from visual state. Missing variants fall back to base. |
| Transition | `transition(duration_secs)`, `TwVariants::resolve_animated` | Bounded implementation | Variant rendering animates a safe subset: background color, foreground color, border color, opacity, border width, and uniform border radius. Layout-affecting utilities snap to target values. `MotionPolicy::from_ctx` sets duration to zero when reduced motion is enabled. |
| Filters | none | Unsupported | CSS filters (`brightness`, `contrast`, `saturate`, `hue-rotate`, `blur` as image filter) still have no public `Tw` API in this Phase 4 slice. Use lower-level image/draw code or document app-specific behavior. |

## Phase 6 Exact Supported Subsets

- **Phase 6 exact rect-frame subset:** `Tw` can count as exact evidence only for
  deterministic egui-owned rectangular frame styling: padding, fixed pixel sizing,
  resolved theme/background/foreground tokens, uniform border radius, uniform
  border/ring strokes, and linear rectangle gradients. The fixture row
  `tailwind-supported-gradient-card` locks this subset with zero tolerance.
- Browser layout/flex/grid parity remains bounded: the exact subset does not claim
  CSS containing blocks, browser flex/grid layout, global child opacity, filters,
  divide semantics, or native backdrop capture.
- `Tw::to_type_spec` bridges the exact-capable ASCII/default-font typography
  subset into `TypeSpec` and `render_text_block` for size, tracking, foreground
  color, and — after R100-005A — numeric weight intent. Font weight remains
  non-exact evidence in this bridge because the current `TypeSpec` egui render
  path and `egui::FontId` do not select weight-specific fonts. `RichText` weight rendering
  remains bounded weak/normal/strong egui emphasis for direct
  `Tw::rich_text` use and for `TypeSpec::to_rich_text`.

## Phase 7 Exact Core Slices

- **State endpoint subset:** `tailwind-supported-state-endpoints` proves resolved
  hover/focus/selected/disabled endpoint styling for fixed-size rectangular
  surfaces: token colors, opacity, uniform radius, border/ring strokes, and
  snapped padding/sizing. Live in-between animation frames, layout-affecting
  transitions, flex/grid/layout parity, divide semantics, global child opacity,
  and default backdrop blur remain bounded.
- **Typography decoration/overflow subset:** `typography-supported-decoration-overflow`
  proves ASCII/default-font underline, strikethrough, foreground color, tracking,
  alignment, and clip/ellipsis boundaries. Font weight is not exact evidence
  unless a future render path selects weight-specific fonts.
- **M3 button/card subset:** `m3-button-card-states` proves deterministic fixed-size
  token surfaces for filled/outlined/elevated buttons and filled/outlined cards.
  It does not promote the broad M3 family to Stable.

## Phase 8 Exact Built-in Families and Real-Page Slices

- **Built-in family subset:** `font_mono`, `font_sans`, and `Tw::to_type_spec`
  can select egui's built-in `Monospace` and `Proportional` families exactly for
  fixed ASCII/default-font fixtures. This does not claim bundled font files,
  fallback stacks, or weight-specific glyph selection.
- **M3 broader endpoint subset:** `m3-input-control-states`,
  `m3-text-field-states`, and `m3-navigation-list-states` prove deterministic
  fixed endpoint token visuals for input controls, text fields, navigation, and
  list items. Animated/indeterminate/platform accessibility breadth remains
  bounded.
- **Real-page crop slices:** `ui-assets-page1-el3-fill` and
  `ui-assets-page1-el4-fill` are exact crop-slice evidence from the existing
  real page corpus. The full `ui-assets-page1` row remains bounded and does not
  become a whole-page parity claim.

## Test / Fixture Traceability

| Contract area | Proof path |
| --- | --- |
| Box model, spacing, borders, sizing, colors, typography, shadows, effects, and bounded methods are represented in the public contract | `src/tailwind/render.rs::tw_render_contract_names_supported_and_bounded_methods` |
| Responsive and theme variants | `src/tailwind/responsive.rs` and `src/tailwind/theme_tokens.rs` unit tests |
| State variants and bounded transitions | `src/tailwind/state.rs` unit tests and `docs/ui-framework/tw-render-contract.md` transition row |
| Elevation/shadow conversion | `src/tailwind/shadow.rs` unit tests |
| Blur helper boundaries used by bounded blur/backdrop/drop-shadow docs | `src/blur/mod.rs` unit tests |
| Exact/approx/unsupported fidelity vocabulary used by future render paths | `src/render/mod.rs` and `docs/ui-framework/render-fidelity-contract.md` |
| Spacing constants and edge values | `src/tailwind/spacing.rs` unit tests |
| Minimal committed visual regression fixture for Tailwind/effects | `tests/visual_diff/fixtures/manifest.tsv` row `stage7-tailwind-effects` and `tests/visual_diff_harness.rs` |
| Stage 12/Phase 5 egui-bounded layout, soft-shadow, and backdrop proof | `tests/visual_diff/fixtures/manifest.tsv` rows `tailwind-layout-bounds`, `tailwind-soft-shadow`, `tailwind-backdrop-layered`; `src/tailwind/render.rs` Stage 12 and Phase 5 unit tests |
| Phase 6 exact rect-frame/Tw and TypeSpec typography subsets | `tests/visual_diff/fixtures/manifest.tsv` rows `tailwind-supported-gradient-card`, `typography-supported-ascii-panel`; `src/tailwind/render.rs`, `src/tailwind/typography.rs`, and `src/typography/text.rs` unit tests |
| Phase 7 exact state, typography decoration/overflow, and M3 button/card slices | `tests/visual_diff/fixtures/manifest.tsv` rows `tailwind-supported-state-endpoints`, `typography-supported-decoration-overflow`, `m3-button-card-states`; `src/tailwind/state.rs`, `src/typography/text.rs`, and `src/m3/components/*` unit tests |
| Phase 8 built-in family, M3 endpoint breadth, and real-page crop slices | `tests/visual_diff/fixtures/manifest.tsv` rows `typography-supported-family-selection`, `m3-input-control-states`, `m3-text-field-states`, `m3-navigation-list-states`, `ui-assets-page1-el3-fill`, and `ui-assets-page1-el4-fill`; `src/tailwind/typography.rs`, `src/typography/core.rs`, `src/m3/components/inputs.rs`, `src/m3/tier2/*`, and `tests/visual_diff_harness.rs` |
| Phase 9A source-layer blur/feather effects | `tests/visual_diff/fixtures/manifest.tsv` rows `scene-supported-gaussian-blur` and `scene-supported-feather`; `src/gpu.rs`, `src/scene/effects_geom.rs`, `src/scene/render.rs`, and `docs/ui-framework/render-fidelity-contract.md` |
| Phase 9B source-layer shadow/glow effects | `tests/visual_diff/fixtures/manifest.tsv` rows `scene-supported-drop-shadow` and `scene-supported-outer-glow`; `src/gpu.rs`, `src/scene/effects_geom.rs`, and `docs/ui-framework/render-fidelity-contract.md` |
| R100-002 Tailwind source-qualified effects | `tests/visual_diff/fixtures/manifest.tsv` rows `tailwind-supported-drop-shadow-wgpu` and `tailwind-supported-backdrop-snapshot-blur`; `src/tailwind/exact_effects.rs`, `src/tailwind/render.rs`, `src/backdrop.rs`, and `tests/visual_diff_harness.rs` |
| R100-005A typography weight-intent propagation | `src/tailwind/typography.rs`, `src/typography/core.rs`, `src/m3/typography.rs`, and this contract. Numeric 100–900 weight intent is preserved through Tailwind/M3-to-`TypeSpec`; bounded `RichText` emphasis remains weak/normal/strong; no exact weight-specific font-face fixture or claim is added. |
| Clipping approximation on layered backgrounds | `tests/visual_diff/fixtures/manifest.tsv` row `clip-layered-background`; `src/draw/clipping.rs` docs |
| Release visual regression harness and committed parity corpus | `tests/visual_diff_harness.rs`, required manifest rows `ui-assets-page1`, `gradient-mesh-quad`, `vector-clip-nested`, `compound-clip-hole`, plus `docs/ui-framework/module-map.md` Release Validation section |

## Stage 7 Visual Fixture Policy

Stage 7 added a minimal deterministic fixture row (`stage7-tailwind-effects`) to
prove the harness can carry Tailwind/effect fixtures. Stage 12 expands the
headless fixture corpus with egui-bounded layout, soft-shadow, backdrop-layer,
editor/canvas, and clipping/layered-background cases. These fixtures prove the
documented egui-native contract, not browser/CSS pixel parity.

Phase 5 keeps the default `backdrop_blur` row in the bounded corpus. WGPU exact
effect reports are not evidence for `Tw` overlay exactness unless a public API
explicitly selects `GpuSourceLayerEffectCallback` for a library-owned
source-layer effect path and validates it with an exact score-class fixture.

Phase 9A adds exact source-layer `GaussianBlur`/`Feather` evidence only for
initialized, scene-owned, solid rectangular RGBA layers through
`GpuSourceLayerEffectCallback`. This does not change the
default `Tw::backdrop_blur` overlay, `Tw::drop_shadow` soft-shadow approximation,
or unsupported CSS filter row.

Phase 9B adds exact source-layer `DropShadow`/`OuterGlow` evidence only for
initialized scene-owned non-rounded solid rectangles with normal requested blend
requested blur/radius at least `1.0`, and padded RGBA layers through
`GpuSourceLayerEffectCallback`. Zero/sub-pixel shadow blur requests fall back.
This still does not promote default `Tw::drop_shadow`, `Tw::backdrop_blur`, CSS
`box-shadow`/`filter`, host framebuffer backdrop, or broad Tailwind effects to
exact parity.

R100-001A adds an exact app-provided snapshot backdrop row only for explicit
`app_provided_backdrop_blur_shape(...)` use with an installed provider,
context-marked initialized WGPU resources, `radius >= 1.0`, and a valid
tightly-packed RGBA snapshot. Default `Tw::backdrop_blur` remains the bounded
overlay/tint path and `tailwind-backdrop-layered` remains `score-class: bounded`.

R100-001B B3 adds renderer-bound runtime sampling for hosts that can provide
same-frame app-owned WGPU pixels and bind them to the active egui-wgpu renderer.
That exact path is available only through the explicit app-owned report/shape
helpers after successful source binding. It does not change default
`Tw::backdrop_blur`, browser `backdrop-filter`, or native/host framebuffer
capture behavior.

R100-001B B4 keeps this Tailwind boundary explicit: no `Tw` method is promoted
by the app-owned WGPU source contract. A future Tailwind exact backdrop API would
need its own source contract, helper selection rule, tests, docs, and Oracle
approval before `Tw::backdrop_blur` can do anything beyond the current bounded
overlay/tint behavior.

R100-002 adds source-qualified Tailwind effect subsets without changing the
bounded defaults. `Tw::drop_shadow` can select the exact egui-wgpu source-layer
callback only for the narrow solid rectangular subset described in the matrix,
and it keeps the existing soft-shadow fallback for all other cases.
`Tw::backdrop_blur_app_provided` is the only Tailwind backdrop method that routes
through the R100-001A app-provided snapshot helper; default `Tw::backdrop_blur`
continues to use the overlay/tint path. Exact evidence is limited to the
`tailwind-supported-drop-shadow-wgpu` and
`tailwind-supported-backdrop-snapshot-blur` fixture rows plus unit tests. CSS
filters, browser `backdrop-filter`, native/host framebuffer capture, app-owned
WGPU Tailwind tokens, codegen effect emission, rounded shadows, and CSS-complete
shadow parity remain open or unsupported.

R100-005A narrows typography weight/font parity only by preserving numeric weight
intent through `Tw::to_type_spec`, `TypeSpec`, and the M3 `to_type_spec` bridge.
It does not add a font registry, fallback resolver, codegen/plugin emission,
font assets, OS font enumeration, or an exact visual weight fixture. Parent
`R100-005` remains open for weight-specific font-face selection and broader font
parity work.

## Non-Goals

- No native compositor blur or platform backdrop capture.
- No browser-complete CSS layout engine.
- No full visual regression suite across examples; Stage 9 owns that.
- No oversized-module splitting; DEBT-017 owns cleanup.

See `docs/ui-framework/render-fidelity-contract.md` for the shared exact vs
approximate reporting contract used by draw/scene/backdrop expansion work.
