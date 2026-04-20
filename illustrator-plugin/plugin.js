// egui_expressive Illustrator Exporter — UXP Plugin for Adobe Illustrator 2021+
"use strict";

const BLEND_MODES = { NORMAL: "normal", MULTIPLY: "multiply", SCREEN: "screen", OVERLAY: "overlay", DARKEN: "darken", LIGHTEN: "lighten", COLORDODGE: "color_dodge", COLORBURN: "color_burn", HARDLIGHT: "hard_light", SOFTLIGHT: "soft_light", DIFFERENCE: "difference", EXCLUSION: "exclusion", HUE: "hue", SATURATION: "saturation", COLOR: "color", LUMINOSITY: "luminosity" };
const BLEND_MODES_BY_NUM = { 0: "normal", 1: "multiply", 2: "screen", 3: "overlay", 4: "darken", 5: "lighten", 6: "color_dodge", 7: "color_burn", 8: "hard_light", 9: "soft_light", 10: "difference", 11: "exclusion", 12: "hue", 13: "saturation", 14: "color", 15: "luminosity" };

// ─── Artboard Discovery ───────────────────────────────────────────────────────
async function getArtboards() {
  const doc = app.activeDocument;
  if (!doc) return [];
  const boards = [];
  for (let i = 0; i < doc.artboards.length; i++) {
    const ab = doc.artboards[i];
    const r = ab.artboardRect;
    boards.push({ index: i, name: ab.name, width: Math.abs(r[2] - r[0]), height: Math.abs(r[3] - r[1]), x: r[0], y: r[1] });
  }
  return boards;
}

// ─── Third-party Plugin/Effects Detection ─────────────────────────────────────
function detectThirdPartyEffects(item) {
  const effects = [];

  // MeshItem — gradient mesh, completely opaque
  if (item.typename === 'MeshItem') {
    effects.push({ type: 'gradientMesh', opaque: true,
      note: 'Gradient mesh — approximate with radial_gradient or linear_gradient_rect' });
  }

  // PluginItem — envelope distortion, 3D effects, etc.
  if (item.typename === 'PluginItem') {
    const isTracing = item.isTracing || false;
    effects.push({
      type: isTracing ? 'liveTrace' : 'envelopeOrEffect',
      opaque: true,
      note: isTracing ? 'Live Trace — rasterize for visual result' : 'Envelope/3D effect — manual recreation required'
    });
  }

  // Pattern fill — custom pattern swatch
  try {
    if (item.fillColor && item.fillColor.typename === 'PatternColor') {
      effects.push({
        type: 'patternFill',
        opaque: false,  // name is readable
        patternName: item.fillColor.pattern ? item.fillColor.pattern.name : 'unknown',
        rotation: item.fillColor.rotation || 0,
        note: 'Pattern fill — tile artwork not accessible, use pattern name as reference'
      });
    }
  } catch(e) {}

  // Art/Pattern brush stroke
  try {
    if (item.stroked && item.strokeColor && item.strokeColor.typename === 'PatternColor') {
      effects.push({
        type: 'brushStroke',
        opaque: true,
        brushName: item.strokeColor.pattern ? item.strokeColor.pattern.name : 'unknown',
        note: 'Art/Pattern brush stroke — expand to get path data'
      });
    }
  } catch(e) {}

  // Detect live effects via expand+compare (expensive, only if item has complex appearance)
  try {
    if (item.typename === 'PathItem' || item.typename === 'GroupItem') {
      const hasComplexAppearance = detectComplexAppearance(item);
      if (hasComplexAppearance) {
        effects.push({
          type: 'liveEffect',
          opaque: true,
          note: 'Live effect detected (Phantasm/Astute/etc.) — visual approximation only, parameters not accessible'
        });
      }
    }
  } catch(e) {}

  return effects;
}

function detectComplexAppearance(item) {
  // Heuristic: items with live effects often have unusual bounds or typename changes after expand
  try {
    // Check if item has non-default graphic style
    if (item.graphicStyle && item.graphicStyle.name !== 'Default Graphic Style') {
      return true;
    }
  } catch(e) {}
  return false;
}

// ─── Element Extraction ──────────────────────────────────────────────────────
function extractElements(pageItems, artboardRect) {
  const elements = [];
  for (const item of pageItems) extractRecursive(item, artboardRect, elements, 0);
  return elements;
}

