// egui_expressive Illustrator Exporter
// UXP Plugin for Adobe Illustrator 2021+

const { app, core } = require("photoshop"); // UXP uses same require pattern
// For Illustrator UXP:
const ai = require("illustrator");

// ─── Artboard Discovery ───────────────────────────────────────────────────────

async function getArtboards() {
  const doc = app.activeDocument;
  if (!doc) return [];

  const boards = [];
  for (let i = 0; i < doc.artboards.length; i++) {
    const ab = doc.artboards[i];
    const rect = ab.artboardRect; // [left, top, right, bottom]
    boards.push({
      index: i,
      name: ab.name,
      width: Math.abs(rect[2] - rect[0]),
      height: Math.abs(rect[3] - rect[1]),
      x: rect[0],
      y: rect[1],
    });
  }
  return boards;
}

// ─── Element Extraction ───────────────────────────────────────────────────────

function extractElements(pageItem, artboardRect) {
  const elements = [];
  extractRecursive(pageItem, artboardRect, elements, 0);
  return elements;
}

function extractRecursive(item, artboardRect, elements, depth) {
  if (!item.visible) return;

  const bounds = item.geometricBounds; // [left, top, right, bottom]
  const x = bounds[0] - artboardRect[0];
  const y = artboardRect[1] - bounds[1]; // flip Y (Illustrator Y is inverted)
  const w = Math.abs(bounds[2] - bounds[0]);
  const h = Math.abs(bounds[1] - bounds[3]);

  const el = {
    id: item.name || `element_${elements.length}`,
    type: getElementType(item),
    x, y, w, h,
    depth,
    fill: getFill(item),
    stroke: getStroke(item),
    text: item.typename === "TextFrame" ? item.contents : null,
    textStyle: item.typename === "TextFrame" ? getTextStyle(item) : null,
    children: [],
    // Extended properties
    opacity: item.opacity !== undefined ? item.opacity / 100.0 : 1.0,
    rotation: item.rotation !== undefined ? item.rotation : 0,
    cornerRadius: item.typename === "PathItem" && item.cornerRadius !== undefined ? item.cornerRadius : 0,
    gradient: getGradient(item),
    blendMode: item.blendingMode || "normal",
    strokeDash: getStrokeDash(item),
    textAlign: item.typename === "TextFrame" ? getTextAlign(item) : null,
    letterSpacing: item.typename === "TextFrame" ? getLetterSpacing(item) : null,
    lineHeight: item.typename === "TextFrame" ? getLineHeight(item) : null,
  };

  if (item.typename === "GroupItem" && item.pageItems) {
    for (let i = 0; i < item.pageItems.length; i++) {
      extractRecursive(item.pageItems[i], artboardRect, el.children, depth + 1);
    }
  }

  elements.push(el);
}

function getElementType(item) {
  switch (item.typename) {
    case "TextFrame": return "text";
    case "PathItem": return item.closed ? "shape" : "path";
    case "GroupItem": return "group";
    case "RasterItem": return "image";
    case "PlacedItem": return "image";
    case "CompoundPathItem": return "shape";
    default: return "unknown";
  }
}

function getFill(item) {
  try {
    if (item.filled && item.fillColor) {
      const c = item.fillColor;
      if (c.typename === "RGBColor") {
        return { r: Math.round(c.red), g: Math.round(c.green), b: Math.round(c.blue), a: 255 };
      }
      if (c.typename === "CMYKColor") {
        // Convert CMYK to RGB
        const k = c.black / 100;
        const r = Math.round(255 * (1 - c.cyan/100) * (1 - k));
        const g = Math.round(255 * (1 - c.magenta/100) * (1 - k));
        const b = Math.round(255 * (1 - c.yellow/100) * (1 - k));
        return { r, g, b, a: 255 };
      }
      if (c.typename === "GrayColor") {
        const v = Math.round(255 * (1 - c.gray/100));
        return { r: v, g: v, b: v, a: 255 };
      }
    }
  } catch(e) {}
  return null;
}

function getStroke(item) {
  try {
    if (item.stroked && item.strokeColor) {
      const c = item.strokeColor;
      const width = item.strokeWidth || 1;
      if (c.typename === "RGBColor") {
        return { r: Math.round(c.red), g: Math.round(c.green), b: Math.round(c.blue), width };
      }
    }
  } catch(e) {}
  return null;
}

