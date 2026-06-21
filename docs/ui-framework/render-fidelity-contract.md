# Render Fidelity Contract

This contract keeps `egui_expressive` honest while it grows from egui-native drawing into optional offscreen and WGPU paths. Public authoring remains immediate-mode: callers build UI each frame with egui `Ui`/`Response` APIs, while render/effect helpers report what fidelity they actually achieved.

## Core Rules

- Default behavior stays egui-native and portable; default builds are compatibility builds, not proof of the WGPU production-fidelity tier.
- High-fidelity CPU paths are opt-in by capability or caller choice; production-fidelity WGPU paths require the `wgpu`/`gpu-effects` feature path plus initialized runtime/context resources.
- No hidden DOM, retained app tree, or runtime scheduler may be introduced by render fidelity work.
- Bounded caches are allowed only for renderer resources such as textures, fonts, shader pipelines, or content hashes.
- Exact paths must return/report issues instead of silently degrading to approximations.
- >95 visual claims apply only to the declared supported contract and its validation corpus, not universal CSS/browser/Figma/Illustrator parity.

the current release candidate WGPU policy boundary: builds without the `wgpu`/`gpu-effects` feature path are compatibility builds for render-fidelity purposes. Exact production-fidelity WGPU claims require the feature-gated WGPU API surface, successful runtime/resource/context initialization, and a `RenderReport` without lifecycle issues. The bounded the current release candidate surfaces are `RenderCapabilities`, `RenderReport`, `RenderIssueKind`, `WgpuLifecycleFailure`, `wgpu_lifecycle_report(...)`, `init_gpu_effects(...)`, `init_gpu_effects_for_context(...)`, and `bind_app_owned_offscreen_backdrop_source_for_context(...)`; they are validated policy/lifecycle/reporting APIs, not whole-product production-readiness or stable-API closure.

The 2026-06-07 the current release candidate refresh revalidated this boundary with render tests,
WGPU GPU tests, WGPU API smoke, default and WGPU rustdoc builds, examples,
no-default compatibility, default library check, and a `wasm32-unknown-unknown`
WGPU compile gate. The wasm gate is compile-only Web/WGPU evidence; Web remains
unsupported until a the current release candidate `WebSupportArtifact` records browser visible-render,
logs, resize/DPI, and lifecycle proof.

The the current release candidate plugin/codegen font fix prevents weighted/family text sidecars from being labeled exact unless explicit contour-backed glyph data or a future approved font registry/provenance contract supports the claim; generated code no longer synthesizes a fake `Bold` family for weight-only text.

The 2026-06-07 the current release candidate fidelity refresh revalidated the current scene/draw/WGPU, Tailwind/layout, typography/M3, codegen, visual-fixture, docs, and Illustrator-plugin evidence without changing Cargo defaults, dependencies, lockfiles, CI, SDKs, or source configuration. The refresh confirms the existing source-qualified exact subsets and typed non-exact diagnostics remain coherent; it does not promote unsupported group/mesh/open-path/non-normal blend/inner/bevel/noise/live/unrecognized/oversized scene cases, broad CSS/browser layout parity, exact codegen WGPU callbacks, exact weight-specific font rendering, or true gradient-mesh visual parity into supported claims.

The 2026-06-07 the current release candidate corpus/current-code proof refresh revalidated the existing deterministic current-code subset and visual fixture governance: ten `current-render/` rows regenerated through crate-local draw/raster/composite helpers, the visual diff harness, draw tests, and fixture manifest/asset review all passed. This is proof for the named rows only; full-page external corpus parity, broad screenshot/live exporter proof, and unconverted manifest rows remain out of scope until the current release candidate follow-up work add provenance/license/exporter/resource decisions and the current release candidate follow-up work add approved current-render/screenshot-runner evidence; the current release candidate owns final label audit.

The 2026-06-07 the current release candidate repo-local fidelity pass reclassified the currently
repo-local fidelity backlog parents without adding dependencies, assets, platform actions,
or new visual baselines. fidelity item, fidelity item, fidelity item, and fidelity item remain
the current release candidate candidates only for exact source-backed follow-up work; fidelity item remains
split between a the current release candidate proof path and a the current release candidate gradient-mesh decision path.
Until such follow-up work add deterministic current-render, screenshot-runner, or
codegen proof, the public contract stays demoted to the named exact fixture rows
and source-qualified subsets listed here. This pass does not close any parent
fidelity backlog row, promote regression-only manifest entries to current-code proof, or
change runtime/source behavior.

