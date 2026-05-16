# Render Fidelity Contract

This contract keeps `egui_expressive` honest while it grows from egui-native drawing into optional offscreen and WGPU paths. Public authoring remains immediate-mode: callers build UI each frame with egui `Ui`/`Response` APIs, while render/effect helpers report what fidelity they actually achieved.

## Core Rules

- Default behavior stays egui-native and portable.
- High-fidelity CPU/WGPU paths are opt-in by capability, feature, or caller choice.
- No hidden DOM, retained app tree, or runtime scheduler may be introduced by render fidelity work.
- Bounded caches are allowed only for renderer resources such as textures, fonts, shader pipelines, or content hashes.
- Exact paths must return/report issues instead of silently degrading to approximations.
- >95 visual claims apply only to the declared supported contract and its validation corpus, not universal CSS/browser/Figma/Illustrator parity.

## Code Surface

`src/render/mod.rs` owns the shared vocabulary:

- `RenderBackendKind` — egui painter, CPU offscreen, existing egui-wgpu callback, future WGPU offscreen.
- `RenderFeature` — blend groups, clips, blur, backdrop, shadows, masks, text/layout categories.
- `RenderQuality` — `Exact`, `Approximate`, or `Unsupported`.
- `RenderCapabilities` — backend capability flags and offscreen pixel budget.
- `OffscreenRequest` — bounded allocation request for group rendering.
- `RenderIssue` / `RenderIssueKind` — explicit reason an exact request degraded or could not run.
- `RenderReport` — per-call outcome with backend, requested quality, actual quality, and issues.
- `EffectFallback` — docs/API-facing exact/approx/unsupported classification.

## Active Render Ownership

| Area | Active owner | Contract |
| --- | --- | --- |
| Layer compositing | `src/draw/composite_core.rs` + `src/draw/rasterize.rs` | CPU offscreen per-pixel blend for supported shapes; report unsupported shapes, invalid bounds, and size-budget failures. |
| Pixel raster helpers | `src/draw/raster_pixels.rs` | Internal implementation detail for supported rect/circle/ellipse/path/mesh pixels. |
| Polygon and compound clipping on groups | `src/draw/composite_core.rs` + `src/draw/transform_clip_layout.rs` | CPU alpha mask after group rasterization for polygon, rect, rounded-rect contours, even-odd/non-zero compound masks, and bounded alpha masks; invalid masks report approximation and paint fallback. |
| Shape-level clipping helpers | `src/draw/clipping.rs` | Bounded egui-native approximation unless a CPU mask path is explicitly used. |
| Blur/shadow helpers | `src/blur/mod.rs` | Deterministic egui-native approximations unless a later offscreen backend reports exact blur. |
| Tailwind-style rendering | `src/tailwind/render.rs` + `src/tailwind/exact_effects.rs` | Consumes egui-native contract in `tw-render-contract.md`; R100-002 adds exact source-qualified Tailwind drop-shadow and app-provided backdrop subsets while preserving bounded defaults. Browser CSS parity is not implied. |
| Typography weight intent | `src/tailwind/typography.rs` + `src/typography/core.rs` + `src/m3/typography.rs` | R100-005A propagates numeric 100–900 weight intent from Tailwind and M3 into `TypeSpec`; `RichText`/`TypeSpec::to_rich_text` stay bounded to weak / normal / strong emphasis, and `egui::FontId` still does not select weight-specific font faces. |
| Scene rendering | `src/scene/render.rs` | Emits egui shapes and uses draw compositing helpers for appearance stacks. |
| GPU callback upload | `src/gpu.rs` + `src/backdrop.rs` + `src/draw/blend_shader.wgsl` + `src/draw/blur_shader.wgsl` | Supported feature-gated path for CPU-composited textures, Phase 9A/9B source-layer effects, and R100-001A app-provided backdrop snapshots through egui-wgpu callbacks. Phase 3B uploads source pixels, renders them into a callback-owned `Rgba8UnormSrgb` offscreen target with fixed-function blending disabled, then presents that target with normal uniforms. Phase 9A runs initialized, library-owned, solid rectangular RGBA blur layers through a two-pass separable blur callback before presentation. Phase 9B extends that same source-layer path to exact solid-rect `DropShadow`/`OuterGlow` subsets with requested blur/radius at least `1.0` by blurring a padded transparent RGBA source layer. R100-003A broadens the scene source layer to approved rasterized rounded-rect, ellipse, closed-path, and rotated-rect-as-closed-path sources while preserving normal-blend, WGPU-readiness, budget, radius, and shaped spread-zero gates. R100-001A allows exact backdrop blur only for an app-provided RGBA snapshot from the same single egui context/surface. R100-001B B2 freezes a contract for app-owned same-frame WGPU offscreen backdrop sources; B3 samples them only after renderer-bound sidecar proof. Not true framebuffer/native capture. |

