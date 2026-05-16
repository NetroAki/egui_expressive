use super::*;

/// Parse AIPrivateData stream content
pub(crate) fn parse_aip_private_stream(
    content: &[u8],
    stream_idx: usize,
    errors: &mut Vec<String>,
) -> Option<Element> {
    let content_str = std::str::from_utf8(content).ok()?;

    let mut element = Element {
        id: format!("element_{}", stream_idx),
        ..Default::default()
    };

    if let Some(name) = extract_layer_name(content_str) {
        element.name = Some(name.clone());
        element.id = format!("{}_{}", name, stream_idx);
    }

    let effects = parse_live_effect_xml(content_str);
    if !effects.is_empty() {
        element.live_effects = effects;
    }

    let (fills, strokes) = parse_appearance(content_str);
    if !fills.is_empty() {
        element.appearance_fills = fills;
    }
    if !strokes.is_empty() {
        element.appearance_strokes = strokes;
    }

    if let Some(mesh) = parse_envelope_mesh(content_str) {
        element.envelope_mesh = Some(mesh);
    }

    if let Some(threed) = parse_3d_effect(content_str) {
        element.three_d = Some(threed);
    }

    let patches = parse_mesh_patches(content_str);
    if !patches.is_empty() {
        element.mesh_patches = patches;
    }

    // Parse CTM rotation candidates
    let ctms = parse_ctms_from_stream(content_str);
    element.transform_candidates = ctms
        .iter()
        .map(|&(r, sx, sy, tx, ty)| [r, sx, sy, tx, ty])
        .collect();
    if let Some(&(rot, sx, sy, tx, ty)) = ctms.last() {
        if rot.abs() > 0.01 {
            element.rotation_deg = rot;
        }
        element.scale_x = sx;
        element.scale_y = sy;
        element.translate_x = tx;
        element.translate_y = ty;
    }

    // Parse path geometry and detect corner radius
    let (path_pts, path_closed) = parse_path_geometry(content_str);
    if !path_pts.is_empty() {
        element.corner_radius = detect_corner_radius(&path_pts);
        element.path_points = path_pts;
        element.path_closed = path_closed;
    }

    if element.live_effects.is_empty()
        && element.appearance_fills.is_empty()
        && element.appearance_strokes.is_empty()
        && element.envelope_mesh.is_none()
        && element.three_d.is_none()
        && element.mesh_patches.is_empty()
        && element.rotation_deg.abs() < 0.01
        && element.path_points.is_empty()
    {
        errors.push(format!("warning: could not parse stream {}", stream_idx));
        return None;
    }

    Some(element)
}