The 2026-06-07 the current release candidate decision-required pass records that the remaining
high-risk fidelity/capture parents are blocked or demoted without explicit
resource approval. fidelity item broad native/host capture remains unsupported beyond
app-owned/app-provided source contracts; no monitor, foreign-window, browser, or
host framebuffer capture is implied. fidelity item exact font/codegen/layout parity
remains blocked without approved font bytes, license/provenance IDs, glyph
coverage, and codegen/plugin emission proof. fidelity item full-page external corpus
remains blocked without approved asset provenance, license review, deterministic
exporter versioning, and zero-tolerance current/source proof. fidelity item browser
layout/flex/grid parity remains unsupported without an approved layout
architecture and browser-runtime evidence. fidelity item true gradient-mesh exactness
remains non-exact unless a future approved proof adds report-backed mesh
tessellation plus fixtures. the current release candidate performs no dependency, asset, OS,
browser/device, network, SDK, CI, Cargo, release, signing, or native-permission
action and closes no parent fidelity backlog row.

## Code Surface

`src/render/mod.rs` owns the shared vocabulary:

- `RenderBackendKind` — egui painter, CPU offscreen, existing egui-wgpu callback, future WGPU offscreen.
- `RenderFeature` — blend groups, clips, blur, backdrop, shadows, scene effects, gradient meshes, masks, text/layout categories.
- `RenderQuality` — `Exact`, `Approximate`, or `Unsupported`.
- `RenderCapabilities` — backend capability flags and offscreen pixel budget.
- `OffscreenRequest` — bounded allocation request for group rendering.
- `RenderIssue` / `RenderIssueKind` — explicit reason an exact request degraded or could not run, including WGPU lifecycle states such as missing runtime/context, unsupported adapter/device, device loss, pipeline/resource allocation failure, WGPU budget failure, the current release candidate scene-source/effect/blend diagnostics, and gradient-mesh contract failures.
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
| Tailwind-style rendering | `src/tailwind/render.rs` + `src/tailwind/exact_effects.rs` | Consumes egui-native contract in `tw-render-contract.md`; fidelity item adds exact source-qualified Tailwind drop-shadow and app-provided backdrop subsets while preserving bounded defaults. Browser CSS parity is not implied. |
| Typography weight intent and registry selection | `src/tailwind/typography.rs` + `src/typography/core.rs` + `src/m3/typography.rs` | fidelity item propagates numeric 100–900 weight intent from Tailwind and M3 into `TypeSpec`; the current release candidate own current runtime/manual font registry, fallback, and codegen/plugin parity closure work, with the current release candidate owning final release/API claim labels. `RichText`/`TypeSpec::to_rich_text` stay bounded to weak / normal / strong emphasis, and `egui::FontId` still does not select weight-specific font faces. |
| Scene rendering | `src/scene/render.rs` + `src/scene/effects_geom.rs` | Emits egui shapes, uses draw compositing helpers for appearance stacks, and reports the current release candidate scene-effect exactness/non-exactness before any exact WGPU claim. |
| GPU callback upload | `src/gpu.rs` + `src/backdrop.rs` + `src/platform/backdrop.rs` + `src/draw/blend_shader.wgsl` + `src/draw/blur_shader.wgsl` | Supported feature-gated path for CPU-composited textures, Phase 9A/9B source-layer effects, and app-provided backdrop snapshots through egui-wgpu callbacks. Phase 3B uploads source pixels, renders them into a callback-owned `Rgba8UnormSrgb` offscreen target with fixed-function blending disabled, then presents that target with normal uniforms. Phase 9A runs initialized, library-owned, solid rectangular RGBA blur layers through a two-pass separable blur callback before presentation. Phase 9B extends that same source-layer path to exact solid-rect `DropShadow`/`OuterGlow` subsets with requested blur/radius at least `1.0` by blurring a padded transparent RGBA source layer. fidelity item broadens the scene source layer to approved rasterized rounded-rect, ellipse, closed-path, and rotated-rect-as-closed-path sources while preserving normal-blend, WGPU-readiness, budget, radius, and shaped spread-zero gates. the current release candidateA allows exact app-provided backdrop blur only when a `BackdropCaptureSourceContract` proves same-context app-owned/source identity, consent, current frame freshness, unoccluded source, matching DPI scale, in-bounds physical size, and RGBA8 straight-alpha format. fidelity item B2 freezes a contract for app-owned same-frame WGPU offscreen backdrop sources; B3 samples them only after renderer-bound sidecar proof. Not true framebuffer/native capture. |

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
| `EguiWgpuCallback` | Supported feature-gated production-fidelity callback path | Presents supported CPU-composited blend/mask group textures, Phase 9A initialized solid-rect source-layer `GaussianBlur`/`Feather`, Phase 9B initialized solid-rect source-layer `DropShadow`/`OuterGlow`, fidelity item initialized rasterized rounded-rect, ellipse, closed-path, and rotated-rect-as-closed-path scene source layers for `GaussianBlur`, `Feather`, `DropShadow`, and `OuterGlow` only, fidelity item initialized app-provided backdrop snapshots, fidelity item B3 renderer-bound app-owned WGPU backdrop sources, fidelity item Tailwind source-qualified exact drop-shadow/app-provided backdrop subsets, and the current release candidate report-backed scene-effect/gradient-mesh contracts via bounded egui-wgpu callback caches, typed uniforms, two-pass separable blur shaders, source-allocation sidecar proof for app-owned WGPU sources, reserved Tailwind background painter slots, and callback-owned offscreen render targets. Exact claims must report the current release candidate lifecycle issues for missing runtime/resources/context binding, unsupported adapter/device state, device loss, pipeline/resource allocation failure, and WGPU budget failure. | No host/native framebuffer capture, no default Tailwind backdrop exactness, no CSS-complete Tailwind shadow/filter parity, no backend-global backdrop exactness; group/mesh/open-path/non-normal/inner/bevel/noise/live/unrecognized/oversized scene cases require explicit the current release candidate reports and are not silently exact |
| `WgpuOffscreen` | Feature-gated WGPU production-fidelity vocabulary and Phase 5 approved implementation path | Exact may be reported only for library-owned source layers through `GpuSourceLayerEffectCallback`/`wgpu_source_layer_effect_report` after code and fixtures validate that path; lifecycle failures must be reported through specific WGPU issue kinds instead of silent exactness or generic fallback. | Host framebuffer capture, native backdrop sampling from arbitrary app pixels, and live GPU screenshot CI remain unsupported |

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
- missing WGPU runtime/resources or missing renderer-bound egui context,
- unsupported WGPU adapter/device/runtime state, device loss, pipeline creation failure, resource allocation failure, or WGPU budget failure,
- missing app-provided backdrop source contract, permission denial, stale frame, occluded source, source-scale mismatch, or source-bounds mismatch,
- unsupported the current release candidate scene sources such as group, mesh, open path, or unsupported shape sources,
- unsupported the current release candidate scene effects such as inner shadow/glow, bevel, noise, live, or unrecognized variants,
- non-normal blend requests for exact scene effects,
- gradient-mesh subdivision or budget failures,
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
- fidelity item effect rows `scene-supported-rounded-rect-blur`, `scene-supported-ellipse-drop-shadow`, `scene-supported-path-feather`, and `scene-supported-rotated-rect-drop-shadow` are exact evidence only for `GaussianBlur`, `Feather`, `DropShadow`, and `OuterGlow` over context-marked initialized WGPU callback resources, normal blend, finite in-budget rasterized library-owned shaped RGBA source layers, requested blur/radius at least `1.0`, and shaped shadow/glow spread equal to `0.0`. They do not promote group, mesh, open path, non-normal blend, zero/sub-pixel shadow, oversized, inner effect, Tailwind, codegen, browser, native, or host paths to exact.
- fidelity item row `backdrop-supported-app-snapshot-blur` is exact evidence only for an app-provided tightly packed 8-bit sRGB straight-alpha RGBA snapshot on the initialized egui-wgpu callback path. It proves snapshot-input backdrop blur for one egui context/surface; it does not prove host/native framebuffer capture, browser `backdrop-filter`, default Tailwind backdrop, or broad current-render proof.
- fidelity item rows `tailwind-supported-drop-shadow-wgpu` and `tailwind-supported-backdrop-snapshot-blur` are exact evidence only for the source-qualified Tailwind subsets they name. The drop-shadow row covers initialized egui-wgpu callbacks for solid non-rounded rectangular Tailwind frames with safe background ordering. The backdrop row covers explicit `Tw::backdrop_blur_app_provided` over an app-provided snapshot and includes a source-traceability PNG. They do not prove default `Tw::backdrop_blur`, browser `backdrop-filter`, native/host framebuffer capture, codegen effect parity, or CSS-complete Tailwind effects.

