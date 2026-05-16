use super::*;

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

pub(crate) fn median(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

/// Cluster elements into rows by Y-overlap.
/// Returns `Vec<Vec<LayoutElement>>` where each inner Vec is one row.
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

    for elem in sorted.iter().skip(1) {
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

    nodes
}

pub(crate) fn infer_element(elem: &LayoutElement, options: &InferenceOptions) -> LayoutNode {
    // Check if it's a rich element that requires scene rendering
    let is_rich = !elem.path_points.is_empty()
        || !elem.appearance_stack.is_empty()
        || !elem.appearance_fills.is_empty()
        || !elem.appearance_strokes.is_empty()
        || elem.blend_mode != BlendMode::Normal
        || elem.clip_children
        || elem
            .effects
            .iter()
            .any(|e| e.blend_mode != BlendMode::Normal);

    if is_rich {
        return LayoutNode::RichScene(crate::scene::SceneNode::from_layout_element(elem));
    }

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
                    horizontal: false,
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
                    font_family: None,
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
                    style: VisualStyle::from_element(elem),
                };
            }
            NamingHint::Chip(label) => {
                // Chip is a small button-like element
                return LayoutNode::Button {
                    label,
                    id: elem.id.clone(),
                };
            }
            NamingHint::Toggle(label) => {
                // Toggle/checkbox - treat as button (no Checkbox variant in LayoutNode)
                return LayoutNode::Button {
                    label,
                    id: elem.id.clone(),
                };
            }
            NamingHint::Slider(label) => {
                // Slider - treat as shape (no Slider variant in LayoutNode)
                return LayoutNode::Shape {
                    x: elem.x,
                    y: elem.y,
                    w: elem.w,
                    h: elem.h,
                    fill: elem.fill.unwrap_or(Color32::from_gray(128)),
                    id: label,
                    style: VisualStyle::from_element(elem),
                };
            }
            NamingHint::Grid(label) => {
                // Grid layout - treat as column with tight spacing
                let children = infer_children(&elem.children, options);
                return LayoutNode::Column {
                    gap: 2.0,
                    children,
                    bg: elem.fill,
                    id: label,
                };
            }
            NamingHint::None => {}
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
        ElementType::Circle | ElementType::Ellipse => {
            LayoutNode::RichScene(crate::scene::SceneNode::from_layout_element(elem))
        }
        ElementType::Shape => LayoutNode::Shape {
            x: elem.x,
            y: elem.y,
            w: elem.w,
            h: elem.h,
            fill: elem.fill.unwrap_or(Color32::from_gray(128)),
            id: elem.id.clone(),
            style: VisualStyle::from_element(elem),
        },
        ElementType::Text => LayoutNode::Label {
            text: elem.text.clone().unwrap_or_default(),
            size: elem.text_size.unwrap_or(14.0),
            color: elem.fill,
            font_family: None,
            id: elem.id.clone(),
        },
        ElementType::Image => LayoutNode::Image {
            x: elem.x,
            y: elem.y,
            w: elem.w,
            h: elem.h,
            id: elem.id.clone(),
            style: VisualStyle::from_element(elem),
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
                style: VisualStyle::from_element(elem),
            }
        }
        ElementType::Unknown => LayoutNode::Unknown {
            id: elem.id.clone(),
            comment: format!("{:?}", elem.el_type),
        },
    }
}

pub(crate) fn infer_children(
    children: &[LayoutElement],
    options: &InferenceOptions,
) -> Vec<LayoutNode> {
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

pub(crate) fn is_horizontal_group(elements: &[LayoutElement]) -> bool {
    if elements.len() < 2 {
        return false;
    }

    let total_width: f32 = elements.iter().map(|e| e.w).sum();
    let total_height: f32 = elements.iter().map(|e| e.h).sum::<f32>() / elements.len() as f32;

    // Horizontal if total width is significantly greater than total height
    total_width > total_height * 1.5
}

pub(crate) fn is_vertical_group(elements: &[LayoutElement]) -> bool {
    if elements.len() < 2 {
        return false;
    }

    // Check if elements are stacked vertically by comparing positional variance
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
