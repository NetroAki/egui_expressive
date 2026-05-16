// host.jsx — ExtendScript entry points for CEP mode
// This file runs in Illustrator's ExtendScript engine where `app` is available.
// It provides the Illustrator DOM access that the CEP panel needs.
// The code generation logic lives in plugin.js (browser-side).

if (typeof JSON !== 'object') {
    JSON = {};
}

var __eguiHostDiagnostics = [];

function sanitizeHostDiagnosticText(value) {
    return String(value || "")
        .replace(/[A-Za-z]:[\\\/][^\n\r;)]*egui_expressive_raster_trace[\\\/][^\s;),]+/g, "[temporary raster extraction input]")
        .replace(/(?:\/|\\\\)[^\n\r;)]*egui_expressive_raster_trace[\\\/][^\s;),]+/g, "[temporary raster extraction input]")
        .replace(/egui_expressive_raster_trace[\\\/][^\s;),]+/g, "egui_expressive_raster_trace/[temporary input]");
}

function noteHostDiagnostic(context, error) {
    try {
        var message = sanitizeHostDiagnosticText(context + ": " + (error && error.message ? error.message : String(error)));
        if (__eguiHostDiagnostics.length < 200) {
            __eguiHostDiagnostics.push({ id: "host", note: message });
        }
    } catch (ignored) {
        // Last-resort guard: diagnostics must never break export.
    }
}

function consumeHostDiagnostics() {
    var out = __eguiHostDiagnostics.slice(0);
    __eguiHostDiagnostics = [];
    return out;
}

function ensureElementNotes(el) {
    if (!el) return [];
    if (!el.notes || Object.prototype.toString.call(el.notes) !== '[object Array]') el.notes = [];
    return el.notes;
}

function safeTempFileSlug(value) {
    var raw = String(value || "embedded_raster");
    var slug = raw.replace(/[^A-Za-z0-9._-]/g, "_").replace(/^_+|_+$/g, "");
    return slug || "embedded_raster";
}

function rasterExtractionTempFolder() {
    try {
        if (typeof Folder === "undefined") return null;
        var base = Folder.temp || Folder.desktop || Folder.myDocuments;
        if (!base || !base.fsName) return null;
        var folder = new Folder(base.fsName + "/egui_expressive_raster_trace");
        if (!folder.exists && typeof folder.create === "function") folder.create();
        return folder.exists ? folder : null;
    } catch (e) {
        noteHostDiagnostic("embedded raster temp folder unavailable", e);
        return null;
    }
}

function closeTempDocumentWithoutSaving(doc) {
    if (!doc || typeof doc.close !== "function") return;
    try {
        if (typeof SaveOptions !== "undefined" && SaveOptions.DONOTSAVECHANGES !== undefined) doc.close(SaveOptions.DONOTSAVECHANGES);
        else doc.close();
    } catch (e) { noteHostDiagnostic("embedded raster temp document close failed", e); }
}

function extractEmbeddedRasterToTempPng(item, el) {
    if (!item || item.typename !== "RasterItem") return null;
    if (typeof File === "undefined" || typeof Folder === "undefined") return null;
    if (typeof app === "undefined" || !app.documents || typeof ExportOptionsPNG24 === "undefined" || typeof ExportType === "undefined") {
        noteHostDiagnostic("embedded raster extraction unavailable", "Illustrator export APIs unavailable");
        return null;
    }

    var folder = rasterExtractionTempFolder();
    if (!folder) return null;

    var width = Math.max(1, Math.ceil(Number(el && el.w || 1)));
    var height = Math.max(1, Math.ceil(Number(el && el.h || 1)));
    var stamp = (typeof Date !== "undefined" && Date.now) ? Date.now() : Math.floor(Math.random() * 1000000);
    var file = new File(folder.fsName + "/" + safeTempFileSlug(el && el.id) + "_" + stamp + ".png");
    var tempDoc = null;

    try {
        try {
            if (typeof DocumentColorSpace !== "undefined" && DocumentColorSpace.RGB !== undefined) tempDoc = app.documents.add(DocumentColorSpace.RGB, width, height);
        } catch (colorError) { noteHostDiagnostic("embedded raster temp document color setup failed", colorError); }
        if (!tempDoc) tempDoc = app.documents.add();

        try {
            if (tempDoc.artboards && tempDoc.artboards.length > 0) tempDoc.artboards[0].artboardRect = [0, height, width, 0];
        } catch (artboardError) { noteHostDiagnostic("embedded raster temp artboard setup failed", artboardError); }

        var target = (tempDoc.layers && tempDoc.layers.length > 0) ? tempDoc.layers[0] : tempDoc;
        var duplicate = null;
        if (typeof ElementPlacement !== "undefined" && ElementPlacement.PLACEATEND !== undefined) duplicate = item.duplicate(target, ElementPlacement.PLACEATEND);
        else duplicate = item.duplicate(target);
        try {
            var b = duplicate.geometricBounds || duplicate.visibleBounds;
            if (b && typeof duplicate.translate === "function") duplicate.translate(-Number(b[0] || 0), height - Number(b[1] || height));
        } catch (positionError) { noteHostDiagnostic("embedded raster temp positioning failed", positionError); }

        var options = new ExportOptionsPNG24();
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
        noteHostDiagnostic("embedded raster extraction failed", e);
    } finally {
        closeTempDocumentWithoutSaving(tempDoc);
    }

    return null;
}

function extractRasterTransformScale(item) {
    try {
        var matrix = item && item.matrix;
        if (!matrix) return null;
        var a = Number(matrix.mValueA !== undefined ? matrix.mValueA : (matrix.a !== undefined ? matrix.a : (matrix.A !== undefined ? matrix.A : 1)));
        var b = Number(matrix.mValueB !== undefined ? matrix.mValueB : (matrix.b !== undefined ? matrix.b : (matrix.B !== undefined ? matrix.B : 0)));
        var c = Number(matrix.mValueC !== undefined ? matrix.mValueC : (matrix.c !== undefined ? matrix.c : (matrix.C !== undefined ? matrix.C : 0)));
        var d = Number(matrix.mValueD !== undefined ? matrix.mValueD : (matrix.d !== undefined ? matrix.d : (matrix.D !== undefined ? matrix.D : 1)));
        var scaleX = Math.sqrt(a * a + b * b);
        var scaleY = Math.sqrt(c * c + d * d);
        if (isFinite(scaleX) && scaleX > 0 && isFinite(scaleY) && scaleY > 0) return { scaleX: scaleX, scaleY: scaleY };
    } catch (e) { noteHostDiagnostic("optional Illustrator raster transform unavailable", e); }
    return null;
}

function extractItemRotationDeg(item) {
    try {
        var direct = Number(item && item.rotation);
        if (isFinite(direct) && Math.abs(direct) > 0.0001) return direct;
    } catch (e) { noteHostDiagnostic("optional Illustrator rotation unavailable", e); }
    try {
        var matrix = item && item.matrix;
        if (!matrix) return 0;
        var a = Number(matrix.mValueA !== undefined ? matrix.mValueA : (matrix.a !== undefined ? matrix.a : (matrix.A !== undefined ? matrix.A : 1)));
        var b = Number(matrix.mValueB !== undefined ? matrix.mValueB : (matrix.b !== undefined ? matrix.b : (matrix.B !== undefined ? matrix.B : 0)));
        if (!isFinite(a) || !isFinite(b)) return 0;
        var deg = Math.atan2(b, a) * 180 / Math.PI;
        return isFinite(deg) ? deg : 0;
    } catch (e2) { noteHostDiagnostic("optional Illustrator matrix rotation unavailable", e2); }
    return 0;
}

function rasterEffectTypeFromName(value) {
    var name = String(value || "").toLowerCase().replace(/[\s_-]+/g, "");
    if (name.indexOf("dropshadow") !== -1) return "dropShadow";
    if (name.indexOf("innershadow") !== -1) return "innerShadow";
    if (name.indexOf("outerglow") !== -1) return "outerGlow";
    if (name.indexOf("innerglow") !== -1) return "innerGlow";
    if (name.indexOf("gaussianblur") !== -1) return "gaussianBlur";
    if (name.indexOf("feather") !== -1) return "feather";
    if (name.indexOf("bevel") !== -1) return "bevel";
    if (name.indexOf("noise") !== -1 || name.indexOf("grain") !== -1 || name.indexOf("mezzotint") !== -1) return "noise";
    return null;
}