/// Main parsing function
pub fn parse_ai_file(path: &Path) -> Result<AiParseResult, String> {
    let source_file = path.to_string_lossy().to_string();

    let doc = Document::load(path).map_err(|e| format!("not a valid .ai file: {}", e))?;

    let mut result = AiParseResult {
        version: "1.0".to_string(),
        source_file,
        ai_version: String::new(),
        artboards: Vec::new(),
        page_tiles: Vec::new(),
        elements: Vec::new(),
        transform_candidates: Vec::new(),
        errors: Vec::new(),
    };

    static AI_ARTBOARD_RECT_RE: OnceLock<Regex> = OnceLock::new();
    let ai_artboard_rect_re = AI_ARTBOARD_RECT_RE.get_or_init(|| {
        Regex::new(
            r"%%AI_ArtboardRect\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)",
        )
        .unwrap()
    });

    for (object_id, object) in doc.objects.iter() {
        let content = if let Ok(stream) = object.as_stream() {
            match stream.decompressed_content() {
                Ok(decompressed) => decompressed,
                Err(_) => stream.content.clone(),
            }
        } else {
            continue;
        };

        let content_str = std::str::from_utf8(&content).unwrap_or("");

        // Always scan every decompressed stream for CTM (cm) operators and path geometry.
        // Illustrator stores per-object transforms in main PDF content streams, not just AIPrivateData.
        if content_str.contains("AIPrivateData")
            || content_str.contains("LiveEffect")
            || content_str.contains("%AI9_")
        {
            if let Some(element) =
                parse_aip_private_stream(&content, object_id.0 as usize, &mut result.errors)
            {
                let is_dup = result.elements.iter().any(|e| e.id == element.id);
                if !is_dup {
                    result.elements.push(element);
                }
            }
        } else if content_str.contains(" cm")
            || content_str.contains("\ncm")
            || content_str.contains(" re")
            || content_str.contains(" f")
            || content_str.contains(" S")
        {
            // Main PDF content stream: scan painted vector paths with their current PDF graphics
            // state. This keeps Illustrator comparison fixtures code/vector-based instead of
            // falling back to the PDF or PNG raster render.
            let painted_paths = parse_pdf_painted_path_elements(content_str, object_id.0 as usize);
            if !painted_paths.is_empty() {
                for element in painted_paths {
                    let is_dup = result
                        .transform_candidates
                        .iter()
                        .any(|e| e.id == element.id);
                    if !is_dup {
                        result.transform_candidates.push(element);
                    }
                }
            } else {
                // Fallback for streams that have transform/path metadata but no paint operator that
                // the vector parser can safely associate with an element yet.
                let mut element = Element {
                    id: format!("ctm_element_{}", object_id.0),
                    is_pseudo_element: true,
                    ..Default::default()
                };
                let ctms = parse_ctms_from_stream(content_str);
                element.transform_candidates = ctms
                    .iter()
                    .map(|&(r, sx, sy, tx, ty)| [r, sx, sy, tx, ty])
                    .collect();
                if let Some(&(rot, sx, sy, tx, ty)) = ctms.last() {
                    element.rotation_deg = rot;
                    element.scale_x = sx;
                    element.scale_y = sy;
                    element.translate_x = tx;
                    element.translate_y = ty;
                }
                let (path_pts, path_closed) = parse_path_geometry(content_str);
                if !path_pts.is_empty() {
                    element.corner_radius = detect_corner_radius(&path_pts);
                    element.path_points = path_pts;
                    element.path_closed = path_closed;
                }
                // Only add if we found meaningful data
                if element.rotation_deg.abs() > 0.01 || !element.path_points.is_empty() {
                    let is_dup = result
                        .transform_candidates
                        .iter()
                        .any(|e| e.id == element.id);
                    if !is_dup {
                        result.transform_candidates.push(element);
                    }
                }
            }
        }

        // Parse artboard definitions from %AI9_Artboard markers
        if content_str.contains("%AI9_Artboard") || content_str.contains("%%BeginSetup") {
            {
                let re = artboard_re();
                let mut names = artboard_name_re()
                    .captures_iter(content_str)
                    .map(|c| c.get(1).map_or("", |m| m.as_str().trim()).to_string());
                for caps in re.captures_iter(content_str) {
                    if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                        (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
                    {
                        let x1: f64 = x1.as_str().parse().unwrap_or(0.0);
                        let y1: f64 = y1.as_str().parse().unwrap_or(0.0);
                        let x2: f64 = x2.as_str().parse().unwrap_or(0.0);
                        let y2: f64 = y2.as_str().parse().unwrap_or(0.0);
                        let name = names
                            .next()
                            .filter(|n| !n.is_empty())
                            .unwrap_or_else(|| format!("Artboard_{}", result.artboards.len() + 1));
                        if !result.artboards.iter().any(|a: &Artboard| a.name == name) {
                            result.artboards.push(Artboard {
                                name,
                                x: x1,
                                y: y1,
                                width: (x2 - x1).abs(),
                                height: (y2 - y1).abs(),
                            });
                        }
                    }
                }
            }
        }

        // Fallback: %%AI_ArtboardRect x1 y1 x2 y2
        if result.artboards.is_empty() {
            for caps in ai_artboard_rect_re.captures_iter(content_str) {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                    (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
                {
                    let x1: f64 = x1.as_str().parse().unwrap_or(0.0);
                    let y1: f64 = y1.as_str().parse().unwrap_or(0.0);
                    let x2: f64 = x2.as_str().parse().unwrap_or(0.0);
                    let y2: f64 = y2.as_str().parse().unwrap_or(0.0);
                    let name = format!("Artboard_{}", result.artboards.len() + 1);
                    if !result.artboards.iter().any(|a: &Artboard| a.name == name) {
                        result.artboards.push(Artboard {
                            name,
                            x: x1,
                            y: y1,
                            width: (x2 - x1).abs(),
                            height: (y2 - y1).abs(),
                        });
                    }
                }
            }
        }

        if result.ai_version.is_empty() {
            let version = extract_ai_version(content_str);
            if !version.is_empty() {
                result.ai_version = version;
            }
        }
    }

    // Fallback: %%HiResBoundingBox or %%BoundingBox
    if result.artboards.is_empty() {
        if let Ok(bytes) = std::fs::read(path) {
            let full_content = String::from_utf8_lossy(&bytes);
            static BBOX_RE: OnceLock<Regex> = OnceLock::new();
            let re = BBOX_RE.get_or_init(|| {
                Regex::new(r"%%(?:HiRes)?BoundingBox:\s*(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)").unwrap()
            });
            if let Some(caps) = re.captures(&full_content) {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                    (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
                {
                    let x1: f64 = x1.as_str().parse().unwrap_or(0.0);
                    let y1: f64 = y1.as_str().parse().unwrap_or(0.0);
                    let x2: f64 = x2.as_str().parse().unwrap_or(0.0);
                    let y2: f64 = y2.as_str().parse().unwrap_or(0.0);
                    result.artboards.push(Artboard {
                        name: "Artboard_1".to_string(),
                        x: x1,
                        y: y1,
                        width: (x2 - x1).abs(),
                        height: (y2 - y1).abs(),
                    });
                }
            }
        }
    }

    // Parse page tiles from ArtBox entries in PDF stream
    if result.page_tiles.is_empty() {
        if let Ok(bytes) = std::fs::read(path) {
            let full_content = String::from_utf8_lossy(&bytes);
            static ARTBOX_RE: OnceLock<Regex> = OnceLock::new();
            let re = ARTBOX_RE.get_or_init(|| {
                Regex::new(
                    r"ArtBox\[(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\]",
                )
                .unwrap()
            });
            for (i, caps) in re.captures_iter(&full_content).enumerate() {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                    (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
                {
                    let x1: f64 = x1.as_str().parse().unwrap_or(0.0);
                    let y1: f64 = y1.as_str().parse().unwrap_or(0.0);
                    let x2: f64 = x2.as_str().parse().unwrap_or(0.0);
                    let y2: f64 = y2.as_str().parse().unwrap_or(0.0);
                    result.page_tiles.push(PageTile {
                        name: format!("Page_{}", i + 1),
                        x: x1,
                        y: y1,
                        width: (x2 - x1).abs(),
                        height: (y2 - y1).abs(),
                    });
                }
            }
        }
    }

    // Parse tile region from %AI3_TileBox
    if result.page_tiles.is_empty() {
        if let Ok(bytes) = std::fs::read(path) {
            let full_content = String::from_utf8_lossy(&bytes);
            static TILEBOX_RE: OnceLock<Regex> = OnceLock::new();
            let re = TILEBOX_RE.get_or_init(|| {
                Regex::new(r"%AI3_TileBox:\s*(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)").unwrap()
            });
            if let Some(caps) = re.captures(&full_content) {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                    (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
                {
                    let x1: f64 = x1.as_str().parse().unwrap_or(0.0);
                    let y1: f64 = y1.as_str().parse().unwrap_or(0.0);
                    let x2: f64 = x2.as_str().parse().unwrap_or(0.0);
                    let y2: f64 = y2.as_str().parse().unwrap_or(0.0);
                    result.page_tiles.push(PageTile {
                        name: "TileRegion_1".to_string(),
                        x: x1,
                        y: y1,
                        width: (x2 - x1).abs(),
                        height: (y2 - y1).abs(),
                    });
                }
            }
        }
    }

    // If page tiles exist, prefer them over a single bounding-box artboard
    // (page tiles are more specific than a whole-canvas %%HiResBoundingBox)
    if !result.page_tiles.is_empty()
        && (result.artboards.is_empty()
            || (result.artboards.len() == 1 && !result.page_tiles.is_empty()))
    {
        result.artboards.clear();
        for tile in &result.page_tiles {
            result.artboards.push(Artboard {
                name: tile.name.clone(),
                x: tile.x,
                y: tile.y,
                width: tile.width,
                height: tile.height,
            });
        }
    }

    if result.ai_version.is_empty() {
        // Try to get version from document info dictionary
        if let Ok(info_ref) = doc.trailer.get(b"Info") {
            if let Ok(info_id) = info_ref.as_reference() {
                if let Ok(info) = doc.get_object(info_id) {
                    if let Ok(dict) = info.as_dict() {
                        if let Ok(creator_obj) = dict.get(b"Creator") {
                            if let Ok(s) = creator_obj.as_string() {
                                let creator_str = String::from_utf8_lossy(s.as_bytes()).to_string();
                                if let Ok(re) = Regex::new(r"Adobe Illustrator[^\d]*([\d.]+)") {
                                    if let Some(caps) = re.captures(&creator_str) {
                                        result.ai_version = caps
                                            .get(1)
                                            .map(|m| m.as_str().to_string())
                                            .unwrap_or_default();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Promote CTM/path candidates into transform-backed elements so they reach downstream
    // generation without leaking the internal `ctm_element_*` pseudo IDs into public output.
    for (idx, candidate) in result.transform_candidates.iter_mut().enumerate() {
        if candidate.is_pseudo_element {
            candidate.is_pseudo_element = false;
            candidate.id = format!("transform_candidate_{}", idx);
            candidate.element_type = Some("shape".to_string());
        }
    }
    result.elements.append(&mut result.transform_candidates);

    // Populate stable matching metadata
    let all_artboards: Vec<_> = result
        .artboards
        .iter()
        .map(|a| (a.name.clone(), a.x, a.y, a.x + a.width, a.y + a.height))
        .collect();
    for elem in &mut result.elements {
        let (x, y, w, h) = element_bounds(elem);
        elem.bounds = Some([x, y, w, h]);

        // Assign artboard_name by bounds if not already set
        if elem.artboard_name.is_none() {
            let mut assigned = false;
            for (name, ax, ay, ax2, ay2) in &all_artboards {
                let aw = ax2 - ax;
                let ah = ay2 - ay;
                if x < ax + aw && x + w > *ax && y < ay + ah && y + h > *ay {
                    elem.artboard_name = Some(name.clone());
                    assigned = true;
                    break;
                }
            }
            if !assigned && !all_artboards.is_empty() {
                elem.artboard_name = Some(all_artboards[0].0.clone());
            }
        }

        if elem.name.is_none() {
            elem.name = elem.artboard_name.clone();
        }
        if elem.element_type.is_none() {
            if !elem.path_points.is_empty() {
                elem.element_type = Some("path".to_string());
            } else if elem.is_pseudo_element {
                elem.element_type = Some("transform".to_string());
            } else {
                elem.element_type = Some("shape".to_string());
            }
        }
    }

    Ok(result)
}