function getTextStyle(item) {
  try {
    const chars = item.textRange.characterAttributes;
    return {
      size: chars.size || 14,
      weight: chars.textFont ? (chars.textFont.name.includes("Bold") ? 700 : 400) : 400,
      family: chars.textFont ? chars.textFont.name : "default",
    };
  } catch(e) {
    return { size: 14, weight: 400, family: "default" };
  }
}

function getGradient(item) {
  try {
    if (item.fillColor && item.fillColor.typename === "GradientColor") {
      const grad = item.fillColor.gradient;
      const angle = item.fillColor.angle || 0;
      return {
        type: grad.type === 1 ? "linear" : "radial",
        angle: angle,
        stops: grad.gradientStops.map(s => ({
          position: s.rampPoint / 100,
          color: colorToHex(s.color),
        })),
      };
    }
  } catch(e) {}
  return null;
}

function colorToHex(c) {
  try {
    if (c.typename === "RGBColor") {
      const r = Math.round(c.red);
      const g = Math.round(c.green);
      const b = Math.round(c.blue);
      return `rgba(${r},${g},${b},1)`;
    }
  } catch(e) {}
  return null;
}

function getStrokeDash(item) {
  try {
    if (item.strokeDashes && item.strokeDashes.length > 0) {
      return item.strokeDashes;
    }
  } catch(e) {}
  return null;
}

function getTextAlign(item) {
  if (item.typename !== "TextFrame") return null;
  try {
    const just = item.textRange.paragraphAttributes.justification;
    if (just === Justification.LEFT) return "left";
    if (just === Justification.CENTER) return "center";
    if (just === Justification.RIGHT) return "right";
    if (just === Justification.FULLJUSTIFY) return "justified";
  } catch(e) {}
  return "left";
}

function getLetterSpacing(item) {
  if (item.typename !== "TextFrame") return null;
  try {
    return item.textRange.characterAttributes.tracking / 1000.0;
  } catch(e) { return null; }
}

function getLineHeight(item) {
  if (item.typename !== "TextFrame") return null;
  try {
    const leading = item.textRange.characterAttributes.leading;
    return leading > 0 ? leading : null;
  } catch(e) { return null; }
}

// ─── Color Extraction ─────────────────────────────────────────────────────────

function extractColors(elements) {
  const colors = new Map();
  function walk(els) {
    for (const el of els) {
      if (el.fill) {
        const key = `${el.fill.r},${el.fill.g},${el.fill.b}`;
        colors.set(key, el.fill);
      }
      if (el.children) walk(el.children);
    }
  }
  walk(elements);
  return Array.from(colors.values());
}

// ─── Layout Inference ─────────────────────────────────────────────────────────

function inferLayout(elements, options) {
  // Sort by Y then X
  const sorted = [...elements].sort((a, b) => a.y - b.y || a.x - b.x);

  // Cluster into rows by Y overlap
  const rows = clusterIntoRows(sorted);

  return { rows, elements: sorted };
}

function clusterIntoRows(elements) {
  if (elements.length === 0) return [];
  const rows = [];
  let currentRow = [elements[0]];

  for (let i = 1; i < elements.length; i++) {
    const el = elements[i];
    const prev = currentRow[currentRow.length - 1];
    // Y overlap check: elements are in the same row if their Y ranges overlap
    const prevBottom = prev.y + prev.h;
    const elMid = el.y + el.h / 2;
    if (elMid < prevBottom) {
      currentRow.push(el);
    } else {
      rows.push(currentRow);
      currentRow = [el];
    }
  }
  rows.push(currentRow);
  return rows;
}

function inferGap(elements) {
  if (elements.length < 2) return 0;
  const sorted = [...elements].sort((a, b) => a.x - b.x);
  const gaps = [];
  for (let i = 1; i < sorted.length; i++) {
    const gap = sorted[i].x - (sorted[i-1].x + sorted[i-1].w);
    if (gap > 0) gaps.push(gap);
  }
  if (gaps.length === 0) return 0;
  // Return median gap
  gaps.sort((a, b) => a - b);
  return Math.round(gaps[Math.floor(gaps.length / 2)]);
}

// ─── Naming Convention Parser ─────────────────────────────────────────────────

