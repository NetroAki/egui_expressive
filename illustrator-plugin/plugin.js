// egui_expressive Illustrator Exporter — CEP Extension for Adobe Illustrator 2022+
"use strict";

var EGUI_EXPORT_CHANNEL = "egui_expressive_exporter";
var AI_PARSER_MAX_BUFFER_BYTES = 64 * 1024 * 1024;

let extractionDiagnostics = [];
let aiParserStatus = {
  checked: false,
  available: false,
  binaryPath: null,
  diagnostics: []
};
let unscopedAiParserDiagnosticKeys = new Set();

function rasterImageUnsupportedReason(el) {
  const origin = el && el.embeddedRaster ? "Embedded" : "Linked";
  if (el && el.vectorizationFailed && el.embeddedRaster) return "Embedded raster/images could not be vectorized and will not be exported as raster";
  if (el && el.vectorizationFailed && (el.imagePath || el.extractedImagePath || el.vectorSourcePath || el.sourcePath)) return "Linked raster/images could not be vectorized and will not be exported as raster";
  if (el && rasterVectorSourcePath(el) && rasterHasUnsafeVectorEffects(el)) return `${origin} raster/images with Illustrator effects need effect-aware vector tracing and will not be exported as raster`;
  if (el && rasterVectorSourcePath(el) && Math.abs(Number(el.rotation || 0)) > 0.001) {
    return hasRasterTransformScale(el)
      ? `Rotated ${origin.toLowerCase()} raster/images with transform scale metadata need transform-aware vector tracing and will not be exported as raster`
      : `Rotated ${origin.toLowerCase()} raster/images without transform scale metadata need matrix-aware vector tracing and will not be exported as raster`;
  }
  if (el && el.embeddedRaster && el.extractedImagePath) return "Embedded raster/images could not be vectorized and will not be exported as raster";
  if (el && el.embeddedRaster) return "Embedded raster/images need extractable pixels for vector tracing and will not be exported as raster";
  if (el && el.imagePath) return "Linked raster/images could not be vectorized and will not be exported as raster";
  return "Raster/image elements need vector tracing and will not be exported as raster";
}

function rasterEffectTypeFromName(value) {
  const name = String(value || "").toLowerCase().replace(/[\s_-]+/g, "");
  if (name.includes("dropshadow")) return "dropShadow";
  if (name.includes("innershadow")) return "innerShadow";
  if (name.includes("outerglow")) return "outerGlow";
  if (name.includes("innerglow")) return "innerGlow";
  if (name.includes("gaussianblur")) return "gaussianBlur";
  if (name.includes("feather")) return "feather";
  if (name.includes("bevel")) return "bevel";
  if (name.includes("noise") || name.includes("grain") || name.includes("mezzotint")) return "noise";
  return null;
}

function classifyLiveEffectCategory(value) {
  const name = String(value || "").toLowerCase().replace(/[\s_-]+/g, "");
  if (!name) return "other";
  if (name.includes("motionblur") || name.includes("radialblur") || name.includes("zoomblur") || name.includes("blur")) return "blur_variant";
  if (name.includes("twist") || name.includes("pucker") || name.includes("bloat") || name.includes("roughen") || name.includes("zigzag") || name.includes("warp") || name.includes("distort") || name.includes("free distort") || name.includes("transform")) return "distort";
  if (name.includes("scribble") || name.includes("roundcorners") || name.includes("glow") || name.includes("shadow") || name.includes("stylize")) return "stylize";
  if (name.includes("texture") || name.includes("grain") || name.includes("mosaic") || name.includes("crystallize") || name.includes("mezzotint") || name.includes("plasticwrap")) return "texture";
  if (name.includes("3d") || name.includes("extrude") || name.includes("revolve") || name.includes("inflate") || name.includes("bevel")) return "3d";
  if (name.includes("plugin") || name.includes("thirdparty") || name.includes("third-party") || name.includes("aifx")) return "plugin";
  return "other";
}

function liveEffectDisplayName(effect) {
  return String(effect && (effect.name || effect.effectName || effect.effect_name || effect.type || effect.effectType || effect.effect_type) || "liveEffect");
}

function liveEffectGuidance(effect) {
  const name = liveEffectDisplayName(effect);
  const category = effect && effect.category ? String(effect.category) : classifyLiveEffectCategory(name);
  return `unsupported live effect '${name}' (${category}) requires expanded vector geometry — exporter first tries duplicate + Expand Appearance fallback; if Illustrator menu commands are unavailable, use Object > Expand Appearance in Illustrator, or enable ai-parser recovery if expanded vectors are available`;
}

function expansionFallbackFailureNote(el) {
  const notes = Array.isArray(el && el.notes) ? el.notes : [];
  return notes.find(note => /duplicate \+ Expand Appearance fallback (?:unavailable|failed)/i.test(String(note)) || /Expand Appearance fallback (?:unavailable|failed)/i.test(String(note))) || null;
}

function ensureElementNotes(el) {
  if (!el) return [];
  if (!Array.isArray(el.notes)) el.notes = [];
  return el.notes;
}