## Approval Gates

Explicit user approval is required before:

- making `wgpu`/`egui-wgpu` part of the default Cargo feature set or requiring it for compatibility builds,
- making optional `clip-mask`/`tiny-skia` paths required for validation,
- adding layout/text/SVG/vector dependencies,
- adding live desktop/GPU screenshot CI,
- adding large binary reference assets or proprietary-tool requirements.

the current release candidate policy records WGPU as required for the production-fidelity tier while preserving lightweight default compatibility builds. This does not add dependencies, edit the lockfile, enable WGPU by default, add live GPU CI, or certify native/browser capture.

Phase 3B makes the existing optional egui-wgpu callback path own a first offscreen render-target pass for bounded CPU-composited textures. Host framebuffer/backdrop capture, live capture, new layout/text/vector dependencies, shader blur, and native backdrop sampling remain approval-gated later work.

Phase 5 approval records the user decision to use the existing optional `wgpu`/`egui-wgpu` path for bounded high-fidelity work on library-owned source layers only. The implemented production path is `GpuSourceLayerEffectCallback`, which uploads caller-owned RGBA source pixels, runs `src/draw/blur_shader.wgsl` into a callback-owned offscreen target, then presents that target through the existing callback blend path. This approval does not include new dependencies, host framebuffer capture, proprietary live export, or arbitrary native backdrop sampling. Any exact WGPU effect claim must return/report `Unsupported` for host-framebuffer backdrop requests and must remain feature-gated. Apps that want automatic scene exact-effect selection must call `init_gpu_effects_for_context(...)`; `init_gpu_effects(...)` alone installs direct callback resources but does not mark unrelated egui contexts as exact-ready.

