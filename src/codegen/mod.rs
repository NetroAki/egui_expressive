#![allow(dead_code)]

//! SVG layout inference and Rust scaffold code generation.
//!
//! This module provides a pure-Rust pipeline for converting SVG exports from
//! design tools (Illustrator, Figma) into egui layout code.

use egui::Color32;

// ============================================================================
// Types
// ============================================================================

/// Gradient stop definition.
#[derive(Clone, Debug)]
pub struct GradientStop {
    pub position: f32, // 0.0–1.0
    pub color: Color32,
}

/// Type of gradient.
#[derive(Clone, Debug, PartialEq)]
pub enum GradientType {
    Linear,
    Radial,
}

/// Gradient definition with angle and stops.
#[derive(Clone, Debug)]
pub struct GradientDef {
    pub gradient_type: GradientType,
    pub angle_deg: f32,
    pub stops: Vec<GradientStop>,
}

/// Blend mode for compositing.
#[derive(Clone, Debug, PartialEq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
}

/// Effect type for shadows/glow.
#[derive(Clone, Debug, PartialEq)]
pub enum EffectType {
    DropShadow,
    InnerShadow,
    Glow,
}

/// Effect definition.
#[derive(Clone, Debug)]
pub struct EffectDef {
    pub effect_type: EffectType,
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub color: Color32,
}

/// Text alignment options.
#[derive(Clone, Debug, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justified,
}

/// A parsed element from SVG/Illustrator export.
#[derive(Clone, Debug)]
pub struct LayoutElement {
    pub id: String,
    pub el_type: ElementType,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub fill: Option<Color32>,
    pub stroke: Option<(f32, Color32)>, // (width, color)
    pub text: Option<String>,
    pub text_size: Option<f32>,
    pub children: Vec<LayoutElement>,
    // Extended fields
    pub opacity: f32,
    pub rotation_deg: f32,
    pub corner_radius: f32,
    pub gradient: Option<GradientDef>,
    pub blend_mode: BlendMode,
    pub effects: Vec<EffectDef>,
    pub stroke_dash: Option<Vec<f32>>,
    pub clip_children: bool,
    pub text_align: Option<TextAlign>,
    pub letter_spacing: Option<f32>,
    pub line_height: Option<f32>,
}

