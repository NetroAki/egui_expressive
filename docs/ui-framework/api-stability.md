# API Stability Map

Updated: 2026-05-13

This map is the release-facing status guide for public `egui_expressive` APIs. It is not a new feature list; it classifies the existing public surface exposed from `src/lib.rs` and nearby modules so downstream apps know which paths to prefer, which paths are compatibility aliases, and which paths remain experimental.

## Stability Categories

| Category | Meaning | Compatibility expectation |
| --- | --- | --- |
| Stable / preferred | General-purpose primitives with docs, tests, examples or release-smoke proof, and no known unsupported core behavior for their documented scope. | Prefer these paths for new app code. Breaking changes require migration notes and a major-version plan once public releases begin. |
| Beta / supported | Usable by app code, but still has bounded limitations, approximation contracts, or incomplete per-family proof. | Safe for pre-1.0 use when the linked docs match the app's needs; APIs may still evolve with migration notes. |
| Compatibility-only | Retained to avoid churn from earlier naming/domain decisions. | Existing code may keep using it, but new code should use the preferred replacement path. Removal requires an explicit migration plan. |
| Experimental / diagnostic | Developer tooling, optional acceleration, or surfaces without enough release proof for app-level guarantees. | Do not treat as stable app API. Names/behavior may change; docs must avoid support overclaims. |
| Feature-gated | Public only when the named Cargo feature is enabled. | Stability applies only to the documented feature surface; default builds must not depend on it. |

## Preferred Stable Public Paths

| Surface | Preferred public paths | Proof / docs |
| --- | --- | --- |
| Responsive values | `egui_expressive::{Responsive, Breakpoints, BreakpointName}`, `egui_expressive::responsive::*` | Stable in `component-maturity-rubric.md`; responsive tests and layout docs. |
| Data-heavy read-only widgets | `egui_expressive::{DataTable, DataGridModel, TreeTable, PropertyGrid}`, `egui_expressive::widgets::data::*` | `docs/ui-framework/data-widgets.md`, `examples/data_explorer_dashboard.rs`, interaction/performance smoke tests. |

These stable paths are still scoped by their docs. For example, data widgets are stable for read-only virtualized display, sort/filter/select fallback, property/tree presentation, and documented Forms v2 edit descriptors; advanced spreadsheet interactions remain unsupported in the current release boundary.

## Beta / Supported Public Paths