function parseNamingConvention(name) {
  const lower = name.toLowerCase();

  // Layout direction
  if (lower.startsWith("row-") || lower.startsWith("hstack-")) return { layout: "horizontal", label: name.slice(name.indexOf("-")+1) };
  if (lower.startsWith("col-") || lower.startsWith("vstack-")) return { layout: "vertical", label: name.slice(name.indexOf("-")+1) };
  if (lower.startsWith("panel-")) return { layout: "panel", label: name.slice(6) };
  if (lower.startsWith("grid-")) return { layout: "grid", label: name.slice(5) };
  if (lower.startsWith("scroll-")) return { layout: "scroll", label: name.slice(7) };

  // Widget types
  if (lower.startsWith("btn-") || lower.startsWith("button-")) return { widget: "button", label: name.slice(name.indexOf("-")+1) };
  if (lower.startsWith("input-") || lower.startsWith("field-")) return { widget: "text_edit", label: name.slice(name.indexOf("-")+1) };
  if (lower.startsWith("label-") || lower.startsWith("text-")) return { widget: "label", label: name.slice(name.indexOf("-")+1) };
  if (lower.startsWith("img-") || lower.startsWith("image-")) return { widget: "image", label: name.slice(name.indexOf("-")+1) };
  if (lower.startsWith("icon-")) return { widget: "icon", label: name.slice(5) };
  if (lower.startsWith("card-")) return { widget: "card", label: name.slice(5) };
  if (lower.startsWith("divider-") || lower === "divider") return { widget: "divider", label: "" };
  if (lower.startsWith("spacer-") || lower === "spacer") return { widget: "spacer", label: "" };
  if (lower.startsWith("badge-")) return { widget: "badge", label: name.slice(6) };
  if (lower.startsWith("chip-")) return { widget: "chip", label: name.slice(5) };
  if (lower.startsWith("toggle-") || lower.startsWith("switch-")) return { widget: "toggle", label: name.slice(name.indexOf("-")+1) };
  if (lower.startsWith("slider-") || lower.startsWith("knob-")) return { widget: "slider", label: name.slice(name.indexOf("-")+1) };

  // Gap hints: gap-8, gap-16
  const gapMatch = lower.match(/^gap-(\d+)$/);
  if (gapMatch) return { gap: parseInt(gapMatch[1]) };

  return null;
}

// ─── Rust Code Generator ──────────────────────────────────────────────────────

function colorToRust(fill) {
  if (!fill) return "egui::Color32::TRANSPARENT";
  return `egui::Color32::from_rgb(${fill.r}, ${fill.g}, ${fill.b})`;
}