Phase 9A hardens that same approved WGPU path for the narrow effects score gap: `GpuSourceLayerEffectCallback` now uses a two-pass separable blur over a library-owned RGBA source layer, scene `GaussianBlur` and `Feather` may select it only for context-marked initialized WGPU callback resources, normal blend, solid rectangular source geometry, and in-budget requests. Non-WGPU, uninitialized/unmarked WGPU, non-normal-blend, non-rect, rounded, ellipse, path, group, and oversized cases keep the existing egui-native soft-shadow fallback. Backdrop blur, host framebuffer capture, and broad Tailwind effect parity remain bounded or unsupported.

Phase 9B extends the same context-marked initialized WGPU source-layer callback to exact scene `DropShadow` and `OuterGlow` only for non-rounded solid rectangles with requested blur/radius at least `1.0`, normal requested blend, and in-budget padded RGBA layers. The scene helper rejects non-normal requested blend even when internal offscreen painting asks for forced normal paint mode, and it falls back for zero/sub-pixel shadow blur requests rather than silently widening them. Non-WGPU, uninitialized/unmarked WGPU, rounded/non-rect/path/group, oversized, inner-shadow/glow, Tailwind `drop_shadow`, Tailwind `backdrop_blur`, host framebuffer capture, and broad CSS shadow/backdrop parity remain bounded or unsupported.

fidelity item extends the scene-only source-layer path from solid rectangles to approved rasterized shaped sources for `GaussianBlur`, `Feather`, `DropShadow`, and `OuterGlow` only: rounded rectangles, ellipses, closed paths, and rotated rectangles after their existing conversion to closed paths. It preserves context-marked WGPU readiness, normal requested blend, finite in-budget callback dimensions, requested blur/radius at least `1.0`, and current solid-rect spread behavior. Shaped `DropShadow` and `OuterGlow` are exact only when spread is exactly `0.0`; group, mesh, open path, non-normal blend, zero/sub-pixel shadow, oversized tiling, inner shadow/glow, bevel/noise/live/unrecognized effect variants, Tailwind, codegen, browser, native, and host framebuffer paths remain bounded or unsupported.