function rasterEffectMetadataHasUnmappedEffect(text, mappedEffects) {
  const raw = String(text || "");
  if (!raw) return false;
  const lower = raw.toLowerCase();
  const hasMapped = Array.isArray(mappedEffects) && mappedEffects.length > 0;
  if (!/(effect|filter|liveeffect|aifx)/i.test(raw)) return false;
  if (!hasMapped) return true;
  const namePattern = /(?:effect|filter|liveeffect|aifx)\s*(?:name|type)?\s*[:=]\s*["']?([A-Za-z][A-Za-z0-9 _-]{1,80})/gi;
  let match;
  while ((match = namePattern.exec(raw)) !== null) {
    if (!rasterEffectTypeFromName(match[1])) return true;
  }
  return false;
}

function defaultRasterEffect(type) {
  if (type === "dropShadow") return { type, x: 4, y: 4, blur: 8, color: { r: 0, g: 0, b: 0, a: 0.3 } };
  if (type === "innerShadow") return { type, x: 0, y: 0, blur: 4, color: { r: 0, g: 0, b: 0, a: 0.35 } };
  if (type === "outerGlow" || type === "innerGlow") return { type, blur: 6, color: { r: 255, g: 255, b: 255, a: 0.45 } };
  if (type === "gaussianBlur" || type === "feather") return { type, radius: 4, blur: 4 };
  if (type === "bevel") return { type, depth: 2, angle: 135, radius: 1, highlight: { r: 255, g: 255, b: 255, a: 0.55 }, shadowColor: { r: 0, g: 0, b: 0, a: 0.6 } };
  if (type === "noise") return { type, amount: 0.16, scale: 2, seed: 0 };
  return { type };
}

function mergeEffectByType(effects, effect) {
  if (!effect || !effect.type) return;
  if (!effects.some(existing => String(existing.type || existing.effectType || existing.effect_type) === String(effect.type))) effects.push(effect);
}

function effectsFromMetadataText(text) {
  const effects = [];
  const raw = String(text || "");
  for (const token of ["dropShadow", "innerShadow", "outerGlow", "innerGlow", "gaussianBlur", "feather", "bevel", "noise", "grain", "mezzotint"]) {
    if (raw.toLowerCase().includes(token.toLowerCase())) mergeEffectByType(effects, defaultRasterEffect(rasterEffectTypeFromName(token)));
  }
  if (rasterEffectMetadataHasUnmappedEffect(text, effects)) mergeEffectByType(effects, { type: "unknown", source: "xmp" });
  return effects;
}

function maxCountFromRegex(raw, regex) {
  let max = 0;
  let match;
  while ((match = regex.exec(raw)) !== null) {
    const value = Number(match[1]);
    if (Number.isFinite(value)) max = Math.max(max, value);
  }
  return max;
}

function appearanceProbeFromMetadataText(text, source) {
  const raw = String(text || "");
  if (!raw) return null;
  const explicitFillCount = Math.max(
    maxCountFromRegex(raw, /(?:appearance[-_\s]*)?fills?[-_\s]*count\s*[:=]\s*["']?(\d+)/gi),
    maxCountFromRegex(raw, /(?:appearance[-_\s]*)?fillcount\s*[:=]\s*["']?(\d+)/gi)
  );
  const explicitStrokeCount = Math.max(
    maxCountFromRegex(raw, /(?:appearance[-_\s]*)?strokes?[-_\s]*count\s*[:=]\s*["']?(\d+)/gi),
    maxCountFromRegex(raw, /(?:appearance[-_\s]*)?strokecount\s*[:=]\s*["']?(\d+)/gi)
  );

  let fillOps = 0;
  let strokeOps = 0;
  const arrayPaintRe = /\[\s*-?\d*\.?\d+\s+-?\d*\.?\d+\s+-?\d*\.?\d+\s+-?\d*\.?\d+\s*\]\s*([Xx][aA])\b/g;
  let opMatch;
  while ((opMatch = arrayPaintRe.exec(raw)) !== null) {
    if (String(opMatch[1]).charAt(0) === "X") fillOps++;
    else strokeOps++;
  }

  const fillCount = Math.max(explicitFillCount, fillOps);
  const strokeCount = Math.max(explicitStrokeCount, strokeOps);
  if (fillCount <= 1 && strokeCount <= 1) return null;
  return {
    fillCount,
    strokeCount,
    source: source || "metadata"
  };
}

function maxAppearanceProbe(a, b) {
  if (!a) return b || null;
  if (!b) return a || null;
  return {
    fillCount: Math.max(Number(a.fillCount || 0), Number(b.fillCount || 0)),
    strokeCount: Math.max(Number(a.strokeCount || 0), Number(b.strokeCount || 0)),
    source: [a.source, b.source].filter(Boolean).join("+")
  };
}

function extractAppearanceProbe(item) {
  let probe = null;
  try { probe = maxAppearanceProbe(probe, appearanceProbeFromMetadataText(item && item.XMPString, "XMPString")); } catch (e) { noteExtractionDiagnostic("optional Illustrator appearance metadata unavailable", e); }
  try { probe = maxAppearanceProbe(probe, appearanceProbeFromMetadataText(item && item.note, "note")); } catch (e) { noteExtractionDiagnostic("optional Illustrator appearance note unavailable", e); }
  try {
    if (item && item.tags) {
      for (let i = 0; i < item.tags.length; i++) {
        const tag = item.tags[i];
        probe = maxAppearanceProbe(probe, appearanceProbeFromMetadataText(`${tag.name || ""}=${tag.value || ""}`, "tag"));
      }
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator appearance tags unavailable", e); }
  return probe;
}

function diagnosticMessage(error) {
  if (!error) return "unknown error";
  return sanitizeDiagnosticText(error.message || String(error));
}

function sanitizeDiagnosticText(value) {
  return String(value || "")
    .replace(/[A-Za-z]:[\\/][^\n\r;)]*egui_expressive_raster_trace[\\/][^\s;),]+/g, "[temporary raster extraction input]")
    .replace(/(?:\/|\\\\)[^\n\r;)]*egui_expressive_raster_trace[\\/][^\s;),]+/g, "[temporary raster extraction input]")
    .replace(/egui_expressive_raster_trace[\\/][^\s;),]+/g, "egui_expressive_raster_trace/[temporary input]");
}

function sanitizeDiagnosticEntry(entry) {
  if (!entry || typeof entry !== "object") return { id: "diagnostic", note: sanitizeDiagnosticText(entry) };
  const out = { ...entry };
  if (out.note !== undefined) out.note = sanitizeDiagnosticText(out.note);
  if (out.message !== undefined) out.message = sanitizeDiagnosticText(out.message);
  if (out.error !== undefined) out.error = sanitizeDiagnosticText(out.error);
  return out;
}

function noteExtractionDiagnostic(context, error) {
  if (extractionDiagnostics.length >= 200) return;
  extractionDiagnostics.push({
    id: "exporter",
    note: sanitizeDiagnosticText(`${context}: ${diagnosticMessage(error)}`)
  });
}

function consumeExtractionDiagnostics() {
  const out = extractionDiagnostics.slice();
  extractionDiagnostics = [];
  return out;
}

function safeTempFileSlug(value) {
  const raw = String(value || "embedded_raster");
  const slug = raw.replace(/[^A-Za-z0-9._-]/g, "_").replace(/^_+|_+$/g, "");
  return slug || "embedded_raster";
}

function rasterExtractionTempFolder() {
  if (typeof Folder === "undefined") return null;
  try {
    const base = Folder.temp || Folder.desktop || Folder.myDocuments;
    if (!base || !base.fsName) return null;
    const folder = new Folder(`${base.fsName}/egui_expressive_raster_trace`);
    if (!folder.exists && typeof folder.create === "function") folder.create();
    return folder.exists ? folder : null;
  } catch (e) {
    noteExtractionDiagnostic("embedded raster temp folder unavailable", e);
    return null;
  }
}

function closeTempDocumentWithoutSaving(doc) {
  if (!doc || typeof doc.close !== "function") return;
  try {
    if (typeof SaveOptions !== "undefined" && SaveOptions.DONOTSAVECHANGES !== undefined) doc.close(SaveOptions.DONOTSAVECHANGES);
    else doc.close();
  } catch (e) { noteExtractionDiagnostic("embedded raster temp document close failed", e); }
}

function extractEmbeddedRasterToTempPng(item, el) {
  if (!item || item.typename !== "RasterItem") return null;
  if (typeof File === "undefined" || typeof Folder === "undefined") return null;
  if (typeof app === "undefined" || !app.documents || typeof ExportOptionsPNG24 === "undefined" || typeof ExportType === "undefined") {
    noteExtractionDiagnostic("embedded raster extraction unavailable", "Illustrator export APIs unavailable");
    return null;
  }

  const folder = rasterExtractionTempFolder();
  if (!folder) return null;

  const width = Math.max(1, Math.ceil(Number(el && el.w || 1)));
  const height = Math.max(1, Math.ceil(Number(el && el.h || 1)));
  const stamp = Date.now ? Date.now() : Math.floor(Math.random() * 1000000);
  const file = new File(`${folder.fsName}/${safeTempFileSlug(el && el.id)}_${stamp}.png`);
  let tempDoc = null;

  try {
    try {
      if (typeof DocumentColorSpace !== "undefined" && DocumentColorSpace.RGB !== undefined) tempDoc = app.documents.add(DocumentColorSpace.RGB, width, height);
    } catch (e) { noteExtractionDiagnostic("embedded raster temp document color setup failed", e); }
    if (!tempDoc) tempDoc = app.documents.add();

    try {
      if (tempDoc.artboards && tempDoc.artboards.length > 0) tempDoc.artboards[0].artboardRect = [0, height, width, 0];
    } catch (e) { noteExtractionDiagnostic("embedded raster temp artboard setup failed", e); }

    const target = tempDoc.layers && tempDoc.layers.length > 0 ? tempDoc.layers[0] : tempDoc;
    const duplicate = (typeof ElementPlacement !== "undefined" && ElementPlacement.PLACEATEND !== undefined)
      ? item.duplicate(target, ElementPlacement.PLACEATEND)
      : item.duplicate(target);
    try {
      const b = duplicate.geometricBounds || duplicate.visibleBounds;
      if (b && typeof duplicate.translate === "function") duplicate.translate(-Number(b[0] || 0), height - Number(b[1] || height));
    } catch (e) { noteExtractionDiagnostic("embedded raster temp positioning failed", e); }

    const options = new ExportOptionsPNG24();
    options.antiAliasing = true;
    options.transparency = true;
    options.artBoardClipping = true;
    options.horizontalScale = 100;
    options.verticalScale = 100;
    tempDoc.exportFile(file, ExportType.PNG24, options);
    closeTempDocumentWithoutSaving(tempDoc);
    tempDoc = null;
    if (file.exists) return file.fsName || String(file);
  } catch (e) {
    noteExtractionDiagnostic("embedded raster extraction failed", e);
  } finally {
    closeTempDocumentWithoutSaving(tempDoc);
  }

  return null;
}

function extractRasterTransformScale(item) {
  try {
    const matrix = item && item.matrix;
    if (!matrix) return null;
    const a = Number(matrix.mValueA ?? matrix.a ?? matrix.A ?? 1);
    const b = Number(matrix.mValueB ?? matrix.b ?? matrix.B ?? 0);
    const c = Number(matrix.mValueC ?? matrix.c ?? matrix.C ?? 0);
    const d = Number(matrix.mValueD ?? matrix.d ?? matrix.D ?? 1);
    const scaleX = Math.sqrt(a * a + b * b);
    const scaleY = Math.sqrt(c * c + d * d);
    if (Number.isFinite(scaleX) && scaleX > 0 && Number.isFinite(scaleY) && scaleY > 0) return { scaleX, scaleY };
  } catch (e) { noteExtractionDiagnostic("optional Illustrator raster transform unavailable", e); }
  return null;
}

function extractItemRotationDeg(item) {
  try {
    const direct = Number(item && item.rotation);
    if (Number.isFinite(direct) && Math.abs(direct) > 0.0001) return direct;
  } catch (e) { noteExtractionDiagnostic("optional Illustrator rotation unavailable", e); }
  try {
    const matrix = item && item.matrix;
    if (!matrix) return 0;
    const a = Number(matrix.mValueA ?? matrix.a ?? matrix.A ?? 1);
    const b = Number(matrix.mValueB ?? matrix.b ?? matrix.B ?? 0);
    if (!Number.isFinite(a) || !Number.isFinite(b)) return 0;
    const deg = Math.atan2(b, a) * 180 / Math.PI;
    return Number.isFinite(deg) ? deg : 0;
  } catch (e) { noteExtractionDiagnostic("optional Illustrator matrix rotation unavailable", e); }
  return 0;
}

function noteAiParserDiagnostic(context, error) {
  aiParserStatus.diagnostics.push({
    id: "ai-parser",
    note: sanitizeDiagnosticText(`${context}: ${diagnosticMessage(error)}`)
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

function pathPointKind(pathPoint) {
  try {
    if (typeof PointType !== "undefined" && pathPoint.pointType === PointType.SMOOTH) return "smooth";
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return String(pathPoint && pathPoint.pointType || "").toLowerCase().includes("smooth") ? "smooth" : "corner";
}

function mapIllustratorPathPoint(pathPoint, artboardRect) {
  return {
    anchor: [pathPoint.anchor[0] - artboardRect[0], artboardRect[1] - pathPoint.anchor[1]],
    leftDir: [pathPoint.leftDirection[0] - artboardRect[0], artboardRect[1] - pathPoint.leftDirection[1]],
    rightDir: [pathPoint.rightDirection[0] - artboardRect[0], artboardRect[1] - pathPoint.rightDirection[1]],
    left_ctrl: [pathPoint.leftDirection[0] - artboardRect[0], artboardRect[1] - pathPoint.leftDirection[1]],
    right_ctrl: [pathPoint.rightDirection[0] - artboardRect[0], artboardRect[1] - pathPoint.rightDirection[1]],
    kind: pathPointKind(pathPoint)
  };
}

function extractPathItemPoints(pathItem, artboardRect) {
  const points = [];
  try {
    if (!pathItem || !pathItem.pathPoints) return points;
    for (let pi = 0; pi < pathItem.pathPoints.length; pi++) {
      try { points.push(mapIllustratorPathPoint(pathItem.pathPoints[pi], artboardRect)); }
      catch (ppe) { noteExtractionDiagnostic("optional Illustrator property unavailable", ppe); }
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return points;
}

function extractPathSubpaths(item, artboardRect) {
  const subpaths = [];
  try {
    if (item && item.typename === "CompoundPathItem" && item.pathItems) {
      for (let si = 0; si < item.pathItems.length; si++) {
        const pathItem = item.pathItems[si];
        const points = extractPathItemPoints(pathItem, artboardRect);
        if (points.length > 0) subpaths.push({ points, closed: pathItem.closed !== false });
      }
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  if (subpaths.length === 0) {
    const points = extractPathItemPoints(item, artboardRect);
    if (points.length > 0) subpaths.push({ points, closed: item.closed || false });
  }

  return subpaths;
}

function illustratorFillRule(item) {
  // Illustrator scripting does not expose a reliable compound-path fill rule here.
  // Keep this explicit so strict export requires parser-side fill_rule metadata.
  return null;
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

  return effects;
}

// ─── Element Extraction ──────────────────────────────────────────────────────
function extractElements(pageItems, artboardRect) {
  const elements = [];
  for (const item of pageItems) extractRecursive(item, artboardRect, elements, 0);
  return elements;
}

function pageItemLike(value) {
  if (!value || typeof value !== "object") return false;
  try {
    const type = String(value.typename || "");
    return !!type && type !== "Symbol" && type !== "SymbolItem" && type !== "Document" && type !== "Layer";
  } catch (e) { noteExtractionDiagnostic("optional Illustrator symbol item typename unavailable", e); return false; }
}

function addSymbolDefinitionItems(candidates, container, source) {
  if (!container) return;
  let added = false;
  for (const property of ["pageItems", "pathItems", "compoundPathItems", "groupItems", "symbolItems"]) {
    const items = collectionToArray(safeReadProperty(container, property, `optional Illustrator ${source}.${property} unavailable`))
      .filter(pageItemLike);
    if (items.length > 0) {
      candidates.push({ items, source: `${source}.${property}` });
      added = true;
    }
  }
  if (!added && pageItemLike(container)) candidates.push({ items: [container], source });
}

function symbolDefinitionCandidates(symbolItem) {
  const candidates = [];
  const symbol = safeReadProperty(symbolItem, "symbol", "optional Illustrator symbol reference unavailable");
  const directDefinition = safeReadProperty(symbolItem, "definition", "optional Illustrator symbolItem.definition unavailable");
  addSymbolDefinitionItems(candidates, directDefinition, "symbolItem.definition");
  addSymbolDefinitionItems(candidates, safeReadProperty(symbolItem, "artwork", "optional Illustrator symbolItem.artwork unavailable"), "symbolItem.artwork");
  addSymbolDefinitionItems(candidates, symbolItem, "symbolItem");
  if (symbol) {
    addSymbolDefinitionItems(candidates, safeReadProperty(symbol, "definition", "optional Illustrator symbol.definition unavailable"), "symbol.definition");
    addSymbolDefinitionItems(candidates, safeReadProperty(symbol, "artwork", "optional Illustrator symbol.artwork unavailable"), "symbol.artwork");
    addSymbolDefinitionItems(candidates, safeReadProperty(symbol, "groupItem", "optional Illustrator symbol.groupItem unavailable"), "symbol.groupItem");
    addSymbolDefinitionItems(candidates, symbol, "symbol");
  }
  return candidates;
}

function elementTreeBounds(elements) {
  const points = [];
  const walk = el => {
    if (!el) return;
    points.push(...geometryBoundsTuples(el));
    for (const child of el.children || []) walk(child);
  };
  for (const el of elements || []) walk(el);
  return boundsFromTuples(points);
}

function mapTupleBetweenBounds(tuple, source, target) {
  const [x, y] = tupleFromPoint(tuple, [source.x, source.y]);
  const sx = source.w > 0.0001 ? target.w / source.w : 1;
  const sy = source.h > 0.0001 ? target.h / source.h : 1;
  return [target.x + (x - source.x) * sx, target.y + (y - source.y) * sy];
}

function mapPathPointBetweenBounds(point, source, target) {
  if (Array.isArray(point)) return mapTupleBetweenBounds(point, source, target);
  const anchor = mapTupleBetweenBounds(point.anchor, source, target);
  const leftDir = mapTupleBetweenBounds(point.leftDir || point.left_ctrl || point.leftCtrl || point.anchor, source, target);
  const rightDir = mapTupleBetweenBounds(point.rightDir || point.right_ctrl || point.rightCtrl || point.anchor, source, target);
  return { ...point, anchor, leftDir, rightDir, left_ctrl: leftDir, right_ctrl: rightDir };
}

function fitElementTreeToBounds(el, source, target) {
  const min = mapTupleBetweenBounds([el.x || 0, el.y || 0], source, target);
  const max = mapTupleBetweenBounds([Number(el.x || 0) + Number(el.w || 0), Number(el.y || 0) + Number(el.h || 0)], source, target);
  el.x = min[0];
  el.y = min[1];
  el.w = Math.abs(max[0] - min[0]);
  el.h = Math.abs(max[1] - min[1]);
  if (Array.isArray(el.pathPoints)) el.pathPoints = el.pathPoints.map(point => mapPathPointBetweenBounds(point, source, target));
  if (Array.isArray(el.subpaths)) {
    el.subpaths = el.subpaths.map(subpath => ({
      ...subpath,
      points: (subpath.points || []).map(point => mapPathPointBetweenBounds(point, source, target)),
    }));
    if (el.subpaths.length > 0) el.pathPoints = el.subpaths[0].points || el.pathPoints;
  }
  for (const child of el.children || []) fitElementTreeToBounds(child, source, target);
}

function prefixExpandedSymbolChildIds(children, prefix) {
  for (let i = 0; i < (children || []).length; i++) {
    const child = children[i];
    child.id = `${prefix}_${child.id || `child_${i}`}`;
    prefixExpandedSymbolChildIds(child.children || [], child.id);
  }
}

function rotateSymbolExpandedElement(el, center, degrees) {
  if (Array.isArray(el.pathPoints) || Array.isArray(el.subpaths)) return rotateVectorElementGeometry(el, center, degrees);
  const out = { ...el, rotation: Number(el.rotation || 0) + Number(degrees || 0) };
  const childCenter = [Number(el.x || 0) + Number(el.w || 0) / 2, Number(el.y || 0) + Number(el.h || 0) / 2];
  const rotatedCenter = rotateTuple(childCenter, center, degrees);
  out.x = rotatedCenter[0] - Number(el.w || 0) / 2;
  out.y = rotatedCenter[1] - Number(el.h || 0) / 2;
  if (Array.isArray(out.children)) out.children = out.children.map(child => rotateSymbolExpandedElement(child, center, degrees));
  return out;
}

function expandSymbolDefinitionIntoElement(item, artboardRect, el, depth) {
  if (depth > 32) {
    ensureElementNotes(el).push("symbol definition expansion depth limit reached; expand symbol before strict export");
    return false;
  }
  const candidates = symbolDefinitionCandidates(item);
  for (const candidate of candidates) {
    const children = [];
    for (const childItem of candidate.items) extractRecursive(childItem, artboardRect, children, depth + 1);
    if (children.length === 0) continue;
    const sourceBounds = elementTreeBounds(children);
    if (sourceBounds && sourceBounds.w > 0.0001 && sourceBounds.h > 0.0001) {
      const targetBounds = { x: el.x, y: el.y, w: el.w, h: el.h };
      for (const child of children) fitElementTreeToBounds(child, sourceBounds, targetBounds);
    }
    const rotation = Number(el.rotation || 0);
    if (Number.isFinite(rotation) && Math.abs(rotation) > 0.0001) {
      const center = { x: Number(el.x || 0) + Number(el.w || 0) / 2, y: Number(el.y || 0) + Number(el.h || 0) / 2 };
      for (let i = 0; i < children.length; i++) children[i] = rotateSymbolExpandedElement(children[i], center, rotation);
    }
    el.rotation = 0;
    prefixExpandedSymbolChildIds(children, el.id || "symbol");
    el.children = children;
    el.symbolExpanded = true;
    el.symbolExpansionSource = candidate.source;
    ensureElementNotes(el).push(`symbol definition expanded from ${candidate.source}; instance transform fitted to symbol bounds`);
    return true;
  }
  return false;
}

function extractRecursive(item, artboardRect, elements, depth) {
  try { if (item.locked || item.hidden) return; } catch (e) { noteExtractionDiagnostic("skip hidden/locked state error", e); return; }

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
    } catch (e2) { noteExtractionDiagnostic("skip bounds error", e2); return; }
  }

  const el = {
    id: item.name || `el_${elements.length}`, type: getElementType(item), x, y, w, h, depth,
    fill: getFill(item), stroke: getStroke(item, artboardRect), text: null, textStyle: null, children: [],
    opacity: 1.0, rotation: 0, cornerRadius: 0, gradient: null, blendMode: "normal",
    strokeCap: null, strokeJoin: null, strokeDash: null, strokeMiterLimit: null, strokeAlignment: null,
    effects: [], textDecoration: null, textTransform: null, textRuns: null,
    textAlign: null, letterSpacing: null, lineHeight: null, openTypeFeatures: null,
    baselineShift: null, horizontalScale: null, verticalScale: null, clipMask: false,
    symbolName: null, isCompoundPath: false, isGradientMesh: false, isChart: false, notes: [],
    pathPoints: null, pathClosed: false, subpaths: null, fillRule: "nonzero",
    appearanceProbe: null,
    imagePath: null, extractedImagePath: null, extractedRasterAlreadyTransformed: false,
    rasterScaleX: null, rasterScaleY: null, embeddedRaster: false
  };

  // Path geometry extraction
  try {
    if (item.typename === "PathItem" || item.typename === "CompoundPathItem") {
      const subpaths = extractPathSubpaths(item, artboardRect);
      if (subpaths.length > 0) {
        el.subpaths = subpaths;
        el.pathPoints = subpaths[0].points;
        el.pathClosed = subpaths[0].closed;
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
    if (item.typename === "PlacedItem" || item.typename === "RasterItem") {
      const transformScale = extractRasterTransformScale(item);
      if (transformScale) {
        el.rasterScaleX = transformScale.scaleX;
        el.rasterScaleY = transformScale.scaleY;
      }
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator raster transform unavailable", e); }
  try {
    if (item.typename === "RasterItem") {
      el.embeddedRaster = true;
      ensureElementNotes(el).push("embedded raster image");
      const extractedImagePath = extractEmbeddedRasterToTempPng(item, el);
      if (extractedImagePath) {
        el.extractedImagePath = extractedImagePath;
        el.extractedRasterAlreadyTransformed = true;
        ensureElementNotes(el).push("embedded raster extracted for vector tracing");
      }
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  try { el.opacity = item.opacity !== undefined ? item.opacity / 100 : 1; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  el.rotation = extractItemRotationDeg(item);
  try { if (item.typename === "PathItem" && item.cornerRadius !== undefined) el.cornerRadius = item.cornerRadius; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  // Stroke details
  try { if (item.strokeCap !== undefined) el.strokeCap = { 0: "butt", 1: "round", 2: "square" }[item.strokeCap] || "butt"; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.strokeJoin !== undefined) el.strokeJoin = { 0: "miter", 1: "round", 2: "bevel" }[item.strokeJoin] || "miter"; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.strokeDashes?.length > 0) el.strokeDash = [...item.strokeDashes]; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.strokeMiterLimit !== undefined) el.strokeMiterLimit = item.strokeMiterLimit; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.strokeAlignment !== undefined) el.strokeAlignment = normalizeStrokeAlignment(item.strokeAlignment); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  // Blend mode
  try {
    if (item.blendingMode !== undefined) {
      el.blendMode = normalizeBlendModeValue(item.blendingMode) || "normal";
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  el.gradient = getGradient(item, artboardRect);
  el.appearanceProbe = extractAppearanceProbe(item);

  if (item.typename === "TextFrame") {
    try { el.text = item.contents || ""; } catch (e) { el.text = ""; }
    el.textStyle = getTextStyle(item);
    el.textAlign = getTextAlign(item);
    el.letterSpacing = getLetterSpacing(item);
    el.lineHeight = getLineHeight(item);
    el.textDecoration = getTextDecoration(item);
    el.textTransform = getTextTransform(item);
    el.textRuns = getTextRuns(item);
    // Propagate OpenType feature and text-metric overrides from textStyle to element level
    if (el.textStyle) {
      if (el.textStyle.openTypeFeatures) el.openTypeFeatures = el.textStyle.openTypeFeatures;
      if (el.textStyle.baselineShift !== undefined) el.baselineShift = el.textStyle.baselineShift;
      if (el.textStyle.horizontalScale !== undefined) el.horizontalScale = el.textStyle.horizontalScale;
      if (el.textStyle.verticalScale !== undefined) el.verticalScale = el.textStyle.verticalScale;
    }
  }

  try { if (item.clipping || item.clipped) { el.clipMask = true; ensureElementNotes(el).push("clipping mask"); } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.typename === "CompoundPathItem") { el.isCompoundPath = true; el.fillRule = illustratorFillRule(item); ensureElementNotes(el).push("compound path"); } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }


  // SymbolItem — explicit handling with full metadata
  try {
    if (item.typename === "SymbolItem") {
      el.type = 'symbol';
      el.symbolName = item.symbol ? item.symbol.name : 'unknown';
      if (!expandSymbolDefinitionIntoElement(item, artboardRect, el, depth)) {
        ensureElementNotes(el).push(`Symbol instance: "${el.symbolName}" — definition artwork unavailable; expand symbol before strict export`);
      }
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.typename === "MeshItem") { el.isGradientMesh = true; ensureElementNotes(el).push("gradient mesh"); } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  try { if (item.typename === "GraphItem") { el.isChart = true; ensureElementNotes(el).push("chart/graph"); } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

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

  try {
    for (const effect of effectsFromMetadataText(item && item.XMPString)) mergeEffectByType(fx, effect);
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

  // Approach 2: tags
  try { if (item.tags?.length > 0) for (const tag of item.tags) { try { const n = String(tag.name||"").toLowerCase(); const v = String(tag.value||"").toLowerCase(); const effectType = rasterEffectTypeFromName(`${n} ${v}`); if (effectType) mergeEffectByType(fx, defaultRasterEffect(effectType)); else if (n.includes("effect")||n.includes("shadow")||n.includes("glow")) mergeEffectByType(fx, { type: "effect_from_tag", tagName: n }); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); } } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }

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

function colorToRGB(c, depth = 0) {
  if (!c) return null;
  if (depth > 4) return null;
  try {
    if (c.typename === "SpotColor") {
      const tint = Number(c.tint);
      let rgb = null;
      if (c.spot && c.spot.color) rgb = colorToRGB(c.spot.color, depth + 1);
      if (!rgb && c.color) rgb = colorToRGB(c.color, depth + 1);
      if (!rgb) return null;
      if (Number.isFinite(tint)) {
        const amount = Math.max(0, Math.min(1, tint / 100));
        return {
          r: clampByte(rgb.r * amount + 255 * (1 - amount), 0),
          g: clampByte(rgb.g * amount + 255 * (1 - amount), 0),
          b: clampByte(rgb.b * amount + 255 * (1 - amount), 0),
          a: rgb.a !== undefined ? rgb.a : 255,
        };
      }
      return rgb;
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

function collectionToArray(collection, limit = 512) {
  if (!collection) return [];
  if (typeof collection === "number" || typeof collection === "string") return [collection];
  if (Array.isArray(collection)) return collection.slice(0, limit);
  const length = Number(collection.length);
  if (Number.isFinite(length) && length >= 0) {
    const out = [];
    for (let i = 0; i < Math.min(length, limit); i++) {
      try { if (collection[i]) out.push(collection[i]); } catch (e) { noteExtractionDiagnostic("optional Illustrator collection item unavailable", e); }
    }
    return out;
  }
  return typeof collection === "object" ? [collection] : [];
}

function safeReadProperty(object, property, label) {
  try { return object && object[property]; }
  catch (e) { noteExtractionDiagnostic(label || `optional Illustrator property ${property} unavailable`, e); return null; }
}

function patternPageItemCandidates(pattern) {
  const candidates = [];
  const add = (items, source) => {
    const array = collectionToArray(items);
    if (array.length > 0) candidates.push({ items: array, source });
  };
  const patternItem = safeReadProperty(pattern, "patternItem", "optional Illustrator patternItem unavailable");
  if (patternItem) {
    add(safeReadProperty(patternItem, "pageItems", "optional Illustrator patternItem.pageItems unavailable"), "pattern.patternItem.pageItems");
    add(safeReadProperty(patternItem, "pathItems", "optional Illustrator patternItem.pathItems unavailable"), "pattern.patternItem.pathItems");
    add(safeReadProperty(patternItem, "compoundPathItems", "optional Illustrator patternItem.compoundPathItems unavailable"), "pattern.patternItem.compoundPathItems");
    add(safeReadProperty(patternItem, "groupItems", "optional Illustrator patternItem.groupItems unavailable"), "pattern.patternItem.groupItems");
  }
  add(safeReadProperty(pattern, "pageItems", "optional Illustrator pattern.pageItems unavailable"), "pattern.pageItems");
  add(safeReadProperty(pattern, "pathItems", "optional Illustrator pattern.pathItems unavailable"), "pattern.pathItems");
  const artwork = safeReadProperty(pattern, "artwork", "optional Illustrator pattern.artwork unavailable");
  if (artwork) add(safeReadProperty(artwork, "pageItems", "optional Illustrator pattern.artwork.pageItems unavailable"), "pattern.artwork.pageItems");
  return candidates;
}

function addPatternSwatchColor(stats, color) {
  if (!color) return;
  const c = {
    r: clampByte(color.r, 0),
    g: clampByte(color.g, 0),
    b: clampByte(color.b, 0),
    a: color.a === undefined ? 255 : clampByte(color.a, 255),
  };
  const key = `${c.r},${c.g},${c.b},${c.a}`;
  const entry = stats.get(key) || { ...c, count: 0 };
  entry.count += 1;
  stats.set(key, entry);
}

function collectPatternItemColors(item, stats, depth = 0) {
  if (!item || depth > 6) return;
  try {
    if (item.filled !== false && item.fillColor) addPatternSwatchColor(stats, colorToRGB(item.fillColor));
  } catch (e) { noteExtractionDiagnostic("optional Illustrator pattern fill unavailable", e); }
  try {
    if (item.stroked !== false && item.strokeColor) addPatternSwatchColor(stats, colorToRGB(item.strokeColor));
  } catch (e) { noteExtractionDiagnostic("optional Illustrator pattern stroke unavailable", e); }

  for (const property of ["pageItems", "pathItems", "compoundPathItems", "groupItems", "children"]) {
    const children = collectionToArray(safeReadProperty(item, property, `optional Illustrator pattern ${property} unavailable`));
    for (const child of children) collectPatternItemColors(child, stats, depth + 1);
  }
}

function patternItemStrokeWidth(item) {
  try {
    const width = Number(item && item.strokeWidth);
    return Number.isFinite(width) && width > 0 ? width : 1;
  } catch (e) { noteExtractionDiagnostic("optional Illustrator pattern stroke width unavailable", e); }
  return 1;
}

function collectPatternTileGeometry(item, artboardRect, shapes, depth = 0) {
  if (!item || depth > 6) return;
  try {
    const type = String(item.typename || "");
    if (type === "PathItem" || type === "CompoundPathItem") {
      const subpaths = extractPathSubpaths(item, artboardRect || [0, 0, 0, 0]);
      const fill = item.filled !== false ? colorToRGB(item.fillColor) : null;
      const stroke = item.stroked !== false ? colorToRGB(item.strokeColor) : null;
      const strokeWidth = stroke ? patternItemStrokeWidth(item) : 0;
      for (const subpath of subpaths) {
        const points = (subpath.points || []).map(point => ({
          anchor: tupleFromPoint(point.anchor, [0, 0]),
          leftDir: tupleFromPoint(point.leftDir || point.left_ctrl || point.anchor, [0, 0]),
          rightDir: tupleFromPoint(point.rightDir || point.right_ctrl || point.anchor, [0, 0]),
          left_ctrl: tupleFromPoint(point.left_ctrl || point.leftDir || point.anchor, [0, 0]),
          right_ctrl: tupleFromPoint(point.right_ctrl || point.rightDir || point.anchor, [0, 0]),
          kind: point.kind || "corner",
        }));
        if (points.length >= 2 && (fill || stroke)) {
          shapes.push({ points, closed: subpath.closed !== false, fill, stroke, strokeWidth });
        }
      }
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator pattern geometry unavailable", e); }

  for (const property of ["pageItems", "pathItems", "compoundPathItems", "groupItems", "children"]) {
    const children = collectionToArray(safeReadProperty(item, property, `optional Illustrator pattern geometry ${property} unavailable`));
    for (const child of children) collectPatternTileGeometry(child, artboardRect, shapes, depth + 1);
  }
}

function normalizePatternTileGeometry(shapes) {
  const allPoints = [];
  for (const shape of shapes || []) {
    for (const point of shape.points || []) {
      allPoints.push(point.anchor, point.leftDir, point.rightDir, point.left_ctrl, point.right_ctrl);
    }
  }
  const bounds = boundsFromTuples(allPoints);
  if (!bounds || bounds.w <= 0.0001 || bounds.h <= 0.0001) return [];
  const basis = Math.max(bounds.w, bounds.h, 1);
  return (shapes || []).map(shape => ({
    points: (shape.points || []).map(point => ({
      anchor: [
        (Number((point.anchor || [0, 0])[0] || 0) - bounds.x) / bounds.w,
        (Number((point.anchor || [0, 0])[1] || 0) - bounds.y) / bounds.h,
      ],
      leftDir: [
        (Number((point.leftDir || point.left_ctrl || point.anchor || [0, 0])[0] || 0) - bounds.x) / bounds.w,
        (Number((point.leftDir || point.left_ctrl || point.anchor || [0, 0])[1] || 0) - bounds.y) / bounds.h,
      ],
      rightDir: [
        (Number((point.rightDir || point.right_ctrl || point.anchor || [0, 0])[0] || 0) - bounds.x) / bounds.w,
        (Number((point.rightDir || point.right_ctrl || point.anchor || [0, 0])[1] || 0) - bounds.y) / bounds.h,
      ],
      left_ctrl: [
        (Number((point.left_ctrl || point.leftDir || point.anchor || [0, 0])[0] || 0) - bounds.x) / bounds.w,
        (Number((point.left_ctrl || point.leftDir || point.anchor || [0, 0])[1] || 0) - bounds.y) / bounds.h,
      ],
      right_ctrl: [
        (Number((point.right_ctrl || point.rightDir || point.anchor || [0, 0])[0] || 0) - bounds.x) / bounds.w,
        (Number((point.right_ctrl || point.rightDir || point.anchor || [0, 0])[1] || 0) - bounds.y) / bounds.h,
      ],
      kind: point.kind || "corner",
    })),
    closed: shape.closed !== false,
    fill: shape.fill || null,
    stroke: shape.stroke || null,
    strokeWidth: shape.stroke ? Math.max(0.001, Number(shape.strokeWidth || 1) / basis) : 0,
  })).filter(shape => shape.points.length >= 2 && (shape.fill || shape.stroke));
}

function patternTileGeometryFromItems(items, artboardRect) {
  const shapes = [];
  for (const item of items || []) collectPatternTileGeometry(item, artboardRect, shapes, 0);
  return {
    tileGeometry: normalizePatternTileGeometry(shapes),
    tileGeometryTruncated: false,
  };
}

function patternSwatchFromColor(patternColor, artboardRect) {
  const pattern = patternColor && patternColor.pattern;
  if (!pattern) return null;
  const candidates = patternPageItemCandidates(pattern);
  for (const candidate of candidates) {
    const stats = new Map();
    for (const item of candidate.items) collectPatternItemColors(item, stats, 0);
    const colors = [...stats.values()].sort((a, b) => b.count - a.count);
    if (colors.length > 0) {
      const { tileGeometry, tileGeometryTruncated } = patternTileGeometryFromItems(candidate.items, artboardRect);
      const foreground = { r: colors[0].r, g: colors[0].g, b: colors[0].b, a: colors[0].a };
      const backgroundColor = colors[1]
        ? { r: colors[1].r, g: colors[1].g, b: colors[1].b, a: colors[1].a }
        : { r: 255, g: 255, b: 255, a: 0 };
      return {
        swatchExtracted: true,
        sampled: true,
        swatchSource: candidate.source,
        pageItemCount: candidate.items.length,
        foreground,
        background: backgroundColor,
        colors: colors.slice(0, 8).map(color => ({ r: color.r, g: color.g, b: color.b, a: color.a, count: color.count })),
        tileGeometry,
        tileGeometryTruncated,
      };
    }
  }
  return null;
}

function patternScaleFromColor(patternColor) {
  let values = [];
  try {
    const scale = patternColor && patternColor.scaleFactor;
    values = collectionToArray(scale, 2).map(Number).filter(Number.isFinite);
  } catch (e) { noteExtractionDiagnostic("optional Illustrator pattern scale unavailable", e); }
  if (values.length === 0) return [1, 1];
  if (values.length === 1) values.push(values[0]);
  return values.slice(0, 2).map(value => Math.abs(value) > 10 ? value / 100 : value);
}

function patternMatrixFromColor(patternColor) {
  try {
    const matrix = patternColor && (patternColor.matrix || patternColor.transform || patternColor.patternMatrix || patternColor.pattern_matrix);
    if (Array.isArray(matrix) && matrix.length >= 6 && matrix.slice(0, 6).every(value => Number.isFinite(Number(value)))) {
      return matrix.slice(0, 6).map(Number);
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator pattern transform unavailable", e); }
  return null;
}

function patternTransformFromColor(patternColor) {
  const scale = patternScaleFromColor(patternColor);
  const matrix = patternMatrixFromColor(patternColor);
  const rotationDeg = Number(patternColor && patternColor.rotation);
  return {
    rotationDeg: Number.isFinite(rotationDeg) ? rotationDeg : 0,
    scaleX: scale[0] || 1,
    scaleY: scale[1] || 1,
    offsetX: matrix ? matrix[4] || 0 : 0,
    offsetY: matrix ? matrix[5] || 0 : 0,
    matrix,
  };
}

function getFill(item) {
  try { if (item.filled && item.fillColor) return colorToRGB(item.fillColor); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return null;
}

function getStroke(item, artboardRect) {
  try {
    if (item.stroked && item.strokeColor) {
      const c = colorToRGB(item.strokeColor) || { r: 0, g: 0, b: 0, a: 255 };
      const stroke = { ...c, width: item.strokeWidth || 1 };
      const gradient = getGradientFromColor(item.strokeColor, artboardRect);
      if (gradient) stroke.gradient = gradient;
      return stroke;
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return null;
}

function normalizeStrokeAlignment(value) {
  if (value === null || value === undefined) return null;
  const raw = String(value).toLowerCase();
  if (raw.includes("inside") || raw.includes("inner")) return "inside";
  if (raw.includes("outside") || raw.includes("outer")) return "outside";
  if (raw.includes("center") || raw.includes("middle")) return "center";
  const numeric = Number(value);
  if (Number.isFinite(numeric)) {
    if (numeric === 1) return "inside";
    if (numeric === 2) return "outside";
    if (numeric === 0) return "center";
  }
  return null;
}

function normalizeBlendModeValue(value) {
  if (value === null || value === undefined) return null;
  const ordinalMap = {
    0: "normal", 1: "multiply", 2: "screen", 3: "overlay", 4: "darken", 5: "lighten",
    6: "color_dodge", 7: "color_burn", 8: "hard_light", 9: "soft_light", 10: "difference",
    11: "exclusion", 12: "hue", 13: "saturation", 14: "color", 15: "luminosity"
  };
  if (typeof value === "number" && Number.isFinite(value)) return ordinalMap[Math.trunc(value)] || null;
  const raw = String(value).trim();
  if (!raw) return null;
  if (/^\d+$/.test(raw)) return ordinalMap[Number(raw)] || null;
  const key = raw.toLowerCase().split(".").pop().replace(/[^a-z0-9]+/g, "");
  const map = {
    normal: "normal",
    multiply: "multiply",
    screen: "screen",
    overlay: "overlay",
    darken: "darken",
    lighten: "lighten",
    colordodge: "color_dodge",
    colorburn: "color_burn",
    hardlight: "hard_light",
    softlight: "soft_light",
    difference: "difference",
    exclusion: "exclusion",
    hue: "hue",
    saturation: "saturation",
    saturationblend: "saturation",
    color: "color",
    colorblend: "color",
    luminosity: "luminosity"
  };
  return map[key] || null;
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

function getGradientFromColor(color, artboardRect) {
  if (!color) return null;
  try {
    if (color.typename === "GradientColor") {
      const grad = color.gradient;
      if (!grad) return null;
      const angle = color.angle || 0;
      const stops = [];
      try { for (const s of grad.gradientStops) stops.push({ position: s.rampPoint/100, color: gradientColorToRGB(s.color), opacity: s.opacity !== undefined ? s.opacity/100 : 1 }); } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
      const origin = illustratorPointToEgui(color.origin, artboardRect);
      const length = Number(color.length);
      const hiliteLength = Number(color.hiliteLength);
      const hiliteAngle = Number(color.hiliteAngle);
      const focalPoint = Number.isFinite(hiliteLength) && Number.isFinite(hiliteAngle)
        ? offsetIllustratorPoint(origin, hiliteLength, -hiliteAngle)
        : origin;
      const transform = readGradientMatrix(color.matrix, artboardRect);
      return { type: grad.type === 1 ? "linear" : "radial", angle, center: origin, focalPoint, radius: Number.isFinite(length) && length > 0 ? length : null, transform, stops };
    }
    // PatternColor — not a gradient but handled here for consistency
    if (color.typename === "PatternColor") {
      const swatch = patternSwatchFromColor(color, artboardRect) || {};
      const transform = patternTransformFromColor(color);
      return {
        type: 'pattern',
        patternName: color.pattern ? color.pattern.name : 'unknown',
        rotation: transform.rotationDeg,
        rotationDeg: transform.rotationDeg,
        scale: [transform.scaleX, transform.scaleY],
        scaleX: transform.scaleX,
        scaleY: transform.scaleY,
        offsetX: transform.offsetX,
        offsetY: transform.offsetY,
        matrix: transform.matrix,
        ...swatch,
      };
    }
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return null;
}

function getGradient(item, artboardRect) {
  return getGradientFromColor(item && item.fillColor, artboardRect);
}

function gradientColorToRGB(c) {
  const rgb = colorToRGB(c);
  return rgb ? { r: rgb.r, g: rgb.g, b: rgb.b } : { r: 128, g: 128, b: 128 };
}

// ─── Text Details ────────────────────────────────────────────────────────────
function getOpenTypeFeatures(attrs) {
  if (!attrs) return undefined;
  const read = (name) => {
    try { return attrs[name]; }
    catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); return undefined; }
  };
  const bool = (name) => {
    const value = read(name);
    return value === undefined ? undefined : !!value;
  };
  const features = {};
  const setIfOverride = (name, value, defaultValue) => {
    if (value !== undefined && value !== defaultValue) features[name] = value;
  };

  setIfOverride("ligatures", bool("ligatures"), true);
  setIfOverride("contextualLigatures", bool("contextualLigatures"), true);
  setIfOverride("discretionaryLigatures", bool("discretionaryLigatures"), false);
  setIfOverride("fractions", bool("fractions"), false);
  setIfOverride("ordinals", bool("ordinals"), false);
  setIfOverride("swash", bool("swash"), false);
  setIfOverride("titlingAlternates", bool("titlingAlternates"), false);
  setIfOverride("stylisticAlternates", bool("stylisticAlternates"), false);

  const kerningMethod = read("kerningMethod");
  if (kerningMethod !== undefined) {
    const normalized = String(kerningMethod).toLowerCase();
    if (kerningMethod === false || normalized.includes("none") || normalized.includes("off")) {
      features.kerning = false;
    }
  }

  return Object.keys(features).length > 0 ? features : undefined;
}

function getTextMetricsOverrides(attrs) {
  if (!attrs) return undefined;
  try {
    const bs = attrs.baselineShift;
    const hs = attrs.horizontalScale;
    const vs = attrs.verticalScale;
    const metrics = {};
    if (Number.isFinite(Number(bs)) && Math.abs(Number(bs)) > 0.0001) metrics.baselineShift = Number(bs);
    if (Number.isFinite(Number(hs)) && Math.abs(Number(hs) - 100) > 0.0001) metrics.horizontalScale = Number(hs) / 100;
    if (Number.isFinite(Number(vs)) && Math.abs(Number(vs) - 100) > 0.0001) metrics.verticalScale = Number(vs) / 100;
    return Object.keys(metrics).length > 0 ? metrics : undefined;
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return undefined;
}

function getTextStyle(item) {
  try {
    const chars = item.textRange.characterAttributes;
    let size = 14, weight = 400, family = "default";
    try { size = chars.size || 14; } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
    try { if (chars.textFont) { const n = chars.textFont.name || ""; weight = n.includes("Bold") ? 700 : n.includes("Light") ? 300 : 400; family = n; } } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
    const otf = getOpenTypeFeatures(chars);
    const metrics = getTextMetricsOverrides(chars);
    const result = { size, fontSize: size, weight, family };
    if (otf) result.openTypeFeatures = otf;
    if (metrics) {
      if (metrics.baselineShift !== undefined) result.baselineShift = metrics.baselineShift;
      if (metrics.horizontalScale !== undefined) result.horizontalScale = metrics.horizontalScale;
      if (metrics.verticalScale !== undefined) result.verticalScale = metrics.verticalScale;
    }
    result.letterSpacing = illustratorTrackingToPx(chars.tracking, size);
    result.lineHeight = illustratorLeadingToMultiplier(chars.leading, size);
    result.textDecoration = getTextDecoration(item);
    result.textTransform = getTextTransform(item);
    return result;
  } catch (e) { return { size: 14, fontSize: 14, weight: 400, family: "default" }; }
}

function getTextAlign(item) {
  if (item.typename !== "TextFrame") return null;
  try {
    const j = item.textRange.paragraphAttributes.justification;
    const name = String(j || "").toUpperCase();
    if (typeof Justification !== "undefined" && j === Justification.LEFT) return "left";
    if (typeof Justification !== "undefined" && j === Justification.CENTER) return "center";
    if (typeof Justification !== "undefined" && j === Justification.RIGHT) return "right";
    if (typeof Justification !== "undefined") {
      if (j === Justification.FULLJUSTIFYLASTLINECENTER) return "justified_last_line_center";
      if (j === Justification.FULLJUSTIFYLASTLINERIGHT) return "justified_last_line_right";
      if (j === Justification.FULLJUSTIFYLASTLINELEFT) return "justified";
      if (j === Justification.FULLJUSTIFY) return "justified_all";
    }
    if (name.includes("LASTLINECENTER")) return "justified_last_line_center";
    if (name.includes("LASTLINERIGHT")) return "justified_last_line_right";
    if (name.includes("JUSTIFY")) return name.includes("LASTLINELEFT") ? "justified" : "justified_all";
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return "left";
}

function illustratorTrackingToPx(tracking, fontSize) {
  const t = Number(tracking);
  const size = Number(fontSize) || 14;
  if (!Number.isFinite(t) || t === 0) return null;
  return (t / 1000) * size;
}

function illustratorLeadingToMultiplier(leading, fontSize) {
  const l = Number(leading);
  const size = Number(fontSize) || 14;
  if (!Number.isFinite(l) || l <= 0 || size <= 0) return null;
  return l / size;
}

function getLetterSpacing(item) {
  if (item.typename !== "TextFrame") return null;
  try {
    const attrs = item.textRange.characterAttributes;
    return illustratorTrackingToPx(attrs.tracking, attrs.size || 14);
  } catch (e) { return null; }
}

function getLineHeight(item) {
  if (item.typename !== "TextFrame") return null;
  try {
    const attrs = item.textRange.characterAttributes;
    return illustratorLeadingToMultiplier(attrs.leading, attrs.size || 14);
  } catch (e) { return null; }
}

function getTextDecoration(item) {
  if (item.typename !== "TextFrame") return null;
  try {
    const u = item.textRange.characterAttributes.underline;
    const s = item.textRange.characterAttributes.strikeThrough;
    if (u && s) return "both";
    if (u) return "underline";
    if (s) return "strikethrough";
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return null;
}

function getTextTransform(item) {
  if (item.typename !== "TextFrame") return null;
  try {
    if (item.textRange.characterAttributes.smallCaps) return "small_caps";
    if (item.textRange.characterAttributes.allCaps) return "uppercase";
  } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); }
  return null;
}

function getTextRuns(item) {
  if (item.typename !== "TextFrame") return null;
  try {
    const runs = [], trs = item.textRanges;
    if (trs && trs.length > 1) { for (const tr of trs) { try {
      const a = tr.characterAttributes;
      const fontName = a.textFont?.name || "";
      const style = { size: a.size||14, fontSize: a.size||14, weight: fontName.includes("Bold") ? 700 : fontName.includes("Light") ? 300 : 400, family: fontName || null, color: colorToRGB(a.fillColor), letterSpacing: illustratorTrackingToPx(a.tracking, a.size || 14), lineHeight: illustratorLeadingToMultiplier(a.leading, a.size || 14), textDecoration: (a.underline && a.strikeThrough) ? "both" : a.underline ? "underline" : a.strikeThrough ? "strikethrough" : null, textTransform: a.smallCaps ? "small_caps" : a.allCaps ? "uppercase" : null };
      const otf = getOpenTypeFeatures(a);
      const metrics = getTextMetricsOverrides(a);
      if (otf) style.openTypeFeatures = otf;
      if (metrics) {
        if (metrics.baselineShift !== undefined) style.baselineShift = metrics.baselineShift;
        if (metrics.horizontalScale !== undefined) style.horizontalScale = metrics.horizontalScale;
        if (metrics.verticalScale !== undefined) style.verticalScale = metrics.verticalScale;
      }
      runs.push({ text: tr.contents || "", style });
    } catch (e) { noteExtractionDiagnostic("optional Illustrator property unavailable", e); } } }
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
  const scale = Array.isArray(g.scale) ? g.scale : [g.scaleX, g.scaleY].filter(value => Number.isFinite(Number(value)));
  const rotation = g.rotationDeg !== undefined ? g.rotationDeg : g.rotation || 0;
  return stableHash32(`${g.patternName || g.pattern_name || g.name || g.type || "pattern"}:${rotation}:${JSON.stringify(scale)}`);
}

function patternMetrics(g) {
  const seed = patternSeed(g);
  const scale = Array.isArray(g.scale)
    ? g.scale.filter(v => Number.isFinite(Number(v))).map(Number)
    : [g.scaleX, g.scaleY].filter(v => Number.isFinite(Number(v))).map(Number);
  const avgScale = scale.length > 0 ? scale.reduce((a, v) => a + v, 0) / scale.length : 1;
  const cell = Math.max(2, Math.min(64, 8 * avgScale));
  const mark = Math.max(0.5, Math.min(16, cell * 0.12));
  return { seed, cell, mark };
}

function normalizePatternColor(value, fallback, alphaFallback = 255) {
  if (!value) return fallback;
  const rgb = stopColorToRgb(value);
  return {
    r: clampByte(rgb.r, fallback.r),
    g: clampByte(rgb.g, fallback.g),
    b: clampByte(rgb.b, fallback.b),
    a: value.a === undefined ? alphaFallback : clampByte(value.a, alphaFallback),
  };
}

function patternRenderColors(g, opacity = 1) {
  const { seed } = patternMetrics(g);
  const seededForeground = {
    r: 64 + (seed & 0x7f),
    g: 64 + ((seed >>> 8) & 0x7f),
    b: 64 + ((seed >>> 16) & 0x7f),
    a: 220,
  };
  const seededBackground = {
    r: 255 - Math.floor(seededForeground.r / 2),
    g: 255 - Math.floor(seededForeground.g / 2),
    b: 255 - Math.floor(seededForeground.b / 2),
    a: 48,
  };
  const safeOpacity = Math.max(0, Math.min(1, Number(opacity === undefined ? 1 : opacity)));
  const foreground = normalizePatternColor(g.foreground, seededForeground, g.foreground ? 255 : seededForeground.a);
  const background = normalizePatternColor(g.background, seededBackground, g.background ? 255 : seededBackground.a);
  return {
    foreground: { ...foreground, a: clampByte(foreground.a * safeOpacity, seededForeground.a) },
    background: { ...background, a: clampByte(background.a * safeOpacity, seededBackground.a) },
    swatchExtracted: !!(g.swatchExtracted || g.sampled),
  };
}

function rgbaUnmultipliedExpr(color) {
  return `egui::Color32::from_rgba_unmultiplied(${clampByte(color.r, 0)}, ${clampByte(color.g, 0)}, ${clampByte(color.b, 0)}, ${clampByte(color.a, 255)})`;
}

function patternFillPathCode(g, pointsExpr, pad, opacity) {
  const { seed, cell, mark } = patternMetrics(g);
  const colors = patternRenderColors(g, opacity);
  const name = sanitizeComment(g.patternName || g.pattern_name || g.name || g.type || "pattern");
  const tileGeometry = g.tileGeometry || g.tile_geometry || [];
  const hasTileGeometry = Array.isArray(tileGeometry) && tileGeometry.length > 0;
  const note = hasTileGeometry
    ? `Pattern fill "${name}" — tiled from sampled Illustrator pattern swatch geometry`
    : colors.swatchExtracted
    ? `Pattern fill "${name}" — sampled from Illustrator pattern swatch colors`
    : `Pattern fill "${name}" — procedural approximation; non-strict diagnostic only`
  if (hasTileGeometry) {
    return `${pad}// ${note}\n${pad}{\n${pad}    let pattern = ${patternDefExpr(g)};\n${pad}    for s in egui_expressive::scene::pattern_fill_path_from_def(${pointsExpr}, &pattern, ${fmtF32(opacity === undefined ? 1 : opacity)}) { painter.add(s); }\n${pad}}\n`;
  }
  return `${pad}// ${note}\n${pad}for s in egui_expressive::pattern_fill_path(${pointsExpr}, ${seed}u32, ${rgbaUnmultipliedExpr(colors.foreground)}, ${rgbaUnmultipliedExpr(colors.background)}, ${fmtF32(cell)}, ${fmtF32(mark)}) { painter.add(s); }\n`;
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
  return `// Auto-generated component hook.\n// Local wrapper primitives are intentionally not emitted here.\n// Reusable design primitives live in egui_expressive (scene, typography).\n`;
}

function generateArtboardFile(ab, els, colorMap, stateName, comps, options) {
  const sn = toSnakeName(ab.name);
  let usesShadow = false, usesBlur = false, usesComponents = false, usesClipPath = false, usesBlendMode = false;
  const walk = (elements) => {
    for (const el of elements) {
      if (el.effects?.some(e => e.type === "dropShadow" || e.type === "innerShadow" || e.type === "outerGlow" || e.type === "innerGlow")) usesShadow = true;
      if (el.effects?.some(e => e.type === "gaussianBlur" || e.type === "feather")) usesBlur = true;
      if (el.clipMask && clipPathTuplesForElement(el)) usesClipPath = true;
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

  let c = `// Auto-generated by egui_expressive Illustrator Exporter\n// Artboard: "${sanitizeComment(ab.name)}" (${Math.round(ab.width)} × ${Math.round(ab.height)} px)\n// Options: semantic_color_names=${options?.naming !== false}, sidecar=${options?.sidecar !== false || options?.includeSidecar !== false}\n\n#[allow(unused_imports)]\nuse egui::{${imports.join(", ")}};\n#[allow(unused_imports)]\nuse egui_expressive::{${exprImports.join(", ")}};\n#[allow(unused_imports)]\nuse super::tokens;\nuse super::state::${stateName}State;\n`;
  if (usesComponents) c += `use super::components;\n`;
  c += `\n#[allow(unused_variables)]\npub fn draw_${sn}(ui: &mut Ui, state: &mut ${stateName}State) -> Option<super::state::${stateName}Action> {\n    let origin = ui.cursor().min;\n    ui.allocate_space(egui::vec2(${fmtF32(ab.width)}, ${fmtF32(ab.height)}));\n    let painter = ui.painter().clone();\n\n`;
  c += `    // Transparent artboard background; explicit Illustrator background objects are rendered below.\n\n`;
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

function hasMeshPatches(el) {
  return !!(el && el.mesh_patches && el.mesh_patches.length > 0);
}

function truthyExpansionFlag(value) {
  return value === true || value === 1 || value === "1" || String(value).toLowerCase() === "true";
}

function parserExpansionFlags(el) {
  return {
    parserRecovered: truthyExpansionFlag(el && (el.parserRecovered || el.parser_recovered || el.recoveredVectors || el.recovered_vectors)),
    appearanceExpanded: truthyExpansionFlag(el && (el.appearanceExpanded || el.appearance_expanded || el.recoveredAppearance || el.recovered_appearance)),
    symbolExpanded: truthyExpansionFlag(el && (el.symbolExpanded || el.symbol_expanded)),
    expandedChildren: truthyExpansionFlag(el && (el.expandedChildren || el.expanded_children || el.recoveredChildren || el.recovered_children)),
  };
}

function hasParserRecoveredVectors(el) {
  if (!el || !el.children || el.children.length === 0) return false;
  const flags = parserExpansionFlags(el);
  return !!(flags.parserRecovered || flags.expandedChildren || flags.symbolExpanded) && el.children.every(child => isSceneRenderableElement(child));
}

function hasParserExpandedAppearance(el) {
  if (!el) return false;
  const flags = parserExpansionFlags(el);
  if (!flags.appearanceExpanded) return false;
  if (Array.isArray(el.appearanceStack) && el.appearanceStack.length > 0) return true;
  if (Array.isArray(el.appearance_stack) && el.appearance_stack.length > 0) return true;
  if ((el.appearance_fills || el.appearanceFills || []).length > 0) return true;
  if ((el.appearance_strokes || el.appearanceStrokes || []).length > 0) return true;
  return false;
}

function hasParserExpandedVectorContract(el) {
  return hasParserRecoveredVectors(el) || hasParserExpandedAppearance(el);
}

function unsupportedOpaquePrimitiveReason(el) {
  if (el && (el.type === "chart" || el.isChart)) return "Chart/graph";
  if (el && (el.type === "mesh" || el.isGradientMesh)) return "Gradient mesh";
  if (el && el.type === "plugin") return "Plugin item";
  return "Unsupported opaque primitive";
}

function requiresOpaqueVectorRecovery(el) {
  if (!el) return false;
  if (hasParserRecoveredVectors(el)) return false;
  return el.type === "unknown"
    || el.type === "plugin"
    || el.type === "chart"
    || el.isChart
    || ((el.type === "mesh" || el.isGradientMesh) && !hasMeshPatches(el));
}

function generateElementCodeInner(el, indent, colorMap, comps, options) {
  const pad = "    ".repeat(indent);
  let c = "";

  if (requiresOpaqueVectorRecovery(el)) {
    throw new Error(`Cannot export ${sanitizeComment(el.id)} without vector geometry: ${unsupportedOpaquePrimitiveReason(el)}`);
  }
  if (hasMeshPatches(el)) {
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
  c += generateElementComment(el) + "\n";
  for (const n of el.notes || []) c += `${pad}// ${sanitizeComment(n)}\n`;

  if (isSceneVectorElement(el)) {
    return c + sceneBackedAppearanceCode(
      el,
      pad,
      sceneLayersForElement(el),
      options,
      "Vector primitive routed through egui_expressive::scene for exporter/code-first parity."
    );
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
      if (isStrictCodeOnly(options)) {
        const expansionNote = expansionFallbackFailureNote(el);
        throw new Error(`Cannot export code-only Rust: ${el.id}: [unsupported] unexpanded symbol requires expanded vector geometry${expansionNote ? `; ${expansionNote}` : ""}`);
      }
      c += `${pad}    let rect = egui::Rect::from_min_size(origin + egui::vec2(${fmtF32(el.x)}, ${fmtF32(el.y)}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)}));\n`;
      c += `${pad}    painter.rect_stroke(rect, 2u8, egui::Stroke::new(1.0, egui::Color32::from_gray(150)), egui::StrokeKind::Outside);\n`;
    }
    c += `${pad}}\n`;
    return c;
  }

  if (el.type === "text" && el.text) return c + textBlockCode(el, pad, colorMap);

  if (el.type === "image") {
    const reason = rasterImageUnsupportedReason(el);
    if (isStrictCodeOnly(options)) {
      throw new Error(`Cannot export code-only Rust: ${el.id}: [unsupported] ${reason}`);
    }
    c += `${pad}// ${sanitizeComment(reason)}.\n`;
    return c;
  }

  if (el.type === "group" && el.children?.length > 0 && (isSceneRenderableElement(el) || (el.clipMask && (el.children || []).every(isSceneRenderableClipChild)))) {
    return c + sceneBackedAppearanceCode(
      el,
      pad,
      sceneLayersForElement(el),
      options,
      "Group routed through egui_expressive::scene so clipping/blending stays in core primitives."
    );
  }

  if (el.type === "group" && el.children?.length > 0) {
    // Render children at their absolute positions (preserves Illustrator layout)
    c += `${pad}// Group: ${el.id}\n`;
    c += `${pad}{\n`;
    if (el.clipMask) {
      const unsupportedClipReason = mixedClipUnsupportedReason(el);
      if (unsupportedClipReason && isStrictCodeOnly(options)) {
        throw new Error(`Cannot export code-only Rust: ${el.id}: [unsupported] ${unsupportedClipReason}`);
      }
      const clipPath = clipPathTuplesForElement(el);
      if (clipPath) {
        c += `${pad}    // Mixed clip group: applying vector path clip to child painter output.\n`;
        c += `${pad}    let clip_path = ${rustPointsVec(clipPath, pad + "    ")};\n`;
        c += `${pad}    let painter = egui_expressive::with_clip_path(&painter, clip_path);\n`;
      } else {
        c += `${pad}    let clip_rect = egui::Rect::from_min_size(origin + egui::vec2(${fmtF32(el.x)}, ${fmtF32(el.y)}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)}));\n`;
        c += `${pad}    let painter = painter.with_clip_rect(clip_rect);\n`;
      }
    }
    for (const ch of el.children) c += generateElementCode(ch, indent + 1, colorMap, comps, options);
    c += `${pad}}\n`;
    return c;
  }

  return `${pad}// ${el.id} (${el.type})\n`;
}

function generateElementCode(el, indent, colorMap, comps, options) {
  if (isSceneVectorElement(el)) return generateElementCodeInner(el, indent, colorMap, comps, options);
  if (el.blendMode && el.blendMode !== "normal") {
    const variant = blendModeRust(el.blendMode);
    if (variant) {
      const pad = "    ".repeat(indent);

      let c = `${pad}{\n`;
      if (isStrictCodeOnly(options)) {
        throw new Error(`Cannot export code-only Rust: ${el.id}: [unsupported] non-vector element with blend mode ${el.blendMode} requires scene-routed compositing before strict export`);
      }
      c += `${pad}    // WARNING: Non-vector element with blend mode ${el.blendMode} requires scene-routed compositing for exact strict export. Emitting with fallback.\n`;
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

function rustString(s) {
  return JSON.stringify(String(s || ""));
}

function blendModeRust(mode) {
  const map = {
    multiply: "Multiply", screen: "Screen", overlay: "Overlay", darken: "Darken", lighten: "Lighten",
    color_dodge: "ColorDodge", color_burn: "ColorBurn", hard_light: "HardLight", soft_light: "SoftLight",
    difference: "Difference", exclusion: "Exclusion", hue: "Hue", saturation: "Saturation", color: "Color", luminosity: "Luminosity"
  };
  return map[normalizeBlendModeValue(mode) || String(mode || "normal")] || null;
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

function textBlockAlignExpr(align) {
  const value = String(align || "left").toLowerCase();
  if (value === "center" || value === "centre") return "egui_expressive::TextBlockAlign::Center";
  if (value === "right") return "egui_expressive::TextBlockAlign::Right";
  if (value === "justified_last_line_center" || value === "justify_last_line_center") return "egui_expressive::TextBlockAlign::JustifiedLastLineCenter";
  if (value === "justified_last_line_right" || value === "justify_last_line_right") return "egui_expressive::TextBlockAlign::JustifiedLastLineRight";
  if (value === "justified_all" || value === "full_justify") return "egui_expressive::TextBlockAlign::JustifiedAll";
  if (value === "justified" || value === "justify") return "egui_expressive::TextBlockAlign::Justified";
  return "egui_expressive::TextBlockAlign::Left";
}

function textDecorationExpr(decoration) {
  const value = String(decoration || "none").toLowerCase();
  if (value === "underline") return "egui_expressive::TextDecoration::Underline";
  if (value === "strikethrough" || value === "line-through") return "egui_expressive::TextDecoration::Strikethrough";
  if (value === "both" || value === "underline_strikethrough") return "egui_expressive::TextDecoration::Both";
  return "egui_expressive::TextDecoration::None";
}

function textTransformExpr(transform) {
  const value = String(transform || "none").toLowerCase();
  if (value === "uppercase" || value === "all_caps") return "egui_expressive::TextTransform::Uppercase";
  if (value === "small_caps") return "egui_expressive::TextTransform::SmallCaps";
  if (value === "lowercase") return "egui_expressive::TextTransform::Lowercase";
  if (value === "capitalize") return "egui_expressive::TextTransform::Capitalize";
  return "egui_expressive::TextTransform::None";
}

function normalizedAlpha(value, fallback) {
  const raw = Number(value);
  if (!Number.isFinite(raw)) return fallback;
  return raw > 1 ? raw / 255 : raw;
}

function textColorExpr(color, colorMap, fallback, opacity) {
  const alpha = normalizedAlpha(opacity, 1) * normalizedAlpha(color && color.a !== undefined ? color.a : 1, 1);
  const base = color ? `tokens::${colorMap.get(`${color.r},${color.g},${color.b}`) || "ON_SURFACE"}` : (fallback || "tokens::ON_SURFACE");
  return alpha < 0.999 ? `egui_expressive::with_alpha(${base}, ${fmtF32(alpha)})` : base;
}

function typeSpecExpr(style, colorExpr, inherited) {
  const resolved = style || {};
  const parent = inherited || {};
  const size = resolved.size || resolved.fontSize || parent.size || parent.fontSize || 14;
  const weight = resolved.weight || parent.weight || 400;
  const family = resolved.family || resolved.fontFamily || parent.family || parent.fontFamily || null;
  const letterSpacing = resolved.letterSpacing !== undefined ? resolved.letterSpacing : (parent.letterSpacing || 0);
  const decoration = resolved.textDecoration || parent.textDecoration || "none";
  const transform = resolved.textTransform || parent.textTransform || "none";
  const otf = resolved.openTypeFeatures || parent.openTypeFeatures || null;
  const baselineShift = resolved.baselineShift !== undefined ? resolved.baselineShift : (parent.baselineShift || 0);
  const hScale = resolved.horizontalScale !== undefined ? resolved.horizontalScale : (parent.horizontalScale || 1);
  const vScale = resolved.verticalScale !== undefined ? resolved.verticalScale : (parent.verticalScale || 1);
  let expr = `egui_expressive::TypeSpec::new(${fmtF32(size)}).weight(${Math.round(weight)}).letter_spacing(${fmtF32(letterSpacing)}).color(${colorExpr}).decoration(${textDecorationExpr(decoration)}).text_transform(${textTransformExpr(transform)})`;
  if (family && String(family).trim()) expr += `.font_family(${rustString(family)})`;
  else if (weight >= 600) expr += `.font_family("Bold")`;

  // Baseline shift (Illustrator baselineShift is upward-positive)
  if (baselineShift) expr += `.baseline_shift(${fmtF32(baselineShift)})`;

  // Horizontal/vertical scale (non-default)
  if (hScale !== 1) expr += `.horizontal_scale(${fmtF32(hScale)})`;
  if (vScale !== 1) expr += `.vertical_scale(${fmtF32(vScale)})`;

  // OpenType features — emit if anything is overridden from defaults
  if (otf) {
    const parts = [];
    if (otf.ligatures !== undefined && otf.ligatures !== true) parts.push('ligatures: false');
    if (otf.contextualLigatures !== undefined && otf.contextualLigatures !== true) parts.push('contextual_ligatures: false');
    if (otf.discretionaryLigatures !== undefined && otf.discretionaryLigatures !== false) parts.push('discretionary_ligatures: true');
    if (otf.fractions !== undefined && otf.fractions !== false) parts.push('fractions: true');
    if (otf.ordinals !== undefined && otf.ordinals !== false) parts.push('ordinals: true');
    if (otf.swash !== undefined && otf.swash !== false) parts.push('swash: true');
    if (otf.titlingAlternates !== undefined && otf.titlingAlternates !== false) parts.push('titling_alternates: true');
    if (otf.stylisticAlternates !== undefined && otf.stylisticAlternates !== false) parts.push('stylistic_alternates: true');
    if (otf.kerning !== undefined && otf.kerning !== true) parts.push('kerning: false');
    if (parts.length > 0) {
      expr += `.open_type_features(egui_expressive::OpenTypeFeatures { ${parts.join(", ")}, ..Default::default() })`;
    }
  }
  return expr;
}

function textBlockCode(el, pad, colorMap) {
  const spans = [];
  const inherited = {
    ...(el.textStyle || {}),
    letterSpacing: el.letterSpacing !== undefined ? el.letterSpacing : el.textStyle?.letterSpacing,
    lineHeight: el.lineHeight !== undefined ? el.lineHeight : el.textStyle?.lineHeight,
    textDecoration: el.textDecoration,
    textTransform: el.textTransform,
    openTypeFeatures: el.openTypeFeatures || el.textStyle?.openTypeFeatures,
    baselineShift: el.baselineShift !== undefined ? el.baselineShift : el.textStyle?.baselineShift,
    horizontalScale: el.horizontalScale !== undefined ? el.horizontalScale : el.textStyle?.horizontalScale,
    verticalScale: el.verticalScale !== undefined ? el.verticalScale : el.textStyle?.verticalScale,
  };
  if (el.textRuns && el.textRuns.length > 0) {
    for (const run of el.textRuns) {
      if (!run.text) continue;
      const runColor = run.style?.color || el.fill || null;
      const runOpacity = (run.style && run.style.opacity !== undefined ? run.style.opacity : 1) * (el.opacity !== undefined ? el.opacity : 1);
      spans.push({ text: run.text, spec: typeSpecExpr(run.style || {}, textColorExpr(runColor, colorMap, null, runOpacity), inherited) });
    }
  } else {
    spans.push({ text: el.text || "", spec: typeSpecExpr(el.textStyle || {}, textColorExpr(el.fill, colorMap, null, el.opacity), inherited) });
  }
  const spanExpr = spans.map(span => `egui_expressive::TextSpan::new(${rustString(span.text)}, ${span.spec})`).join(", ");
  const lineHeight = el.lineHeight || el.textStyle?.lineHeight || 1.2;
  const layoutWidth = Number.isFinite(Number(el.w)) && Number(el.w) > 0 ? fmtF32(el.w) : null;
  const align = textBlockAlignExpr(el.textAlign || el.textStyle?.align);

  if (hasTextShapingContract(el) && (!el.textRuns || el.textRuns.length === 0)) {
    const glyphs = shapedGlyphsForExactExport(el).map(g => shapedGlyphExpr(g, pad + "    ")).join(", ");
    let c = `${pad}// Text routed through shaped glyph contract for exact OpenType export.\n`;
    c += `${pad}{\n`;
    c += `${pad}    let spec = ${typeSpecExpr(el.textStyle || {}, textColorExpr(el.fill, colorMap, null, el.opacity), inherited)};\n`;
    c += `${pad}    let shaped = egui_expressive::ShapedGlyphRun { text: ${rustString(el.text || "")}.to_string(), glyphs: vec![${glyphs}] };\n`;
    const glyphOrigin = glyphRunUsesAbsoluteContours(el)
      ? "origin"
      : `origin + egui::vec2(${fmtF32(el.x)}, ${fmtF32(el.y)})`;
    c += `${pad}    egui_expressive::render_shaped_glyph_run(&painter, ${glyphOrigin}, &shaped, &spec);\n`;
    c += `${pad}}\n`;
    return c;
  }

  let c = `${pad}// Text routed through egui_expressive::TextBlock primitive.\n`;
  c += `${pad}{\n`;
  c += `${pad}    let text_block = egui_expressive::TextBlock::from_spans(vec![${spanExpr}])\n`;
  c += `${pad}        .align(${align})\n`;
  c += `${pad}        .line_height(${fmtF32(lineHeight)})`;
  if (layoutWidth) c += `\n${pad}        .layout_width(${layoutWidth})`;
  c += `;\n`;
  c += `${pad}    egui_expressive::render_text_block(&painter, origin + egui::vec2(${fmtF32(el.x)}, ${fmtF32(el.y)}), &text_block);\n`;
  c += `${pad}}\n`;
  return c;
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
  const name = g.patternName || g.pattern_name || g.name || g.type || "pattern";
  const colors = patternRenderColors(g, 1);
  const matrix = Array.isArray(g.matrix) ? g.matrix : Array.isArray(g.transform) ? g.transform : null;
  const rotation = Number.isFinite(Number(g.rotationDeg)) ? Number(g.rotationDeg) : Number.isFinite(Number(g.rotation)) ? Number(g.rotation) : 0;
  const scale = Array.isArray(g.scale) ? g.scale : [g.scaleX, g.scaleY];
  const offset = Array.isArray(g.offset) ? g.offset : [g.offsetX, g.offsetY];
  const transform = Array.isArray(matrix) && matrix.length >= 6 && matrix.slice(0, 6).every(v => Number.isFinite(Number(v)))
    ? `Some([${matrix.slice(0, 6).map(fmtF32).join(", ")}])`
    : "None";
  return `egui_expressive::scene::PatternDef { name: ${rustString(name)}.to_string(), seed: ${seed}u32, foreground: ${rgbaUnmultipliedExpr(colors.foreground)}, background: ${rgbaUnmultipliedExpr(colors.background)}, cell_size: ${fmtF32(cell)}, mark_size: ${fmtF32(mark)}, rotation_deg: ${fmtF32(rotation)}, scale_x: ${fmtF32(scale[0] || 1)}, scale_y: ${fmtF32(scale[1] || 1)}, offset_x: ${fmtF32(offset[0] || 0)}, offset_y: ${fmtF32(offset[1] || 0)}, transform: ${transform}, tile_shapes: ${patternTileShapesExpr(g)} }`;
}

function patternTileShapesExpr(g) {
  const shapes = g.tileGeometry || g.tile_geometry || [];
  if (!Array.isArray(shapes) || shapes.length === 0) return "Vec::new()";
  const entries = shapes.map(shape => {
    const points = (shape.points || []).map(point => {
      const anchor = point.anchor || point;
      const left = point.leftDir || point.left_ctrl || point.leftCtrl || anchor;
      const right = point.rightDir || point.right_ctrl || point.rightCtrl || anchor;
      return `egui_expressive::scene::PatternTilePoint { anchor: egui::pos2(${fmtF32((anchor[0] || 0))}, ${fmtF32((anchor[1] || 0))}), left_ctrl: egui::pos2(${fmtF32((left[0] || 0))}, ${fmtF32((left[1] || 0))}), right_ctrl: egui::pos2(${fmtF32((right[0] || 0))}, ${fmtF32((right[1] || 0))}) }`;
    }).join(", ");
    const fill = shape.fill ? `Some(${rgbaUnmultipliedExpr(shape.fill)})` : "None";
    const stroke = shape.stroke ? `Some(${rgbaUnmultipliedExpr(shape.stroke)})` : "None";
    return `egui_expressive::scene::PatternTileShape { points: vec![${points}], closed: ${shape.closed !== false}, fill: ${fill}, stroke: ${stroke}, stroke_width: ${fmtF32(shape.strokeWidth || shape.stroke_width || 0)} }`;
  }).join(", ");
  return `vec![${entries}]`;
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

function rustPointTuples(points, indent) {
  const pad = indent || "";
  if (!points || points.length === 0) return "&[]";
  return `&[\n${points.map(p => `${pad}    (${fmtF32(p[0])}, ${fmtF32(p[1])}),`).join("\n")}\n${pad}]`;
}

function rustLocalPointsVec(points, indent) {
  return `egui_expressive::scene::path_points(${rustPointTuples(points, indent)})`;
}

function normalizeSubpaths(el) {
  const rawSubpaths = Array.isArray(el?.subpaths) ? el.subpaths : [];
  const contours = [];
  for (const raw of rawSubpaths) {
    const rawPoints = Array.isArray(raw?.points) ? raw.points : (Array.isArray(raw?.pathPoints) ? raw.pathPoints : []);
    const closed = raw?.closed !== false;
    const points = samplePathPoints(rawPoints, closed);
    if (points.length >= 2) contours.push({ points, closed });
  }
  if (contours.length === 0 && el?.pathPoints && el.pathPoints.length >= 2) {
    const closed = el.pathClosed !== false;
    const points = samplePathPoints(el.pathPoints, closed);
    if (points.length >= 2) contours.push({ points, closed });
  }
  return contours;
}

function clipPathTuplesForElement(el) {
  const contours = normalizeSubpaths(el);
  if (contours.length !== 1) return null;
  const points = contours[0].points || [];
  return points.length >= 3 ? points : null;
}

function mixedClipUnsupportedReason(el) {
  if (!isMixedClipGroup(el)) return null;
  const contours = normalizeSubpaths(el);
  if (contours.length > 1) return "mixed compound clipping groups with holes are not parity-safe yet";
  const hasTextChild = (el.children || []).some(child => child.type === "text" || mixedClipHasTextDescendant(child));
  if (hasTextChild) return "mixed clipping groups containing text are not parity-safe yet";
  return null;
}

function mixedClipHasTextDescendant(el) {
  if (!el || !el.children) return false;
  for (const child of el.children) {
    if (child.type === "text") return true;
    if (mixedClipHasTextDescendant(child)) return true;
  }
  return false;
}

function mixedClipHasUnvectorizedRasterChild(el) {
  if (!el || !el.children) return false;
  return el.children.some(child => {
    if (child.type === "image") return true;
    return mixedClipHasUnvectorizedRasterChild(child);
  });
}

function mixedClipRasterVectorizationReason(el) {
  if (!el || !el.children) return null;
  for (const child of el.children) {
    if (child.type === "image") {
      const sourcePath = rasterVectorSourcePath(child);
      if (!sourcePath) return "mixed clipping group contains raster image child without a vector source path; vectorize raster before clip as a preflight requirement";
      if (rasterHasUnsafeVectorEffects(child)) return "mixed clipping group contains raster image child with unsupported effects; vectorize raster before clip as a preflight requirement";
      if (!canBakeRasterRotation(child)) return "mixed clipping group contains raster image child with non-bakeable rotation metadata; vectorize raster before clip as a preflight requirement";
      continue;
    }
    const nested = mixedClipRasterVectorizationReason(child);
    if (nested) return nested;
  }
  return null;
}

function rustPathContourExpr(contour, indent) {
  const pad = indent || "";
  return `egui_expressive::scene::PathContour {\n${pad}    points: ${rustLocalPointsVec(contour.points, pad + "    ")},\n${pad}    closed: ${contour.closed === false ? "false" : "true"},\n${pad}}`;
}

function rustPathContoursVec(contours, indent) {
  const pad = indent || "";
  if (!contours || contours.length === 0) return "vec![]";
  return `vec![\n${contours.map(contour => `${pad}    ${rustPathContourExpr(contour, pad + "    ")},`).join("\n")}\n${pad}]`;
}

function compoundFillRuleExpr(fillRule, elId, options) {
  const value = String(fillRule || "").toLowerCase();
  if (value.includes("even")) return { expr: "egui_expressive::scene::FillRule::EvenOdd", warning: null };
  if (value.includes("nonzero")) return { expr: "egui_expressive::scene::FillRule::NonZero", warning: null };
  throw new Error(`Cannot export code-only Rust: ${elId}: [unsupported] compound path fill rule is unavailable from host extraction; provide explicit ai-parser fill_rule metadata before export`);
}

function rectExpr(el) {
  return `egui::Rect::from_min_size(egui::pos2(${fmtF32(el.x)}, ${fmtF32(el.y)}), egui::vec2(${fmtF32(el.w)}, ${fmtF32(el.h)}))`;
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

function sceneLayersForElement(el) {
  const appearanceFills = el.appearance_fills || el.appearanceFills || [];
  const appearanceStrokes = el.appearance_strokes || el.appearanceStrokes || [];
  if (el.appearanceStack?.length > 0 || appearanceFills.length > 0 || appearanceStrokes.length > 0) {
    return appearanceLayers(el, appearanceFills, appearanceStrokes);
  }

  const layers = [];
  if (el.gradient) layers.push({ type: "fill", gradient: el.gradient, opacity: 1.0, blendMode: "normal" });
  else if (el.fill) layers.push({ type: "fill", color: el.fill, opacity: 1.0, blendMode: "normal" });

  for (const effect of el.effects || []) {
    layers.push({ ...effect, type: effect.type || effect.effectType || "effect", entryType: "effect" });
  }

  if (el.stroke) {
    layers.push({
      type: "stroke",
      color: el.stroke,
      gradient: el.stroke.gradient || el.strokeGradient,
      pattern: el.stroke.pattern,
      width: el.stroke.width || 1,
      opacity: 1.0,
      blendMode: "normal",
      cap: el.strokeCap,
      join: el.strokeJoin,
      dash: el.strokeDash,
      miterLimit: el.strokeMiterLimit,
      alignment: el.strokeAlignment
    });
  }
  return layers;
}

function isSceneVectorElement(el) {
  return ["circle", "ellipse", "path", "shape"].includes(el.type);
}

function isSceneRenderableElement(el) {
  if (isSceneVectorElement(el)) return true;
  return el.type === "group" && (el.children || []).every(isSceneRenderableElement);
}

function isSceneRenderableClipChild(el) {
  return isSceneRenderableElement(el) || el.type === "text";
}

function appearanceHasNonNormalBlend(layers) {
  return layers.some(layer => {
    const mode = layer.blendMode || layer.blend_mode;
    return mode && mode !== "normal";
  });
}

function sceneLayerChainExpr(layer) {
  const kind = appearanceLayerKind(layer);
  const opacity = appearanceOpacity(layer, 1);
  const blend = codegenBlendModeExpr(layer.blendMode || layer.blend_mode);
  const chainOpacity = Math.abs(opacity - 1) > 0.0001 ? `.opacity(${fmtF32(opacity)})` : "";
  const chainBlend = blend.endsWith("::Normal") ? "" : `.blend_mode(${blend})`;

  if (kind === "fill") {
    return `.with_fill_layer(egui_expressive::scene::FillLayer::paint(${paintSourceExpr(layer)})${chainOpacity}${chainBlend})`;
  }

  if (kind === "stroke") {
    const dash = layer.dash || layer.strokeDash;
    const alignment = normalizeStrokeAlignment(layer.alignment || layer.strokeAlignment || layer.stroke_align);
    let expr = `.with_stroke_layer(egui_expressive::scene::StrokeLayer::new(${fmtF32(layer.width || 1)}, ${paintSourceExpr(layer)})${chainOpacity}${chainBlend}`;
    if (layer.cap) expr += `.cap(egui_expressive::codegen::StrokeCap::${strokeCapVariant(layer.cap)})`;
    if (layer.join) expr += `.join(egui_expressive::codegen::StrokeJoin::${strokeJoinVariant(layer.join, layer.miterLimit || layer.miter_limit)})`;
    if (dash && dash.length > 0) expr += `.dash(vec![${dash.map(fmtF32).join(", ")}])`;
    if (Number.isFinite(Number(layer.miterLimit || layer.miter_limit))) expr += `.miter_limit(${fmtF32(layer.miterLimit || layer.miter_limit)})`;
    if (alignment && alignment !== "center") expr += `.alignment(egui_expressive::scene::StrokeAlignment::${strokeAlignmentVariant(alignment)})`;
    return expr + `)`;
  }

  if (kind === "effect") {
    return `.with_effect_layer(egui_expressive::scene::EffectLayer::new(${effectDefExpr(layer)})${chainOpacity}${chainBlend})`;
  }

  return "";
}

function sceneNodeExpr(el, pad, layers, options) {
  const contours = normalizeSubpaths(el);
  const compoundBacked = contours.length > 1;
  const pathBacked = !compoundBacked && el.pathPoints && el.pathPoints.length >= 2;
  const vectorPathBacked = pathBacked || compoundBacked;
  const nodeLayers = layers || sceneLayersForElement(el);
  const compoundFillRule = compoundBacked ? compoundFillRuleExpr(el.fillRule, el.id, options) : null;
  let c;
  if (el.type === "text") {
    const textSize = el.textStyle?.size || el.textSize || 14;
    const textColor = rgbaUnmultipliedExpr(el.fill || el.textStyle?.color || { r: 0, g: 0, b: 0, a: 255 });
    c = `egui_expressive::scene::SceneNode::text(${rustString(el.id)}, ${rustString(el.text || "")}, egui::pos2(${fmtF32(el.x || 0)}, ${fmtF32(el.y || 0)}), ${fmtF32(textSize)}, ${textColor})`;
    const opacity = el.opacity !== undefined ? Number(el.opacity) : 1;
    if (Number.isFinite(opacity) && Math.abs(opacity - 1) > 0.0001) c += `
${pad}    .with_opacity(${fmtF32(opacity)})`;
    const blendMode = codegenBlendModeExpr(el.blendMode);
    if (!blendMode.endsWith("::Normal")) c += `
${pad}    .with_blend_mode(${blendMode})`;
    if (Number(el.rotation || 0) !== 0) c += `
${pad}    .with_rotation(${fmtF32(el.rotation || 0)})`;
    return c;
  }
  if (el.type === "group") {
    if (el.clipMask) {
      if (compoundBacked) {
        c = `${compoundFillRule && compoundFillRule.warning ? `${pad}// WARNING: ${sanitizeComment(compoundFillRule.warning)}\n` : ""}egui_expressive::scene::SceneNode::compound_path(\n${pad}        ${rustString(el.id)},\n${pad}        ${rustPathContoursVec(contours, pad + "        ")},\n${pad}        ${compoundFillRule.expr},\n${pad}    ).with_clip_children(true)`;
      } else if (pathBacked) {
        const sampled = samplePathPoints(el.pathPoints, el.pathClosed !== false);
        c = `egui_expressive::scene::SceneNode::path(\n${pad}        ${rustString(el.id)},\n${pad}        ${rustLocalPointsVec(sampled, pad + "        ")},\n${pad}        ${el.pathClosed === false ? "false" : "true"},\n${pad}    ).with_clip_children(true)`;
      } else {
        c = `egui_expressive::scene::SceneNode::clip_group(${rustString(el.id)}, ${rectExpr(el)})`;
      }
    } else {
      c = `egui_expressive::scene::SceneNode::group(${rustString(el.id)}, ${rectExpr(el)})`;
    }
  } else if (compoundBacked) {
    c = `${compoundFillRule && compoundFillRule.warning ? `${pad}// WARNING: ${sanitizeComment(compoundFillRule.warning)}\n` : ""}egui_expressive::scene::SceneNode::compound_path(\n${pad}        ${rustString(el.id)},\n${pad}        ${rustPathContoursVec(contours, pad + "        ")},\n${pad}        ${compoundFillRule.expr},\n${pad}    )${el.clipMask ? '.with_clip_children(true)' : ''}`;
  } else if (pathBacked) {
    const sampled = samplePathPoints(el.pathPoints, el.pathClosed !== false);
    c = `egui_expressive::scene::SceneNode::path(\n${pad}        ${rustString(el.id)},\n${pad}        ${rustLocalPointsVec(sampled, pad + "        ")},\n${pad}        ${el.pathClosed === false ? "false" : "true"},\n${pad}    )${el.clipMask ? '.with_clip_children(true)' : ''}`;
  } else if (el.type === "circle" || el.type === "ellipse") {
    c = `egui_expressive::scene::SceneNode::ellipse(${rustString(el.id)}, ${rectExpr(el)})`;
  } else {
    c = `egui_expressive::scene::SceneNode::rect(${rustString(el.id)}, ${rectExpr(el)}, ${fmtF32(el.cornerRadius || 0)})`;
  }
  for (const layer of nodeLayers) {
    const chain = sceneLayerChainExpr(layer);
    if (chain) c += `\n${pad}    ${chain}`;
  }
  const opacity = el.opacity !== undefined ? Number(el.opacity) : 1;
  if (Number.isFinite(opacity) && Math.abs(opacity - 1) > 0.0001) c += `\n${pad}    .with_opacity(${fmtF32(opacity)})`;
  const blendMode = codegenBlendModeExpr(el.blendMode);
  if (!blendMode.endsWith("::Normal")) c += `\n${pad}    .with_blend_mode(${blendMode})`;
  if (!vectorPathBacked && Number(el.rotation || 0) !== 0) c += `\n${pad}    .with_rotation(${fmtF32(el.rotation || 0)})`;
  for (const child of el.children || []) {
    if (isSceneRenderableClipChild(child)) c += `\n${pad}    .with_child(${sceneNodeExpr(child, pad + "        ", null, options)})`;
  }
  return c;
}

function sceneBackedAppearanceCode(el, pad, layers, options, reason) {
  let c = `${pad}// ${sanitizeComment(reason || "Vector appearance routed through egui_expressive::scene primitives")}\n`;
  c += `${pad}{\n`;
  c += `${pad}    let scene_node = ${sceneNodeExpr(el, pad + "        ", layers, options)};\n`;
  c += `${pad}    egui_expressive::scene::render_node(ui, &painter, origin.to_vec2(), &scene_node, 1.0);\n`;
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
    || (normalizeStrokeAlignment(el.strokeAlignment) && normalizeStrokeAlignment(el.strokeAlignment) !== "center")
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

function strokeAlignmentVariant(alignment) {
  const normalized = normalizeStrokeAlignment(alignment);
  if (normalized === "inside") return "Inside";
  if (normalized === "outside") return "Outside";
  return "Center";
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

function tupleFromPoint(value, fallback) {
  if (Array.isArray(value)) return [Number(value[0] || 0), Number(value[1] || 0)];
  if (value && typeof value === "object") return [Number(value.x ?? value[0] ?? fallback?.[0] ?? 0), Number(value.y ?? value[1] ?? fallback?.[1] ?? 0)];
  const tuple = fallback || [0, 0];
  return [Number(tuple[0] || 0), Number(tuple[1] || 0)];
}

function normalizeGlyphContour(contour) {
  if (!contour) return null;
  const closed = contour.closed !== false;
  const rawPoints = Array.isArray(contour.points)
    ? contour.points
    : Array.isArray(contour.pathPoints)
      ? contour.pathPoints
      : Array.isArray(contour.contours)
        ? contour.contours
        : [];
  const points = rawPoints.length > 0 && rawPoints[0] && rawPoints[0].anchor
    ? samplePathPoints(rawPoints, closed)
    : rawPoints.map(point => tupleFromPoint(point, [0, 0]));
  if (points.length < 2) return null;
  return { points, closed };
}

function glyphContoursForExport(glyph) {
  return (Array.isArray(glyph && glyph.contours) ? glyph.contours : [])
    .map(normalizeGlyphContour)
    .filter(Boolean);
}

function shapedGlyphExpr(glyph, indent) {
  const pad = indent || "";
  const contours = glyphContoursForExport(glyph);
  const contoursAbsolute = glyph.contoursAbsolute === true || glyph.contours_are_absolute === true;
  return `egui_expressive::ShapedGlyph { glyph_id: ${Number(glyph.glyphId ?? glyph.glyph_id ?? 0)}u32, cluster: ${Number(glyph.cluster ?? 0)}u32, advance_x: ${fmtF32(glyph.advanceX ?? glyph.advance_x ?? 0)}, advance_y: ${fmtF32(glyph.advanceY ?? glyph.advance_y ?? 0)}, offset_x: ${fmtF32(glyph.offsetX ?? glyph.offset_x ?? 0)}, offset_y: ${fmtF32(glyph.offsetY ?? glyph.offset_y ?? 0)}, contours: ${rustPathContoursVec(contours, pad + "    ")}, contours_are_absolute: ${contoursAbsolute}, ..Default::default() }`;
}

function shapedGlyphForSidecar(glyph) {
  if (!glyph) return glyph;
  const contoursAreAbsolute = glyph.contours_are_absolute === true || glyph.contoursAbsolute === true;
  const { contoursAbsolute, contours_are_absolute, ...rest } = glyph;
  return { ...rest, contours_are_absolute: contoursAreAbsolute };
}

function shapedGlyphsForSidecar(glyphs) {
  return Array.isArray(glyphs) && glyphs.length > 0
    ? glyphs.map(shapedGlyphForSidecar)
    : undefined;
}

function textStyleForSidecar(style) {
  if (!style) return undefined;
  const result = {
    fontSize: style.size ?? style.fontSize,
    fontWeight: style.weight,
    fontFamily: style.family,
  };
  for (const key of [
    "openTypeFeatures",
    "baselineShift",
    "horizontalScale",
    "verticalScale",
    "letterSpacing",
    "lineHeight",
    "textDecoration",
    "textTransform",
  ]) {
    if (style[key] !== undefined && style[key] !== null) result[key] = style[key];
  }
  return result;
}

function glyphRunUsesAbsoluteContours(el) {
  const glyphs = shapedGlyphsForExactExport(el);
  return glyphs.some(glyph => (glyph.contoursAbsolute === true || glyph.contours_are_absolute === true)
    && Array.isArray(glyph.contours)
    && glyph.contours.some(contour => Array.isArray(contour && contour.points) && contour.points.length > 0));
}

function glyphHasContours(glyph) {
  return Array.isArray(glyph && glyph.contours)
    && glyph.contours.some(contour => Array.isArray(contour && contour.points) && contour.points.length > 0);
}

function glyphContoursAreAbsolute(glyph) {
  return glyph && (glyph.contoursAbsolute === true || glyph.contours_are_absolute === true);
}

function glyphsUseMixedContourSpaces(glyphs) {
  if (!Array.isArray(glyphs) || glyphs.length === 0) return false;
  let hasAbsolute = false;
  let hasLocal = false;
  for (const glyph of glyphs) {
    if (!glyphHasContours(glyph)) continue;
    if (glyphContoursAreAbsolute(glyph)) hasAbsolute = true;
    else hasLocal = true;
  }
  return hasAbsolute && hasLocal;
}

function rotateTuple(point, center, degrees) {
  const radians = Number(degrees || 0) * Math.PI / 180;
  const cos = Math.cos(radians);
  const sin = Math.sin(radians);
  const [x, y] = tupleFromPoint(point, [0, 0]);
  const dx = x - center.x;
  const dy = y - center.y;
  return [center.x + dx * cos - dy * sin, center.y + dx * sin + dy * cos];
}

function rotatePathPoint(point, center, degrees) {
  const anchor = rotateTuple(point.anchor, center, degrees);
  const leftDir = rotateTuple(point.leftDir || point.left_ctrl || point.leftCtrl || point.anchor, center, degrees);
  const rightDir = rotateTuple(point.rightDir || point.right_ctrl || point.rightCtrl || point.anchor, center, degrees);
  return { ...point, anchor, leftDir, rightDir, left_ctrl: leftDir, right_ctrl: rightDir };
}

function pathPointBoundsTuples(point) {
  return [
    tupleFromPoint(point.anchor, [0, 0]),
    tupleFromPoint(point.leftDir || point.left_ctrl || point.leftCtrl || point.anchor, [0, 0]),
    tupleFromPoint(point.rightDir || point.right_ctrl || point.rightCtrl || point.anchor, [0, 0]),
  ];
}

function rectCornerTuples(el) {
  const x = Number(el.x || 0);
  const y = Number(el.y || 0);
  const w = Number(el.w || 0);
  const h = Number(el.h || 0);
  return [[x, y], [x + w, y], [x + w, y + h], [x, y + h]];
}

function boundsFromTuples(points) {
  if (!points || points.length === 0) return null;
  const xs = points.map(point => Number(point[0] || 0));
  const ys = points.map(point => Number(point[1] || 0));
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minY = Math.min(...ys);
  const maxY = Math.max(...ys);
  return { x: minX, y: minY, w: Math.max(0, maxX - minX), h: Math.max(0, maxY - minY) };
}

function geometryBoundsTuples(el) {
  const points = [];
  const subpaths = Array.isArray(el.subpaths) ? el.subpaths : [];
  for (const subpath of subpaths) for (const point of subpath.points || []) points.push(...pathPointBoundsTuples(point));
  if (points.length === 0 && Array.isArray(el.pathPoints)) for (const point of el.pathPoints) points.push(...pathPointBoundsTuples(point));
  if (Array.isArray(el.children) && el.children.length > 0) for (const child of el.children) points.push(...rectCornerTuples(child));
  if (points.length === 0) points.push(...rectCornerTuples(el));
  return points;
}

function applyBoundsFromGeometry(el) {
  const bounds = boundsFromTuples(geometryBoundsTuples(el));
  return bounds ? { ...el, ...bounds } : el;
}

function rotateVectorElementGeometry(el, center, degrees) {
  const out = { ...el, rotation: 0 };
  if (Array.isArray(out.pathPoints)) out.pathPoints = out.pathPoints.map(point => rotatePathPoint(point, center, degrees));
  if (Array.isArray(out.subpaths)) {
    out.subpaths = out.subpaths.map(subpath => ({
      ...subpath,
      points: (subpath.points || []).map(point => rotatePathPoint(point, center, degrees)),
    }));
    if (out.subpaths.length > 0 && (!out.pathPoints || out.pathPoints.length === 0)) out.pathPoints = out.subpaths[0].points || [];
  }
  if (Array.isArray(out.children)) out.children = out.children.map(child => rotateVectorElementGeometry(child, center, degrees));
  return applyBoundsFromGeometry(out);
}

function rustPointsVec(points, indent) {
  return `egui_expressive::scene::offset_path_points(origin, ${rustPointTuples(points, indent)})`;
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

function mergeParityStatus(a, b) {
  const rank = { exact: 0, approximate: 1, unsupported: 2 };
  return rank[b] > rank[a] ? b : a;
}

function hasOpenTypeFeatureOverrides(features) {
  if (!features) return false;
  return features.ligatures === false
    || features.contextualLigatures === false
    || features.contextual_ligatures === false
    || features.discretionaryLigatures === true
    || features.discretionary_ligatures === true
    || features.fractions === true
    || features.ordinals === true
    || features.swash === true
    || features.titlingAlternates === true
    || features.titling_alternates === true
    || features.stylisticAlternates === true
    || features.stylistic_alternates === true
    || features.kerning === false;
}

function hasNonEmptyByteSource(value) {
  if (!value) return false;
  if (Array.isArray(value)) return value.length > 0;
  if (typeof ArrayBuffer !== 'undefined') {
    if (value instanceof ArrayBuffer) return value.byteLength > 0;
    if (ArrayBuffer.isView && ArrayBuffer.isView(value)) return value.byteLength > 0;
  }
  return typeof value.byteLength === 'number' && value.byteLength > 0;
}

function hasFontByteShapingContract(el) {
  // Font bytes alone are not an exact export contract in the current generator:
  // rustybuzz can shape ids/advances, but codegen does not serialize or render
  // glyph outlines from supplied font bytes. Only explicit contour-bearing glyph
  // contracts are exact.
  return false;
}

function hasTextTypographyOverrides(style) {
  if (!style) return false;
  const nonZero = value => Number.isFinite(Number(value)) && Math.abs(Number(value)) > 0.0001;
  const nonDefaultScale = value => Number.isFinite(Number(value)) && Math.abs(Number(value) - 1) > 0.0001;
  return hasOpenTypeFeatureOverrides(style.openTypeFeatures)
    || nonZero(style.baselineShift)
    || nonDefaultScale(style.horizontalScale)
    || nonDefaultScale(style.verticalScale);
}

function openTypeFeaturesRequireFullShaper(features) {
  if (!features) return false;
  return features.ligatures === false
    || features.contextualLigatures === false
    || features.contextual_ligatures === false
    || features.discretionaryLigatures === true
    || features.discretionary_ligatures === true
    || features.fractions === true
    || features.ordinals === true
    || features.swash === true
    || features.titlingAlternates === true
    || features.titling_alternates === true
    || features.stylisticAlternates === true
    || features.stylistic_alternates === true
    || features.kerning === false;
}

function textStyleRequiresFullShaper(style) {
  if (!style) return false;
  const transform = String(style.textTransform || style.text_transform || "").toLowerCase();
  return openTypeFeaturesRequireFullShaper(style.openTypeFeatures) || transform === "small_caps";
}

function hasTextShapingContract(el) {
  const glyphsHaveCompleteContours = glyphs => Array.isArray(glyphs)
    && glyphs.length > 0
    && glyphs.every(glyphHasContours);
  const hasCompleteSingleSpaceContours = glyphs => glyphsHaveCompleteContours(glyphs)
    && !glyphsUseMixedContourSpaces(glyphs);
  return hasCompleteSingleSpaceContours(el && el.shapedGlyphs)
    || hasCompleteSingleSpaceContours(el && el.outlinedGlyphs);
}

function hasPartialTextShapingContract(el) {
  const glyphsHaveAnyContours = glyphs => Array.isArray(glyphs) && glyphs.some(glyphHasContours);
  return !hasTextShapingContract(el)
    && (glyphsHaveAnyContours(el && el.shapedGlyphs) || glyphsHaveAnyContours(el && el.outlinedGlyphs));
}

function hasMixedContourSpaceTextShapingContract(el) {
  const complete = glyphs => Array.isArray(glyphs) && glyphs.length > 0 && glyphs.every(glyphHasContours);
  return (complete(el && el.shapedGlyphs) && glyphsUseMixedContourSpaces(el.shapedGlyphs))
    || (complete(el && el.outlinedGlyphs) && glyphsUseMixedContourSpaces(el.outlinedGlyphs));
}

function shapedGlyphsForExactExport(el) {
  const complete = glyphs => Array.isArray(glyphs)
    && glyphs.length > 0
    && glyphs.every(glyphHasContours)
    && !glyphsUseMixedContourSpaces(glyphs);
  if (complete(el && el.shapedGlyphs)) return el.shapedGlyphs;
  if (complete(el && el.outlinedGlyphs)) return el.outlinedGlyphs;
  return [];
}

function hasTextRuns(el) {
  return Array.isArray(el && el.textRuns) && el.textRuns.length > 0;
}

function textShapingUnsupportedReasons(el) {
  const reasons = [];
  if (hasMixedContourSpaceTextShapingContract(el)) {
    reasons.push("contour-backed shaped export requires one coordinate space per glyph run");
    return reasons;
  }
  if (hasPartialTextShapingContract(el)) {
    reasons.push("contour-backed shaped export requires contours for every glyph");
    return reasons;
  }
  if (hasTextShapingContract(el)) {
    if (hasTextRuns(el)) reasons.push("contour-backed shaped export is not supported for styled text runs");
    return reasons;
  }
  if (hasFontByteShapingContract(el)) return reasons;
  const add = (source, style) => {
    if (!textStyleRequiresFullShaper(style)) return;
    const features = style && style.openTypeFeatures ? style.openTypeFeatures : {};
    const enabled = [];
    if (features.ligatures === false) enabled.push("standard ligatures disabled");
    if (features.contextualLigatures === false || features.contextual_ligatures === false) enabled.push("contextual ligature toggle");
    if (features.discretionaryLigatures === true || features.discretionary_ligatures === true) enabled.push("discretionary ligatures");
    if (features.fractions === true) enabled.push("fractions");
    if (features.ordinals === true) enabled.push("ordinals");
    if (features.swash === true) enabled.push("swash alternates");
    if (features.titlingAlternates === true || features.titling_alternates === true) enabled.push("titling alternates");
    if (features.stylisticAlternates === true || features.stylistic_alternates === true) enabled.push("stylistic alternates");
    if (features.kerning === false) enabled.push("kerning disabled");
    const transform = String(style && (style.textTransform || style.text_transform) || "").toLowerCase();
    if (transform === "small_caps") enabled.push("true small-caps substitution");
    const detail = enabled.length > 0 ? enabled.join(", ") : "advanced OpenType features";
    reasons.push(`${source}: ${detail} needs shaper-backed shaping or outlined/parser-backed glyph vectors for exact export`);
  };
  add("text style", el && el.textStyle);
  add("text element", el);
  for (const [idx, run] of (el && el.textRuns || []).entries()) add(`text run ${idx + 1}`, run && run.style);
  return [...new Set(reasons)];
}

function hasPatternStroke(el) {
  if (!el) return false;
  if (el.stroke && (isPatternPaint(el.stroke.pattern) || isPatternPaint(el.stroke.gradient))) return true;
  const strokes = el.appearance_strokes || el.appearanceStrokes || [];
  return strokes.some(stroke => stroke && (isPatternPaint(stroke.pattern) || isPatternPaint(stroke.gradient)));
}

function hasPatternFill(el) {
  if (!el) return false;
  if (isPatternPaint(el.gradient) || isPatternPaint(el.pattern)) return true;
  const fills = el.appearance_fills || el.appearanceFills || [];
  return fills.some(fill => isPatternPaint(fill) || isPatternPaint(fill.gradient) || isPatternPaint(fill.pattern));
}

function isPatternPaint(paint) {
  if (!paint) return false;
  const pattern = paint.pattern || paint.gradient || paint;
  const type = String(pattern.type || "").toLowerCase();
  return type === "pattern" || !!pattern.patternName || !!pattern.pattern_name;
}

function patternPaintHasSwatch(paint) {
  if (!isPatternPaint(paint)) return true;
  const pattern = paint.pattern || paint.gradient || paint;
  return !!pattern.foreground && !!(pattern.swatchExtracted || pattern.sampled || pattern.background);
}

function patternPaintHasTileGeometry(paint) {
  if (!isPatternPaint(paint)) return false;
  const pattern = paint.pattern || paint.gradient || paint;
  const shapes = pattern.tileGeometry || pattern.tile_geometry;
  return Array.isArray(shapes) && shapes.length > 0;
}

function patternPaintHasSampledSwatch(paint) {
  if (!isPatternPaint(paint)) return false;
  const pattern = paint.pattern || paint.gradient || paint;
  return !!(pattern.swatchExtracted || pattern.sampled);
}

function patternPaintHasExactTileGeometry(paint) {
  if (!isPatternPaint(paint)) return false;
  const pattern = paint.pattern || paint.gradient || paint;
  const shapes = pattern.tileGeometry || pattern.tile_geometry;
  return patternPaintHasSampledSwatch(paint)
    && Array.isArray(shapes)
    && shapes.length > 0
    && !pattern.tileGeometryTruncated
    && !pattern.tile_geometry_truncated;
}

function patternPaintTileGeometryTruncated(paint) {
  if (!isPatternPaint(paint)) return false;
  const pattern = paint.pattern || paint.gradient || paint;
  return !!(pattern.tileGeometryTruncated || pattern.tile_geometry_truncated);
}

function patternFillsHaveTileGeometry(el) {
  if (!el) return false;
  const paints = [el.gradient, el.pattern];
  const fills = el.appearance_fills || el.appearanceFills || [];
  for (const fill of fills) paints.push(fill, fill && fill.gradient, fill && fill.pattern);
  return paints.some(patternPaintHasTileGeometry);
}

function patternFillsHaveExactTileGeometry(el) {
  if (!el) return false;
  const paints = [el.gradient, el.pattern];
  const fills = el.appearance_fills || el.appearanceFills || [];
  for (const fill of fills) paints.push(fill, fill && fill.gradient, fill && fill.pattern);
  return paints.some(patternPaintHasExactTileGeometry);
}

function patternFillsTileGeometryTruncated(el) {
  if (!el) return false;
  const paints = [el.gradient, el.pattern];
  const fills = el.appearance_fills || el.appearanceFills || [];
  for (const fill of fills) paints.push(fill, fill && fill.gradient, fill && fill.pattern);
  return paints.some(patternPaintTileGeometryTruncated);
}

function patternStrokesHaveTileGeometry(el) {
  if (!el) return false;
  const paints = [el.stroke && el.stroke.pattern, el.stroke && el.stroke.gradient];
  const strokes = el.appearance_strokes || el.appearanceStrokes || [];
  for (const stroke of strokes) paints.push(stroke && stroke.pattern, stroke && stroke.gradient);
  return paints.some(patternPaintHasTileGeometry);
}

function patternStrokesHaveExactTileGeometry(el) {
  if (!el) return false;
  const paints = [el.stroke && el.stroke.pattern, el.stroke && el.stroke.gradient];
  const strokes = el.appearance_strokes || el.appearanceStrokes || [];
  for (const stroke of strokes) paints.push(stroke && stroke.pattern, stroke && stroke.gradient);
  return paints.some(patternPaintHasExactTileGeometry);
}

function patternStrokesTileGeometryTruncated(el) {
  if (!el) return false;
  const paints = [el.stroke && el.stroke.pattern, el.stroke && el.stroke.gradient];
  const strokes = el.appearance_strokes || el.appearanceStrokes || [];
  for (const stroke of strokes) paints.push(stroke && stroke.pattern, stroke && stroke.gradient);
  return paints.some(patternPaintTileGeometryTruncated);
}

function patternFillsMissingSwatch(el) {
  if (!el) return false;
  const paints = [el.gradient, el.pattern];
  const fills = el.appearance_fills || el.appearanceFills || [];
  for (const fill of fills) paints.push(fill, fill && fill.gradient, fill && fill.pattern);
  return paints.some(paint => isPatternPaint(paint) && !patternPaintHasSwatch(paint));
}

function patternStrokesMissingSwatch(el) {
  if (!el) return false;
  const paints = [el.stroke && el.stroke.pattern, el.stroke && el.stroke.gradient];
  const strokes = el.appearance_strokes || el.appearanceStrokes || [];
  for (const stroke of strokes) paints.push(stroke && stroke.pattern, stroke && stroke.gradient);
  return paints.some(paint => isPatternPaint(paint) && !patternPaintHasSwatch(paint));
}

function compoundUsesPatternFill(el) {
  if (isPatternPaint(el.gradient) || isPatternPaint(el.pattern)) return true;
  const fills = el.appearance_fills || el.appearanceFills || [];
  return fills.some(fill => isPatternPaint(fill) || isPatternPaint(fill.gradient) || isPatternPaint(fill.pattern));
}

function hasUnsupportedLiveEffect(el) {
  return unsupportedLiveEffects(el).length > 0;
}

function unsupportedLiveEffects(el) {
  return (el.effects || []).filter(effect => {
    const type = String(effect.type || effect.effectType || effect.effect_type || "").toLowerCase();
    return type === "liveeffect" || type === "live-effect" || type === "unknown";
  });
}

function hasAnyLiveEffect(el) {
  return (el.effects || []).some(effect => {
    const type = String(effect.type || effect.effectType || effect.effect_type || "").toLowerCase();
    return type === "liveeffect" || type === "live-effect" || type === "unknown";
  });
}

function hasCodeRenderedIllustratorEffect(el) {
  return (el.effects || []).some(effect => isSafeRasterVectorEffect(effect));
}

function isMixedClipGroup(el) {
  return el && el.clipMask && el.children && el.children.length > 0 && !el.children.every(isSceneRenderableElement);
}

function isStrictCodeOnly(options) {
  return options?.codeOnlyStrict !== false;
}

function appearanceEntryKind(layer) {
  return appearanceLayerKind(layer || {});
}

function appearanceProbeForElement(el) {
  const probe = el && (el.appearanceProbe || el.appearance_probe || el.multiAppearanceProbe || el.multi_appearance_probe);
  if (!probe) return null;
  const fillCount = Number(probe.fillCount ?? probe.fill_count ?? probe.fills ?? 0);
  const strokeCount = Number(probe.strokeCount ?? probe.stroke_count ?? probe.strokes ?? 0);
  if (!Number.isFinite(fillCount) && !Number.isFinite(strokeCount)) return null;
  if ((fillCount || 0) <= 1 && (strokeCount || 0) <= 1) return null;
  return { fillCount: fillCount || 0, strokeCount: strokeCount || 0, source: probe.source || "metadata" };
}

function knownAppearancePaintCounts(el) {
  const stack = el && (el.appearanceStack || el.appearance_stack);
  if (Array.isArray(stack) && stack.length > 0) {
    return {
      fillCount: stack.filter(layer => appearanceEntryKind(layer) === "fill").length,
      strokeCount: stack.filter(layer => appearanceEntryKind(layer) === "stroke").length
    };
  }
  const fills = el && (el.appearance_fills || el.appearanceFills) || [];
  const strokes = el && (el.appearance_strokes || el.appearanceStrokes) || [];
  return {
    fillCount: fills.length || ((el && (el.fill || el.gradient || el.pattern)) ? 1 : 0),
    strokeCount: strokes.length || ((el && el.stroke) ? 1 : 0)
  };
}

function multiAppearanceFlattenReason(el) {
  if (el && (el.appearanceExpanded || hasParserExpandedAppearance(el))) return null;
  const probe = appearanceProbeForElement(el);
  if (!probe) return null;
  const known = knownAppearancePaintCounts(el);
  const missing = [];
  if (probe.fillCount > known.fillCount) missing.push(`${probe.fillCount} fills detected, ${known.fillCount} exported`);
  if (probe.strokeCount > known.strokeCount) missing.push(`${probe.strokeCount} strokes detected, ${known.strokeCount} exported`);
  if (missing.length === 0) return null;
  const label = el && (el.name || el.id || el.parserId || el.layerName || el.layer_name) || "element";
  const missingFills = Math.max(0, Number(probe.fillCount || 0) - Number(known.fillCount || 0));
  const missingStrokes = Math.max(0, Number(probe.strokeCount || 0) - Number(known.strokeCount || 0));
  const fillLabel = `${known.fillCount} fill${known.fillCount === 1 ? "" : "s"}`;
  const strokeLabel = `${known.strokeCount} stroke${known.strokeCount === 1 ? "" : "s"}`;
  const missingParts = [];
  if (missingFills > 0) missingParts.push(`${missingFills} fill${missingFills === 1 ? "" : "s"}`);
  if (missingStrokes > 0) missingParts.push(`${missingStrokes} stroke${missingStrokes === 1 ? "" : "s"}`);
  return `${label}: native multi-fill/multi-stroke Appearance panel stack flattened (${missing.join("; ")}); ai-parser recovered ${fillLabel}/${strokeLabel} from ${probe.source}, but ${missingParts.join(" and ")} still need exact source paint data. Expand the appearance or improve parser paint recovery for this element before strict export.`;
}

function tuplePointInPolygon(point, polygon) {
  if (!point || !polygon || polygon.length < 3) return false;
  let inside = false;
  let prev = polygon[polygon.length - 1];
  for (const curr of polygon) {
    const crosses = (curr[1] > point[1]) !== (prev[1] > point[1]);
    if (crosses) {
      const xIntersection = (prev[0] - curr[0]) * (point[1] - curr[1]) / (prev[1] - curr[1]) + curr[0];
      if (point[0] < xIntersection) inside = !inside;
    }
    prev = curr;
  }
  return inside;
}

function tupleOrientation(a, b, c) {
  const value = (b[1] - a[1]) * (c[0] - b[0]) - (b[0] - a[0]) * (c[1] - b[1]);
  if (Math.abs(value) < 0.0001) return 0;
  return value > 0 ? 1 : 2;
}

function tupleOnSegment(a, b, c) {
  return b[0] <= Math.max(a[0], c[0]) + 0.0001
    && b[0] + 0.0001 >= Math.min(a[0], c[0])
    && b[1] <= Math.max(a[1], c[1]) + 0.0001
    && b[1] + 0.0001 >= Math.min(a[1], c[1]);
}

function tupleSegmentsIntersect(a, b, c, d) {
  const o1 = tupleOrientation(a, b, c);
  const o2 = tupleOrientation(a, b, d);
  const o3 = tupleOrientation(c, d, a);
  const o4 = tupleOrientation(c, d, b);
  if (o1 !== o2 && o3 !== o4) return true;
  if (o1 === 0 && tupleOnSegment(a, c, b)) return true;
  if (o2 === 0 && tupleOnSegment(a, d, b)) return true;
  if (o3 === 0 && tupleOnSegment(c, a, d)) return true;
  if (o4 === 0 && tupleOnSegment(c, b, d)) return true;
  return false;
}

function sameTuplePoint(a, b) {
  return !!a && !!b && Math.abs(a[0] - b[0]) < 0.0001 && Math.abs(a[1] - b[1]) < 0.0001;
}

function openContourPoints(points) {
  if (!points || points.length < 2) return points || [];
  return sameTuplePoint(points[0], points[points.length - 1]) ? points.slice(0, -1) : points;
}

function tupleSegments(points) {
  points = openContourPoints(points);
  const segments = [];
  for (let i = 0; i < points.length; i++) segments.push([points[i], points[(i + 1) % points.length]]);
  return segments;
}

function supportedEvenOddCompoundTopology(contours) {
  if (!contours || contours.length < 2) return false;
  contours = contours.map(contour => ({ ...contour, points: openContourPoints(contour.points) }));
  if (contours.some(contour => contour.closed === false || !contour.points || contour.points.length < 3)) return false;

  for (const contour of contours) {
    const segments = tupleSegments(contour.points);
    for (let i = 0; i < segments.length; i++) {
      for (let j = i + 1; j < segments.length; j++) {
        const adjacent = Math.abs(i - j) === 1 || (i === 0 && j === segments.length - 1);
        if (!adjacent && tupleSegmentsIntersect(segments[i][0], segments[i][1], segments[j][0], segments[j][1])) return false;
      }
    }
  }

  for (let i = 0; i < contours.length; i++) {
    for (let j = i + 1; j < contours.length; j++) {
      for (const [a, b] of tupleSegments(contours[i].points)) {
        for (const [c, d] of tupleSegments(contours[j].points)) {
          if (tupleSegmentsIntersect(a, b, c, d)) return false;
        }
      }

      const iInJ = contours[i].points.map(point => tuplePointInPolygon(point, contours[j].points));
      const jInI = contours[j].points.map(point => tuplePointInPolygon(point, contours[i].points));
      const iPartlyInJ = iInJ.some(Boolean) && !iInJ.every(Boolean);
      const jPartlyInI = jInI.some(Boolean) && !jInI.every(Boolean);
      if (iPartlyInJ || jPartlyInI) return false;
    }
  }
  return true;
}

function strictParityFailures(elements, options) {
  const failures = [];
  const parserFindings = parserParityFindings(options);
  for (const finding of parserFindings) {
    if (finding.status === "unsupported") failures.push({ id: "ai-parser", reason: finding.reason });
  }

  const walk = (els) => {
    for (const el of els) {
      const findings = parityFindingsForElement(el, options);
      for (const finding of findings) {
        if (finding.status === "unsupported") {
          failures.push({ id: el.id, reason: finding.reason });
        }
      }
      if (el.children) walk(el.children);
    }
  };
  walk(elements);
  return failures;
}

function parityFindingsForElement(el, options) {
  const findings = [];
  const type = sidecarType(el.type);
  const blendMode = normalizeBlendModeValue(el.blendMode) || String(el.blendMode || "normal").toLowerCase();
  const add = (status, reason) => findings.push({ status, reason });
  const parserExpandedVectorContract = hasParserExpandedVectorContract(el);

  if (el.rasterVectorized) {
    const origin = el.rasterSourceOrigin === "embedded" ? "embedded" : "linked";
    add("approximate", `${origin} raster traced into vector paths for code-only export`);
  }
  if (el.type === "image") {
    add("unsupported", rasterImageUnsupportedReason(el));
  }
  if (hasParserRecoveredVectors(el)) {
    add("approximate", "parser-backed flattened vector appearance");
  } else {
    if (el.type === "plugin") add("unsupported", "Illustrator plugin item exposes only bounds/metadata");
    if (el.type === "unknown") add("unsupported", "unknown Illustrator item exposes only bounds/metadata");
    if (el.type === "chart" || el.isChart) add("unsupported", "Illustrator chart/graph object exposes only bounds/metadata");
    if ((el.type === "mesh" || el.isGradientMesh) && !hasMeshPatches(el)) add("unsupported", "gradient mesh has no parsed mesh patches");
    if (el.envelope_mesh) add("unsupported", "envelope distort mesh requires expanded vector geometry");
    if (el.three_d) add("unsupported", "3D/extrude appearance requires expanded vector geometry");
    const unsupportedEffects = unsupportedLiveEffects(el);
    if (!parserExpandedVectorContract) {
      if (isStrictCodeOnly(options)) {
        const expansionNote = expansionFallbackFailureNote(el) || "GUI expansion unavailable and parser-expanded vectors missing";
        for (const effect of unsupportedEffects) add("unsupported", `${liveEffectGuidance(effect)}; ${expansionNote}`);
      } else if (hasAnyLiveEffect(el)) {
        for (const effect of unsupportedEffects) add("approximate", `live effect '${liveEffectDisplayName(effect)}' skipped in code export — appearance differs from Illustrator; ${liveEffectGuidance(effect)}`);
      }
    }
    if (el.isOpaque && !requiresOpaqueVectorRecovery(el)) add("unsupported", "opaque Illustrator effect requires parser-backed vector geometry");
  }
  const multiAppearanceReason = multiAppearanceFlattenReason(el);
  if (multiAppearanceReason) add(isStrictCodeOnly(options) ? "unsupported" : "approximate", multiAppearanceReason);
  if (el.isCompoundPath) {
    const contours = normalizeSubpaths(el);
    const fillRule = String(el.fillRule || "unknown").toLowerCase();
    if (contours.length <= 1) add("unsupported", "compound path has no extracted subpath boundary data");
    else if (!supportedEvenOddCompoundTopology(contours)) add("unsupported", "compound path has overlapping or intersecting contours beyond supported nested contour topology");
    else if (fillRule === "unknown") add("unsupported", "compound path fill rule is unavailable from host extraction; provide explicit ai-parser fill_rule metadata before export");
    else if (el.clipMask) {
      if (!fillRule.includes("even")) add("unsupported", "compound clipping mask with non-even-odd fill rule is not parity-safe yet");
      // else: valid even-odd compound clip mask — allowed when children remain code-renderable.
    } else if (!fillRule.includes("even") && compoundUsesPatternFill(el)) add("unsupported", `compound path pattern fill with fill rule ${fillRule} is not represented as parity-safe geometry`);
  }
  if (patternFillsMissingSwatch(el)) add(isStrictCodeOnly(options) ? "unsupported" : "approximate", "PatternColor fills use procedural placeholder colors; extract swatch artwork before strict exact export");
  else if (hasPatternFill(el) && !patternFillsHaveExactTileGeometry(el)) add(isStrictCodeOnly(options) ? "unsupported" : "approximate", patternFillsTileGeometryTruncated(el) ? "PatternColor fills use sampled swatch tile geometry but were truncated during extraction" : "PatternColor fills use sampled swatch colors with simplified tile geometry");
  if (patternStrokesMissingSwatch(el)) add(isStrictCodeOnly(options) ? "unsupported" : "approximate", "PatternColor strokes use procedural placeholder colors; extract swatch artwork before strict exact export");
  else if (hasPatternStroke(el) && !patternStrokesHaveExactTileGeometry(el)) add(isStrictCodeOnly(options) ? "unsupported" : "approximate", patternStrokesTileGeometryTruncated(el) ? "PatternColor strokes use sampled swatch tile geometry but were truncated during extraction" : "PatternColor strokes use sampled swatch colors with simplified tile geometry");
  if (isMixedClipGroup(el)) {
    const unsupportedClipReason = mixedClipUnsupportedReason(el);
    if (unsupportedClipReason) add("unsupported", unsupportedClipReason);
    else if (mixedClipHasUnvectorizedRasterChild(el)) {
      const rasterReason = mixedClipRasterVectorizationReason(el);
      if (rasterReason) add("unsupported", rasterReason);
    }
  }
  if ((type === "text" || type === "image") && blendMode && blendMode !== "normal") add("unsupported", `non-vector element with blend mode ${blendMode} requires scene-routed compositing before strict export`);
  if (el.type === "symbol") {
    const hasExpandedSymbolVectors = !!(el.children && el.children.length > 0);
    if (isStrictCodeOnly(options) && !hasExpandedSymbolVectors) {
      const expansionNote = expansionFallbackFailureNote(el) || "GUI expansion unavailable and parser-expanded vectors missing";
      add("unsupported", `unexpanded symbol requires expanded vector geometry; ${expansionNote}`);
    }
    else if (!hasExpandedSymbolVectors) add("approximate", "symbol instances preserve metadata; expand symbols for editable parity");
  }
  if (el.clipMask && !isMixedClipGroup(el)) {
    if (isStrictCodeOnly(options)) add("approximate", "vector clipping/masking is code-rendered and covered by visual fixtures (see visual_diff)");
    else add("approximate", "clipping masks are supported for vector scene groups but should be image-diff verified");
  }
  if (el.isGradientMesh && hasMeshPatches(el)) add("approximate", "gradient mesh patches rendered as code-generated mesh, covered by visual fixtures");
  if (el.parserOnly) add("approximate", "ai-parser-only vectors are code-drawn but lack Illustrator hierarchy/depth context until matched with DOM ordering");
  if (hasCodeRenderedIllustratorEffect(el)) add("approximate", "Illustrator effects are code-rendered as vector approximations (shadow/glow/blur/feather/bevel/noise)");

  // Advanced OpenType: feature flags preserved and bounded metrics applied.
  // Features needing glyph substitution/positioning are strict-unsupported until
  // text is outlined or a font-shaping asset pipeline is available.
  const shapingReasons = textShapingUnsupportedReasons(el);
  if (shapingReasons.length > 0) {
    const shapingStatus = isStrictCodeOnly(options) ? "unsupported" : "approximate";
    for (const reason of shapingReasons) add(shapingStatus, `advanced OpenType shaping: ${reason}`);
  } else if (!hasTextShapingContract(el) && !hasFontByteShapingContract(el)) {
    let hasBoundedTypographyApprox = hasTextTypographyOverrides(el) || hasTextTypographyOverrides(el.textStyle);
    if (!hasBoundedTypographyApprox && el.textRuns) {
      for (const run of el.textRuns) {
        if (hasTextTypographyOverrides(run.style)) { hasBoundedTypographyApprox = true; break; }
      }
    }
    if (hasBoundedTypographyApprox) add("approximate", "advanced typography metrics preserved with bounded code-rendered approximation");
  }

  return findings;
}

function parityStatusForElement(el, options) {
  let status = "exact";
  for (const finding of parityFindingsForElement(el, options)) status = mergeParityStatus(status, finding.status);
  for (const child of el.children || []) status = mergeParityStatus(status, parityStatusForElement(child, options));
  return status;
}

function parityReasonsForElement(el, options) {
  const reasons = parityFindingsForElement(el, options).map(finding => `[${finding.status}] ${finding.reason}`);
  for (const child of el.children || []) reasons.push(...parityReasonsForElement(child, options));
  return [...new Set(reasons)];
}

function parityStatusForElements(elements, options) {
  let status = "exact";
  for (const el of elements || []) status = mergeParityStatus(status, parityStatusForElement(el, options));
  return status;
}

function parserParityFindings(options) {
  const diagnostics = [
    ...((options && Array.isArray(options.parserDiagnostics)) ? options.parserDiagnostics : []),
    ...getAiParserDiagnostics(),
  ];
  const strict = isStrictCodeOnly(options);
  const status = strict ? "unsupported" : "approximate";
  const hasUnscopedParserElement = diagnostics.some(diagnostic => {
    const id = String(diagnostic.id || "").toLowerCase();
    const note = String(diagnostic.note || diagnostic.message || "").toLowerCase();
    return id === "ai-parser" && /(unscoped|missing artboard_name|missing artboard provenance)/.test(note);
  });
  const hasParserGap = diagnostics.some(diagnostic => {
    const id = String(diagnostic.id || "").toLowerCase();
    const note = String(diagnostic.note || diagnostic.message || "").toLowerCase();
    return id === "ai-parser" && /(skipped|unavailable|not found|failed|cannot|no document path)/.test(note);
  });
  if (hasUnscopedParserElement) {
    return [{ status, reason: "ai-parser artboard provenance missing; strict code-only export cannot merge unscoped parser vectors safely" }];
  }
  if (hasParserGap || (aiParserStatus.checked && !aiParserStatus.available)) {
    const reason = strict
      ? "ai-parser enrichment unavailable; strict code-only export requires parser-backed Illustrator appearance data"
      : "ai-parser enrichment unavailable; non-strict diagnostic export may be incomplete for parser-only Illustrator appearance data";
    return [{ status, reason }];
  }
  return [];
}

function generateSidecar(ab, els, colorMap, options) {
  const parserFindings = parserParityFindings(options);
  const mapElement = (el, childDepth) => {
    const parityReasons = [
      ...parityReasonsForElement(el, options),
      ...parserFindings.map(finding => `[${finding.status}] ${finding.reason}`),
    ];
    let elementParityStatus = parityStatusForElement(el, options);
    for (const finding of parserFindings) elementParityStatus = mergeParityStatus(elementParityStatus, finding.status);
    const result = {
      id: el.id, type: sidecarType(el.type), x: el.x, y: el.y, w: el.w, h: el.h, depth: childDepth !== undefined ? childDepth : el.depth,
      fill: colorToHex(el.fill),
      stroke: colorToHex(el.stroke),
      strokeWidth: el.stroke?.width || undefined,
      strokeGradient: mapGradientForSidecar(el.stroke && el.stroke.gradient),
      text: el.text || undefined,
      textStyle: textStyleForSidecar(el.textStyle),
      opacity: el.opacity !== 1 ? el.opacity : undefined, rotation: el.rotation !== 0 ? el.rotation : undefined,
      cornerRadius: el.cornerRadius > 0 ? el.cornerRadius : undefined,
      gradient: mapGradientForSidecar(el.gradient),
      blendMode: el.blendMode !== "normal" ? el.blendMode : undefined, strokeCap: el.strokeCap, strokeJoin: el.strokeJoin,
      strokeDash: el.strokeDash, strokeMiterLimit: el.strokeMiterLimit, strokeAlignment: el.strokeAlignment,
      appearanceProbe: el.appearanceProbe || el.appearance_probe || undefined,
      appearanceFills: el.appearance_fills && el.appearance_fills.length > 0 ? el.appearance_fills.map(f => ({ color: colorToHex(f.color || f), opacity: f.opacity, blendMode: f.blendMode || f.blend_mode, gradient: mapGradientForSidecar(f.gradient), pattern: mapGradientForSidecar(f.pattern) })) : undefined,
      appearanceStrokes: el.appearance_strokes && el.appearance_strokes.length > 0 ? el.appearance_strokes.map(s => ({ color: colorToHex(s.color || s), width: s.width, opacity: s.opacity, blendMode: s.blendMode || s.blend_mode, gradient: mapGradientForSidecar(s.gradient), pattern: mapGradientForSidecar(s.pattern), cap: s.cap, join: s.join, dash: s.dash, miterLimit: s.miterLimit, alignment: normalizeStrokeAlignment(s.alignment || s.strokeAlignment || s.stroke_align) })) : undefined,
      effects: (el.effects || []).length > 0 ? (el.effects || []).map(mapEffectForSidecar) : undefined,
      appearanceStack: el.appearanceStack || (
        (el.appearance_fills?.length || el.appearance_strokes?.length || el.effects?.length) ?
        [
          ...(el.appearance_fills || []).map(f => ({ type: 'fill', color: colorToHex(f.color || f), opacity: f.opacity, blendMode: f.blendMode || f.blend_mode, gradient: mapGradientForSidecar(f.gradient), pattern: mapGradientForSidecar(f.pattern) })),
          ...(el.effects || []).map(e => ({ ...mapEffectForSidecar(e), entryType: 'effect', effectType: e.type })),
          ...(el.appearance_strokes || []).map(s => ({ type: 'stroke', color: colorToHex(s.color || s), width: s.width, opacity: s.opacity, blendMode: s.blendMode || s.blend_mode, gradient: mapGradientForSidecar(s.gradient), pattern: mapGradientForSidecar(s.pattern), cap: s.cap, join: s.join, dash: s.dash, miterLimit: s.miterLimit, alignment: normalizeStrokeAlignment(s.alignment || s.strokeAlignment || s.stroke_align) }))
        ] : undefined
      ),
      textAlign: el.textAlign, letterSpacing: el.letterSpacing, lineHeight: el.lineHeight,
      textDecoration: el.textDecoration, textTransform: el.textTransform, textRuns: el.textRuns,
      shapedGlyphs: shapedGlyphsForSidecar(el.shapedGlyphs),
      outlinedGlyphs: shapedGlyphsForSidecar(el.outlinedGlyphs),
      openTypeFeatures: el.openTypeFeatures || undefined,
      baselineShift: el.baselineShift || undefined, horizontalScale: el.horizontalScale || undefined,
      verticalScale: el.verticalScale || undefined,
      clipChildren: el.clipMask || undefined, symbolName: el.symbolName, isCompoundPath: el.isCompoundPath || undefined,
      fillRule: el.fillRule && el.fillRule !== "nonzero" ? el.fillRule : undefined,
      isGradientMesh: el.isGradientMesh || undefined, isChart: el.isChart || undefined,
      meshPatches: el.mesh_patches || undefined,
      thirdPartyEffects: el.thirdPartyEffects && el.thirdPartyEffects.length > 0 ? el.thirdPartyEffects : undefined,
      isOpaque: el.isOpaque || undefined, notes: el.notes && el.notes.length > 0 ? el.notes : undefined,
      pathPoints: el.pathPoints ? el.pathPoints.map(p => ({ ...p, left_ctrl: p.leftDir, right_ctrl: p.rightDir })) : undefined, pathClosed: el.pathClosed || undefined,
      subpaths: Array.isArray(el.subpaths) && el.subpaths.length > 0
        ? el.subpaths.map(subpath => ({
            points: (subpath.points || []).map(p => ({ ...p, left_ctrl: p.leftDir, right_ctrl: p.rightDir })),
            closed: subpath.closed !== false,
          }))
        : undefined,
      imagePath: el.imagePath ? portableAssetPath(el.imagePath) : undefined,
      embeddedRaster: el.embeddedRaster || undefined,
      parityStatus: elementParityStatus,
      parityReasons: parityReasons.length > 0 ? parityReasons : undefined,
    };
    // Preserve full nesting — recursively map children
    if (el.children?.length > 0) {
      result.children = el.children.map(ch => mapElement(ch, (childDepth !== undefined ? childDepth : el.depth) + 1));
    }
    return result;
  };
  let artboardParityStatus = parityStatusForElements(els, options);
  for (const finding of parserFindings) artboardParityStatus = mergeParityStatus(artboardParityStatus, finding.status);
  return JSON.stringify({
    artboard: {
      name: ab.name,
      width: ab.width,
      height: ab.height,
      parityStatus: artboardParityStatus,
      parityReasons: parserFindings.length > 0 ? parserFindings.map(finding => `[${finding.status}] ${finding.reason}`) : undefined,
    },
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
    if (platform === "linux") return "linux";
    return "unsupported";
}

function getAiParserCandidates(pluginDir, platformValue) {
    const path = getNodeModule("path") || { join: (...args) => args.join("/").replace(/\/+/g, "/") };
    const platformDir = getAiParserPlatformDir(platformValue);
    if (platformDir === "unsupported") return [];
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

function resetAiParserStateForTests() {
    aiParserStatus.checked = false;
    aiParserStatus.available = false;
    aiParserStatus.binaryPath = null;
    aiParserStatus.diagnostics = [];
    unscopedAiParserDiagnosticKeys = new Set();
}

function runAiParserCommand(args, label) {
    const binaryPath = findAiParserBinary();
    if (!binaryPath) return null;
    const childProcess = getNodeModule("child_process");
    if (!childProcess || typeof childProcess.execFileSync !== "function") {
        aiParserStatus.available = false;
        noteAiParserDiagnostic("Cannot run bundled ai-parser", "child_process.execFileSync unavailable in this host");
        return null;
    }

    try {
        const output = childProcess.execFileSync(binaryPath, args, {
            encoding: "utf8",
            maxBuffer: AI_PARSER_MAX_BUFFER_BYTES,
            timeout: 15000,
            windowsHide: true
        });
        aiParserStatus.available = true;
        aiParserStatus.binaryPath = binaryPath;
        return JSON.parse(output);
    } catch (e) {
        aiParserStatus.available = false;
        noteAiParserDiagnostic(`ai-parser failed for ${label || args[0] || "command"}`, e);
        return null;
    }
}

async function runAiParser(filePath) {
    if (!filePath) {
        aiParserStatus.available = false;
        noteAiParserDiagnostic("Cannot run ai-parser", "No Illustrator document path was available");
        return null;
    }
    return runAiParserCommand([filePath, "--pretty"], basename(filePath));
}

function rasterVectorSourcePath(el) {
    if (!el) return null;
    return el.extractedImagePath || el.imagePath || el.vectorSourcePath || el.sourcePath || null;
}

function shouldBakeRasterRotation(el) {
    return !!(el && !el.extractedRasterAlreadyTransformed && Math.abs(Number(el.rotation || 0)) > 0.001);
}

function isOrthogonalRasterRotation(el) {
    const normalized = Math.abs(Number(el && el.rotation || 0)) % 180;
    return normalized <= 0.001 || Math.abs(normalized - 90) <= 0.001;
}

function hasRasterTransformScale(el) {
    const scaleX = Number(el && el.rasterScaleX || 0);
    const scaleY = Number(el && el.rasterScaleY || 0);
    return Number.isFinite(scaleX) && scaleX > 0 && Number.isFinite(scaleY) && scaleY > 0;
}

function canBakeRasterRotation(el) {
    if (!shouldBakeRasterRotation(el)) return true;
    return isOrthogonalRasterRotation(el) || hasRasterTransformScale(el);
}

function isSafeRasterVectorEffect(effect) {
    const type = String(effect && (effect.type || effect.effectType || effect.effect_type) || "").toLowerCase();
    return [
        "dropshadow", "drop-shadow",
        "innershadow", "inner-shadow",
        "outerglow", "outer-glow",
        "innerglow", "inner-glow",
        "gaussianblur", "gaussian-blur",
        "feather",
        "bevel",
        "noise", "grain", "mezzotint",
    ].includes(type);
}

function rasterHasUnsafeVectorEffects(el) {
    const effects = Array.isArray(el && el.effects) ? el.effects : [];
    return effects.some(effect => !isSafeRasterVectorEffect(effect));
}

async function runAiParserVectorizeImage(el, artboardName) {
    const sourcePath = rasterVectorSourcePath(el);
    if (!el || !sourcePath) return null;
    const args = [
        "--vectorize-image", String(sourcePath),
        "--id", String(el.id || (el.embeddedRaster ? "embedded_raster" : "linked_raster")),
        "--x", String(Number(el.x || 0)),
        "--y", String(Number(el.y || 0)),
        "--w", String(Number(el.w || 0)),
        "--h", String(Number(el.h || 0)),
        "--pretty"
    ];
    if (shouldBakeRasterRotation(el)) args.push("--rotation-deg", String(Number(el.rotation || 0)));
    if (hasRasterTransformScale(el)) {
        args.push("--scale-x", String(Number(el.rasterScaleX || 1)));
        args.push("--scale-y", String(Number(el.rasterScaleY || 1)));
    }
    if (artboardName) args.push("--artboard", String(artboardName));
    const label = el.embeddedRaster ? "embedded raster vectorization" : basename(sourcePath);
    return runAiParserCommand(args, label);
}

function effectsFromLiveEffects(liveEffects) {
    const out = [];
    for (const fx of liveEffects || []) {
        const name = String(fx.name || fx.type || "").toLowerCase();
        const params = fx.params && fx.params.params ? fx.params.params : (fx.params || {});
        const normalized = rasterEffectTypeFromName(name);
        if (normalized === "noise") {
            out.push({
                type: "noise",
                amount: Number(params.amount ?? params.opacity ?? params.intensity ?? 0.16),
                scale: Number(params.scale ?? params.size ?? params.cellSize ?? 2),
                seed: Number(params.seed ?? 0)
            });
        } else if (normalized === "gaussianBlur") {
            out.push({ type: "gaussianBlur", radius: Number(params.radius ?? params.blur ?? 4) });
        } else if (normalized === "dropShadow") {
            out.push({ type: "dropShadow", x: Number(params.x ?? params.horz ?? 0), y: Number(params.y ?? params.vert ?? 0), blur: Number(params.blur ?? params.radius ?? 4), spread: Number(params.spread ?? 0), color: params.color || { r: 0, g: 0, b: 0, a: 1 } });
        } else if (normalized === "innerShadow") {
            out.push({ type: "innerShadow", x: Number(params.x ?? params.horz ?? 0), y: Number(params.y ?? params.vert ?? 0), blur: Number(params.blur ?? params.radius ?? 4), color: params.color || { r: 0, g: 0, b: 0, a: 1 } });
        } else if (normalized === "outerGlow" || normalized === "innerGlow") {
            out.push({ type: normalized, blur: Number(params.blur ?? params.radius ?? 4), color: params.color || { r: 255, g: 255, b: 255, a: 1 } });
        } else if (normalized === "feather") {
            out.push({ type: "feather", radius: Number(params.radius ?? params.blur ?? 4) });
        } else if (normalized === "bevel") {
            out.push({ type: "bevel", depth: Number(params.depth ?? 2), angle: Number(params.angle ?? 135), radius: Number(params.radius ?? params.blur ?? 1), highlight: params.highlight, shadowColor: params.shadowColor || params.shadow });
        } else {
            const displayName = fx.name || fx.type || "liveEffect";
            out.push({ type: "liveEffect", name: displayName, category: classifyLiveEffectCategory(displayName), params });
        }
    }
    return out;
}

function aiParserRootElements(aiParserResult) {
    if (!aiParserResult) return [];
    if (Array.isArray(aiParserResult)) {
        return aiParserResult.flatMap(entry => Array.isArray(entry.elements) ? entry.elements : []);
    }
    return Array.isArray(aiParserResult.elements) ? aiParserResult.elements : [];
}

function annotateAiParserElement(aiEl, depth, parentId, order) {
    const annotated = {
        ...aiEl,
        __parser_depth: depth,
        __parser_parent_id: parentId || null,
        __parser_order: order,
    };
    if (Array.isArray(aiEl.children)) {
        annotated.children = aiEl.children.map((child, index) =>
            annotateAiParserElement(child, depth + 1, aiEl.id || parentId || null, index)
        );
    }
    return annotated;
}

function flattenAnnotatedAiParserElement(aiEl, out) {
    out.push(aiEl);
    for (const child of aiEl.children || []) flattenAnnotatedAiParserElement(child, out);
}

function flattenAiParserElements(aiParserResult) {
    const out = [];
    const roots = aiParserRootElements(aiParserResult).map((el, index) => annotateAiParserElement(el, 0, null, index));
    for (const root of roots) flattenAnnotatedAiParserElement(root, out);
    return out;
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

function findAiParserMatch(domElement, parserElements, usedIds, artboardName) {
    const isCandidateAvailable = (el) => !(el.id && usedIds.has(el.id)) && parserElementBelongsToArtboard(el, artboardName);

    const exactId = parserElements.find(el => isCandidateAvailable(el) && el.id && el.id === domElement.id);
    if (exactId) return exactId;

    const exactName = parserElements.find(el => isCandidateAvailable(el) && el.name && el.name === domElement.id);
    if (exactName) return exactName;

    let best = null;
    let bestScore = Number.POSITIVE_INFINITY;
    for (const candidate of parserElements) {
        if (!isCandidateAvailable(candidate)) continue;
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

function parserBounds(aiEl) {
    if (Array.isArray(aiEl.bounds) && aiEl.bounds.length >= 4) {
        return aiEl.bounds.map(value => Number(value) || 0);
    }
    const pathPoints = parserPathPoints(aiEl) || [];
    if (!pathPoints.length) {
        const childBounds = (aiEl.children || [])
            .map(parserBounds)
            .filter(bounds => Array.isArray(bounds) && bounds.length >= 4);
        if (childBounds.length > 0) {
            const minX = Math.min(...childBounds.map(bounds => bounds[0]));
            const minY = Math.min(...childBounds.map(bounds => bounds[1]));
            const maxX = Math.max(...childBounds.map(bounds => bounds[0] + bounds[2]));
            const maxY = Math.max(...childBounds.map(bounds => bounds[1] + bounds[3]));
            return [minX, minY, Math.max(0, maxX - minX), Math.max(0, maxY - minY)];
        }
        noteAiParserDiagnostic("Skipped bounds-less ai-parser element", aiEl.id || aiEl.name || "unknown");
        return null;
    }
    const xs = [];
    const ys = [];
    for (const point of pathPoints) {
        for (const coord of [point.anchor, point.leftDir, point.rightDir]) {
            if (Array.isArray(coord)) {
                xs.push(Number(coord[0]) || 0);
                ys.push(Number(coord[1]) || 0);
            }
        }
    }
    const minX = Math.min(...xs);
    const minY = Math.min(...ys);
    const maxX = Math.max(...xs);
    const maxY = Math.max(...ys);
    return [minX, minY, Math.max(1, maxX - minX), Math.max(1, maxY - minY)];
}

function normalizedArtboardName(name) {
    const normalized = String(name || "").trim().toLowerCase().replace(/\s+/g, "_");
    const numbered = normalized.match(/^(?:artboard|page)_?0*(\d+)$/);
    return numbered ? `artboard_${Number(numbered[1])}` : normalized;
}

function noteUnscopedAiParserElement(aiEl, artboardName) {
    const descriptor = aiEl.id || aiEl.name || (Array.isArray(aiEl.bounds) ? `bounds:${aiEl.bounds.join(",")}` : "unknown");
    const key = `${normalizedArtboardName(artboardName)}:${descriptor}`;
    if (unscopedAiParserDiagnosticKeys.has(key)) return;
    unscopedAiParserDiagnosticKeys.add(key);
    noteAiParserDiagnostic("Skipped unscoped ai-parser element", `Missing artboard_name for ${descriptor} while exporting ${artboardName}`);
}

function parserElementBelongsToArtboard(aiEl, artboardName) {
    if (!artboardName) return true;
    if (!aiEl.artboard_name) {
        noteUnscopedAiParserElement(aiEl, artboardName);
        return false;
    }
    return normalizedArtboardName(aiEl.artboard_name) === normalizedArtboardName(artboardName);
}

function parserElementToDomElement(aiEl, artboardName) {
    const bounds = parserBounds(aiEl);
    if (!bounds) return null;
    const [x, y, w, h] = bounds;
    const pathPoints = parserPathPoints(aiEl);
    const children = (aiEl.children || [])
        .filter(child => parserElementBelongsToArtboard(child, artboardName) && parserElementHasSafeVectorPaint(child))
        .map(child => parserElementToDomElement(child, artboardName))
        .filter(Boolean);
    const liveEffects = effectsFromLiveEffects(aiEl.live_effects || []);
    const parserFlags = parserExpansionFlags(aiEl);
    const parserRecovered = parserFlags.parserRecovered || (parserFlags.expandedChildren && children.length > 0);
    const appearanceExpanded = parserFlags.appearanceExpanded && (children.length > 0 || (aiEl.appearanceStack && aiEl.appearanceStack.length > 0) || (aiEl.appearance_stack && aiEl.appearance_stack.length > 0));
    const symbolExpanded = parserFlags.symbolExpanded && children.length > 0;
    const parserType = aiEl.element_type || aiEl.type;
    const domType = pathPoints && pathPoints.length >= 2
        ? "path"
        : (children.length > 0 ? "group" : (parserType || "shape"));
    return {
        id: aiEl.id || `parser_${Math.round(x)}_${Math.round(y)}`,
        parserId: aiEl.id,
        parserOnly: true,
        parserRecovered: parserRecovered || undefined,
        appearanceExpanded: appearanceExpanded || undefined,
        symbolExpanded: symbolExpanded || undefined,
        artboardName: aiEl.artboard_name || artboardName,
        type: domType,
        x,
        y,
        w,
        h,
        depth: Number.isFinite(Number(aiEl.depth)) ? Number(aiEl.depth) : Number(aiEl.__parser_depth || 0),
        zOrder: Number.isFinite(Number(aiEl.z_order)) ? Number(aiEl.z_order) : Number(aiEl.__parser_order || 0),
        layerName: aiEl.layer_name || aiEl.layer || undefined,
        parentId: aiEl.parent_id || aiEl.__parser_parent_id || undefined,
        fill: null,
        stroke: null,
        opacity: 1,
        rotation: Number(aiEl.rotation_deg || 0),
        scaleX: Number(aiEl.scale_x || 1),
        scaleY: Number(aiEl.scale_y || 1),
        translateX: Number(aiEl.translate_x || 0),
        translateY: Number(aiEl.translate_y || 0),
        cornerRadius: Number(aiEl.corner_radius || 0),
        pathPoints,
        pathClosed: aiEl.path_closed !== undefined ? !!aiEl.path_closed : false,
        fillRule: aiEl.fill_rule || aiEl.fillRule || (aiEl.subpaths && aiEl.subpaths.length > 1 ? "unknown" : undefined),
        subpaths: aiEl.subpaths || undefined,
        live_effects: aiEl.live_effects?.length ? aiEl.live_effects : undefined,
        effects: (parserRecovered || appearanceExpanded || symbolExpanded) ? [] : (liveEffects.length ? liveEffects : []),
        appearance_fills: aiEl.appearance_fills || [],
        appearance_strokes: aiEl.appearance_strokes || [],
        mesh_patches: aiEl.mesh_patches || [],
        envelope_mesh: aiEl.envelope_mesh,
        three_d: aiEl.three_d,
        notes: [],
        children
    };
}

function parserElementHasSafeVectorPaint(aiEl) {
    if ((aiEl.children || []).some(parserElementHasSafeVectorPaint)) return true;
    const pathPoints = parserPathPoints(aiEl);
    if (aiEl.mesh_patches?.length) return true;
    if (!pathPoints || pathPoints.length < 2) return false;
    if (aiEl.appearance_strokes?.length) return true;
    if (aiEl.appearance_fills?.length) return !!(aiEl.path_closed && pathPoints.length >= 3);
    return false;
}

function markAiParserElementUsed(aiEl, usedIds) {
    if (aiEl.id) usedIds.add(aiEl.id);
    for (const child of aiEl.children || []) markAiParserElementUsed(child, usedIds);
}

function parserElementOverlapsDomBounds(domElement, parserElement) {
    const bounds = parserBounds(parserElement);
    if (!bounds) return false;
    const [cx, cy, cw, ch] = bounds;
    const dx = Number(domElement.x || 0);
    const dy = Number(domElement.y || 0);
    const dw = Number(domElement.w || 0);
    const dh = Number(domElement.h || 0);
    if (cw <= 0 || ch <= 0 || dw <= 0 || dh <= 0) return false;

    const centerX = cx + cw / 2;
    const centerY = cy + ch / 2;
    const pad = 1;
    const centerInBounds = centerX >= dx - pad && centerX <= dx + dw + pad && centerY >= dy - pad && centerY <= dy + dh + pad;
    if (!centerInBounds) return false;

    const ix = Math.max(0, Math.min(cx + cw, dx + dw) - Math.max(cx, dx));
    const iy = Math.max(0, Math.min(cy + ch, dy + dh) - Math.max(cy, dy));
    const overlap = ix * iy;
    if (overlap <= 0) return false;

    const candidateArea = Math.max(1, cw * ch);
    const domArea = Math.max(1, dw * dh);
    return overlap / candidateArea >= 0.5 && overlap / domArea >= 0.0005;
}

function mergeAiParserData(domElements, aiParserResult, artboardName) {
    const parserElements = flattenAiParserElements(aiParserResult);
    if (!parserElements.length) return domElements;
    const usedIds = new Set();

    const mergeElement = (el) => {
        const children = (el.children || []).map(mergeElement);
        const base = { ...el, children };

        if (requiresOpaqueVectorRecovery(base) || base.isOpaque) {
            const recoveredChildren = [];
            for (const candidate of parserElements) {
                if (candidate.id && usedIds.has(candidate.id)) continue;
                if (candidate.is_pseudo_element) continue;
                if (!parserElementBelongsToArtboard(candidate, artboardName)) continue;
                if (!parserElementHasSafeVectorPaint(candidate)) continue;
                if (!parserElementOverlapsDomBounds(base, candidate)) continue;
                const recoveredChild = parserElementToDomElement(candidate, artboardName);
                if (recoveredChild) recoveredChildren.push(recoveredChild);
            }

            if (recoveredChildren.length > 0) {
                for (const child of recoveredChildren) {
                    markAiParserElementUsed(child, usedIds);
                }
                return {
                    ...base,
                    type: 'group',
                    originalType: base.type,
                    parserRecovered: true,
                    appearanceExpanded: false,
                    symbolExpanded: false,
                    fill: null,
                    stroke: null,
                    gradient: null,
                    appearanceProbe: null,
                    appearance_probe: null,
                    appearance_fills: [],
                    appearance_strokes: [],
                    appearanceStack: undefined,
                    effects: [],
                    children: recoveredChildren
                };
            }
        }

        const aiEl = findAiParserMatch(base, parserElements, usedIds, artboardName);
        if (!aiEl) return base;
        markAiParserElementUsed(aiEl, usedIds);

        const liveEffects = effectsFromLiveEffects(aiEl.live_effects || []);
        const pathPoints = parserPathPoints(aiEl);
        const parserChildren = (aiEl.children || [])
            .filter(child => parserElementBelongsToArtboard(child, artboardName) && parserElementHasSafeVectorPaint(child))
            .map(child => parserElementToDomElement(child, artboardName))
            .filter(Boolean);
        const parserFlags = parserExpansionFlags(aiEl);
        return {
            ...base,
            parserId: aiEl.id,
            parserRecovered: parserFlags.parserRecovered || undefined,
            appearanceExpanded: parserFlags.appearanceExpanded || undefined,
            symbolExpanded: parserFlags.symbolExpanded || undefined,
            artboardName: aiEl.artboard_name || base.artboardName,
            layerName: aiEl.layer_name || aiEl.layer || base.layerName,
            zOrder: Number.isFinite(Number(aiEl.z_order)) ? Number(aiEl.z_order) : base.zOrder,
            rotation: Number.isFinite(Number(aiEl.rotation_deg)) && Number(aiEl.rotation_deg) !== 0 ? Number(aiEl.rotation_deg) : base.rotation,
            scaleX: Number.isFinite(Number(aiEl.scale_x)) && Number(aiEl.scale_x) !== 0 ? Number(aiEl.scale_x) : base.scaleX,
            scaleY: Number.isFinite(Number(aiEl.scale_y)) && Number(aiEl.scale_y) !== 0 ? Number(aiEl.scale_y) : base.scaleY,
            translateX: Number.isFinite(Number(aiEl.translate_x)) ? Number(aiEl.translate_x) : base.translateX,
            translateY: Number.isFinite(Number(aiEl.translate_y)) ? Number(aiEl.translate_y) : base.translateY,
            cornerRadius: Number(aiEl.corner_radius || 0) > 0 ? Number(aiEl.corner_radius) : base.cornerRadius,
            pathPoints: pathPoints || base.pathPoints,
            pathClosed: aiEl.path_closed !== undefined ? !!aiEl.path_closed : base.pathClosed,
            fillRule: aiEl.fill_rule || aiEl.fillRule || ((aiEl.subpaths && aiEl.subpaths.length > 1) || base.isCompoundPath ? "unknown" : base.fillRule),
            subpaths: aiEl.subpaths?.length ? aiEl.subpaths : base.subpaths,
            live_effects: aiEl.live_effects?.length ? aiEl.live_effects : undefined,
            effects: (parserFlags.parserRecovered || parserFlags.appearanceExpanded || parserFlags.symbolExpanded) ? [] : (liveEffects.length ? [...(base.effects || []), ...liveEffects] : base.effects),
            appearance_fills: aiEl.appearance_fills?.length ? aiEl.appearance_fills : base.appearance_fills,
            appearance_strokes: aiEl.appearance_strokes?.length ? aiEl.appearance_strokes : base.appearance_strokes,
            mesh_patches: aiEl.mesh_patches?.length ? aiEl.mesh_patches : base.mesh_patches,
            envelope_mesh: aiEl.envelope_mesh || base.envelope_mesh,
            three_d: aiEl.three_d || base.three_d,
            children: parserChildren.length ? parserChildren : base.children,
        };
    };

    const merged = domElements.map(mergeElement);
    for (const aiEl of parserElements) {
        if (aiEl.id && usedIds.has(aiEl.id)) continue;
        if (aiEl.__parser_parent_id) continue;
        if (aiEl.is_pseudo_element) continue;
        if (!parserElementBelongsToArtboard(aiEl, artboardName)) continue;
        if (!parserElementHasSafeVectorPaint(aiEl)) continue;
        const parserDomElement = parserElementToDomElement(aiEl, artboardName);
        if (parserDomElement) {
            merged.push(parserDomElement);
            markAiParserElementUsed(aiEl, usedIds);
        }
    }
    return merged;
}

async function vectorizeRasterElement(el, artboardName, vectorizeFn) {
    if (!el || el.type !== "image") return el;
    const sourcePath = rasterVectorSourcePath(el);
    if (!sourcePath) return el;
    if (rasterHasUnsafeVectorEffects(el)) return el;
    if (!canBakeRasterRotation(el)) return el;
    let result = null;
    try {
        result = await (vectorizeFn || runAiParserVectorizeImage)(el, artboardName);
    } catch (e) {
        noteAiParserDiagnostic(el.embeddedRaster ? "embedded raster vectorization failed" : "linked raster vectorization failed", e);
        return { ...el, vectorizationFailed: true };
    }
    const vectorChildren = aiParserRootElements(result)
        .filter(candidate => parserElementBelongsToArtboard(candidate, artboardName) && parserElementHasSafeVectorPaint(candidate))
        .map(candidate => parserElementToDomElement(candidate, artboardName))
        .filter(Boolean);
    if (vectorChildren.length === 0) return { ...el, vectorizationFailed: true };
    const origin = el.embeddedRaster ? "embedded" : "linked";
    const rotation = Number(el.rotation || 0);
    const center = { x: Number(el.x || 0) + Number(el.w || 0) / 2, y: Number(el.y || 0) + Number(el.h || 0) / 2 };
    const transformedChildren = shouldBakeRasterRotation(el)
        ? vectorChildren.map(child => rotateVectorElementGeometry(child, center, rotation))
        : vectorChildren;
    const childBounds = boundsFromTuples(transformedChildren.flatMap(child => rectCornerTuples(child)));
    return {
        ...el,
        ...(childBounds || {}),
        type: "group",
        originalType: "image",
        rasterVectorized: true,
        rasterSourceOrigin: origin,
        rotation: 0,
        imagePath: null,
        extractedImagePath: null,
        embeddedRaster: false,
        fill: null,
        stroke: null,
        gradient: null,
        effects: Array.isArray(el.effects) ? el.effects.slice() : [],
        children: transformedChildren,
        notes: [...(el.notes || []), `${origin} raster vectorized to code-only paths`]
    };
}

async function vectorizeRasterChildrenInClipGroups(elements, artboardName, vectorizeFn, inClipGroup = false) {
    const out = [];
    for (const el of elements || []) {
        const nextInClipGroup = inClipGroup || !!el.clipMask;
        const children = el.children?.length ? await vectorizeRasterChildrenInClipGroups(el.children, artboardName, vectorizeFn, nextInClipGroup) : [];
        const base = { ...el, children };
        if (nextInClipGroup && el.type === "image") out.push(await vectorizeRasterElement(base, artboardName, vectorizeFn));
        else out.push(base);
    }
    return out;
}

async function vectorizeRasterElements(elements, artboardName, vectorizeFn) {
    const out = [];
    for (const el of elements || []) {
        const children = el.children?.length ? await vectorizeRasterElements(el.children, artboardName, vectorizeFn) : [];
        const base = { ...el, children };
        out.push(await vectorizeRasterElement(base, artboardName, vectorizeFn));
    }
    return out;
}

async function vectorizeRasterImagesForResults(artboardsData, vectorizeFn) {
    for (const result of artboardsData || []) {
        result.elements = await vectorizeRasterChildrenInClipGroups(result.elements || [], result.artboard?.name, vectorizeFn);
        result.elements = await vectorizeRasterElements(result.elements || [], result.artboard?.name, vectorizeFn);
    }
    return artboardsData;
}

async function vectorizeLinkedRastersForResults(artboardsData) {
    return vectorizeRasterImagesForResults(artboardsData);
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
            const artboardName = artboard.artboard?.name || artboard.name;
            artboard.elements = mergeAiParserData(artboard.elements, aiParserResult, artboardName);
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
  await vectorizeRasterImagesForResults(results);

  // Re-collect all elements after potential enrichment
  allEls.length = 0;
  for (const r of results) allEls.push(...r.elements);

  if (isStrictCodeOnly(options)) {
    const failures = strictParityFailures(allEls, options);
    if (failures.length > 0) {
      const msgs = failures.map(f => `${f.id}: [unsupported] ${f.reason}`).join("; ");
      throw new Error(`Cannot export code-only Rust: ${msgs}. Remove unsupported Illustrator content or convert it to supported vector/text geometry.`);
    }
  }

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
    if (options?.sidecar || options?.includeSidecar) files[`${sn}.json`] = generateSidecar(r.artboard, r.elements, colorMap, options);
  }

  const assets = {};

  let zipBlob = null;
  if (typeof JSZip !== "undefined") { const zip = new JSZip(); for (const [fn, ct] of Object.entries(files)) zip.file(fn, ct); zipBlob = await zip.generateAsync({ type: "blob" }); }
  return { files, assets, zipBlob, colorMap: Object.fromEntries(colorMap), warnings: collectWarnings(allEls, options) };
}

function exportFromRawData(results, options) {
  const allEls = [];
  for (const r of results) allEls.push(...r.elements);

  if (isStrictCodeOnly(options)) {
    const failures = strictParityFailures(allEls, options);
    if (failures.length > 0) {
      const msgs = failures.map(f => `${f.id}: [unsupported] ${f.reason}`).join("; ");
      throw new Error(`Cannot export code-only Rust: ${msgs}. Remove unsupported Illustrator content or convert it to supported vector/text geometry.`);
    }
  }

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
    if (options?.sidecar || options?.includeSidecar) files[`${sn}.json`] = generateSidecar(r.artboard, r.elements, colorMap, options);
  }

  const assets = {};

  return { files, assets, colorMap: Object.fromEntries(colorMap), warnings: collectWarnings(allEls, options) };
}

function collectWarnings(elements, options) {
  const warnings = [];
  const addWarning = warning => warnings.push(sanitizeDiagnosticEntry(warning));
  if (options && Array.isArray(options.parserDiagnostics)) for (const diagnostic of options.parserDiagnostics) addWarning(diagnostic);
  for (const diagnostic of getAiParserDiagnostics()) addWarning(diagnostic);
  for (const finding of parserParityFindings(options)) addWarning({ id: "ai-parser", parityStatus: finding.status, note: `[${finding.status}] ${finding.reason}` });
  for (const diagnostic of consumeExtractionDiagnostics()) addWarning(diagnostic);
  const walk = (els) => { for (const el of els) {
    for (const finding of parityFindingsForElement(el, options)) addWarning({ id: el.id, parityStatus: finding.status, note: `[${finding.status}] ${finding.reason}` });
    if (el.type === "mesh" || el.isGradientMesh) addWarning({ id: el.id, note: "Gradient mesh — emitted as editable mesh patches when patches are available" });
    if (el.type === "chart" || el.isChart) addWarning({ id: el.id, note: "Chart/graph — preserved bounds/metadata" });
    if (el.type === "image") {
      const suffix = el.imagePath ? `: ${portableAssetPath(el.imagePath) || basename(el.imagePath)}` : "";
      addWarning({ id: el.id, parityStatus: "unsupported", note: `[unsupported] ${rasterImageUnsupportedReason(el)}${suffix}` });
    }
    if (el.clipMask) addWarning({ id: el.id, note: "Clipping mask — emitted through shape/stencil clipping primitive metadata" });
    if (el.blendMode && el.blendMode !== "normal") addWarning({ id: el.id, note: `Blend mode ${el.blendMode} — emitted through compositing primitive metadata` });
    if (el.thirdPartyEffects?.length > 0) for (const fx of el.thirdPartyEffects) addWarning({ id: el.id, note: fx.note });
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
    AI_PARSER_MAX_BUFFER_BYTES,
    portableAssetPath,
    getAiParserCandidates,
    getAiParserPlatformDir,
    mergeAiParserData,
    collectWarnings,
    isTrustedPanelMessage,
    getLocalTargetOrigin,
    applyBlendExpr,
    normalizeBlendModeValue,
    getGradient,
    generateSidecar,
    exportFromRawData,
    illustratorTrackingToPx,
    illustratorLeadingToMultiplier,
    appearanceProbeFromMetadataText,
    getTextAlign,
    normalizeStrokeAlignment,
    parityStatusForElement,
    parityFindingsForElement,
    rasterVectorSourcePath,
    vectorizeRasterElement,
    vectorizeRasterImagesForResults,
    resetAiParserStateForTests,
  };
}

// ─── CEP ExtendScript Entry Points ──────────────────────────────────────
// These functions are called from index.html via CSInterface.evalScript()