| Surface | Preferred public paths | Current support boundary |
| --- | --- | --- |
| Tailwind-style utility DSL | `egui_expressive::{Tw, ResponsiveTw, TwVariants, TwThemeVariants}` and `egui_expressive::tailwind::*` | Beta+. Rendered/recorded/bounded/unsupported behavior is defined in `docs/ui-framework/tw-render-contract.md`; Stage 12 adds deterministic egui-native layout/effect fixtures, but browser/CSS parity is still not claimed. |
| Drawing/effects | `egui_expressive::draw::*`, crate-root draw helpers such as `gradient_rect`, `zstack`, `with_opacity`, `clipped_shape` | Beta. Uses egui shapes and documented approximations; not a browser/CSS compositor. Stage 12 fixtures lock clipping/layered-background approximation behavior. |
| Blur/shadow helpers | `egui_expressive::{blur_image, soft_shadow, soft_inner_shadow, soft_glow, BlurQuality}` | Beta. CPU/bounded helpers with unit tests; backdrop/native compositor blur is not promised. |
| App-provided backdrop snapshots | `egui_expressive::{app_provided_backdrop_blur_report, app_provided_backdrop_blur_shape, BackdropCaptureRequest, BackdropSnapshot, BackdropSnapshotProvider, install_backdrop_snapshot_provider, load_backdrop_snapshot_provider}` | Beta. R100-001A exactness is limited to app-provided tightly packed RGBA snapshots for one egui context/surface, with `wgpu`, `init_gpu_effects_for_context(...)`, installed provider, `radius >= 1.0`, and in-budget dimensions. It is not host/native capture, browser `backdrop-filter`, or Tailwind `backdrop_blur` parity. |
| WGPU app-owned backdrop source contract | `egui_expressive::{AppOwnedBackdropSurfaceId, AppOwnedBackdropFrameId, AppOwnedBackdropAlphaMode, AppOwnedOffscreenBackdropSource, install_app_owned_offscreen_backdrop_source, load_app_owned_offscreen_backdrop_source, bind_app_owned_offscreen_backdrop_source_for_context, app_owned_offscreen_backdrop_blur_report, app_owned_offscreen_backdrop_blur_shape}` with `wgpu` | Experimental in R100-001B B3/B4. Host apps may register an app-owned same-frame offscreen `TextureView` source for one egui context/surface and bind it to the active `egui-wgpu` renderer before exact report/shape helpers can sample it. Exactness also requires the current installed source to be the same source allocation retained by the sidecar. Missing/unbound/stale/wrong metadata, same-metadata source reinstall without rebinding, and absent WebGPU support fail non-exact. B4 documents this as an app-owned WGPU support contract only: native/host capture, browser `backdrop-filter`, and default Tailwind backdrop blur remain out of scope until separate source contracts are approved. |
| Native backdrop adapter substrate | `egui_expressive::{NativeBackdropInitError, NativeBackdropPlatform}` with `native-backdrop` | Experimental / feature-gated. R100-001B common substrate freezes feature names and init-error vocabulary only. It does not capture native pixels, does not enable `HostFramebufferBackdrop`, and does not imply any platform provider is implemented. |
| Animation primitives | `egui_expressive::{Tween, Spring, Transition, AnimatedState, AnimatedF32, AnimatedColor}` | Beta. Reduced-motion policy is documented through accessibility guidance. |
| Layout and app-shell primitives | `egui_expressive::layout::*`, `egui_expressive::{DockPanel, ResizableSplit, TabBar, SidebarNav, StatusBar, AppShellLayoutState}` | Beta. Persistence/app-shell docs and examples exist; app code owns complex product-specific layout policy. |
| Forms v2 | `egui_expressive::forms::*`, crate-root `FormSchema`, `FormFieldDef`, `ValidationRule`, `InlineEditSession`, field widgets | Beta. Schema, validation, rich descriptors, platform handoff, and inline edit contracts are documented; native IME/platform certification remains app-owned. |
| Command/focus/undo/feedback | `egui_expressive::interaction::*`, crate-root `FocusScope`, `FeedbackQueue`, gesture and drag helpers | Beta. Architecture is documented and tested; app-level command catalogs and persistence policies remain app-owned. |
| State primitives | `egui_expressive::{InteractionState, StateMachine, StateSlot}`, `egui_expressive::state::*` | Beta. Shared state/persistence/audio bridge primitives are public and tested, but app-owned persistence schema and product-level state architecture remain outside this crate's guarantee. |
| Editor/canvas | `egui_expressive::editor::*`, crate-root `EditorCanvas`, `CanvasInteraction`, `SelectionModel`, `SnapGrid`, alignment/distribution helpers | Beta+. Generic canvas interactions and deterministic tests exist; broad editor-product behavior remains app-owned. |
| Surface/canvas scaling | `egui_expressive::{LargeCanvas, ViewportCuller}`, `egui_expressive::surface::*` | Beta. Large-canvas and viewport-culling models are public and performance-smoke covered; broad pan/zoom/editor products should use `editor` abstractions where appropriate. |
| Accessibility/platform descriptors | `egui_expressive::accessibility::*`, `egui_expressive::platform::*` | Beta. Pure descriptors and audit guidance; native OS integrations are dependency/app-owned unless future reviewed deps are added. |
| Style/theme/tokens/icons/typography | `egui_expressive::{DesignTokens, Theme, SemanticColors, Icon, TypeScale, TypeSpec}`, `style`, `theme`, `icons`, `typography` modules | Beta. Token and typography docs exist; Phase 4B reduced `typography/mod.rs` to a facade while deeper typography layout/render files remain justified exceptions. |
| Material 3 components | `egui_expressive::m3::*`, crate-root `M3*` types | Beta. Broad component family exists; Stable requires per-family release proof. |
| Core widget families | `egui_expressive::{Knob, Fader, Meter, StepGrid, TransportButton, TreeView, TimelineClip, ChannelStrip}` and `egui_expressive::widgets::*` | Experimental/Beta mix. Default-feature widgets are app-usable where examples/tests exist, but per-family docs/accessibility/performance proof is not uniform; prefer documented families and avoid treating all widget exports as Stable. |
| Scene/render model | `egui_expressive::scene::*`, crate-root `SceneNode`, `ArtboardScene`, `render_scene` | Beta. Scene model is public and tested; visual fidelity follows Tailwind/draw contract boundaries. |
| SVG/ASE and import/codegen | `egui_expressive::svg::*`, `figma`, `codegen`, `vectorize` modules | Beta. Useful for import/codegen workflows; fixture breadth and generated-output snapshots determine release confidence. |
| Visual diff and raster/vector validation | `egui_expressive::{diff_rgba_images, diff_image_paths, VisualDiffReport, VisualDiffConfig, vectorize_rgba_to_scene_nodes}`, `visual_diff`, `vectorize` modules | Beta. Release harnesses are Stable as validation entry points, but the public visual-diff/vectorize APIs follow the rubric's Beta classification until fixture breadth and API guarantees are separately promoted. |
| Compatibility adapters | `egui_expressive::compat::*`, `egui_expressive::swiftui::*` | Beta. Adapter vocabulary is public, but target-framework parity is not guaranteed beyond documented aliases. |

