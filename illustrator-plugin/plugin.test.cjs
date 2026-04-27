const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const pluginSourceForVm = fs.readFileSync(path.join(__dirname, 'plugin.js'), 'utf8');
const sandbox = {
  module: { exports: {} },
  exports: {},
  require,
  console: { warn() {}, error() {} },
  URL,
  process,
};
vm.runInNewContext(pluginSourceForVm, sandbox, { filename: 'plugin.js' });
const plugin = sandbox.module.exports;

function testPortableAssetPath() {
  assert.strictEqual(plugin.portableAssetPath('/Users/alice/design assets/Hero Image.png'), 'assets/4a3a86_Hero_Image.png');
  assert.strictEqual(plugin.portableAssetPath('C:\\work\\icon.svg'), 'assets/4a9b19_icon.svg');
  // Avoid double hashing
  assert.strictEqual(plugin.portableAssetPath('assets/4a3a86_Hero_Image.png'), 'assets/4a3a86_Hero_Image.png');
}

function testBundledParserCandidates() {
  const candidates = plugin.getAiParserCandidates('/extension/root', 'linux');
  assert.strictEqual(candidates[0], path.join('/extension/root', 'bin', 'linux', 'ai-parser'));
  assert(!candidates.some(candidate => candidate.includes(path.join('target', 'release'))));
}

function testMergeParserDataByBounds() {
  const dom = [{ id: 'el_0', type: 'shape', x: 10, y: 20, w: 30, h: 40, children: [], effects: [] }];
  const parser = {
    elements: [{
      id: 'element_7',
      element_type: 'path',
      bounds: [10, 20, 30, 40],
      rotation_deg: 90,
      corner_radius: 8,
      path_closed: true,
      path_points: [{ anchor: [10, 20], left_ctrl: [10, 20], right_ctrl: [20, 20] }]
    }]
  };
  const merged = plugin.mergeAiParserData(dom, parser);
  assert.strictEqual(merged[0].parserId, 'element_7');
  assert.strictEqual(merged[0].rotation, 90);
  assert.strictEqual(merged[0].cornerRadius, 8);
  assert.strictEqual(merged[0].pathClosed, true);
  assert.deepStrictEqual(merged[0].pathPoints[0].rightDir, [20, 20]);
}

function testWarningsUsePortableImagePath() {
  const warnings = plugin.collectWarnings([{ id: 'img', type: 'image', imagePath: '/tmp/Secret Folder/photo.jpg', blendMode: 'normal' }], {});
  assert(warnings.some(w => w.note.includes('_photo.jpg')));
  assert(!warnings.some(w => w.note.includes('/tmp/Secret Folder')));
}