Phase 7 exact external fixture rows map specific active owners to strict zero-tolerance evidence: `phase7-supported-polygon-clip-gradient` covers supported polygon clip plus linear gradient/stroke, `phase7-supported-compound-hole-fill` covers even-odd compound masks with simple fill/stroke, and `phase7-supported-multiply-stack` covers supported CPU blend-stack overlap. These rows do not promote broad Illustrator/page parity or gradient-mesh parity.

Inactive/stale-looking files such as historical `src/draw/composite.rs`, `src/draw/composite_masks.rs`, and `src/draw/effects.rs` must not become competing owners without a follow-up cleanup stage that wires them into `src/draw/mod.rs` or removes them.

## Fidelity Levels

### Exact

The rendered result is exact or near-exact for the declared supported subset, with deterministic validation. Examples: supported CPU blend equations over supported rasterized shapes; exact fixture rows with strict tolerance.

### Approximate

The output is intentionally bounded and documented. Examples: egui-native backdrop overlay instead of sampling pixels behind the widget; shape-layered soft shadows instead of browser compositor shadows.

### Unsupported

The feature is not implemented for the requested backend/contract. Unsupported paths must return `RenderIssueKind::UnsupportedFeature` or a more specific issue instead of pretending to be exact.

## Current Backend Capabilities

| Backend | Status | Exact today | Bounded today |
| --- | --- | --- | --- |
| `EguiPainter` | Default | Basic shapes/widgets/tokens within egui semantics | Global opacity, backdrop blur, complex clipping/compositing, CSS layout parity |
| `CpuOffscreen` | Additive deterministic path | Supported blend groups plus polygon, compound vector, and bounded alpha masks within size budget | Large groups, unsupported egui shapes, blur/backdrop sampling |
| `EguiWgpuCallback` | Supported optional callback path | Presents supported CPU-composited blend/mask group textures, Phase 9A initialized solid-rect source-layer `GaussianBlur`/`Feather`, Phase 9B initialized solid-rect source-layer `DropShadow`/`OuterGlow`, R100-003A initialized rasterized rounded-rect, ellipse, closed-path, and rotated-rect-as-closed-path scene source layers for `GaussianBlur`, `Feather`, `DropShadow`, and `OuterGlow` only, R100-001A initialized app-provided backdrop snapshots, R100-001B B3 renderer-bound app-owned WGPU backdrop sources, and R100-002 Tailwind source-qualified exact drop-shadow/app-provided backdrop subsets with requested blur/radius at least `1.0` via bounded egui-wgpu callback caches, typed uniforms, two-pass separable blur shaders, source-allocation sidecar proof for app-owned WGPU sources, reserved Tailwind background painter slots, and callback-owned offscreen render targets | No host/native framebuffer capture, no default Tailwind backdrop exactness, no CSS-complete Tailwind shadow/filter parity, no backend-global backdrop exactness, no group/mesh/non-normal/zero-radius/inner-effect/oversized shaped scene exactness claim |
| `WgpuOffscreen` | Phase 5 approved implementation path for existing optional `wgpu`/`egui-wgpu` features only | Exact may be reported only for library-owned source layers through `GpuSourceLayerEffectCallback`/`wgpu_source_layer_effect_report` after code and fixtures validate that path | Host framebuffer capture, native backdrop sampling from arbitrary app pixels, and live GPU screenshot CI remain unsupported |

## Reporting Requirements

Any helper that accepts an exact render/effect request should either:

1. return a `RenderReport`, or
2. expose a report-returning sibling while retaining a compatibility wrapper.

Compatibility wrappers may ignore reports for old callers, but new high-fidelity code must inspect them before making exactness claims.

Required issue cases:

- unsupported shape/content in exact group rasterization,
- offscreen dimensions beyond budget,
- invalid/empty clip masks,
- missing backend/disabled feature,
- approximate fallback chosen for performance or portability,
- unsupported feature outside the declared contract.

## Validation Governance