impl LayoutElement {
    pub fn new(id: String, el_type: ElementType, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            id,
            el_type,
            x,
            y,
            w,
            h,
            fill: None,
            stroke: None,
            text: None,
            text_size: None,
            children: vec![],
            opacity: 1.0,
            rotation_deg: 0.0,
            corner_radius: 0.0,
            gradient: None,
            blend_mode: BlendMode::Normal,
            effects: vec![],
            stroke_dash: None,
            clip_children: false,
            text_align: None,
            letter_spacing: None,
            line_height: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElementType {
    Group,
    Shape,
    Path,
    Text,
    Image,
    Unknown,
}

/// Inferred layout node — the intermediate representation between raw elements and code.
#[derive(Clone, Debug)]
pub enum LayoutNode {
    Row {
        gap: f32,
        children: Vec<LayoutNode>,
        bg: Option<Color32>,
        id: String,
    },
    Column {
        gap: f32,
        children: Vec<LayoutNode>,
        bg: Option<Color32>,
        id: String,
    },
    ScrollArea {
        vertical: bool,
        children: Vec<LayoutNode>,
        id: String,
    },
    Panel {
        side: PanelSide,
        children: Vec<LayoutNode>,
        id: String,
    },
    Card {
        children: Vec<LayoutNode>,
        bg: Color32,
        rounding: f32,
        id: String,
    },
    Button {
        label: String,
        id: String,
    },
    Label {
        text: String,
        size: f32,
        color: Option<Color32>,
        id: String,
    },
    TextEdit {
        placeholder: String,
        id: String,
    },
    Separator {
        id: String,
    },
    Spacer {
        size: f32,
        id: String,
    },
    Badge {
        text: String,
        id: String,
    },
    Icon {
        name: String,
        id: String,
    },
    Shape {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill: Color32,
        id: String,
    },
    Image {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        id: String,
    },
    Unknown {
        id: String,
        comment: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelSide {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

// ============================================================================
// Naming Convention Parser
// ============================================================================

/// Parse Illustrator/Figma layer naming conventions into layout hints.
pub fn parse_naming(name: &str) -> NamingHint {
    let lower = name.to_lowercase();

    // Check for gap-N first (might be standalone)
    if let Some(gap_idx) = lower.find("gap-") {
        let after = &lower[gap_idx + 4..];
        if let Some(end_idx) = after
            .find(char::is_whitespace)
            .or_else(|| Some(after.len()))
        {
            let num_str = &after[..end_idx];
            if let Ok(gap) = num_str.parse::<f32>() {
                return NamingHint::Gap(gap);
            }
        }
        // Also try just parsing the first number
        let mut num_str = String::new();
        for ch in after.chars() {
            if ch.is_ascii_digit() || ch == '.' {
                num_str.push(ch);
            } else {
                break;
            }
        }
        if let Ok(gap) = num_str.parse::<f32>() {
            return NamingHint::Gap(gap);
        }
    }

    // Row patterns: row-*, hstack-*
    if lower.starts_with("row-") || lower.starts_with("hstack-") {
        let label = extract_label(name);
        return NamingHint::Row(label);
    }

    // Column patterns: col-*, vstack-*
    if lower.starts_with("col-") || lower.starts_with("vstack-") {
        let label = extract_label(name);
        return NamingHint::Column(label);
    }

    // Panel patterns: panel-left, panel-right, panel-top, panel-bottom, panel-center
    if lower.starts_with("panel-") {
        let side = if lower.contains("left") {
            PanelSide::Left
        } else if lower.contains("right") {
            PanelSide::Right
        } else if lower.contains("top") {
            PanelSide::Top
        } else if lower.contains("bottom") {
            PanelSide::Bottom
        } else {
            PanelSide::Center
        };
        return NamingHint::Panel(side);
    }

    // Grid pattern: grid-*
    if lower.starts_with("grid-") {
        let label = extract_label(name);
        return NamingHint::Grid(label);
    }

    // Scroll pattern: scroll-*
    if lower.starts_with("scroll-") {
        let label = extract_label(name);
        return NamingHint::Scroll(label);
    }

    // Button patterns: btn-*, button-*
    if lower.starts_with("btn-") || lower.starts_with("button-") {
        let label = extract_label(name);
        return NamingHint::Button(label);
    }

    // TextEdit patterns: input-*, field-*, textedit-*
    if lower.starts_with("input-") || lower.starts_with("field-") || lower.starts_with("textedit-")
    {
        let label = extract_label(name);
        return NamingHint::TextEdit(label);
    }

    // Label patterns: label-*, text-*
    if lower.starts_with("label-") || lower.starts_with("text-") {
        let label = extract_label(name);
        return NamingHint::Label(label);
    }

    // Image patterns: img-*, image-*
    if lower.starts_with("img-") || lower.starts_with("image-") {
        let label = extract_label(name);
        return NamingHint::Image(label);
    }

    // Icon pattern: icon-*
    if lower.starts_with("icon-") {
        let label = extract_label(name);
        return NamingHint::Icon(label);
    }

    // Card pattern: card-*
    if lower.starts_with("card-") {
        let label = extract_label(name);
        return NamingHint::Card(label);
    }

    // Divider: divider, divider-*
    if lower.starts_with("divider") {
        return NamingHint::Divider;
    }

    // Spacer: spacer, spacer-*
    if lower.starts_with("spacer") {
        return NamingHint::Spacer;
    }

    // Badge pattern: badge-*
    if lower.starts_with("badge-") {
        let label = extract_label(name);
        return NamingHint::Badge(label);
    }

    // Chip pattern: chip-*
    if lower.starts_with("chip-") {
        let label = extract_label(name);
        return NamingHint::Chip(label);
    }

    // Toggle pattern: toggle-*, switch-*
    if lower.starts_with("toggle-") || lower.starts_with("switch-") {
        let label = extract_label(name);
        return NamingHint::Toggle(label);
    }

    // Slider pattern: slider-*, knob-*
    if lower.starts_with("slider-") || lower.starts_with("knob-") {
        let label = extract_label(name);
        return NamingHint::Slider(label);
    }

    NamingHint::None
}

fn extract_label(name: &str) -> String {
    // Extract the part after the prefix, trimmed
    let lower = name.to_lowercase();
    let prefixes = [
        "row-",
        "hstack-",
        "col-",
        "vstack-",
        "panel-",
        "grid-",
        "scroll-",
        "btn-",
        "button-",
        "input-",
        "field-",
        "textedit-",
        "label-",
        "text-",
        "img-",
        "image-",
        "icon-",
        "card-",
        "badge-",
        "chip-",
        "toggle-",
        "switch-",
        "slider-",
        "knob-",
    ];

    for prefix in &prefixes {
        if lower.starts_with(prefix) {
            let label = &name[prefix.len()..];
            return label.trim().to_string();
        }
    }

    name.trim().to_string()
}

#[derive(Clone, Debug, PartialEq)]
pub enum NamingHint {
    Row(String),
    Column(String),
    Panel(PanelSide),
    Grid(String),
    Scroll(String),
    Button(String),
    TextEdit(String),
    Label(String),
    Image(String),
    Icon(String),
    Card(String),
    Divider,
    Spacer,
    Badge(String),
    Chip(String),
    Toggle(String),
    Slider(String),
    Gap(f32),
    None,
}

// ============================================================================
// Gap Inference
// ============================================================================

/// Compute the median gap between a sorted list of elements along the X axis.
pub fn infer_horizontal_gap(elements: &[LayoutElement]) -> f32 {
    if elements.len() < 2 {
        return 0.0;
    }

    let mut sorted = elements.to_vec();
    sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

    let mut gaps: Vec<f32> = Vec::new();
    for i in 1..sorted.len() {
        let gap = sorted[i].x - (sorted[i - 1].x + sorted[i - 1].w);
        if gap > 0.0 {
            gaps.push(gap);
        }
    }

    if gaps.is_empty() {
        return 0.0;
    }

    median(&gaps)
}

/// Compute the median gap between a sorted list of elements along the Y axis.
pub fn infer_vertical_gap(elements: &[LayoutElement]) -> f32 {
    if elements.len() < 2 {
        return 0.0;
    }

    let mut sorted = elements.to_vec();
    sorted.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

    let mut gaps: Vec<f32> = Vec::new();
    for i in 1..sorted.len() {
        let gap = sorted[i].y - (sorted[i - 1].y + sorted[i - 1].h);
        if gap > 0.0 {
            gaps.push(gap);
        }
    }

    if gaps.is_empty() {
        return 0.0;
    }

    median(&gaps)
}

fn median(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

/// Cluster elements into rows by Y-overlap.
/// Returns Vec<Vec<LayoutElement>> where each inner Vec is one row.
pub fn cluster_into_rows(
    elements: &[LayoutElement],
    overlap_threshold: f32,
) -> Vec<Vec<LayoutElement>> {
    if elements.is_empty() {
        return vec![];
    }

    let mut sorted = elements.to_vec();
    // Sort by Y position first, then by X for stable ordering
    sorted.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap()
            .then(a.x.partial_cmp(&b.x).unwrap())
    });

    let mut rows: Vec<Vec<LayoutElement>> = Vec::new();
    let mut current_row: Vec<LayoutElement> = vec![sorted[0].clone()];

    for i in 1..sorted.len() {
        let elem = &sorted[i];
        let prev_in_row = current_row.last().unwrap();

        // Calculate vertical overlap between this element and the current row
        let row_top = current_row[0].y;
        let row_bottom = current_row
            .iter()
            .map(|e| e.y + e.h)
            .fold(f32::NEG_INFINITY, f32::max);
        let row_height = row_bottom - row_top;

        let elem_top = elem.y;
        let elem_bottom = elem.y + elem.h;

        // Calculate overlap as fraction of element height
        let overlap_start = elem_top.max(row_top);
        let overlap_end = elem_bottom.min(row_bottom);
        let overlap = (overlap_end - overlap_start).max(0.0);
        let overlap_fraction = if elem.h > 0.0 { overlap / elem.h } else { 0.0 };

        // Also check if element is close to the row (within tolerance)
        let tolerance = row_height * overlap_threshold.max(0.3);
        let vertical_distance = (elem_top - row_bottom)
            .abs()
            .min((row_top - elem_bottom).abs());

        if overlap_fraction >= overlap_threshold || vertical_distance <= tolerance {
            current_row.push(elem.clone());
        } else {
            rows.push(current_row);
            current_row = vec![elem.clone()];
        }
    }

    if !current_row.is_empty() {
        rows.push(current_row);
    }

    rows
}

// ============================================================================
// Layout Inference
// ============================================================================

/// Emit mode for code generation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EmitMode {
    /// Absolute positioning with painter calls.
    Absolute,
    /// Responsive layout using flex_row!/flex_col! macros.
    Responsive,
    /// Hybrid mode - absolute for complex layouts, responsive for simple ones.
    Hybrid,
}

#[derive(Clone, Debug)]
pub struct InferenceOptions {
    pub use_naming_conventions: bool,
    pub infer_gaps: bool,
    pub gap_tolerance: f32,
    pub row_overlap_threshold: f32,
    pub emit_mode: EmitMode,
}

impl Default for InferenceOptions {
    fn default() -> Self {
        Self {
            use_naming_conventions: true,
            infer_gaps: true,
            gap_tolerance: 4.0,
            row_overlap_threshold: 0.5,
            emit_mode: EmitMode::Responsive,
        }
    }
}

/// Infer layout structure from a flat list of elements.
/// Returns a tree of LayoutNodes.
pub fn infer_layout(elements: &[LayoutElement], options: &InferenceOptions) -> Vec<LayoutNode> {
    if elements.is_empty() {
        return vec![];
    }

    let mut nodes: Vec<LayoutNode> = Vec::new();

    for elem in elements {
        let node = infer_element(elem, options);
        nodes.push(node);
    }

    // Post-process to detect panels and cards
    let artboard_w = elements.iter().map(|e| e.x + e.w).fold(0.0f32, f32::max);
    let artboard_h = elements.iter().map(|e| e.y + e.h).fold(0.0f32, f32::max);

    detect_panels_and_cards(&mut nodes, artboard_w, artboard_h);

    nodes
}

fn infer_element(elem: &LayoutElement, options: &InferenceOptions) -> LayoutNode {
    // Check naming convention
    if options.use_naming_conventions {
        let hint = parse_naming(&elem.id);
        match hint {
            NamingHint::Row(label) => {
                let children = infer_children(&elem.children, options);
                let gap = if options.infer_gaps {
                    infer_horizontal_gap(&elem.children)
                } else {
                    8.0
                };
                return LayoutNode::Row {
                    gap,
                    children,
                    bg: elem.fill,
                    id: label,
                };
            }
            NamingHint::Column(label) => {
                let children = infer_children(&elem.children, options);
                let gap = if options.infer_gaps {
                    infer_vertical_gap(&elem.children)
                } else {
                    8.0
                };
                return LayoutNode::Column {
                    gap,
                    children,
                    bg: elem.fill,
                    id: label,
                };
            }
            NamingHint::Panel(side) => {
                let children = infer_children(&elem.children, options);
                return LayoutNode::Panel {
                    side,
                    children,
                    id: elem.id.clone(),
                };
            }
            NamingHint::Card(label) => {
                let children = infer_children(&elem.children, options);
                return LayoutNode::Card {
                    children,
                    bg: elem.fill.unwrap_or(Color32::from_gray(40)),
                    rounding: 8.0,
                    id: label,
                };
            }
            NamingHint::Scroll(label) => {
                let children = infer_children(&elem.children, options);
                return LayoutNode::ScrollArea {
                    vertical: true,
                    children,
                    id: label,
                };
            }
            NamingHint::Button(label) => {
                return LayoutNode::Button {
                    label: if label.is_empty() {
                        elem.text.clone().unwrap_or_else(|| "Button".to_string())
                    } else {
                        label
                    },
                    id: elem.id.clone(),
                };
            }
            NamingHint::Label(label) => {
                return LayoutNode::Label {
                    text: if label.is_empty() {
                        elem.text.clone().unwrap_or_default()
                    } else {
                        label
                    },
                    size: elem.text_size.unwrap_or(14.0),
                    color: elem.fill,
                    id: elem.id.clone(),
                };
            }
            NamingHint::TextEdit(label) => {
                return LayoutNode::TextEdit {
                    placeholder: if label.is_empty() {
                        elem.text.clone().unwrap_or_default()
                    } else {
                        label
                    },
                    id: elem.id.clone(),
                };
            }
            NamingHint::Icon(label) => {
                return LayoutNode::Icon {
                    name: label,
                    id: elem.id.clone(),
                };
            }
            NamingHint::Badge(label) => {
                return LayoutNode::Badge {
                    text: if label.is_empty() {
                        elem.text.clone().unwrap_or_default()
                    } else {
                        label
                    },
                    id: elem.id.clone(),
                };
            }
            NamingHint::Divider => {
                return LayoutNode::Separator {
                    id: elem.id.clone(),
                };
            }
            NamingHint::Spacer => {
                return LayoutNode::Spacer {
                    size: elem.h.max(elem.w).max(8.0),
                    id: elem.id.clone(),
                };
            }
            NamingHint::Gap(size) => {
                return LayoutNode::Spacer {
                    size,
                    id: elem.id.clone(),
                };
            }
            NamingHint::Image(label) => {
                return LayoutNode::Image {
                    x: elem.x,
                    y: elem.y,
                    w: elem.w,
                    h: elem.h,
                    id: label,
                };
            }
            NamingHint::None => {}
            _ => {}
        }
    }

    // Handle by element type
    match elem.el_type {
        ElementType::Group => {
            // If it's a group with children, infer layout from children
            if !elem.children.is_empty() {
                let children = infer_children(&elem.children, options);

                // Determine if it's primarily horizontal or vertical
                if is_horizontal_group(&elem.children) {
                    let gap = if options.infer_gaps {
                        infer_horizontal_gap(&elem.children)
                    } else {
                        8.0
                    };
                    LayoutNode::Row {
                        gap,
                        children,
                        bg: elem.fill,
                        id: elem.id.clone(),
                    }
                } else {
                    let gap = if options.infer_gaps {
                        infer_vertical_gap(&elem.children)
                    } else {
                        8.0
                    };
                    LayoutNode::Column {
                        gap,
                        children,
                        bg: elem.fill,
                        id: elem.id.clone(),
                    }
                }
            } else {
                LayoutNode::Unknown {
                    id: elem.id.clone(),
                    comment: "empty group".to_string(),
                }
            }
        }
        ElementType::Shape => LayoutNode::Shape {
            x: elem.x,
            y: elem.y,
            w: elem.w,
            h: elem.h,
            fill: elem.fill.unwrap_or(Color32::from_gray(128)),
            id: elem.id.clone(),
        },
        ElementType::Text => LayoutNode::Label {
            text: elem.text.clone().unwrap_or_default(),
            size: elem.text_size.unwrap_or(14.0),
            color: elem.fill,
            id: elem.id.clone(),
        },
        ElementType::Image => LayoutNode::Image {
            x: elem.x,
            y: elem.y,
            w: elem.w,
            h: elem.h,
            id: elem.id.clone(),
        },
        ElementType::Path => {
            // Paths get rendered as shapes
            LayoutNode::Shape {
                x: elem.x,
                y: elem.y,
                w: elem.w.max(1.0),
                h: elem.h.max(1.0),
                fill: elem.fill.unwrap_or(Color32::TRANSPARENT),
                id: elem.id.clone(),
            }
        }
        ElementType::Unknown => LayoutNode::Unknown {
            id: elem.id.clone(),
            comment: format!("{:?}", elem.el_type),
        },
    }
}

fn infer_children(children: &[LayoutElement], options: &InferenceOptions) -> Vec<LayoutNode> {
    if children.is_empty() {
        return vec![];
    }

    // Cluster children into rows
    let rows = cluster_into_rows(children, options.row_overlap_threshold);

    let mut nodes: Vec<LayoutNode> = Vec::new();

    for row in rows {
        if row.len() == 1 {
            // Single element, no need to wrap in row/column
            nodes.push(infer_element(&row[0], options));
        } else {
            // Multiple elements in a row
            let gap = if options.infer_gaps {
                infer_horizontal_gap(&row)
            } else {
                8.0
            };

            let row_children: Vec<LayoutNode> = row
                .iter()
                .map(|elem| infer_element(elem, options))
                .collect();

            // Determine if this should be a Row or Column based on aspect ratio
            let is_vertical = is_vertical_group(&row);
            if is_vertical {
                let vgap = if options.infer_gaps {
                    infer_vertical_gap(&row)
                } else {
                    8.0
                };
                nodes.push(LayoutNode::Column {
                    gap: vgap,
                    children: row_children,
                    bg: None,
                    id: format!("col_{}", nodes.len()),
                });
            } else {
                nodes.push(LayoutNode::Row {
                    gap,
                    children: row_children,
                    bg: None,
                    id: format!("row_{}", nodes.len()),
                });
            }
        }
    }

    nodes
}

fn is_horizontal_group(elements: &[LayoutElement]) -> bool {
    if elements.len() < 2 {
        return false;
    }

    let total_width: f32 = elements.iter().map(|e| e.w).sum();
    let total_height: f32 = elements.iter().map(|e| e.h).sum::<f32>() / elements.len() as f32;

    // Horizontal if total width is significantly greater than total height
    total_width > total_height * 1.5
}

fn is_vertical_group(elements: &[LayoutElement]) -> bool {
    if elements.len() < 2 {
        return false;
    }

    // Check if elements are stacked vertically
    let sorted_by_y: Vec<_> = elements.iter().collect();
    let sorted_by_x: Vec<_> = elements.iter().collect();

    let mut y_variance = 0.0f32;
    let mut x_variance = 0.0f32;
    let y_mean = elements.iter().map(|e| e.y).sum::<f32>() / elements.len() as f32;
    let x_mean = elements.iter().map(|e| e.x).sum::<f32>() / elements.len() as f32;

    for e in elements {
        y_variance += (e.y - y_mean).powi(2);
        x_variance += (e.x - x_mean).powi(2);
    }
    y_variance /= elements.len() as f32;
    x_variance /= elements.len() as f32;

    // More vertical variance means vertical stacking
    y_variance > x_variance
}

fn detect_panels_and_cards(nodes: &mut [LayoutNode], _artboard_w: f32, _artboard_h: f32) {
    // This is a post-processing step that would identify panels and cards
    // based on spanning elements. For now, this is handled during inference
    // via naming conventions.
    // A full implementation would look for elements that span >80% of the artboard.
}

// ============================================================================
// Rust Code Generator
// ============================================================================

/// Generate a complete Rust source file from a list of LayoutNodes.
/// Returns a String containing valid Rust code.
pub fn generate_rust(
    fn_name: &str,
    artboard_w: f32,
    artboard_h: f32,
    nodes: &[LayoutNode],
    bg_color: Option<Color32>,
) -> String {
    let mut output = String::new();

    output.push_str("// Auto-generated by egui_expressive\n");
    output.push_str(&format!(
        "// Artboard: \"{}\" ({} × {} px)\n",
        fn_name, artboard_w, artboard_h
    ));
    output.push_str("\n#[allow(unused_variables, dead_code)]\n");
    output.push_str(&format!(
        "pub fn draw_{}(ui: &mut egui::Ui) {{\n",
        sanitize_fn_name(fn_name)
    ));
    output.push_str("    let origin = ui.cursor().min;\n");
    output.push_str("    let painter = ui.painter();\n");
    output.push_str("\n");

    // Background
    if let Some(bg) = bg_color {
        output.push_str("    // Background\n");
        output.push_str(&format!(
            "    painter.rect_filled(\n        egui::Rect::from_min_size(origin, egui::vec2({:.1}, {:.1})),\n",
            artboard_w, artboard_h
        ));
        output.push_str("        0.0,\n");
        output.push_str(&format!(
            "        egui::Color32::from_rgb({}, {}, {}),\n",
            bg.r(),
            bg.g(),
            bg.b()
        ));
        output.push_str("    );\n");
        output.push_str("\n");
    }

    // Generate code for each top-level node
    for node in nodes {
        output.push_str(&generate_node(node, 4));
    }

    output.push_str("}\n");

    output
}

fn sanitize_fn_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Generate Rust code for a single LayoutNode (recursive).
pub fn generate_node(node: &LayoutNode, indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    let mut output = String::new();

    match node {
        LayoutNode::Row {
            gap,
            children,
            bg,
            id,
        } => {
            if let Some(bg_color) = bg {
                output.push_str(&format!(
                    "{}// Row: {}\n{}ui.horizontal(|ui| {{\n{}{}ui.spacing_mut().item_spacing.x = {:.1};\n",
                    indent_str, id, indent_str, indent_str, indent_str, gap
                ));
                output.push_str(&format!(
                    "{}{}ui.painter().rect_filled(\n{}{}{}egui::Rect::from_min_size(ui.cursor().min, egui::vec2(\n",
                    indent_str, indent_str, indent_str, indent_str, indent_str
                ));
                // Calculate row bounds
                let row_w: f32 = children.iter().map(|c| get_node_width(c)).sum();
                let row_h: f32 = children
                    .iter()
                    .map(|c| get_node_height(c))
                    .fold(0.0f32, f32::max);
                output.push_str(&format!(
                    "{}{}{}{:.1}, {:.1}),\n",
                    indent_str, indent_str, indent_str, row_w, row_h
                ));
                output.push_str(&format!("{}{}{}),\n", indent_str, indent_str, indent_str));
                output.push_str(&format!("{}{}{}0.0,\n", indent_str, indent_str, indent_str));
                output.push_str(&format!(
                    "{}{}{}egui::Color32::from_rgb({}, {}, {}),\n",
                    indent_str,
                    indent_str,
                    indent_str,
                    bg_color.r(),
                    bg_color.g(),
                    bg_color.b()
                ));
                output.push_str(&format!("{}{}{});\n", indent_str, indent_str, indent_str));
                for child in children {
                    output.push_str(&generate_node(child, indent + 4));
                }
                output.push_str(&format!("{}}});\n", indent_str));
            } else {
                output.push_str(&format!(
                    "{}// Row: {}\n{}ui.horizontal(|ui| {{\n{}{}ui.spacing_mut().item_spacing.x = {:.1};\n",
                    indent_str, id, indent_str, indent_str, indent_str, gap
                ));
                for child in children {
                    output.push_str(&generate_node(child, indent + 4));
                }
                output.push_str(&format!("{}}});\n", indent_str));
            }
        }
        LayoutNode::Column {
            gap,
            children,
            bg,
            id,
        } => {
            if let Some(bg_color) = bg {
                output.push_str(&format!(
                    "{}// Column: {}\n{}ui.vertical(|ui| {{\n{}{}ui.spacing_mut().item_spacing.y = {:.1};\n",
                    indent_str, id, indent_str, indent_str, indent_str, gap
                ));
                output.push_str(&format!(
                    "{}{}ui.painter().rect_filled(\n{}{}{}egui::Rect::from_min_size(ui.cursor().min, egui::vec2(\n",
                    indent_str, indent_str, indent_str, indent_str, indent_str
                ));
                let col_w: f32 = children
                    .iter()
                    .map(|c| get_node_width(c))
                    .fold(0.0f32, f32::max);
                let col_h: f32 = children.iter().map(|c| get_node_height(c)).sum();
                output.push_str(&format!(
                    "{}{}{}{:.1}, {:.1}),\n",
                    indent_str, indent_str, indent_str, col_w, col_h
                ));
                output.push_str(&format!("{}{}{}),\n", indent_str, indent_str, indent_str));
                output.push_str(&format!("{}{}{}0.0,\n", indent_str, indent_str, indent_str));
                output.push_str(&format!(
                    "{}{}{}egui::Color32::from_rgb({}, {}, {}),\n",
                    indent_str,
                    indent_str,
                    indent_str,
                    bg_color.r(),
                    bg_color.g(),
                    bg_color.b()
                ));
                output.push_str(&format!("{}{}{});\n", indent_str, indent_str, indent_str));
                for child in children {
                    output.push_str(&generate_node(child, indent + 4));
                }
                output.push_str(&format!("{}}});\n", indent_str));
            } else {
                output.push_str(&format!(
                    "{}// Column: {}\n{}ui.vertical(|ui| {{\n{}{}ui.spacing_mut().item_spacing.y = {:.1};\n",
                    indent_str, id, indent_str, indent_str, indent_str, gap
                ));
                for child in children {
                    output.push_str(&generate_node(child, indent + 4));
                }
                output.push_str(&format!("{}}});\n", indent_str));
            }
        }
        LayoutNode::ScrollArea {
            vertical,
            children,
            id,
        } => {
            output.push_str(&format!(
                "{}// ScrollArea: {}\n{}ui.scroll_area({})\n",
                indent_str,
                id,
                indent_str,
                if *vertical { "true" } else { "false" }
            ));
            output.push_str(&format!("{}.show(|ui| {{\n", indent_str));
            for child in children {
                output.push_str(&generate_node(child, indent + 4));
            }
            output.push_str(&format!("{}}});\n", indent_str));
        }
        LayoutNode::Panel { side, children, id } => {
            output.push_str(&format!("{}// Panel: {:?} - {}\n", indent_str, side, id));
            let (width, height) = calculate_panel_dimensions(children, *side);
            output.push_str(&format!(
                "{}ui.allocate_ui(egui::vec2({:.1}, {:.1}), |ui| {{\n",
                indent_str, width, height
            ));
            for child in children {
                output.push_str(&generate_node(child, indent + 4));
            }
            output.push_str(&format!("{}}});\n", indent_str));
        }
        LayoutNode::Card {
            children,
            bg,
            rounding,
            id,
        } => {
            output.push_str(&format!("{}// Card: {}\n", indent_str, id));
            let (w, h) = calculate_card_dimensions(children);
            output.push_str(&format!(
                "{}let card_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2({:.1}, {:.1}));\n",
                indent_str, w, h
            ));
            output.push_str(&format!(
                "{}ui.painter().rect_filled(card_rect, {:.1}, egui::Color32::from_rgb({}, {}, {}));\n",
                indent_str, rounding, bg.r(), bg.g(), bg.b()
            ));
            output.push_str(&format!("{}ui.vertical(|ui| {{\n", indent_str));
            for child in children {
                output.push_str(&generate_node(child, indent + 4));
            }
            output.push_str(&format!("{}}});\n", indent_str));
        }
        LayoutNode::Button { label, id } => {
            output.push_str(&format!(
                "{}// Button: {}\n{}if ui.button(\"{}\").clicked() {{\n{}{}// TODO: handle click\n{}{}}}\n",
                indent_str, id, indent_str, label, indent_str, indent_str, indent_str, indent_str
            ));
        }
        LayoutNode::Label {
            text,
            size,
            color,
            id,
        } => {
            let color_str = if let Some(c) = color {
                format!("egui::Color32::from_rgb({}, {}, {})", c.r(), c.g(), c.b())
            } else {
                "egui::Color32::from_gray(200)".to_string()
            };
            output.push_str(&format!(
                "{}// Label: {}\n{}ui.label(egui::RichText::new(\"{}\").size({:.1}).color({}));\n",
                indent_str,
                id,
                indent_str,
                text.replace('"', "\\\""),
                size,
                color_str
            ));
        }
        LayoutNode::TextEdit { placeholder, id } => {
            output.push_str(&format!(
                "{}// TextEdit: {}\n{}ui.text_edit_singleline(&mut {})",
                indent_str,
                id,
                indent_str,
                format!("{}_text", id.replace('-', "_"))
            ));
            if !placeholder.is_empty() {
                output.push_str(&format!(
                    ".hint_text(\"{}\")",
                    placeholder.replace('"', "\\\"")
                ));
            }
            output.push_str(";\n");
        }
        LayoutNode::Separator { id } => {
            output.push_str(&format!(
                "{}// Separator: {}\n{}ui.separator();\n",
                indent_str, id, indent_str
            ));
        }
        LayoutNode::Spacer { size, id } => {
            output.push_str(&format!(
                "{}// Spacer: {}\n{}ui.add_space({:.1});\n",
                indent_str, id, indent_str, size
            ));
        }
        LayoutNode::Badge { text, id } => {
            output.push_str(&format!(
                "{}// Badge: {}\n{}ui.label(egui::RichText::new(\"{}\")\n",
                indent_str,
                id,
                indent_str,
                text.replace('"', "\\\"")
            ));
            output.push_str(&format!("{}.small()\n", indent_str));
            output.push_str(&format!(
                "{}.color(egui::Color32::from_rgb(100, 200, 255)));\n",
                indent_str
            ));
        }
        LayoutNode::Icon { name, id } => {
            output.push_str(&format!(
                "{}// Icon: {} - {}\n{}ui.label(egui::RichText::new(\"⬛\").size(24.0));\n",
                indent_str, id, name, indent_str
            ));
        }
        LayoutNode::Shape {
            x,
            y,
            w,
            h,
            fill,
            id,
        } => {
            output.push_str(&format!(
                "{}// Shape: {}\n{}painter.rect_filled(\n{}{}egui::Rect::from_min_size(\n{}{}{}egui::Pos2::new({:.1}, {:.1}),\n{}{}{}egui::vec2({:.1}, {:.1})\n{}{}),\n{}{}{}{:.1},\n{}{}{}egui::Color32::from_rgb({}, {}, {}),\n{}{});\n",
                indent_str, id, indent_str,
                indent_str, indent_str,
                indent_str, indent_str, indent_str, x, y,
                indent_str, indent_str, indent_str, w, h,
                indent_str, indent_str,
                indent_str, indent_str, indent_str, 0.0,
                indent_str, indent_str, indent_str, fill.r(), fill.g(), fill.b(),
                indent_str, indent_str
            ));
        }
        LayoutNode::Image { x, y, w, h, id } => {
            output.push_str(&format!(
                "{}// Image: {}\n{}// TODO: Load and display image at ({:.1}, {:.1}) size ({:.1}, {:.1})\n",
                indent_str, id, indent_str, x, y, w, h
            ));
        }
        LayoutNode::Unknown { id, comment } => {
            output.push_str(&format!("{}// Unknown: {} ({})\n", indent_str, id, comment));
        }
    }

    output
}

fn get_node_width(node: &LayoutNode) -> f32 {
    match node {
        LayoutNode::Shape { w, .. } => *w,
        LayoutNode::Image { w, .. } => *w,
        LayoutNode::Card { children, .. } => {
            children.iter().map(get_node_width).fold(0.0f32, f32::max)
        }
        LayoutNode::Row { children, .. } => children.iter().map(get_node_width).sum(),
        LayoutNode::Column { children, .. } => {
            children.iter().map(get_node_width).fold(0.0f32, f32::max)
        }
        LayoutNode::Panel { children, .. } => {
            children.iter().map(get_node_width).fold(0.0f32, f32::max)
        }
        LayoutNode::ScrollArea { children, .. } => {
            children.iter().map(get_node_width).fold(0.0f32, f32::max)
        }
        LayoutNode::Spacer { size, .. } => *size,
        _ => 100.0,
    }
}

fn get_node_height(node: &LayoutNode) -> f32 {
    match node {
        LayoutNode::Shape { h, .. } => *h,
        LayoutNode::Image { h, .. } => *h,
        LayoutNode::Card { children, .. } => children.iter().map(get_node_height).sum(),
        LayoutNode::Row { children, .. } => {
            children.iter().map(get_node_height).fold(0.0f32, f32::max)
        }
        LayoutNode::Column { children, .. } => children.iter().map(get_node_height).sum(),
        LayoutNode::Panel { children, .. } => {
            children.iter().map(get_node_height).fold(0.0f32, f32::max)
        }
        LayoutNode::ScrollArea { children, .. } => {
            children.iter().map(get_node_height).fold(0.0f32, f32::max)
        }
        LayoutNode::Spacer { size, .. } => *size,
        _ => 24.0,
    }
}

fn calculate_panel_dimensions(children: &[LayoutNode], side: PanelSide) -> (f32, f32) {
    let w = children.iter().map(get_node_width).fold(0.0f32, f32::max);
    let h = children.iter().map(get_node_height).fold(0.0f32, f32::max);

    match side {
        PanelSide::Left | PanelSide::Right => (w.max(200.0), 800.0),
        PanelSide::Top | PanelSide::Bottom => (375.0, h.max(100.0)),
        PanelSide::Center => (w.max(300.0), h.max(200.0)),
    }
}

fn calculate_card_dimensions(children: &[LayoutNode]) -> (f32, f32) {
    let w = children
        .iter()
        .map(get_node_width)
        .fold(0.0f32, f32::max)
        .max(100.0);
    let h = children.iter().map(get_node_height).sum::<f32>().max(60.0);
    (w + 16.0, h + 16.0) // Add padding
}

// ============================================================================
// SVG-to-LayoutElement Parser
// ============================================================================

/// Parse an SVG string into a flat list of LayoutElements.
/// Uses simple string scanning (no XML parser dependency).
pub fn parse_svg_elements(svg: &str) -> Vec<LayoutElement> {
    let mut elements: Vec<LayoutElement> = Vec::new();

    // Find all groups
    let mut search_start = 0;
    while let Some(g_start) = svg[search_start..].find("<g") {
        let g_start = search_start + g_start;
        if let Some(g_tag_end) = svg[g_start..].find('>') {
            let g_tag_end = g_start + g_tag_end;
            let g_tag = &svg[g_start..g_tag_end + 1];

            // Extract group id
            let id =
                extract_attr(g_tag, "id").unwrap_or_else(|| format!("group_{}", elements.len()));

            // Check for transform attribute (might contain x, y)
            let (x, y) = extract_transform_xy(g_tag);

            // Find the group's direct children (rect, text, path, image)
            let group_content_start = g_tag_end + 1;
            if let Some(g_end) = find_matching_close(&svg[group_content_start..], "g") {
                let group_content = &svg[group_content_start..group_content_start + g_end];

                let children = parse_group_children(group_content);

                // If children exist, create a group element
                if !children.is_empty() {
                    // Calculate bounding box from children
                    let min_x = children.iter().map(|c| c.x).fold(f32::INFINITY, f32::min);
                    let min_y = children.iter().map(|c| c.y).fold(f32::INFINITY, f32::min);
                    let max_x = children.iter().map(|c| c.x + c.w).fold(0.0f32, f32::max);
                    let max_y = children.iter().map(|c| c.y + c.h).fold(0.0f32, f32::max);

                    elements.push(LayoutElement {
                        id,
                        el_type: ElementType::Group,
                        x: x.unwrap_or(min_x),
                        y: y.unwrap_or(min_y),
                        w: if x.is_some() {
                            max_x - min_x
                        } else {
                            max_x - min_x
                        },
                        h: if y.is_some() {
                            max_y - min_y
                        } else {
                            max_y - min_y
                        },
                        fill: extract_fill_from_tag(g_tag),
                        stroke: extract_stroke_from_tag(g_tag),
                        text: None,
                        text_size: None,
                        children,
                        opacity: 1.0,
                        rotation_deg: 0.0,
                        corner_radius: 0.0,
                        gradient: None,
                        blend_mode: BlendMode::Normal,
                        effects: vec![],
                        stroke_dash: None,
                        clip_children: false,
                        text_align: None,
                        letter_spacing: None,
                        line_height: None,
                    });
                }
            }

            search_start = g_tag_end + 1;
        } else {
            search_start = g_start + 1;
        }
    }

    // Also look for top-level elements not in groups
    elements.extend(parse_top_level_elements(svg));

    // If no elements found, try to parse rects/texts directly
    if elements.is_empty() {
        elements.extend(parse_top_level_elements(svg));
    }

    elements
}

fn parse_group_children(content: &str) -> Vec<LayoutElement> {
    let mut elements = Vec::new();

    // Parse rects
    let mut rect_start = 0;
    while let Some(idx) = content[rect_start..].find("<rect") {
        let idx = rect_start + idx;
        if let Some(tag_end) = content[idx..].find('>') {
            let tag_end = idx + tag_end;
            let tag = &content[idx..tag_end + 1];

            let id = extract_attr(tag, "id").unwrap_or_else(|| format!("rect_{}", elements.len()));
            let x: f32 = extract_float_attr(tag, "x").unwrap_or(0.0);
            let y: f32 = extract_float_attr(tag, "y").unwrap_or(0.0);
            let w: f32 = extract_float_attr(tag, "width").unwrap_or(0.0);
            let h: f32 = extract_float_attr(tag, "height").unwrap_or(0.0);

            elements.push(LayoutElement {
                id,
                el_type: ElementType::Shape,
                x,
                y,
                w,
                h,
                fill: extract_fill_from_tag(tag),
                stroke: extract_stroke_from_tag(tag),
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
            });

            rect_start = tag_end + 1;
        } else {
            rect_start = idx + 1;
        }
    }

    // Parse text elements
    let mut text_start = 0;
    while let Some(idx) = content[text_start..].find("<text") {
        let idx = text_start + idx;
        if let Some(tag_end) = content[idx..].find('>') {
            let tag_end = idx + tag_end;
            let tag = &content[idx..tag_end + 1];

            // Find text content (between > and </text>)
            let text_content_start = tag_end + 1;
            if let Some(text_end) = content[text_content_start..].find("</text>") {
                let text_content = &content[text_content_start..text_content_start + text_end];
                let text = text_content.trim().to_string();

                let id =
                    extract_attr(tag, "id").unwrap_or_else(|| format!("text_{}", elements.len()));
                let x: f32 = extract_float_attr(tag, "x").unwrap_or(0.0);
                let y: f32 = extract_float_attr(tag, "y").unwrap_or(0.0);
                let font_size: f32 = extract_float_attr(tag, "font-size")
                    .or_else(|| extract_float_attr(tag, "fontsize"))
                    .unwrap_or(14.0);

                // Try to get fill color from style attribute
                let fill = extract_fill_from_tag(tag);

                elements.push(LayoutElement {
                    id,
                    el_type: ElementType::Text,
                    x,
                    y,
                    w: text.len() as f32 * font_size * 0.6,
                    h: font_size * 1.2,
                    fill,
                    stroke: None,
                    text: Some(text),
                    text_size: Some(font_size),
                    children: vec![],
                    opacity: 1.0,
                    rotation_deg: 0.0,
                    corner_radius: 0.0,
                    gradient: None,
                    blend_mode: BlendMode::Normal,
                    effects: vec![],
                    stroke_dash: None,
                    clip_children: false,
                    text_align: None,
                    letter_spacing: None,
                    line_height: None,
                });
            }

            text_start = tag_end + 1;
        } else {
            text_start = idx + 1;
        }
    }

    // Parse path elements
    let mut path_start = 0;
    while let Some(idx) = content[path_start..].find("<path") {
        let idx = path_start + idx;
        if let Some(tag_end) = content[idx..].find('>') {
            let tag_end = idx + tag_end;
            let tag = &content[idx..tag_end + 1];

            let id = extract_attr(tag, "id").unwrap_or_else(|| format!("path_{}", elements.len()));

            // Try to extract approximate bounds from path data
            let (w, h) = if let Some(d_start) = tag.find("d=\"") {
                let d_start = d_start + 3;
                if let Some(d_end) = tag[d_start..].find('"') {
                    let d = &tag[d_start..d_start + d_end];
                    estimate_path_bounds(d)
                } else {
                    (100.0, 100.0)
                }
            } else {
                (100.0, 100.0)
            };

            elements.push(LayoutElement {
                id,
                el_type: ElementType::Path,
                x: 0.0,
                y: 0.0,
                w,
                h,
                fill: extract_fill_from_tag(tag),
                stroke: extract_stroke_from_tag(tag),
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
            });

            path_start = tag_end + 1;
        } else {
            path_start = idx + 1;
        }
    }

    // Parse image elements
    let mut img_start = 0;
    while let Some(idx) = content[img_start..].find("<image") {
        let idx = img_start + idx;
        if let Some(tag_end) = content[idx..].find('>') {
            let tag_end = idx + tag_end;
            let tag = &content[idx..tag_end + 1];

            let id = extract_attr(tag, "id").unwrap_or_else(|| format!("image_{}", elements.len()));
            let x: f32 = extract_float_attr(tag, "x").unwrap_or(0.0);
            let y: f32 = extract_float_attr(tag, "y").unwrap_or(0.0);
            let w: f32 = extract_float_attr(tag, "width").unwrap_or(100.0);
            let h: f32 = extract_float_attr(tag, "height").unwrap_or(100.0);

            elements.push(LayoutElement {
                id,
                el_type: ElementType::Image,
                x,
                y,
                w,
                h,
                fill: None,
                stroke: None,
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
            });

            img_start = tag_end + 1;
        } else {
            img_start = idx + 1;
        }
    }

    elements
}

fn parse_top_level_elements(svg: &str) -> Vec<LayoutElement> {
    let mut elements = Vec::new();

    // Parse rects
    let mut rect_start = 0;
    while let Some(idx) = svg[rect_start..].find("<rect") {
        let idx = rect_start + idx;
        if let Some(tag_end) = svg[idx..].find('>') {
            let tag_end = idx + tag_end;
            let tag = &svg[idx..tag_end + 1];

            // Skip if inside a group
            let preceding = &svg[rect_start..idx];
            if !preceding.ends_with("<g") && !preceding.trim().ends_with('<') {
                let id =
                    extract_attr(tag, "id").unwrap_or_else(|| format!("rect_{}", elements.len()));
                let x: f32 = extract_float_attr(tag, "x").unwrap_or(0.0);
                let y: f32 = extract_float_attr(tag, "y").unwrap_or(0.0);
                let w: f32 = extract_float_attr(tag, "width").unwrap_or(0.0);
                let h: f32 = extract_float_attr(tag, "height").unwrap_or(0.0);

                elements.push(LayoutElement {
                    id,
                    el_type: ElementType::Shape,
                    x,
                    y,
                    w,
                    h,
                    fill: extract_fill_from_tag(tag),
                    stroke: extract_stroke_from_tag(tag),
                    text: None,
                    text_size: None,
                    children: vec![],
                    opacity: 1.0,
                    rotation_deg: 0.0,
                    corner_radius: 0.0,
                    gradient: None,
                    blend_mode: BlendMode::Normal,
                    effects: vec![],
                    stroke_dash: None,
                    clip_children: false,
                    text_align: None,
                    letter_spacing: None,
                    line_height: None,
                });
            }

            rect_start = tag_end + 1;
        } else {
            rect_start = idx + 1;
        }
    }

    // Parse text elements
    let mut text_start = 0;
    while let Some(idx) = svg[text_start..].find("<text") {
        let idx = text_start + idx;
        if let Some(tag_end) = svg[idx..].find('>') {
            let tag_end = idx + tag_end;

            // Skip if inside a group
            let preceding = &svg[text_start..idx];
            if preceding
                .rfind("<g")
                .map_or(true, |g_pos| g_pos < preceding.rfind("</g").unwrap_or(0))
            {
                let tag = &svg[idx..tag_end + 1];
                let text_content_start = tag_end + 1;
                if let Some(text_end) = svg[text_content_start..].find("</text>") {
                    let text_content = &svg[text_content_start..text_content_start + text_end];
                    let text = text_content.trim().to_string();

                    let id = extract_attr(tag, "id")
                        .unwrap_or_else(|| format!("text_{}", elements.len()));
                    let x: f32 = extract_float_attr(tag, "x").unwrap_or(0.0);
                    let y: f32 = extract_float_attr(tag, "y").unwrap_or(0.0);
                    let font_size: f32 = extract_float_attr(tag, "font-size")
                        .or_else(|| extract_float_attr(tag, "fontsize"))
                        .unwrap_or(14.0);

                    elements.push(LayoutElement {
                        id,
                        el_type: ElementType::Text,
                        x,
                        y,
                        w: text.len() as f32 * font_size * 0.6,
                        h: font_size * 1.2,
                        fill: extract_fill_from_tag(tag),
                        stroke: None,
                        text: Some(text),
                        text_size: Some(font_size),
                        children: vec![],
                        opacity: 1.0,
                        rotation_deg: 0.0,
                        corner_radius: 0.0,
                        gradient: None,
                        blend_mode: BlendMode::Normal,
                        effects: vec![],
                        stroke_dash: None,
                        clip_children: false,
                        text_align: None,
                        letter_spacing: None,
                        line_height: None,
                    });
                }
            }

            text_start = tag_end + 1;
        } else {
            text_start = idx + 1;
        }
    }

    elements
}

fn find_matching_close(s: &str, tag: &str) -> Option<usize> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    let mut depth = 1;
    let mut pos = 0;

    while depth > 0 && pos < s.len() {
        if s[pos..].starts_with(&open) && !s[pos..].starts_with(&format!("{}/", open)) {
            depth += 1;
            pos += open.len();
        } else if s[pos..].starts_with(&close) {
            depth -= 1;
            if depth == 0 {
                return Some(pos);
            }
            pos += close.len();
        } else {
            pos += 1;
        }
    }

    None
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let patterns = [format!("{}=", attr), format!("{} =", attr)];

    for pattern in &patterns {
        if let Some(idx) = tag.find(pattern) {
            let idx = idx + pattern.len();
            if idx < tag.len() && (tag[idx..].starts_with('"') || tag[idx..].starts_with('\'')) {
                let quote = tag[idx..].chars().next()?;
                let rest = &tag[idx + 1..];
                if let Some(end_idx) = rest.find(quote) {
                    return Some(rest[..end_idx].to_string());
                }
            }
        }
    }

    None
}

fn extract_float_attr(tag: &str, attr: &str) -> Option<f32> {
    extract_attr(tag, attr)?.parse().ok()
}

fn extract_fill_from_tag(tag: &str) -> Option<Color32> {
    // Try fill attribute
    if let Some(fill) = extract_attr(tag, "fill") {
        if fill != "none" {
            if let Some(c) = crate::svg::parse_svg_color(&fill) {
                return Some(c);
            }
        }
    }

    // Try style attribute
    if let Some(style) = extract_attr(tag, "style") {
        // Look for fill: in style
        if let Some(f_start) = style.find("fill:") {
            let after_fill = &style[f_start + 5..];
            // Get the value until ; or end
            let value = after_fill.trim_start_matches(' ').trim_start_matches(':');
            let end = value.find(';').unwrap_or(value.len());
            let fill_value = value[..end].trim();
            if fill_value != "none" {
                if let Some(c) = crate::svg::parse_svg_color(fill_value) {
                    return Some(c);
                }
            }
        }
    }

    None
}

fn extract_stroke_from_tag(tag: &str) -> Option<(f32, Color32)> {
    let stroke_color = extract_attr(tag, "stroke");
    let stroke_width = extract_float_attr(tag, "stroke-width");

    if let Some(color_str) = stroke_color {
        if color_str != "none" {
            if let Some(c) = crate::svg::parse_svg_color(&color_str) {
                return Some((stroke_width.unwrap_or(1.0), c));
            }
        }
    }

    None
}

fn extract_transform_xy(tag: &str) -> (Option<f32>, Option<f32>) {
    if let Some(transform) = extract_attr(tag, "transform") {
        // Parse translate(x, y) or translate(x y)
        if transform.starts_with("translate(") {
            let inner = &transform[10..];
            if let Some(end) = inner.find(')') {
                let coords = &inner[..end];
                let parts: Vec<&str> = coords
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .filter(|s| !s.is_empty())
                    .collect();

                if parts.len() >= 2 {
                    let x = parts[0].parse().ok();
                    let y = parts[1].parse().ok();
                    return (x, y);
                } else if parts.len() == 1 {
                    let x = parts[0].parse().ok();
                    return (x, None);
                }
            }
        }
    }

    (None, None)
}

fn estimate_path_bounds(d: &str) -> (f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    let mut current_x = 0.0f32;
    let mut current_y = 0.0f32;

    let tokens: Vec<&str> = d.split(|c: char| c.is_whitespace() || c == ',').collect();
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];

        match token {
            "M" | "L" | "m" | "l" => {
                if i + 2 < tokens.len() {
                    let x: f32 = tokens[i + 1].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 2].parse().unwrap_or(0.0);

                    if token == "m" || token == "l" {
                        current_x += x;
                        current_y += y;
                    } else {
                        current_x = x;
                        current_y = y;
                    }

                    min_x = min_x.min(current_x);
                    min_y = min_y.min(current_y);
                    max_x = max_x.max(current_x);
                    max_y = max_y.max(current_y);

                    i += 3;
                } else {
                    i += 1;
                }
            }
            "H" | "h" | "V" | "v" => {
                if i + 1 < tokens.len() {
                    let val: f32 = tokens[i + 1].parse().unwrap_or(0.0);
                    if token == "h" {
                        current_x += val;
                    } else if token == "v" {
                        current_y += val;
                    } else if token == "H" {
                        current_x = val;
                    } else {
                        current_y = val;
                    }

                    min_x = min_x.min(current_x);
                    min_y = min_y.min(current_y);
                    max_x = max_x.max(current_x);
                    max_y = max_y.max(current_y);

                    i += 2;
                } else {
                    i += 1;
                }
            }
            "C" | "c" => {
                if i + 6 < tokens.len() {
                    let x: f32 = tokens[i + 5].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 6].parse().unwrap_or(0.0);

                    if token == "c" {
                        current_x += x;
                        current_y += y;
                    } else {
                        current_x = x;
                        current_y = y;
                    }

                    min_x = min_x.min(current_x);
                    min_y = min_y.min(current_y);
                    max_x = max_x.max(current_x);
                    max_y = max_y.max(current_y);

                    i += 7;
                } else {
                    i += 1;
                }
            }
            "Q" | "q" => {
                if i + 4 < tokens.len() {
                    let x: f32 = tokens[i + 3].parse().unwrap_or(0.0);
                    let y: f32 = tokens[i + 4].parse().unwrap_or(0.0);

                    if token == "q" {
                        current_x += x;
                        current_y += y;
                    } else {
                        current_x = x;
                        current_y = y;
                    }

                    min_x = min_x.min(current_x);
                    min_y = min_y.min(current_y);
                    max_x = max_x.max(current_x);
                    max_y = max_y.max(current_y);

                    i += 5;
                } else {
                    i += 1;
                }
            }
            "Z" | "z" => {
                i += 1;
            }
            _ => {
                // Try to parse as a number
                if let Ok(val) = token.parse::<f32>() {
                    if i + 1 < tokens.len() {
                        if let Ok(y_val) = tokens[i + 1].parse::<f32>() {
                            current_x += val;
                            current_y += y_val;

                            min_x = min_x.min(current_x);
                            min_y = min_y.min(current_y);
                            max_x = max_x.max(current_x);
                            max_y = max_y.max(current_y);

                            i += 2;
                        } else {
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
        }
    }

    if min_x == f32::INFINITY {
        return (100.0, 100.0);
    }

    ((max_x - min_x).max(1.0), (max_y - min_y).max(1.0))
}

// ============================================================================
// JSON Sidecar Parser
// ============================================================================

#[derive(Clone, Debug)]
pub struct ArtboardInfo {
    pub name: String,
    pub width: f32,
    pub height: f32,
}

/// Parse a JSON sidecar (from Illustrator plugin) into LayoutElements.
/// JSON format: { "artboard": {...}, "elements": [{id, type, x, y, w, h, text, textStyle}] }
pub fn parse_json_sidecar(json: &str) -> Result<(ArtboardInfo, Vec<LayoutElement>), String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("JSON parse error: {}", e))?;