function extractRecursive(item, artboardRect, elements, depth) {
  try { if (item.locked || !item.visible) return; } catch (e) { return; }

  let x = 0, y = 0, w = 0, h = 0;
  try {
    const b = item.geometricBounds;
    x = b[0] - artboardRect[0]; y = artboardRect[1] - b[1];
    w = Math.abs(b[2] - b[0]); h = Math.abs(b[1] - b[3]);
  } catch (e) {
    try {
      const b = item.visibleBounds;
      x = b[0] - artboardRect[0]; y = artboardRect[1] - b[1];
      w = Math.abs(b[2] - b[0]); h = Math.abs(b[1] - b[3]);
    } catch (e2) { return; }
  }

  const el = {
    id: item.name || `el_${elements.length}`, type: getElementType(item), x, y, w, h, depth,
    fill: getFill(item), stroke: getStroke(item), text: null, textStyle: null, children: [],
    opacity: 1.0, rotation: 0, cornerRadius: 0, gradient: null, blendMode: "normal",
    strokeCap: null, strokeJoin: null, strokeDash: null, strokeMiterLimit: null,
    effects: [], textDecoration: null, textTransform: null, textRuns: null,
    textAlign: null, letterSpacing: null, lineHeight: null, clipMask: false,
    symbolName: null, isCompoundPath: false, isGradientMesh: false, isChart: false, notes: [],
    pathPoints: null, pathClosed: false, imagePath: null
  };

  // Path geometry extraction
  try {
    if ((item.typename === "PathItem" || item.typename === "CompoundPathItem") && item.pathPoints) {
      const pts = [];
      for (let pi = 0; pi < item.pathPoints.length; pi++) {
        const pp = item.pathPoints[pi];
        try {
          pts.push({
            anchor: [pp.anchor[0] - artboardRect[0], artboardRect[1] - pp.anchor[1]],
            leftDir: [pp.leftDirection[0] - artboardRect[0], artboardRect[1] - pp.leftDirection[1]],
            rightDir: [pp.rightDirection[0] - artboardRect[0], artboardRect[1] - pp.rightDirection[1]],
            kind: pp.pointType === PointType.SMOOTH ? "smooth" : "corner"
          });
        } catch(ppe) {}
      }
      if (pts.length > 0) {
        el.pathPoints = pts;
        el.pathClosed = item.closed || false;
      }
    }
  } catch(e) {}

  // Image/placed file extraction
  try {
    if (item.typename === "PlacedItem" && item.file) {
      el.imagePath = item.file.fsName || item.file.name || null;
    }
  } catch(e) {}
  try {
    if (item.typename === "RasterItem") {
      el.imagePath = `raster_${Math.round(el.w)}x${Math.round(el.h)}`;
    }
  } catch(e) {}

  try { el.opacity = item.opacity !== undefined ? item.opacity / 100 : 1; } catch (e) {}
  try { el.rotation = item.rotation !== undefined ? item.rotation : 0; } catch (e) {}
  try { if (item.typename === "PathItem" && item.cornerRadius !== undefined) el.cornerRadius = item.cornerRadius; } catch (e) {}

  // Stroke details
  try { if (item.strokeCap !== undefined) el.strokeCap = { 0: "butt", 1: "round", 2: "square" }[item.strokeCap] || "butt"; } catch (e) {}
  try { if (item.strokeJoin !== undefined) el.strokeJoin = { 0: "miter", 1: "round", 2: "bevel" }[item.strokeJoin] || "miter"; } catch (e) {}
  try { if (item.strokeDashes?.length > 0) el.strokeDash = [...item.strokeDashes]; } catch (e) {}
  try { if (item.strokeMiterLimit !== undefined) el.strokeMiterLimit = item.strokeMiterLimit; } catch (e) {}

  // Blend mode
  try {
    if (item.blendingMode !== undefined) {
      const BLEND_MODE_MAP = {
        "BlendModes.NORMAL": "normal",
        "BlendModes.MULTIPLY": "multiply", 
        "BlendModes.SCREEN": "screen",
        "BlendModes.OVERLAY": "overlay",
        "BlendModes.DARKEN": "darken",
        "BlendModes.LIGHTEN": "lighten",
        "BlendModes.COLORDODGE": "color_dodge",
        "BlendModes.COLORBURN": "color_burn",
        "BlendModes.HARDLIGHT": "hard_light",
        "BlendModes.SOFTLIGHT": "soft_light",
        "BlendModes.DIFFERENCE": "difference",
        "BlendModes.EXCLUSION": "exclusion",
        "BlendModes.HUE": "hue",
        "BlendModes.SATURATIONBLEND": "saturation",
        "BlendModes.COLORBLEND": "color",
        "BlendModes.LUMINOSITY": "luminosity",
      };
      el.blendMode = BLEND_MODE_MAP[String(item.blendingMode)] || "normal";
    }
  } catch (e) {}

  el.gradient = getGradient(item);

  if (item.typename === "TextFrame") {
    try { el.text = item.contents || ""; } catch (e) { el.text = ""; }
    el.textStyle = getTextStyle(item);
    el.textAlign = getTextAlign(item);
    el.letterSpacing = getLetterSpacing(item);
    el.lineHeight = getLineHeight(item);
    el.textDecoration = getTextDecoration(item);
    el.textTransform = getTextTransform(item);
    el.textRuns = getTextRuns(item);
  }

  try { if (item.clipping || item.clipped) { el.clipMask = true; el.notes.push("clipping mask"); } } catch (e) {}
  try { if (item.typename === "CompoundPathItem") { el.isCompoundPath = true; el.notes.push("compound path"); } } catch (e) {}


  // SymbolItem — explicit handling with full metadata
  try {
    if (item.typename === "SymbolItem") {
      el.type = 'symbol';
      el.symbolName = item.symbol ? item.symbol.name : 'unknown';
      el.note = `Symbol instance: "${el.symbolName}" — expand to access contents`;
    }
  } catch (e) {}
  try { if (item.typename === "MeshItem") { el.isGradientMesh = true; el.notes.push("gradient mesh"); } } catch (e) {}
  try { if (item.typename === "GraphItem") { el.isChart = true; el.notes.push("chart/graph"); } } catch (e) {}

  el.effects = extractEffects(item);

  // Third-party plugin effects detection
  el.thirdPartyEffects = detectThirdPartyEffects(item);
  el.isOpaque = el.thirdPartyEffects.length > 0 && el.thirdPartyEffects.some(e => e.opaque);

  if (item.typename === "GroupItem") {
    try { if (item.pageItems) for (let i = 0; i < item.pageItems.length; i++) extractRecursive(item.pageItems[i], artboardRect, el.children, depth + 1); } catch (e) {}
  }
  elements.push(el);
}

function getElementType(item) {
  try {
    const t = item.typename;
    if (t === "TextFrame") return "text";
    if (t === "PathItem") return item.closed ? "shape" : "path";
    if (t === "GroupItem") return "group";
    if (t === "RasterItem" || t === "PlacedItem") return "image";
    if (t === "CompoundPathItem") return "shape";
    if (t === "SymbolItem") return "symbol";
    if (t === "MeshItem") return "mesh";
    if (t === "GraphItem") return "chart";
    if (t === "PluginItem") return "plugin";
    return "unknown";
  } catch (e) { return "unknown"; }
}