- Visual fixtures live in `tests/visual_diff/fixtures/manifest.tsv`.
- Required fixture rows must carry preceding `fixture-intent`, `fixture-source`, and `fixture-backend` comments naming the case.
- Required fixture rows must carry a preceding `score-class` comment naming the case and classifying it as `exact`, `bounded`, or `plumbing`.
- Only `exact` score-class rows with strict zero tolerance may count as strict parity evidence for >95 visual-fidelity claims.
- Broad tolerances still require a preceding `tolerance-justification` comment naming the case.
- Approximate fixtures prove bounded behavior only; they must not be counted as exact parity evidence.
- Default CI remains no-GPU unless a later approved stage adds optional WGPU/live screenshot validation.
- Phase 7 exact rows are contract traceability evidence for declared supported subsets only; broad rows such as `ui-assets-page1` remain bounded, and `gradient-mesh-quad` remains plumbing.
- Phase 8 crop-slice rows are exact evidence for named interior regions of the existing real-page corpus only. They do not promote full-page `ui-assets-page1`, Illustrator antialiasing edges, color-management outliers, or gradient mesh to exact parity.
- Phase 9A effect rows `scene-supported-gaussian-blur` and `scene-supported-feather` are exact evidence only for context-marked initialized WGPU callback resources and library-owned solid rectangular RGBA source layers within budget. Existing bounded rows such as `tailwind-soft-shadow` and `tailwind-backdrop-layered` remain bounded evidence.
- Phase 9B effect rows `scene-supported-drop-shadow` and `scene-supported-outer-glow` are exact evidence only for context-marked initialized WGPU callback resources, normal blend, requested blur/radius at least `1.0`, non-rounded solid scene rectangles, and padded library-owned RGBA source layers within budget. They do not promote Tailwind `drop_shadow`, Tailwind `backdrop_blur`, host-framebuffer backdrop, rounded/non-rect scene effects, or codegen shadow parity to exact.
- R100-003A effect rows `scene-supported-rounded-rect-blur`, `scene-supported-ellipse-drop-shadow`, `scene-supported-path-feather`, and `scene-supported-rotated-rect-drop-shadow` are exact evidence only for `GaussianBlur`, `Feather`, `DropShadow`, and `OuterGlow` over context-marked initialized WGPU callback resources, normal blend, finite in-budget rasterized library-owned shaped RGBA source layers, requested blur/radius at least `1.0`, and shaped shadow/glow spread equal to `0.0`. They do not promote group, mesh, open path, non-normal blend, zero/sub-pixel shadow, oversized, inner effect, Tailwind, codegen, browser, native, or host paths to exact.
- R100-001A row `backdrop-supported-app-snapshot-blur` is exact evidence only for an app-provided tightly packed 8-bit sRGB straight-alpha RGBA snapshot on the initialized egui-wgpu callback path. It proves snapshot-input backdrop blur for one egui context/surface; it does not prove host/native framebuffer capture, browser `backdrop-filter`, default Tailwind backdrop, or broad current-render proof.
- R100-002 rows `tailwind-supported-drop-shadow-wgpu` and `tailwind-supported-backdrop-snapshot-blur` are exact evidence only for the source-qualified Tailwind subsets they name. The drop-shadow row covers initialized egui-wgpu callbacks for solid non-rounded rectangular Tailwind frames with safe background ordering. The backdrop row covers explicit `Tw::backdrop_blur_app_provided` over an app-provided snapshot and includes a source-traceability PNG. They do not prove default `Tw::backdrop_blur`, browser `backdrop-filter`, native/host framebuffer capture, codegen effect parity, or CSS-complete Tailwind effects.

## Approval Gates

Explicit user approval is required before:

- treating `wgpu`/`egui-wgpu` as a supported high-fidelity backend,
- making optional `clip-mask`/`tiny-skia` paths required for validation,
- adding layout/text/SVG/vector dependencies,
- adding live desktop/GPU screenshot CI,
- adding large binary reference assets or proprietary-tool requirements.

Phase 3B makes the existing optional egui-wgpu callback path own a first offscreen render-target pass for bounded CPU-composited textures. Host framebuffer/backdrop capture, live capture, new layout/text/vector dependencies, shader blur, and native backdrop sampling remain approval-gated later work.