    // Extract artboard info
    let artboard = value.get("artboard").ok_or("Missing 'artboard' field")?;

    let name = artboard
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();

    let width = artboard
        .get("width")
        .and_then(|v| v.as_f64())
        .unwrap_or(375.0) as f32;

    let height = artboard
        .get("height")
        .and_then(|v| v.as_f64())
        .unwrap_or(812.0) as f32;

    let artboard_info = ArtboardInfo {
        name,
        width,
        height,
    };

    // Extract elements
    let elements_array = value
        .get("elements")
        .ok_or("Missing 'elements' field")?
        .as_array()
        .ok_or("'elements' must be an array")?;

    let mut elements = Vec::new();

    for (i, elem_value) in elements_array.iter().enumerate() {
        let id = elem_value
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("elem_{}", i))
            .to_string();

        let type_str = elem_value
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let el_type = match type_str.to_lowercase().as_str() {
            "group" | "g" => ElementType::Group,
            "shape" | "rect" => ElementType::Shape,
            "path" => ElementType::Path,
            "text" => ElementType::Text,
            "image" | "img" => ElementType::Image,
            _ => ElementType::Unknown,
        };

        let x = elem_value.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

        let y = elem_value.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

        let w = elem_value
            .get("w")
            .or_else(|| elem_value.get("width"))
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0) as f32;

        let h = elem_value
            .get("h")
            .or_else(|| elem_value.get("height"))
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0) as f32;

        let text = elem_value
            .get("text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let text_size = elem_value
            .get("textStyle")
            .and_then(|ts| ts.get("fontSize"))
            .and_then(|v| v.as_f64())
            .map(|f| f as f32)
            .or_else(|| {
                elem_value
                    .get("textStyle")
                    .and_then(|ts| ts.get("font-size"))
                    .and_then(|v| v.as_f64())
                    .map(|f| f as f32)
            });

        let fill = elem_value
            .get("fill")
            .and_then(|v| v.as_str())
            .and_then(|s| crate::svg::parse_svg_color(s));

        let stroke_width = elem_value
            .get("strokeWidth")
            .or_else(|| elem_value.get("stroke-width"))
            .and_then(|v| v.as_f64())
            .map(|f| f as f32);

        let stroke_color = elem_value
            .get("stroke")
            .and_then(|v| v.as_str())
            .and_then(|s| crate::svg::parse_svg_color(s));

        let stroke = stroke_width.and_then(|w| stroke_color.map(|c| (w, c)));

        // Parse extended fields
        let opacity = elem_value
            .get("opacity")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(1.0);
        let rotation_deg = elem_value
            .get("rotation")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
        let corner_radius = elem_value
            .get("cornerRadius")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);
        let stroke_dash = elem_value
            .get("strokeDash")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_f64())
                    .map(|f| f as f32)
                    .collect()
            });
        let clip_children = elem_value
            .get("clipChildren")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let text_align = elem_value
            .get("textAlign")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "left" => TextAlign::Left,
                "center" => TextAlign::Center,
                "right" => TextAlign::Right,
                "justified" => TextAlign::Justified,
                _ => TextAlign::Left,
            });
        let letter_spacing = elem_value
            .get("letterSpacing")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32);
        let line_height = elem_value
            .get("lineHeight")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32);

        // Parse blend mode
        let blend_mode = elem_value
            .get("blendMode")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "multiply" => BlendMode::Multiply,
                "screen" => BlendMode::Screen,
                "overlay" => BlendMode::Overlay,
                "darken" => BlendMode::Darken,
                "lighten" => BlendMode::Lighten,
                _ => BlendMode::Normal,
            })
            .unwrap_or(BlendMode::Normal);

        // Parse gradient if present
        let gradient = elem_value
            .get("gradient")
            .and_then(|v| v.as_object())
            .map(|g| {
                let gradient_type = if g.get("type").and_then(|t| t.as_str()) == Some("radial") {
                    GradientType::Radial
                } else {
                    GradientType::Linear
                };
                let angle_deg = g
                    .get("angle")
                    .and_then(|a| a.as_f64())
                    .map(|v| v as f32)
                    .unwrap_or(0.0);
                let stops = g
                    .get("stops")
                    .and_then(|s| s.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|stop| {
                                let position = stop.get("position")?.as_f64()? as f32;
                                let color = stop
                                    .get("color")?
                                    .as_str()
                                    .and_then(|s| crate::svg::parse_svg_color(s))
                                    .unwrap_or(egui::Color32::BLACK);
                                Some(GradientStop { position, color })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                GradientDef {
                    gradient_type,
                    angle_deg,
                    stops,
                }
            });

        // Parse effects if present
        let effects: Vec<EffectDef> = elem_value
            .get("effects")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| {
                        let effect_type = match e.get("type")?.as_str()? {
                            "dropShadow" | "drop-shadow" => EffectType::DropShadow,
                            "innerShadow" | "inner-shadow" => EffectType::InnerShadow,
                            "glow" => EffectType::Glow,
                            _ => return None,
                        };
                        let x = e
                            .get("x")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32)
                            .unwrap_or(0.0);
                        let y = e
                            .get("y")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32)
                            .unwrap_or(0.0);
                        let blur = e
                            .get("blur")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32)
                            .unwrap_or(0.0);
                        let color = e
                            .get("color")?
                            .as_str()
                            .and_then(|s| crate::svg::parse_svg_color(s))
                            .unwrap_or(egui::Color32::BLACK);
                        Some(EffectDef {
                            effect_type,
                            x,
                            y,
                            blur,
                            color,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse children if present
        let children = if el_type == ElementType::Group {
            elem_value
                .get("children")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|child| {
                            let child_id = child
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("child")
                                .to_string();
                            let child_type = child
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let child_el_type = match child_type.to_lowercase().as_str() {
                                "group" | "g" => ElementType::Group,
                                "shape" | "rect" => ElementType::Shape,
                                "path" => ElementType::Path,
                                "text" => ElementType::Text,
                                "image" | "img" => ElementType::Image,
                                _ => ElementType::Unknown,
                            };
                            let child_x =
                                child.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                            let child_y =
                                child.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                            let child_w = child
                                .get("w")
                                .or_else(|| child.get("width"))
                                .and_then(|v| v.as_f64())
                                .unwrap_or(100.0) as f32;
                            let child_h = child
                                .get("h")
                                .or_else(|| child.get("height"))
                                .and_then(|v| v.as_f64())
                                .unwrap_or(100.0) as f32;
                            let child_text = child
                                .get("text")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let child_fill = child
                                .get("fill")
                                .and_then(|v| v.as_str())
                                .and_then(|s| crate::svg::parse_svg_color(s));

                            Some(LayoutElement {
                                id: child_id,
                                el_type: child_el_type,
                                x: child_x,
                                y: child_y,
                                w: child_w,
                                h: child_h,
                                fill: child_fill,
                                stroke: None,
                                text: child_text,
                                text_size: None,
                                children: vec![],
                                opacity: 1.0,
                                rotation_deg: 0.0,
                                corner_radius: 0.0,
                                gradient: None,
                                blend_mode: BlendMode::Normal,
                                effects: vec![],
                                stroke_dash: None,
                                clip_children: false,
                                text_align: None,
                                letter_spacing: None,
                                line_height: None,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };

        elements.push(LayoutElement {
            id,
            el_type,
            x,
            y,
            w,
            h,
            fill,
            stroke,
            text,
            text_size,
            children,
            opacity,
            rotation_deg,
            corner_radius,
            gradient,
            blend_mode,
            effects,
            stroke_dash,
            clip_children,
            text_align,
            letter_spacing,
            line_height,
        });
    }

    Ok((artboard_info, elements))
}

// ============================================================================
// Sidecar Diffing
// ============================================================================

/// Represents a change between two sidecar JSON files.
#[derive(Clone, Debug)]
pub enum SidecarChange {
    /// A new element was added.
    Added(LayoutElement),
    /// An element was removed.
    Removed(String),
    /// An element moved positions.
    Moved {
        id: String,
        old_pos: (f32, f32),
        new_pos: (f32, f32),
    },
    /// An element was resized.
    Resized {
        id: String,
        old_size: (f32, f32),
        new_size: (f32, f32),
    },
    /// An element's color changed.
    ColorChanged {
        id: String,
        old: Color32,
        new: Color32,
    },
    /// An element's text content changed.
    TextChanged {
        id: String,
        old: String,
        new: String,
    },
}

/// Diff two sidecar JSON strings and return a list of changes.
pub fn diff_sidecars(old_json: &str, new_json: &str) -> Vec<SidecarChange> {
    let old_els = parse_json_sidecar(old_json)
        .map(|(_, els)| els)
        .unwrap_or_default();
    let new_els = parse_json_sidecar(new_json)
        .map(|(_, els)| els)
        .unwrap_or_default();

    let old_map: std::collections::HashMap<String, &LayoutElement> =
        old_els.iter().map(|e| (e.id.clone(), e)).collect();
    let new_map: std::collections::HashMap<String, &LayoutElement> =
        new_els.iter().map(|e| (e.id.clone(), e)).collect();

    let mut changes = Vec::new();

    // Added
    for (id, el) in &new_map {
        if !old_map.contains_key(id) {
            changes.push(SidecarChange::Added((*el).clone()));
        }
    }

    // Removed
    for id in old_map.keys() {
        if !new_map.contains_key(id) {
            changes.push(SidecarChange::Removed(id.clone()));
        }
    }

    // Changed
    for (id, new_el) in &new_map {
        if let Some(old_el) = old_map.get(id) {
            if (old_el.x - new_el.x).abs() > 0.5 || (old_el.y - new_el.y).abs() > 0.5 {
                changes.push(SidecarChange::Moved {
                    id: id.clone(),
                    old_pos: (old_el.x, old_el.y),
                    new_pos: (new_el.x, new_el.y),
                });
            }
            if (old_el.w - new_el.w).abs() > 0.5 || (old_el.h - new_el.h).abs() > 0.5 {
                changes.push(SidecarChange::Resized {
                    id: id.clone(),
                    old_size: (old_el.w, old_el.h),
                    new_size: (new_el.w, new_el.h),
                });
            }
            if old_el.fill != new_el.fill {
                changes.push(SidecarChange::ColorChanged {
                    id: id.clone(),
                    old: old_el.fill.unwrap_or(Color32::BLACK),
                    new: new_el.fill.unwrap_or(Color32::BLACK),
                });
            }
            if old_el.text != new_el.text {
                changes.push(SidecarChange::TextChanged {
                    id: id.clone(),
                    old: old_el.text.clone().unwrap_or_default(),
                    new: new_el.text.clone().unwrap_or_default(),
                });
            }
        }
    }

    changes
}

// ============================================================================
// Public API Entry Point
// ============================================================================

/// Full pipeline: SVG string → Rust scaffold code.
/// This is the main entry point for the Illustrator export workflow.
pub fn svg_to_rust_scaffold(svg: &str, fn_name: &str, options: &InferenceOptions) -> String {
    // Parse SVG into elements
    let elements = parse_svg_elements(svg);

    // Infer layout
    let nodes = infer_layout(&elements, options);

    // Generate Rust code
    let artboard_w = elements.iter().map(|e| e.x + e.w).fold(375.0f32, f32::max);

    let artboard_h = elements.iter().map(|e| e.y + e.h).fold(812.0f32, f32::max);

    let bg_color = elements
        .iter()
        .find(|e| e.id.to_lowercase().contains("background") || e.id.to_lowercase().contains("bg"))
        .and_then(|e| e.fill);

    generate_rust(fn_name, artboard_w, artboard_h, &nodes, bg_color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_naming_row() {
        assert!(matches!(parse_naming("row-login"), NamingHint::Row(_)));
        assert!(matches!(parse_naming("hstack-buttons"), NamingHint::Row(_)));
    }

    #[test]
    fn test_parse_naming_column() {
        assert!(matches!(parse_naming("col-sidebar"), NamingHint::Column(_)));
        assert!(matches!(
            parse_naming("vstack-items"),
            NamingHint::Column(_)
        ));
    }

    #[test]
    fn test_parse_naming_panel() {
        assert!(matches!(
            parse_naming("panel-left"),
            NamingHint::Panel(PanelSide::Left)
        ));
        assert!(matches!(
            parse_naming("panel-right"),
            NamingHint::Panel(PanelSide::Right)
        ));
        assert!(matches!(
            parse_naming("panel-top"),
            NamingHint::Panel(PanelSide::Top)
        ));
        assert!(matches!(
            parse_naming("panel-bottom"),
            NamingHint::Panel(PanelSide::Bottom)
        ));
    }

    #[test]
    fn test_parse_naming_button() {
        assert!(matches!(parse_naming("btn-submit"), NamingHint::Button(_)));
        assert!(matches!(
            parse_naming("button-cancel"),
            NamingHint::Button(_)
        ));
    }

    #[test]
    fn test_parse_naming_gap() {
        assert!(matches!(parse_naming("gap-16"), NamingHint::Gap(16.0)));
        assert!(matches!(parse_naming("gap-8"), NamingHint::Gap(8.0)));
    }

    #[test]
    fn test_parse_naming_none() {
        assert!(matches!(parse_naming("Rectangle 1"), NamingHint::None));
        assert!(matches!(parse_naming("some random name"), NamingHint::None));
    }

    #[test]
    fn test_cluster_into_rows() {
        let elements = vec![
            LayoutElement {
                id: "a".to_string(),
                el_type: ElementType::Shape,
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 50.0,
                fill: None,
                stroke: None,
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
            },
            LayoutElement {
                id: "b".to_string(),
                el_type: ElementType::Shape,
                x: 110.0,
                y: 5.0,
                w: 100.0,
                h: 50.0,
                fill: None,
                stroke: None,
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
            },
            LayoutElement {
                id: "c".to_string(),
                el_type: ElementType::Shape,
                x: 50.0,
                y: 100.0,
                w: 100.0,
                h: 50.0,
                fill: None,
                stroke: None,
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
            },
        ];

        let rows = cluster_into_rows(&elements, 0.5);
        // a and b should be in the same row (Y overlap), c in a different row
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].len(), 2);
        assert_eq!(rows[1].len(), 1);
    }

    #[test]
    fn test_infer_horizontal_gap() {
        let elements = vec![
            LayoutElement {
                id: "a".to_string(),
                el_type: ElementType::Shape,
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 50.0,
                fill: None,
                stroke: None,
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
            },
            LayoutElement {
                id: "b".to_string(),
                el_type: ElementType::Shape,
                x: 108.0,
                y: 0.0,
                w: 100.0,
                h: 50.0,
                fill: None,
                stroke: None,
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
            },
            LayoutElement {
                id: "c".to_string(),
                el_type: ElementType::Shape,
                x: 216.0,
                y: 0.0,
                w: 100.0,
                h: 50.0,
                fill: None,
                stroke: None,
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
            },
        ];

        let gap = infer_horizontal_gap(&elements);
        assert!((gap - 8.0).abs() < 0.1);
    }

    #[test]
    fn test_infer_vertical_gap() {
        let elements = vec![
            LayoutElement {
                id: "a".to_string(),
                el_type: ElementType::Shape,
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 50.0,
                fill: None,
                stroke: None,
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
            },
            LayoutElement {
                id: "b".to_string(),
                el_type: ElementType::Shape,
                x: 0.0,
                y: 58.0,
                w: 100.0,
                h: 50.0,
                fill: None,
                stroke: None,
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
            },
        ];

        let gap = infer_vertical_gap(&elements);
        assert!((gap - 8.0).abs() < 0.1);
    }

    #[test]
    fn test_parse_svg_elements() {
        let svg = r##"<svg>
            <rect id="bg-rect" x="0" y="0" width="375" height="812" fill="#121212"/>
            <g id="row-buttons">
                <rect id="btn-login" x="20" y="400" width="100" height="40"/>
                <rect id="btn-cancel" x="130" y="400" width="100" height="40"/>
            </g>
            <text id="label-welcome" x="20" y="50">Welcome</text>
        </svg>"##;

        let elements = parse_svg_elements(svg);
        assert!(!elements.is_empty());
    }

    #[test]
    fn test_parse_json_sidecar() {
        let json = r##"{
            "artboard": {
                "name": "Login",
                "width": 375,
                "height": 812
            },
            "elements": [
                {"id": "bg", "type": "rect", "x": 0, "y": 0, "w": 375, "h": 812, "fill": "#121212"},
                {"id": "btn-login", "type": "rect", "x": 20, "y": 400, "w": 100, "h": 40},
                {"id": "label-welcome", "type": "text", "x": 20, "y": 50, "text": "Welcome"}
            ]
        }"##;

        let result = parse_json_sidecar(json);
        assert!(result.is_ok());

        let (artboard, elements) = result.unwrap();
        assert_eq!(artboard.name, "Login");
        assert_eq!(artboard.width, 375.0);
        assert_eq!(artboard.height, 812.0);
        assert_eq!(elements.len(), 3);
    }

    #[test]
    fn test_svg_to_rust_scaffold() {
        let svg = r##"<svg>
            <rect id="bg" x="0" y="0" width="375" height="812" fill="#121212"/>
            <text id="label-welcome" x="20" y="50">Welcome</text>
            <rect id="btn-submit" x="20" y="100" width="100" height="40"/>
        </svg>"##;

        let options = InferenceOptions::default();
        let code = svg_to_rust_scaffold(svg, "login", &options);

        assert!(code.contains("pub fn draw_login"));
        assert!(code.contains("egui::Color32::from_rgb"));
        assert!(code.contains("ui.label"));
        assert!(code.contains("ui.button"));
    }

    #[test]
    fn test_generate_rust_with_background() {
        let nodes = vec![
            LayoutNode::Label {
                text: "Hello".to_string(),
                size: 24.0,
                color: Some(Color32::WHITE),
                id: "greeting".to_string(),
            },
            LayoutNode::Button {
                label: "Click Me".to_string(),
                id: "main-btn".to_string(),
            },
        ];

        let code = generate_rust(
            "test",
            375.0,
            812.0,
            &nodes,
            Some(Color32::from_rgb(18, 18, 18)),
        );

        assert!(code.contains("pub fn draw_test"));
        assert!(code.contains("Background"));
        assert!(code.contains("egui::Color32::from_rgb(18, 18, 18)"));
    }

    #[test]
    fn test_infer_layout_with_naming_conventions() {
        let elements = vec![LayoutElement {
            id: "row-buttons".to_string(),
            el_type: ElementType::Group,
            x: 0.0,
            y: 0.0,
            w: 300.0,
            h: 50.0,
            fill: None,
            stroke: None,
            text: None,
            text_size: None,
            children: vec![
                LayoutElement {
                    id: "btn-a".to_string(),
                    el_type: ElementType::Shape,
                    x: 0.0,
                    y: 0.0,
                    w: 100.0,
                    h: 40.0,
                    fill: None,
                    stroke: None,
                    text: None,
                    text_size: None,
                    children: vec![],
                    opacity: 1.0,
                    rotation_deg: 0.0,
                    corner_radius: 0.0,
                    gradient: None,
                    blend_mode: BlendMode::Normal,
                    effects: vec![],
                    stroke_dash: None,
                    clip_children: false,
                    text_align: None,
                    letter_spacing: None,
                    line_height: None,
                },
                LayoutElement {
                    id: "btn-b".to_string(),
                    el_type: ElementType::Shape,
                    x: 110.0,
                    y: 0.0,
                    w: 100.0,
                    h: 40.0,
                    fill: None,
                    stroke: None,
                    text: None,
                    text_size: None,
                    children: vec![],
                    opacity: 1.0,
                    rotation_deg: 0.0,
                    corner_radius: 0.0,
                    gradient: None,
                    blend_mode: BlendMode::Normal,
                    effects: vec![],
                    stroke_dash: None,
                    clip_children: false,
                    text_align: None,
                    letter_spacing: None,
                    line_height: None,
                },
            ],
            opacity: 1.0,
            rotation_deg: 0.0,
            corner_radius: 0.0,
            gradient: None,
            blend_mode: BlendMode::Normal,
            effects: vec![],
            stroke_dash: None,
            clip_children: false,
            text_align: None,
            letter_spacing: None,
            line_height: None,
        }];

        let options = InferenceOptions::default();
        let nodes = infer_layout(&elements, &options);

        assert!(!nodes.is_empty());
        // The row should be inferred from the naming convention
        match &nodes[0] {
            LayoutNode::Row { id, .. } => {
                assert_eq!(id, "buttons");
            }
            _ => {}
        }
    }
}