the current release candidate adds explicit report vocabulary for the remaining scene-effect and mesh-gradient slices rather than letting silent fallbacks look exact. `scene_effect_report(...)` classifies group, mesh, open-path, unsupported-shape, non-normal blend, zero/oversized source-layer, inner shadow/glow, bevel, noise, live, and unrecognized-effect requests before any WGPU exactness claim. Existing exact WGPU source-layer behavior remains limited to approved solid/shaped library-owned sources for `GaussianBlur`, `Feather`, `DropShadow`, and `OuterGlow`; unsupported the current release candidate cases now return `UnsupportedSceneSource`, `UnsupportedSceneEffect`, `UnsupportedBlendMode`, or budget issues. `scene_mesh_gradient_report(...)` records the existing mesh-gradient subdivision contract as exact only within the documented 1..=64 subdivision and offscreen-budget limits, otherwise returning `GradientMeshUnsupported` or `SizeBudgetExceeded`. These reports narrow fidelity item/fidelity item honesty gaps; they do not add dependencies, browser layout, Tailwind/codegen parity, native capture, or host framebuffer sampling.

fidelity item adds `app_provided_backdrop_blur_report(...)` and `app_provided_backdrop_blur_shape(...)` for app-provided backdrop snapshots. Exact output requires the `wgpu` feature, `init_gpu_effects_for_context(...)` on the same context, an installed `BackdropSnapshotProvider`, `radius >= 1.0`, valid in-budget physical dimensions, and a provider snapshot whose size and tightly packed RGBA byte length exactly match `BackdropCaptureRequest`. The generic `wgpu_source_layer_effect_report(...)` no longer reports exact `GpuEffectSource::AppProvidedBackdropSnapshot` by itself; exact app-provided backdrop reports must use the contract-bearing `wgpu_app_provided_backdrop_snapshot_report(...)` helper. `GpuEffectSource::HostFramebufferBackdrop` remains unsupported, and `RenderCapabilities::egui_wgpu_callback(...)` still does not globally claim `exact_backdrop_blur`.

the current release candidateA hardens that app-provided snapshot contract with
`BackdropCaptureSourceContract`. Exact app-provided WGPU backdrop output now also
requires source/provider identifiers, surface and frame tokens, app-owned or
explicit consent, current-frame freshness, explicitly unoccluded source state,
matching DPI scale, physical-size bounds, and `Rgba8SrgbStraightAlpha` format.
Missing source contracts, permission failures, stale frames, occlusion including
unchecked occlusion, source/provider identity mismatch, scale mismatch, and
source-bounds mismatch return typed errors or redacted non-exact reports. The
contract is still app-provided/app-owned only; it does not authorize native
compositor, foreign window, monitor, browser, or host framebuffer capture.

fidelity item begins native adapter work as a staged program. The common
`native-backdrop` substrate freezes adapter feature names and initialization
errors only; it does not capture native pixels, does not change any backend
capability flag, and does not promote `GpuEffectSource::HostFramebufferBackdrop`.
Future native adapters must feed validated snapshots through the fidelity item
`AppProvidedBackdropSnapshot` path until a separate independently reviewed
host-framebuffer contract exists.

the current release candidateB extends the native-adapter substrate with platform-family labels,
support-state, permission-state, source-scope, redacted diagnostics, and manual
smoke artifact vocabulary. Those APIs are contract diagnostics only: a smoke
artifact is production evidence only for app-window/surface scope with permission
explicitly granted and redaction confirmed. Foreign-window and monitor scopes are
not production evidence, and no native provider, permission prompt, dependency,
lockfile change, or OS backend is introduced.

the current release candidateC keeps host framebuffer capture fail-closed. The public
`host_framebuffer_backdrop_report(...)` helper returns the existing unsupported
host-framebuffer report path for `GpuEffectSource::HostFramebufferBackdrop`; it
does not sample host pixels or promote a backend capability.

fidelity item B2 adds the WGPU-first app-owned backdrop source contract. The contract
is source-qualified to app-owned same-frame `TextureView` inputs, same
context/surface identity, frame freshness, `Rgba8UnormSrgb`, sample count `1`,
straight alpha, validated size/scale, and a WebGPU-capable host on web/wasm. B2
does not implement runtime sampling from that source; `GpuEffectSource::AppOwnedOffscreenBackdrop`
must report non-exact until B3 or a later independently reviewed child implements and
validates sampling. This does not change `HostFramebufferBackdrop`, native capture,
browser `backdrop-filter`, or default Tailwind behavior.