Phase 5 approval records the user decision to use the existing optional `wgpu`/`egui-wgpu` path for bounded high-fidelity work on library-owned source layers only. The implemented production path is `GpuSourceLayerEffectCallback`, which uploads caller-owned RGBA source pixels, runs `src/draw/blur_shader.wgsl` into a callback-owned offscreen target, then presents that target through the existing callback blend path. This approval does not include new dependencies, host framebuffer capture, proprietary live export, or arbitrary native backdrop sampling. Any exact WGPU effect claim must return/report `Unsupported` for host-framebuffer backdrop requests and must remain feature-gated. Apps that want automatic scene exact-effect selection must call `init_gpu_effects_for_context(...)`; `init_gpu_effects(...)` alone installs direct callback resources but does not mark unrelated egui contexts as exact-ready.

Phase 9A hardens that same approved WGPU path for the narrow effects score gap: `GpuSourceLayerEffectCallback` now uses a two-pass separable blur over a library-owned RGBA source layer, scene `GaussianBlur` and `Feather` may select it only for context-marked initialized WGPU callback resources, normal blend, solid rectangular source geometry, and in-budget requests. Non-WGPU, uninitialized/unmarked WGPU, non-normal-blend, non-rect, rounded, ellipse, path, group, and oversized cases keep the existing egui-native soft-shadow fallback. Backdrop blur, host framebuffer capture, and broad Tailwind effect parity remain bounded or unsupported.

Phase 9B extends the same context-marked initialized WGPU source-layer callback to exact scene `DropShadow` and `OuterGlow` only for non-rounded solid rectangles with requested blur/radius at least `1.0`, normal requested blend, and in-budget padded RGBA layers. The scene helper rejects non-normal requested blend even when internal offscreen painting asks for forced normal paint mode, and it falls back for zero/sub-pixel shadow blur requests rather than silently widening them. Non-WGPU, uninitialized/unmarked WGPU, rounded/non-rect/path/group, oversized, inner-shadow/glow, Tailwind `drop_shadow`, Tailwind `backdrop_blur`, host framebuffer capture, and broad CSS shadow/backdrop parity remain bounded or unsupported.

R100-003A extends the scene-only source-layer path from solid rectangles to approved rasterized shaped sources for `GaussianBlur`, `Feather`, `DropShadow`, and `OuterGlow` only: rounded rectangles, ellipses, closed paths, and rotated rectangles after their existing conversion to closed paths. It preserves context-marked WGPU readiness, normal requested blend, finite in-budget callback dimensions, requested blur/radius at least `1.0`, and current solid-rect spread behavior. Shaped `DropShadow` and `OuterGlow` are exact only when spread is exactly `0.0`; group, mesh, open path, non-normal blend, zero/sub-pixel shadow, oversized tiling, inner shadow/glow, bevel/noise/live/unrecognized effect variants, Tailwind, codegen, browser, native, and host framebuffer paths remain bounded or unsupported.

R100-001A adds `app_provided_backdrop_blur_report(...)` and `app_provided_backdrop_blur_shape(...)` for app-provided backdrop snapshots. Exact output requires the `wgpu` feature, `init_gpu_effects_for_context(...)` on the same context, an installed `BackdropSnapshotProvider`, `radius >= 1.0`, valid in-budget physical dimensions, and a provider snapshot whose size and tightly packed RGBA byte length exactly match `BackdropCaptureRequest`. `GpuEffectSource::AppProvidedBackdropSnapshot` can report exact `BackdropBlur`; `GpuEffectSource::HostFramebufferBackdrop` remains unsupported, and `RenderCapabilities::egui_wgpu_callback(...)` still does not globally claim `exact_backdrop_blur`.

R100-001B begins native adapter work as a staged program. The common
`native-backdrop` substrate freezes adapter feature names and initialization
errors only; it does not capture native pixels, does not change any backend
capability flag, and does not promote `GpuEffectSource::HostFramebufferBackdrop`.
Future native adapters must feed validated snapshots through the R100-001A
`AppProvidedBackdropSnapshot` path until a separate Oracle-approved
host-framebuffer contract exists.

R100-001B B2 adds the WGPU-first app-owned backdrop source contract. The contract
is source-qualified to app-owned same-frame `TextureView` inputs, same
context/surface identity, frame freshness, `Rgba8UnormSrgb`, sample count `1`,
straight alpha, validated size/scale, and a WebGPU-capable host on web/wasm. B2
does not implement runtime sampling from that source; `GpuEffectSource::AppOwnedOffscreenBackdrop`
must report non-exact until B3 or a later Oracle-approved child implements and
validates sampling. This does not change `HostFramebufferBackdrop`, native capture,
browser `backdrop-filter`, or default Tailwind behavior.