function rasterEffectMetadataHasUnmappedEffect(text, mappedEffects) {
    var raw = String(text || "");
    if (!raw) return false;
    var lower = raw.toLowerCase();
    var hasMapped = mappedEffects && mappedEffects.length > 0;
    if (lower.indexOf("effect") === -1 && lower.indexOf("filter") === -1 && lower.indexOf("liveeffect") === -1 && lower.indexOf("aifx") === -1) return false;
    if (!hasMapped) return true;
    var namePattern = /(?:effect|filter|liveeffect|aifx)\s*(?:name|type)?\s*[:=]\s*["']?([A-Za-z][A-Za-z0-9 _-]{1,80})/gi;
    var match;
    while ((match = namePattern.exec(raw)) !== null) {
        if (!rasterEffectTypeFromName(match[1])) return true;
    }
    return false;
}

function defaultRasterEffect(type) {
    if (type === "dropShadow") return { type: type, x: 4, y: 4, blur: 8, color: { r: 0, g: 0, b: 0, a: 0.3 } };
    if (type === "innerShadow") return { type: type, x: 0, y: 0, blur: 4, color: { r: 0, g: 0, b: 0, a: 0.35 } };
    if (type === "outerGlow" || type === "innerGlow") return { type: type, blur: 6, color: { r: 255, g: 255, b: 255, a: 0.45 } };
    if (type === "gaussianBlur" || type === "feather") return { type: type, radius: 4, blur: 4 };
    return { type: type };
}

function mergeEffectByType(effects, effect) {
    if (!effect || !effect.type) return;
    for (var i = 0; i < effects.length; i++) {
        if (String(effects[i].type) === String(effect.type)) return;
    }
    effects.push(effect);
}

function effectsFromMetadataText(text) {
    var effects = [];
    var raw = String(text || "").toLowerCase();
    var tokens = ["dropShadow", "drop shadow", "innerShadow", "inner shadow", "outerGlow", "outer glow", "innerGlow", "inner glow", "gaussianBlur", "gaussian blur", "feather", "bevel", "noise", "grain", "mezzotint"];
    for (var i = 0; i < tokens.length; i++) {
        if (raw.indexOf(tokens[i].toLowerCase()) !== -1) mergeEffectByType(effects, defaultRasterEffect(rasterEffectTypeFromName(tokens[i])));
    }
    if (rasterEffectMetadataHasUnmappedEffect(text, effects)) mergeEffectByType(effects, { type: "unknown", source: "xmp" });
    return effects;
}

if (typeof JSON.parse !== 'function') {
    JSON.parse = function(str) {
        var text = String(str), i = 0;
        function fail(msg) { throw new Error('Invalid JSON: ' + msg + ' at ' + i); }
        function ws() { while (/\s/.test(text.charAt(i))) i++; }
        function parseString() {
            var out = '';
            if (text.charAt(i++) !== '"') fail('expected string');
            while (i < text.length) {
                var ch = text.charAt(i++);
                if (ch === '"') return out;
                if (ch === '\\') {
                    var esc = text.charAt(i++);
                    if (esc === '"' || esc === '\\' || esc === '/') out += esc;
                    else if (esc === 'b') out += '\b';
                    else if (esc === 'f') out += '\f';
                    else if (esc === 'n') out += '\n';
                    else if (esc === 'r') out += '\r';
                    else if (esc === 't') out += '\t';
                    else if (esc === 'u') { out += String.fromCharCode(parseInt(text.substr(i, 4), 16)); i += 4; }
                    else fail('bad escape');
                } else out += ch;
            }
            fail('unterminated string');
        }
        function parseNumber() {
            var start = i;
            if (text.charAt(i) === '-') i++;
            while (/\d/.test(text.charAt(i))) i++;
            if (text.charAt(i) === '.') { i++; while (/\d/.test(text.charAt(i))) i++; }
            if (/[eE]/.test(text.charAt(i))) { i++; if (/[+\-]/.test(text.charAt(i))) i++; while (/\d/.test(text.charAt(i))) i++; }
            var n = Number(text.substring(start, i));
            if (!isFinite(n)) fail('bad number');
            return n;
        }
        function parseArray() {
            var arr = [];
            i++; ws();
            if (text.charAt(i) === ']') { i++; return arr; }
            while (i < text.length) {
                arr.push(parseValue()); ws();
                if (text.charAt(i) === ']') { i++; return arr; }
                if (text.charAt(i++) !== ',') fail('expected comma');
            }
            fail('unterminated array');
        }
        function parseObject() {
            var obj = {};
            i++; ws();
            if (text.charAt(i) === '}') { i++; return obj; }
            while (i < text.length) {
                ws(); var key = parseString(); ws();
                if (text.charAt(i++) !== ':') fail('expected colon');
                obj[key] = parseValue(); ws();
                if (text.charAt(i) === '}') { i++; return obj; }
                if (text.charAt(i++) !== ',') fail('expected comma');
            }
            fail('unterminated object');
        }
        function parseValue() {
            ws();
            var ch = text.charAt(i);
            if (ch === '"') return parseString();
            if (ch === '{') return parseObject();
            if (ch === '[') return parseArray();
            if (ch === 't' && text.substr(i, 4) === 'true') { i += 4; return true; }
            if (ch === 'f' && text.substr(i, 5) === 'false') { i += 5; return false; }
            if (ch === 'n' && text.substr(i, 4) === 'null') { i += 4; return null; }
            return parseNumber();
        }
        var value = parseValue(); ws();
        if (i !== text.length) fail('trailing input');
        return value;
    };
}
if (typeof JSON.stringify !== 'function') {
    JSON.stringify = function(obj) {
        var t = typeof obj;
        if (t !== 'object' || obj === null) {
            if (t === 'string') return '"' + obj.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n').replace(/\r/g, '\\r').replace(/\t/g, '\\t') + '"';
            return String(obj);
        }
        var isArr = (Object.prototype.toString.call(obj) === '[object Array]');
        var arr = [];
        for (var k in obj) {
            if (Object.prototype.hasOwnProperty.call(obj, k)) {
                var v = obj[k];
                if (v !== undefined && typeof v !== 'function') {
                    if (isArr) arr.push(JSON.stringify(v));
                    else arr.push('"' + k + '":' + JSON.stringify(v));
                }
            }
        }
        return isArr ? '[' + arr.join(',') + ']' : '{' + arr.join(',') + '}';
    };
}

function hostJsonEscape(value) {
    return String(value === undefined || value === null ? "" : value)
        .replace(/\\/g, "\\\\")
        .replace(/"/g, "\\\"")
        .replace(/\n/g, "\\n")
        .replace(/\r/g, "\\r")
        .replace(/\t/g, "\\t");
}

function hostJsonPrimitive(value) {
    var t = typeof value;
    if (value === null || value === undefined) return "null";
    if (t === "number") return isFinite(value) ? String(value) : "0";
    if (t === "boolean") return value ? "true" : "false";
    return "\"" + hostJsonEscape(value) + "\"";
}

function safeJsonStringify(value) {
    try { return JSON.stringify(value); } catch (jsonErr) { noteHostDiagnostic("safeJsonStringify", jsonErr); }
    try {
        var isArr = (Object.prototype.toString.call(value) === "[object Array]");
        if (isArr) {
            var rows = [];
            for (var i = 0; i < value.length; i++) rows.push(safeJsonStringify(value[i]));
            return "[" + rows.join(",") + "]";
        }
        var parts = [];
        for (var k in value) {
            if (Object.prototype.hasOwnProperty.call(value, k)) {
                var v = value[k];
                if (typeof v !== "function") parts.push("\"" + hostJsonEscape(k) + "\":" + hostJsonPrimitive(v));
            }
        }
        return "{" + parts.join(",") + "}";
    } catch (fallbackErr) {
        return "{\"error\":\"JSON serialization failed: " + hostJsonEscape(fallbackErr) + "\"}";
    }
}

function openDocumentCount(context) {
    try { return app.documents.length; } catch (e) { noteHostDiagnostic(context || "documents.length", e); }
    try { if (typeof $ !== "undefined" && $.sleep) $.sleep(200); } catch (ignored) { /* optional sleep unavailable */ }
    try { return app.documents.length; } catch (e2) { noteHostDiagnostic((context || "documents.length") + " retry", e2); }
    return 0;
}

function activeIllustratorDocument(context) {
    try { return app.activeDocument; } catch (e) { noteHostDiagnostic(context || "activeDocument", e); }
    try { if (typeof $ !== "undefined" && $.sleep) $.sleep(200); } catch (ignored) { /* optional sleep unavailable */ }
    try { return app.activeDocument; } catch (e2) { noteHostDiagnostic((context || "activeDocument") + " retry", e2); }
    return null;
}

function getDiagnosticsJSON() {
    var result = { hasApp: false, hasDoc: false, artboardCount: 0, docName: "", error: "" };
    try {
        var appExists = false;
        try { appExists = (typeof app !== 'undefined'); } catch(e) { noteHostDiagnostic("getDiagnosticsJSON error", e); return JSON.stringify({ error: e.message || String(e) }); }
        result.hasApp = appExists;
        if (result.hasApp) {
            var docCount = openDocumentCount("getDiagnosticsJSON documents.length");
            result.hasDoc = (docCount > 0);
            if (result.hasDoc) {
                var doc = activeIllustratorDocument("getDiagnosticsJSON activeDocument");
                if (!doc) {
                    result.hasDoc = false;
                    result.error = "Illustrator reports an open document, but activeDocument is not ready yet. Try clicking the document canvas, then press Refresh.";
                    return safeJsonStringify(result);
                }
                try { result.docName = doc.name || ""; } catch (nameErr) { noteHostDiagnostic("getDiagnosticsJSON doc.name", nameErr); }
                try { result.artboardCount = doc.artboards ? doc.artboards.length : 0; } catch (abErr) { noteHostDiagnostic("getDiagnosticsJSON artboards.length", abErr); result.artboardCount = 0; }
                
                // Page tile detection is handled by ai_parser, not by size heuristics.
                // Always report hasPageTiles = false here; the panel will check ai_parser separately.
                result.hasPageTiles = false;
                result.estimatedPageCount = 1;
            }
        }
    } catch(e) { noteHostDiagnostic("getDiagnosticsJSON error", e); return safeJsonStringify({ error: String(e) }); }
    return safeJsonStringify(result);
}

function getDocumentPath(doc) {
    try {
        if (doc && doc.fullName && doc.fullName.fsName) return doc.fullName.fsName;
    } catch (e) { noteHostDiagnostic("getDocumentPath fullName", e); }
    try {
        if (doc && doc.path && doc.name) return doc.path + "/" + doc.name;
    } catch (e2) { noteHostDiagnostic("getDocumentPath path/name", e2); }
    return "";
}

function portableAssetPath(pathValue) {
    var raw = String(pathValue || "").replace(/\\/g, "/");
    var parts = raw.split("/");
    var name = parts.length ? parts[parts.length - 1] : "";
    name = name.replace(/[^A-Za-z0-9._-]/g, "_");
    if (!name) return null;
    var hash = 0;
    for (var i = 0; i < raw.length; i++) {
        hash = ((hash << 5) - hash) + raw.charCodeAt(i);
        hash |= 0;
    }
    var hashStr = Math.abs(hash).toString(16);
    hashStr = hashStr.substring(0, 6);
    return "assets/" + hashStr + "_" + name;
}

function sanitizeOutputFilename(filename) {
    var raw = String(filename || "").replace(/\\/g, "/");
    var parts = raw.split("/");
    var base = parts.length ? parts[parts.length - 1] : "";
    base = base.replace(/[^A-Za-z0-9._-]/g, "_");
    if (!base || base === "." || base === "..") base = "exported.rs";
    return base;
}

function getArtboardsJSON() {
    try {
        var appExists = false;
        try { appExists = (typeof app !== 'undefined'); } catch(e) { noteHostDiagnostic("getArtboardsJSON error", e); return JSON.stringify({ error: e.message || String(e) }); }
        if (!appExists) { return "[]"; }
        if (openDocumentCount("getArtboardsJSON documents.length") === 0) { return "[]"; }
        var doc = activeIllustratorDocument("getArtboardsJSON activeDocument");
        if (!doc) { return "[]"; }
        var boards = [];
        var artboards = null;
        var artboardCount = 0;
        try {
            artboards = doc.artboards;
            artboardCount = artboards ? artboards.length : 0;
        } catch (listErr) {
            noteHostDiagnostic("getArtboardsJSON artboards", listErr);
            return safeJsonStringify({ error: "Could not access Illustrator artboards: " + String(listErr) });
        }
        for (var i = 0; i < artboardCount; i++) {
            var ab = null;
            var r = null;
            try { ab = artboards[i]; } catch (itemErr) { noteHostDiagnostic("getArtboardsJSON artboard", itemErr); continue; }
            try { r = ab.artboardRect; } catch (rectErr) { noteHostDiagnostic("getArtboardsJSON artboardRect", rectErr); continue; }
            if (!r || r.length < 4) { continue; }
            var name = "Artboard " + (i + 1);
            try { if (ab.name) name = ab.name; } catch (nameErr) { noteHostDiagnostic("getArtboardsJSON name", nameErr); }
            boards.push({
                index: i,
                name: name,
                width: Math.abs(r[2] - r[0]),
                height: Math.abs(r[3] - r[1]),
                x: r[0],
                y: r[1]
            });
        }
        return safeJsonStringify(boards);
    } catch (e) {
        noteHostDiagnostic("getArtboardsJSON error", e);
        return safeJsonStringify({ error: String(e) });
    }
}




function saveFilesToFolderJSON(payloadJSON) {
    try {
        var payload = JSON.parse(payloadJSON);
        var files = payload.files;
        
        var folder = Folder.selectDialog("Select destination folder");
        if (!folder) {
            return JSON.stringify({ canceled: true });
        }
        
        var saved = [];
        var errors = [];
        for (var filename in files) {
            if (Object.prototype.hasOwnProperty.call(files, filename)) {
                var safeFilename = sanitizeOutputFilename(filename);
                var file = new File(folder.fsName + "/" + safeFilename);
                file.encoding = "UTF-8";
                if (file.open("w")) {
                    if (file.write(files[filename])) {
                        saved.push(safeFilename);
                    } else {
                        errors.push("Failed to write " + safeFilename);
                    }
                    file.close();
                } else {
                    errors.push("Failed to open " + safeFilename);
                }
            }
        }
        
        var assets = payload.assets || {};
        var assetsFolder = new Folder(folder.fsName + "/assets");
        var createdAssetsFolder = false;
        for (var assetPath in assets) {
            if (Object.prototype.hasOwnProperty.call(assets, assetPath)) {
                var sourceFile = new File(assets[assetPath]);
                if (sourceFile.exists) {
                    if (!createdAssetsFolder) {
                        assetsFolder.create();
                        createdAssetsFolder = true;
                    }
                    var destFile = new File(folder.fsName + "/" + assetPath);
                    if (sourceFile.copy(destFile)) {
                        saved.push(assetPath);
                    } else {
                        errors.push("Failed to copy " + assetPath);
                    }
                } else {
                    errors.push("Source asset missing: " + assets[assetPath]);
                }
            }
        }

        if (errors.length > 0) {
            return JSON.stringify({ error: errors.join(", "), saved: saved });
        }
        return JSON.stringify({ success: true, folder: folder.fsName, saved: saved });
    } catch (e) {
        noteHostDiagnostic("saveFilesToFolderJSON exception", e);
        return JSON.stringify({ error: String(e) });
    }
}

function selectSaveFolderJSON() {
    try {
        var folder = Folder.selectDialog("Select destination folder");
        if (!folder) {
            return safeJsonStringify({ canceled: true });
        }
        return safeJsonStringify({ success: true, folder: folder.fsName });
    } catch (e) {
        noteHostDiagnostic("selectSaveFolderJSON exception", e);
        return safeJsonStringify({ error: String(e) });
    }
}

function writeGeneratedFileChunkJSON(payloadJSON) {
    try {
        var payload = JSON.parse(payloadJSON || "{}");
        var folderPath = String(payload.folder || "");
        var filename = sanitizeOutputFilename(payload.filename || "generated.rs");
        var mode = payload.mode === "a" ? "a" : "w";
        var content = String(payload.content || "");
        if (!folderPath) { return safeJsonStringify({ error: "Missing destination folder" }); }

        var folder = new Folder(folderPath);
        if (!folder.exists) { return safeJsonStringify({ error: "Destination folder does not exist: " + folderPath }); }

        var file = new File(folder.fsName + "/" + filename);
        file.encoding = "UTF-8";
        if (!file.open(mode)) { return safeJsonStringify({ error: "Failed to open " + filename }); }
        var wrote = file.write(content);
        file.close();
        if (!wrote) { return safeJsonStringify({ error: "Failed to write " + filename }); }
        return safeJsonStringify({ success: true, filename: filename, bytes: content.length });
    } catch (e) {
        noteHostDiagnostic("writeGeneratedFileChunkJSON exception", e);
        return safeJsonStringify({ error: String(e) });
    }
}

function copyGeneratedAssetJSON(payloadJSON) {
    try {
        var payload = JSON.parse(payloadJSON || "{}");
        var folderPath = String(payload.folder || "");
        var assetPath = String(payload.assetPath || "").replace(/\\/g, "/");
        var sourcePath = String(payload.sourcePath || "");
        if (!folderPath) { return safeJsonStringify({ error: "Missing destination folder" }); }
        if (!assetPath || assetPath.indexOf("assets/") !== 0) { return safeJsonStringify({ error: "Invalid asset path: " + assetPath }); }
        if (!sourcePath) { return safeJsonStringify({ error: "Missing source asset for " + assetPath }); }

        var folder = new Folder(folderPath);
        if (!folder.exists) { return safeJsonStringify({ error: "Destination folder does not exist: " + folderPath }); }
        var assetsFolder = new Folder(folder.fsName + "/assets");
        if (!assetsFolder.exists) {
            assetsFolder.create();
        }

        var assetName = sanitizeOutputFilename(assetPath.substring("assets/".length));
        var sourceFile = new File(sourcePath);
        if (!sourceFile.exists) { return safeJsonStringify({ error: "Source asset missing: " + sourcePath }); }
        var destFile = new File(assetsFolder.fsName + "/" + assetName);
        if (!sourceFile.copy(destFile)) { return safeJsonStringify({ error: "Failed to copy " + assetPath }); }
        return safeJsonStringify({ success: true, filename: "assets/" + assetName });
    } catch (e) {
        noteHostDiagnostic("copyGeneratedAssetJSON exception", e);
        return safeJsonStringify({ error: String(e) });
    }
}

function extractArtboardDataJSON(exportPayloadJSON) {
    try {
        var appExists = false;
        try { appExists = (typeof app !== 'undefined'); } catch(e) { noteHostDiagnostic("extractArtboardDataJSON app check failed", e); return JSON.stringify({ error: e.message || String(e) }); }
        if (!appExists || app.documents.length === 0) { return "[]"; }
        var doc = app.activeDocument;
        if (!doc) { return "[]"; }
        
        var payload = JSON.parse(exportPayloadJSON);
        var selectedIndices = [];
        var selectedTiles = [];
        if (Object.prototype.toString.call(payload) === '[object Array]') {
            selectedIndices = payload;
        } else {
            selectedIndices = payload.selected || [];
            selectedTiles = payload.selectedTiles || [];
        }
        var results = [];
        
        function isTopLevelItem(item) {
            try {
                var parentType = item.parent ? item.parent.typename : null;
                return parentType === 'Layer' || parentType === 'Document' || parentType === null;
            } catch(e) { return true; }
        }
        
        function colorToRGB(c, depth) {
            if (!c) return null;
            depth = depth || 0;
            if (depth > 4) return null;
            try {
                if (c.typename === "RGBColor") return { r: Math.round(c.red), g: Math.round(c.green), b: Math.round(c.blue), a: 255 };
                if (c.typename === "CMYKColor") { var k = c.black/100; return { r: Math.round(255*(1-c.cyan/100)*(1-k)), g: Math.round(255*(1-c.magenta/100)*(1-k)), b: Math.round(255*(1-c.yellow/100)*(1-k)), a: 255 }; }
                if (c.typename === "GrayColor") { var v = Math.round(255*(1-c.gray/100)); return { r: v, g: v, b: v, a: 255 }; }
                if (c.typename === "SpotColor" && c.spot && c.spot.color) { var base = colorToRGB(c.spot.color, depth + 1); if (!base) return null; var tint = Number(c.tint); if (!isFinite(tint)) tint = 100; tint = Math.max(0, Math.min(100, tint)) / 100; return { r: Math.round(255 - (255 - base.r) * tint), g: Math.round(255 - (255 - base.g) * tint), b: Math.round(255 - (255 - base.b) * tint), a: base.a === undefined ? 255 : base.a }; }
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return null;
        }

        function collectionToArray(collection, limit) {
            var out = [];
            if (!collection) return out;
            if (typeof collection === "number" || typeof collection === "string") return [collection];
            limit = limit || 512;
            try {
                if (collection.length !== undefined) {
                    var length = Number(collection.length);
                    for (var i = 0; i < Math.min(length, limit); i++) {
                        try { if (collection[i]) out.push(collection[i]); } catch (e) { noteHostDiagnostic("optional Illustrator collection item unavailable", e); }
                    }
                    return out;
                }
            } catch (e2) { noteHostDiagnostic("optional Illustrator collection unavailable", e2); }
            if (typeof collection === "object") out.push(collection);
            return out;
        }

        function safeReadProperty(object, property, label) {
            try { return object && object[property]; }
            catch (e) { noteHostDiagnostic(label || ("optional Illustrator property " + property + " unavailable"), e); return null; }
        }

        function addPatternCandidate(candidates, items, source) {
            var array = collectionToArray(items, 512);
            if (array.length > 0) candidates.push({ items: array, source: source });
        }

        function patternPageItemCandidates(pattern) {
            var candidates = [];
            var patternItem = safeReadProperty(pattern, "patternItem", "optional Illustrator patternItem unavailable");
            if (patternItem) {
                addPatternCandidate(candidates, safeReadProperty(patternItem, "pageItems", "optional Illustrator patternItem.pageItems unavailable"), "pattern.patternItem.pageItems");
                addPatternCandidate(candidates, safeReadProperty(patternItem, "pathItems", "optional Illustrator patternItem.pathItems unavailable"), "pattern.patternItem.pathItems");
                addPatternCandidate(candidates, safeReadProperty(patternItem, "compoundPathItems", "optional Illustrator patternItem.compoundPathItems unavailable"), "pattern.patternItem.compoundPathItems");
                addPatternCandidate(candidates, safeReadProperty(patternItem, "groupItems", "optional Illustrator patternItem.groupItems unavailable"), "pattern.patternItem.groupItems");
            }
            addPatternCandidate(candidates, safeReadProperty(pattern, "pageItems", "optional Illustrator pattern.pageItems unavailable"), "pattern.pageItems");
            addPatternCandidate(candidates, safeReadProperty(pattern, "pathItems", "optional Illustrator pattern.pathItems unavailable"), "pattern.pathItems");
            var artwork = safeReadProperty(pattern, "artwork", "optional Illustrator pattern.artwork unavailable");
            if (artwork) addPatternCandidate(candidates, safeReadProperty(artwork, "pageItems", "optional Illustrator pattern.artwork.pageItems unavailable"), "pattern.artwork.pageItems");
            return candidates;
        }

        function addPatternSwatchColor(stats, color) {
            if (!color) return;
            var c = {
                r: Math.max(0, Math.min(255, Math.round(Number(color.r) || 0))),
                g: Math.max(0, Math.min(255, Math.round(Number(color.g) || 0))),
                b: Math.max(0, Math.min(255, Math.round(Number(color.b) || 0))),
                a: color.a === undefined ? 255 : Math.max(0, Math.min(255, Math.round(Number(color.a) || 255)))
            };
            var key = c.r + "," + c.g + "," + c.b + "," + c.a;
            if (!stats[key]) stats[key] = { r: c.r, g: c.g, b: c.b, a: c.a, count: 0 };
            stats[key].count += 1;
        }

        function collectPatternItemColors(item, stats, depth) {
            if (!item || depth > 6) return;
            try { if (item.filled !== false && item.fillColor) addPatternSwatchColor(stats, colorToRGB(item.fillColor)); } catch (e) { noteHostDiagnostic("optional Illustrator pattern fill unavailable", e); }
            try { if (item.stroked !== false && item.strokeColor) addPatternSwatchColor(stats, colorToRGB(item.strokeColor)); } catch (e2) { noteHostDiagnostic("optional Illustrator pattern stroke unavailable", e2); }
            var properties = ["pageItems", "pathItems", "compoundPathItems", "groupItems", "children"];
            for (var p = 0; p < properties.length; p++) {
                var children = collectionToArray(safeReadProperty(item, properties[p], "optional Illustrator pattern child collection unavailable"), 512);
                for (var c = 0; c < children.length; c++) collectPatternItemColors(children[c], stats, depth + 1);
            }
        }

        function patternItemStrokeWidth(item) {
            try {
                var width = Number(item && item.strokeWidth);
                return isFinite(width) && width > 0 ? width : 1;
            } catch (e) { noteHostDiagnostic("optional Illustrator pattern stroke width unavailable", e); }
            return 1;
        }

        function collectPatternTileGeometry(item, artboardRect, shapes, depth) {
            if (!item || depth > 6) return;
            try {
                var t = String(item.typename || "");
                if (t === "PathItem" || t === "CompoundPathItem") {
                    var pathData = getPathPoints(item, artboardRect || [0, 0, 0, 0]);
                    var fill = item.filled !== false ? colorToRGB(item.fillColor) : null;
                    var stroke = item.stroked !== false ? colorToRGB(item.strokeColor) : null;
                    var strokeWidth = stroke ? patternItemStrokeWidth(item) : 0;
                    var subpaths = pathData && pathData.subpaths ? pathData.subpaths : (pathData ? [{ points: pathData.points, closed: pathData.closed }] : []);
                    for (var s = 0; s < subpaths.length; s++) {
                        var pts = [];
                        for (var p = 0; p < (subpaths[s].points || []).length; p++) pts.push(tupleFromPoint(subpaths[s].points[p].anchor, [0, 0]));
                        if (pts.length >= 2 && (fill || stroke)) shapes.push({ points: pts, closed: subpaths[s].closed !== false, fill: fill, stroke: stroke, strokeWidth: strokeWidth });
                    }
                }
            } catch (e) { noteHostDiagnostic("optional Illustrator pattern geometry unavailable", e); }
            var properties = ["pageItems", "pathItems", "compoundPathItems", "groupItems", "children"];
            for (var pi = 0; pi < properties.length; pi++) {
                var children = collectionToArray(safeReadProperty(item, properties[pi], "optional Illustrator pattern geometry collection unavailable"), 512);
                for (var c = 0; c < children.length; c++) collectPatternTileGeometry(children[c], artboardRect, shapes, depth + 1);
            }
        }

        function normalizePatternTileGeometry(shapes) {
            var allPoints = [];
            for (var s = 0; s < (shapes || []).length; s++) for (var p = 0; p < (shapes[s].points || []).length; p++) allPoints.push(shapes[s].points[p]);
            var bounds = boundsFromTuples(allPoints);
            if (!bounds || bounds.w <= 0.0001 || bounds.h <= 0.0001) return [];
            var basis = Math.max(bounds.w, bounds.h, 1);
            var out = [];
            for (var i = 0; i < (shapes || []).length; i++) {
                var shape = shapes[i];
                var pts = [];
                for (var j = 0; j < (shape.points || []).length; j++) pts.push([(Number(shape.points[j][0]) - bounds.x) / bounds.w, (Number(shape.points[j][1]) - bounds.y) / bounds.h]);
                if (pts.length >= 2 && (shape.fill || shape.stroke)) out.push({ points: pts, closed: shape.closed !== false, fill: shape.fill || null, stroke: shape.stroke || null, strokeWidth: shape.stroke ? Math.max(0.001, Number(shape.strokeWidth || 1) / basis) : 0 });
            }
            return out;
        }

        function patternTileGeometryFromItems(items, artboardRect) {
            var shapes = [];
            for (var i = 0; i < (items || []).length; i++) collectPatternTileGeometry(items[i], artboardRect, shapes, 0);
            return normalizePatternTileGeometry(shapes);
        }

        function patternSwatchFromColor(patternColor, artboardRect) {
            var pattern = patternColor && patternColor.pattern;
            if (!pattern) return null;
            var candidates = patternPageItemCandidates(pattern);
            for (var i = 0; i < candidates.length; i++) {
                var stats = {};
                for (var j = 0; j < candidates[i].items.length; j++) collectPatternItemColors(candidates[i].items[j], stats, 0);
                var colors = [];
                for (var key in stats) if (stats.hasOwnProperty(key)) colors.push(stats[key]);
                colors.sort(function(a, b) { return b.count - a.count; });
                if (colors.length > 0) {
                    var tileGeometry = patternTileGeometryFromItems(candidates[i].items, artboardRect);
                    var foreground = { r: colors[0].r, g: colors[0].g, b: colors[0].b, a: colors[0].a };
                    var background = colors.length > 1 ? { r: colors[1].r, g: colors[1].g, b: colors[1].b, a: colors[1].a } : { r: 255, g: 255, b: 255, a: 0 };
                    return { swatchExtracted: true, sampled: true, swatchSource: candidates[i].source, pageItemCount: candidates[i].items.length, foreground: foreground, background: background, colors: colors.slice(0, 8), tileGeometry: tileGeometry };
                }
            }
            return null;
        }

        function patternScaleFromColor(patternColor) {
            var values = [];
            try {
                var scale = patternColor && patternColor.scaleFactor;
                var raw = collectionToArray(scale, 2);
                for (var i = 0; i < raw.length; i++) {
                    var value = Number(raw[i]);
                    if (isFinite(value)) values.push(Math.abs(value) > 10 ? value / 100 : value);
                }
            } catch (e) { noteHostDiagnostic("optional Illustrator pattern scale unavailable", e); }
            if (values.length === 0) return [1, 1];
            if (values.length === 1) values.push(values[0]);
            return values.slice(0, 2);
        }

        function symbolPageItemLike(value) {
            if (!value || typeof value !== "object") return false;
            try {
                var type = String(value.typename || "");
                return !!type && type !== "Symbol" && type !== "SymbolItem" && type !== "Document" && type !== "Layer";
            } catch (e) { noteHostDiagnostic("optional Illustrator symbol item typename unavailable", e); return false; }
        }

        function addSymbolDefinitionItems(candidates, container, source) {
            if (!container) return;
            var added = false;
            var properties = ["pageItems", "pathItems", "compoundPathItems", "groupItems", "symbolItems"];
            for (var p = 0; p < properties.length; p++) {
                var raw = collectionToArray(safeReadProperty(container, properties[p], "optional Illustrator " + source + "." + properties[p] + " unavailable"), 512);
                var items = [];
                for (var i = 0; i < raw.length; i++) if (symbolPageItemLike(raw[i])) items.push(raw[i]);
                if (items.length > 0) {
                    candidates.push({ items: items, source: source + "." + properties[p] });
                    added = true;
                }
            }
            if (!added && symbolPageItemLike(container)) candidates.push({ items: [container], source: source });
        }

        function symbolDefinitionCandidates(symbolItem) {
            var candidates = [];
            var symbol = safeReadProperty(symbolItem, "symbol", "optional Illustrator symbol reference unavailable");
            addSymbolDefinitionItems(candidates, safeReadProperty(symbolItem, "definition", "optional Illustrator symbolItem.definition unavailable"), "symbolItem.definition");
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

        function tupleFromPoint(value, fallback) {
            var tuple = value && value.length !== undefined ? value : fallback;
            return [Number(tuple && tuple[0] || 0), Number(tuple && tuple[1] || 0)];
        }

        function pathPointBoundsTuples(point) {
            return [
                tupleFromPoint(point.anchor, [0, 0]),
                tupleFromPoint(point.leftDir || point.left_ctrl || point.leftCtrl || point.anchor, [0, 0]),
                tupleFromPoint(point.rightDir || point.right_ctrl || point.rightCtrl || point.anchor, [0, 0])
            ];
        }

        function rectCornerTuples(el) {
            var x = Number(el.x || 0), y = Number(el.y || 0), w = Number(el.w || 0), h = Number(el.h || 0);
            return [[x, y], [x + w, y], [x + w, y + h], [x, y + h]];
        }

        function boundsFromTuples(points) {
            if (!points || points.length === 0) return null;
            var minX = Number.POSITIVE_INFINITY, minY = Number.POSITIVE_INFINITY, maxX = Number.NEGATIVE_INFINITY, maxY = Number.NEGATIVE_INFINITY;
            for (var i = 0; i < points.length; i++) {
                var p = tupleFromPoint(points[i], [0, 0]);
                minX = Math.min(minX, p[0]); maxX = Math.max(maxX, p[0]);
                minY = Math.min(minY, p[1]); maxY = Math.max(maxY, p[1]);
            }
            return { x: minX, y: minY, w: Math.max(0, maxX - minX), h: Math.max(0, maxY - minY) };
        }

        function geometryBoundsTuples(el) {
            var points = [];
            var subpaths = el && el.subpaths ? el.subpaths : [];
            for (var s = 0; s < subpaths.length; s++) {
                var subpathPoints = subpaths[s].points || [];
                for (var sp = 0; sp < subpathPoints.length; sp++) points = points.concat(pathPointBoundsTuples(subpathPoints[sp]));
            }
            if (points.length === 0 && el && el.pathPoints) for (var pp = 0; pp < el.pathPoints.length; pp++) points = points.concat(pathPointBoundsTuples(el.pathPoints[pp]));
            if (points.length === 0) points = points.concat(rectCornerTuples(el));
            return points;
        }

        function elementTreeBounds(elements) {
            var points = [];
            function walk(el) {
                if (!el) return;
                points = points.concat(geometryBoundsTuples(el));
                var children = el.children || [];
                for (var i = 0; i < children.length; i++) walk(children[i]);
            }
            for (var i = 0; i < (elements || []).length; i++) walk(elements[i]);
            return boundsFromTuples(points);
        }

        function mapTupleBetweenBounds(tuple, source, target) {
            var p = tupleFromPoint(tuple, [source.x, source.y]);
            var sx = source.w > 0.0001 ? target.w / source.w : 1;
            var sy = source.h > 0.0001 ? target.h / source.h : 1;
            return [target.x + (p[0] - source.x) * sx, target.y + (p[1] - source.y) * sy];
        }

        function mapPathPointBetweenBounds(point, source, target) {
            if (point && point.length !== undefined) return mapTupleBetweenBounds(point, source, target);
            var anchor = mapTupleBetweenBounds(point.anchor, source, target);
            var leftDir = mapTupleBetweenBounds(point.leftDir || point.left_ctrl || point.leftCtrl || point.anchor, source, target);
            var rightDir = mapTupleBetweenBounds(point.rightDir || point.right_ctrl || point.rightCtrl || point.anchor, source, target);
            var mapped = {};
            for (var key in point) if (point.hasOwnProperty(key)) mapped[key] = point[key];
            mapped.anchor = anchor; mapped.leftDir = leftDir; mapped.rightDir = rightDir; mapped.left_ctrl = leftDir; mapped.right_ctrl = rightDir;
            return mapped;
        }

        function fitElementTreeToBounds(el, source, target) {
            var min = mapTupleBetweenBounds([el.x || 0, el.y || 0], source, target);
            var max = mapTupleBetweenBounds([Number(el.x || 0) + Number(el.w || 0), Number(el.y || 0) + Number(el.h || 0)], source, target);
            el.x = min[0]; el.y = min[1]; el.w = Math.abs(max[0] - min[0]); el.h = Math.abs(max[1] - min[1]);
            if (el.pathPoints) for (var p = 0; p < el.pathPoints.length; p++) el.pathPoints[p] = mapPathPointBetweenBounds(el.pathPoints[p], source, target);
            if (el.subpaths) for (var s = 0; s < el.subpaths.length; s++) {
                var pts = el.subpaths[s].points || [];
                for (var sp = 0; sp < pts.length; sp++) pts[sp] = mapPathPointBetweenBounds(pts[sp], source, target);
                if (s === 0) el.pathPoints = pts;
            }
            for (var c = 0; c < (el.children || []).length; c++) fitElementTreeToBounds(el.children[c], source, target);
        }

        function prefixExpandedSymbolChildIds(children, prefix) {
            for (var i = 0; i < (children || []).length; i++) {
                var child = children[i];
                child.id = prefix + "_" + (child.id || ("child_" + i));
                prefixExpandedSymbolChildIds(child.children || [], child.id);
            }
        }

        function rotateTuple(point, center, degrees) {
            var radians = Number(degrees || 0) * Math.PI / 180;
            var cos = Math.cos(radians), sin = Math.sin(radians);
            var p = tupleFromPoint(point, [0, 0]);
            var dx = p[0] - center.x, dy = p[1] - center.y;
            return [center.x + dx * cos - dy * sin, center.y + dx * sin + dy * cos];
        }

        function rotatePathPoint(point, center, degrees) {
            var anchor = rotateTuple(point.anchor, center, degrees);
            var leftDir = rotateTuple(point.leftDir || point.left_ctrl || point.leftCtrl || point.anchor, center, degrees);
            var rightDir = rotateTuple(point.rightDir || point.right_ctrl || point.rightCtrl || point.anchor, center, degrees);
            var rotated = {};
            for (var key in point) if (point.hasOwnProperty(key)) rotated[key] = point[key];
            rotated.anchor = anchor; rotated.leftDir = leftDir; rotated.rightDir = rightDir; rotated.left_ctrl = leftDir; rotated.right_ctrl = rightDir;
            return rotated;
        }

        function applyBoundsFromGeometry(el) {
            var bounds = boundsFromTuples(geometryBoundsTuples(el));
            if (bounds) { el.x = bounds.x; el.y = bounds.y; el.w = bounds.w; el.h = bounds.h; }
            return el;
        }

        function rotateSymbolExpandedElement(el, center, degrees) {
            if (el.pathPoints || el.subpaths) {
                if (el.pathPoints) for (var p = 0; p < el.pathPoints.length; p++) el.pathPoints[p] = rotatePathPoint(el.pathPoints[p], center, degrees);
                if (el.subpaths) for (var s = 0; s < el.subpaths.length; s++) {
                    var pts = el.subpaths[s].points || [];
                    for (var sp = 0; sp < pts.length; sp++) pts[sp] = rotatePathPoint(pts[sp], center, degrees);
                    if (s === 0) el.pathPoints = pts;
                }
                el.rotation = 0;
                return applyBoundsFromGeometry(el);
            }
            el.rotation = Number(el.rotation || 0) + Number(degrees || 0);
            var childCenter = [Number(el.x || 0) + Number(el.w || 0) / 2, Number(el.y || 0) + Number(el.h || 0) / 2];
            var rotatedCenter = rotateTuple(childCenter, center, degrees);
            el.x = rotatedCenter[0] - Number(el.w || 0) / 2;
            el.y = rotatedCenter[1] - Number(el.h || 0) / 2;
            for (var c = 0; c < (el.children || []).length; c++) el.children[c] = rotateSymbolExpandedElement(el.children[c], center, degrees);
            return el;
        }

        function expandSymbolDefinitionIntoElement(item, artboardRect, el, depth) {
            if (depth > 32) {
                ensureElementNotes(el).push("symbol definition expansion depth limit reached; expand symbol before strict export");
                return false;
            }
            var candidates = symbolDefinitionCandidates(item);
            for (var i = 0; i < candidates.length; i++) {
                var children = [];
                for (var j = 0; j < candidates[i].items.length; j++) extractRecursive(candidates[i].items[j], artboardRect, children, depth + 1);
                if (children.length === 0) continue;
                var sourceBounds = elementTreeBounds(children);
                if (sourceBounds && sourceBounds.w > 0.0001 && sourceBounds.h > 0.0001) {
                    var targetBounds = { x: el.x, y: el.y, w: el.w, h: el.h };
                    for (var c = 0; c < children.length; c++) fitElementTreeToBounds(children[c], sourceBounds, targetBounds);
                }
                var rotation = Number(el.rotation || 0);
                if (isFinite(rotation) && Math.abs(rotation) > 0.0001) {
                    var center = { x: Number(el.x || 0) + Number(el.w || 0) / 2, y: Number(el.y || 0) + Number(el.h || 0) / 2 };
                    for (var r = 0; r < children.length; r++) children[r] = rotateSymbolExpandedElement(children[r], center, rotation);
                }
                el.rotation = 0;
                prefixExpandedSymbolChildIds(children, el.id || "symbol");
                el.children = children;
                el.symbolExpanded = true;
                el.symbolExpansionSource = candidates[i].source;
                ensureElementNotes(el).push("symbol definition expanded from " + candidates[i].source + "; instance transform fitted to symbol bounds");
                return true;
            }
            return false;
        }

        function hasUnsupportedLiveEffectExtraction(el) {
            try {
                if (!el || !el.effects) return false;
                for (var i = 0; i < el.effects.length; i++) {
                    var type = String(el.effects[i] && (el.effects[i].type || el.effects[i].effectType || el.effects[i].effect_type) || "").toLowerCase();
                    if (type === "liveeffect" || type === "live-effect" || type === "live_effect" || type === "unknown") return true;
                }
            } catch (e) { noteHostDiagnostic("optional Illustrator effect inspection unavailable", e); }
            return false;
        }

        function expandAppearanceViaDuplicate(item, artboardRect, el, depth, label) {
            if (depth > 32) {
                ensureElementNotes(el).push((label || "appearance") + " expansion depth limit reached; duplicate + Expand Appearance fallback unavailable");
                return null;
            }
            try {
                if (typeof app === "undefined" || !app || !app.documents || typeof app.executeMenuCommand !== "function") {
                    noteHostDiagnostic((label || "appearance") + " Expand Appearance fallback unavailable", "Illustrator menu command APIs unavailable");
                    return null;
                }
                if (typeof ElementPlacement === "undefined" || ElementPlacement.PLACEATEND === undefined) {
                    noteHostDiagnostic((label || "appearance") + " Expand Appearance fallback unavailable", "Illustrator element placement API unavailable");
                    return null;
                }

                var tempDoc = null;
                var duplicate = null;
                var width = Math.max(1, Math.ceil(Math.abs(Number(artboardRect && artboardRect[2] || 1) - Number(artboardRect && artboardRect[0] || 0))));
                var height = Math.max(1, Math.ceil(Math.abs(Number(artboardRect && artboardRect[1] || 1) - Number(artboardRect && artboardRect[3] || 0))));

                try {
                    if (typeof DocumentColorSpace !== "undefined" && DocumentColorSpace.RGB !== undefined) tempDoc = app.documents.add(DocumentColorSpace.RGB, width, height);
                } catch (docColorError) { noteHostDiagnostic((label || "appearance") + " temp document color setup failed", docColorError); }
                if (!tempDoc) tempDoc = app.documents.add();

                try { if (tempDoc.artboards && tempDoc.artboards.length > 0) tempDoc.artboards[0].artboardRect = [0, height, width, 0]; } catch (artboardError) { noteHostDiagnostic((label || "appearance") + " temp artboard setup failed", artboardError); }

                var target = (tempDoc.layers && tempDoc.layers.length > 0) ? tempDoc.layers[0] : tempDoc;
                duplicate = item.duplicate(target, ElementPlacement.PLACEATEND);

                try { if (typeof tempDoc.activate === "function") tempDoc.activate(); } catch (activateError) { noteHostDiagnostic((label || "appearance") + " temp document activate failed", activateError); }
                try { tempDoc.selection = [duplicate]; } catch (selectionError) { noteHostDiagnostic((label || "appearance") + " temp selection unavailable", selectionError); }
                try { app.selection = [duplicate]; } catch (appSelectionError) { noteHostDiagnostic((label || "appearance") + " application selection unavailable", appSelectionError); }

                try { app.executeMenuCommand("expandStyle"); } catch (expandError) {
                    noteHostDiagnostic((label || "appearance") + " Expand Appearance fallback failed", expandError);
                    return null;
                }

                var roots = [];
                try { if (tempDoc.selection && tempDoc.selection.length > 0) roots = collectionToArray(tempDoc.selection, 256); } catch (selectionReadError) { noteHostDiagnostic((label || "appearance") + " expanded selection unavailable", selectionReadError); }
                if (roots.length === 0) {
                    try { roots = collectionToArray(tempDoc.pageItems, 256); } catch (pageItemsError) { noteHostDiagnostic((label || "appearance") + " expanded pageItems unavailable", pageItemsError); }
                }
                if (roots.length === 0 && duplicate) roots = [duplicate];

                var expandedChildren = [];
                for (var i = 0; i < roots.length; i++) {
                    try { extractRecursive(roots[i], artboardRect, expandedChildren, depth + 1, false); } catch (childError) { noteHostDiagnostic((label || "appearance") + " expanded geometry extraction failed", childError); }
                }
                if (expandedChildren.length === 0) {
                    noteHostDiagnostic((label || "appearance") + " Expand Appearance fallback produced no vector geometry", "duplicate expansion returned no extractable page items");
                    return null;
                }

                return { children: expandedChildren, source: "duplicate + Expand Appearance" };
            } catch (e) {
                noteHostDiagnostic((label || "appearance") + " Expand Appearance fallback failed", e);
                return null;
            } finally {
                closeTempDocumentWithoutSaving(tempDoc);
            }
        }
        
        function getFill(item) {
            try { if (item.filled && item.fillColor) return colorToRGB(item.fillColor); } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return null;
        }
        
        function getStroke(item, artboardRect) {
            try {
                if (item.stroked && item.strokeColor) {
                    var c = colorToRGB(item.strokeColor) || { r: 0, g: 0, b: 0, a: 255 };
                    c.width = item.strokeWidth || 1;
                    var gradient = getGradientFromColor(item.strokeColor, artboardRect);
                    if (gradient) c.gradient = gradient;
                    return c;
                }
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return null;
        }

        function normalizeStrokeAlignment(value) {
            if (value === null || value === undefined) return null;
            var raw = String(value).toLowerCase();
            if (raw.indexOf("inside") >= 0 || raw.indexOf("inner") >= 0) return "inside";
            if (raw.indexOf("outside") >= 0 || raw.indexOf("outer") >= 0) return "outside";
            if (raw.indexOf("center") >= 0 || raw.indexOf("middle") >= 0) return "center";
            var numeric = Number(value);
            if (isFinite(numeric)) {
                if (numeric === 1) return "inside";
                if (numeric === 2) return "outside";
                if (numeric === 0) return "center";
            }
            return null;
        }

        function normalizeBlendModeValue(value) {
            if (value === null || value === undefined) return null;
            var ordinalMap = {
                0: "normal", 1: "multiply", 2: "screen", 3: "overlay", 4: "darken", 5: "lighten",
                6: "color_dodge", 7: "color_burn", 8: "hard_light", 9: "soft_light", 10: "difference",
                11: "exclusion", 12: "hue", 13: "saturation", 14: "color", 15: "luminosity"
            };
            if (typeof value === "number" && isFinite(value)) return ordinalMap[Math.floor(value)] || null;
            var raw = String(value).replace(/^\s+|\s+$/g, "");
            if (!raw) return null;
            if (/^\d+$/.test(raw)) return ordinalMap[Number(raw)] || null;
            var key = raw.toLowerCase().split(".").pop().replace(/[^a-z0-9]+/g, "");
            var map = {
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
            var x = null;
            var y = null;
            if (point.length !== undefined && point.length >= 2) {
                x = Number(point[0]);
                y = Number(point[1]);
            } else {
                try {
                    x = Number(point.x !== undefined ? point.x : point[0]);
                    y = Number(point.y !== undefined ? point.y : point[1]);
                } catch (e) { return null; }
            }
            if (!isFinite(x) || !isFinite(y)) return null;
            if (!artboardRect || artboardRect.length < 2) return { x: x, y: y };
            return { x: x - Number(artboardRect[0]), y: Number(artboardRect[1]) - y };
        }

        function offsetIllustratorPoint(point, distance, angleDeg) {
            if (!point || !isFinite(distance) || !isFinite(angleDeg)) return null;
            var angle = angleDeg * Math.PI / 180;
            return { x: point.x + Math.cos(angle) * distance, y: point.y + Math.sin(angle) * distance };
        }

        function readGradientMatrix(matrix, artboardRect) {
            if (!matrix) return null;
            function read(names) {
                for (var i = 0; i < names.length; i++) {
                    var value = Number(matrix[names[i]]);
                    if (isFinite(value)) return value;
                }
                return null;
            }
            var a, b, c, d, e, f;
            if (matrix.length !== undefined && matrix.length >= 6) {
                a = Number(matrix[0]); b = Number(matrix[1]); c = Number(matrix[2]);
                d = Number(matrix[3]); e = Number(matrix[4]); f = Number(matrix[5]);
            } else {
                a = read(["a", "mValueA"]); b = read(["b", "mValueB"]);
                c = read(["c", "mValueC"]); d = read(["d", "mValueD"]);
                e = read(["e", "tx", "mValueTX"]); f = read(["f", "ty", "mValueTY"]);
            }
            if (!isFinite(a) || !isFinite(b) || !isFinite(c) || !isFinite(d) || !isFinite(e) || !isFinite(f)) return null;
            if (!artboardRect || artboardRect.length < 2) return [a, b, c, d, e, f];
            var left = Number(artboardRect[0]);
            var top = Number(artboardRect[1]);
            return [a, -b, -c, d, a * left + c * top + e - left, top - b * left - d * top - f];
        }
        
        function colorToHex(c) {
            if (!c) return undefined;
            var r = Math.max(0, Math.min(255, c.r));
            var g = Math.max(0, Math.min(255, c.g));
            var b = Math.max(0, Math.min(255, c.b));
            var toHex = function(v) { var s = v.toString(16); return s.length < 2 ? "0" + s : s; };
            return "#" + toHex(r) + toHex(g) + toHex(b);
        }

        function gradientColorToRGB(c) {
            if (!c) return { r: 128, g: 128, b: 128 };
            try {
                if (c.typename === "RGBColor") return { r: Math.round(c.red), g: Math.round(c.green), b: Math.round(c.blue) };
                if (c.typename === "CMYKColor") { var k = c.black/100; return { r: Math.round(255*(1-c.cyan/100)*(1-k)), g: Math.round(255*(1-c.magenta/100)*(1-k)), b: Math.round(255*(1-c.yellow/100)*(1-k)) }; }
                if (c.typename === "GrayColor") { var v = Math.round(255*(1-c.gray/100)); return { r: v, g: v, b: v }; }
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return { r: 128, g: 128, b: 128 };
        }

        function getGradientFromColor(color, artboardRect) {
            if (!color) return null;
            try {
                if (color.typename === "GradientColor") {
                    var grad = color.gradient;
                    if (!grad) return null;
                    var angle = color.angle || 0;
                    var stops = [];
                    try {
                        for (var si = 0; si < grad.gradientStops.length; si++) {
                            var s = grad.gradientStops[si];
                            var sc = gradientColorToRGB(s.color);
                            stops.push({ position: s.rampPoint/100, color: colorToHex(sc), opacity: s.opacity !== undefined ? s.opacity/100 : 1 });
                        }
                    } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
                    var origin = illustratorPointToEgui(color.origin, artboardRect);
                    var length = Number(color.length);
                    var hiliteLength = Number(color.hiliteLength);
                    var hiliteAngle = Number(color.hiliteAngle);
                    var focalPoint = (isFinite(hiliteLength) && isFinite(hiliteAngle))
                        ? offsetIllustratorPoint(origin, hiliteLength, -hiliteAngle)
                        : origin;
                    var transform = readGradientMatrix(color.matrix, artboardRect);
                    return {
                        type: grad.type === 1 ? "linear" : "radial",
                        angle: angle,
                        center: origin,
                        focalPoint: focalPoint,
                        radius: isFinite(length) && length > 0 ? length : null,
                        transform: transform,
                        stops: stops
                    };
                }
                if (color.typename === "PatternColor") {
                    var patternResult = {
                        type: "pattern",
                        patternName: color.pattern ? color.pattern.name : "unknown",
                        rotation: color.rotation || 0,
                        scale: patternScaleFromColor(color)
                    };
                    var swatch = patternSwatchFromColor(color, artboardRect);
                    if (swatch) {
                        for (var swatchKey in swatch) if (swatch.hasOwnProperty(swatchKey)) patternResult[swatchKey] = swatch[swatchKey];
                    }
                    return patternResult;
                }
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return null;
        }

        function getGradient(item, artboardRect) {
            return getGradientFromColor(item && item.fillColor, artboardRect);
        }

        function extractEffects(item) {
            var fx = [];
            try {
                // Try to detect Illustrator raster/vector effects via XMPString (CS5+)
                try {
                    var xmp = item.XMPString;
                    var xmpEffects = effectsFromMetadataText(xmp);
                    for (var xi = 0; xi < xmpEffects.length; xi++) {
                        mergeEffectByType(fx, xmpEffects[xi]);
                    }
                } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return fx;
        }

        function maxCountFromRegex(raw, regex) {
            var max = 0;
            var match;
            while ((match = regex.exec(raw)) !== null) {
                var value = Number(match[1]);
                if (isFinite(value) && value > max) max = value;
            }
            return max;
        }

        function appearanceProbeFromMetadataText(text, source) {
            var raw = String(text || "");
            if (!raw) return null;
            var explicitFillCount = Math.max(
                maxCountFromRegex(raw, /(?:appearance[-_\s]*)?fills?[-_\s]*count\s*[:=]\s*["']?(\d+)/gi),
                maxCountFromRegex(raw, /(?:appearance[-_\s]*)?fillcount\s*[:=]\s*["']?(\d+)/gi)
            );
            var explicitStrokeCount = Math.max(
                maxCountFromRegex(raw, /(?:appearance[-_\s]*)?strokes?[-_\s]*count\s*[:=]\s*["']?(\d+)/gi),
                maxCountFromRegex(raw, /(?:appearance[-_\s]*)?strokecount\s*[:=]\s*["']?(\d+)/gi)
            );
            var fillOps = 0;
            var strokeOps = 0;
            var arrayPaintRe = /\[\s*-?\d*\.?\d+\s+-?\d*\.?\d+\s+-?\d*\.?\d+\s+-?\d*\.?\d+\s*\]\s*([Xx][aA])\b/g;
            var opMatch;
            while ((opMatch = arrayPaintRe.exec(raw)) !== null) {
                if (String(opMatch[1]).charAt(0) === "X") fillOps++;
                else strokeOps++;
            }
            var fillCount = Math.max(explicitFillCount, fillOps);
            var strokeCount = Math.max(explicitStrokeCount, strokeOps);
            if (fillCount <= 1 && strokeCount <= 1) return null;
            return { fillCount: fillCount, strokeCount: strokeCount, source: source || "metadata" };
        }

        function maxAppearanceProbe(a, b) {
            if (!a) return b || null;
            if (!b) return a || null;
            return {
                fillCount: Math.max(Number(a.fillCount || 0), Number(b.fillCount || 0)),
                strokeCount: Math.max(Number(a.strokeCount || 0), Number(b.strokeCount || 0)),
                source: [a.source, b.source].join("+")
            };
        }

        function extractAppearanceProbe(item) {
            var probe = null;
            try { probe = maxAppearanceProbe(probe, appearanceProbeFromMetadataText(item && item.XMPString, "XMPString")); } catch (e) { noteHostDiagnostic("optional Illustrator appearance metadata unavailable", e); }
            try { probe = maxAppearanceProbe(probe, appearanceProbeFromMetadataText(item && item.note, "note")); } catch (e) { noteHostDiagnostic("optional Illustrator appearance note unavailable", e); }
            try {
                if (item && item.tags) {
                    for (var i = 0; i < item.tags.length; i++) {
                        var tag = item.tags[i];
                        probe = maxAppearanceProbe(probe, appearanceProbeFromMetadataText(String(tag.name || "") + "=" + String(tag.value || ""), "tag"));
                    }
                }
            } catch (e) { noteHostDiagnostic("optional Illustrator appearance tags unavailable", e); }
            return probe;
        }

        function textFontWeight(fontName) {
            var name = fontName || "";
            if (name.indexOf("Bold") !== -1) return 700;
            if (name.indexOf("Light") !== -1) return 300;
            return 400;
        }

        function illustratorTrackingToPx(tracking, fontSize) {
            var t = Number(tracking);
            var size = Number(fontSize) || 14;
            if (!isFinite(t) || t === 0) return null;
            return (t / 1000) * size;
        }

        function illustratorLeadingToMultiplier(leading, fontSize) {
            var l = Number(leading);
            var size = Number(fontSize) || 14;
            if (!isFinite(l) || l <= 0 || size <= 0) return null;
            return l / size;
        }

        function getOpenTypeFeatures(attrs) {
            if (!attrs) return null;
            function read(name) {
                try { return attrs[name]; }
                catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); return undefined; }
            }
            function bool(name) {
                var value = read(name);
                return value === undefined ? undefined : !!value;
            }
            var features = {};
            function setIfOverride(name, value, defaultValue) {
                if (value !== undefined && value !== defaultValue) features[name] = value;
            }
            setIfOverride("ligatures", bool("ligatures"), true);
            setIfOverride("contextualLigatures", bool("contextualLigatures"), true);
            setIfOverride("discretionaryLigatures", bool("discretionaryLigatures"), false);
            setIfOverride("fractions", bool("fractions"), false);
            setIfOverride("ordinals", bool("ordinals"), false);
            setIfOverride("swash", bool("swash"), false);
            setIfOverride("titlingAlternates", bool("titlingAlternates"), false);
            setIfOverride("stylisticAlternates", bool("stylisticAlternates"), false);
            var kerningMethod = read("kerningMethod");
            if (kerningMethod !== undefined) {
                var normalized = String(kerningMethod).toLowerCase();
                if (kerningMethod === false || normalized.indexOf("none") !== -1 || normalized.indexOf("off") !== -1) features.kerning = false;
            }
            return Object.keys(features).length > 0 ? features : null;
        }

        function getTextMetricsOverrides(attrs) {
            if (!attrs) return null;
            try {
                var metrics = {};
                var bs = Number(attrs.baselineShift);
                var hs = Number(attrs.horizontalScale);
                var vs = Number(attrs.verticalScale);
                if (isFinite(bs) && Math.abs(bs) > 0.0001) metrics.baselineShift = bs;
                if (isFinite(hs) && Math.abs(hs - 100) > 0.0001) metrics.horizontalScale = hs / 100;
                if (isFinite(vs) && Math.abs(vs - 100) > 0.0001) metrics.verticalScale = vs / 100;
                return Object.keys(metrics).length > 0 ? metrics : null;
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return null;
        }

        function getTextStyle(item) {
            try {
                var chars = item.textRange.characterAttributes;
                var size = 14, weight = 400, family = "default";
                try { size = chars.size || 14; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
                try { if (chars.textFont) { family = chars.textFont.name || ""; weight = textFontWeight(family); } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
                var result = {
                    size: size,
                    fontSize: size,
                    weight: weight,
                    family: family
                };
                var otf = getOpenTypeFeatures(chars);
                var metrics = getTextMetricsOverrides(chars);
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
                var j = item.textRange.paragraphAttributes.justification;
                var name = String(j || "").toUpperCase();
                if (typeof Justification !== "undefined" && j === Justification.LEFT) return "left";
                if (typeof Justification !== "undefined" && j === Justification.CENTER) return "center";
                if (typeof Justification !== "undefined" && j === Justification.RIGHT) return "right";
                if (typeof Justification !== "undefined") {
                    if (j === Justification.FULLJUSTIFYLASTLINECENTER) return "justified_last_line_center";
                    if (j === Justification.FULLJUSTIFYLASTLINERIGHT) return "justified_last_line_right";
                    if (j === Justification.FULLJUSTIFYLASTLINELEFT) return "justified";
                    if (j === Justification.FULLJUSTIFY) return "justified_all";
                }
                if (name.indexOf("LASTLINECENTER") !== -1) return "justified_last_line_center";
                if (name.indexOf("LASTLINERIGHT") !== -1) return "justified_last_line_right";
                if (name.indexOf("JUSTIFY") !== -1) return name.indexOf("LASTLINELEFT") !== -1 ? "justified" : "justified_all";
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return "left";
        }

        function getLetterSpacing(item) {
            if (item.typename !== "TextFrame") return null;
            try {
                var attrs = item.textRange.characterAttributes;
                return illustratorTrackingToPx(attrs.tracking, attrs.size || 14);
            } catch (e) { return null; }
        }

        function getLineHeight(item) {
            if (item.typename !== "TextFrame") return null;
            try {
                var attrs = item.textRange.characterAttributes;
                return illustratorLeadingToMultiplier(attrs.leading, attrs.size || 14);
            } catch (e) { return null; }
        }

        function getTextDecoration(item) {
            if (item.typename !== "TextFrame") return null;
            try {
                var u = item.textRange.characterAttributes.underline;
                var s = item.textRange.characterAttributes.strikeThrough;
                if (u && s) return "both";
                if (u) return "underline";
                if (s) return "strikethrough";
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return null;
        }

        function getTextTransform(item) {
            if (item.typename !== "TextFrame") return null;
            try {
                if (item.textRange.characterAttributes.smallCaps) return "small_caps";
                if (item.textRange.characterAttributes.allCaps) return "uppercase";
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return null;
        }

        function pathItemToOutlinedContour(pathItem, artboardRect) {
            var contour = getPathPoints(pathItem, artboardRect);
            if (!contour || !contour.points || contour.points.length === 0) return null;
            return { points: contour.points, closed: contour.closed !== false };
        }

        function collectOutlinedContours(item, artboardRect, contours) {
            if (!item) return contours;
            if (!contours) contours = [];
            try {
                if (item.typename === "PathItem") {
                    var pathContour = pathItemToOutlinedContour(item, artboardRect);
                    if (pathContour) contours.push(pathContour);
                    return contours;
                }
                if (item.typename === "CompoundPathItem" && item.pathItems) {
                    for (var ci = 0; ci < item.pathItems.length; ci++) {
                        collectOutlinedContours(item.pathItems[ci], artboardRect, contours);
                    }
                    return contours;
                }
                if (item.typename === "GroupItem") {
                    var children = [];
                    try {
                        if (item.pageItems) children = collectionToArray(item.pageItems, 256);
                        else if (item.pathItems) children = collectionToArray(item.pathItems, 256);
                    } catch (childError) { noteHostDiagnostic("text shaping outlined group children unavailable", childError); }
                    for (var gi = 0; gi < children.length; gi++) {
                        collectOutlinedContours(children[gi], artboardRect, contours);
                    }
                    return contours;
                }
            } catch (e) { noteHostDiagnostic("text shaping outlined contour collection failed", e); }
            return contours;
        }

        function outlinedContourBounds(contours) {
            var bounds = null;
            if (!contours || contours.length === 0) return bounds;
            for (var ci = 0; ci < contours.length; ci++) {
                var contour = contours[ci];
                if (!contour || !contour.points) continue;
                for (var pi = 0; pi < contour.points.length; pi++) {
                    var point = contour.points[pi];
                    if (!point || point.length < 2) continue;
                    var x = Number(point[0]);
                    var y = Number(point[1]);
                    if (!isFinite(x) || !isFinite(y)) continue;
                    if (!bounds) bounds = { minX: x, minY: y, maxX: x, maxY: y };
                    else {
                        if (x < bounds.minX) bounds.minX = x;
                        if (y < bounds.minY) bounds.minY = y;
                        if (x > bounds.maxX) bounds.maxX = x;
                        if (y > bounds.maxY) bounds.maxY = y;
                    }
                }
            }
            return bounds;
        }

        function outlinedGlyphFromItem(item, artboardRect, glyphIndex) {
            var contours = collectOutlinedContours(item, artboardRect, []);
            if (!contours || contours.length === 0) return null;
            var bounds = outlinedContourBounds(contours);
            var advanceX = 0;
            if (bounds) advanceX = Math.max(0, bounds.maxX - bounds.minX);
            try {
                if (!advanceX && item && item.geometricBounds) {
                    var gb = item.geometricBounds;
                    advanceX = Math.max(0, Number(gb[2]) - Number(gb[0]));
                }
            } catch (e) { noteHostDiagnostic("text shaping outlined advance fallback unavailable", e); }
            return {
                glyphId: glyphIndex,
                cluster: glyphIndex,
                advanceX: advanceX,
                advanceY: 0,
                offsetX: 0,
                offsetY: 0,
                contoursAbsolute: true,
                contours: contours
            };
        }

        function outlinedGlyphsFromRoots(roots, artboardRect) {
            var glyphs = [];
            var glyphIndex = 0;
            function appendGlyph(item) {
                var glyph = outlinedGlyphFromItem(item, artboardRect, glyphIndex);
                if (glyph) {
                    glyphs.push(glyph);
                    glyphIndex += 1;
                }
            }
            try {
                for (var ri = 0; ri < roots.length; ri++) {
                    var root = roots[ri];
                    if (!root) continue;
                    if (root.typename === "GroupItem") {
                        var children = [];
                        try {
                            if (root.pageItems) children = collectionToArray(root.pageItems, 256);
                            else if (root.pathItems) children = collectionToArray(root.pathItems, 256);
                        } catch (groupChildrenError) { noteHostDiagnostic("text shaping outlined root children unavailable", groupChildrenError); }
                        if (children.length > 0) {
                            for (var ci = 0; ci < children.length; ci++) appendGlyph(children[ci]);
                            continue;
                        }
                    }
                    appendGlyph(root);
                }
            } catch (e) { noteHostDiagnostic("text shaping outlined glyph walk failed", e); }
            return glyphs;
        }

        function textRequiresShapingContract(item) {
            if (item.typename !== "TextFrame") return false;
            try {
                var attrs = item.textRange.characterAttributes;
                var features = getOpenTypeFeatures(attrs) || {};
                return features.ligatures === false
                    || attrs.contextualLigatures === false
                    || attrs.ligatures === false
                    || attrs.contextual_ligatures === false
                    || attrs.discretionaryLigatures === true
                    || attrs.discretionary_ligatures === true
                    || attrs.fractions === true
                    || attrs.ordinals === true
                    || attrs.swash === true
                    || attrs.titlingAlternates === true
                    || attrs.titling_alternates === true
                    || attrs.stylisticAlternates === true
                    || attrs.stylistic_alternates === true
                    || attrs.kerning === false
                    || features.kerning === false
                    || attrs.smallCaps === true;
            } catch (e) {
                noteHostDiagnostic("optional Illustrator text shaping inspection unavailable", e);
            }
            return false;
        }

        function approximateOutlinedGlyphContract(item, el, reason) {
            return null;
        }

        function extractTextShapingContract(item, artboardRect, el) {
            if (!textRequiresShapingContract(item)) return null;
            var tempDoc = null;
            var duplicate = null;
            try {
                if (typeof app === "undefined" || !app || !app.documents || typeof ElementPlacement === "undefined" || ElementPlacement.PLACEATEND === undefined) {
                    noteHostDiagnostic("text shaping contract fallback unavailable", "Illustrator duplicate/menu command APIs unavailable");
                    return approximateOutlinedGlyphContract(item, el, "duplicate + createOutlines fallback unavailable");
                }

                var width = Math.max(1, Math.ceil(Math.abs(Number(artboardRect && artboardRect[2] || 1) - Number(artboardRect && artboardRect[0] || 0))));
                var height = Math.max(1, Math.ceil(Math.abs(Number(artboardRect && artboardRect[1] || 1) - Number(artboardRect && artboardRect[3] || 0))));
                try {
                    if (typeof DocumentColorSpace !== "undefined" && DocumentColorSpace.RGB !== undefined) tempDoc = app.documents.add(DocumentColorSpace.RGB, width, height);
                } catch (docColorError) { noteHostDiagnostic("text shaping temp document color setup failed", docColorError); }
                if (!tempDoc) tempDoc = app.documents.add();

                try { if (tempDoc.artboards && tempDoc.artboards.length > 0) tempDoc.artboards[0].artboardRect = [0, height, width, 0]; } catch (artboardError) { noteHostDiagnostic("text shaping temp artboard setup failed", artboardError); }

                var target = (tempDoc.layers && tempDoc.layers.length > 0) ? tempDoc.layers[0] : tempDoc;
                duplicate = item.duplicate(target, ElementPlacement.PLACEATEND);
                try { if (typeof tempDoc.activate === "function") tempDoc.activate(); } catch (activateError) { noteHostDiagnostic("text shaping temp document activate failed", activateError); }
                try { tempDoc.selection = [duplicate]; } catch (selectionError) { noteHostDiagnostic("text shaping temp selection unavailable", selectionError); }
                try { app.selection = [duplicate]; } catch (appSelectionError) { noteHostDiagnostic("text shaping application selection unavailable", appSelectionError); }

                var outlined = false;
                try {
                    app.executeMenuCommand("createOutlines");
                    outlined = true;
                } catch (outlineError) {
                    noteHostDiagnostic("text shaping createOutlines fallback failed", outlineError);
                }

                var roots = [];
                try { if (tempDoc.selection && tempDoc.selection.length > 0) roots = collectionToArray(tempDoc.selection, 256); } catch (selectionReadError) { noteHostDiagnostic("text shaping outlined selection unavailable", selectionReadError); }
                if (roots.length === 0) {
                    try { roots = collectionToArray(tempDoc.pageItems, 256); } catch (pageItemsError) { noteHostDiagnostic("text shaping outlined pageItems unavailable", pageItemsError); }
                }
                var outlinedGlyphs = outlinedGlyphsFromRoots(roots, artboardRect);
                if (outlined && outlinedGlyphs.length > 0) {
                    return { outlinedGlyphs: outlinedGlyphs, source: "outlined glyph contours via createOutlines" };
                }
                if (outlined && roots.length > 0) {
                    noteHostDiagnostic("text shaping contract extraction incomplete", "outlined glyph paths observed but no glyph contours could be collected");
                }
            } catch (e) {
                noteHostDiagnostic("text shaping contract extraction failed", e);
            } finally {
                closeTempDocumentWithoutSaving(tempDoc);
            }
            return null;
        }

        function getTextRuns(item) {
            if (item.typename !== "TextFrame") return null;
            try {
                var runs = [];
                var trs = item.textRanges;
                if (trs && trs.length > 1) {
                    for (var ri = 0; ri < trs.length; ri++) {
                        try {
                            var tr = trs[ri];
                            var a = tr.characterAttributes;
                            var runColor = null;
                            try { runColor = colorToRGB(a.fillColor); } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
                            var fontName = a.textFont && a.textFont.name ? a.textFont.name : null;
                            var style = {
                                size: a.size || 14,
                                fontSize: a.size || 14,
                                weight: textFontWeight(fontName),
                                family: fontName,
                                color: runColor,
                                letterSpacing: illustratorTrackingToPx(a.tracking, a.size || 14),
                                lineHeight: illustratorLeadingToMultiplier(a.leading, a.size || 14),
                                textDecoration: (a.underline && a.strikeThrough) ? "both" : a.underline ? "underline" : a.strikeThrough ? "strikethrough" : null,
                                textTransform: a.smallCaps ? "small_caps" : a.allCaps ? "uppercase" : null
                            };
                            var otf = getOpenTypeFeatures(a);
                            var metrics = getTextMetricsOverrides(a);
                            if (otf) style.openTypeFeatures = otf;
                            if (metrics) {
                                if (metrics.baselineShift !== undefined) style.baselineShift = metrics.baselineShift;
                                if (metrics.horizontalScale !== undefined) style.horizontalScale = metrics.horizontalScale;
                                if (metrics.verticalScale !== undefined) style.verticalScale = metrics.verticalScale;
                            }
                            runs.push({ text: tr.contents || "", style: style });
                        } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
                    }
                }
                return runs.length > 0 ? runs : null;
            } catch(e) { return null; }
        }

        function getCompoundFillRule(item) {
            // Illustrator scripting does not expose a reliable compound-path fill rule here.
            // Keep this explicit so strict export requires parser-side fill_rule metadata.
            return null;
        }

        function getPathPoints(item, artboardRect) {
            try {
                function mapPathPoints(pathItem) {
                    var pts = [];
                    if (!pathItem || !pathItem.pathPoints) return pts;
                    for (var pi = 0; pi < pathItem.pathPoints.length; pi++) {
                        var pp = pathItem.pathPoints[pi];
                        try {
                            pts.push({
                                anchor: [pp.anchor[0] - artboardRect[0], artboardRect[1] - pp.anchor[1]],
                                leftDir: [pp.leftDirection[0] - artboardRect[0], artboardRect[1] - pp.leftDirection[1]],
                                rightDir: [pp.rightDirection[0] - artboardRect[0], artboardRect[1] - pp.rightDirection[1]],
                                left_ctrl: [pp.leftDirection[0] - artboardRect[0], artboardRect[1] - pp.leftDirection[1]],
                                right_ctrl: [pp.rightDirection[0] - artboardRect[0], artboardRect[1] - pp.rightDirection[1]],
                                kind: pp.pointType === PointType.SMOOTH ? "smooth" : "corner"
                            });
                        } catch (ppe) { noteHostDiagnostic("optional Illustrator property unavailable", ppe); }
                    }
                    return pts;
                }

                if (item.typename === "CompoundPathItem" && item.pathItems) {
                    var subpaths = [];
                    for (var si = 0; si < item.pathItems.length; si++) {
                        var childPath = item.pathItems[si];
                        var childPoints = mapPathPoints(childPath);
                        if (childPoints.length > 0) subpaths.push({ points: childPoints, closed: childPath.closed !== false });
                    }
                    if (subpaths.length > 0) return { points: subpaths[0].points, closed: subpaths[0].closed, subpaths: subpaths, fillRule: getCompoundFillRule(item) };
                }

                if (item.typename === "PathItem" || item.typename === "CompoundPathItem") {
                    var pts = mapPathPoints(item);
                    if (pts.length > 0) return { points: pts, closed: item.closed || false };
                }
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return null;
        }
        
        function getElementType(item) {
            try {
                var t = item.typename;
                if (t === "TextFrame") return "text";
                if (t === "PathItem") {
                    if (!item.closed) return "path";
                    // Detect circle/ellipse: 4 smooth points
                    try {
                        if (item.pathPoints && item.pathPoints.length === 4) {
                            var allSmooth = true;
                            for (var pi = 0; pi < item.pathPoints.length; pi++) {
                                if (item.pathPoints[pi].pointType !== PointType.SMOOTH) { allSmooth = false; break; }
                            }
                            if (allSmooth) {
                                var bb = item.geometricBounds;
                                var bw = Math.abs(bb[2] - bb[0]), bh = Math.abs(bb[1] - bb[3]);
                                var ratio = (bw > 0 && bh > 0) ? Math.min(bw, bh) / Math.max(bw, bh) : 0;
                                if (ratio > 0.985) return "circle";
                                return "ellipse";
                            }
                        }
                    } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
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
        
        function extractRecursive(item, artboardRect, elements, depth, allowProgrammaticExpansion) {
            if (allowProgrammaticExpansion === undefined) allowProgrammaticExpansion = true;
            try {
                if (item.locked || item.hidden) {
                    return;
                }
            } catch (e) { noteHostDiagnostic("extract skip state error", e); return; }
            
            var x = 0, y = 0, w = 0, h = 0;
            try {
                var b = item.geometricBounds;
                x = b[0] - artboardRect[0]; y = artboardRect[1] - b[1];
                w = Math.abs(b[2] - b[0]); h = Math.abs(b[1] - b[3]);
            } catch (e) {
                try {
                    var b2 = item.visibleBounds;
                    x = b2[0] - artboardRect[0]; y = artboardRect[1] - b2[1];
                    w = Math.abs(b2[2] - b2[0]); h = Math.abs(b2[1] - b2[3]);
                } catch (e2) { noteHostDiagnostic("extract skip bounds", e2); return; }
            }
            
            var el = {
                id: item.name || ("el_" + elements.length),
                type: getElementType(item),
                x: x, y: y, w: w, h: h, depth: depth,
                fill: getFill(item),
                stroke: getStroke(item, artboardRect),
                text: null, textStyle: null, textRuns: null,
                textAlign: null, letterSpacing: null, lineHeight: null,
                textDecoration: null, textTransform: null,
                shapedGlyphs: null, outlinedGlyphs: null,
                children: [],
                opacity: 1.0, rotation: 0, cornerRadius: 0,
                gradient: null, blendMode: "normal",
                effects: [], notes: [],
                appearanceProbe: null,
                pathPoints: null, pathClosed: false, subpaths: null, fillRule: null,
                imagePath: null, extractedImagePath: null, extractedRasterAlreadyTransformed: false,
                rasterScaleX: null, rasterScaleY: null, embeddedRaster: false, symbolName: null,
                isChart: false, isGradientMesh: false, isCompoundPath: false,
                strokeCap: null, strokeJoin: null, strokeAlignment: null
            };
            
            try { el.opacity = item.opacity !== undefined ? item.opacity / 100 : 1; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            el.rotation = extractItemRotationDeg(item);
            try { if (item.typename === "PathItem" && item.cornerRadius !== undefined) el.cornerRadius = item.cornerRadius; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.strokeCap !== undefined) el.strokeCap = ({0:"butt",1:"round",2:"square"})[item.strokeCap] || "butt"; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.strokeJoin !== undefined) el.strokeJoin = ({0:"miter",1:"round",2:"bevel"})[item.strokeJoin] || "miter"; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.strokeDashes && item.strokeDashes.length > 0) { var dashes = []; for(var di=0; di<item.strokeDashes.length; di++) dashes.push(item.strokeDashes[di]); el.strokeDash = dashes; } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.strokeMiterLimit !== undefined) el.strokeMiterLimit = item.strokeMiterLimit; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.strokeAlignment !== undefined) el.strokeAlignment = normalizeStrokeAlignment(item.strokeAlignment); } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            
            // Blend mode
            try {
                if (item.blendingMode !== undefined) el.blendMode = normalizeBlendModeValue(item.blendingMode) || "normal";
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            
            // Gradient
            el.gradient = getGradient(item, artboardRect);
            el.appearanceProbe = extractAppearanceProbe(item);

            if (allowProgrammaticExpansion && item.typename !== "TextFrame" && el.appearanceProbe && ((el.appearanceProbe.fillCount || 0) > 1 || (el.appearanceProbe.strokeCount || 0) > 1)) {
                try {
                    var appearanceExpansion = expandAppearanceViaDuplicate(item, artboardRect, el, depth, "appearance");
                    if (appearanceExpansion && appearanceExpansion.children && appearanceExpansion.children.length > 0) {
                        el.type = "group";
                        el.children = appearanceExpansion.children;
                        el.appearanceExpanded = true;
                        el.appearanceExpansionSource = appearanceExpansion.source;
                        el.appearanceProbe = null;
                        el.fill = null;
                        el.stroke = null;
                        el.gradient = null;
                        el.effects = [];
                        el.text = null;
                        el.textStyle = null;
                        el.textRuns = null;
                        ensureElementNotes(el).push("appearance expanded via duplicate + Expand Appearance fallback; original document left untouched");
                    } else {
                        ensureElementNotes(el).push("appearance: duplicate + Expand Appearance fallback unavailable; strict export may require expanded vector geometry");
                    }
                } catch (e2) { noteHostDiagnostic("appearance Expand Appearance fallback unavailable", e2); }
            }
            
            // Effects
            if (!el.appearanceExpanded) el.effects = extractEffects(item);
            if (allowProgrammaticExpansion && (item.typename === "PluginItem" || hasUnsupportedLiveEffectExtraction(el))) {
                var expansion = null;
                if (item.typename === "PluginItem" || hasUnsupportedLiveEffectExtraction(el)) {
                    try { expansion = expandAppearanceViaDuplicate(item, artboardRect, el, depth, item.typename === "PluginItem" ? "plugin item" : "live effect"); } catch (e2) { noteHostDiagnostic("appearance Expand Appearance fallback unavailable", e2); }
                    if (expansion && expansion.children && expansion.children.length > 0) {
                        el.type = "group";
                        el.children = expansion.children;
                        el.appearanceExpanded = true;
                        el.appearanceExpansionSource = expansion.source;
                        ensureElementNotes(el).push("appearance expanded via duplicate + Expand Appearance fallback; original document left untouched");
                        el.effects = [];
                    } else if (item.typename === "PluginItem" || hasUnsupportedLiveEffectExtraction(el)) {
                        ensureElementNotes(el).push("appearance: duplicate + Expand Appearance fallback unavailable; strict export may require expanded vector geometry");
                    }
                }
            }
            
            // Path points
            var ppResult = getPathPoints(item, artboardRect);
            if (ppResult) { el.pathPoints = ppResult.points; el.pathClosed = ppResult.closed; el.subpaths = ppResult.subpaths || [{ points: ppResult.points, closed: ppResult.closed }]; el.fillRule = ppResult.fillRule || el.fillRule; }
            
            // Image path. Keep the original filesystem path here; plugin.js derives
            // the portable assets/... path later while saveFilesToFolderJSON uses
            // this raw source path for the actual copy.
            try { if (item.typename === "PlacedItem" && item.file) el.imagePath = item.file.fsName || item.file.name || null; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try {
                if (item.typename === "PlacedItem" || item.typename === "RasterItem") {
                    var transformScale = extractRasterTransformScale(item);
                    if (transformScale) {
                        el.rasterScaleX = transformScale.scaleX;
                        el.rasterScaleY = transformScale.scaleY;
                    }
                }
            } catch (e) { noteHostDiagnostic("optional Illustrator raster transform unavailable", e); }
            try {
                if (item.typename === "RasterItem") {
                    el.embeddedRaster = true;
                    ensureElementNotes(el).push("embedded raster image");
                    var extractedImagePath = extractEmbeddedRasterToTempPng(item, el);
                    if (extractedImagePath) {
                        el.extractedImagePath = extractedImagePath;
                        el.extractedRasterAlreadyTransformed = true;
                        ensureElementNotes(el).push("embedded raster extracted for vector tracing");
                    }
                }
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            
            // Symbol
            try {
                if (item.typename === "SymbolItem") {
                    el.type = "symbol";
                    el.symbolName = item.symbol ? item.symbol.name : "unknown";
                    if (!el.symbolExpanded && (!el.children || el.children.length === 0)) {
                        if (!expandSymbolDefinitionIntoElement(item, artboardRect, el, depth)) {
                            if (allowProgrammaticExpansion) {
                                var symbolExpansion = expandAppearanceViaDuplicate(item, artboardRect, el, depth, "symbol");
                                if (symbolExpansion && symbolExpansion.children && symbolExpansion.children.length > 0) {
                                    el.children = symbolExpansion.children;
                                    el.symbolExpanded = true;
                                    el.symbolExpansionSource = symbolExpansion.source;
                                    ensureElementNotes(el).push("symbol instance expanded via duplicate + Expand Appearance fallback; original document left untouched");
                                    el.effects = [];
                                } else {
                                    ensureElementNotes(el).push("Symbol instance: \"" + el.symbolName + "\" — duplicate + Expand Appearance fallback unavailable; expand symbol before strict export");
                                }
                            } else {
                                ensureElementNotes(el).push("Symbol instance: \"" + el.symbolName + "\" — definition artwork unavailable; expand symbol before strict export");
                            }
                        }
                    }
                }
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            
            // Flags
            try { if (item.typename === "CompoundPathItem") { el.isCompoundPath = true; el.fillRule = getCompoundFillRule(item); ensureElementNotes(el).push("compound path"); } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.typename === "MeshItem") { el.isGradientMesh = true; ensureElementNotes(el).push("gradient mesh"); } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.typename === "GraphItem") { el.isChart = true; ensureElementNotes(el).push("chart/graph"); } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.clipping || item.clipped) { el.clipMask = true; ensureElementNotes(el).push("clipping mask"); } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            
            // Text
            if (item.typename === "TextFrame") {
                try { el.text = item.contents || ""; } catch (e) { el.text = ""; }
                el.textStyle = getTextStyle(item);
                el.textAlign = getTextAlign(item);
                el.letterSpacing = getLetterSpacing(item);
                el.lineHeight = getLineHeight(item);
                el.textDecoration = getTextDecoration(item);
                el.textTransform = getTextTransform(item);
                el.textRuns = getTextRuns(item);
                try {
                    var textShapingContract = extractTextShapingContract(item, artboardRect, el);
                    if (textShapingContract) {
                        el.shapedGlyphs = textShapingContract.shapedGlyphs;
                        el.outlinedGlyphs = textShapingContract.outlinedGlyphs;
                        ensureElementNotes(el).push("text shaping contract: " + textShapingContract.source);
                    }
                } catch (textShapingError) { noteHostDiagnostic("optional Illustrator text shaping contract unavailable", textShapingError); }
            }
            
            // Group children
            if (item.typename === "GroupItem") {
                try {
                    if (item.pageItems) {
                        for (var ci = 0; ci < item.pageItems.length; ci++) {
                            extractRecursive(item.pageItems[ci], artboardRect, el.children, depth + 1, allowProgrammaticExpansion);
                        }
                    }
                } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            }
            elements.push(el);
        }

        for (var i = 0; i < selectedIndices.length; i++) {
            var idx = selectedIndices[i];
            var ab = doc.artboards[idx];
            var rect = ab.artboardRect;
            var abInfo = { name: ab.name, index: idx, width: Math.abs(rect[2] - rect[0]), height: Math.abs(rect[3] - rect[1]), x: rect[0], y: rect[1], bounds: [rect[0], rect[1], rect[2], rect[3]] };
            var items = [];
            var scanStats = { total: 0, lockedHidden: 0, boundsFailed: 0, outside: 0, nested: 0, included: 0 };
            for (var j = 0; j < doc.pageItems.length; j++) {
                var it = doc.pageItems[j];
                scanStats.total += 1;
                try {
                    if (it.locked || it.hidden) { scanStats.lockedHidden += 1; continue; }
                    var b = it.geometricBounds;
                    var overlaps = b[2] > rect[0] && b[0] < rect[2] && b[1] > rect[3] && b[3] < rect[1];
                    if (!overlaps) { scanStats.outside += 1; continue; }
                    if (!isTopLevelItem(it)) { scanStats.nested += 1; continue; }
                    if (overlaps) {
                        items.push(it);
                        scanStats.included += 1;
                    }
                } catch (e) { scanStats.boundsFailed += 1; noteHostDiagnostic("optional Illustrator property unavailable", e); }
            }
            var els = [];
            for (var k = 0; k < items.length; k++) {
                extractRecursive(items[k], rect, els, 0);
            }
            results.push({ artboard: abInfo, elements: els, documentPath: getDocumentPath(doc) });
        }
        
        for (var i = 0; i < selectedTiles.length; i++) {
            var tile = selectedTiles[i];
            var rect = [tile.x, tile.y, tile.x + tile.width, tile.y - tile.height];
            var abInfo = { name: tile.name, width: tile.width, height: tile.height, x: tile.x, y: tile.y, bounds: [rect[0], rect[1], rect[2], rect[3]] };
            var items = [];
            var tileScanStats = { total: 0, lockedHidden: 0, boundsFailed: 0, outside: 0, nested: 0, included: 0 };
            for (var j = 0; j < doc.pageItems.length; j++) {
                var it = doc.pageItems[j];
                tileScanStats.total += 1;
                try {
                    if (it.locked || it.hidden) { tileScanStats.lockedHidden += 1; continue; }
                    var b = it.geometricBounds;
                    var overlaps = b[2] > rect[0] && b[0] < rect[2] && b[1] > rect[3] && b[3] < rect[1];
                    if (!overlaps) { tileScanStats.outside += 1; continue; }
                    if (!isTopLevelItem(it)) { tileScanStats.nested += 1; continue; }
                    if (overlaps) {
                        items.push(it);
                        tileScanStats.included += 1;
                    }
                } catch (e) { tileScanStats.boundsFailed += 1; noteHostDiagnostic("optional Illustrator property unavailable", e); }
            }
            var els = [];
            for (var k = 0; k < items.length; k++) {
                extractRecursive(items[k], rect, els, 0);
            }
            results.push({ artboard: abInfo, elements: els, documentPath: getDocumentPath(doc) });
        }
        
        var hostDiagnostics = consumeHostDiagnostics();
        if (hostDiagnostics.length > 0) {
            for (var ri = 0; ri < results.length; ri++) results[ri].hostDiagnostics = hostDiagnostics;
        }
        var resultJSON = JSON.stringify(results);
        return resultJSON;
    } catch (e) {
        noteHostDiagnostic("extractArtboardDataJSON exception", e);
        return JSON.stringify({ error: String(e) });
    }
}
