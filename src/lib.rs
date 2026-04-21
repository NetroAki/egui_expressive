//! # egui_expressive
//!
//! Authoring-layer helpers for advanced custom UI in egui.
//!
//! `egui_expressive` sits on top of egui, reducing the boilerplate and low-level plumbing
//! required to build polished, advanced custom widgets. It does **not** replace egui's renderer,
//! layout engine, or widget system. Every shape, interaction, and pixel ultimately goes through
//! egui's existing `Painter`, `Shape`, `Ui`, and `Response` systems.
//!
//! ## Modules
//!
//! - [`draw`]       — Layered painter helpers and fluent shape builders
//! - [`style`]      — Visual state system (hover/press/select variants)
//! - [`state`]      — Typed persistent state and state machines
//! - [`interaction`]— Drag, pan/zoom, and gesture helpers
//! - [`animation`]  — Easing curves and spring physics
//! - [`surface`]    — Large canvas viewport culling (50k+ px)
//! - [`widgets`]    — Reusable controls (Knob, Fader, Meter, StepGrid, and more)
//! - [`debug`]      — Visual debugging overlays
//! - [`devtools`]   — Runtime visual property editor
//! - `daw`          — DAW-specific widgets (Fader, Meter, ChannelStrip, StepGrid, Waveform, etc.)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use egui_expressive::widgets::Knob;
//!
//! fn show(ui: &mut egui::Ui, gain: &mut f64) {
//!     ui.add(Knob::new(gain, -60.0..=6.0).size(48.0).label("GAIN"));
//! }
//! ```

pub mod animation;
pub mod blur;
pub mod codegen;
pub mod debug;
pub mod devtools;
pub mod draw;
pub mod figma;
pub mod icons;
pub mod m3;
pub mod svg;
pub mod swiftui;
pub mod theme;
pub mod typography;

// Re-export FigmaExportError for ergonomic error handling
pub use figma::FigmaExportError;
pub mod interaction;
pub mod layout;
pub mod state;
pub mod style;
pub mod surface;
pub mod tailwind;
pub mod widgets;

#[cfg(feature = "daw")]
pub mod daw;

// Re-export commonly used types at crate root
pub use animation::{
    transition_color, transition_f32, AnimatedColor, AnimatedF32, AnimatedState, AnimatedVec2,
    Spring, Transition, Tween,
};
pub use blur::{
    blur_image, blurred_image_shape, soft_glow, soft_inner_shadow, soft_shadow, BlurQuality,
};
pub use codegen::{
    diff_sidecars, generate_components_file, generate_mod_file, generate_multi_file_output,
    generate_rust, generate_state_file, generate_tokens_file, infer_layout, parse_json_sidecar,
    parse_naming, parse_svg_elements, svg_to_rust_scaffold, AppearanceFill, AppearanceStroke,
    ArtboardInfo, ArtboardOutput, ArtboardState, BlendMode, ComponentDef, EffectDef, EffectType,
    ElementType, EmitMode, GradientDef, GradientStop, GradientType, InferenceOptions,
    LayoutElement, LayoutNode, MultiFileOutput, NamingHint, PanelSide, SidecarChange, TextAlign,
    TextRun, ThirdPartyEffect,
};
pub use devtools::{DevToolsPanel, Prop, PropRegistry, PropValue};
#[cfg(feature = "clip-mask")]
pub use draw::clipped_shape_cpu;
pub use draw::composite_layers_gpu;
pub use draw::{
    blend_color, box_shadow, clip_to, clipped_rounded_rect, clipped_shape, composite_layers,
    dashed_path, dot_matrix, glow, gradient_rect, icon, icon_loop, icon_play, icon_record,
    icon_stop, inner_shadow, linear_gradient_rect, radial_gradient, radial_gradient_rect,
    scan_lines, transform_shape, vignette, with_opacity, zstack, zstack_layers, BlendLayer,
    ClipShape, DashPattern, GradientDir, LayeredPainter, RadialGradientDir, RichStroke,
    ShadowOffset, ShapeBuilder, StackAlign, StrokeCap, StrokeJoin, Transform2D,
};
pub use figma::design_tokens_from_json;
pub use icons::chars as icon_constants;
pub use icons::{Icon, IconButton, IconSize};
pub use interaction::{
    denormalize, normalize, DragAxis, DragDelta, FocusScope, LongPressEvent, LongPressGesture,
    PanZoom, SwipeDirection, SwipeEvent, SwipeGesture, TapEvent, TapGesture,
};
pub use layout::{
    aspect_ratio_fit, auto_layout, hrule, styled_frame, vrule, FlexAlign, FlexContainer,
    FlexJustify, FlexSize,
};
pub use state::{InteractionState, StateMachine, StateSlot};
pub use style::{
    apply_default_scrollbar_style, apply_scrollbar_style, fade_shapes, styled_text, with_alpha,
    AccentColors, DesignTokens, SpacingScale, SurfacePalette, TextStyle, TextStyles, VisualState,
    VisualVariant, WidgetTheme,
};
pub use surface::{LargeCanvas, ViewportCuller};
pub use svg::{
    ase_to_colors, parse_ase, parse_svg_color, svg_path_to_points, svg_path_to_shape,
    svg_to_shapes, AseError,
};
pub use swiftui::{GeometryProxy, Navigator, ScrollList, ViewModifier};
pub use tailwind::{
    Edges, FontWeight, Size, Tw, TW_0, TW_1, TW_10, TW_12, TW_16, TW_2, TW_20, TW_24, TW_3, TW_32,
    TW_4, TW_40, TW_48, TW_5, TW_6, TW_64, TW_8,
};
pub use theme::{border_rect, Border, Elevation, SemanticColors, Theme};
pub use typography::{
    render_text, TextDecoration, TextOverflow, TextTransform, TypeLabel, TypeScale, TypeSpec,
};
pub use widgets::{
    ChannelStrip, ClipKind, CollapsePanel, ContextMenuBuilder, ContinuousControl, DotState,
    DragNumber, DragReorder, Fader, FloatingPanel, Knob, KnobSize, KnobStyle, Meter, Orientation,
    ResizableSplit, Ruler, SplitAxis, StepGrid, TabBar, TimelineClip, ToggleDot, TransportButton,
    TransportKind, TreeNode, TreeView, VerticalDrag, Waveform,
};

// M3 Material Design 3 foundation modules
pub use m3::{
    blend_overlay,
    M3Badge,
    // Tier 1 components:
    M3Button,
    M3ButtonVariant,
    M3Card,
    M3CardVariant,
    M3Checkbox,
    M3Chip,
    M3ChipVariant,
    M3CircularProgress,
    M3ColorScheme,
    // Tier 3 components:
    M3Dialog,
    M3Divider,
    M3DropdownMenu,
    M3Elevation,
    M3Fab,
    M3FabColor,
    M3FabSize,
    M3FontWeight,
    M3LinearProgress,
    M3ListItem,
    M3NavItem,
    M3NavigationBar,
    M3NavigationRail,
    M3RadioButton,
    M3Slider,
    M3Snackbar,
    M3SnackbarState,
    M3Switch,
    // Tier 2 components:
    M3TextField,
    M3TextFieldVariant,
    M3TextStyle,
    M3Theme,
    M3Tooltip,
    M3TopAppBar,
    M3TopAppBarVariant,
    M3TypeScale,
};