// ─── Effects Extraction ──────────────────────────────────────────────────────
function extractEffects(item) {
  const fx = [];

  // Approach 1: graphicStyles
  try {
    if (item.graphicStyles?.[0]?.attributes) {
      const a = item.graphicStyles[0].attributes;
      const parseColor = (c) => { if (!c) return { r: 0, g: 0, b: 0, a: 1 }; try { if (c.typename === "RGBColor") return { r: Math.round(c.red), g: Math.round(c.green), b: Math.round(c.blue), a: 1 }; if (c.typename === "CMYKColor") { const k = c.black/100; return { r: Math.round(255*(1-c.cyan/100)*(1-k)), g: Math.round(255*(1-c.magenta/100)*(1-k)), b: Math.round(255*(1-c.yellow/100)*(1-k)), a: 1 }; } } catch(e) {} return { r:0,g:0,b:0,a:1 }; };

      try { if (a.dropShadow?.enabled !== false) fx.push({ type: "dropShadow", x: a.dropShadow.distance||0, y: a.dropShadow.angle||0, blur: a.dropShadow.blur||0, spread: a.dropShadow.spread||0, color: parseColor(a.dropShadow.color), blendMode: a.dropShadow.blendMode||"normal" }); } catch(e) {}
      try { if (a.innerShadow?.enabled !== false) fx.push({ type: "innerShadow", x: a.innerShadow.distance||0, y: a.innerShadow.angle||0, blur: a.innerShadow.blur||0, color: parseColor(a.innerShadow.color) }); } catch(e) {}
      try { if (a.outerGlow?.enabled !== false) fx.push({ type: "outerGlow", blur: a.outerGlow.blur||0, color: parseColor(a.outerGlow.color) }); } catch(e) {}
      try { if (a.innerGlow?.enabled !== false) fx.push({ type: "innerGlow", blur: a.innerGlow.blur||0, color: parseColor(a.innerGlow.color) }); } catch(e) {}
      try { if (a.gaussianBlur?.enabled !== false) fx.push({ type: "gaussianBlur", radius: a.gaussianBlur.radius||0 }); } catch(e) {}
      try { if (a.bevel?.enabled !== false) fx.push({ type: "bevel", depth: a.bevel.depth||0, angle: a.bevel.angle||0, highlight: parseColor(a.bevel.highlight), shadow: parseColor(a.bevel.shadow) }); } catch(e) {}
      try { if (a.feather?.enabled !== false) fx.push({ type: "feather", radius: a.feather.radius||0 }); } catch(e) {}
    }
  } catch (e) {}

  // Approach 2: tags
  try { if (item.tags?.length > 0) for (const tag of item.tags) { try { const n = tag.name||""; if (n.includes("effect")||n.includes("shadow")||n.includes("glow")) fx.push({ type: "effect_from_tag", tagName: n }); } catch(e) {} } } catch (e) {}

  // Approach 3: PluginItem
  try { if (item.typename === "PluginItem") fx.push({ type: "live_effect" }); } catch (e) {}

  return fx;
}

// ─── Color/Gradient ──────────────────────────────────────────────────────────
function colorToRGB(c) {
  if (!c) return null;
  try {
    if (c.typename === "RGBColor") return { r: Math.round(c.red), g: Math.round(c.green), b: Math.round(c.blue), a: 255 };
    if (c.typename === "CMYKColor") { const k = c.black/100; return { r: Math.round(255*(1-c.cyan/100)*(1-k)), g: Math.round(255*(1-c.magenta/100)*(1-k)), b: Math.round(255*(1-c.yellow/100)*(1-k)), a: 255 }; }
    if (c.typename === "GrayColor") { const v = Math.round(255*(1-c.gray/100)); return { r: v, g: v, b: v, a: 255 }; }
  } catch (e) {}
  return null;
}

function getFill(item) {
  try { if (item.filled && item.fillColor) return colorToRGB(item.fillColor); } catch (e) {}
  return null;
}

function getStroke(item) {
  try { if (item.stroked && item.strokeColor) { const c = colorToRGB(item.strokeColor); if (c) return { ...c, width: item.strokeWidth || 1 }; } } catch (e) {}
  return null;
}

function getGradient(item) {
  try {
    if (item.fillColor?.typename === "GradientColor") {
      const grad = item.fillColor.gradient;
      if (!grad) return null;
      const angle = item.fillColor.angle || 0;
      const stops = [];
      try { for (const s of grad.gradientStops) stops.push({ position: s.rampPoint/100, color: gradientColorToRGB(s.color), opacity: s.opacity !== undefined ? s.opacity/100 : 1 }); } catch (e) {}
      return { type: grad.type === 1 ? "linear" : "radial", angle, center: item.fillColor.origin ? { x: item.fillColor.origin.x, y: item.fillColor.origin.y } : null, focalPoint: item.fillColor.focalPoint, radius: item.fillColor.radius, stops };
    }
    // PatternColor — not a gradient but handled here for consistency
    if (item.fillColor?.typename === "PatternColor") {
      return {
        type: 'pattern',
        patternName: item.fillColor.pattern ? item.fillColor.pattern.name : 'unknown',
        rotation: item.fillColor.rotation || 0,
        scale: [item.fillColor.scaleFactor || 1, item.fillColor.scaleFactor || 1]
      };
    }
  } catch (e) {}
  return null;
}

function gradientColorToRGB(c) {
  if (!c) return { r: 128, g: 128, b: 128 };
  try { if (c.typename === "RGBColor") return { r: Math.round(c.red), g: Math.round(c.green), b: Math.round(c.blue) }; if (c.typename === "CMYKColor") { const k = c.black/100; return { r: Math.round(255*(1-c.cyan/100)*(1-k)), g: Math.round(255*(1-c.magenta/100)*(1-k)), b: Math.round(255*(1-c.yellow/100)*(1-k)) }; } if (c.typename === "GrayColor") { const v = Math.round(255*(1-c.gray/100)); return { r: v, g: v, b: v }; } } catch (e) {}
  return { r: 128, g: 128, b: 128 };
}

// ─── Text Details ────────────────────────────────────────────────────────────
function getTextStyle(item) {
  try {
    const chars = item.textRange.characterAttributes;
    let size = 14, weight = 400, family = "default";
    try { size = chars.size || 14; } catch (e) {}
    try { if (chars.textFont) { const n = chars.textFont.name || ""; weight = n.includes("Bold") ? 700 : n.includes("Light") ? 300 : 400; family = n; } } catch (e) {}
    return { size, weight, family };
  } catch (e) { return { size: 14, weight: 400, family: "default" }; }
}

function getTextAlign(item) {
  if (item.typename !== "TextFrame") return null;
  try { const j = item.textRange.paragraphAttributes.justification; if (j === Justification.LEFT) return "left"; if (j === Justification.CENTER) return "center"; if (j === Justification.RIGHT) return "right"; if (j === Justification.FULLJUSTIFY) return "justified"; } catch (e) {}
  return "left";
}

