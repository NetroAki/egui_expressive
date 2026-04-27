// host.jsx — ExtendScript entry points for CEP mode
// This file runs in Illustrator's ExtendScript engine where `app` is available.
// It provides the Illustrator DOM access that the CEP panel needs.
// The code generation logic lives in plugin.js (browser-side).

if (typeof JSON !== 'object') {
    JSON = {};
}

var __eguiHostDiagnostics = [];
var __eguiHostLogFileName = "egui_expressive_export.log";
var __eguiHostLogInitialized = false;

function stringifyHostLogValue(value) {
    try {
        if (value === undefined || value === null) return "";
        if (value && value.message) return String(value.message);
        if (typeof value === "object" && typeof JSON === "object" && typeof JSON.stringify === "function") return JSON.stringify(value);
        return String(value);
    } catch (ignored) {
        return "<unprintable>";
    }
}

function hostFolderPath(folder) {
    try { if (folder && folder.fsName) return folder.fsName; } catch (ignored) { /* optional Folder.fsName unavailable */ }
    try { if (folder) return String(folder); } catch (ignored2) { /* optional Folder stringify unavailable */ }
    return "";
}

function getHostLogFile() {
    var folders = [];
    try { if (Folder.myDocuments) folders.push(Folder.myDocuments); } catch (ignored) { /* optional host folder unavailable */ }
    try { if (Folder.desktop) folders.push(Folder.desktop); } catch (ignored2) { /* optional host folder unavailable */ }
    try { if (Folder.temp) folders.push(Folder.temp); } catch (ignored3) { /* optional host folder unavailable */ }

    for (var i = 0; i < folders.length; i++) {
        var base = hostFolderPath(folders[i]);
        if (!base) continue;
        try {
            var file = new File(base + "/" + __eguiHostLogFileName);
            if (!file.parent || file.parent.exists) return file;
        } catch (ignored4) { /* candidate path unavailable */ }
    }
    return null;
}

function removeOtherHostLogFiles(activePath) {
    var folders = [];
    try { if (Folder.myDocuments) folders.push(Folder.myDocuments); } catch (ignored) { /* optional host folder unavailable */ }
    try { if (Folder.desktop) folders.push(Folder.desktop); } catch (ignored2) { /* optional host folder unavailable */ }
    try { if (Folder.temp) folders.push(Folder.temp); } catch (ignored3) { /* optional host folder unavailable */ }

    for (var i = 0; i < folders.length; i++) {
        var base = hostFolderPath(folders[i]);
        if (!base) continue;
        try {
            var file = new File(base + "/" + __eguiHostLogFileName);
            var path = file.fsName || String(file);
            if (path !== activePath && file.exists) file.remove();
        } catch (ignored4) { /* stale log cleanup best effort */ }
    }
}

function writeHostLogLine(mode, stage, detail) {
    var file = getHostLogFile();
    if (!file) return "";
    try {
        file.encoding = "UTF-8";
        if (!file.open(mode)) return "";
        var line = "[" + (new Date()).toString() + "] " + stringifyHostLogValue(stage);
        var extra = stringifyHostLogValue(detail);
        if (extra) line += " - " + extra;
        file.writeln(line);
        file.close();
        return file.fsName || String(file);
    } catch (ignored) {
        try { file.close(); } catch (ignored2) { /* close best effort */ }
        return "";
    }
}

function resetHostLog(context) {
    __eguiHostLogInitialized = true;
    var path = writeHostLogLine("w", "log reset", context || "export");
    if (path) removeOtherHostLogFiles(path);
    return path;
}

function appendHostLog(stage, detail) {
    if (!__eguiHostLogInitialized) resetHostLog("implicit start");
    return writeHostLogLine("a", stage || "log", detail || "");
}

function resetHostLogJSON(payloadJSON) {
    var detail = payloadJSON || "export";
    try {
        var payload = JSON.parse(payloadJSON || "{}");
        detail = payload.detail || payload.stage || payload.message || payloadJSON || "export";
    } catch (ignored) { /* non-JSON payload is logged as raw text */ }
    var path = resetHostLog(detail);
    return JSON.stringify({ success: !!path, path: path });
}

function appendHostLogJSON(payloadJSON) {
    var stage = "panel";
    var detail = payloadJSON || "";
    try {
        var payload = JSON.parse(payloadJSON || "{}");
        stage = payload.stage || stage;
        detail = payload.detail || payload.message || detail;
    } catch (ignored) { /* non-JSON payload is logged as raw text */ }
    var path = appendHostLog(stage, detail);
    return JSON.stringify({ success: !!path, path: path });
}

