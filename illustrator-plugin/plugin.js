// egui_expressive Illustrator Exporter — CEP Extension for Adobe Illustrator 2022+
"use strict";

var EGUI_EXPORT_CHANNEL = "egui_expressive_exporter";

let extractionDiagnostics = [];
let aiParserStatus = {
  checked: false,
  available: false,
  binaryPath: null,
  diagnostics: []
};

function diagnosticMessage(error) {
  if (!error) return "unknown error";
  return error.message || String(error);
}

function noteExtractionDiagnostic(context, error) {
  if (extractionDiagnostics.length >= 200) return;
  extractionDiagnostics.push({
    id: "exporter",
    note: `${context}: ${diagnosticMessage(error)}`
  });
}

function consumeExtractionDiagnostics() {
  const out = extractionDiagnostics.slice();
  extractionDiagnostics = [];
  return out;
}

function noteAiParserDiagnostic(context, error) {
  aiParserStatus.diagnostics.push({
    id: "ai-parser",
    note: `${context}: ${diagnosticMessage(error)}`
  });
}

function getLocalTargetOrigin() {
  return "/";
}

function postPanelMessage(message) {
  if (typeof window === "undefined" || !window.postMessage) return;
  window.postMessage({ ...message, channel: EGUI_EXPORT_CHANNEL }, getLocalTargetOrigin());
}

function isTrustedPanelMessage(event) {
  return !!(
    event &&
    event.source === window &&
    event.data &&
    event.data.channel === EGUI_EXPORT_CHANNEL
  );
}

function basename(pathValue) {
  const raw = String(pathValue || "");
  const normalized = raw.replace(/\\/g, "/");
  return normalized.split("/").filter(Boolean).pop() || "";
}

function portableAssetPath(pathValue) {
  const raw = String(pathValue || "");
  if (raw.startsWith("assets/")) return raw;
  const normalized = raw.replace(/\\/g, "/");
  const name = normalized.split("/").filter(Boolean).pop() || "";
  const safeName = name.replace(/[^A-Za-z0-9._-]/g, "_");
  if (!safeName) return null;
  let hash = 0;
  for (let i = 0; i < raw.length; i++) {
    hash = ((hash << 5) - hash) + raw.charCodeAt(i);
    hash |= 0;
  }
  const hashStr = Math.abs(hash).toString(16).substring(0, 6);
  return `assets/${hashStr}_${safeName}`;
}

// ─── Artboard Discovery ───────────────────────────────────────────────────────
function getIllustratorApp() {
  try {
    // UXP
    return require('illustrator').app;
  } catch(e) {
    // Not in UXP — app not available in panel JS
    if (typeof app !== 'undefined') return app;
    return null;
  }
}

