// host.jsx — ExtendScript entry points for CEP mode
// This file runs in Illustrator's ExtendScript engine where `app` is available.
// It provides the Illustrator DOM access that the CEP panel needs.
// The code generation logic lives in plugin.js (browser-side).

function getDiagnosticsJSON() {
    var result = { hasApp: false, hasDoc: false, artboardCount: 0, docName: "", error: "" };
    try {
        var appExists = false;
        try { appExists = (typeof app !== 'undefined'); } catch(e) { return JSON.stringify({ error: e.message || String(e) }); }
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
    } catch(e) { return JSON.stringify({ error: String(e) }); }
    return JSON.stringify(result);
}

function getArtboardsJSON() {
    try {
        var appExists = false;
        try { appExists = (typeof app !== 'undefined'); } catch(e) { return JSON.stringify({ error: e.message || String(e) }); }
        if (!appExists || app.documents.length === 0) return "[]";
        var doc = app.activeDocument;
        if (!doc) return "[]";
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
        return JSON.stringify(boards);
    } catch (e) {
        return JSON.stringify({ error: String(e) });
    }
}

function getDocumentInfoJSON() {
    try {
        var appExists = false;
        try { appExists = (typeof app !== 'undefined'); } catch(e) { return JSON.stringify({ error: e.message || String(e) }); }
        if (!appExists || app.documents.length === 0) return JSON.stringify({ error: "No document" });
        var doc = app.activeDocument;
        if (!doc) return JSON.stringify({ error: "No active document" });
        
        var info = {
            name: doc.name,
            artboardCount: doc.artboards.length,
            pageCount: 1,
            hasPageTiles: false,
            filePath: ""
        };
        
        try { info.filePath = doc.fullName ? doc.fullName.fsName : ""; } catch(e) {}
        
        // Page tile detection is done by ai_parser (Rust binary), not by size heuristics.
        // Return the file path so the panel can invoke ai_parser if needed.
        info.hasPageTiles = false;
        info.pageTiles = [];
        
        return JSON.stringify(info);
    } catch (e) {
        return JSON.stringify({ error: String(e) });
    }
}