function testStaticSecurityChecks() {
  const root = __dirname;
  const index = fs.readFileSync(path.join(root, 'index.html'), 'utf8');
  const host = fs.readFileSync(path.join(root, 'host.jsx'), 'utf8');
  const pluginSource = fs.readFileSync(path.join(root, 'plugin.js'), 'utf8');
  const remoteZipHost = [99, 100, 110, 106, 115].map(code => String.fromCharCode(code)).join('');
  assert(!index.includes([remoteZipHost, 'cloudflare', 'com'].join('.')));
  assert(!/postMessage\([^\n]*['"]\*['"]/.test(index));
  assert(!/postMessage\([^\n]*['"]\*['"]/.test(pluginSource));
  assert(!/catch\s*\([^)]*\)\s*\{\s*\}/.test(host));
  assert(!/catch\s*\([^)]*\)\s*\{\s*\}/.test(pluginSource));
  assert(!/eval\s*\(/.test(host));
}

function testIndexBootstrap() {
  const indexHtml = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
  const scriptMatch = indexHtml.match(/<script>([\s\S]*?)<\/script>/g);
  assert(scriptMatch && scriptMatch.length > 0, "Could not find inline script in index.html");
  const inlineScript = scriptMatch[0].replace(/<\/?script>/g, '');

  const domSandbox = {
    window: {
      addEventListener: () => {},
      location: { href: 'http://localhost/index.html' },
      postMessage: () => {}
    },
    document: {
      getElementById: () => ({ classList: { toggle: () => {} }, setAttribute: () => {} }),
      querySelectorAll: () => []
    },
    URL: URL,
    console: console,
    setTimeout: setTimeout,
    escapeHtml: (s) => s,
    options: { naming: true }
  };

  // First run plugin.js
  vm.runInNewContext(pluginSourceForVm, domSandbox, { filename: 'plugin.js' });

  // Then run inline script
  vm.runInNewContext(inlineScript, domSandbox, { filename: 'index.html' });

  assert.strictEqual(typeof domSandbox.toggleOption, 'function');
  assert.strictEqual(typeof domSandbox.doExport, 'function');
}

function testAriaPressedToggle() {
  const indexHtml = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
  const scriptMatch = indexHtml.match(/<script>([\s\S]*?)<\/script>/g);
  const inlineScript = scriptMatch[0].replace(/<\/?script>/g, '');

  let attributes = {};
  const mockElement = {
    classList: { toggle: () => {} },
    setAttribute: (attr, val) => { attributes[attr] = val; }
  };

  const domSandbox = {
    window: { addEventListener: () => {}, location: { href: 'http://localhost/index.html' }, postMessage: () => {} },
    document: { getElementById: () => mockElement, querySelectorAll: () => [] },
    URL: URL, console: console, setTimeout: setTimeout, escapeHtml: (s) => s,
    options: { naming: true }
  };

  vm.runInNewContext(pluginSourceForVm, domSandbox, { filename: 'plugin.js' });
  vm.runInNewContext(inlineScript, domSandbox, { filename: 'index.html' });

  domSandbox.toggleOption('naming');
  assert.strictEqual(attributes['aria-pressed'], 'false');

  domSandbox.toggleOption('naming');
  assert.strictEqual(attributes['aria-pressed'], 'true');
}

function testHostJsx() {
  const hostSource = fs.readFileSync(path.join(__dirname, 'host.jsx'), 'utf8');
  const hostSandbox = {
    Folder: function(path) { this.fsName = path; this.create = function() {}; },
    File: function(path) { this.fsName = path; this.exists = true; this.copy = function() { return true; }; this.open = function() { return true; }; this.write = function() { return true; }; this.close = function() {}; }
  };
  hostSandbox.Folder.selectDialog = function() { return new hostSandbox.Folder('/tmp/export'); };

  vm.runInNewContext(hostSource, hostSandbox, { filename: 'host.jsx' });

  const assetPath = hostSandbox.portableAssetPath('C:\\test\\image.png');
  assert(assetPath.startsWith('assets/'));
  assert(assetPath.endsWith('_image.png'));

  const doc = {
    get fullName() { throw new Error("Not saved"); },
    path: "/tmp",
    name: "test.ai"
  };
  assert.strictEqual(hostSandbox.getDocumentPath(doc), "/tmp/test.ai");
  const diags = hostSandbox.consumeHostDiagnostics();
  assert(diags.length > 0);
  assert(diags[0].note.includes("Not saved"));

  // Test saveFilesToFolderJSON
  const payload = {
    files: { 'test.rs': 'fn main() {}' },
    assets: { 'assets/123_img.png': '/tmp/img.png' }
  };
  const saveResult = JSON.parse(hostSandbox.saveFilesToFolderJSON(JSON.stringify(payload)));
  assert.strictEqual(saveResult.success, true);
  assert.strictEqual(saveResult.folder, '/tmp/export');
  assert(saveResult.saved.includes('test.rs'));
  assert(saveResult.saved.includes('assets/123_img.png'));

  // Test extractArtboardDataJSON with no app
  const extractResult = JSON.parse(hostSandbox.extractArtboardDataJSON(JSON.stringify([0])));
  assert(Array.isArray(extractResult));
  assert.strictEqual(extractResult.length, 0);
}

function testFileTreeAndCodePreview() {
  const indexHtml = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
  const scriptMatch = indexHtml.match(/<script>([\s\S]*?)<\/script>/g);
  const inlineScript = scriptMatch[0].replace(/<\/?script>/g, '');

  const domSandbox = {
    window: { addEventListener: () => {}, location: { href: 'http://localhost/index.html' }, postMessage: () => {} },
    document: {
      getElementById: (id) => {
        if (!domSandbox.elements[id]) domSandbox.elements[id] = { classList: { toggle: () => {}, add: () => {}, remove: () => {} }, setAttribute: () => {}, innerHTML: '', style: {} };
        return domSandbox.elements[id];
      },
      querySelectorAll: () => [],
      createElement: (tag) => ({ href: '', download: '', click: () => {} })
    },
    URL: { createObjectURL: () => 'blob:url', revokeObjectURL: () => {} },
    Blob: class Blob { constructor(content, opts) { this.content = content; this.opts = opts; } },
    console: console, setTimeout: setTimeout, escapeHtml: (s) => s,
    options: { naming: true },
    elements: {},
    navigator: { clipboard: { writeText: async () => {} } }
  };

  vm.runInNewContext(pluginSourceForVm, domSandbox, { filename: 'plugin.js' });
  vm.runInNewContext(inlineScript, domSandbox, { filename: 'index.html' });

  domSandbox.handleExportResult({
    payload: {
      files: { 'main.rs': 'fn main() { println!("Hello"); }' },
      assets: {},
      warnings: []
    }
  });

  assert(domSandbox.elements['file-tree'].innerHTML.includes('main.rs'));
  assert(domSandbox.elements['code-preview'].innerHTML.includes('main'));

  domSandbox.copyCode();
}

function testHostSaveFailureHandling() {
  const hostSource = fs.readFileSync(path.join(__dirname, 'host.jsx'), 'utf8');
  const hostSandbox = {
    Folder: function(path) { this.fsName = path; this.create = function() {}; },
    File: function(path) {
      this.fsName = path;
      this.exists = true;
      this.copy = function() { return false; };
      this.open = function() { return false; };
      this.write = function() { return false; };
      this.close = function() {};
    }
  };
  hostSandbox.Folder.selectDialog = function() { return new hostSandbox.Folder('/tmp/export'); };

  vm.runInNewContext(hostSource, hostSandbox, { filename: 'host.jsx' });

  const payload = {
    files: { 'test.rs': 'fn main() {}' },
    assets: { 'assets/123_img.png': '/tmp/img.png' }
  };
  const saveResult = JSON.parse(hostSandbox.saveFilesToFolderJSON(JSON.stringify(payload)));
  assert.strictEqual(saveResult.error.includes('Failed to open'), true);
  assert.strictEqual(saveResult.error.includes('Failed to copy'), true);
}

function testApplyBlendExpr() {
  const expr = "tokens::SURFACE";
  assert.strictEqual(plugin.applyBlendExpr(expr, "normal"), expr);
  assert.strictEqual(plugin.applyBlendExpr(expr, "multiply"), "egui_expressive::blend_color(tokens::SURFACE, tokens::SURFACE, egui_expressive::BlendMode::Multiply)");
}

function testGenerateSidecar() {
  const ab = { name: "Artboard 1", width: 100, height: 100 };
  const els = [{
    id: "el_1", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0,
    appearance_fills: [{ color: { r: 255, g: 0, b: 0 }, opacity: 0.5, blendMode: "multiply" }],
    appearance_strokes: [{ color: { r: 0, g: 255, b: 0 }, width: 2, opacity: 1, blendMode: "screen" }],
    effects: [{ type: "dropShadow", color: { r: 0, g: 0, b: 0, a: 0.5 }, blur: 4, x: 0, y: 2 }], notes: [],
    children: [{
      id: "el_2", type: "text", x: 0, y: 0, w: 10, h: 10, depth: 1,
      text: "Hello", textStyle: { size: 12, weight: 400, family: "Arial" },
      effects: [], notes: []
    }]
  }, {
    id: "ell_1", type: "ellipse", x: 10, y: 10, w: 20, h: 12, depth: 0,
    effects: [], notes: []
  }];
  const colorMap = new Map();
  const sidecar = JSON.parse(plugin.generateSidecar(ab, els, colorMap));
  assert.strictEqual(sidecar.elements[0].appearanceFills[0].color, "#ff0000");
  assert.strictEqual(sidecar.elements[0].appearanceFills[0].blendMode, "multiply");
  assert.strictEqual(sidecar.elements[0].appearanceStrokes[0].color, "#00ff00");
  assert.strictEqual(sidecar.elements[0].appearanceStrokes[0].blendMode, "screen");
  assert.strictEqual(sidecar.elements[0].children[0].text, "Hello");
  assert.strictEqual(sidecar.elements[1].type, "ellipse");

  const stack = sidecar.elements[0].appearanceStack;
  assert(stack.some(e => e.entryType === 'effect' && e.effectType === 'dropShadow'));
}

function testNoWithBlendModeEmission() {
  const results = [{
    artboard: { name: "Artboard 1", width: 100, height: 100 },
    elements: [{
      id: "el_1", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0,
      fill: { r: 255, g: 0, b: 0 }, blendMode: "multiply", opacity: 0.5,
      effects: [], notes: []
    }]
  }];
  const options = { naming: false };
  const exported = plugin.exportFromRawData ? plugin.exportFromRawData(results, options) : null;
  if (exported) {
    const code = exported.files["artboard_1.rs"];
    assert(!code.includes("with_blend_mode"), "Should not emit with_blend_mode");
    assert(code.includes("composite_layers_gpu"), "Should emit composite_layers_gpu");
    assert(code.includes("BlendLayer"), "Should emit BlendLayer");
  }
}

function testImageOpacityEmission() {
  const results = [{
    artboard: { name: "Artboard 1", width: 100, height: 100 },
    elements: [{
      id: "img_1", type: "image", x: 0, y: 0, w: 10, h: 10, depth: 0,
      imagePath: "test.png", opacity: 0.5,
      effects: [], notes: []
    }]
  }];
  const options = { naming: false };
  const exported = plugin.exportFromRawData ? plugin.exportFromRawData(results, options) : null;
  if (exported) {
    const code = exported.files["artboard_1.rs"];
    assert(code.includes("egui::Color32::from_rgba_unmultiplied(255, 255, 255, 128)"), "Should emit alpha tint for image");
  }
}

function testPathRichStrokeAndAppearanceEmission() {
  const results = [{
    artboard: { name: "Artboard 1", width: 100, height: 100 },
    elements: [{
      id: "path_1", type: "shape", x: 0, y: 0, w: 20, h: 20, depth: 0,
      pathPoints: [
        { anchor: [0, 0], leftDir: [0, 0], rightDir: [0, 0] },
        { anchor: [20, 0], leftDir: [20, 0], rightDir: [20, 0] },
        { anchor: [20, 20], leftDir: [20, 20], rightDir: [20, 20] }
      ],
      pathClosed: true,
      appearanceStack: [
        { type: "fill", color: { r: 255, g: 0, b: 0 }, opacity: 1 },
        { type: "stroke", color: { r: 0, g: 0, b: 0 }, width: 2, cap: "round", join: "bevel" }
      ],
      effects: [], notes: []
    }]
  }];
  const exported = plugin.exportFromRawData ? plugin.exportFromRawData(results, { naming: false }) : null;
  if (exported) {
    const code = exported.files["artboard_1.rs"];
    assert(code.includes("scene::render_node"), "Should render path appearance stack through scene renderer");
    assert(code.includes("egui_expressive::codegen::StrokeCap::Round"), "Should preserve path stroke cap");
    assert(code.includes("egui_expressive::codegen::StrokeJoin::Bevel"), "Should preserve path stroke join");
  }
}

function testRichCircleAndStrokeOpacityEmission() {
  const results = [{
    artboard: { name: "Artboard 1", width: 100, height: 100 },
    elements: [
      {
        id: "circle_1", type: "circle", x: 0, y: 0, w: 20, h: 20, depth: 0,
        stroke: { r: 0, g: 0, b: 0, width: 2 }, strokeDash: [2, 4], strokeCap: "round",
        opacity: 0.5, effects: [], notes: []
      },
      {
        id: "rect_1", type: "shape", x: 30, y: 0, w: 20, h: 20, depth: 1,
        stroke: { r: 255, g: 0, b: 0, width: 1 }, opacity: 0.5, blendMode: "multiply",
        effects: [], notes: []
      },
      {
        id: "rect_2", type: "shape", x: 60, y: 0, w: 20, h: 20, depth: 2,
        stroke: { r: 0, g: 255, b: 0, width: 1 }, opacity: 0.5,
        effects: [], notes: []
      },
      {
        id: "rot_1", type: "shape", x: 0, y: 30, w: 20, h: 20, depth: 3,
        stroke: { r: 0, g: 0, b: 255, width: 1 }, strokeDash: [3, 3], rotation: 15,
        effects: [], notes: []
      },
      {
        id: "ellipse_1", type: "ellipse", x: 0, y: 60, w: 30, h: 10, depth: 4,
        fill: { r: 0, g: 0, b: 255 }, stroke: { r: 0, g: 0, b: 0, width: 1 },
        pathPoints: [
          { anchor: [0, 65], leftDir: [0, 65], rightDir: [0, 65] },
          { anchor: [15, 50], leftDir: [15, 50], rightDir: [15, 50] },
          { anchor: [30, 65], leftDir: [30, 65], rightDir: [30, 65] },
          { anchor: [15, 80], leftDir: [15, 80], rightDir: [15, 80] }
        ],
        effects: [], notes: []
      }
    ]
  }];
  const exported = plugin.exportFromRawData ? plugin.exportFromRawData(results, { naming: false }) : null;
  if (exported) {
    const code = exported.files["artboard_1.rs"];
    assert(code.includes("egui_expressive::dashed_path"), "Circle rich stroke should use dashed_path");
    assert(code.includes("egui_expressive::StrokeCap::Round"), "Circle cap should be preserved");
    assert(code.includes("0..=48"), "Circle rich stroke path should be closed");
    assert(code.includes("pts.push(pts[0])"), "Rotated rich stroke path should be closed");
    assert(code.includes("origin + egui::vec2(15.0, 50.0)"), "Ellipse should use parser path points");
    assert(code.includes("with_alpha(tokens::COLOR_00FF00, 0.5)"), "Normal stroke opacity should be emitted");
    assert(code.includes("egui_expressive::BlendMode::Multiply"), "Stroke blend mode should be emitted");
  }
}

function testGradientOnlyVectorPaths() {
  const results = [{
    artboard: { name: "Artboard 1", width: 100, height: 100 },
    elements: [
      {
        id: "circle_1", type: "circle", x: 0, y: 0, w: 20, h: 20, depth: 0,
        gradient: { type: "linear", stops: [{color: {r:255,g:0,b:0}, position: 0}, {color: {r:0,g:255,b:0}, position: 1}], origin: [0,0], destination: [20,20] },
        effects: [], notes: []
      },
      {
        id: "ellipse_1", type: "ellipse", x: 30, y: 0, w: 30, h: 20, depth: 1,
        gradient: { type: "radial", center: { x: 44, y: 9 }, focalPoint: { x: 46, y: 11 }, radius: 18, transform: [1, 0, 0, 1, 4, 5], stops: [{color: {r:255,g:0,b:0}, position: 0}, {color: {r:0,g:255,b:0}, position: 1}], origin: [45,10], destination: [60,10] },
        pathPoints: [
          { anchor: [30, 10], leftDir: [30, 10], rightDir: [30, 10] },
          { anchor: [45, 0], leftDir: [45, 0], rightDir: [45, 0] },
          { anchor: [60, 10], leftDir: [60, 10], rightDir: [60, 10] },
          { anchor: [45, 20], leftDir: [45, 20], rightDir: [45, 20] }
        ],
        effects: [], notes: []
      },
      {
        id: "path_1", type: "shape", x: 0, y: 30, w: 20, h: 20, depth: 2,
        gradient: { type: "linear", stops: [{color: {r:255,g:0,b:0}, position: 0}, {color: {r:0,g:255,b:0}, position: 1}], origin: [0,30], destination: [20,50] },
        pathPoints: [
          { anchor: [0, 30], leftDir: [0, 30], rightDir: [0, 30] },
          { anchor: [20, 30], leftDir: [20, 30], rightDir: [20, 30] },
          { anchor: [20, 50], leftDir: [20, 50], rightDir: [20, 50] }
        ],
        pathClosed: true,
        effects: [], notes: []
      },
      {
        id: "rounded_gradient", type: "shape", x: 30, y: 30, w: 20, h: 20, depth: 3,
        cornerRadius: 6,
        stroke: { r: 0, g: 0, b: 0, width: 1 }, strokeDash: [2, 2],
        gradient: { type: "linear", stops: [{color: {r:255,g:0,b:0}, position: 0}, {color: {r:0,g:255,b:0}, position: 1}] },
        effects: [], notes: []
      }
    ]
  }];
  const exported = plugin.exportFromRawData ? plugin.exportFromRawData(results, { naming: false }) : null;
  if (exported) {
    const code = exported.files["artboard_1.rs"];
    const meshMatches = code.match(/egui_expressive::gradient_path_mesh/g);
    assert(meshMatches && meshMatches.length >= 3, "Should emit gradient_path_mesh for gradient-only vector paths");
    assert(!code.includes("radial_gradient_rect_stops"), "Should not emit radial_gradient_rect_stops for vector paths");
    assert(!code.includes("linear_gradient_rect"), "Should not emit linear_gradient_rect for vector paths");
    assert(code.includes("Some(origin + egui::vec2(44.0, 9.0))"), "Radial center should be emitted");
    assert(code.includes("Some(origin + egui::vec2(46.0, 11.0))"), "Radial focal point should be emitted");
    assert(code.includes("Some(18.0)"), "Radial radius should be emitted");
    assert(code.includes("egui_expressive::Transform2D"), "Radial transform should be emitted");
    assert(code.includes("egui_expressive::rounded_rect_path(rect, 6.0)"), "Rounded gradient rect should use rounded path clip");
    assert((code.match(/rounded_rect_path\(rect, 6\.0\)/g) || []).length >= 2, "Rounded gradient stroke should reuse rounded path");
    assert(code.includes("origin + egui::vec2(30.0, 10.0)"), "Ellipse should use parser path points");
    assert(code.includes("origin + egui::vec2(20.0, 30.0)"), "Path should use parser path points");
  }
}

function testPatternFillEmitsVectorPrimitive() {
  const results = [{
    artboard: { name: "Artboard 1", width: 100, height: 100 },
    elements: [
      {
        id: "pattern_rect", type: "shape", x: 10, y: 10, w: 40, h: 20, depth: 0,
        gradient: { type: "pattern", patternName: "Diagonal Dots", scale: [1, 1] },
        stroke: { r: 0, g: 0, b: 0, width: 1 },
        effects: [], notes: []
      },
      {
        id: "unknown_gradient", type: "shape", x: 60, y: 10, w: 20, h: 20, depth: 1,
        gradient: { type: "conic", scale: [2, 2] },
        effects: [], notes: []
      }
    ]
  }];
  const exported = plugin.exportFromRawData ? plugin.exportFromRawData(results, { naming: false, includeSidecar: true }) : null;
  if (exported) {
    const code = exported.files["artboard_1.rs"];
    assert(code.includes("egui_expressive::pattern_fill_path"), "Pattern fill should emit vector pattern primitive");
    assert(code.includes("Pattern fill \"Diagonal Dots\""), "Pattern metadata should be preserved in generated code");
    assert(code.includes("Pattern fill \"conic\""), "Unknown gradient metadata should be preserved as a vector pattern primitive");
    assert(!code.includes("approximate with solid fill"), "Pattern fill should not fall back to solid fill");
    assert(!code.includes("linear_gradient_rect(rect"), "Pattern fill should not be treated as a linear gradient");
    const seedMatch = code.match(/pattern_fill_path\([^,]+,\s*(\d+)u32,\s*egui::Color32::from_rgba_unmultiplied\([^)]+\),\s*egui::Color32::from_rgba_unmultiplied\([^)]+\),\s*([\d.]+),\s*([\d.]+)\)/);
    assert(seedMatch, "Pattern code should include seed/cell/mark parameters");
    const sidecar = JSON.parse(exported.files["artboard_1.json"]);
    assert.strictEqual(sidecar.elements[0].gradient.seed, Number(seedMatch[1]));
    assert.strictEqual(sidecar.elements[0].gradient.cellSize, Number(seedMatch[2]));
    assert.strictEqual(sidecar.elements[0].gradient.markSize, Number(seedMatch[3]));
    assert.strictEqual(sidecar.elements[1].gradient.patternName, "conic");
    assert(Number.isInteger(sidecar.elements[1].gradient.seed));
    assert.strictEqual(sidecar.elements[1].gradient.cellSize, 16);
  }
}

function testAppearanceBlendStackUsesSceneRenderer() {
  const results = [{
    artboard: { name: "Artboard 1", width: 100, height: 100 },
    elements: [
      {
        id: "blend_stack", type: "shape", x: 0, y: 0, w: 50, h: 30, depth: 0,
        appearanceStack: [
          { type: "fill", blendMode: "multiply", opacity: 0.8, gradient: { type: "linear", angle: 30, stops: [{ color: { r: 255, g: 0, b: 0 }, position: 0 }, { color: { r: 0, g: 0, b: 255 }, position: 1 }] } },
          { type: "stroke", color: { r: 0, g: 0, b: 0 }, width: 2, blendMode: "screen", dash: [2, 2], cap: "round", join: "bevel" }
        ],
        effects: [], notes: []
      },
      {
        id: "circle_stack", type: "circle", x: 60, y: 0, w: 20, h: 20, depth: 1,
        appearanceStack: [{ type: "fill", color: { r: 0, g: 255, b: 0 }, blendMode: "multiply" }],
        effects: [], notes: []
      },
      {
        id: "path_stack", type: "path", x: 0, y: 40, w: 30, h: 30, depth: 2,
        pathPoints: [
          { anchor: [0, 40], leftDir: [0, 40], rightDir: [0, 40] },
          { anchor: [30, 40], leftDir: [30, 40], rightDir: [30, 40] },
          { anchor: [15, 70], leftDir: [15, 70], rightDir: [15, 70] }
        ],
        pathClosed: true,
        appearanceStack: [
          { type: "fill", color: { r: 255, g: 255, b: 0 }, blendMode: "normal" },
          { type: "stroke", color: { r: 0, g: 0, b: 0 }, width: 1, blendMode: "normal" }
        ],
        effects: [], notes: []
      },
      {
        id: "open_path_stack", type: "path", x: 0, y: 80, w: 30, h: 1, depth: 3,
        pathPoints: [
          { anchor: [0, 80], leftDir: [0, 80], rightDir: [0, 80] },
          { anchor: [30, 80], leftDir: [30, 80], rightDir: [30, 80] }
        ],
        pathClosed: false,
        appearanceStack: [{ type: "stroke", color: { r: 0, g: 0, b: 0 }, width: 1, blendMode: "normal" }],
        effects: [], notes: []
      },
      {
        id: "shape_effect_stack", type: "shape", x: 40, y: 40, w: 20, h: 20, depth: 4,
        appearanceStack: [
          { type: "fill", color: { r: 255, g: 255, b: 255 }, blendMode: "normal" },
          { type: "dropShadow", color: { r: 0, g: 0, b: 0, a: 0.5 }, x: 2, y: 2, blur: 4, blendMode: "normal" }
        ],
        effects: [], notes: []
      },
      {
        id: "parser_effect_stack", type: "shape", x: 70, y: 40, w: 20, h: 20, depth: 5,
        appearance_fills: [{ color: { r: 255, g: 255, b: 255 }, opacity: 1, blendMode: "normal" }],
        effects: [{ type: "dropShadow", color: { r: 0, g: 0, b: 0, a: 0.5 }, x: 1, y: 1, blur: 2, blendMode: "normal" }],
        appearance_strokes: [{ color: { r: 0, g: 0, b: 0 }, width: 1, opacity: 1, blendMode: "normal" }],
        notes: []
      }
    ]
  }];
  const exported = plugin.exportFromRawData ? plugin.exportFromRawData(results, { naming: false }) : null;
  if (exported) {
    const code = exported.files["artboard_1.rs"];
    assert(code.includes("scene::render_node"), "Blend appearance stack should route through scene renderer");
    assert(code.includes("Appearance stack contains layer blend modes"));
    assert(code.includes("egui_expressive::codegen::BlendMode::Multiply"));
    assert(code.includes("egui_expressive::codegen::BlendMode::Screen"));
    assert(code.includes("PaintSource::LinearGradient"));
    assert(code.includes("id: \"circle_stack\".to_string()"), "Circle appearance stack should use scene renderer");
    assert(code.includes("id: \"path_stack\".to_string()"), "Path appearance stack should use scene renderer");
    assert(code.includes("id: \"open_path_stack\".to_string()"), "Open 2-point path appearance stack should use scene renderer");
    assert(code.includes("closed: false"), "Open path should remain open scene path");
    assert(code.includes("id: \"shape_effect_stack\".to_string()"), "Explicit shape effect stack should use scene renderer");
    assert(code.includes("id: \"parser_effect_stack\".to_string()"), "Parser-sourced fill/effect/stroke stack should use scene renderer");
    assert(code.includes("EffectType::DropShadow"), "Appearance-stack effects should be preserved in scene renderer");
    assert(code.includes("Geometry::Path"), "Path appearance stack should preserve path geometry");
  }
}

function testIllustratorRadialGradientGeometryExtraction() {
  const item = {
    fillColor: {
      typename: "GradientColor",
      angle: 15,
      origin: { x: 110, y: 190 },
      length: 25,
      hiliteLength: 10,
      hiliteAngle: 0,
      matrix: { mValueA: 2, mValueB: 0.5, mValueC: 0.25, mValueD: 3, mValueTX: 5, mValueTY: 7 },
      gradient: {
        type: 2,
        gradientStops: [
          { rampPoint: 0, opacity: 50, color: { typename: "RGBColor", red: 255, green: 0, blue: 0 } },
          { rampPoint: 100, color: { typename: "RGBColor", red: 0, green: 0, blue: 255 } }
        ]
      }
    }
  };
  const gradient = plugin.getGradient(item, [100, 200, 300, 0]);
  assert.strictEqual(gradient.type, "radial");
  assert.strictEqual(gradient.center.x, 10);
  assert.strictEqual(gradient.center.y, 10);
  assert.strictEqual(gradient.focalPoint.x, 20);
  assert.strictEqual(gradient.focalPoint.y, 10);
  assert.strictEqual(gradient.radius, 25);
  assert.deepStrictEqual(Array.from(gradient.transform), [2, -0.5, -0.25, 3, 155, -457]);
  assert.strictEqual(gradient.stops[0].opacity, 0.5);
}

testIllustratorRadialGradientGeometryExtraction();
testGradientOnlyVectorPaths();
testPatternFillEmitsVectorPrimitive();
testAppearanceBlendStackUsesSceneRenderer();
testPortableAssetPath();
testApplyBlendExpr();
testGenerateSidecar();
testNoWithBlendModeEmission();
testImageOpacityEmission();
testPathRichStrokeAndAppearanceEmission();
testRichCircleAndStrokeOpacityEmission();
testBundledParserCandidates();
testMergeParserDataByBounds();
testWarningsUsePortableImagePath();
testStaticSecurityChecks();
testIndexBootstrap();
testAriaPressedToggle();
testHostJsx();
testFileTreeAndCodePreview();
testHostSaveFailureHandling();