## Compatibility-Only Paths

| Compatibility path | Prefer instead | Status |
| --- | --- | --- |
| `egui_expressive::daw` feature namespace | `egui_expressive::widgets::*` for generic controls; `egui_expressive::widgets::editor_tools` for creative-editor primitives | Retained behind `daw` / `creative-editors` features for older app code. |
| `egui_expressive::widgets::daw_editors::*` | `egui_expressive::widgets::editor_tools` where generic editor primitives exist | Compatibility namespace; do not add new generic APIs here first. |
| `PianoRoll` alias | `PianoRollView` | Compatibility alias for the Stage 6 view-only rename decision. |
| DAW-flavored examples such as `neutraudio_shell`, `daw_strip`, and `step_sequencer` | General framework examples when learning app-shell/editor/data/form APIs | Examples remain build proofs and migration context, not the naming model for new public APIs. |

Compatibility paths remain usable until a major release or explicit migration plan removes them. New documentation should teach preferred paths first.

## Experimental / Diagnostic Paths

| Surface | Why experimental |
| --- | --- |
| `egui_expressive::debug` | Debug overlays are development aids, not release UI contracts. |
| `egui_expressive::devtools::*` | Runtime visual property editor and registry are diagnostic tooling; the file-size exception remains until the surface is promoted or split. |
| `egui_expressive::gpu::*` | Optional acceleration bridge behind `wgpu`; platform/device behavior and dependency policy are not default API guarantees. |
| Low-level parser/codegen internals beyond documented entrypoints | Large import/codegen modules are fixture-sensitive; prefer documented `generate_*`, `infer_layout`, sidecar diff, SVG scaffold, and Figma token entrypoints. |

## Feature-Gated Public API

| Feature | Public surface | Boundary |
| --- | --- | --- |
| `wgpu` / `gpu-effects` | `egui_expressive::gpu::{init_gpu_effects, init_gpu_effects_for_context, bind_app_owned_offscreen_backdrop_source_for_context, GpuEffectsResources, GpuCompositeCallback, GpuSourceLayerEffectCallback, GpuEffectSource, wgpu_source_layer_effect_report}` | Optional acceleration only. Default builds must not require it. Source-layer exact-effect claims are bounded to initialized callback resources and caller/library-owned solid rectangular RGBA sources using the Phase 9A/9B two-pass blur callback for blur and padded shadow subsets; R100-001A also permits exact `BackdropBlur` reports only for `GpuEffectSource::AppProvidedBackdropSnapshot`. R100-001B B3 permits app-owned `TextureView` backdrop sampling only through the renderer-bound binding API plus app-owned report/shape helpers; direct generic `GpuEffectSource::AppOwnedOffscreenBackdrop` reports remain non-exact without sidecar proof. Scene auto-selection requires `init_gpu_effects_for_context` so readiness is scoped to an egui context rather than process-global. Phase 9B shadow/glow exactness requires requested blur/radius at least `1.0`. These APIs must not imply host framebuffer capture, non-rect scene effects, zero/sub-pixel shadow exactness, browser `backdrop-filter`, or broad CSS backdrop/drop-shadow parity. |
| `native-backdrop` | `egui_expressive::{NativeBackdropInitError, NativeBackdropPlatform, NATIVE_BACKDROP_*_FEATURE}` | Common R100-001B substrate only. Platform-specific providers require later child blockers and target features such as `native-backdrop-x11`; no native capture feature is default-enabled. |
| `clip-mask` | `egui_expressive::clipped_shape_cpu` | CPU clipping helper; support is limited to documented draw/visual contracts. |
| `daw` / `creative-editors` | `egui_expressive::daw` | Compatibility namespace for existing domain-flavored code. New generic API should land under neutral modules first. |

## Public Re-Export Policy

- `src/lib.rs` may re-export ergonomic, documented entrypoints, but broad module ownership remains in the named modules.
- New crate-root re-exports should be added only when the item is a recommended app-facing type/function, not an implementation detail.
- New public modules start as Experimental unless the adding stage also ships docs, tests/examples, limitations, and Oracle approval for Beta or Stable.
- Approximate visual/platform behavior must link to a support contract before being advertised as app-facing API.
- If a public name is replaced, keep a compatibility alias when practical and document the preferred path in this file plus `docs/migration-guide.md`.

## Validation References

Stage 11 API status should be validated with:

- `cargo test --test api_surface_smoke` for representative compile-time reachability of preferred and beta-supported public paths.
- `cargo doc --lib --no-deps` for rustdoc buildability.
- `cargo test --all-targets` for existing API behavior and doctests/tests.
- `cargo build --examples` for public example imports.
- `cargo clippy --all-targets --all-features -- -D warnings` for feature-gated public surfaces.
- Release smoke tests listed in `docs/release-checklist.md` when changing docs that make release claims.