function getLetterSpacing(item) { if (item.typename !== "TextFrame") return null; try { return item.textRange.characterAttributes.tracking / 1000; } catch (e) { return null; } }
function getLineHeight(item) { if (item.typename !== "TextFrame") return null; try { const l = item.textRange.characterAttributes.leading; return l > 0 ? l : null; } catch (e) { return null; } }
function getTextDecoration(item) { if (item.typename !== "TextFrame") return null; try { const u = item.textRange.characterAttributes.underline, s = item.textRange.characterAttributes.strikeThrough; if (u && s) return "underline_strikethrough"; if (u) return "underline"; if (s) return "strikethrough"; } catch (e) {} return null; }
function getTextTransform(item) { if (item.typename !== "TextFrame") return null; try { if (item.textRange.characterAttributes.allCaps) return "all_caps"; if (item.textRange.characterAttributes.smallCaps) return "small_caps"; } catch (e) {} return null; }

function getTextRuns(item) {
  if (item.typename !== "TextFrame") return null;
  try {
    const runs = [], trs = item.textRanges;
    if (trs && trs.length > 1) { for (const tr of trs) { try { const a = tr.characterAttributes; runs.push({ text: tr.contents || "", style: { size: a.size||14, weight: a.textFont?.name?.includes("Bold") ? 700 : 400, color: colorToRGB(a.fillColor) } }); } catch(e) {} } }
    return runs.length > 0 ? runs : null;
  } catch (e) { return null; }
}

// ─── Color Deduplication ──────────────────────────────────────────────────────
function extractAndNameColors(allElements) {
  const usage = new Map();
  const walk = (els) => {
    for (const el of els) {
      if (el.fill) { const k = `${el.fill.r},${el.fill.g},${el.fill.b}`; const e = usage.get(k); e ? e.count++ : usage.set(k, { color: el.fill, count: 1 }); }
      if (el.stroke) { const k = `${el.stroke.r},${el.stroke.g},${el.stroke.b}`; const e = usage.get(k); e ? e.count++ : usage.set(k, { color: el.stroke, count: 1 }); }
      if (el.gradient?.stops) for (const s of el.gradient.stops) { const k = `${s.color.r},${s.color.g},${s.color.b}`; const e = usage.get(k); e ? e.count++ : usage.set(k, { color: s.color, count: 1 }); }
      if (el.children) walk(el.children);
    }
  };
  walk(allElements);
  const sorted = [...usage.entries()].sort((a, b) => b[1].count - a[1].count);
  const names = ["SURFACE", "ON_SURFACE", "PRIMARY", "ON_PRIMARY", "SECONDARY", "ON_SECONDARY", "SURFACE_VARIANT", "OUTLINE", "BACKGROUND", "FOREGROUND", "ERROR", "ON_ERROR"];
  const colorMap = new Map(), constants = [];
  let i = 0;
  for (const [key, { color, count }] of sorted) { const name = i < names.length ? names[i] : `COLOR_${i - names.length + 1}`; colorMap.set(key, name); constants.push({ name, r: color.r, g: color.g, b: color.b, count }); i++; }
  return { colorMap, constants };
}

// ─── Component Fingerprinting ───────────────────────────────────────────────
function fingerprintElement(el) {
  const p = []; p.push(el.type || "unknown");
  if (el.fill) p.push(`f:${el.fill.r},${el.fill.g},${el.fill.b}`);
  if (el.stroke) p.push(`s:${el.stroke.r},${el.stroke.g},${el.stroke.b}:${el.stroke.width}`);
  if (el.cornerRadius) p.push(`r:${el.cornerRadius}`);
  if (el.opacity !== undefined && el.opacity !== 1.0) p.push(`o:${el.opacity.toFixed(2)}`);
  if (el.gradient) p.push(`g:${el.gradient.type}`);
  if (el.effects?.length > 0) for (const e of el.effects) p.push(`e:${e.type}`);
  if (el.textStyle) p.push(`t:${el.textStyle.size}:${el.textStyle.weight}`);
  return p.join("|");
}

function findReusableComponents(allElements) {
  const groups = new Map();
  const walk = (els) => {
    for (const el of els) {
      const fp = fingerprintElement(el);
      if (fp && !fp.includes("unknown")) {
        const e = groups.get(fp); if (e) e.elements.push(el); else groups.set(fp, { fingerprint: fp, elements: [el], suggestedName: null });
      }
      if (el.children) walk(el.children);
    }
  };
  walk(allElements);
  const comps = [];
  for (const [fp, g] of groups) {
    if (g.elements.length >= 2) {
      const f = g.elements[0];
      g.suggestedName = f.type === "text" ? "text_label" : f.type === "shape" && f.cornerRadius > 0 ? "rounded_rect_button" : "rect_shape";
      comps.push(g);
    }
  }
  return comps;
}

// ─── Code Generators ─────────────────────────────────────────────────────────
function generateTokensFile(consts) {
  let c = `// Auto-generated by egui_expressive Illustrator Exporter\nuse egui::Color32;\n\n`;
  for (const k of consts) c += `pub const ${k.name}: Color32 = Color32::from_rgb(${k.r}, ${k.g}, ${k.b});\n`;
  c += `\npub const SPACING_XS: f32 = 4.0;\npub const SPACING_SM: f32 = 8.0;\npub const SPACING_MD: f32 = 16.0;\npub const SPACING_LG: f32 = 24.0;\npub const SPACING_XL: f32 = 32.0;\npub const TEXT_SIZE_BODY: f32 = 14.0;\npub const TEXT_SIZE_SMALL: f32 = 12.0;\npub const TEXT_SIZE_HEADING: f32 = 24.0;\npub const TEXT_SIZE_TITLE: f32 = 32.0;\n`;
  return c;
}

function generateStateFile(results) {
  let c = `// Auto-generated state structs.\n\n`;
  for (const r of results) {
    const sn = toStructName(r.artboard.name);
    c += `#[derive(Default)]\npub struct ${sn}State {\n`;
    const tf = []; const walk = (els) => { for (const el of els) { if (el.type === "text" && el.textStyle?.size >= 14) tf.push(el); if (el.children) walk(el.children); } }; walk(r.elements);
    for (const t of tf) c += `    pub ${sanitize(t.text || t.id)}: String,\n`;
    c += `}\n\n#[derive(Debug, Clone, PartialEq)]\npub enum ${sn}Action {\n`;
    const btns = []; const walk2 = (els) => { for (const el of els) { if (el.type === "text" && el.text && el.text.length < 30) btns.push(el); if (el.children) walk2(el.children); } }; walk2(r.elements);
    for (const b of btns) c += `    ${toActionName(b.text || b.id)},\n`;
    c += `}\n\n`;
  }
  return c;
}