fidelity item B3 implements the first app-owned WGPU runtime sampling path for that
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

fidelity item B4 records the support boundary for that WGPU-first path. The B3
implementation meaningfully narrows `fidelity item` only for pixels the host already
owns and binds into the active egui-wgpu renderer. It does not close the parent
host/native framebuffer blocker. Platform claims stay limited to the app-owned
source contract plus validation evidence: Linux local gates passed, wasm/WebGPU
compile passed, and manual/device smoke remains required for WGPU context loss,
resize, high-DPI, mobile rotation, and browser runtime support before an app can
claim those environments are production-proven. `Tw::backdrop_blur` remains the
bounded overlay path until a separate Tailwind source contract explicitly routes
through an exact source-backed helper.

fidelity item implements that separate Tailwind source contract for two named subsets.
`Tw::drop_shadow` may select an exact egui-wgpu source-layer callback only for an
opaque solid non-rounded frame rectangle with initialized WGPU resources, blur at least
`1.0`, a parent-painter slot reserved before frame/content paint, and no border/ring/gradient/
directional-border/divide mismatch. `Tw::backdrop_blur_app_provided` explicitly
selects the fidelity item app-provided snapshot helper and falls back to the bounded
overlay when the helper is not exact. Default `Tw::backdrop_blur` remains bounded,
CSS filters stay unsupported, codegen exact emission remains open beyond
`fidelity item`, and no native/host/browser capture claim is introduced.

fidelity item narrows codegen effect parity by making active direct generated shape
effect output explicit: bounded helper output is annotated with
`fidelity item bounded codegen`, and effects with no exported direct helper or exact
generated callback are annotated with `fidelity item unsupported codegen`. The active
emitter is `src/codegen/node_emit.rs` through `src/codegen/effect_emit.rs`;
`src/codegen/render_shape.rs` remains inactive legacy evidence. fidelity item does
not emit exact WGPU callbacks, WGPU initialization, render-state plumbing, or
context-readiness code. Full parent `fidelity item` stays open for exact generated
callback parity.

fidelity item narrows typography weight/font parity without changing renderer
backends: Tailwind and M3 weights survive as numeric `TypeSpec.weight` intent,
and `TypeSpec::to_rich_text` uses the same bounded weak / normal / strong
emphasis as `Tw::rich_text`. the current release candidate adds exact runtime/manual selection reports
only when an application registers a `FontRegistry` with app-provided bytes,
approved license IDs, provenance IDs, and coverage metadata. Missing family,
face, glyph, license, approved-license membership, bytes, or deterministic
fallback/weight substitution is reported explicitly as non-exact. No bundled font
assets, OS font enumeration, browser text layout, codegen/plugin emission, or
exact visual weight fixture is introduced; parent `fidelity item` stays open for the current release candidate codegen/plugin/font decisions, the current release candidate repo-local proof where approved,
broader typography parity, and the current release candidate final label audit.

### fidelity item Support Matrix Boundary

| Target family | App-owned WGPU source claim | Host/native capture claim | Required extra proof before release claim |
| --- | --- | --- | --- |
| Linux desktop | Implemented and locally validated for the egui-wgpu app-owned source contract | Unsupported | Device smoke for context loss, resize, and high-DPI on the shipping host app |
| Windows desktop | Contract intended to apply through WGPU backends | Unsupported | Host-app smoke on Windows with same-source allocation rebind checks |
| macOS/iOS | Contract intended to apply through WGPU/Metal backends | Unsupported and permission-gated | Host-app smoke plus platform permission/privacy review before any capture claim |
| Android | Contract intended to apply through WGPU/Vulkan or GLES-backed host support | Unsupported and permission-gated | Rotation, DPI, lifecycle/context-loss smoke on target devices |
| Web/wasm | Compile gate passed for the WGPU/WebGPU contract | Browser `backdrop-filter`, canvas readback, and screenshot fallback unsupported | Runtime browser smoke with WebGPU available; non-WebGPU browsers must remain non-exact |