function generateRustForElement(el, indent, options) {
  const pad = "    ".repeat(indent);
  const conv = options.useNaming ? parseNamingConvention(el.id) : null;

  // Named widget override
  if (conv) {
    if (conv.widget === "button") {
      return `${pad}if ui.button("${conv.label}").clicked() {\n${pad}    // TODO: handle click\n${pad}}\n`;
    }
    if (conv.widget === "label") {
      return `${pad}ui.label("${conv.label}");\n`;
    }
    if (conv.widget === "text_edit") {
      return `${pad}// TODO: bind to state\n${pad}// ui.text_edit_singleline(&mut self.${conv.label.replace(/-/g,"_")});\n`;
    }
    if (conv.widget === "divider") {
      return `${pad}ui.separator();\n`;
    }
    if (conv.widget === "spacer") {
      return `${pad}ui.add_space(8.0);\n`;
    }
    if (conv.widget === "badge") {
      return `${pad}ui.add(egui_expressive::Badge::new("${conv.label}"));\n`;
    }
    if (conv.widget === "icon") {
      return `${pad}// TODO: ui.add(egui_expressive::Icon::new(egui_expressive::icon_constants::${conv.label.toUpperCase()}));\n`;
    }
    if (conv.widget === "toggle") {
      return `${pad}// TODO: bind to state\n${pad}// ui.checkbox(&mut self.${conv.label.replace(/-/g,"_")}, "${conv.label}");\n`;
    }
    if (conv.widget === "card") {
      const bg = colorToRust(el.fill);
      return `${pad}egui::Frame::NONE.fill(${bg}).corner_radius(8u8).inner_margin(12i8).show(ui, |ui| {\n${pad}    // TODO: card contents for "${conv.label}"\n${pad}});\n`;
    }
    if (conv.layout === "horizontal") {
      const gap = options.inferGaps ? inferGap(el.children || []) : 8;
      let code = `${pad}ui.horizontal(|ui| {\n`;
      code += `${pad}    ui.spacing_mut().item_spacing.x = ${gap}.0;\n`;
      for (const child of (el.children || [])) {
        code += generateRustForElement(child, indent + 1, options);
      }
      code += `${pad}});\n`;
      return code;
    }
    if (conv.layout === "vertical") {
      const gap = options.inferGaps ? inferGap(el.children || []) : 8;
      let code = `${pad}ui.vertical(|ui| {\n`;
      code += `${pad}    ui.spacing_mut().item_spacing.y = ${gap}.0;\n`;
      for (const child of (el.children || [])) {
        code += generateRustForElement(child, indent + 1, options);
      }
      code += `${pad}});\n`;
      return code;
    }
    if (conv.layout === "scroll") {
      let code = `${pad}egui::ScrollArea::vertical().show(ui, |ui| {\n`;
      for (const child of (el.children || [])) {
        code += generateRustForElement(child, indent + 1, options);
      }
      code += `${pad}});\n`;
      return code;
    }
  }

  // Auto-infer from structure
  if (el.type === "text" && el.text) {
    const size = el.textStyle ? el.textStyle.size : 14;
    const color = colorToRust(el.fill);
    return `${pad}ui.label(egui::RichText::new("${el.text.replace(/"/g, '\\"')}").size(${size}.0).color(${color}));\n`;
  }

  if (el.type === "group" && el.children && el.children.length > 0) {
    // Infer row vs column from children positions
    const isRow = isHorizontalLayout(el.children);
    const gap = options.inferGaps ? inferGap(el.children) : 8;
    const bg = el.fill ? `.fill(${colorToRust(el.fill)})` : "";
    let code = "";
    if (el.fill) {
      code += `${pad}egui::Frame::NONE${bg}.show(ui, |ui| {\n`;
      indent++;
    }
    const layoutFn = isRow ? "horizontal" : "vertical";
    const spacingAxis = isRow ? "x" : "y";
    code += `${"    ".repeat(indent)}ui.${layoutFn}(|ui| {\n`;
    code += `${"    ".repeat(indent+1)}ui.spacing_mut().item_spacing.${spacingAxis} = ${gap}.0;\n`;
    for (const child of el.children) {
      code += generateRustForElement(child, indent + 2, options);
    }
    code += `${"    ".repeat(indent)}}); // ${el.id}\n`;
    if (el.fill) {
      indent--;
      code += `${pad}});\n`;
    }
    return code;
  }

  if (el.type === "shape" || el.type === "path") {
    const fill = colorToRust(el.fill);
    const stroke = el.stroke
      ? `egui::Stroke::new(${el.stroke.width}.0, egui::Color32::from_rgb(${el.stroke.r}, ${el.stroke.g}, ${el.stroke.b}))`
      : "egui::Stroke::NONE";
    const opacityComment = el.opacity !== 1.0 ? ` opacity: ${el.opacity.toFixed(2)}` : "";
    const rotationComment = el.rotation !== 0 ? ` rotation: ${el.rotation}°` : "";
    const cornerComment = el.cornerRadius > 0 ? ` corner_radius: ${el.cornerRadius}` : "";
    return `${pad}// Shape: ${el.id} (${Math.round(el.w)}×${Math.round(el.h)})${opacityComment}${rotationComment}${cornerComment}\n${pad}painter.rect_filled(\n${pad}    egui::Rect::from_min_size(origin + egui::vec2(${Math.round(el.x)}.0, ${Math.round(el.y)}.0), egui::vec2(${Math.round(el.w)}.0, ${Math.round(el.h)}.0)),\n${pad}    0.0,\n${pad}    ${fill},\n${pad});\n`;
  }

  if (el.type === "image") {
    return `${pad}// TODO: Image "${el.id}" at (${Math.round(el.x)}, ${Math.round(el.y)}) size ${Math.round(el.w)}×${Math.round(el.h)}\n`;
  }

  return `${pad}// Unknown element: ${el.id} (${el.type})\n`;
}

function isHorizontalLayout(children) {
  if (children.length < 2) return true;
  const sorted = [...children].sort((a, b) => a.x - b.x);
  let xSpread = 0, ySpread = 0;
  for (let i = 1; i < sorted.length; i++) {
    xSpread += Math.abs(sorted[i].x - sorted[i-1].x);
    ySpread += Math.abs(sorted[i].y - sorted[i-1].y);
  }
  return xSpread > ySpread;
}

// ─── Full Artboard Code Generator ────────────────────────────────────────────