function generateModFile(results) {
  let c = `pub mod tokens;\npub mod state;\npub mod components;\n`;
  for (const r of results) c += `pub mod ${toSnakeName(r.artboard.name)};\n`;
  return c;
}

function generateComponentsFile(comps, colorMap) {
  let c = `// Auto-generated reusable components.\n\nuse egui::{Color32, RichText, Ui};\nuse super::tokens;\n\n`;
  for (const comp of comps) {
    const f = comp.elements[0], fn = comp.suggestedName.replace(/-/g, "_");
    if (f.type === "shape" && f.cornerRadius > 0) {
      const col = f.fill ? `tokens::${getColorName(f.fill, colorMap)}` : "tokens::PRIMARY";
      c += `pub fn ${fn}(ui: &mut Ui, label: &str) -> egui::Response {\n    let btn = egui::Button::new(RichText::new(label).size(${f.textStyle?.size || 14}.0).color(tokens::ON_PRIMARY)).fill(${col}).corner_radius(${Math.min(255, Math.round(f.cornerRadius || 0))}u8);\n    ui.add(btn)\n}\n\n`;
    } else if (f.type === "text") {
      const col = f.fill ? `tokens::${getColorName(f.fill, colorMap)}` : "tokens::ON_SURFACE";
      c += `pub fn ${fn}(ui: &mut Ui, text: &str) {\n    ui.label(RichText::new(text).size(${f.textStyle?.size || 14}.0).color(${col}));\n}\n\n`;
    } else {
      const col = f.fill ? `tokens::${getColorName(f.fill, colorMap)}` : "tokens::SURFACE";
      c += `pub fn ${fn}(ui: &mut Ui, rect: egui::Rect) {\n    ui.painter().rect_filled(rect, ${Math.min(255, Math.round(f.cornerRadius || 0))}u8, ${col});\n}\n\n`;
    }
  }
  return c;
}

function generateArtboardFile(ab, els, colorMap, stateName, comps) {
  const sn = toSnakeName(ab.name);
  let c = `// Auto-generated by egui_expressive Illustrator Exporter\n// Artboard: "${ab.name}" (${Math.round(ab.width)} × ${Math.round(ab.height)} px)\n\nuse egui::{Color32, RichText, Ui, Vec2, Rect};\nuse egui_expressive::{vstack, hstack, TypeSpec, soft_shadow, with_alpha};\nuse super::tokens;\nuse super::state::${stateName}State;\nuse super::components;\n\n#[allow(unused_variables)]\npub fn draw_${sn}(ui: &mut Ui, state: &mut ${stateName}State) -> Option<super::state::${stateName}Action> {\n    let origin = ui.cursor().min;\n    let painter = ui.painter();\n\n    // Background\n    painter.rect_filled(egui::Rect::from_min_size(origin, egui::vec2(${ab.width}.0, ${ab.height}.0)), 0u8, tokens::SURFACE);\n\n`;
  for (const el of els) c += generateElementCode(el, 1, colorMap, comps);
  c += `\n    None\n}\n`;
  return c;
}

function generateElementComment(el) {
  let comment = `// ${el.type}: ${el.id}`;
  if (el.thirdPartyEffects && el.thirdPartyEffects.length > 0) {
    el.thirdPartyEffects.forEach(effect => {
      comment += `\n// ⚠️  ${effect.note}`;
    });
  }
  return comment;
}