function getHostLogPathJSON() {
    var file = getHostLogFile();
    var path = file ? (file.fsName || String(file)) : "";
    return JSON.stringify({ path: path });
}

resetHostLog("host.jsx loaded");

function noteHostDiagnostic(context, error) {
    try {
        var message = context + ": " + (error && error.message ? error.message : String(error));
        if (__eguiHostDiagnostics.length < 200) {
            __eguiHostDiagnostics.push({ id: "host", note: message });
        }
        appendHostLog("host diagnostic", message);
    } catch (ignored) {
        // Last-resort guard: diagnostics must never break export.
    }
}

function consumeHostDiagnostics() {
    var out = __eguiHostDiagnostics.slice(0);
    __eguiHostDiagnostics = [];
    return out;
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

function getDiagnosticsJSON() {
    var result = { hasApp: false, hasDoc: false, artboardCount: 0, docName: "", error: "" };
    try {
        appendHostLog("getDiagnosticsJSON", "start");
        var appExists = false;
        try { appExists = (typeof app !== 'undefined'); } catch(e) { appendHostLog("getDiagnosticsJSON error", e); return JSON.stringify({ error: e.message || String(e) }); }
        result.hasApp = appExists;
        if (result.hasApp) {
            result.hasDoc = (app.documents.length > 0);
            if (result.hasDoc) {
                result.docName = app.activeDocument.name;
                result.artboardCount = app.activeDocument.artboards.length;
                
                // Page tile detection is handled by ai_parser, not by size heuristics.
                // Always report hasPageTiles = false here; the panel will check ai_parser separately.
                result.hasPageTiles = false;
                result.estimatedPageCount = 1;
            }
        }
        appendHostLog("getDiagnosticsJSON", "hasDoc=" + result.hasDoc + " artboards=" + result.artboardCount + " doc=" + result.docName);
    } catch(e) { appendHostLog("getDiagnosticsJSON error", e); return JSON.stringify({ error: String(e) }); }
    return JSON.stringify(result);
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

function describeHostItem(item) {
    var parts = [];
    try { parts.push("type=" + (item.typename || "unknown")); } catch (ignored) { /* optional Illustrator property unavailable */ }
    try { if (item.name) parts.push("name=" + item.name); } catch (ignored2) { /* optional Illustrator property unavailable */ }
    try {
        var b = item.geometricBounds;
        if (b) parts.push("bounds=" + [b[0], b[1], b[2], b[3]].join(","));
    } catch (ignored3) { /* optional Illustrator property unavailable */ }
    return parts.join(" ");
}

function getArtboardsJSON() {
    try {
        appendHostLog("getArtboardsJSON", "start");
        var appExists = false;
        try { appExists = (typeof app !== 'undefined'); } catch(e) { appendHostLog("getArtboardsJSON error", e); return JSON.stringify({ error: e.message || String(e) }); }
        if (!appExists || app.documents.length === 0) { appendHostLog("getArtboardsJSON", "no document"); return "[]"; }
        var doc = app.activeDocument;
        if (!doc) { appendHostLog("getArtboardsJSON", "no active document"); return "[]"; }
        var boards = [];
        for (var i = 0; i < doc.artboards.length; i++) {
            var ab = doc.artboards[i];
            var r = ab.artboardRect;
            boards.push({
                index: i,
                name: ab.name,
                width: Math.abs(r[2] - r[0]),
                height: Math.abs(r[3] - r[1]),
                x: r[0],
                y: r[1]
            });
        }
        appendHostLog("getArtboardsJSON", "count=" + boards.length + " doc=" + (doc.name || ""));
        return JSON.stringify(boards);
    } catch (e) {
        appendHostLog("getArtboardsJSON error", e);
        return JSON.stringify({ error: String(e) });
    }
}




function saveFilesToFolderJSON(payloadJSON) {
    try {
        appendHostLog("saveFilesToFolderJSON", "start payloadChars=" + String(payloadJSON || "").length);
        var payload = JSON.parse(payloadJSON);
        var files = payload.files;
        
        var folder = Folder.selectDialog("Select destination folder");
        if (!folder) {
            appendHostLog("saveFilesToFolderJSON", "canceled");
            return JSON.stringify({ canceled: true });
        }
        appendHostLog("saveFilesToFolderJSON", "folder=" + folder.fsName);
        
        var saved = [];
        var errors = [];
        for (var filename in files) {
            if (Object.prototype.hasOwnProperty.call(files, filename)) {
                var safeFilename = sanitizeOutputFilename(filename);
                var file = new File(folder.fsName + "/" + safeFilename);
                appendHostLog("save file", safeFilename);
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
                    appendHostLog("copy asset", assetPath);
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
            appendHostLog("saveFilesToFolderJSON error", errors.join(", "));
            return JSON.stringify({ error: errors.join(", "), saved: saved });
        }
        appendHostLog("saveFilesToFolderJSON", "success saved=" + saved.length);
        return JSON.stringify({ success: true, folder: folder.fsName, saved: saved });
    } catch (e) {
        appendHostLog("saveFilesToFolderJSON exception", e);
        return JSON.stringify({ error: String(e) });
    }
}

function extractArtboardDataJSON(exportPayloadJSON) {
    appendHostLog("extractArtboardDataJSON start", "payloadChars=" + String(exportPayloadJSON || "").length);
    try {
        var appExists = false;
        try { appExists = (typeof app !== 'undefined'); } catch(e) { appendHostLog("extractArtboardDataJSON app check failed", e); return JSON.stringify({ error: e.message || String(e) }); }
        if (!appExists || app.documents.length === 0) { appendHostLog("extractArtboardDataJSON", "no document"); return "[]"; }
        var doc = app.activeDocument;
        if (!doc) { appendHostLog("extractArtboardDataJSON", "no active document"); return "[]"; }
        appendHostLog("extractArtboardDataJSON", "doc=" + (doc.name || "") + " pageItems=" + (doc.pageItems ? doc.pageItems.length : 0));
        
        var payload = JSON.parse(exportPayloadJSON);
        var selectedIndices = [];
        var selectedTiles = [];
        if (Object.prototype.toString.call(payload) === '[object Array]') {
            selectedIndices = payload;
        } else {
            selectedIndices = payload.selected || [];
            selectedTiles = payload.selectedTiles || [];
        }
        appendHostLog("extract selection", "artboards=" + selectedIndices.length + " tiles=" + selectedTiles.length);
        
        var results = [];
        
        function isTopLevelItem(item) {
            try {
                var parentType = item.parent ? item.parent.typename : null;
                return parentType === 'Layer' || parentType === 'Document' || parentType === null;
            } catch(e) { return true; }
        }
        
        function colorToRGB(c) {
            if (!c) return null;
            try {
                if (c.typename === "RGBColor") return { r: Math.round(c.red), g: Math.round(c.green), b: Math.round(c.blue), a: 255 };
                if (c.typename === "CMYKColor") { var k = c.black/100; return { r: Math.round(255*(1-c.cyan/100)*(1-k)), g: Math.round(255*(1-c.magenta/100)*(1-k)), b: Math.round(255*(1-c.yellow/100)*(1-k)), a: 255 }; }
                if (c.typename === "GrayColor") { var v = Math.round(255*(1-c.gray/100)); return { r: v, g: v, b: v, a: 255 }; }
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return null;
        }
        
        function getFill(item) {
            try { if (item.filled && item.fillColor) return colorToRGB(item.fillColor); } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return null;
        }
        
        function getStroke(item) {
            try { if (item.stroked && item.strokeColor) { var c = colorToRGB(item.strokeColor); if (c) { c.width = item.strokeWidth || 1; return c; } } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return null;
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

        function getGradient(item) {
            try {
                if (item.fillColor && item.fillColor.typename === "GradientColor") {
                    var grad = item.fillColor.gradient;
                    if (!grad) return null;
                    var angle = item.fillColor.angle || 0;
                    var stops = [];
                    try {
                        for (var si = 0; si < grad.gradientStops.length; si++) {
                            var s = grad.gradientStops[si];
                            var sc = gradientColorToRGB(s.color);
                            stops.push({ position: s.rampPoint/100, color: colorToHex(sc), opacity: s.opacity !== undefined ? s.opacity/100 : 1 });
                        }
                    } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
                    return { type: grad.type === 1 ? "linear" : "radial", angle: angle, stops: stops };
                }
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return null;
        }

        function extractEffects(item) {
            var fx = [];
            try {
                // Try to detect drop shadow via XMPString (CS5+)
                try {
                    var xmp = item.XMPString;
                    if (xmp && xmp.indexOf("dropShadow") !== -1) {
                        // XMP has shadow data but parsing it is complex — emit a generic shadow
                        fx.push({ type: "dropShadow", x: 4, y: 4, blur: 8, color: { r: 0, g: 0, b: 0, a: 0.3 } });
                    }
                } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            return fx;
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
                            runs.push({ text: tr.contents || "", style: { size: a.size||14, weight: (a.textFont && a.textFont.name && a.textFont.name.indexOf("Bold") !== -1) ? 700 : 400, color: runColor } });
                        } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
                    }
                }
                return runs.length > 0 ? runs : null;
            } catch(e) { return null; }
        }

        function getPathPoints(item, artboardRect) {
            try {
                if ((item.typename === "PathItem" || item.typename === "CompoundPathItem") && item.pathPoints) {
                    var pts = [];
                    for (var pi = 0; pi < item.pathPoints.length; pi++) {
                        var pp = item.pathPoints[pi];
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
        
        function extractRecursive(item, artboardRect, elements, depth) {
            try { if (item.locked || item.hidden) return; } catch (e) { return; }
            
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
                } catch (e2) { return; }
            }
            
            var el = {
                id: item.name || ("el_" + elements.length),
                type: getElementType(item),
                x: x, y: y, w: w, h: h, depth: depth,
                fill: getFill(item),
                stroke: getStroke(item),
                text: null, textStyle: null, textRuns: null, children: [],
                opacity: 1.0, rotation: 0, cornerRadius: 0,
                gradient: null, blendMode: "normal",
                effects: [], notes: [],
                pathPoints: null, pathClosed: false,
                imagePath: null, embeddedRaster: false, symbolName: null,
                isChart: false, isGradientMesh: false,
                strokeCap: null, strokeJoin: null
            };
            
            try { el.opacity = item.opacity !== undefined ? item.opacity / 100 : 1; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { el.rotation = item.rotation !== undefined ? item.rotation : 0; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.typename === "PathItem" && item.cornerRadius !== undefined) el.cornerRadius = item.cornerRadius; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.strokeCap !== undefined) el.strokeCap = ({0:"butt",1:"round",2:"square"})[item.strokeCap] || "butt"; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.strokeJoin !== undefined) el.strokeJoin = ({0:"miter",1:"round",2:"bevel"})[item.strokeJoin] || "miter"; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.strokeDashes && item.strokeDashes.length > 0) { var dashes = []; for(var di=0; di<item.strokeDashes.length; di++) dashes.push(item.strokeDashes[di]); el.strokeDash = dashes; } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.strokeMiterLimit !== undefined) el.strokeMiterLimit = item.strokeMiterLimit; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            
            // Blend mode
            try {
                var BLEND_MAP = { "BlendModes.NORMAL":"normal","BlendModes.MULTIPLY":"multiply","BlendModes.SCREEN":"screen","BlendModes.OVERLAY":"overlay","BlendModes.DARKEN":"darken","BlendModes.LIGHTEN":"lighten","BlendModes.COLORDODGE":"color_dodge","BlendModes.COLORBURN":"color_burn","BlendModes.HARDLIGHT":"hard_light","BlendModes.SOFTLIGHT":"soft_light","BlendModes.DIFFERENCE":"difference","BlendModes.EXCLUSION":"exclusion", "BlendModes.HUE":"hue", "BlendModes.SATURATIONBLEND":"saturation", "BlendModes.COLORBLEND":"color", "BlendModes.LUMINOSITY":"luminosity" };
                if (item.blendingMode !== undefined) el.blendMode = BLEND_MAP[String(item.blendingMode)] || "normal";
            } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            
            // Gradient
            el.gradient = getGradient(item);
            
            // Effects
            el.effects = extractEffects(item);
            
            // Path points
            var ppResult = getPathPoints(item, artboardRect);
            if (ppResult) { el.pathPoints = ppResult.points; el.pathClosed = ppResult.closed; }
            
            // Image path
            try { if (item.typename === "PlacedItem" && item.file) el.imagePath = portableAssetPath(item.file.fsName || item.file.name || null); } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.typename === "RasterItem") { el.embeddedRaster = true; el.notes.push("embedded raster image"); } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            
            // Symbol
            try { if (item.typename === "SymbolItem") { el.type = "symbol"; el.symbolName = item.symbol ? item.symbol.name : "unknown"; } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            
            // Flags
            try { if (item.typename === "MeshItem") { el.isGradientMesh = true; el.notes.push("gradient mesh"); } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.typename === "GraphItem") { el.isChart = true; el.notes.push("chart/graph"); } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            try { if (item.clipping || item.clipped) { el.clipMask = true; el.notes.push("clipping mask"); } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            
            // Text
            if (item.typename === "TextFrame") {
                try { el.text = item.contents || ""; } catch (e) { el.text = ""; }
                try {
                    var chars = item.textRange.characterAttributes;
                    var size = 14, weight = 400, family = "default";
                    try { size = chars.size || 14; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
                    try { if (chars.textFont) { var fn = chars.textFont.name || ""; weight = fn.indexOf("Bold") !== -1 ? 700 : fn.indexOf("Light") !== -1 ? 300 : 400; family = fn; } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
                    el.textStyle = { size: size, fontSize: size, weight: weight, family: family };
                } catch (e) { el.textStyle = { size: 14, fontSize: 14, weight: 400, family: "default" }; }
                el.textRuns = getTextRuns(item);
            }
            
            // Group children
            if (item.typename === "GroupItem") {
                try {
                    if (item.pageItems) {
                        for (var ci = 0; ci < item.pageItems.length; ci++) {
                            extractRecursive(item.pageItems[ci], artboardRect, el.children, depth + 1);
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
            appendHostLog("extract artboard", "index=" + idx + " name=" + abInfo.name + " size=" + abInfo.width + "x" + abInfo.height);
            
            var items = [];
            for (var j = 0; j < doc.pageItems.length; j++) {
                var it = doc.pageItems[j];
                try {
                    if (it.locked || it.hidden) continue;
                    var b = it.geometricBounds;
                    if (b[2] > rect[0] && b[0] < rect[2] && b[1] > rect[3] && b[3] < rect[1] && isTopLevelItem(it)) {
                        items.push(it);
                    }
                } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            }
            appendHostLog("extract artboard items", "index=" + idx + " topLevelItems=" + items.length);
            
            var els = [];
            for (var k = 0; k < items.length; k++) {
                appendHostLog("extract item", "artboard=" + idx + " item=" + k + " " + describeHostItem(items[k]));
                extractRecursive(items[k], rect, els, 0);
            }
            appendHostLog("extract artboard done", "index=" + idx + " elements=" + els.length);
            
            results.push({ artboard: abInfo, elements: els, documentPath: getDocumentPath(doc) });
        }
        
        for (var i = 0; i < selectedTiles.length; i++) {
            var tile = selectedTiles[i];
            var rect = [tile.x, tile.y, tile.x + tile.width, tile.y - tile.height];
            var abInfo = { name: tile.name, width: tile.width, height: tile.height, x: tile.x, y: tile.y, bounds: [rect[0], rect[1], rect[2], rect[3]] };
            appendHostLog("extract tile", "name=" + abInfo.name + " size=" + abInfo.width + "x" + abInfo.height);
            
            var items = [];
            for (var j = 0; j < doc.pageItems.length; j++) {
                var it = doc.pageItems[j];
                try {
                    if (it.locked || it.hidden) continue;
                    var b = it.geometricBounds;
                    if (b[2] > rect[0] && b[0] < rect[2] && b[1] > rect[3] && b[3] < rect[1] && isTopLevelItem(it)) {
                        items.push(it);
                    }
                } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
            }
            appendHostLog("extract tile items", "name=" + abInfo.name + " topLevelItems=" + items.length);
            
            var els = [];
            for (var k = 0; k < items.length; k++) {
                appendHostLog("extract tile item", "tile=" + abInfo.name + " item=" + k + " " + describeHostItem(items[k]));
                extractRecursive(items[k], rect, els, 0);
            }
            appendHostLog("extract tile done", "name=" + abInfo.name + " elements=" + els.length);
            
            results.push({ artboard: abInfo, elements: els, documentPath: getDocumentPath(doc) });
        }
        
        var hostDiagnostics = consumeHostDiagnostics();
        if (hostDiagnostics.length > 0) {
            for (var ri = 0; ri < results.length; ri++) results[ri].hostDiagnostics = hostDiagnostics;
        }
        appendHostLog("extractArtboardDataJSON done", "results=" + results.length + " diagnostics=" + hostDiagnostics.length);
        return JSON.stringify(results);
    } catch (e) {
        appendHostLog("extractArtboardDataJSON exception", e);
        return JSON.stringify({ error: String(e) });
    }
}