function generateArtboardCode(artboard, elements, colors, options) {
  const fnName = artboard.name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_|_$/g, "");

  let code = `// Auto-generated by egui_expressive Illustrator Exporter\n`;
  code += `// Artboard: "${artboard.name}" (${Math.round(artboard.width)} × ${Math.round(artboard.height)} px)\n`;
  code += `// Generated: ${new Date().toISOString()}\n`;
  code += `//\n`;
  code += `// USAGE: Call draw_${fnName}(ui) inside your egui update function.\n`;
  code += `// TODO: Replace placeholder values with your actual state bindings.\n\n`;
  code += `#[allow(unused_variables, dead_code)]\n`;
  code += `pub fn draw_${fnName}(ui: &mut egui::Ui) {\n`;
  code += `    let origin = ui.cursor().min;\n`;
  code += `    let painter = ui.painter();\n\n`;

  // Background
  if (colors.length > 0) {
    const bg = colors[0];
    code += `    // Background\n`;
    code += `    painter.rect_filled(\n`;
    code += `        egui::Rect::from_min_size(origin, egui::vec2(${Math.round(artboard.width)}.0, ${Math.round(artboard.height)}.0)),\n`;
    code += `        0.0,\n`;
    code += `        egui::Color32::from_rgb(${bg.r}, ${bg.g}, ${bg.b}),\n`;
    code += `    );\n\n`;
  }

  // Elements
  for (const el of elements) {
    code += generateRustForElement(el, 1, options);
  }

  code += `}\n`;
  return code;
}

// ─── JSON Sidecar Generator ───────────────────────────────────────────────────

function generateSidecar(artboard, elements, colors) {
  return JSON.stringify({
    artboard: {
      name: artboard.name,
      width: artboard.width,
      height: artboard.height,
    },
    colors: colors.map(c => ({ r: c.r, g: c.g, b: c.b })),
    elements: elements.map(el => ({
      id: el.id,
      type: el.type,
      x: Math.round(el.x),
      y: Math.round(el.y),
      w: Math.round(el.w),
      h: Math.round(el.h),
      text: el.text,
      textStyle: el.textStyle,
      // Extended properties
      opacity: el.opacity !== 1.0 ? el.opacity : undefined,
      rotation: el.rotation !== 0 ? el.rotation : undefined,
      cornerRadius: el.cornerRadius > 0 ? el.cornerRadius : undefined,
      gradient: el.gradient,
      blendMode: el.blendMode !== "normal" ? el.blendMode : undefined,
      strokeDash: el.strokeDash,
      textAlign: el.textAlign,
      letterSpacing: el.letterSpacing,
      lineHeight: el.lineHeight,
    })),
  }, null, 2);
}

// ─── Main Export Handler ──────────────────────────────────────────────────────

async function exportArtboards(selectedIndices, options) {
  const doc = app.activeDocument;
  if (!doc) throw new Error("No active document");

  const results = [];

  for (const idx of selectedIndices) {
    const ab = doc.artboards[idx];
    const rect = ab.artboardRect;

    // Get all page items on this artboard
    const items = [];
    for (let i = 0; i < doc.pageItems.length; i++) {
      const item = doc.pageItems[i];
      const bounds = item.geometricBounds;
      // Check if item is within artboard bounds
      if (bounds[0] >= rect[0] && bounds[2] <= rect[2] &&
          bounds[3] >= rect[3] && bounds[1] <= rect[1]) {
        items.push(item);
      }
    }

    const artboardInfo = {
      name: ab.name,
      width: Math.abs(rect[2] - rect[0]),
      height: Math.abs(rect[3] - rect[1]),
      x: rect[0],
      y: rect[1],
    };

    const elements = [];
    for (const item of items) {
      extractRecursive(item, rect, elements, 0);
    }

    const colors = extractColors(elements);
    const code = generateArtboardCode(artboardInfo, elements, colors, options);
    const sidecar = options.includeSidecar ? generateSidecar(artboardInfo, elements, colors) : null;

    results.push({
      name: artboardInfo.name,
      filename: artboardInfo.name.toLowerCase().replace(/[^a-z0-9]+/g, "_") + ".rs",
      code,
      sidecar,
      sidecarFilename: artboardInfo.name.toLowerCase().replace(/[^a-z0-9]+/g, "_") + ".json",
    });
  }

  return results;
}

// ─── UI Message Handler ───────────────────────────────────────────────────────

window.addEventListener("message", async (event) => {
  const { type, payload } = event.data;

  if (type === "GET_ARTBOARDS") {
    try {
      const boards = await getArtboards();
      window.postMessage({ type: "ARTBOARDS_RESULT", boards });
    } catch (e) {
      window.postMessage({ type: "ERROR", message: e.message });
    }
  }

  if (type === "EXPORT") {
    try {
      const { selectedIndices, options } = payload;
      const results = await exportArtboards(selectedIndices, options);
      window.postMessage({ type: "EXPORT_RESULT", results });
    } catch (e) {
      window.postMessage({ type: "ERROR", message: e.message });
    }
  }
});

// Notify panel that plugin is ready
window.postMessage({ type: "READY" });