function generateElementCode(el, indent, colorMap, comps) {
  const pad = "    ".repeat(indent);
  let c = "";

  if (el.type === "unknown" || el.type === "mesh" || el.type === "chart") return `${pad}// Skipped: ${el.id} (${el.type})\n`;
  c += generateElementComment(el) + "\n";
  for (const n of el.notes || []) c += `${pad}// ${n}\n`;

  const hasShadow = el.effects?.some(e => e.type === "dropShadow");
  const hasBlur = el.effects?.some(e => e.type === "gaussianBlur");
  const hasFeather = el.effects?.some(e => e.type === "feather");
  // Shadow is now emitted inline in the shape/path branch
  if (hasBlur) { const bl = el.effects.find(e => e.type === "gaussianBlur"); c += `${pad}// Gaussian blur (${bl?.radius || 0}px)\n`; }
  if (hasFeather) { const ft = el.effects.find(e => e.type === "feather"); c += `${pad}// Feather (${ft?.radius || 0}px)\n`; }
  if (el.blendMode && el.blendMode !== "normal") c += `${pad}// blend_mode: ${el.blendMode}\n`;
  if (el.opacity !== undefined && el.opacity < 1.0) c += `${pad}// opacity: ${el.opacity}\n`;
  if (el.symbolName) return `${pad}// Symbol: ${el.symbolName}\n`;

  if (el.type === "text" && el.text) {
    const cn = el.fill ? (colorMap.get(`${el.fill.r},${el.fill.g},${el.fill.b}`) || "ON_SURFACE") : "ON_SURFACE";
    const sz = el.textStyle?.size || 14, wt = el.textStyle?.weight || 400;
    const td = el.textDecoration === "underline" ? ".underline()" : el.textDecoration === "strikethrough" ? ".strikethrough()" : "";
    const txt = el.text.replace(/"/g, '\\"').replace(/\n/g, "\\n");
    return `${pad}ui.label(egui::RichText::new("${txt}").size(${sz}.0).color(tokens::${cn})${wt !== 400 ? `.strong()` : ""}${td});\n`;
  }

  if (el.type === "shape" || el.type === "path") {
    const cn = el.fill ? (colorMap.get(`${el.fill.r},${el.fill.g},${el.fill.b}`) || "SURFACE") : "SURFACE";
    const fc = el.opacity < 1.0 ? `with_alpha(tokens::${cn}, ${el.opacity})` : `tokens::${cn}`;
    const cr = Math.min(255, Math.round(el.cornerRadius || 0)), rot = el.rotation || 0;

    c += `${pad}let rect = egui::Rect::from_min_size(origin + egui::vec2(${Math.round(el.x)}.0, ${Math.round(el.y)}.0), egui::vec2(${Math.round(el.w)}.0, ${Math.round(el.h)}.0));\n`;
    if (el.gradient) {
      const g = el.gradient;
      if (g.type === "linear") {
        const stopsStr = (g.stops || []).map(s => `(${s.position.toFixed(3)}, egui::Color32::from_rgb(${s.color.r}, ${s.color.g}, ${s.color.b}))`).join(", ");
        c += `${pad}painter.add(egui_expressive::linear_gradient_rect(rect, &[${stopsStr}], egui_expressive::GradientDir::Angle(${(g.angle || 0).toFixed(1)})));\n`;
      } else if (g.type === "radial") {
        const stops = g.stops || [];
        const innerStop = stops[0] || { color: { r: 255, g: 255, b: 255 } };
        const outerStop = stops[stops.length - 1] || { color: { r: 0, g: 0, b: 0 } };
        const innerColor = `egui::Color32::from_rgb(${innerStop.color.r}, ${innerStop.color.g}, ${innerStop.color.b})`;
        const outerColor = `egui::Color32::from_rgb(${outerStop.color.r}, ${outerStop.color.g}, ${outerStop.color.b})`;
        c += `${pad}painter.add(egui_expressive::radial_gradient_rect(rect, ${innerColor}, ${outerColor}, 32));\n`;
      } else {
        // pattern or unknown gradient type
        c += `${pad}// gradient type "${g.type}" — approximate with solid fill\n`;
        if (el.fill) c += `${pad}painter.rect_filled(rect, ${cr}u8, ${fc});\n`;
      }
    } else if (el.fill) {
      if (rot !== 0) {
        c += `${pad}let _rot = egui_expressive::Transform2D::rotate_around(${rot.toFixed(4)}, rect.center());\n`;
        c += `${pad}let _rot_pts = vec![_rot.apply(rect.min), _rot.apply(egui::pos2(rect.max.x, rect.min.y)), _rot.apply(rect.max), _rot.apply(egui::pos2(rect.min.x, rect.max.y))];\n`;
        c += `${pad}// Use _rot_pts with egui::Shape::convex_polygon for true rotation\n`;
      }
      c += `${pad}painter.rect_filled(rect, ${cr}u8, ${fc});\n`;
    }
    if (el.stroke) {
      const scn = colorMap.get(`${el.stroke.r},${el.stroke.g},${el.stroke.b}`) || "SURFACE";
      c += `${pad}painter.rect_stroke(rect, ${cr}u8, egui::Stroke::new(${el.stroke.width}.0, tokens::${scn}), egui::StrokeKind::Outside);\n`;
    }
    // Drop shadow
    const shadowFx = el.effects?.find(e => e.type === "dropShadow");
    if (shadowFx) {
      c += `${pad}for s in egui_expressive::box_shadow(rect, egui::Color32::from_rgba_unmultiplied(${shadowFx.color?.r||0}, ${shadowFx.color?.g||0}, ${shadowFx.color?.b||0}, ${Math.round((shadowFx.color?.a||0.5)*255)}), ${(shadowFx.blur||0).toFixed(1)}, 0.0, egui_expressive::ShadowOffset::new(${(shadowFx.x||0).toFixed(1)}, ${(shadowFx.y||0).toFixed(1)})) { painter.add(s); }\n`;
    }
    return c;
  }

  if (el.type === "image") {
    const imgPath = el.imagePath ? el.imagePath : `assets/${el.id}.png`;
    const isLinked = el.imagePath && !el.imagePath.startsWith("raster_");
    c += `${pad}{\n`;
    c += `${pad}    let rect = egui::Rect::from_min_size(origin + egui::vec2(${Math.round(el.x)}.0, ${Math.round(el.y)}.0), egui::vec2(${Math.round(el.w)}.0, ${Math.round(el.h)}.0));\n`;
    if (isLinked) {
      c += `${pad}    // Linked image: "${imgPath}"\n`;
      c += `${pad}    // ui.painter().image(texture_id, rect, egui::Rect::from_min_max(egui::pos2(0.0,0.0), egui::pos2(1.0,1.0)), egui::Color32::WHITE);\n`;
      c += `${pad}    painter.rect_filled(rect, 0u8, egui::Color32::from_rgba_premultiplied(80, 80, 80, 180)); // placeholder\n`;
    } else {
      c += `${pad}    // Embedded raster image (${Math.round(el.w)}×${Math.round(el.h)}px)\n`;
      c += `${pad}    painter.rect_filled(rect, 0u8, egui::Color32::from_rgba_premultiplied(80, 80, 80, 180)); // placeholder\n`;
    }
    c += `${pad}    painter.rect_stroke(rect, 0u8, egui::Stroke::new(1.0, egui::Color32::from_gray(120)), egui::StrokeKind::Outside);\n`;
    c += `${pad}}\n`;
    return c;
  }

  if (el.type === "group" && el.children?.length > 0) {
    const isRow = isHorizontal(el.children), gap = 8, lf = isRow ? "hstack" : "vstack", ax = isRow ? "x" : "y";
    c += `${pad}// Group: ${el.id}\n${pad}${lf}!(ui, gap: ${gap}, {\n`;
    for (const ch of el.children) c += generateElementCode(ch, indent + 1, colorMap, comps);
    c += `${pad}});\n`;
    return c;
  }

  return `${pad}// ${el.id} (${el.type})\n`;
}

function isHorizontal(children) {
  if (children.length < 2) return true;
  let xs = 0, ys = 0;
  const s = [...children].sort((a, b) => a.x - b.x);
  for (let i = 1; i < s.length; i++) { xs += Math.abs(s[i].x - s[i-1].x); ys += Math.abs(s[i].y - s[i-1].y); }
  return xs > ys;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────
function toSnakeName(n) {
  const RUST_KEYWORDS = new Set(["as","break","const","continue","crate","else","enum","extern","false","fn","for","if","impl","in","let","loop","match","mod","move","mut","pub","ref","return","self","static","struct","super","trait","true","type","unsafe","use","where","while","async","await","dyn"]);
  let s = (n || "").toLowerCase().replace(/[^a-z0-9]+/g, "_").replace(/^_+|_+$/g, "") || "field";
  if (/^[0-9]/.test(s)) s = "f_" + s;
  if (RUST_KEYWORDS.has(s)) s = s + "_";
  return s;
}
function toStructName(n) {
  let s = toSnakeName(n).split("_").filter(Boolean).map(s => s.charAt(0).toUpperCase() + s.slice(1)).join("") || "Component";
  if (/^[0-9]/.test(s)) s = "S" + s;
  return s;
}
function toPascalCase(s) { return s.split(/[_\- ]+/).map(s => s.charAt(0).toUpperCase() + s.slice(1)).join(""); }
function toActionName(t) { return (t || "Action").trim().replace(/[^a-zA-Z0-9]+/g, "_").split("_").map(s => s.charAt(0).toUpperCase() + s.slice(1)).join(""); }
function sanitize(n) {
  const RUST_KEYWORDS = new Set(["as","break","const","continue","crate","else","enum","extern","false","fn","for","if","impl","in","let","loop","match","mod","move","mut","pub","ref","return","self","static","struct","super","trait","true","type","unsafe","use","where","while","async","await","dyn"]);
  let s = (n || "field").toLowerCase().replace(/[^a-z0-9_]+/g, "_").replace(/^_+|_+$/g, "").slice(0, 32) || "field";
  if (/^[0-9]/.test(s)) s = "f_" + s;
  if (RUST_KEYWORDS.has(s)) s = s + "_";
  return s;
}
function getColorName(fill, colorMap) { return fill ? (colorMap.get(`${fill.r},${fill.g},${fill.b}`) || "SURFACE") : "SURFACE"; }

// ─── JSON Sidecar ────────────────────────────────────────────────────────────
function generateSidecar(ab, els, colorMap) {
  return JSON.stringify({
    artboard: { name: ab.name, width: ab.width, height: ab.height },
    colors: [...colorMap.entries()].map(([k, n]) => { const [r, g, b] = k.split(",").map(Number); return { name: n, r, g, b }; }),
    elements: els.map(el => ({
      id: el.id, type: el.type, x: Math.round(el.x), y: Math.round(el.y), w: Math.round(el.w), h: Math.round(el.h), depth: el.depth,
      fill: el.fill, stroke: el.stroke, text: el.text, textStyle: el.textStyle,
      opacity: el.opacity !== 1 ? el.opacity : undefined, rotation: el.rotation !== 0 ? el.rotation : undefined,
      cornerRadius: el.cornerRadius > 0 ? el.cornerRadius : undefined, gradient: el.gradient,
      blendMode: el.blendMode !== "normal" ? el.blendMode : undefined, strokeCap: el.strokeCap, strokeJoin: el.strokeJoin,
      strokeDash: el.strokeDash, strokeMiterLimit: el.strokeMiterLimit, effects: el.effects.length > 0 ? el.effects : undefined,
      textAlign: el.textAlign, letterSpacing: el.letterSpacing, lineHeight: el.lineHeight,
      textDecoration: el.textDecoration, textTransform: el.textTransform, textRuns: el.textRuns,
      clipMask: el.clipMask || undefined, symbolName: el.symbolName, isCompoundPath: el.isCompoundPath || undefined,
      isGradientMesh: el.isGradientMesh || undefined, isChart: el.isChart || undefined,
      thirdPartyEffects: el.thirdPartyEffects && el.thirdPartyEffects.length > 0 ? el.thirdPartyEffects : undefined,
      isOpaque: el.isOpaque || undefined, notes: el.notes.length > 0 ? el.notes : undefined,
      pathPoints: el.pathPoints || undefined, pathClosed: el.pathClosed || undefined,
      imagePath: el.imagePath || undefined,
    })),
  }, null, 2);
}

// ─── ai-parser Integration ────────────────────────────────────────────────────
let aiParserAvailable = false;

async function runAiParser(filePath) {
    try {
        // Find binary relative to plugin location
        const pluginDir = __dirname || '.';
        const binaryName = process.platform === 'win32' ? 'ai-parser.exe' : 'ai-parser';
        const binaryPath = path.join(pluginDir, '..', 'target', 'release', binaryName);

        // Try execSync (UXP may have limited Node.js support)
        let output;
        try {
            const { execSync } = require('child_process');
            output = execSync(`"${binaryPath}" "${filePath}" --pretty`, {
                encoding: 'utf8',
                timeout: 10000
            });
        } catch (e) {
            // Fallback: try using shell if execSync fails
            try {
                const { shell } = require('uxp');
                // Try to execute via shell.openExternal would require URL scheme
                // Instead, just log and return null
                console.warn('ai-parser not available via execSync:', e.message);
                aiParserAvailable = false;
                return null;
            } catch (e2) {
                console.warn('ai-parser not available:', e.message);
                aiParserAvailable = false;
                return null;
            }
        }
        
        aiParserAvailable = true;
        return JSON.parse(output);
    } catch (e) {
        console.warn('ai-parser execution failed:', e.message);
        aiParserAvailable = false;
        return null;
    }
}

function mergeAiParserData(domElements, aiParserResult) {
    if (!aiParserResult || !aiParserResult.elements) return domElements;

    const aiMap = {};
    for (const el of aiParserResult.elements) {
        aiMap[el.id] = el;
    }

    return domElements.map(el => {
        const aiEl = aiMap[el.id];
        if (!aiEl) return el;

        return {
            ...el,
            live_effects: aiEl.live_effects?.length ? aiEl.live_effects : el.effects,
            appearance_fills: aiEl.appearance_fills?.length ? aiEl.appearance_fills : undefined,
            appearance_strokes: aiEl.appearance_strokes?.length ? aiEl.appearance_strokes : undefined,
            mesh_patches: aiEl.mesh_patches?.length ? aiEl.mesh_patches : undefined,
            envelope_mesh: aiEl.envelope_mesh || undefined,
            three_d: aiEl.three_d || undefined,
        };
    });
}

async function extractFromProjectFile(artboardsData) {
    try {
        const doc = app.activeDocument;
        if (!doc) return artboardsData;

        const docPath = doc.fullName?.fsName || (doc.path && doc.name ? doc.path + '/' + doc.name : null);
        if (!docPath) {
            console.warn('Could not get document path for ai-parser');
            return artboardsData;
        }

        const aiParserResult = await runAiParser(docPath);
        if (!aiParserResult) return artboardsData;

        // Merge ai-parser data into each artboard's elements
        for (const artboard of artboardsData) {
            artboard.elements = mergeAiParserData(artboard.elements, aiParserResult);
        }

        return artboardsData;
    } catch (e) {
        console.warn('extractFromProjectFile failed:', e.message);
        return artboardsData;
    }
}

// Expose availability check for UI
function isAiParserAvailable() {
    return aiParserAvailable;
}

// ─── Main Export ─────────────────────────────────────────────────────────────
async function exportArtboards(selectedIndices, options) {
  const doc = app.activeDocument;
  if (!doc) throw new Error("No active document");

  const allEls = [], results = [];

  for (const idx of selectedIndices) {
    const ab = doc.artboards[idx], rect = ab.artboardRect;
    const abInfo = { name: ab.name, width: Math.abs(rect[2] - rect[0]), height: Math.abs(rect[3] - rect[1]), x: rect[0], y: rect[1] };
    const items = [];
    try { for (let i = 0; i < doc.pageItems.length; i++) { const it = doc.pageItems[i]; try { if (it.locked || !it.visible) continue; const b = it.geometricBounds; if (b[0] >= rect[0]-1 && b[2] <= rect[2]+1 && b[3] >= rect[3]-1 && b[1] <= rect[1]+1) items.push(it); } catch(e) {} } } catch(e) {}
    const els = extractElements(items, rect);
    allEls.push(...els);
    results.push({ artboard: abInfo, elements: els });
  }

  // Try to enrich with project file data from ai-parser
  try {
    await extractFromProjectFile(results);
  } catch (e) {
    // Gracefully degrade if ai-parser integration fails
    console.warn('ai-parser enrichment skipped:', e.message);
  }

  // Re-collect all elements after potential enrichment
  allEls.length = 0;
  for (const r of results) allEls.push(...r.elements);

  const { colorMap, constants } = extractAndNameColors(allEls);
  const comps = findReusableComponents(allEls);
  const files = {};

  files["mod.rs"] = generateModFile(results);
  files["tokens.rs"] = generateTokensFile(constants);
  files["state.rs"] = generateStateFile(results);
  files["components.rs"] = generateComponentsFile(comps, colorMap);

  for (const r of results) {
    const sn = toSnakeName(r.artboard.name), st = toStructName(r.artboard.name);
    files[`${sn}.rs`] = generateArtboardFile(r.artboard, r.elements, colorMap, st, comps);
    if (options?.includeSidecar) files[`${sn}.json`] = generateSidecar(r.artboard, r.elements, colorMap);
  }

  let zipBlob = null;
  if (typeof JSZip !== "undefined") { const zip = new JSZip(); for (const [fn, ct] of Object.entries(files)) zip.file(fn, ct); zipBlob = await zip.generateAsync({ type: "blob" }); }
  return { files, zipBlob, colorMap: Object.fromEntries(colorMap) };
}

// ─── Message Handler ─────────────────────────────────────────────────────────
window.addEventListener("message", async (event) => {
  const { type, payload } = event.data;
  if (type === "GET_ARTBOARDS") { try { window.postMessage({ type: "ARTBOARDS_RESULT", boards: await getArtboards() }); } catch (e) { window.postMessage({ type: "ERROR", message: e.message }); } }
  if (type === "CHECK_AI_PARSER") { window.postMessage({ type: "AI_PARSER_STATUS", available: aiParserAvailable }); }
  if (type === "EXPORT") { try { const { selectedIndices, options } = payload || {}; const r = await exportArtboards(selectedIndices || [], options || {}); window.postMessage({ type: "EXPORT_RESULT", payload: { files: r.files, colorMap: r.colorMap, zipBlob: r.zipBlob } }); } catch (e) { window.postMessage({ type: "ERROR", message: e.message }); } }
  if (type === "EXPORT_SINGLE") { try { const { artboardIndex, options } = payload || {}; const r = await exportArtboards([artboardIndex], options || {}); window.postMessage({ type: "EXPORT_RESULT", payload: { files: r.files, colorMap: r.colorMap, zipBlob: r.zipBlob } }); } catch (e) { window.postMessage({ type: "ERROR", message: e.message }); } }
  if (type === "EXPAND_AND_EXTRACT") {
    // Duplicate the artboard, expand all appearances, extract, then delete duplicate
    // This gives more accurate data at the cost of being destructive to the duplicate
    // Returns the same format as EXPORT but with expanded elements
    try {
      const { artboardIndex, options } = payload || {};
      const doc = app.activeDocument;
      if (!doc) throw new Error("No active document");

      // Duplicate artboard for expansion
      const ab = doc.artboards[artboardIndex];
      const duplicateName = `__expand_temp_${Date.now()}`;
      const duplicate = doc.artboards.add(duplicateName);
      const rect = ab.artboardRect;

      // Copy items to duplicate artboard
      const items = [];
      try { for (let i = 0; i < doc.pageItems.length; i++) { const it = doc.pageItems[i]; try { if (it.locked || !it.visible) continue; const b = it.geometricBounds; if (b[0] >= rect[0]-1 && b[2] <= rect[2]+1 && b[3] >= rect[3]-1 && b[1] <= rect[1]+1) items.push(it); } catch(e) {} } } catch(e) {}

      // Expand appearance on items
      for (const item of items) {
        try {
          if (item.typename === 'PathItem' || item.typename === 'GroupItem') {
            // Attempt to expand the appearance
            const expanded = item.expand();
            if (expanded) {
              // Continue processing the expanded item
            }
          }
        } catch(e) { /* expansion may not be supported for all items */ }
      }

      // Export from duplicate
      const r = await exportArtboards([artboardIndex], options || {});

      // Delete the duplicate artboard
      try { doc.artboards.remove(duplicate); } catch(e) {}

      window.postMessage({ type: "EXPORT_RESULT", payload: { files: r.files, colorMap: r.colorMap, zipBlob: r.zipBlob } });
    } catch (e) { window.postMessage({ type: "ERROR", message: e.message }); }
  }
});

window.postMessage({ type: "READY" });