R100-001B B3 implements the first app-owned WGPU runtime sampling path for that
B2 source. Exact output now requires `app_owned_offscreen_backdrop_blur_report(...)`
or `app_owned_offscreen_backdrop_blur_shape(...)`, `init_gpu_effects_for_context(...)`,
an installed B2 source, successful `bind_app_owned_offscreen_backdrop_source_for_context(...)`
against the active `egui-wgpu` renderer, matching surface/frame/scale metadata,
matching the installed source allocation retained by the renderer-bound sidecar,
an in-bounds physical subrect, and the egui-wgpu callback backend. The callback
samples the validated source subrect via a UV-transform first pass, then reuses
the existing separable blur/present path. Direct generic
`GpuEffectSource::AppOwnedOffscreenBackdrop` reports remain non-exact without the
renderer-bound sidecar, and `RenderCapabilities::egui_wgpu_callback(...)` still
does not globally claim `exact_backdrop_blur`. This remains app-owned-pixel
sampling only; it is not host framebuffer, native compositor, browser CSS, or
Tailwind default backdrop capture.

R100-001B B4 records the support boundary for that WGPU-first path. The B3
implementation meaningfully narrows `R100-001` only for pixels the host already
owns and binds into the active egui-wgpu renderer. It does not close the parent
host/native framebuffer blocker. Platform claims stay limited to the app-owned
source contract plus validation evidence: Linux local gates passed, wasm/WebGPU
compile passed, and manual/device smoke remains required for WGPU context loss,
resize, high-DPI, mobile rotation, and browser runtime support before an app can
claim those environments are production-proven. `Tw::backdrop_blur` remains the
bounded overlay path until a separate Tailwind source contract explicitly routes
through an exact source-backed helper.

R100-002 implements that separate Tailwind source contract for two named subsets.
`Tw::drop_shadow` may select an exact egui-wgpu source-layer callback only for an
opaque solid non-rounded frame rectangle with initialized WGPU resources, blur at least
`1.0`, a parent-painter slot reserved before frame/content paint, and no border/ring/gradient/
directional-border/divide mismatch. `Tw::backdrop_blur_app_provided` explicitly
selects the R100-001A app-provided snapshot helper and falls back to the bounded
overlay when the helper is not exact. Default `Tw::backdrop_blur` remains bounded,
CSS filters stay unsupported, codegen exact emission remains open beyond
`R100-004A`, and no native/host/browser capture claim is introduced.

R100-004A narrows codegen effect parity by making active direct generated shape
effect output explicit: bounded helper output is annotated with
`R100-004A bounded codegen`, and effects with no exported direct helper or exact
generated callback are annotated with `R100-004A unsupported codegen`. The active
emitter is `src/codegen/node_emit.rs` through `src/codegen/effect_emit.rs`;
`src/codegen/render_shape.rs` remains inactive legacy evidence. R100-004A does
not emit exact WGPU callbacks, WGPU initialization, render-state plumbing, or
context-readiness code. Full parent `R100-004` stays open for exact generated
callback parity.

R100-005A narrows typography weight/font parity without changing renderer
backends: Tailwind and M3 weights now survive as numeric `TypeSpec.weight`
intent, and `TypeSpec::to_rich_text` uses the same bounded weak / normal / strong
emphasis as `Tw::rich_text`. This is not exact font-face rendering. No font
registry, fallback resolver, font assets, OS font enumeration, codegen/plugin
emission, or exact visual weight fixture is introduced; parent `R100-005` stays
open for weight-specific font-face selection and broader typography parity.

### R100-001B Support Matrix Boundary

| Target family | App-owned WGPU source claim | Host/native capture claim | Required extra proof before release claim |
| --- | --- | --- | --- |
| Linux desktop | Implemented and locally validated for the egui-wgpu app-owned source contract | Unsupported | Device smoke for context loss, resize, and high-DPI on the shipping host app |
| Windows desktop | Contract intended to apply through WGPU backends | Unsupported | Host-app smoke on Windows with same-source allocation rebind checks |
| macOS/iOS | Contract intended to apply through WGPU/Metal backends | Unsupported and permission-gated | Host-app smoke plus platform permission/privacy review before any capture claim |
| Android | Contract intended to apply through WGPU/Vulkan or GLES-backed host support | Unsupported and permission-gated | Rotation, DPI, lifecycle/context-loss smoke on target devices |
| Web/wasm | Compile gate passed for the WGPU/WebGPU contract | Browser `backdrop-filter`, canvas readback, and screenshot fallback unsupported | Runtime browser smoke with WebGPU available; non-WebGPU browsers must remain non-exact |