function extractArtboardDataJSON(exportPayloadJSON) {
    try {
        var appExists = false;
        try { appExists = (typeof app !== 'undefined'); } catch(e) { return JSON.stringify({ error: e.message || String(e) }); }
        if (!appExists || app.documents.length === 0) return "[]";
        var doc = app.activeDocument;
        if (!doc) return "[]";
        
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
        
        function colorToRGB(c) {
            if (!c) return null;
            try {
                if (c.typename === "RGBColor") return { r: Math.round(c.red), g: Math.round(c.green), b: Math.round(c.blue), a: 255 };
                if (c.typename === "CMYKColor") { var k = c.black/100; return { r: Math.round(255*(1-c.cyan/100)*(1-k)), g: Math.round(255*(1-c.magenta/100)*(1-k)), b: Math.round(255*(1-c.yellow/100)*(1-k)), a: 255 }; }
                if (c.typename === "GrayColor") { var v = Math.round(255*(1-c.gray/100)); return { r: v, g: v, b: v, a: 255 }; }
            } catch (e) {}
            return null;
        }
        
        function getFill(item) {
            try { if (item.filled && item.fillColor) return colorToRGB(item.fillColor); } catch (e) {}
            return null;
        }
        
        function getStroke(item) {
            try { if (item.stroked && item.strokeColor) { var c = colorToRGB(item.strokeColor); if (c) { c.width = item.strokeWidth || 1; return c; } } } catch (e) {}
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
            } catch(e) {}
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
                    } catch(e) {}
                    return { type: grad.type === 1 ? "linear" : "radial", angle: angle, stops: stops };
                }
            } catch(e) {}
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
                } catch(e) {}
            } catch(e) {}
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
                            try { runColor = colorToRGB(a.fillColor); } catch(e) {}
                            runs.push({ text: tr.contents || "", style: { size: a.size||14, weight: (a.textFont && a.textFont.name && a.textFont.name.indexOf("Bold") !== -1) ? 700 : 400, color: runColor } });
                        } catch(e) {}
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
                                kind: pp.pointType === PointType.SMOOTH ? "smooth" : "corner"
                            });
                        } catch(ppe) {}
                    }
                    if (pts.length > 0) return { points: pts, closed: item.closed || false };
                }
            } catch(e) {}
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
                                if (ratio > 0.85) return "circle";
                                return "ellipse";
                            }
                        }
                    } catch(e) {}
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
                imagePath: null, symbolName: null,
                isChart: false, isGradientMesh: false,
                strokeCap: null, strokeJoin: null
            };
            
            try { el.opacity = item.opacity !== undefined ? item.opacity / 100 : 1; } catch (e) {}
            try { el.rotation = item.rotation !== undefined ? item.rotation : 0; } catch (e) {}
            try { if (item.typename === "PathItem" && item.cornerRadius !== undefined) el.cornerRadius = item.cornerRadius; } catch (e) {}
            try { if (item.strokeCap !== undefined) el.strokeCap = ({0:"butt",1:"round",2:"square"})[item.strokeCap] || "butt"; } catch(e) {}
            try { if (item.strokeJoin !== undefined) el.strokeJoin = ({0:"miter",1:"round",2:"bevel"})[item.strokeJoin] || "miter"; } catch(e) {}
            
            // Blend mode
            try {
                var BLEND_MAP = { "BlendModes.NORMAL":"normal","BlendModes.MULTIPLY":"multiply","BlendModes.SCREEN":"screen","BlendModes.OVERLAY":"overlay","BlendModes.DARKEN":"darken","BlendModes.LIGHTEN":"lighten","BlendModes.COLORDODGE":"color_dodge","BlendModes.COLORBURN":"color_burn","BlendModes.HARDLIGHT":"hard_light","BlendModes.SOFTLIGHT":"soft_light","BlendModes.DIFFERENCE":"difference","BlendModes.EXCLUSION":"exclusion" };
                if (item.blendingMode !== undefined) el.blendMode = BLEND_MAP[String(item.blendingMode)] || "normal";
            } catch(e) {}
            
            // Gradient
            el.gradient = getGradient(item);
            
            // Effects
            el.effects = extractEffects(item);
            
            // Path points
            var ppResult = getPathPoints(item, artboardRect);
            if (ppResult) { el.pathPoints = ppResult.points; el.pathClosed = ppResult.closed; }
            
            // Image path
            try { if (item.typename === "PlacedItem" && item.file) el.imagePath = item.file.fsName || item.file.name || null; } catch(e) {}
            try { if (item.typename === "RasterItem") el.imagePath = "raster_" + Math.round(w) + "x" + Math.round(h); } catch(e) {}
            
            // Symbol
            try { if (item.typename === "SymbolItem") { el.type = "symbol"; el.symbolName = item.symbol ? item.symbol.name : "unknown"; } } catch(e) {}
            
            // Flags
            try { if (item.typename === "MeshItem") { el.isGradientMesh = true; el.notes.push("gradient mesh"); } } catch(e) {}
            try { if (item.typename === "GraphItem") { el.isChart = true; el.notes.push("chart/graph"); } } catch(e) {}
            try { if (item.clipping || item.clipped) el.notes.push("clipping mask"); } catch(e) {}
            
            // Text
            if (item.typename === "TextFrame") {
                try { el.text = item.contents || ""; } catch (e) { el.text = ""; }
                try {
                    var chars = item.textRange.characterAttributes;
                    var size = 14, weight = 400, family = "default";
                    try { size = chars.size || 14; } catch (e) {}
                    try { if (chars.textFont) { var fn = chars.textFont.name || ""; weight = fn.indexOf("Bold") !== -1 ? 700 : fn.indexOf("Light") !== -1 ? 300 : 400; family = fn; } } catch (e) {}
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
                } catch (e) {}
            }
            
            elements.push(el);
        }

        for (var i = 0; i < selectedIndices.length; i++) {
            var idx = selectedIndices[i];
            var ab = doc.artboards[idx];
            var rect = ab.artboardRect;
            var abInfo = { name: ab.name, width: Math.abs(rect[2] - rect[0]), height: Math.abs(rect[3] - rect[1]), x: rect[0], y: rect[1] };
            
            var items = [];
            for (var j = 0; j < doc.pageItems.length; j++) {
                var it = doc.pageItems[j];
                try {
                    if (it.locked || it.hidden) continue;
                    var b = it.geometricBounds;
                    if (b[2] > rect[0] && b[0] < rect[2] && b[1] > rect[3] && b[3] < rect[1] && isTopLevelItem(it)) {
                        items.push(it);
                    }
                } catch(e) {}
            }
            
            var els = [];
            for (var k = 0; k < items.length; k++) {
                extractRecursive(items[k], rect, els, 0);
            }
            
            results.push({ artboard: abInfo, elements: els });
        }
        
        for (var i = 0; i < selectedTiles.length; i++) {
            var tile = selectedTiles[i];
            var rect = [tile.x, tile.y, tile.x + tile.width, tile.y - tile.height];
            var abInfo = { name: tile.name, width: tile.width, height: tile.height, x: tile.x, y: tile.y };
            
            var items = [];
            for (var j = 0; j < doc.pageItems.length; j++) {
                var it = doc.pageItems[j];
                try {
                    if (it.locked || it.hidden) continue;
                    var b = it.geometricBounds;
                    if (b[2] > rect[0] && b[0] < rect[2] && b[1] > rect[3] && b[3] < rect[1] && isTopLevelItem(it)) {
                        items.push(it);
                    }
                } catch(e) {}
            }
            
            var els = [];
            for (var k = 0; k < items.length; k++) {
                extractRecursive(items[k], rect, els, 0);
            }
            
            results.push({ artboard: abInfo, elements: els });
        }
        
        return JSON.stringify(results);
    } catch (e) {
        return JSON.stringify({ error: String(e) });
    }
}