function getArtboards() {
  const app = getIllustratorApp();
  if (!app) return { error: 'Not running inside Illustrator. Install the plugin via the .zxp installer.' };
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
      note: 'Gradient mesh — emitted as mesh_gradient_patch primitives when mesh data is available' });
  }

  // PluginItem — envelope distortion, 3D effects, etc.
  if (item.typename === 'PluginItem') {
    const isTracing = item.isTracing || false;
    effects.push({
      type: isTracing ? 'liveTrace' : 'envelopeOrEffect',
      opaque: true,
      note: isTracing ? 'Live Trace — preserved as traced-vector metadata for generated Rust follow-up' : 'Envelope/3D effect — preserved as effect metadata and bounded vector primitive for generated Rust follow-up'
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
        note: 'Pattern fill — emitted as named pattern metadata with deterministic procedural fill primitive'
      });
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  // Art/Pattern brush stroke
  try {
    if (item.stroked && item.strokeColor && item.strokeColor.typename === 'PatternColor') {
      effects.push({
        type: 'brushStroke',
        opaque: true,
        brushName: item.strokeColor.pattern ? item.strokeColor.pattern.name : 'unknown',
        note: 'Art/Pattern brush stroke — emitted as named brush metadata with dashed/vector stroke primitive'
      });
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  // Detect live effects via expand+compare (expensive, only if item has complex appearance)
  try {
    if (item.typename === 'PathItem' || item.typename === 'GroupItem') {
      const hasComplexAppearance = detectComplexAppearance(item);
      if (hasComplexAppearance) {
        effects.push({
          type: 'liveEffect',
          opaque: true,
          note: 'Live effect detected (Phantasm/Astute/etc.) — preserved as live-effect metadata with generated bounded vector primitive'
        });
      }
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  return effects;
}

function detectComplexAppearance(item) {
  // Heuristic: items with live effects often have unusual bounds or typename changes after expand
  try {
    // Check if item has non-default graphic style
    if (item.graphicStyle && item.graphicStyle.name !== 'Default Graphic Style') {
      return true;
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return false;
}

// ─── Element Extraction ──────────────────────────────────────────────────────
function extractElements(pageItems, artboardRect) {
  const elements = [];
  for (const item of pageItems) extractRecursive(item, artboardRect, elements, 0);
  return elements;
}

function extractRecursive(item, artboardRect, elements, depth) {
  try { if (item.locked || item.hidden) return; } catch (e) { return; }

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
    pathPoints: null, pathClosed: false, imagePath: null, embeddedRaster: false
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
            left_ctrl: [pp.leftDirection[0] - artboardRect[0], artboardRect[1] - pp.leftDirection[1]],
            right_ctrl: [pp.rightDirection[0] - artboardRect[0], artboardRect[1] - pp.rightDirection[1]],
            kind: pp.pointType === PointType.SMOOTH ? "smooth" : "corner"
          });
        } catch (ppe) { noteExtractionDiagnostic("optional Illustrator property unavailable", ppe); }
      }
      if (pts.length > 0) {
        el.pathPoints = pts;
        el.pathClosed = item.closed || false;
      }
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  // Image/placed file extraction
  try {
    if (item.typename === "PlacedItem" && item.file) {
      el.imagePath = item.file.fsName || item.file.name || null;
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try {
    if (item.typename === "RasterItem") {
      el.embeddedRaster = true;
      el.notes.push("embedded raster image");
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  try { el.opacity = item.opacity !== undefined ? item.opacity / 100 : 1; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { el.rotation = item.rotation !== undefined ? item.rotation : 0; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.typename === "PathItem" && item.cornerRadius !== undefined) el.cornerRadius = item.cornerRadius; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  // Stroke details
  try { if (item.strokeCap !== undefined) el.strokeCap = { 0: "butt", 1: "round", 2: "square" }[item.strokeCap] || "butt"; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.strokeJoin !== undefined) el.strokeJoin = { 0: "miter", 1: "round", 2: "bevel" }[item.strokeJoin] || "miter"; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.strokeDashes?.length > 0) el.strokeDash = [...item.strokeDashes]; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.strokeMiterLimit !== undefined) el.strokeMiterLimit = item.strokeMiterLimit; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

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
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  el.gradient = getGradient(item, artboardRect);

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

  try { if (item.clipping || item.clipped) { el.clipMask = true; el.notes.push("clipping mask"); } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.typename === "CompoundPathItem") { el.isCompoundPath = true; el.notes.push("compound path"); } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }


  // SymbolItem — explicit handling with full metadata
  try {
    if (item.typename === "SymbolItem") {
      el.type = 'symbol';
      el.symbolName = item.symbol ? item.symbol.name : 'unknown';
      el.note = `Symbol instance: "${el.symbolName}" — expand to access contents`;
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.typename === "MeshItem") { el.isGradientMesh = true; el.notes.push("gradient mesh"); } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.typename === "GraphItem") { el.isChart = true; el.notes.push("chart/graph"); } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  el.effects = extractEffects(item);

  // Third-party plugin effects detection
  el.thirdPartyEffects = detectThirdPartyEffects(item);
  el.isOpaque = el.thirdPartyEffects.length > 0 && el.thirdPartyEffects.some(e => e.opaque);

  if (item.typename === "GroupItem") {
    try { if (item.pageItems) for (let i = 0; i < item.pageItems.length; i++) extractRecursive(item.pageItems[i], artboardRect, el.children, depth + 1); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  }
  elements.push(el);
}

function getElementType(item) {
  try {
    const t = item.typename;
    if (t === "TextFrame") return "text";
    if (t === "PathItem") {
      if (!item.closed) return "path";
      // Detect circle/ellipse: 4 smooth points, roughly equal width/height
      try {
        if (item.pathPoints && item.pathPoints.length === 4) {
          const allSmooth = Array.from(item.pathPoints).every(p => p.pointType === PointType.SMOOTH);
          if (allSmooth) {
            const b = item.geometricBounds;
            const w = Math.abs(b[2] - b[0]), h = Math.abs(b[1] - b[3]);
            const ratio = w > 0 && h > 0 ? Math.min(w,h)/Math.max(w,h) : 0;
            if (ratio > 0.985) return "circle";
            return "ellipse";
          }
        }
      } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
      return "shape";
    }
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
      const parseColor = (c) => { if (!c) return { r: 0, g: 0, b: 0, a: 1 }; try { if (c.typename === "RGBColor") return { r: Math.round(c.red), g: Math.round(c.green), b: Math.round(c.blue), a: 1 }; if (c.typename === "CMYKColor") { const k = c.black/100; return { r: Math.round(255*(1-c.cyan/100)*(1-k)), g: Math.round(255*(1-c.magenta/100)*(1-k)), b: Math.round(255*(1-c.yellow/100)*(1-k)), a: 1 }; } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); } return { r:0,g:0,b:0,a:1 }; };

      try { if (a.dropShadow?.enabled !== false) { const _d = a.dropShadow.distance||0, _a = (a.dropShadow.angle||0) * Math.PI / 180; fx.push({ type: "dropShadow", x: Math.round(_d * Math.cos(_a)), y: -Math.round(_d * Math.sin(_a)), blur: a.dropShadow.blur||0, spread: a.dropShadow.spread||0, color: parseColor(a.dropShadow.color), blendMode: a.dropShadow.blendMode||"normal" }); } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
      try { if (a.innerShadow?.enabled !== false) { const _d = a.innerShadow.distance||0, _a = (a.innerShadow.angle||0) * Math.PI / 180; fx.push({ type: "innerShadow", x: Math.round(_d * Math.cos(_a)), y: -Math.round(_d * Math.sin(_a)), blur: a.innerShadow.blur||0, color: parseColor(a.innerShadow.color) }); } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
      try { if (a.outerGlow?.enabled !== false) fx.push({ type: "outerGlow", blur: a.outerGlow.blur||0, color: parseColor(a.outerGlow.color) }); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
      try { if (a.innerGlow?.enabled !== false) fx.push({ type: "innerGlow", blur: a.innerGlow.blur||0, color: parseColor(a.innerGlow.color) }); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
      try { if (a.gaussianBlur?.enabled !== false) fx.push({ type: "gaussianBlur", radius: a.gaussianBlur.radius||0 }); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
      try { if (a.bevel?.enabled !== false) fx.push({ type: "bevel", depth: a.bevel.depth||0, angle: a.bevel.angle||0, highlight: parseColor(a.bevel.highlight), shadow: parseColor(a.bevel.shadow) }); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
      try { if (a.feather?.enabled !== false) fx.push({ type: "feather", radius: a.feather.radius||0 }); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  // Approach 2: tags
  try { if (item.tags?.length > 0) for (const tag of item.tags) { try { const n = String(tag.name||"").toLowerCase(); const v = String(tag.value||"").toLowerCase(); if (n.includes("noise")||n.includes("grain")||v.includes("noise")||v.includes("grain")) fx.push({ type: "noise", amount: 0.16, scale: 2, seed: 0 }); else if (n.includes("effect")||n.includes("shadow")||n.includes("glow")) fx.push({ type: "effect_from_tag", tagName: n }); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); } } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  // Approach 3: PluginItem
  try { if (item.typename === "PluginItem") fx.push({ type: "live_effect" }); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  return fx;
}

// ─── Color/Gradient ──────────────────────────────────────────────────────────
function clampByte(value, fallback) {
  const n = Number(value);
  const safe = Number.isFinite(n) ? n : fallback;
  return Math.max(0, Math.min(255, Math.round(safe)));
}

function colorToRGB(c) {
  if (!c) return null;
  try {
    if (c.typename === "SpotColor") {
      if (c.spot && c.spot.color) return colorToRGB(c.spot.color);
      if (c.color) return colorToRGB(c.color);
      return null;
    }
    if (c.typename === "RGBColor") return { r: clampByte(c.red, 0), g: clampByte(c.green, 0), b: clampByte(c.blue, 0), a: 255 };
    if (c.typename === "CMYKColor") {
      const cyan = Number(c.cyan);
      const magenta = Number(c.magenta);
      const yellow = Number(c.yellow);
      const black = Number(c.black);
      const k = Number.isFinite(black) ? black / 100 : 0;
      return {
        r: clampByte(255 * (1 - (Number.isFinite(cyan) ? cyan : 0) / 100) * (1 - k), 0),
        g: clampByte(255 * (1 - (Number.isFinite(magenta) ? magenta : 0) / 100) * (1 - k), 0),
        b: clampByte(255 * (1 - (Number.isFinite(yellow) ? yellow : 0) / 100) * (1 - k), 0),
        a: 255,
      };
    }
    if (c.typename === "GrayColor") {
      const gray = Number(c.gray);
      const v = clampByte(255 * (1 - (Number.isFinite(gray) ? gray : 0) / 100), 0);
      return { r: v, g: v, b: v, a: 255 };
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return null;
}

function getFill(item) {
  try { if (item.filled && item.fillColor) return colorToRGB(item.fillColor); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return null;
}

function getStroke(item) {
  try { if (item.stroked && item.strokeColor) { const c = colorToRGB(item.strokeColor); if (c) return { ...c, width: item.strokeWidth || 1 }; } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return null;
}

function illustratorPointToEgui(point, artboardRect) {
  if (!point) return null;
  let x = null;
  let y = null;
  if (Array.isArray(point) && point.length >= 2) {
    x = Number(point[0]);
    y = Number(point[1]);
  } else if (typeof point === "object") {
    x = Number(point.x !== undefined ? point.x : point[0]);
    y = Number(point.y !== undefined ? point.y : point[1]);
  }
  if (!Number.isFinite(x) || !Number.isFinite(y)) return null;
  if (!artboardRect || artboardRect.length < 2) return { x, y };
  return { x: x - Number(artboardRect[0]), y: Number(artboardRect[1]) - y };
}

function offsetIllustratorPoint(point, distance, angleDeg) {
  if (!point || !Number.isFinite(distance) || !Number.isFinite(angleDeg)) return null;
  const angle = angleDeg * Math.PI / 180;
  return { x: point.x + Math.cos(angle) * distance, y: point.y + Math.sin(angle) * distance };
}

function readGradientMatrix(matrix, artboardRect) {
  if (!matrix) return null;
  const read = (...names) => {
    for (const name of names) {
      const value = Number(matrix[name]);
      if (Number.isFinite(value)) return value;
    }
    return null;
  };
  let a, b, c, d, e, f;
  if (Array.isArray(matrix) && matrix.length >= 6) {
    [a, b, c, d, e, f] = matrix.map(Number);
  } else {
    a = read("a", "mValueA");
    b = read("b", "mValueB");
    c = read("c", "mValueC");
    d = read("d", "mValueD");
    e = read("e", "tx", "mValueTX");
    f = read("f", "ty", "mValueTY");
  }
  if (![a, b, c, d, e, f].every(Number.isFinite)) return null;
  if (!artboardRect || artboardRect.length < 2) return [a, b, c, d, e, f];
  const left = Number(artboardRect[0]);
  const top = Number(artboardRect[1]);
  return [
    a,
    -b,
    -c,
    d,
    a * left + c * top + e - left,
    top - b * left - d * top - f,
  ];
}

function getGradient(item, artboardRect) {
  try {
    if (item.fillColor?.typename === "GradientColor") {
      const grad = item.fillColor.gradient;
      if (!grad) return null;
      const angle = item.fillColor.angle || 0;
      const stops = [];
      try { for (const s of grad.gradientStops) stops.push({ position: s.rampPoint/100, color: gradientColorToRGB(s.color), opacity: s.opacity !== undefined ? s.opacity/100 : 1 }); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
      const origin = illustratorPointToEgui(item.fillColor.origin, artboardRect);
      const length = Number(item.fillColor.length);
      const hiliteLength = Number(item.fillColor.hiliteLength);
      const hiliteAngle = Number(item.fillColor.hiliteAngle);
      const focalPoint = Number.isFinite(hiliteLength) && Number.isFinite(hiliteAngle)
        ? offsetIllustratorPoint(origin, hiliteLength, -hiliteAngle)
        : origin;
      const transform = readGradientMatrix(item.fillColor.matrix, artboardRect);
      return { type: grad.type === 1 ? "linear" : "radial", angle, center: origin, focalPoint, radius: Number.isFinite(length) && length > 0 ? length : null, transform, stops };
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
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return null;
}

function gradientColorToRGB(c) {
  const rgb = colorToRGB(c);
  return rgb ? { r: rgb.r, g: rgb.g, b: rgb.b } : { r: 128, g: 128, b: 128 };
}

// ─── Text Details ────────────────────────────────────────────────────────────
function getTextStyle(item) {
  try {
    const chars = item.textRange.characterAttributes;
    let size = 14, weight = 400, family = "default";
    try { size = chars.size || 14; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
    try { if (chars.textFont) { const n = chars.textFont.name || ""; weight = n.includes("Bold") ? 700 : n.includes("Light") ? 300 : 400; family = n; } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
    return { size, fontSize: size, weight, family };
  } catch (e) { return { size: 14, fontSize: 14, weight: 400, family: "default" }; }
}

function getTextAlign(item) {
  if (item.typename !== "TextFrame") return null;
  try { const j = item.textRange.paragraphAttributes.justification; if (j === Justification.LEFT) return "left"; if (j === Justification.CENTER) return "center"; if (j === Justification.RIGHT) return "right"; if (j === Justification.FULLJUSTIFY) return "justified"; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return "left";
}

function getLetterSpacing(item) { if (item.typename !== "TextFrame") return null; try { return item.textRange.characterAttributes.tracking / 1000; } catch (e) { return null; } }
function getLineHeight(item) { if (item.typename !== "TextFrame") return null; try { const l = item.textRange.characterAttributes.leading; return l > 0 ? l : null; } catch (e) { return null; } }
function getTextDecoration(item) { if (item.typename !== "TextFrame") return null; try { const u = item.textRange.characterAttributes.underline, s = item.textRange.characterAttributes.strikeThrough; if (u && s) return "underline_strikethrough"; if (u) return "underline"; if (s) return "strikethrough"; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); } return null; }
function getTextTransform(item) { if (item.typename !== "TextFrame") return null; try { if (item.textRange.characterAttributes.allCaps) return "all_caps"; if (item.textRange.characterAttributes.smallCaps) return "small_caps"; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); } return null; }

function getTextRuns(item) {
  if (item.typename !== "TextFrame") return null;
  try {
    const runs = [], trs = item.textRanges;
    if (trs && trs.length > 1) { for (const tr of trs) { try { const a = tr.characterAttributes; runs.push({ text: tr.contents || "", style: { size: a.size||14, weight: a.textFont?.name?.includes("Bold") ? 700 : 400, color: colorToRGB(a.fillColor) } }); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); } } }
    return runs.length > 0 ? runs : null;
  } catch (e) { return null; }
}

// ─── Color Deduplication ──────────────────────────────────────────────────────
function stopColorToRgb(c) {
  if (!c) return { r: 0, g: 0, b: 0 };
  if (typeof c === 'string') {
    const hex = c.replace('#', '');
    return { r: parseInt(hex.slice(0,2),16)||0, g: parseInt(hex.slice(2,4),16)||0, b: parseInt(hex.slice(4,6),16)||0 };
  }
  return { r: c.r||0, g: c.g||0, b: c.b||0 };
}

function gradientStopsExpr(g, opacity) {
  return (g.stops || []).map(s => {
    const c = stopColorToRgb(s.color);
    const a = Math.round((s.opacity !== undefined ? s.opacity : 1) * opacity * 255);
    return `(${Number(s.position || 0).toFixed(3)}, egui::Color32::from_rgba_unmultiplied(${c.r}, ${c.g}, ${c.b}, ${a}))`;
  }).join(", ");
}

function gradientRectCode(g, rectExpr, pad, opacity) {
  const stops = gradientStopsExpr(g, opacity);
  if (g.type === "radial") return `${pad}painter.add(egui_expressive::radial_gradient_rect_stops(${rectExpr}, &[${stops}], 48));\n`;
  return `${pad}painter.add(egui_expressive::linear_gradient_rect(${rectExpr}, &[${stops}], egui_expressive::GradientDir::Angle(${fmtF32(g.angle || 0)})));\n`;
}

function gradientPointExpr(point) {
  if (!point || !Number.isFinite(Number(point.x)) || !Number.isFinite(Number(point.y))) return "None";
  return `Some(origin + egui::vec2(${fmtF32(point.x)}, ${fmtF32(point.y)}))`;
}

function gradientRadiusExpr(radius) {
  return Number.isFinite(Number(radius)) && Number(radius) > 0 ? `Some(${fmtF32(radius)})` : "None";
}

function gradientTransformExpr(g) {
  const m = g.transform || g.matrix;
  if (!Array.isArray(m) || m.length < 6 || !m.every(v => Number.isFinite(Number(v)))) return "None";
  const a = Number(m[0]), b = Number(m[1]), c = Number(m[2]), d = Number(m[3]), e = Number(m[4]), f = Number(m[5]);
  return `Some(egui_expressive::Transform2D { a: ${fmtF32(a)}, b: ${fmtF32(b)}, c: ${fmtF32(c)}, d: ${fmtF32(d)}, e: origin.x + ${fmtF32(e)} - ${fmtF32(a)} * origin.x - ${fmtF32(c)} * origin.y, f: origin.y + ${fmtF32(f)} - ${fmtF32(b)} * origin.x - ${fmtF32(d)} * origin.y })`;
}

function rectGradientPointsExpr(cornerRadius, rotatedExpr) {
  const cr = Number(cornerRadius || 0);
  if (cr > 0) {
    const rounded = `egui_expressive::rounded_rect_path(rect, ${fmtF32(cr)})`;
    return rotatedExpr ? `${rounded}.into_iter().map(|p| _rot.apply(p)).collect::<Vec<_>>()` : rounded;
  }
  return rotatedExpr ? rotatedExpr : "vec![rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom()]";
}

function closedRectStrokePointsExpr(cornerRadius, rotatedExpr) {
  return `{ let mut pts = ${rectGradientPointsExpr(cornerRadius, rotatedExpr)}; pts.push(pts[0]); pts }`;
}

function ellipseSampleCount(width, height, minSegments) {
  const rx = Math.abs(Number(width || 0)) / 2;
  const ry = Math.abs(Number(height || 0)) / 2;
  const perimeter = Math.PI * (3 * (rx + ry) - Math.sqrt(Math.max(0, (3 * rx + ry) * (rx + 3 * ry))));
  return Math.max(minSegments || 48, Math.min(160, Math.ceil(perimeter / 4)));
}

function gradientPathMeshCode(g, pointsExpr, pad, opacity) {
  if (g.type === "pattern" || (g.type !== "linear" && g.type !== "radial")) {
    return patternFillPathCode(g, pointsExpr, pad, opacity);
  }
  const stops = gradientStopsExpr(g, opacity);
  const radial = g.type === "radial" ? "true" : "false";
  return `${pad}if let Some(grad_shape) = egui_expressive::gradient_path_mesh_with_transform(${pointsExpr}, &[${stops}], ${fmtF32(g.angle || 0)}, ${radial}, egui_expressive::GradientPathGeometry { center: ${gradientPointExpr(g.center)}, focal_point: ${gradientPointExpr(g.focalPoint || g.focal_point)}, radius: ${gradientRadiusExpr(g.radius)}, transform: ${gradientTransformExpr(g)} }) { painter.add(grad_shape); }\n`;
}

function stableHash32(value) {
  let hash = 0x811c9dc5;
  const text = String(value || "pattern");
  for (let i = 0; i < text.length; i++) hash = Math.imul(hash ^ text.charCodeAt(i), 0x01000193) >>> 0;
  return hash >>> 0;
}

function patternSeed(g) {
  return stableHash32(`${g.patternName || g.pattern_name || g.name || g.type || "pattern"}:${g.rotation || 0}:${JSON.stringify(g.scale || [])}`);
}

function patternMetrics(g) {
  const seed = patternSeed(g);
  const scale = Array.isArray(g.scale) ? g.scale.filter(v => Number.isFinite(Number(v))).map(Number) : [];
  const avgScale = scale.length > 0 ? scale.reduce((a, v) => a + v, 0) / scale.length : 1;
  const cell = Math.max(2, Math.min(64, 8 * avgScale));
  const mark = Math.max(0.5, Math.min(16, cell * 0.12));
  return { seed, cell, mark };
}

function patternFillPathCode(g, pointsExpr, pad, opacity) {
  const { seed, cell, mark } = patternMetrics(g);
  const r = 64 + (seed & 0x7f);
  const gr = 64 + ((seed >>> 8) & 0x7f);
  const b = 64 + ((seed >>> 16) & 0x7f);
  const safeOpacity = Math.max(0, Math.min(1, Number(opacity === undefined ? 1 : opacity)));
  const fgAlpha = Math.round(safeOpacity * 220);
  const bgAlpha = Math.round(safeOpacity * 48);
  return `${pad}// Pattern fill "${sanitizeComment(g.patternName || g.pattern_name || g.name || g.type || "pattern")}" — editable procedural vector primitive\n${pad}for s in egui_expressive::pattern_fill_path(${pointsExpr}, ${seed}u32, egui::Color32::from_rgba_unmultiplied(${r}, ${gr}, ${b}, ${fgAlpha}), egui::Color32::from_rgba_unmultiplied(${255 - Math.floor(r / 2)}, ${255 - Math.floor(gr / 2)}, ${255 - Math.floor(b / 2)}, ${bgAlpha}), ${fmtF32(cell)}, ${fmtF32(mark)}) { painter.add(s); }\n`;
}

function extractAndNameColors(allElements, useNaming) {
  const usage = new Map();
  const walk = (els) => {
    for (const el of els) {
      if (el.fill) { const k = `${el.fill.r},${el.fill.g},${el.fill.b}`; const e = usage.get(k); e ? e.count++ : usage.set(k, { color: el.fill, count: 1 }); }
      if (el.stroke) { const k = `${el.stroke.r},${el.stroke.g},${el.stroke.b}`; const e = usage.get(k); e ? e.count++ : usage.set(k, { color: el.stroke, count: 1 }); }
      if (el.gradient?.stops) for (const s of el.gradient.stops) { const c = stopColorToRgb(s.color); const k = `${c.r},${c.g},${c.b}`; const e = usage.get(k); e ? e.count++ : usage.set(k, { color: c, count: 1 }); }
      if (el.children) walk(el.children);
    }
  };
  walk(allElements);
  const sorted = [...usage.entries()].sort((a, b) => b[1].count - a[1].count);
  const colorMap = new Map(), constants = [];
  let i = 0;
  for (const [key, { color, count }] of sorted) {
    let name;
    if (useNaming === false) {
      // Flat naming: just use hex-based names, no semantic heuristics
      const hex = colorToHex(color).toUpperCase().replace('#', '');
      name = `COLOR_${hex}`;
    } else {
      // Generate a descriptive name from the hex value
      const hex = colorToHex(color).toUpperCase().replace('#', '');
      // Try to match to a semantic name based on common UI colors
      const r = color.r, g = color.g, b = color.b;
      // Heuristic semantic assignment based on luminance and saturation
      const lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
      if (i === 0) name = lum > 0.5 ? "SURFACE" : "BACKGROUND";
      else if (i === 1) name = lum > 0.5 ? "ON_SURFACE" : "PRIMARY";
      else if (i === 2) name = "SECONDARY";
      else if (i === 3) name = "ACCENT";
      else name = `COLOR_${hex}`;
    }
    colorMap.set(key, name);
    constants.push({ name, r: color.r, g: color.g, b: color.b, count });
    i++;
  }
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
  const nameCount = {};
  for (const comp of comps) {
      const base = comp.suggestedName || "component";
      nameCount[base] = (nameCount[base] || 0) + 1;
      if (nameCount[base] > 1) {
          comp.suggestedName = `${base}_${nameCount[base]}`;
      }
  }
  return comps;
}

// ─── Code Generators ─────────────────────────────────────────────────────────
function generateTokensFile(consts) {
  let c = `// Auto-generated by egui_expressive Illustrator Exporter\nuse egui::Color32;\n\n`;
  for (const k of consts) c += `pub const ${k.name}: Color32 = Color32::from_rgb(${k.r}, ${k.g}, ${k.b});\n`;

  // Guarantee semantic tokens exist even if source had few colors
  const semanticFallbacks = {
    SURFACE: "COLOR_1", ON_SURFACE: "COLOR_2", PRIMARY: "COLOR_3",
    ON_PRIMARY: "COLOR_4", SECONDARY: "COLOR_5", ON_SECONDARY: "COLOR_6",
    SURFACE_VARIANT: "COLOR_7", OUTLINE: "COLOR_8"
  };
  for (const [semantic, fallback] of Object.entries(semanticFallbacks)) {
    // Only add if not already defined by a discovered color
    if (!consts.find(c => c.name === semantic)) {
      c += `pub const ${semantic}: Color32 = Color32::from_rgb(128, 128, 128); // fallback\n`;
    }
  }

  c += `\npub const SPACING_XS: f32 = 4.0;\npub const SPACING_SM: f32 = 8.0;\npub const SPACING_MD: f32 = 16.0;\npub const SPACING_LG: f32 = 24.0;\npub const SPACING_XL: f32 = 32.0;\npub const TEXT_SIZE_BODY: f32 = 14.0;\npub const TEXT_SIZE_SMALL: f32 = 12.0;\npub const TEXT_SIZE_HEADING: f32 = 24.0;\npub const TEXT_SIZE_TITLE: f32 = 32.0;\n`;
  return c;
}

function generateStateFile(results) {
  let c = `// Auto-generated state structs.\n\n`;
  for (const r of results) {
    const sn = toStructName(r.artboard.name);
    c += `#[derive(Default, Clone)]\npub struct ${sn}State {\n`;
    const tf = []; const walk = (els) => { for (const el of els) { if (el.type === "text" && el.textStyle?.size >= 14) tf.push(el); if (el.children) walk(el.children); } }; walk(r.elements);
    const usedFieldNames = new Set();
    for (const t of tf) { let name = sanitize(t.text || t.id); let suffix = 2; while (usedFieldNames.has(name)) { name = sanitize(t.text || t.id) + "_" + suffix; suffix++; } usedFieldNames.add(name); c += `    pub ${name}: String,\n`; }

    // Detect tab bars — groups with 3+ horizontal children of similar size
    const tabBars = [];
    const walkTabs = (els) => {
      for (const el of els) {
        if (el.type === "group" && el.children && el.children.length >= 3) {
          const ch = el.children;
          // Check if horizontal arrangement
          const xSpread = Math.max(...ch.map(c => c.x)) - Math.min(...ch.map(c => c.x));
          const ySpread = Math.max(...ch.map(c => c.y)) - Math.min(...ch.map(c => c.y));
          if (xSpread > ySpread && xSpread > 20) {
            // Check similar sizes
            const avgW = ch.reduce((s, c) => s + c.w, 0) / ch.length;
            const sizeConsistent = ch.every(c => Math.abs(c.w - avgW) < avgW * 0.5);
            if (sizeConsistent) {
              tabBars.push({ groupId: el.id, count: ch.length });
            }
          }
        }
        if (el.children) walkTabs(el.children);
      }
    };
    walkTabs(r.elements);

    // Add tab state fields
    for (const tb of tabBars) {
      const tabFieldName = sanitize(tb.groupId || "tab") + "_index";
      let suffix = 2;
      let finalName = tabFieldName;
      while (usedFieldNames.has(finalName)) { finalName = tabFieldName + "_" + suffix; suffix++; }
      usedFieldNames.add(finalName);
      c += `    pub ${finalName}: usize, // tab bar (${tb.count} tabs)\n`;
    }

    c += `}\n\n#[derive(Debug, Clone, PartialEq)]\npub enum ${sn}Action {\n`;
    const btns = []; const walk2 = (els) => { for (const el of els) { if (el.type === "text" && el.text && el.text.length < 30) btns.push(el); if (el.children) walk2(el.children); } }; walk2(r.elements);
    const usedVariantNames = new Set();
    for (const b of btns) { let name = toActionName(b.text || b.id); let suffix = 2; while (usedVariantNames.has(name)) { name = toActionName(b.text || b.id) + suffix; suffix++; } usedVariantNames.add(name); c += `    ${name},\n`; }

    for (const tb of tabBars) {
      const actionBase = toActionName(tb.groupId || "Tab") + "Select";
      let suffix = 2;
      let finalAction = actionBase;
      while (usedVariantNames.has(finalAction)) { finalAction = actionBase + suffix; suffix++; }
      usedVariantNames.add(finalAction);
      c += `    ${finalAction}(usize),\n`;
    }

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
  let usesRichText = false;
  for (const comp of comps) {
    const f = comp.elements[0];
    if (f.type === "shape" && f.cornerRadius > 0) usesRichText = true;
    else if (f.type === "text") usesRichText = true;
  }
  let c = `// Auto-generated reusable components.\n\n#[allow(unused_imports)]\nuse egui::{Color32, ${usesRichText ? "RichText, " : ""}Ui};\n#[allow(unused_imports)]\nuse super::tokens;\n\n`;
  for (const comp of comps) {
    const f = comp.elements[0], fn = comp.suggestedName.replace(/-/g, "_");
    if (f.type === "shape" && f.cornerRadius > 0) {
      const col = f.fill ? `tokens::${getColorName(f.fill, colorMap)}` : "tokens::PRIMARY";
      const onColor = col.includes("PRIMARY") ? "tokens::ON_PRIMARY" : col.includes("SECONDARY") ? "tokens::ON_SECONDARY" : "tokens::ON_SURFACE";
      c += `pub fn ${fn}(ui: &mut Ui, label: &str) -> egui::Response {\n    let btn = egui::Button::new(RichText::new(label).size(${fmtF32(f.textStyle?.size || 14)}).color(${onColor})).fill(${col}).corner_radius(${Math.min(255, Math.round(f.cornerRadius || 0))}u8);\n    ui.add(btn)\n}\n\n`;
    } else if (f.type === "text") {
      const col = f.fill ? `tokens::${getColorName(f.fill, colorMap)}` : "tokens::ON_SURFACE";
      c += `pub fn ${fn}(ui: &mut Ui, text: &str) {\n    ui.label(RichText::new(text).size(${fmtF32(f.textStyle?.size || 14)}).color(${col}));\n}\n\n`;
    } else {
      const col = f.fill ? `tokens::${getColorName(f.fill, colorMap)}` : "tokens::SURFACE";
      c += `pub fn ${fn}(ui: &mut Ui, rect: egui::Rect) {\n    ui.painter().rect_filled(rect, ${Math.min(255, Math.round(f.cornerRadius || 0))}u8, ${col});\n}\n\n`;
    }
  }
  return c;
}

function generateArtboardFile(ab, els, colorMap, stateName, comps, options) {
  const sn = toSnakeName(ab.name);
  let usesShadow = false, usesBlur = false, usesComponents = comps && comps.length > 0, usesClipPath = false, usesBlendMode = false;
  const walk = (elements) => {
    for (const el of elements) {
      if (el.effects?.some(e => e.type === "dropShadow" || e.type === "innerShadow" || e.type === "outerGlow" || e.type === "innerGlow")) usesShadow = true;
      if (el.effects?.some(e => e.type === "gaussianBlur" || e.type === "feather")) usesBlur = true;
      if (el.clipMask && el.pathPoints && el.pathPoints.length > 2) usesClipPath = true;
      if (el.blendMode && el.blendMode !== "normal") usesBlendMode = true;
      if (el.children) walk(el.children);
    }
  };
  walk(els);

  let imports = ["Color32", "Ui", "Vec2", "Rect", "Align2", "FontId", "FontFamily"];

  let exprImports = ["with_alpha"];
  if (usesBlur || usesShadow) { exprImports.push("soft_shadow", "BlurQuality", "ShadowOffset"); }
  if (usesClipPath) { exprImports.push("with_clip_path", "clipped_layers_gpu", "BlendLayer"); }
  if (usesBlendMode) { exprImports.push("BlendMode", "composite_layers_gpu", "BlendLayer"); }
  exprImports = [...new Set(exprImports)];

  let c = `// Auto-generated by egui_expressive Illustrator Exporter\n// Artboard: "${sanitizeComment(ab.name)}" (${Math.round(ab.width)} × ${Math.round(ab.height)} px)\n// Options: naming=${options?.naming !== false}, gaps=${options?.gaps !== false}, native=${options?.native !== false}, sidecar=${options?.sidecar !== false || options?.includeSidecar !== false}\n\n#[allow(unused_imports)]\nuse egui::{${imports.join(", ")}};\n#[allow(unused_imports)]\nuse egui_expressive::{${exprImports.join(", ")}};\n#[allow(unused_imports)]\nuse super::tokens;\nuse super::state::${stateName}State;\n`;
  if (usesComponents) c += `use super::components;\n`;
  c += `\n#[allow(unused_variables)]\npub fn draw_${sn}(ui: &mut Ui, state: &mut ${stateName}State) -> Option<super::state::${stateName}Action> {\n    let origin = ui.cursor().min;\n    ui.allocate_space(egui::vec2(${fmtF32(ab.width)}, ${fmtF32(ab.height)}));\n    let painter = ui.painter();\n\n`;
  c += `    // Background\n`;
  c += `    painter.rect_filled(egui::Rect::from_min_size(origin, egui::vec2(${fmtF32(ab.width)}, ${fmtF32(ab.height)})), 0u8, tokens::SURFACE);\n\n`;
  for (const el of els) c += generateElementCode(el, 1, colorMap, comps, options);
  c += `\n    None\n}\n`;
  return c;
}

function sanitizeComment(s) { return String(s || "").replace(/[\r\n]/g, " ").replace(/\//g, "/"); }

function generateElementComment(el) {
  let comment = `// ${sanitizeComment(el.type + ": " + el.id)}`;
  if (el.thirdPartyEffects && el.thirdPartyEffects.length > 0) {
    el.thirdPartyEffects.forEach(effect => {
      comment += `\n// ${sanitizeComment("WARNING: " + effect.note)}`;
    });
  }
  return comment;
}

function generateElementCodeInner(el, indent, colorMap, comps, options) {
  const pad = "    ".repeat(indent);
  let c = "";

  if (el.type === "unknown") return `${pad}// Skipped: ${sanitizeComment(el.id)} (${el.type})\n`;
  if (el.mesh_patches && el.mesh_patches.length > 0) {
    c += `${pad}// Gradient mesh: ${sanitizeComment(el.id)} — emitted as code-generated mesh patches\n`;
    c += `${pad}{\n`;
    el.mesh_patches.forEach((patch, i) => {
      const corners = (patch.corners || []).slice(0, 4);
      const colors = (patch.colors || []).slice(0, 4);
      if (corners.length !== 4) return;
      const cornerExpr = corners.map(p => `origin + egui::vec2(${fmtF32(Number(p[0]) || 0)}, ${fmtF32(Number(p[1]) || 0)})`).join(", ");
      const colorExpr = [0, 1, 2, 3].map(idx => {
        const col = colors[idx] || [255, 255, 255, 255];
        return `egui::Color32::from_rgba_unmultiplied(${clampByte(col[0], 255)}, ${clampByte(col[1], 255)}, ${clampByte(col[2], 255)}, ${clampByte(col[3] === undefined ? 255 : col[3], 255)})`;
      }).join(", ");
      c += `${pad}    let mesh_corners_${i} = [${cornerExpr}];\n`;
      c += `${pad}    let mesh_colors_${i} = [${colorExpr}];\n`;
      c += `${pad}    painter.add(egui_expressive::mesh_gradient_patch(mesh_corners_${i}, mesh_colors_${i}, 16));\n`;
    });
    c += `${pad}}\n`;
    return c;
  }
  if (el.type === "mesh" || el.type === "chart") {
    const cn = el.fill ? (colorMap.get(`${el.fill.r},${el.fill.g},${el.fill.b}`) || "SURFACE") : "SURFACE";
    const fc = applyBlendExpr(el.opacity < 1.0 ? `with_alpha(tokens::${cn}, ${el.opacity})` : `tokens::${cn}`, el.blendMode);
    return `${pad}// ${sanitizeComment(el.type)} primitive: ${sanitizeComment(el.id)} — CEP exposes bounds/appearance; emitted as editable bounded vector primitive\n${pad}{\n${pad}    let rect = egui::Rect::from_min_size(origin + egui::vec2(${fmtF32(el.x)}, ${fmtF32(el.y)}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)}));\n${pad}    painter.rect_filled(rect, 0u8, ${fc});\n${pad}    painter.rect_stroke(rect, 0u8, egui::Stroke::new(1.0, egui::Color32::from_gray(140)), egui::StrokeKind::Outside);\n${pad}}\n`;
  }
  c += generateElementComment(el) + "\n";
  for (const n of el.notes || []) c += `${pad}// ${sanitizeComment(n)}\n`;

  if (["circle", "ellipse", "path", "shape"].includes(el.type)) {
    const earlyAppearanceFills = el.appearance_fills || el.appearanceFills || [];
    const earlyAppearanceStrokes = el.appearance_strokes || el.appearanceStrokes || [];
    const earlyHasAppearanceStack = el.appearanceStack?.length > 0 || earlyAppearanceFills.length > 0 || earlyAppearanceStrokes.length > 0;
    if (earlyHasAppearanceStack) {
      const earlyLayers = appearanceLayers(el, earlyAppearanceFills, earlyAppearanceStrokes);
      const synthesizedAppearanceHasEffects = !(el.appearanceStack?.length > 0)
        && (earlyAppearanceFills.length > 0 || earlyAppearanceStrokes.length > 0)
        && (el.effects?.length > 0);
      if (el.type !== "shape" || el.appearanceStack?.length > 0 || appearanceHasNonNormalBlend(earlyLayers) || synthesizedAppearanceHasEffects) {
        return c + sceneBackedAppearanceCode(el, pad, earlyLayers);
      }
    }
  }

  const hasShadow = el.effects?.some(e => e.type === "dropShadow" || e.type === "innerShadow" || e.type === "outerGlow" || e.type === "innerGlow");
  const hasBlur = el.effects?.some(e => e.type === "gaussianBlur");
  const hasFeather = el.effects?.some(e => e.type === "feather");
  // Shadow is now emitted inline in the shape/path branch
  if (hasFeather) { const ft = el.effects.find(e => e.type === "feather"); c += `${pad}// Feather (${ft?.radius || 0}px)\n`; }
  if (el.blendMode && el.blendMode !== "normal") c += `${pad}// blend_mode: ${el.blendMode} (approximated against current egui background)\n`;
  if (el.opacity !== undefined && el.opacity < 1.0) c += `${pad}// opacity: ${el.opacity}\n`;
  if (el.symbolName) {
    c += `${pad}{\n`;
    c += `${pad}    // Symbol instance: "${sanitizeComment(el.symbolName)}"\n`;
    if (el.children && el.children.length > 0) {
      for (const ch of el.children) c += generateElementCode(ch, indent + 1, colorMap, comps, options);
    } else {
      c += `${pad}    let rect = egui::Rect::from_min_size(origin + egui::vec2(${fmtF32(el.x)}, ${fmtF32(el.y)}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)}));\n`;
      c += `${pad}    painter.rect_stroke(rect, 2u8, egui::Stroke::new(1.0, egui::Color32::from_gray(150)), egui::StrokeKind::Outside);\n`;
    }
    c += `${pad}}\n`;
    return c;
  }

  if (el.type === "circle") {
    const cn = el.fill ? (colorMap.get(`${el.fill.r},${el.fill.g},${el.fill.b}`) || "SURFACE") : "SURFACE";
    const fc = applyBlendExpr(el.opacity < 1.0 ? `with_alpha(tokens::${cn}, ${el.opacity})` : `tokens::${cn}`, el.blendMode);
    const cx = fmtF32(el.x + el.w / 2);
    const cy = fmtF32(el.y + el.h / 2);
    const radius = fmtF32(Math.min(el.w, el.h) / 2);
    const circleSegments = ellipseSampleCount(el.w, el.h, 48);

    // Shadow before fill
    const shadowFxList = el.effects?.filter(e => e.type === "dropShadow") || [];
    if (shadowFxList.length > 0) {
      c += `${pad}let _circle_rect = egui::Rect::from_center_size(origin + egui::vec2(${cx}, ${cy}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)}));\n`;
      for (const shadowFx of shadowFxList) {
        c += `${pad}for s in egui_expressive::box_shadow(_circle_rect, egui::Color32::from_rgba_unmultiplied(${shadowFx.color?.r||0}, ${shadowFx.color?.g||0}, ${shadowFx.color?.b||0}, ${Math.round((shadowFx.color?.a||0.5)*255)}), ${fmtF32(shadowFx.blur||0)}, 0.0, egui_expressive::ShadowOffset::new(${fmtF32(shadowFx.x||0)}, ${fmtF32(shadowFx.y||0)})) { painter.add(s); }\n`;
      }
    }
    if (hasBlur) {
      const bl = el.effects.find(e => e.type === "gaussianBlur");
      c += `${pad}for s in egui_expressive::soft_shadow(egui::Rect::from_center_size(origin + egui::vec2(${cx}, ${cy}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)})), egui::Color32::from_rgba_unmultiplied(0, 0, 0, 60), ${fmtF32(bl?.radius||4)}, 0.0, egui_expressive::ShadowOffset::zero(), egui_expressive::BlurQuality::Medium) { painter.add(s); }\n`;
    }
    const circleFeathers = el.effects?.filter(e => e.type === "feather") || [];
    for (const featherFx of circleFeathers) {
      c += `${pad}for s in egui_expressive::soft_shadow(egui::Rect::from_center_size(origin + egui::vec2(${cx}, ${cy}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)})), ${fc}, ${fmtF32(featherFx.radius||4)}, 0.0, egui_expressive::ShadowOffset::zero(), egui_expressive::BlurQuality::High) { painter.add(s); }\n`;
    }
    if (el.gradient) {
      c += `${pad}{\n`;
      c += `${pad}    let path_pts: Vec<egui::Pos2> = (0..=${circleSegments}).map(|i| { let a = i as f32 * std::f32::consts::TAU / ${fmtF32(circleSegments)}; origin + egui::vec2(${cx} + ${radius} * a.cos(), ${cy} + ${radius} * a.sin()) }).collect();\n`;
      c += gradientPathMeshCode(el.gradient, "&path_pts", pad + "    ", el.opacity !== undefined ? el.opacity : 1.0);
      c += `${pad}}\n`;
    } else if (el.fill) {
      c += `${pad}painter.circle_filled(origin + egui::vec2(${cx}, ${cy}), ${radius}, ${fc});\n`;
    }
    if (el.stroke) {
      const scn = colorMap.get(`${el.stroke.r},${el.stroke.g},${el.stroke.b}`) || "SURFACE";
      const strokeColor = strokeColorExpr(el, colorMap, scn);
      if (hasRichStrokeSemantics(el)) {
        const sampled = el.pathPoints && el.pathPoints.length > 1 ? samplePathPoints(el.pathPoints, true) : null;
        c += `${pad}{\n`;
        c += sampled
          ? `${pad}    let path_pts = ${rustPointsVec(sampled)};\n`
          : `${pad}    let path_pts: Vec<egui::Pos2> = (0..=${circleSegments}).map(|i| { let a = i as f32 * std::f32::consts::TAU / ${fmtF32(circleSegments)}; origin + egui::vec2(${cx} + ${radius} * a.cos(), ${cy} + ${radius} * a.sin()) }).collect();\n`;
        c += `${pad}    let rich_stroke = ${richStrokeExpr(el, colorMap, scn)};\n`;
        c += `${pad}    egui_expressive::dashed_path(&painter, &path_pts, &rich_stroke);\n`;
        c += `${pad}}\n`;
      } else {
        c += `${pad}painter.circle_stroke(origin + egui::vec2(${cx}, ${cy}), ${radius}, egui::Stroke::new(${fmtF32(el.stroke.width)}, ${strokeColor}));\n`;
      }
    }
    return c;
  }

  if (el.type === "ellipse") {
    // Ellipse: use convex_polygon approximation (egui has no native ellipse)
    const cn = el.fill ? (colorMap.get(`${el.fill.r},${el.fill.g},${el.fill.b}`) || "SURFACE") : "SURFACE";
    const fc = applyBlendExpr(el.opacity < 1.0 ? `with_alpha(tokens::${cn}, ${el.opacity})` : `tokens::${cn}`, el.blendMode);
    if (el.pathPoints && el.pathPoints.length > 2) {
      const sampled = samplePathPoints(el.pathPoints, true);
      c += `${pad}{\n`;
      c += `${pad}    let path_pts = ${rustPointsVec(sampled)};\n`;
      if (el.gradient) c += gradientPathMeshCode(el.gradient, "&path_pts", pad + "    ", el.opacity !== undefined ? el.opacity : 1.0);
      else if (el.fill) c += `${pad}    painter.add(egui::Shape::Path(egui::epaint::PathShape { points: path_pts.clone(), closed: true, fill: ${fc}, stroke: egui::epaint::PathStroke::NONE }));\n`;
      if (el.stroke) {
        const scn = colorMap.get(`${el.stroke.r},${el.stroke.g},${el.stroke.b}`) || "SURFACE";
        if (hasRichStrokeSemantics(el)) {
          c += `${pad}    let rich_stroke = ${richStrokeExpr(el, colorMap, scn)};\n`;
          c += `${pad}    egui_expressive::dashed_path(&painter, &path_pts, &rich_stroke);\n`;
        } else {
          c += `${pad}    painter.add(egui::Shape::closed_line(path_pts, egui::Stroke::new(${fmtF32(el.stroke.width)}, ${strokeColorExpr(el, colorMap, scn)})));\n`;
        }
      }
      c += `${pad}}\n`;
      return c;
    }
    const cx = fmtF32(el.x + el.w / 2), cy = fmtF32(el.y + el.h / 2);
    const rx = fmtF32(el.w / 2), ry = fmtF32(el.h / 2);
    const ellipseSegments = ellipseSampleCount(el.w, el.h, 48);
    c += `${pad}{\n`;
    c += `${pad}    let cx = origin.x + ${cx};\n`;
    c += `${pad}    let cy = origin.y + ${cy};\n`;
    c += `${pad}    let pts: Vec<egui::Pos2> = (0..=${ellipseSegments}).map(|i| { let a = i as f32 * std::f32::consts::TAU / ${fmtF32(ellipseSegments)}; egui::pos2(cx + ${rx} * a.cos(), cy + ${ry} * a.sin()) }).collect();\n`;
    if (el.gradient) c += gradientPathMeshCode(el.gradient, "&pts", pad + "    ", el.opacity !== undefined ? el.opacity : 1.0);
    else if (el.fill) c += `${pad}    painter.add(egui::Shape::convex_polygon(pts.clone(), ${fc}, egui::Stroke::NONE));\n`;
    if (el.stroke) {
      const scn = colorMap.get(`${el.stroke.r},${el.stroke.g},${el.stroke.b}`) || "SURFACE";
      const strokeColor = strokeColorExpr(el, colorMap, scn);
      if ((el.pathPoints && el.pathPoints.length > 1) || hasRichStrokeSemantics(el)) {
        c += `${pad}    let rich_stroke = ${richStrokeExpr(el, colorMap, scn)};\n`;
        c += `${pad}    egui_expressive::dashed_path(&painter, &pts, &rich_stroke);\n`;
      } else {
        c += `${pad}    painter.add(egui::Shape::closed_line(pts, egui::Stroke::new(${fmtF32(el.stroke.width)}, ${strokeColor})));\n`;
      }
    }
    c += `${pad}}\n`;
    return c;
  }

  if (el.type === "text" && el.text) {
    // Use absolute painter position to preserve Illustrator coordinates.
    // el.x/el.y are the top-left of the bounding box from geometricBounds.
    // Adjust anchor point based on text alignment so egui renders at the correct position.
    const textAlign = el.textAlign || el.textStyle?.align || "left";
    // Compute the correct anchor x: left edge for LEFT, center for CENTER, right edge for RIGHT
    const anchorX = textAlign === "center" ? el.x + (el.w || 0) / 2
                  : textAlign === "right"  ? el.x + (el.w || 0)
                  : el.x;
    const tx = fmtF32(anchorX), ty = fmtF32(el.y);
    const align2 = textAlign === "center" ? "CENTER_TOP" : textAlign === "right" ? "RIGHT_TOP" : "LEFT_TOP";

    if (el.textRuns && el.textRuns.length > 1) {
      // Multi-run text — lay out runs left-to-right with line-break support.
      // For center/right alignment, compute total first-line width and shift the
      // block start so the composed line aligns correctly within the bounding box.
      const defaultSz = el.textStyle?.size || 14;
      // Estimate total width of the first line across all runs (for alignment offset).
      // Stop accumulating as soon as any run contains a newline — subsequent runs
      // are on line 2+ and should not contribute to the first-line width.
      let firstLineWidth = 0;
      let firstLineDone = false;
      for (const run of el.textRuns) {
        if (!run.text || firstLineDone) continue;
        const newlineIdx = run.text.indexOf("\n");
        const firstLinePart = newlineIdx >= 0 ? run.text.slice(0, newlineIdx) : run.text;
        const runSz = run.style?.size || defaultSz;
        firstLineWidth += firstLinePart.length * runSz * 0.55;
        if (newlineIdx >= 0) firstLineDone = true;
      }
      // blockStartX: left edge of the composed text block, adjusted for alignment
      const blockStartX = textAlign === "center" ? el.x + (el.w || 0) / 2 - firstLineWidth / 2
                        : textAlign === "right"  ? el.x + (el.w || 0) - firstLineWidth
                        : el.x;
      c += `${pad}{\n`;
      c += `${pad}    let _text_x0 = ${fmtF32(blockStartX)}f32;\n`;
      c += `${pad}    let _text_y0 = ${fmtF32(el.y)}f32;\n`;
      let xOffset = 0;
      let yOffset = 0;
      for (const run of el.textRuns) {
        if (!run.text) continue;
        // Split on newlines to handle multi-line runs
        const lines = run.text.split("\n");
        const runSz = run.style?.size || defaultSz;
        const runWt = run.style?.weight || el.textStyle?.weight || 400;
        const runColor = run.style?.color;
        const runCn = runColor ? (colorMap.get(`${runColor.r},${runColor.g},${runColor.b}`) || "ON_SURFACE") : "ON_SURFACE";
        const fontFamily = runWt >= 600 ? `egui::FontFamily::Name("Bold".into())` : `egui::FontFamily::Proportional`;
        for (let li = 0; li < lines.length; li++) {
          const lineText = lines[li];
          if (li > 0) {
            // Newline: advance y, reset x to block start
            yOffset += runSz * 1.2;
            xOffset = 0;
          }
          if (!lineText) continue;
          const runTxt = lineText.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\r/g, "\\r").replace(/\t/g, "\\t").replace(/\0/g, "\\0");
          c += `${pad}    painter.text(origin + egui::vec2(_text_x0 + ${fmtF32(xOffset)}, _text_y0 + ${fmtF32(yOffset)}), egui::Align2::LEFT_TOP, "${runTxt}", egui::FontId::new(${fmtF32(runSz)}, ${fontFamily}), tokens::${runCn});\n`;
          xOffset += lineText.length * runSz * 0.55;
        }
      }
      c += `${pad}}\n`;
    } else {
      // Single-style text at absolute position
      const cn = el.fill ? (colorMap.get(`${el.fill.r},${el.fill.g},${el.fill.b}`) || "ON_SURFACE") : "ON_SURFACE";
      const sz = el.textStyle?.size || 14;
      const wt = el.textStyle?.weight || 400;
      const txt = el.text.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n").replace(/\r/g, "\\r").replace(/\t/g, "\\t").replace(/\0/g, "\\0");
      const fontFamily = wt >= 600 ? `egui::FontFamily::Name("Bold".into())` : `egui::FontFamily::Proportional`;
      c += `${pad}painter.text(origin + egui::vec2(${tx}, ${ty}), egui::Align2::${align2}, "${txt}", egui::FontId::new(${fmtF32(sz)}, ${fontFamily}), tokens::${cn});\n`;
    }
    return c;
  }

  if (el.type === "path") {
    // Open path — emit sampled Bezier geometry instead of dropping control handles.
    if (el.pathPoints && el.pathPoints.length >= 2) {
      const cn = el.fill ? (colorMap.get(`${el.fill.r},${el.fill.g},${el.fill.b}`) || "SURFACE") : "SURFACE";
      const fc = applyBlendExpr(el.opacity < 1.0 ? `with_alpha(tokens::${cn}, ${el.opacity})` : `tokens::${cn}`, el.blendMode);
      const scn = el.stroke ? (colorMap.get(`${el.stroke.r},${el.stroke.g},${el.stroke.b}`) || "OUTLINE") : "OUTLINE";
      const sw = el.stroke?.width || 1;
      const sampled = samplePathPoints(el.pathPoints, !!el.pathClosed);
      c += `${pad}{\n`;
      c += `${pad}    let path_pts = ${rustPointsVec(sampled)};\n`;
      if (el.gradient && el.pathClosed !== false) c += gradientPathMeshCode(el.gradient, "&path_pts", pad + "    ", el.opacity !== undefined ? el.opacity : 1.0);
      if (el.stroke && hasRichStrokeSemantics(el)) {
        c += `${pad}    let rich_stroke = ${richStrokeExpr(el, colorMap, scn)};\n`;
        c += `${pad}    egui_expressive::dashed_path(&painter, &path_pts, &rich_stroke);\n`;
      } else if (el.stroke || el.fill) {
        const fillExpr = el.fill && !el.gradient ? fc : "egui::Color32::TRANSPARENT";
        const strokeExpr = el.stroke ? `egui::epaint::PathStroke::new(${fmtF32(sw)}, ${strokeColorExpr(el, colorMap, scn)})` : "egui::epaint::PathStroke::NONE";
        c += `${pad}    painter.add(egui::Shape::Path(egui::epaint::PathShape { points: path_pts, closed: ${el.pathClosed ? "true" : "false"}, fill: ${fillExpr}, stroke: ${strokeExpr} }));\n`;
      }
      c += `${pad}}\n`;
    } else {
      // No path points — fall back to rect stroke on bounding box
      const scn = el.stroke ? (colorMap.get(`${el.stroke.r},${el.stroke.g},${el.stroke.b}`) || "OUTLINE") : "OUTLINE";
      const sw = el.stroke?.width || 1;
      c += `${pad}painter.line_segment([origin + egui::vec2(${fmtF32(el.x)}, ${fmtF32(el.y + el.h/2)}), origin + egui::vec2(${fmtF32(el.x + el.w)}, ${fmtF32(el.y + el.h/2)})], egui::Stroke::new(${fmtF32(sw)}, ${strokeColorExpr(el, colorMap, scn)}));\n`;
    }
    return c;
  }

  if (el.type === "shape") {
    const cn = el.fill ? (colorMap.get(`${el.fill.r},${el.fill.g},${el.fill.b}`) || "SURFACE") : "SURFACE";
    const fc = applyBlendExpr(el.opacity < 1.0 ? `with_alpha(tokens::${cn}, ${el.opacity})` : `tokens::${cn}`, el.blendMode);
    const appearanceFills = el.appearance_fills || el.appearanceFills || [];
    const appearanceStrokes = el.appearance_strokes || el.appearanceStrokes || [];
    const hasAppearanceStack = el.appearanceStack?.length > 0 || appearanceFills.length > 0 || appearanceStrokes.length > 0;
    const layers = appearanceLayers(el, appearanceFills, appearanceStrokes);
    const cr = Math.min(255, Math.round(el.cornerRadius || 0)), rot = el.rotation || 0;

    if (hasAppearanceStack && appearanceHasNonNormalBlend(layers)) {
      return sceneBackedAppearanceCode(el, pad, layers);
    }

    c += `${pad}let rect = egui::Rect::from_min_size(origin + egui::vec2(${fmtF32(el.x)}, ${fmtF32(el.y)}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)}));\n`;

    // Use actual path geometry if available and non-rectangular
    const isRectangular = !el.pathPoints || el.pathPoints.length < 3;
    if (!isRectangular && el.pathPoints.length > 2) {
      const sampled = samplePathPoints(el.pathPoints, el.pathClosed !== false);
      c += `${pad}{\n`;
      c += `${pad}    let path_pts = ${rustPointsVec(sampled)};\n`;
      if (hasAppearanceStack) {
        c += `${pad}    // Illustrator appearance stack on sampled path geometry\n`;
        const renderPathFill = (layer) => {
          const opacity = appearanceOpacity(layer, 1) * (el.opacity !== undefined ? el.opacity : 1);
          const colorExpr = applyBlendExpr(appearanceColorExpr(layer, opacity), layer.blendMode || layer.blend_mode || "normal");
          if (layer.gradient) c += gradientPathMeshCode(layer.gradient, "&path_pts", pad + "    ", opacity);
          else c += `${pad}    painter.add(egui::Shape::Path(egui::epaint::PathShape { points: path_pts.clone(), closed: true, fill: ${colorExpr}, stroke: egui::epaint::PathStroke::NONE }));\n`;
        };
        const renderPathStroke = (layer) => {
          const opacity = appearanceOpacity(layer, 1) * (el.opacity !== undefined ? el.opacity : 1);
          const colorExpr = applyBlendExpr(appearanceColorExpr(layer, opacity), layer.blendMode || layer.blend_mode || "normal");
          const width = fmtF32(layer.width || 1);
          const dash = layer.dash || layer.strokeDash || null;
          const dashExpr = dash && dash.length > 0
            ? `Some(egui_expressive::DashPattern { dashes: vec![${dash.map(fmtF32).join(", ")}], offset: 0.0 })`
            : "None";
          c += `${pad}    let rich_stroke = egui_expressive::RichStroke { width: ${width}, color: ${colorExpr}, dash: ${dashExpr}, cap: egui_expressive::StrokeCap::${strokeCapVariant(layer.cap)}, join: egui_expressive::StrokeJoin::${strokeJoinVariant(layer.join, layer.miterLimit || layer.miter_limit)} };\n`;
          c += `${pad}    egui_expressive::dashed_path(&painter, &path_pts, &rich_stroke);\n`;
        };
        if (el.appearanceStack) {
          el.appearanceStack.forEach(layer => {
            if (layer.type === 'fill') renderPathFill(layer);
            else if (layer.type === 'stroke') renderPathStroke(layer);
          });
        } else {
          appearanceFills.forEach(renderPathFill);
          appearanceStrokes.forEach(renderPathStroke);
        }
        if (el.effects?.length > 0) c += `${pad}    // Path effects preserved in sidecar scene metadata; direct CEP fallback keeps editable path geometry.\n`;
      } else if (el.gradient) {
        c += gradientPathMeshCode(el.gradient, "&path_pts", pad + "    ", el.opacity !== undefined ? el.opacity : 1.0);
        if (el.stroke && hasRichStrokeSemantics(el)) {
          c += `${pad}    let rich_stroke = ${richStrokeExpr(el, colorMap, "SURFACE")};\n`;
          c += `${pad}    egui_expressive::dashed_path(&painter, &path_pts, &rich_stroke);\n`;
        } else if (el.stroke) {
          c += `${pad}    painter.add(egui::Shape::Path(egui::epaint::PathShape { points: path_pts, closed: ${el.pathClosed === false ? "false" : "true"}, fill: egui::Color32::TRANSPARENT, stroke: ${strokePathExpr(el, colorMap, "SURFACE")} }));\n`;
        }
      } else if (hasRichStrokeSemantics(el)) {
        c += `${pad}    painter.add(egui::Shape::Path(egui::epaint::PathShape { points: path_pts.clone(), closed: ${el.pathClosed === false ? "false" : "true"}, fill: ${el.fill ? fc : "egui::Color32::TRANSPARENT"}, stroke: egui::epaint::PathStroke::NONE }));\n`;
        c += `${pad}    let rich_stroke = ${richStrokeExpr(el, colorMap, "SURFACE")};\n`;
        c += `${pad}    egui_expressive::dashed_path(&painter, &path_pts, &rich_stroke);\n`;
      } else {
        c += `${pad}    painter.add(egui::Shape::Path(egui::epaint::PathShape { points: path_pts, closed: ${el.pathClosed === false ? "false" : "true"}, fill: ${el.fill ? fc : "egui::Color32::TRANSPARENT"}, stroke: ${strokePathExpr(el, colorMap, "SURFACE")} }));\n`;
      }
      c += `${pad}}\n`;
      return c;
    }

    // Drop shadow
    const shadowFxList = el.effects?.filter(e => e.type === "dropShadow") || [];
    for (const shadowFx of shadowFxList) {
      c += `${pad}for s in egui_expressive::box_shadow(rect, egui::Color32::from_rgba_unmultiplied(${shadowFx.color?.r||0}, ${shadowFx.color?.g||0}, ${shadowFx.color?.b||0}, ${Math.round((shadowFx.color?.a||0.5)*255)}), ${(shadowFx.blur||0).toFixed(1)}, 0.0, egui_expressive::ShadowOffset::new(${(shadowFx.x||0).toFixed(1)}, ${(shadowFx.y||0).toFixed(1)})) { painter.add(s); }\n`;
    }
    // Inner shadow
    const innerShadowFxList = el.effects?.filter(e => e.type === "innerShadow") || [];
    for (const innerShadowFx of innerShadowFxList) {
      c += `${pad}for s in egui_expressive::inner_shadow(rect, egui::Color32::from_rgba_unmultiplied(${innerShadowFx.color?.r||0}, ${innerShadowFx.color?.g||0}, ${innerShadowFx.color?.b||0}, ${Math.round((innerShadowFx.color?.a||0.5)*255)}), ${fmtF32(innerShadowFx.blur||4)}) { painter.add(s); }\n`;
    }
    // Inner glow
    const innerGlowFxList = el.effects?.filter(e => e.type === "innerGlow") || [];
    for (const innerGlowFx of innerGlowFxList) {
      c += `${pad}for s in egui_expressive::inner_shadow(rect, egui::Color32::from_rgba_unmultiplied(${innerGlowFx.color?.r||0}, ${innerGlowFx.color?.g||0}, ${innerGlowFx.color?.b||0}, ${Math.round((innerGlowFx.color?.a||0.5)*255)}), ${fmtF32(innerGlowFx.blur||4)}) { painter.add(s); }\n`;
    }
    // Outer glow
    const outerGlowFxList = el.effects?.filter(e => e.type === "outerGlow") || [];
    for (const outerGlowFx of outerGlowFxList) {
      c += `${pad}for s in egui_expressive::soft_shadow(rect, egui::Color32::from_rgba_unmultiplied(${outerGlowFx.color?.r||255}, ${outerGlowFx.color?.g||255}, ${outerGlowFx.color?.b||200}, ${Math.round((outerGlowFx.color?.a||0.6)*255)}), ${fmtF32(outerGlowFx.blur||8)}, 0.0, egui_expressive::ShadowOffset::zero(), egui_expressive::BlurQuality::Medium) { painter.add(s); }\n`;
    }
    if (hasBlur) {
      const bl = el.effects.find(e => e.type === "gaussianBlur");
      const blurR = bl?.radius || 4;
      // Gaussian blur approximated as soft glow (soft_shadow with zero offset)
      c += `${pad}for s in egui_expressive::soft_shadow(rect, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 60), ${fmtF32(blurR)}, 0.0, egui_expressive::ShadowOffset::zero(), egui_expressive::BlurQuality::Medium) { painter.add(s); }\n`;
    }
    const noiseFxList = el.effects?.filter(e => e.type === "noise" || e.type === "grain") || [];
    for (const noiseFx of noiseFxList) {
      c += `${pad}for s in egui_expressive::noise_rect(rect, ${Math.round(noiseFx.seed || 0)}, ${fmtF32(noiseFx.scale || 2)}, ${fmtF32(noiseFx.amount || 0.16)}) { painter.add(s); }\n`;
    }
    const featherFxList = el.effects?.filter(e => e.type === "feather") || [];
    for (const featherFx of featherFxList) {
      c += `${pad}for s in egui_expressive::soft_shadow(rect, ${fc}, ${fmtF32(featherFx.radius || 4)}, 0.0, egui_expressive::ShadowOffset::zero(), egui_expressive::BlurQuality::High) { painter.add(s); }\n`;
    }

    if (rot !== 0) {
      c += `${pad}let _rot = egui_expressive::Transform2D::rotate_around(${fmtF32(rot)}, rect.center());\n`;
      c += `${pad}let _pts = vec![_rot.apply(rect.min), _rot.apply(egui::pos2(rect.max.x, rect.min.y)), _rot.apply(rect.max), _rot.apply(egui::pos2(rect.min.x, rect.max.y))];\n`;
    }

    if (hasAppearanceStack) {
      c += `${pad}// Illustrator appearance stack\n`;
      const renderFill = (layer) => {
        const opacity = appearanceOpacity(layer, 1) * (el.opacity !== undefined ? el.opacity : 1);
        let colorExpr = applyBlendExpr(appearanceColorExpr(layer, opacity), layer.blendMode || layer.blend_mode || "normal");
        if (layer.gradient) {
          const g = layer.gradient;
          if (g.type === "linear" && cr <= 0 && rot === 0 && !g.transform && !g.matrix) {
            const stopsStr = (g.stops || []).map(s => { const c = stopColorToRgb(s.color); return `(${Number(s.position || 0).toFixed(3)}, egui::Color32::from_rgba_unmultiplied(${c.r}, ${c.g}, ${c.b}, ${Math.round((s.opacity !== undefined ? s.opacity : 1) * opacity * 255)}))`; }).join(", ");
            c += `${pad}painter.add(egui_expressive::linear_gradient_rect(rect, &[${stopsStr}], egui_expressive::GradientDir::Angle(${fmtF32(g.angle || 0)})));\n`;
          } else {
            c += `${pad}{\n`;
            c += `${pad}    let gradient_rect_pts = ${rectGradientPointsExpr(cr, rot !== 0 ? "_pts.clone()" : null)};\n`;
            c += gradientPathMeshCode(g, "&gradient_rect_pts", pad + "    ", opacity);
            c += `${pad}}\n`;
          }
        } else if (rot !== 0) {
          c += `${pad}painter.add(egui::Shape::convex_polygon(_pts.clone(), ${colorExpr}, egui::Stroke::NONE));\n`;
        } else {
          c += `${pad}painter.rect_filled(rect, ${cr}u8, ${colorExpr});\n`;
        }
      };
      const renderStroke = (layer) => {
        const opacity = appearanceOpacity(layer, 1) * (el.opacity !== undefined ? el.opacity : 1);
        let colorExpr = applyBlendExpr(appearanceColorExpr(layer, opacity), layer.blendMode || layer.blend_mode || "normal");
        const width = fmtF32(layer.width || 1);
        const dash = layer.dash || layer.strokeDash || null;
        if (dash && dash.length > 0) {
          c += `${pad}{\n`;
          c += `${pad}    let path_pts = ${closedRectStrokePointsExpr(cr, rot !== 0 ? "_pts.clone()" : null)};\n`;
          c += `${pad}    let rich_stroke = egui_expressive::RichStroke { width: ${width}, color: ${colorExpr}, dash: Some(egui_expressive::DashPattern { dashes: vec![${dash.map(fmtF32).join(", ")}], offset: 0.0 }), cap: egui_expressive::StrokeCap::${strokeCapVariant(layer.cap)}, join: egui_expressive::StrokeJoin::${strokeJoinVariant(layer.join, layer.miterLimit || layer.miter_limit)} };\n`;
          c += `${pad}    egui_expressive::dashed_path(&painter, &path_pts, &rich_stroke);\n`;
          c += `${pad}}\n`;
        } else if (rot !== 0 || cr > 0) {
          c += `${pad}painter.add(egui::Shape::closed_line(${closedRectStrokePointsExpr(cr, rot !== 0 ? "_pts.clone()" : null)}, egui::Stroke::new(${width}, ${colorExpr})));\n`;
        } else {
          c += `${pad}painter.rect_stroke(rect, ${cr}u8, egui::Stroke::new(${width}, ${colorExpr}), egui::StrokeKind::Outside);\n`;
        }
      };
      if (el.appearanceStack) {
        el.appearanceStack.forEach(layer => {
          if (layer.type === 'fill') renderFill(layer);
          else if (layer.type === 'stroke') renderStroke(layer);
        });
      } else {
        appearanceFills.forEach(renderFill);
        appearanceStrokes.forEach(renderStroke);
      }
    } else if (el.gradient) {
      const g = el.gradient;
      if (g.type === "linear" && cr <= 0 && rot === 0 && !g.transform && !g.matrix) {
        const stopsStr = (g.stops || []).map(s => { const c = stopColorToRgb(s.color); return `(${s.position.toFixed(3)}, egui::Color32::from_rgba_unmultiplied(${c.r}, ${c.g}, ${c.b}, ${Math.round((s.opacity !== undefined ? s.opacity : 1) * (el.opacity !== undefined ? el.opacity : 1) * 255)}))`; }).join(", ");
        c += `${pad}painter.add(egui_expressive::linear_gradient_rect(rect, &[${stopsStr}], egui_expressive::GradientDir::Angle(${(g.angle || 0).toFixed(1)})));\n`;
      } else {
        c += `${pad}{\n`;
        c += `${pad}    let gradient_rect_pts = ${rectGradientPointsExpr(cr, rot !== 0 ? "_pts.clone()" : null)};\n`;
        c += gradientPathMeshCode(g, "&gradient_rect_pts", pad + "    ", el.opacity !== undefined ? el.opacity : 1.0);
        c += `${pad}}\n`;
      }
    } else if (el.fill) {
      if (rot !== 0) {
        c += `${pad}painter.add(egui::Shape::convex_polygon(_pts.clone(), ${fc}, egui::Stroke::NONE));\n`;
      } else {
        c += `${pad}painter.rect_filled(rect, ${cr}u8, ${fc});\n`;
      }
    }
    if (el.stroke && !hasAppearanceStack) {
      const scn = colorMap.get(`${el.stroke.r},${el.stroke.g},${el.stroke.b}`) || "SURFACE";
      if (el.strokeDash && el.strokeDash.length > 0) {
        c += `${pad}{\n`;
        c += `${pad}    let path_pts = ${closedRectStrokePointsExpr(cr, rot !== 0 ? "_pts.clone()" : null)};\n`;
        c += `${pad}    let rich_stroke = ${richStrokeExpr(el, colorMap, scn)};\n`;
        c += `${pad}    egui_expressive::dashed_path(&painter, &path_pts, &rich_stroke);\n`;
        c += `${pad}}\n`;
      } else if (rot !== 0 || cr > 0) {
        c += `${pad}painter.add(egui::Shape::closed_line(${closedRectStrokePointsExpr(cr, rot !== 0 ? "_pts.clone()" : null)}, egui::Stroke::new(${fmtF32(el.stroke.width)}, ${strokeColorExpr(el, colorMap, scn)})));\n`;
      } else {
        c += `${pad}painter.rect_stroke(rect, ${cr}u8, egui::Stroke::new(${fmtF32(el.stroke.width)}, ${strokeColorExpr(el, colorMap, scn)}), egui::StrokeKind::Outside);\n`;
      }
    }
    const bevelFxList = el.effects?.filter(e => e.type === "bevel") || [];
    for (const bevelFx of bevelFxList) {
      const depth = fmtF32(Math.max(1, bevelFx.depth || 2));
      const hi = rgbaExpr(bevelFx.highlight || { r: 255, g: 255, b: 255, a: 0.65 }, 0.65);
      const sh = rgbaExpr(bevelFx.shadow || bevelFx.shadowColor || { r: 0, g: 0, b: 0, a: 0.35 }, 0.35);
      c += `${pad}// Bevel approximation\n`;
      c += `${pad}painter.line_segment([rect.left_top(), rect.right_top()], egui::Stroke::new(${depth}, ${hi}));\n`;
      c += `${pad}painter.line_segment([rect.left_top(), rect.left_bottom()], egui::Stroke::new(${depth}, ${hi}));\n`;
      c += `${pad}painter.line_segment([rect.left_bottom(), rect.right_bottom()], egui::Stroke::new(${depth}, ${sh}));\n`;
      c += `${pad}painter.line_segment([rect.right_top(), rect.right_bottom()], egui::Stroke::new(${depth}, ${sh}));\n`;
    }
    return c;
  }

  if (el.type === "image") {
    c += `${pad}{\n`;
    c += `${pad}    let rect = egui::Rect::from_min_size(origin + egui::vec2(${fmtF32(el.x)}, ${fmtF32(el.y)}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)}));\n`;
    const assetPath = el.imagePath ? (portableAssetPath(el.imagePath) || el.imagePath) : "assets/missing_image.png";
    if (!el.imagePath) {
      const note = el.embeddedRaster
        ? "Embedded raster image: no portable PNG path emitted; vectorization/code primitive needed for code-only workflow"
        : "Image element without linked path: emitted as missing image asset slot";
      c += `${pad}    // ${sanitizeComment(note)}\n`;
    }
    const alpha = clampByte((el.opacity !== undefined ? el.opacity : 1.0) * 255, 255);
    const tintExpr = alpha === 255 ? "egui::Color32::WHITE" : `egui::Color32::from_rgba_unmultiplied(255, 255, 255, ${alpha})`;
    c += `${pad}    if !egui_expressive::paint_image_from_path(ui, &painter, rect, ${rustString(assetPath)}, ${rustString("illustrator_img_" + sanitize(el.id))}, ${tintExpr}) {\n`;
    c += `${pad}        // Linked asset was not found at runtime; copy it to preview/generated/assets/.\n`;
    c += `${pad}        painter.rect_filled(rect, 0u8, egui::Color32::from_rgba_unmultiplied(255, 0, 0, 30));\n`;
    c += `${pad}        painter.rect_stroke(rect, 0u8, egui::Stroke::new(1.0, egui::Color32::RED), egui::StrokeKind::Outside);\n`;
    c += `${pad}        painter.text(rect.center(), egui::Align2::CENTER_CENTER, "Missing Image", egui::FontId::proportional(12.0), egui::Color32::RED);\n`;
    c += `${pad}    }\n`;
    c += `${pad}}\n`;
    return c;
  }

  if (el.type === "group" && el.children?.length > 0) {
    // Render children at their absolute positions (preserves Illustrator layout)
    c += `${pad}// Group: ${el.id}\n`;
    c += `${pad}{\n`;
    let handledAsClippedLayers = false;
    if (el.clipMask) {
      const isRectangular = !el.pathPoints || el.pathPoints.length < 3;
      if (!isRectangular) {
        const { code: layersCode, success } = tryGenerateBlendLayers(el.children, indent, colorMap, comps, options);
        if (success) {
          const sampled = samplePathPoints(el.pathPoints, el.pathClosed !== false);
          c += `${pad}    let clip_path = ${rustPointsVec(sampled)};\n`;
          c += `${pad}    egui_expressive::clipped_layers_gpu(ui, &clip_path, vec![\n`;
          c += layersCode;
          c += `${pad}    ]);\n`;
          handledAsClippedLayers = true;
        } else {
          c += `${pad}    // WARNING: Non-rect clip group children not representable as BlendLayer shapes. Falling back without exact parity.\n`;
          const sampled = samplePathPoints(el.pathPoints, el.pathClosed !== false);
          c += `${pad}    let clip_path = ${rustPointsVec(sampled)};\n`;
          c += `${pad}    let painter = egui_expressive::with_clip_path(&painter, clip_path);\n`;
        }
      } else {
        c += `${pad}    let clip_rect = egui::Rect::from_min_size(origin + egui::vec2(${fmtF32(el.x)}, ${fmtF32(el.y)}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)}));\n`;
        c += `${pad}    let painter = painter.with_clip_rect(clip_rect);\n`;
      }
    }
    if (!handledAsClippedLayers) {
      for (const ch of el.children) c += generateElementCode(ch, indent + 1, colorMap, comps, options);
    }
    c += `${pad}}\n`;
    return c;
  }

  return `${pad}// ${el.id} (${el.type})\n`;
}

function isSimpleVectorElement(el) {
  if (el.type === "group") return false;
  if (el.type === "image") return false;
  if (el.type === "text") return false;
  if (el.type === "symbol") return false;
  if (el.type === "plugin") return false;
  if (el.mesh_patches && el.mesh_patches.length > 0) return false;
  return true;
}

function tryGenerateBlendLayers(children, indent, colorMap, comps, options) {
  let layersCode = "";
  let success = true;
  const pad = "    ".repeat(indent);

  for (let i = 0; i < children.length; i++) {
    const ch = children[i];
    if (!isSimpleVectorElement(ch)) {
      success = false;
      break;
    }

    const originalBlend = ch.blendMode;
    const originalOpacity = ch.opacity;
    ch.blendMode = "normal";
    ch.opacity = 1.0;

    let innerCode = generateElementCodeInner(ch, indent + 1, colorMap, comps, options);

    ch.blendMode = originalBlend;
    ch.opacity = originalOpacity;

    let canConvert = true;
    let shapesCode = innerCode.split('\n').map(line => {
      if (!line.trim()) return line;
      if (line.includes('egui_expressive::dashed_path') || line.includes('painter.text') || line.includes('paint_image_from_path') || line.includes('pattern_fill_path') || line.includes('scene::render_node')) {
        canConvert = false;
      }
      let l = line;
      l = l.replace(/painter\.add\((.*)\);/g, "_blend_shapes.push($1);");
      l = l.replace(/painter\.rect_filled\((.*?),\s*(.*?),\s*(.*?)\);/g, "_blend_shapes.push(egui::Shape::rect_filled($1, $2, $3));");
      l = l.replace(/painter\.rect_stroke\((.*?),\s*(.*?),\s*(.*?),\s*egui::StrokeKind::Outside\);/g, "_blend_shapes.push(egui::Shape::rect_stroke($1.expand($3.width / 2.0), $2, $3, egui::StrokeKind::Outside));");
      l = l.replace(/painter\.rect_stroke\((.*?),\s*(.*?),\s*(.*?),\s*.*?\);/g, "_blend_shapes.push(egui::Shape::rect_stroke($1, $2, $3, egui::StrokeKind::Middle));");
      l = l.replace(/painter\.circle_filled\((.*?),\s*(.*?),\s*(.*?)\);/g, "_blend_shapes.push(egui::Shape::circle_filled($1, $2, $3));");
      l = l.replace(/painter\.circle_stroke\((.*?),\s*(.*?),\s*(.*?)\);/g, "_blend_shapes.push(egui::Shape::circle_stroke($1, $2, $3));");
      l = l.replace(/painter\.line_segment\((.*?),\s*(.*?)\);/g, "_blend_shapes.push(egui::Shape::line_segment($1, $2));");
      return l;
    }).join('\n');

    if (!canConvert) {
      success = false;
      break;
    }

    const variant = blendModeRust(originalBlend) || "Normal";
    layersCode += `${pad}    egui_expressive::BlendLayer {\n`;
    layersCode += `${pad}        shapes: {\n`;
    layersCode += `${pad}            let mut _blend_shapes = vec![];\n`;
    layersCode += shapesCode;
    layersCode += `${pad}            _blend_shapes\n`;
    layersCode += `${pad}        },\n`;
    layersCode += `${pad}        blend_mode: egui_expressive::BlendMode::${variant},\n`;
    layersCode += `${pad}        opacity: ${fmtF32(originalOpacity !== undefined ? originalOpacity : 1.0)},\n`;
    layersCode += `${pad}    },\n`;
  }

  return { code: layersCode, success };
}

function generateElementCode(el, indent, colorMap, comps, options) {
  if (el.blendMode && el.blendMode !== "normal") {
    const variant = blendModeRust(el.blendMode);
    if (variant) {
      const pad = "    ".repeat(indent);

      if (isSimpleVectorElement(el)) {
        const { code: layersCode, success } = tryGenerateBlendLayers([el], indent + 1, colorMap, comps, options);
        if (success) {
          let c = `${pad}{\n`;
          c += `${pad}    egui_expressive::composite_layers_gpu(ui, vec![\n`;
          c += layersCode;
          c += `${pad}    ]);\n`;
          c += `${pad}}\n`;
          return c;
        }
      }

      let c = `${pad}{\n`;
      c += `${pad}    // WARNING: Complex element with blend mode ${el.blendMode} not fully supported by composite_layers_gpu. Emitting with fallback.\n`;
      c += generateElementCodeInner(el, indent + 1, colorMap, comps, options);
      c += `${pad}}\n`;
      return c;
    }
  }
  return generateElementCodeInner(el, indent, colorMap, comps, options);
}

// ─── Helpers ─────────────────────────────────────────────────────────────────
function fmtF32(n) { const v = Number(n) || 0; return Number.isInteger(v) ? v + ".0" : String(v); }
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
function toActionName(t) { const RUST_KEYWORDS = new Set(["Self","Some","None","Ok","Err","True","False","Box","Vec","String","Option","Result","Async","Await","Dyn","Move","Impl","Where","Type"]); let s = (t || "Action").trim().replace(/[^a-zA-Z0-9]+/g, "_").split("_").map(s => s.charAt(0).toUpperCase() + s.slice(1)).join(""); if (/^[0-9]/.test(s)) s = "A" + s; if (RUST_KEYWORDS.has(s)) s = s + "Action"; return s || "Action"; }
function sanitize(n) {
  const RUST_KEYWORDS = new Set(["as","break","const","continue","crate","else","enum","extern","false","fn","for","if","impl","in","let","loop","match","mod","move","mut","pub","ref","return","self","static","struct","super","trait","true","type","unsafe","use","where","while","async","await","dyn"]);
  let s = (n || "field").toLowerCase().replace(/[^a-z0-9_]+/g, "_").replace(/^_+|_+$/g, "").slice(0, 32) || "field";
  if (/^[0-9]/.test(s)) s = "f_" + s;
  if (RUST_KEYWORDS.has(s)) s = s + "_";
  return s;
}
function getColorName(fill, colorMap) { return fill ? (colorMap.get(`${fill.r},${fill.g},${fill.b}`) || "SURFACE") : "SURFACE"; }

function rustString(s) {
  return JSON.stringify(String(s || ""));
}

function blendModeRust(mode) {
  const map = {
    multiply: "Multiply", screen: "Screen", overlay: "Overlay", darken: "Darken", lighten: "Lighten",
    color_dodge: "ColorDodge", color_burn: "ColorBurn", hard_light: "HardLight", soft_light: "SoftLight",
    difference: "Difference", exclusion: "Exclusion", hue: "Hue", saturation: "Saturation", color: "Color", luminosity: "Luminosity"
  };
  return map[String(mode || "normal")] || null;
}

function applyBlendExpr(expr, blendMode) {
  if (!blendMode || blendMode === "normal") return expr;
  const variant = blendModeRust(blendMode);
  if (!variant) return expr;
  return `egui_expressive::blend_color(${expr}, tokens::SURFACE, egui_expressive::BlendMode::${variant})`;
}

function codegenBlendModeExpr(mode) {
  const variant = blendModeRust(mode) || "Normal";
  return `egui_expressive::codegen::BlendMode::${variant}`;
}

function effectTypeExpr(type) {
  const map = {
    dropShadow: "DropShadow", "drop-shadow": "DropShadow",
    innerShadow: "InnerShadow", "inner-shadow": "InnerShadow",
    outerGlow: "OuterGlow", "outer-glow": "OuterGlow",
    innerGlow: "InnerGlow", "inner-glow": "InnerGlow",
    gaussianBlur: "GaussianBlur", "gaussian-blur": "GaussianBlur",
    bevel: "Bevel", feather: "Feather", noise: "Noise", grain: "Noise",
    liveEffect: "LiveEffect", "live-effect": "LiveEffect",
  };
  const variant = map[String(type || "")];
  return variant ? `egui_expressive::codegen::EffectType::${variant}` : `egui_expressive::codegen::EffectType::Unknown(${rustString(type || "unknown")}.to_string())`;
}

function optionColorExpr(color) {
  return color ? `Some(${rgbaExpr(color, 1)})` : "None";
}

function effectDefExpr(effect) {
  const ty = effect.effectType || effect.effect_type || effect.type;
  return `egui_expressive::codegen::EffectDef { effect_type: ${effectTypeExpr(ty)}, x: ${fmtF32(effect.x || 0)}, y: ${fmtF32(effect.y || 0)}, blur: ${fmtF32(effect.blur || 0)}, spread: ${fmtF32(effect.spread || 0)}, color: ${rgbaExpr(effect.color || { r: 0, g: 0, b: 0, a: 1 }, 1)}, blend_mode: ${codegenBlendModeExpr(effect.blendMode || effect.blend_mode)}, depth: ${fmtF32(effect.depth || 0)}, angle: ${fmtF32(effect.angle || 0)}, highlight: ${optionColorExpr(effect.highlight)}, shadow_color: ${optionColorExpr(effect.shadowColor || effect.shadow)}, radius: ${fmtF32(effect.radius || 0)}, amount: ${fmtF32(effect.amount || 0)}, scale: ${fmtF32(effect.scale || 1)}, seed: ${Math.max(0, Math.round(effect.seed || 0))} }`;
}

function gradientDefExpr(g) {
  const kind = g.type === "radial" ? "Radial" : "Linear";
  const stops = (g.stops || []).map(s => {
    const c = stopColorToRgb(s.color);
    const a = Math.round((s.opacity !== undefined ? s.opacity : 1) * 255);
    return `egui_expressive::codegen::GradientStop { position: ${fmtF32(Number(s.position || 0))}, color: egui::Color32::from_rgba_unmultiplied(${c.r}, ${c.g}, ${c.b}, ${a}) }`;
  }).join(", ");
  const point = (p) => p && Number.isFinite(Number(p.x)) && Number.isFinite(Number(p.y)) ? `Some([${fmtF32(p.x)}, ${fmtF32(p.y)}])` : "None";
  const radius = Number.isFinite(Number(g.radius)) && Number(g.radius) > 0 ? `Some(${fmtF32(g.radius)})` : "None";
  const m = g.transform || g.matrix;
  const transform = Array.isArray(m) && m.length >= 6 && m.every(v => Number.isFinite(Number(v)))
    ? `Some([${m.slice(0, 6).map(fmtF32).join(", ")}])`
    : "None";
  return `egui_expressive::codegen::GradientDef { gradient_type: egui_expressive::codegen::GradientType::${kind}, angle_deg: ${fmtF32(g.angle || 0)}, center: ${point(g.center)}, focal_point: ${point(g.focalPoint || g.focal_point)}, radius: ${radius}, transform: ${transform}, stops: vec![${stops}] }`;
}

function patternDefExpr(g) {
  const { seed, cell, mark } = patternMetrics(g);
  const r = 64 + (seed & 0x7f);
  const gr = 64 + ((seed >>> 8) & 0x7f);
  const b = 64 + ((seed >>> 16) & 0x7f);
  const name = g.patternName || g.pattern_name || g.name || g.type || "pattern";
  return `egui_expressive::scene::PatternDef { name: ${rustString(name)}.to_string(), seed: ${seed}u32, foreground: egui::Color32::from_rgba_unmultiplied(${r}, ${gr}, ${b}, 220), background: egui::Color32::from_rgba_unmultiplied(${255 - Math.floor(r / 2)}, ${255 - Math.floor(gr / 2)}, ${255 - Math.floor(b / 2)}, 48), cell_size: ${fmtF32(cell)}, mark_size: ${fmtF32(mark)} }`;
}

function paintSourceExpr(layer) {
  const g = layer.gradient || layer.pattern;
  if (g) {
    if (g.type === "linear") return `egui_expressive::scene::PaintSource::LinearGradient(${gradientDefExpr(g)})`;
    if (g.type === "radial") return `egui_expressive::scene::PaintSource::RadialGradient(${gradientDefExpr(g)})`;
    return `egui_expressive::scene::PaintSource::Pattern(${patternDefExpr(g)})`;
  }
  return `egui_expressive::scene::PaintSource::Solid(${rgbaExpr(layer.color || layer, 1)})`;
}

function rustLocalPointsVec(points) {
  return `vec![${points.map(p => `egui::pos2(${fmtF32(p[0])}, ${fmtF32(p[1])})`).join(", ")}]`;
}

function sceneGeometryExpr(el) {
  if (el.pathPoints && el.pathPoints.length >= 2) {
    const sampled = samplePathPoints(el.pathPoints, el.pathClosed !== false);
    return `egui_expressive::scene::Geometry::Path { points: ${rustLocalPointsVec(sampled)}, closed: ${el.pathClosed === false ? "false" : "true"} }`;
  }
  if (el.type === "circle" || el.type === "ellipse") {
    return `egui_expressive::scene::Geometry::Ellipse { rect: egui::Rect::from_min_size(egui::pos2(${fmtF32(el.x)}, ${fmtF32(el.y)}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)})) }`;
  }
  return `egui_expressive::scene::Geometry::Rect { rect: egui::Rect::from_min_size(egui::pos2(${fmtF32(el.x)}, ${fmtF32(el.y)}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)})), corner_radius: ${fmtF32(el.cornerRadius || 0)} }`;
}

function appearanceLayerKind(layer) {
  const t = layer.entryType || layer.kind || layer.type;
  if (t === "fill" || t === "stroke" || t === "effect") return t;
  if (["dropShadow", "drop-shadow", "innerShadow", "inner-shadow", "outerGlow", "outer-glow", "innerGlow", "inner-glow", "gaussianBlur", "gaussian-blur", "bevel", "feather", "noise", "grain", "liveEffect", "live-effect"].includes(t)) return "effect";
  return t;
}

function appearanceLayers(el, appearanceFills, appearanceStrokes) {
  if (el.appearanceStack && el.appearanceStack.length > 0) return el.appearanceStack;
  return [
    ...(appearanceFills || []).map(layer => ({ ...layer, type: "fill" })),
    ...(el.effects || []).map(effect => ({ ...effect, type: effect.type || effect.effectType || "effect", entryType: "effect" })),
    ...(appearanceStrokes || []).map(layer => ({ ...layer, type: "stroke" })),
  ];
}

function appearanceHasNonNormalBlend(layers) {
  return layers.some(layer => {
    const mode = layer.blendMode || layer.blend_mode;
    return mode && mode !== "normal";
  });
}

function sceneAppearanceEntriesExpr(layers, pad) {
  let c = "";
  for (const layer of layers) {
    const kind = appearanceLayerKind(layer);
    const opacity = appearanceOpacity(layer, 1);
    const blend = codegenBlendModeExpr(layer.blendMode || layer.blend_mode);
    if (kind === "fill") {
      c += `${pad}        egui_expressive::scene::AppearanceEntry::Fill(egui_expressive::scene::FillLayer { paint: ${paintSourceExpr(layer)}, opacity: ${fmtF32(opacity)}, blend_mode: ${blend} }),\n`;
    } else if (kind === "stroke") {
      const dash = layer.dash || layer.strokeDash;
      const dashExpr = dash && dash.length > 0 ? `Some(vec![${dash.map(fmtF32).join(", ")}])` : "None";
      c += `${pad}        egui_expressive::scene::AppearanceEntry::Stroke(egui_expressive::scene::StrokeLayer { paint: egui_expressive::scene::PaintSource::Solid(${rgbaExpr(layer.color || layer, 1)}), width: ${fmtF32(layer.width || 1)}, opacity: ${fmtF32(opacity)}, blend_mode: ${blend}, cap: Some(egui_expressive::codegen::StrokeCap::${strokeCapVariant(layer.cap)}), join: Some(egui_expressive::codegen::StrokeJoin::${strokeJoinVariant(layer.join, layer.miterLimit || layer.miter_limit)}), dash: ${dashExpr}, miter_limit: ${Number.isFinite(Number(layer.miterLimit || layer.miter_limit)) ? `Some(${fmtF32(layer.miterLimit || layer.miter_limit)})` : "None"} }),\n`;
    } else if (kind === "effect") {
      c += `${pad}        egui_expressive::scene::AppearanceEntry::Effect(egui_expressive::scene::EffectLayer { effect_type: ${effectTypeExpr(layer.effectType || layer.effect_type || layer.type)}, params: ${effectDefExpr(layer)}, opacity: ${fmtF32(opacity)}, blend_mode: ${blend} }),\n`;
    }
  }
  return c;
}

function sceneBackedAppearanceCode(el, pad, layers) {
  const pathBacked = el.pathPoints && el.pathPoints.length >= 2;
  let c = `${pad}// Appearance stack contains layer blend modes; route through scene renderer for ordered compositing parity.\n`;
  c += `${pad}{\n`;
  c += `${pad}    let scene_node = egui_expressive::scene::SceneNode {\n`;
  c += `${pad}        id: ${rustString(el.id)}.to_string(),\n`;
  c += `${pad}        geometry: ${sceneGeometryExpr(el)},\n`;
  c += `${pad}        appearance: egui_expressive::scene::AppearanceStack { entries: vec![\n`;
  c += sceneAppearanceEntriesExpr(layers, pad);
  c += `${pad}        ] },\n`;
  c += `${pad}        opacity: ${fmtF32(el.opacity !== undefined ? el.opacity : 1)},\n`;
  c += `${pad}        blend_mode: ${codegenBlendModeExpr(el.blendMode)},\n`;
  c += `${pad}        rotation_deg: ${pathBacked ? "0.0" : fmtF32(el.rotation || 0)},\n`;
  c += `${pad}        clip_children: false,\n`;
  c += `${pad}        children: vec![],\n`;
  c += `${pad}    };\n`;
  c += `${pad}    egui_expressive::scene::render_node(ui, painter, origin.to_vec2(), &scene_node, 1.0);\n`;
  c += `${pad}}\n`;
  return c;
}

function rgbaExpr(c, alphaFallback) {
  const rgb = stopColorToRgb(c);
  const alpha = clampByte((c && c.a !== undefined ? c.a : alphaFallback) * 255, Math.round((alphaFallback || 1) * 255));
  return `egui::Color32::from_rgba_unmultiplied(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha})`;
}

function appearanceOpacity(layer, fallback) {
  const raw = layer && layer.opacity !== undefined ? Number(layer.opacity) : fallback;
  if (!Number.isFinite(raw)) return fallback;
  return raw > 1 ? raw / 100 : raw;
}

function appearanceColorExpr(layer, opacity) {
  return rgbaExpr(layer && layer.color ? stopColorToRgb(layer.color) : layer, opacity);
}

function strokePathExpr(el, colorMap, fallbackToken) {
  if (!el.stroke) return "egui::epaint::PathStroke::NONE";
  const scn = colorMap.get(`${el.stroke.r},${el.stroke.g},${el.stroke.b}`) || fallbackToken || "SURFACE";
  return `egui::epaint::PathStroke::new(${fmtF32(el.stroke.width || 1)}, ${strokeColorExpr(el, colorMap, scn)})`;
}

function richStrokeExpr(el, colorMap, fallbackToken) {
  const scn = el.stroke ? (colorMap.get(`${el.stroke.r},${el.stroke.g},${el.stroke.b}`) || fallbackToken || "SURFACE") : (fallbackToken || "SURFACE");
  const dash = el.strokeDash && el.strokeDash.length > 0
    ? `Some(egui_expressive::DashPattern { dashes: vec![${el.strokeDash.map(fmtF32).join(", ")}], offset: 0.0 })`
    : "None";
  const cap = strokeCapVariant(el.strokeCap);
  const join = strokeJoinVariant(el.strokeJoin, el.strokeMiterLimit || el.miterLimit);
  return `egui_expressive::RichStroke { width: ${fmtF32(el.stroke?.width || 1)}, color: ${strokeColorExpr(el, colorMap, scn)}, dash: ${dash}, cap: egui_expressive::StrokeCap::${cap}, join: egui_expressive::StrokeJoin::${join} }`;
}

function strokeColorExpr(el, colorMap, fallbackToken) {
  if (!el.stroke) return "egui::Color32::TRANSPARENT";
  const scn = colorMap.get(`${el.stroke.r},${el.stroke.g},${el.stroke.b}`) || fallbackToken || "SURFACE";
  const opacity = el.opacity !== undefined ? el.opacity : 1.0;
  const base = opacity < 1.0 ? `with_alpha(tokens::${scn}, ${opacity})` : `tokens::${scn}`;
  return applyBlendExpr(base, el.blendMode);
}

function hasRichStrokeSemantics(el) {
  return (el.strokeDash && el.strokeDash.length > 0)
    || (el.strokeCap && el.strokeCap !== "butt")
    || (el.strokeJoin && el.strokeJoin !== "miter")
    || (Number.isFinite(Number(el.strokeMiterLimit || el.miterLimit)) && Number(el.strokeMiterLimit || el.miterLimit) <= 1);
}

function strokeCapVariant(cap) {
  return cap === "round" ? "Round" : cap === "square" ? "Square" : "Butt";
}

function strokeJoinVariant(join, miterLimit) {
  if (join === "round") return "Round";
  if (join === "bevel") return "Bevel";
  if (Number.isFinite(Number(miterLimit)) && Number(miterLimit) <= 1) return "Bevel";
  return "Miter";
}

function pointsDiffer(a, b) {
  if (!a || !b) return false;
  return Math.abs(Number(a[0]) - Number(b[0])) > 0.01 || Math.abs(Number(a[1]) - Number(b[1])) > 0.01;
}

function cubicAt(p0, c0, c1, p1, t) {
  const mt = 1 - t;
  return [
    mt * mt * mt * p0[0] + 3 * mt * mt * t * c0[0] + 3 * mt * t * t * c1[0] + t * t * t * p1[0],
    mt * mt * mt * p0[1] + 3 * mt * mt * t * c0[1] + 3 * mt * t * t * c1[1] + t * t * t * p1[1]
  ];
}

function samplePathPoints(pathPoints, closed) {
  if (!pathPoints || pathPoints.length === 0) return [];
  const pts = [[Number(pathPoints[0].anchor[0]) || 0, Number(pathPoints[0].anchor[1]) || 0]];
  const segCount = closed ? pathPoints.length : pathPoints.length - 1;
  for (let i = 0; i < segCount; i++) {
    const curr = pathPoints[i];
    const next = pathPoints[(i + 1) % pathPoints.length];
    const p0 = curr.anchor, c0 = curr.rightDir || curr.anchor, c1 = next.leftDir || next.anchor, p1 = next.anchor;
    const isCurve = pointsDiffer(c0, p0) || pointsDiffer(c1, p1);
    if (isCurve) {
      for (let step = 1; step <= 12; step++) pts.push(cubicAt(p0, c0, c1, p1, step / 12));
    } else {
      pts.push([Number(p1[0]) || 0, Number(p1[1]) || 0]);
    }
  }
  return pts;
}

function rustPointsVec(points) {
  return `vec![${points.map(p => `origin + egui::vec2(${fmtF32(p[0])}, ${fmtF32(p[1])})`).join(", ")}]`;
}

// ─── JSON Sidecar ────────────────────────────────────────────────────────────
function colorToHex(c) {
  if (!c) return undefined;
  return "#" + [c.r, c.g, c.b].map(v => clampByte(v, 0).toString(16).padStart(2, "0")).join("");
}

function mapEffectForSidecar(fx) {
  if (!fx) return undefined;
  const mapped = { ...fx };
  if (mapped.color) { mapped.color = colorToHex(mapped.color); }
  else { mapped.color = "#000000"; } // default color for effects without color (gaussianBlur, feather, etc.)
  if (mapped.highlight) { mapped.highlight = colorToHex(mapped.highlight); }
  if (mapped.shadow) { mapped.shadowColor = colorToHex(mapped.shadow); delete mapped.shadow; }
  return mapped;
}

function mapGradientForSidecar(g) {
  if (!g) return undefined;
  const mapped = { ...g };
  const type = mapped.type || "linear";
  if (type === "pattern" || (type !== "linear" && type !== "radial")) {
    const { seed, cell, mark } = patternMetrics(mapped);
    mapped.patternName = mapped.patternName || mapped.pattern_name || mapped.name || type || "pattern";
    mapped.seed = seed;
    mapped.cellSize = cell;
    mapped.markSize = mark;
  }
  if (mapped.stops) {
    mapped.stops = mapped.stops.map(s => ({
      ...s,
      color: colorToHex(s.color),
    }));
  }
  if (mapped.center) mapped.center = mapped.center;
  return mapped;
}

function sidecarType(t) {
  switch(t) {
    case 'circle': return 'circle';
    case 'ellipse': return 'ellipse';
    case 'symbol': return 'shape';
    case 'group': return 'group';
    case 'path': return 'path';
    case 'text': return 'text';
    case 'image': return 'image';
    case 'shape': case 'rect': return 'shape';
    default: return 'shape';
  }
}

function generateSidecar(ab, els, colorMap) {
  const mapElement = (el, childDepth) => {
    const result = {
      id: el.id, type: sidecarType(el.type), x: el.x, y: el.y, w: el.w, h: el.h, depth: childDepth !== undefined ? childDepth : el.depth,
      fill: colorToHex(el.fill),
      stroke: colorToHex(el.stroke),
      strokeWidth: el.stroke?.width || undefined,
      text: el.text || undefined,
      textStyle: el.textStyle ? { fontSize: el.textStyle.size, fontWeight: el.textStyle.weight, fontFamily: el.textStyle.family } : undefined,
      opacity: el.opacity !== 1 ? el.opacity : undefined, rotation: el.rotation !== 0 ? el.rotation : undefined,
      cornerRadius: el.cornerRadius > 0 ? el.cornerRadius : undefined,
      gradient: mapGradientForSidecar(el.gradient),
      blendMode: el.blendMode !== "normal" ? el.blendMode : undefined, strokeCap: el.strokeCap, strokeJoin: el.strokeJoin,
      strokeDash: el.strokeDash, strokeMiterLimit: el.strokeMiterLimit,
      appearanceFills: el.appearance_fills && el.appearance_fills.length > 0 ? el.appearance_fills.map(f => ({ color: colorToHex(f.color || f), opacity: f.opacity, blendMode: f.blendMode || f.blend_mode, gradient: mapGradientForSidecar(f.gradient) })) : undefined,
      appearanceStrokes: el.appearance_strokes && el.appearance_strokes.length > 0 ? el.appearance_strokes.map(s => ({ color: colorToHex(s.color || s), width: s.width, opacity: s.opacity, blendMode: s.blendMode || s.blend_mode, cap: s.cap, join: s.join, dash: s.dash, miterLimit: s.miterLimit })) : undefined,
      effects: el.effects.length > 0 ? el.effects.map(mapEffectForSidecar) : undefined,
      appearanceStack: el.appearanceStack || (
        (el.appearance_fills?.length || el.appearance_strokes?.length || el.effects?.length) ?
        [
          ...(el.appearance_fills || []).map(f => ({ type: 'fill', color: colorToHex(f.color || f), opacity: f.opacity, blendMode: f.blendMode || f.blend_mode, gradient: mapGradientForSidecar(f.gradient) })),
          ...(el.effects || []).map(e => ({ ...mapEffectForSidecar(e), entryType: 'effect', effectType: e.type })),
          ...(el.appearance_strokes || []).map(s => ({ type: 'stroke', color: colorToHex(s.color || s), width: s.width, opacity: s.opacity, blendMode: s.blendMode || s.blend_mode, cap: s.cap, join: s.join, dash: s.dash, miterLimit: s.miterLimit }))
        ] : undefined
      ),
      textAlign: el.textAlign, letterSpacing: el.letterSpacing, lineHeight: el.lineHeight,
      textDecoration: el.textDecoration, textTransform: el.textTransform, textRuns: el.textRuns,
      clipChildren: el.clipMask || undefined, symbolName: el.symbolName, isCompoundPath: el.isCompoundPath || undefined,
      isGradientMesh: el.isGradientMesh || undefined, isChart: el.isChart || undefined,
      meshPatches: el.mesh_patches || undefined,
      thirdPartyEffects: el.thirdPartyEffects && el.thirdPartyEffects.length > 0 ? el.thirdPartyEffects : undefined,
      isOpaque: el.isOpaque || undefined, notes: el.notes.length > 0 ? el.notes : undefined,
      pathPoints: el.pathPoints ? el.pathPoints.map(p => ({ ...p, left_ctrl: p.leftDir, right_ctrl: p.rightDir })) : undefined, pathClosed: el.pathClosed || undefined,
      imagePath: el.imagePath ? portableAssetPath(el.imagePath) : undefined,
      embeddedRaster: el.embeddedRaster || undefined,
    };
    // Preserve full nesting — recursively map children
    if (el.children?.length > 0) {
      result.children = el.children.map(ch => mapElement(ch, (childDepth !== undefined ? childDepth : el.depth) + 1));
    }
    return result;
  };
  return JSON.stringify({
    artboard: { name: ab.name, width: ab.width, height: ab.height },
    colors: [...colorMap.entries()].map(([k, n]) => { const [r, g, b] = k.split(",").map(Number); return { name: n, r, g, b }; }),
    elements: els.map(mapElement),
  }, null, 2);
}

// ─── ai-parser Integration ────────────────────────────────────────────────────
function getNodeModule(name) {
    try { return require(name); } catch (e) { noteAiParserDiagnostic(`Node module ${name} unavailable`, e); return null; }
}

function getPluginDirectory() {
    if (typeof __dirname !== "undefined" && __dirname) return __dirname;
    if (typeof window !== "undefined" && window.location) {
        try {
            const url = new URL(window.location.href.split(/[?#]/)[0]);
            let root = decodeURIComponent(url.pathname || "").replace(/\/index\.html$/i, "");
            if (/^\/[A-Za-z]:/.test(root)) root = root.slice(1);
            return root || ".";
        } catch (e) {
            noteAiParserDiagnostic("Could not resolve plugin directory from panel URL", e);
        }
    }
    return ".";
}

function getAiParserPlatformDir(platformValue) {
    const platform = platformValue || (typeof process !== "undefined" ? process.platform : "unknown");
    if (platform === "win32") return "win32";
    if (platform === "darwin") return "darwin";
    if (platform === "linux") return "linux";
    return platform || "unknown";
}

function getAiParserCandidates(pluginDir, platformValue) {
    const path = getNodeModule("path") || { join: (...args) => args.join("/").replace(/\/+/g, "/") };
    const platformDir = getAiParserPlatformDir(platformValue);
    const binaryName = platformDir === "win32" ? "ai-parser.exe" : "ai-parser";
    return [
        path.join(pluginDir, "bin", platformDir, binaryName),
        path.join(pluginDir, "bin", binaryName),
        path.join(pluginDir, binaryName),
        path.join(pluginDir, "..", "bin", platformDir, binaryName)
    ];
}

function findAiParserBinary() {
    aiParserStatus.checked = true;
    aiParserStatus.diagnostics = [];
    const fs = getNodeModule("fs");
    if (!fs || typeof fs.existsSync !== "function") {
        aiParserStatus.available = false;
        noteAiParserDiagnostic("Cannot probe bundled ai-parser", "Node fs API unavailable in this host");
        return null;
    }

    const pluginDir = getPluginDirectory();
    const candidates = getAiParserCandidates(pluginDir);
    for (const candidate of candidates) {
        try {
            if (fs.existsSync(candidate)) {
                aiParserStatus.available = true;
                aiParserStatus.binaryPath = candidate;
                return candidate;
            }
        } catch (e) {
            noteAiParserDiagnostic(`Cannot access candidate ${candidate}`, e);
        }
    }

    aiParserStatus.available = false;
    aiParserStatus.binaryPath = null;
    noteAiParserDiagnostic("Bundled ai-parser not found", `Checked: ${candidates.join(", ")}`);
    return null;
}

function checkAiParserAvailable() {
    return !!findAiParserBinary();
}

function getAiParserStatus() {
    if (!aiParserStatus.checked) findAiParserBinary();
    return {
        checked: aiParserStatus.checked,
        available: aiParserStatus.available,
        binaryPath: aiParserStatus.binaryPath,
        diagnostics: aiParserStatus.diagnostics.slice()
    };
}

function getAiParserDiagnostics() {
    return aiParserStatus.diagnostics.slice();
}

async function runAiParser(filePath) {
    const binaryPath = findAiParserBinary();
    if (!binaryPath) return null;
    if (!filePath) {
        aiParserStatus.available = false;
        noteAiParserDiagnostic("Cannot run ai-parser", "No Illustrator document path was available");
        return null;
    }

    const childProcess = getNodeModule("child_process");
    if (!childProcess || typeof childProcess.execFileSync !== "function") {
        aiParserStatus.available = false;
        noteAiParserDiagnostic("Cannot run bundled ai-parser", "child_process.execFileSync unavailable in this host");
        return null;
    }

    try {
        const output = childProcess.execFileSync(binaryPath, [filePath, "--pretty"], {
            encoding: "utf8",
            timeout: 15000,
            windowsHide: true
        });
        aiParserStatus.available = true;
        aiParserStatus.binaryPath = binaryPath;
        return JSON.parse(output);
    } catch (e) {
        aiParserStatus.available = false;
        noteAiParserDiagnostic(`ai-parser failed for ${basename(filePath)}`, e);
        return null;
    }
}

function effectsFromLiveEffects(liveEffects) {
    const out = [];
    for (const fx of liveEffects || []) {
        const name = String(fx.name || fx.type || "").toLowerCase();
        const params = fx.params && fx.params.params ? fx.params.params : (fx.params || {});
        if (name.includes("noise") || name.includes("grain") || name.includes("mezzotint")) {
            out.push({
                type: "noise",
                amount: Number(params.amount ?? params.opacity ?? params.intensity ?? 0.16),
                scale: Number(params.scale ?? params.size ?? params.cellSize ?? 2),
                seed: Number(params.seed ?? 0)
            });
        } else if (name.includes("blur")) {
            out.push({ type: "gaussianBlur", radius: Number(params.radius ?? params.blur ?? 4) });
        } else {
            out.push({ type: "liveEffect", name: fx.name || fx.type || "liveEffect", params });
        }
    }
    return out;
}

function flattenAiParserElements(aiParserResult) {
    if (!aiParserResult) return [];
    if (Array.isArray(aiParserResult)) {
        return aiParserResult.flatMap(entry => Array.isArray(entry.elements) ? entry.elements : []);
    }
    return Array.isArray(aiParserResult.elements) ? aiParserResult.elements : [];
}

function typeCompatible(domType, parserType) {
    if (!domType || !parserType) return true;
    if (domType === parserType) return true;
    if (domType === "shape" && (parserType === "path" || parserType === "transform")) return true;
    if (domType === "path" && parserType === "shape") return true;
    return false;
}

function boundsDistance(domElement, parserElement) {
    const bounds = parserElement.bounds;
    if (!bounds || bounds.length < 4) return Number.POSITIVE_INFINITY;
    const [x, y, w, h] = bounds.map(Number);
    const dx = Math.abs((domElement.x || 0) - x);
    const dy = Math.abs((domElement.y || 0) - y);
    const dw = Math.abs((domElement.w || 0) - w);
    const dh = Math.abs((domElement.h || 0) - h);
    return dx + dy + dw + dh;
}

function findAiParserMatch(domElement, parserElements, usedIds) {
    const exactId = parserElements.find(el => !usedIds.has(el.id) && el.id && el.id === domElement.id);
    if (exactId) return exactId;

    const exactName = parserElements.find(el => !usedIds.has(el.id) && el.name && el.name === domElement.id);
    if (exactName) return exactName;

    let best = null;
    let bestScore = Number.POSITIVE_INFINITY;
    for (const candidate of parserElements) {
        if (candidate.id && usedIds.has(candidate.id)) continue;
        if (candidate.is_pseudo_element) continue;
        if (!typeCompatible(domElement.type, candidate.element_type)) continue;
        const score = boundsDistance(domElement, candidate);
        const tolerance = Math.max(12, (domElement.w || 0) * 0.08 + (domElement.h || 0) * 0.08);
        if (score < bestScore && score <= tolerance) {
            best = candidate;
            bestScore = score;
        }
    }
    return best;
}

function parserPathPoints(aiEl) {
    if (!Array.isArray(aiEl.path_points) || aiEl.path_points.length === 0) return undefined;
    return aiEl.path_points.map(point => ({
        anchor: point.anchor,
        leftDir: point.leftDir || point.left_ctrl || point.left || point.anchor,
        rightDir: point.rightDir || point.right_ctrl || point.right || point.anchor,
        left_ctrl: point.leftDir || point.left_ctrl || point.left || point.anchor,
        right_ctrl: point.rightDir || point.right_ctrl || point.right || point.anchor,
        kind: point.kind || "corner"
    }));
}

function mergeAiParserData(domElements, aiParserResult) {
    const parserElements = flattenAiParserElements(aiParserResult);
    if (!parserElements.length) return domElements;
    const usedIds = new Set();

    const mergeElement = (el) => {
        const children = (el.children || []).map(mergeElement);
        const base = { ...el, children };
        const aiEl = findAiParserMatch(base, parserElements, usedIds);
        if (!aiEl) return base;
        if (aiEl.id) usedIds.add(aiEl.id);

        const liveEffects = effectsFromLiveEffects(aiEl.live_effects || []);
        const pathPoints = parserPathPoints(aiEl);
        return {
            ...base,
            parserId: aiEl.id,
            artboardName: aiEl.artboard_name || base.artboardName,
            rotation: Number.isFinite(Number(aiEl.rotation_deg)) && Number(aiEl.rotation_deg) !== 0 ? Number(aiEl.rotation_deg) : base.rotation,
            scaleX: Number.isFinite(Number(aiEl.scale_x)) && Number(aiEl.scale_x) !== 0 ? Number(aiEl.scale_x) : base.scaleX,
            scaleY: Number.isFinite(Number(aiEl.scale_y)) && Number(aiEl.scale_y) !== 0 ? Number(aiEl.scale_y) : base.scaleY,
            translateX: Number.isFinite(Number(aiEl.translate_x)) ? Number(aiEl.translate_x) : base.translateX,
            translateY: Number.isFinite(Number(aiEl.translate_y)) ? Number(aiEl.translate_y) : base.translateY,
            cornerRadius: Number(aiEl.corner_radius || 0) > 0 ? Number(aiEl.corner_radius) : base.cornerRadius,
            pathPoints: pathPoints || base.pathPoints,
            pathClosed: aiEl.path_closed !== undefined ? !!aiEl.path_closed : base.pathClosed,
            live_effects: aiEl.live_effects?.length ? aiEl.live_effects : undefined,
            effects: liveEffects.length ? [...(base.effects || []), ...liveEffects] : base.effects,
            appearance_fills: aiEl.appearance_fills?.length ? aiEl.appearance_fills : base.appearance_fills,
            appearance_strokes: aiEl.appearance_strokes?.length ? aiEl.appearance_strokes : base.appearance_strokes,
            mesh_patches: aiEl.mesh_patches?.length ? aiEl.mesh_patches : base.mesh_patches,
            envelope_mesh: aiEl.envelope_mesh || base.envelope_mesh,
            three_d: aiEl.three_d || base.three_d,
        };
    };

    return domElements.map(mergeElement);
}

async function extractFromProjectFile(artboardsData, documentPath) {
    try {
        const app = getIllustratorApp();
        const doc = app && app.activeDocument ? app.activeDocument : null;
        const docPath = documentPath || doc?.fullName?.fsName || (doc && doc.path && doc.name ? doc.path + '/' + doc.name : null);
        if (!docPath) {
            noteAiParserDiagnostic("Project file analysis skipped", "No document path available from Illustrator");
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
        noteAiParserDiagnostic("Project file analysis failed", e);
        return artboardsData;
    }
}

function isTopLevelItem(item) {
  try {
    const parentType = item.parent ? item.parent.typename : null;
    return parentType === 'Layer' || parentType === 'Document' || parentType === null;
  } catch(e) { return true; }
}

// ─── Main Export ─────────────────────────────────────────────────────────────
async function exportArtboards(selectedIndices, options, selectedTiles) {
  const app = getIllustratorApp();
  if (!app) throw new Error("Illustrator app not available");
  const doc = app.activeDocument;
  if (!doc) throw new Error("No active document");

  const allEls = [], results = [];

  for (const idx of selectedIndices) {
    const ab = doc.artboards[idx], rect = ab.artboardRect;
    const abInfo = { name: ab.name, index: idx, width: Math.abs(rect[2] - rect[0]), height: Math.abs(rect[3] - rect[1]), x: rect[0], y: rect[1], bounds: [rect[0], rect[1], rect[2], rect[3]] };
    const items = [];
    try { for (let i = 0; i < doc.pageItems.length; i++) { const it = doc.pageItems[i]; try { if (it.locked || it.hidden) continue; const b = it.geometricBounds; if (b[2] > rect[0] && b[0] < rect[2] && b[1] > rect[3] && b[3] < rect[1] && isTopLevelItem(it)) items.push(it); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); } } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
    const els = extractElements(items, rect);
    allEls.push(...els);
    results.push({ artboard: abInfo, elements: els });
  }

  if (selectedTiles && selectedTiles.length > 0) {
    for (const tile of selectedTiles) {
      const rect = [tile.x, tile.y, tile.x + tile.width, tile.y - tile.height];
      const abInfo = { name: tile.name, width: tile.width, height: tile.height, x: tile.x, y: tile.y, bounds: [rect[0], rect[1], rect[2], rect[3]] };
      const items = [];
      try { for (let i = 0; i < doc.pageItems.length; i++) { const it = doc.pageItems[i]; try { if (it.locked || it.hidden) continue; const b = it.geometricBounds; if (b[2] > rect[0] && b[0] < rect[2] && b[1] > rect[3] && b[3] < rect[1] && isTopLevelItem(it)) items.push(it); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); } } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
      const els = extractElements(items, rect);
      allEls.push(...els);
      results.push({ artboard: abInfo, elements: els });
    }
  }

  // Try to enrich with project file data from the bundled ai-parser.
  await extractFromProjectFile(results);

  // Re-collect all elements after potential enrichment
  allEls.length = 0;
  for (const r of results) allEls.push(...r.elements);

  const { colorMap, constants } = extractAndNameColors(allEls, options?.naming);
  const comps = findReusableComponents(allEls);
  const files = {};

  files["mod.rs"] = generateModFile(results);
  files["tokens.rs"] = generateTokensFile(constants);
  files["state.rs"] = generateStateFile(results);
  files["components.rs"] = generateComponentsFile(comps, colorMap);

  for (const r of results) {
    const sn = toSnakeName(r.artboard.name), st = toStructName(r.artboard.name);
    files[`${sn}.rs`] = generateArtboardFile(r.artboard, r.elements, colorMap, st, comps, options);
    if (options?.sidecar || options?.includeSidecar) files[`${sn}.json`] = generateSidecar(r.artboard, r.elements, colorMap);
  }

  const assets = {};
  const walkAssets = (els) => {
    for (const el of els) {
      if (el.type === "image" && el.imagePath) {
        const portable = portableAssetPath(el.imagePath);
        if (portable) assets[portable] = el.imagePath;
      }
      if (el.children) walkAssets(el.children);
    }
  };
  walkAssets(allEls);

  let zipBlob = null;
  if (typeof JSZip !== "undefined") { const zip = new JSZip(); for (const [fn, ct] of Object.entries(files)) zip.file(fn, ct); zipBlob = await zip.generateAsync({ type: "blob" }); }
  return { files, assets, zipBlob, colorMap: Object.fromEntries(colorMap), warnings: collectWarnings(allEls, options) };
}

function exportFromRawData(results, options) {
  const allEls = [];
  for (const r of results) allEls.push(...r.elements);

  const { colorMap, constants } = extractAndNameColors(allEls, options?.naming);
  const comps = findReusableComponents(allEls);
  const files = {};

  files["mod.rs"] = generateModFile(results);
  files["tokens.rs"] = generateTokensFile(constants);
  files["state.rs"] = generateStateFile(results);
  files["components.rs"] = generateComponentsFile(comps, colorMap);

  for (const r of results) {
    const sn = toSnakeName(r.artboard.name), st = toStructName(r.artboard.name);
    files[`${sn}.rs`] = generateArtboardFile(r.artboard, r.elements, colorMap, st, comps, options);
    if (options?.sidecar || options?.includeSidecar) files[`${sn}.json`] = generateSidecar(r.artboard, r.elements, colorMap);
  }

  const assets = {};
  const walkAssets = (els) => {
    for (const el of els) {
      if (el.type === "image" && el.imagePath) {
        const portable = portableAssetPath(el.imagePath);
        if (portable) assets[portable] = el.imagePath;
      }
      if (el.children) walkAssets(el.children);
    }
  };
  walkAssets(allEls);

  return { files, assets, colorMap: Object.fromEntries(colorMap), warnings: collectWarnings(allEls, options) };
}

function collectWarnings(elements, options) {
  const warnings = [];
  if (options && Array.isArray(options.parserDiagnostics)) warnings.push(...options.parserDiagnostics);
  warnings.push(...getAiParserDiagnostics());
  warnings.push(...consumeExtractionDiagnostics());
  const walk = (els) => { for (const el of els) {
    if (el.isGradientMesh) warnings.push({ id: el.id, note: "Gradient mesh — emitted as mesh primitive when patches are available, otherwise bounded editable primitive" });
    if (el.isChart) warnings.push({ id: el.id, note: "Chart/graph — emitted as bounded editable vector primitive with preserved chart metadata" });
    if (el.type === "image" && el.embeddedRaster) warnings.push({ id: el.id, note: "Embedded raster — emitted as an image asset slot with embedded-raster metadata" });
    else if (el.type === "image" && !el.imagePath) warnings.push({ id: el.id, note: "Image element has no linked file path; generated code emits an editable image asset slot" });
    if (el.type === "image" && el.imagePath) warnings.push({ id: el.id, note: `Linked image asset reference: ${portableAssetPath(el.imagePath) || basename(el.imagePath)}` });
    if (el.clipMask) warnings.push({ id: el.id, note: "Clipping mask — emitted through shape/stencil clipping primitive metadata" });
    if (el.blendMode !== "normal") warnings.push({ id: el.id, note: `Blend mode ${el.blendMode} — emitted through compositing primitive metadata` });
    if (el.thirdPartyEffects?.length > 0) for (const fx of el.thirdPartyEffects) warnings.push({ id: el.id, note: fx.note });
    if (el.children) walk(el.children);
  } };
  walk(elements);
  const seen = new Set();
  return warnings.filter(w => {
    const key = `${w.id || ''}|${w.note || w.message || String(w)}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

// ─── Message Handler (UXP mode) ──────────────────────────────────────────
if (typeof window !== 'undefined' && window.addEventListener) {
  window.addEventListener("message", async (event) => {
    if (!isTrustedPanelMessage(event)) return;
    const { type, payload } = event.data;
  if (type === "GET_ARTBOARDS") {
    try {
      const app = getIllustratorApp();
      if (!app) {
        postPanelMessage({ type: "ERROR", message: "Not running inside Illustrator. Install the plugin via the .zxp installer." });
      } else if (app.documents.length === 0) {
        postPanelMessage({ type: "ERROR", message: "No document open in Illustrator. Please open an .ai file first." });
      } else {
        const result = await getArtboards();
        if (result && result.error) {
          postPanelMessage({ type: "ERROR", message: result.error });
        } else {
          postPanelMessage({ type: "ARTBOARDS_RESULT", artboards: result });
        }
      }
    } catch (e) {
      postPanelMessage({ type: "ERROR", message: e.message });
    }
  }
  if (type === "CHECK_AI_PARSER") { checkAiParserAvailable(); postPanelMessage({ type: "AI_PARSER_STATUS", status: getAiParserStatus(), available: getAiParserStatus().available }); }
  if (type === "EXPORT") { try { const ed = event.data; const selectedIndices = ed.selectedIndices || ed.artboardIndices; const selectedTiles = ed.selectedTiles || []; const options = ed.options || {}; const r = await exportArtboards(selectedIndices || [], options, selectedTiles); postPanelMessage({ type: "EXPORT_RESULT", payload: { files: r.files, filesArray: Object.entries(r.files || {}).map(([filename, content]) => ({filename, content})), colorMap: r.colorMap, zipBlob: r.zipBlob, warnings: r.warnings || [] } }); } catch (e) { postPanelMessage({ type: "ERROR", message: e.message }); } }
  if (type === "EXPORT_SINGLE") { try { const ed = event.data; const artboardIndex = ed.artboardIndex; const selectedTiles = ed.selectedTiles || []; const options = ed.options || {}; const r = await exportArtboards([artboardIndex], options, selectedTiles); postPanelMessage({ type: "EXPORT_RESULT", payload: { files: r.files, filesArray: Object.entries(r.files || {}).map(([filename, content]) => ({filename, content})), colorMap: r.colorMap, zipBlob: r.zipBlob, warnings: r.warnings || [] } }); } catch (e) { postPanelMessage({ type: "ERROR", message: e.message }); } }
    if (type === "EXPAND_AND_EXTRACT") {
      try {
        const { artboardIndex, options } = payload || {};
        // Export artboard directly (appearance expansion requires Illustrator's Object > Expand Appearance)
        const r = await exportArtboards([artboardIndex], options || {}, payload.selectedTiles || []);
        postPanelMessage({ type: "EXPORT_RESULT", payload: { files: r.files, filesArray: Object.entries(r.files || {}).map(([filename, content]) => ({filename, content})), colorMap: r.colorMap, warnings: r.warnings || [] } });
      } catch (e) { postPanelMessage({ type: "ERROR", message: e.message }); }
    }
  });

  postPanelMessage({ type: "READY" });
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    EGUI_EXPORT_CHANNEL,
    portableAssetPath,
    getAiParserCandidates,
    getAiParserPlatformDir,
    mergeAiParserData,
    collectWarnings,
    isTrustedPanelMessage,
    getLocalTargetOrigin,
    applyBlendExpr,
    getGradient,
    generateSidecar,
    exportFromRawData,
  };
}

// ─── CEP ExtendScript Entry Points ──────────────────────────────────────
// These functions are called from index.html via CSInterface.evalScript()
