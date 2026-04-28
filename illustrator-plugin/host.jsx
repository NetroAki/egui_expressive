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
var __eguiHostLogFallbacksCleaned = false;
var __eguiHostMaxLogBytes = 20 * 1024 * 1024;
var __eguiHostItemTraceLimit = 100000;

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
    if (__eguiHostLogFallbacksCleaned) return;
    __eguiHostLogFallbacksCleaned = true;
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

function trimHostLogFile(file) {
    try {
        if (!file || !file.exists || !file.length || file.length <= __eguiHostMaxLogBytes) return;
        file.encoding = "UTF-8";
        if (!file.open("r")) return;
        var content = file.read();
        file.close();
        if (!content || content.length <= __eguiHostMaxLogBytes) return;
        var keepChars = Math.floor(__eguiHostMaxLogBytes * 0.75);
        var retained = content.substring(Math.max(0, content.length - keepChars));
        if (!file.open("w")) return;
        file.write("[log trimmed to latest entries; older entries discarded]\n" + retained);
        file.close();
    } catch (ignored) {
        try { file.close(); } catch (ignored2) { /* close best effort */ }
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
        trimHostLogFile(file);
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

function startHostLog(context) {
    __eguiHostLogInitialized = true;
    var path = writeHostLogLine("a", "host log start", context || "host.jsx loaded");
    if (path) removeOtherHostLogFiles(path);
    return path;
}

function appendHostLog(stage, detail) {
    if (!__eguiHostLogInitialized) startHostLog("implicit start");
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

startHostLog("host.jsx loaded");

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
    try { return JSON.stringify(value); } catch (jsonErr) { appendHostLog("safeJsonStringify", jsonErr); }
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
    try { return app.documents.length; } catch (e) { appendHostLog(context || "documents.length", e); }
    try { if (typeof $ !== "undefined" && $.sleep) $.sleep(200); } catch (ignored) { /* optional sleep unavailable */ }
    try { return app.documents.length; } catch (e2) { appendHostLog((context || "documents.length") + " retry", e2); }
    return 0;
}

function activeIllustratorDocument(context) {
    try { return app.activeDocument; } catch (e) { appendHostLog(context || "activeDocument", e); }
    try { if (typeof $ !== "undefined" && $.sleep) $.sleep(200); } catch (ignored) { /* optional sleep unavailable */ }
    try { return app.activeDocument; } catch (e2) { appendHostLog((context || "activeDocument") + " retry", e2); }
    return null;
}

function getDiagnosticsJSON() {
    var result = { hasApp: false, hasDoc: false, artboardCount: 0, docName: "", error: "" };
    try {
        appendHostLog("getDiagnosticsJSON", "start");
        var appExists = false;
        try { appExists = (typeof app !== 'undefined'); } catch(e) { appendHostLog("getDiagnosticsJSON error", e); return JSON.stringify({ error: e.message || String(e) }); }
        result.hasApp = appExists;
        if (result.hasApp) {
            var docCount = openDocumentCount("getDiagnosticsJSON documents.length");
            result.hasDoc = (docCount > 0);
            if (result.hasDoc) {
                var doc = activeIllustratorDocument("getDiagnosticsJSON activeDocument");
                if (!doc) {
                    result.hasDoc = false;
                    result.error = "Illustrator reports an open document, but activeDocument is not ready yet. Try clicking the document canvas, then press Refresh.";
                    appendHostLog("getDiagnosticsJSON", result.error);
                    return safeJsonStringify(result);
                }
                try { result.docName = doc.name || ""; } catch (nameErr) { appendHostLog("getDiagnosticsJSON doc.name", nameErr); }
                try { result.artboardCount = doc.artboards ? doc.artboards.length : 0; } catch (abErr) { appendHostLog("getDiagnosticsJSON artboards.length", abErr); result.artboardCount = 0; }
                
                // Page tile detection is handled by ai_parser, not by size heuristics.
                // Always report hasPageTiles = false here; the panel will check ai_parser separately.
                result.hasPageTiles = false;
                result.estimatedPageCount = 1;
            }
        }
        appendHostLog("getDiagnosticsJSON", "hasDoc=" + result.hasDoc + " artboards=" + result.artboardCount + " doc=" + result.docName);
    } catch(e) { appendHostLog("getDiagnosticsJSON error", e); return safeJsonStringify({ error: String(e) }); }
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

function summarizeHostText(value, limit) {
    var text = String(value || "");
    var max = limit || 120;
    text = text.replace(/\r/g, "\\r").replace(/\n/g, "\\n").replace(/\t/g, "\\t");
    return text.length > max ? text.substring(0, max) + "…" : text;
}

function countElementTree(elements) {
    var total = 0;
    if (!elements) return total;
    for (var i = 0; i < elements.length; i++) {
        total += 1 + countElementTree(elements[i].children || []);
    }
    return total;
}

function describeExtractedElement(el) {
    var parts = [];
    parts.push("id=" + summarizeHostText(el.id, 80));
    parts.push("type=" + el.type);
    parts.push("depth=" + el.depth);
    parts.push("bounds=" + [el.x, el.y, el.w, el.h].join(","));
    parts.push("opacity=" + el.opacity);
    parts.push("blend=" + el.blendMode);
    parts.push("rotation=" + el.rotation);
    parts.push("children=" + (el.children ? el.children.length : 0));
    if (el.fill) parts.push("fill=" + safeJsonStringify(el.fill));
    if (el.stroke) parts.push("stroke=" + safeJsonStringify(el.stroke));
    if (el.gradient) parts.push("gradient=" + safeJsonStringify(el.gradient));
    if (el.pathPoints) parts.push("pathPoints=" + el.pathPoints.length + " closed=" + el.pathClosed);
    if (el.text !== null && el.text !== undefined) parts.push("textLen=" + String(el.text).length + " text=\"" + summarizeHostText(el.text, 120) + "\"");
    if (el.textStyle) parts.push("textStyle=" + safeJsonStringify(el.textStyle));
    if (el.textRuns) parts.push("textRuns=" + el.textRuns.length);
    if (el.imagePath) parts.push("imagePath=" + summarizeHostText(el.imagePath, 160));
    if (el.embeddedRaster) parts.push("embeddedRaster=true");
    if (el.symbolName) parts.push("symbol=" + summarizeHostText(el.symbolName, 120));
    if (el.isChart) parts.push("chart=true");
    if (el.isGradientMesh) parts.push("gradientMesh=true");
    if (el.clipMask) parts.push("clipMask=true");
    if (el.notes && el.notes.length) parts.push("notes=" + el.notes.join("|"));
    return parts.join(" ");
}

function getArtboardsJSON() {
    try {
        appendHostLog("getArtboardsJSON", "start");
        var appExists = false;
        try { appExists = (typeof app !== 'undefined'); } catch(e) { appendHostLog("getArtboardsJSON error", e); return JSON.stringify({ error: e.message || String(e) }); }
        if (!appExists) { appendHostLog("getArtboardsJSON", "app unavailable"); return "[]"; }
        if (openDocumentCount("getArtboardsJSON documents.length") === 0) { appendHostLog("getArtboardsJSON", "no document"); return "[]"; }
        var doc = activeIllustratorDocument("getArtboardsJSON activeDocument");
        if (!doc) { appendHostLog("getArtboardsJSON", "no active document"); return "[]"; }
        var boards = [];
        var artboards = null;
        var artboardCount = 0;
        try {
            artboards = doc.artboards;
            artboardCount = artboards ? artboards.length : 0;
        } catch (listErr) {
            appendHostLog("getArtboardsJSON artboards", listErr);
            return safeJsonStringify({ error: "Could not access Illustrator artboards: " + String(listErr) });
        }
        for (var i = 0; i < artboardCount; i++) {
            var ab = null;
            var r = null;
            try { ab = artboards[i]; } catch (itemErr) { appendHostLog("getArtboardsJSON artboard[" + i + "]", itemErr); continue; }
            try { r = ab.artboardRect; } catch (rectErr) { appendHostLog("getArtboardsJSON artboardRect[" + i + "]", rectErr); continue; }
            if (!r || r.length < 4) { appendHostLog("getArtboardsJSON", "invalid artboardRect for index " + i); continue; }
            var name = "Artboard " + (i + 1);
            try { if (ab.name) name = ab.name; } catch (nameErr) { appendHostLog("getArtboardsJSON name[" + i + "]", nameErr); }
            boards.push({
                index: i,
                name: name,
                width: Math.abs(r[2] - r[0]),
                height: Math.abs(r[3] - r[1]),
                x: r[0],
                y: r[1]
            });
        }
        appendHostLog("getArtboardsJSON", "count=" + boards.length + " doc=" + (doc.name || ""));
        return safeJsonStringify(boards);
    } catch (e) {
        appendHostLog("getArtboardsJSON error", e);
        return safeJsonStringify({ error: String(e) });
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

function selectSaveFolderJSON() {
    try {
        appendHostLog("selectSaveFolderJSON", "start");
        var folder = Folder.selectDialog("Select destination folder");
        if (!folder) {
            appendHostLog("selectSaveFolderJSON", "canceled");
            return safeJsonStringify({ canceled: true });
        }
        appendHostLog("selectSaveFolderJSON", "folder=" + folder.fsName);
        return safeJsonStringify({ success: true, folder: folder.fsName });
    } catch (e) {
        appendHostLog("selectSaveFolderJSON exception", e);
        return safeJsonStringify({ error: String(e) });
    }
}

function writeGeneratedFileChunkJSON(payloadJSON) {
    try {
        appendHostLog("writeGeneratedFileChunkJSON", "payloadChars=" + String(payloadJSON || "").length);
        var payload = JSON.parse(payloadJSON || "{}");
        var folderPath = String(payload.folder || "");
        var filename = sanitizeOutputFilename(payload.filename || "generated.rs");
        var mode = payload.mode === "a" ? "a" : "w";
        var content = String(payload.content || "");
        appendHostLog("writeGeneratedFileChunkJSON", "file=" + filename + " mode=" + mode + " chars=" + content.length + " folder=" + folderPath);
        if (!folderPath) { appendHostLog("writeGeneratedFileChunkJSON error", "missing folder for " + filename); return safeJsonStringify({ error: "Missing destination folder" }); }

        var folder = new Folder(folderPath);
        if (!folder.exists) { appendHostLog("writeGeneratedFileChunkJSON error", "folder missing " + folderPath); return safeJsonStringify({ error: "Destination folder does not exist: " + folderPath }); }

        var file = new File(folder.fsName + "/" + filename);
        file.encoding = "UTF-8";
        if (!file.open(mode)) { appendHostLog("writeGeneratedFileChunkJSON error", "open failed file=" + filename + " path=" + file.fsName); return safeJsonStringify({ error: "Failed to open " + filename }); }
        var wrote = file.write(content);
        file.close();
        if (!wrote) { appendHostLog("writeGeneratedFileChunkJSON error", "write failed file=" + filename + " chars=" + content.length); return safeJsonStringify({ error: "Failed to write " + filename }); }
        appendHostLog("writeGeneratedFileChunkJSON", "success file=" + filename + " chars=" + content.length + " path=" + file.fsName);
        return safeJsonStringify({ success: true, filename: filename, bytes: content.length });
    } catch (e) {
        appendHostLog("writeGeneratedFileChunkJSON exception", e);
        return safeJsonStringify({ error: String(e) });
    }
}

function copyGeneratedAssetJSON(payloadJSON) {
    try {
        appendHostLog("copyGeneratedAssetJSON", "payloadChars=" + String(payloadJSON || "").length);
        var payload = JSON.parse(payloadJSON || "{}");
        var folderPath = String(payload.folder || "");
        var assetPath = String(payload.assetPath || "").replace(/\\/g, "/");
        var sourcePath = String(payload.sourcePath || "");
        appendHostLog("copyGeneratedAssetJSON", "asset=" + assetPath + " source=" + sourcePath + " folder=" + folderPath);
        if (!folderPath) { appendHostLog("copyGeneratedAssetJSON error", "missing folder for " + assetPath); return safeJsonStringify({ error: "Missing destination folder" }); }
        if (!assetPath || assetPath.indexOf("assets/") !== 0) { appendHostLog("copyGeneratedAssetJSON error", "invalid asset path " + assetPath); return safeJsonStringify({ error: "Invalid asset path: " + assetPath }); }
        if (!sourcePath) { appendHostLog("copyGeneratedAssetJSON error", "missing source for " + assetPath); return safeJsonStringify({ error: "Missing source asset for " + assetPath }); }

        var folder = new Folder(folderPath);
        if (!folder.exists) { appendHostLog("copyGeneratedAssetJSON error", "folder missing " + folderPath); return safeJsonStringify({ error: "Destination folder does not exist: " + folderPath }); }
        var assetsFolder = new Folder(folder.fsName + "/assets");
        if (!assetsFolder.exists) {
            appendHostLog("copyGeneratedAssetJSON", "creating assets folder=" + assetsFolder.fsName);
            assetsFolder.create();
        }

        var assetName = sanitizeOutputFilename(assetPath.substring("assets/".length));
        var sourceFile = new File(sourcePath);
        if (!sourceFile.exists) { appendHostLog("copyGeneratedAssetJSON error", "source missing " + sourcePath); return safeJsonStringify({ error: "Source asset missing: " + sourcePath }); }
        var destFile = new File(assetsFolder.fsName + "/" + assetName);
        if (!sourceFile.copy(destFile)) { appendHostLog("copyGeneratedAssetJSON error", "copy failed " + sourcePath + " -> " + destFile.fsName); return safeJsonStringify({ error: "Failed to copy " + assetPath }); }
        appendHostLog("copyGeneratedAssetJSON", "success " + sourcePath + " -> " + destFile.fsName);
        return safeJsonStringify({ success: true, filename: "assets/" + assetName });
    } catch (e) {
        appendHostLog("copyGeneratedAssetJSON exception", e);
        return safeJsonStringify({ error: String(e) });
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
        appendHostLog("extract selection", "artboards=" + selectedIndices.length + " indices=" + selectedIndices.join(",") + " tiles=" + selectedTiles.length + " payload=" + summarizeHostText(exportPayloadJSON, 500));
        
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
                    return {
                        type: "pattern",
                        patternName: color.pattern ? color.pattern.name : "unknown",
                        rotation: color.rotation || 0,
                        scale: [color.scaleFactor || 1, color.scaleFactor || 1]
                    };
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

        function getTextStyle(item) {
            try {
                var chars = item.textRange.characterAttributes;
                var size = 14, weight = 400, family = "default";
                try { size = chars.size || 14; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
                try { if (chars.textFont) { family = chars.textFont.name || ""; weight = textFontWeight(family); } } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
                return { size: size, fontSize: size, weight: weight, family: family };
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
                if ((typeof Justification !== "undefined" && j === Justification.FULLJUSTIFY) || name.indexOf("JUSTIFY") !== -1) return "justified";
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
                            runs.push({
                                text: tr.contents || "",
                                style: {
                                    size: a.size || 14,
                                    fontSize: a.size || 14,
                                    weight: textFontWeight(fontName),
                                    family: fontName,
                                    color: runColor,
                                    letterSpacing: illustratorTrackingToPx(a.tracking, a.size || 14),
                                    lineHeight: illustratorLeadingToMultiplier(a.leading, a.size || 14),
                                    textDecoration: (a.underline && a.strikeThrough) ? "both" : a.underline ? "underline" : a.strikeThrough ? "strikethrough" : null,
                                    textTransform: a.smallCaps ? "small_caps" : a.allCaps ? "uppercase" : null
                                }
                            });
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
            try {
                if (item.locked || item.hidden) {
                    appendHostLog("extract skip hidden/locked", "depth=" + depth + " " + describeHostItem(item));
                    return;
                }
            } catch (e) { appendHostLog("extract skip state error", "depth=" + depth + " " + stringifyHostLogValue(e)); return; }
            
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
                } catch (e2) { appendHostLog("extract skip bounds", "depth=" + depth + " " + describeHostItem(item) + " err=" + stringifyHostLogValue(e2)); return; }
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
                children: [],
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
            el.gradient = getGradient(item, artboardRect);
            
            // Effects
            el.effects = extractEffects(item);
            
            // Path points
            var ppResult = getPathPoints(item, artboardRect);
            if (ppResult) { el.pathPoints = ppResult.points; el.pathClosed = ppResult.closed; }
            
            // Image path. Keep the original filesystem path here; plugin.js derives
            // the portable assets/... path later while saveFilesToFolderJSON uses
            // this raw source path for the actual copy.
            try { if (item.typename === "PlacedItem" && item.file) el.imagePath = item.file.fsName || item.file.name || null; } catch (e) { noteHostDiagnostic("optional Illustrator property unavailable", e); }
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
                el.textStyle = getTextStyle(item);
                el.textAlign = getTextAlign(item);
                el.letterSpacing = getLetterSpacing(item);
                el.lineHeight = getLineHeight(item);
                el.textDecoration = getTextDecoration(item);
                el.textTransform = getTextTransform(item);
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

            appendHostLog("extract element", describeExtractedElement(el));
            
            elements.push(el);
        }

        for (var i = 0; i < selectedIndices.length; i++) {
            var idx = selectedIndices[i];
            var ab = doc.artboards[idx];
            var rect = ab.artboardRect;
            var abInfo = { name: ab.name, index: idx, width: Math.abs(rect[2] - rect[0]), height: Math.abs(rect[3] - rect[1]), x: rect[0], y: rect[1], bounds: [rect[0], rect[1], rect[2], rect[3]] };
            appendHostLog("extract artboard", "index=" + idx + " name=" + abInfo.name + " size=" + abInfo.width + "x" + abInfo.height);
            
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
            appendHostLog("extract artboard items", "index=" + idx + " topLevelItems=" + items.length + " scan=" + safeJsonStringify(scanStats));
            
            var els = [];
            for (var k = 0; k < items.length; k++) {
                if (__eguiHostItemTraceLimit > 0 && k < __eguiHostItemTraceLimit) appendHostLog("extract item", "artboard=" + idx + " item=" + k + " " + describeHostItem(items[k]));
                extractRecursive(items[k], rect, els, 0);
            }
            appendHostLog("extract artboard done", "index=" + idx + " elements=" + els.length + " treeElements=" + countElementTree(els));
            
            results.push({ artboard: abInfo, elements: els, documentPath: getDocumentPath(doc) });
        }
        
        for (var i = 0; i < selectedTiles.length; i++) {
            var tile = selectedTiles[i];
            var rect = [tile.x, tile.y, tile.x + tile.width, tile.y - tile.height];
            var abInfo = { name: tile.name, width: tile.width, height: tile.height, x: tile.x, y: tile.y, bounds: [rect[0], rect[1], rect[2], rect[3]] };
            appendHostLog("extract tile", "name=" + abInfo.name + " size=" + abInfo.width + "x" + abInfo.height);
            
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
            appendHostLog("extract tile items", "name=" + abInfo.name + " topLevelItems=" + items.length + " scan=" + safeJsonStringify(tileScanStats));
            
            var els = [];
            for (var k = 0; k < items.length; k++) {
                if (__eguiHostItemTraceLimit > 0 && k < __eguiHostItemTraceLimit) appendHostLog("extract tile item", "tile=" + abInfo.name + " item=" + k + " " + describeHostItem(items[k]));
                extractRecursive(items[k], rect, els, 0);
            }
            appendHostLog("extract tile done", "name=" + abInfo.name + " elements=" + els.length + " treeElements=" + countElementTree(els));
            
            results.push({ artboard: abInfo, elements: els, documentPath: getDocumentPath(doc) });
        }
        
        var hostDiagnostics = consumeHostDiagnostics();
        if (hostDiagnostics.length > 0) {
            for (var ri = 0; ri < results.length; ri++) results[ri].hostDiagnostics = hostDiagnostics;
        }
        var resultJSON = JSON.stringify(results);
        appendHostLog("extractArtboardDataJSON done", "results=" + results.length + " diagnostics=" + hostDiagnostics.length + " resultChars=" + resultJSON.length);
        return resultJSON;
    } catch (e) {
        appendHostLog("extractArtboardDataJSON exception", e);
        return JSON.stringify({ error: String(e) });
    }
}
