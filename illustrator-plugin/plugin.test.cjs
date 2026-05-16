const assert = require('assert');
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const pluginSourceForVm = fs.readFileSync(path.join(__dirname, 'plugin.js'), 'utf8');
const hostSourceForVm = fs.readFileSync(path.join(__dirname, 'host.jsx'), 'utf8');
new Function(hostSourceForVm);

function createHostSandbox(overrides = {}) {
  const sandbox = {
    console,
    JSON,
    URL,
    Math,
    Date,
    Array,
    String,
    Number,
    Boolean,
    RegExp,
    Object,
    parseInt,
    parseFloat,
    isFinite,
    Folder: {
      temp: { fsName: '/tmp', exists: true, create() { this.exists = true; } },
      desktop: { fsName: '/tmp/Desktop', exists: true, create() { this.exists = true; } },
      myDocuments: { fsName: '/tmp/Documents', exists: true, create() { this.exists = true; } },
    },
    File: function File(pathValue) {
      this.fsName = pathValue;
      this.exists = true;
      this.parent = { exists: true };
      this.copy = () => true;
      this.open = () => true;
      this.write = () => true;
      this.writeln = () => true;
      this.close = () => {};
    },
    DocumentColorSpace: { RGB: 'RGB' },
    ElementPlacement: { PLACEATEND: 'PLACEATEND' },
    SaveOptions: { DONOTSAVECHANGES: 'DONOTSAVECHANGES' },
    PointType: { SMOOTH: 'SMOOTH' },
    app: null,
    ...overrides,
  };
  vm.runInNewContext(hostSourceForVm, sandbox, { filename: 'host.jsx' });
  return sandbox;
}

const sandbox = {
  module: { exports: {} },
  exports: {},
  require,
  console: { warn() {}, error() {} },
  URL,
  process,
  Justification: {
    LEFT: 'LEFT',
    CENTER: 'CENTER',
    RIGHT: 'RIGHT',
    FULLJUSTIFY: 'FULLJUSTIFY',
    FULLJUSTIFYLASTLINELEFT: 'FULLJUSTIFYLASTLINELEFT',
    FULLJUSTIFYLASTLINECENTER: 'FULLJUSTIFYLASTLINECENTER',
    FULLJUSTIFYLASTLINERIGHT: 'FULLJUSTIFYLASTLINERIGHT'
  },
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
  assert.strictEqual(plugin.getAiParserCandidates('/extension/root', 'sunos').length, 0);
  assert(plugin.AI_PARSER_MAX_BUFFER_BYTES >= 64 * 1024 * 1024);
  assert(pluginSourceForVm.includes('maxBuffer: AI_PARSER_MAX_BUFFER_BYTES'));
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

  const crossArtboardDom = [{ id: 'shared_id', type: 'shape', x: 10, y: 20, w: 30, h: 40, children: [], effects: [] }];
  const crossArtboardParser = {
    elements: [{
      id: 'shared_id',
      element_type: 'path',
      artboard_name: 'Page_2',
      bounds: [10, 20, 30, 40],
      rotation_deg: 90,
      path_points: [{ anchor: [10, 20], left_ctrl: [10, 20], right_ctrl: [20, 20] }]
    }, {
      id: 'right_artboard_same_bounds',
      element_type: 'path',
      artboard_name: 'Page_1',
      bounds: [10, 20, 30, 40],
      rotation_deg: 45,
      path_points: [{ anchor: [10, 20], left_ctrl: [10, 20], right_ctrl: [25, 20] }]
    }]
  };
  const crossArtboardMerged = plugin.mergeAiParserData(crossArtboardDom, crossArtboardParser, 'Page_1');
  assert.strictEqual(crossArtboardMerged.length, 1);
  assert.strictEqual(crossArtboardMerged[0].parserId, 'right_artboard_same_bounds');
  assert.strictEqual(crossArtboardMerged[0].rotation, 45);

  plugin.resetAiParserStateForTests();
  const unscopedDom = [{ id: 'unscoped_vector', type: 'shape', x: 10, y: 20, w: 30, h: 40, children: [], effects: [] }];
  const unscopedParser = {
    elements: [{
      id: 'unscoped_vector',
      element_type: 'path',
      bounds: [10, 20, 30, 40],
      path_points: [{ anchor: [10, 20], left_ctrl: [10, 20], right_ctrl: [20, 20] }]
    }]
  };
  const unscopedMerged = plugin.mergeAiParserData(unscopedDom, unscopedParser, 'Page_1');
  assert.strictEqual(unscopedMerged.length, 1);
  assert.strictEqual(unscopedMerged[0].parserId, undefined, 'Parser data without artboard provenance must not enrich a known artboard');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Page_1', width: 100, height: 100 }, elements: unscopedMerged }], { naming: false });
  }, /ai-parser artboard provenance missing/);
  plugin.resetAiParserStateForTests();
}

function testMergeParserDataAddsUnmatchedCodeDrawnVectors() {
  const parser = {
    elements: [{
      id: 'pdf_path_1_0',
      element_type: 'shape',
      artboard_name: 'Page_1',
      bounds: [10, 20, 30, 40],
      path_closed: true,
      path_points: [
        { anchor: [10, 20], left_ctrl: [10, 20], right_ctrl: [10, 20] },
        { anchor: [40, 20], left_ctrl: [40, 20], right_ctrl: [40, 20] },
        { anchor: [40, 60], left_ctrl: [40, 60], right_ctrl: [40, 60] },
      ],
      appearance_fills: [{ r: 12, g: 34, b: 56, a: 255, opacity: 1, blend_mode: 'normal' }]
    }, {
      id: 'pdf_path_2_0',
      element_type: 'shape',
      artboard_name: 'Page_2',
      bounds: [0, 0, 10, 10],
      path_points: [{ anchor: [0, 0], left_ctrl: [0, 0], right_ctrl: [0, 0] }],
      appearance_fills: [{ r: 1, g: 2, b: 3, a: 255 }]
    }]
  };
  const merged = plugin.mergeAiParserData([], parser, 'Page_1');
  assert.strictEqual(merged.length, 1);
  assert.strictEqual(merged[0].parserOnly, true);
  assert.strictEqual(merged[0].type, 'path');
  assert.strictEqual(merged[0].appearance_fills[0].r, 12);
  assert.strictEqual(plugin.parityStatusForElement(merged[0]), 'approximate');
  assert(plugin.parityFindingsForElement(merged[0]).some(finding => finding.reason.includes('hierarchy/depth')));

  const exported = plugin.exportFromRawData([{ artboard: { name: 'Page_1', width: 100, height: 100 }, elements: merged }], { includeSidecar: true });
  const code = exported.files['page_1.rs'];
  assert(code.includes('scene::render_node'), 'parser-only vectors should reach generated scene code');
  assert(code.includes('egui::Color32::from_rgba_unmultiplied(12, 34, 56, 255)'), 'parser fill color should be drawn as code');
  const sidecar = JSON.parse(exported.files['page_1.json']);
  assert.strictEqual(sidecar.elements[0].parityStatus, 'approximate');

  const artboardAliasMerged = plugin.mergeAiParserData([], parser, 'Artboard 1');
  assert.strictEqual(artboardAliasMerged.length, 1, 'Parser Page_1 data should merge into Illustrator Artboard 1 exports');
}

function testMergeParserDataPreservesHierarchyAndAppearance() {
  const parser = {
    elements: [{
      id: 'parser_group',
      element_type: 'group',
      artboard_name: 'Page_1',
      bounds: [0, 0, 100, 100],
      layer_name: 'Foreground',
      z_order: 7,
      children: [{
        id: 'parser_child',
        element_type: 'path',
        artboard_name: 'Page_1',
        bounds: [10, 10, 30, 30],
        path_closed: true,
        path_points: [
          { anchor: [10, 10], left_ctrl: [10, 10], right_ctrl: [10, 10] },
          { anchor: [40, 10], left_ctrl: [40, 10], right_ctrl: [40, 10] },
          { anchor: [40, 40], left_ctrl: [40, 40], right_ctrl: [40, 40] }
        ],
        appearance_fills: [{ r: 90, g: 80, b: 70, a: 255, opacity: 0.75, blend_mode: 'multiply' }],
        appearance_strokes: [{ r: 10, g: 20, b: 30, a: 255, width: 2, cap: 'round', join: 'bevel', dash: [3, 2] }]
      }]
    }]
  };

  const merged = plugin.mergeAiParserData([], parser, 'Page_1');
  assert.strictEqual(merged.length, 1, 'Top-level parser group should not be flattened away');
  assert.strictEqual(merged[0].id, 'parser_group');
  assert.strictEqual(merged[0].type, 'group');
  assert.strictEqual(merged[0].layerName, 'Foreground');
  assert.strictEqual(merged[0].zOrder, 7);
  assert.strictEqual(merged[0].children.length, 1, 'Parser child should remain nested under its group');
  assert.strictEqual(merged[0].children[0].id, 'parser_child');
  assert.strictEqual(merged[0].children[0].depth, 1);
  assert.strictEqual(merged[0].children[0].appearance_fills[0].blend_mode, 'multiply');

  const exported = plugin.exportFromRawData([{ artboard: { name: 'Page_1', width: 100, height: 100 }, elements: merged }], { naming: false, codeOnlyStrict: true, sidecar: true });
  const code = exported.files['page_1.rs'];
  assert(code.includes('.with_child('), 'Generated scene code should preserve parser hierarchy');
  assert(code.includes('FillLayer::paint'), 'Generated scene code should preserve parser fills');
  assert(code.includes('StrokeLayer::new(2.0'), 'Generated scene code should preserve parser strokes');
  assert(code.includes('.dash(vec![3.0, 2.0])'), 'Generated scene code should preserve parser stroke dash');
  assert(code.includes('BlendMode::Multiply'), 'Generated scene code should preserve appearance blend modes');
  const sidecar = JSON.parse(exported.files['page_1.json']);
  assert.strictEqual(sidecar.elements[0].children[0].appearanceFills[0].blendMode, 'multiply');
  assert.strictEqual(sidecar.elements[0].children[0].appearanceStrokes[0].width, 2);
}

function testWarningsUsePortableImagePath() {
  const warnings = plugin.collectWarnings([{ id: 'img', type: 'image', imagePath: '/tmp/Secret Folder/photo.jpg', blendMode: 'normal' }], {});
  assert(warnings.some(w => w.note.includes('_photo.jpg')));
  assert(warnings.some(w => w.note.includes('could not be vectorized') && w.note.includes('will not be exported as raster')));
  assert(!warnings.some(w => w.note.includes('/tmp/Secret Folder')));
}

async function testMixedClipRasterVectorizationRecovery() {
  const clipSourceImage = {
    id: 'clip_source_image_recovery',
    type: 'image',
    x: 0,
    y: 0,
    w: 20,
    h: 20,
    depth: 1,
    vectorSourcePath: '/tmp/clip_source_recovery.png',
    effects: [],
    notes: []
  };
  const results = [{
    artboard: { name: 'Artboard 1', width: 100, height: 100 },
    elements: [{
      id: 'clip_raster_recovery',
      type: 'group',
      clipMask: true,
      x: 0,
      y: 0,
      w: 100,
      h: 100,
      depth: 0,
      children: [clipSourceImage],
      effects: [],
      notes: []
    }]
  }];

  let attemptedTrace = false;
  await sandbox.vectorizeRasterImagesForResults(results, async (el, artboardName) => {
    attemptedTrace = true;
    assert.strictEqual(artboardName, 'Artboard 1');
    assert.strictEqual(plugin.rasterVectorSourcePath(el), clipSourceImage.vectorSourcePath);
    return rectTraceResult({ x: 0, y: 0, w: 20, h: 20 }, artboardName);
  });

  assert.strictEqual(attemptedTrace, true, 'Vector source child inside clip group should be traced before strict export');
  assert.strictEqual(results[0].elements[0].children[0].type, 'group');
  assert.strictEqual(results[0].elements[0].children[0].rasterVectorized, true);
  assert.doesNotThrow(() => {
    plugin.exportFromRawData(results, { naming: false, codeOnlyStrict: true });
  });

  const missingSourceResults = [{
    artboard: { name: 'Artboard 1', width: 100, height: 100 },
    elements: [{
      id: 'clip_raster_missing_source',
      type: 'group',
      clipMask: true,
      x: 0,
      y: 0,
      w: 100,
      h: 100,
      depth: 0,
      children: [{ id: 'clip_missing_source_image', type: 'image', x: 0, y: 0, w: 20, h: 20, depth: 1, effects: [], notes: [] }],
      effects: [],
      notes: []
    }]
  }];
  assert.throws(() => {
    plugin.exportFromRawData(missingSourceResults, { naming: false, codeOnlyStrict: true });
  }, /mixed clipping groups containing text are not parity-safe yet|vectorize raster before clip as a preflight requirement|vector tracing/);
}

function testTextUnitsOpacityAndParityStatus() {
  assert(Math.abs(plugin.illustratorTrackingToPx(200, 12) - 2.4) < 0.0001);
  assert.strictEqual(plugin.illustratorLeadingToMultiplier(18, 12), 1.5);
  assert.strictEqual(plugin.getTextAlign({
    typename: 'TextFrame',
    textRange: { paragraphAttributes: { justification: sandbox.Justification.FULLJUSTIFYLASTLINELEFT } }
  }), 'justified');
  assert.strictEqual(plugin.getTextAlign({
    typename: 'TextFrame',
    textRange: { paragraphAttributes: { justification: sandbox.Justification.FULLJUSTIFYLASTLINECENTER } }
  }), 'justified_last_line_center');
  assert.strictEqual(plugin.getTextAlign({
    typename: 'TextFrame',
    textRange: { paragraphAttributes: { justification: sandbox.Justification.FULLJUSTIFYLASTLINERIGHT } }
  }), 'justified_last_line_right');
  assert.strictEqual(plugin.getTextAlign({
    typename: 'TextFrame',
    textRange: { paragraphAttributes: { justification: sandbox.Justification.FULLJUSTIFY } }
  }), 'justified_all');

  const directTextElements = [];
  sandbox.extractRecursive({
    typename: 'TextFrame',
    name: 'direct_rich_text',
    contents: 'Rich',
    geometricBounds: [20, 100, 90, 60],
    visibleBounds: [20, 100, 90, 60],
    opacity: 100,
    textRange: {
      characterAttributes: {
        size: 18,
        textFont: { name: 'Test Sans' },
        ligatures: false,
        baselineShift: 3,
        horizontalScale: 120,
        verticalScale: 90,
        tracking: 100,
        leading: 36,
        underline: true,
        allCaps: true,
      },
      paragraphAttributes: { justification: sandbox.Justification.FULLJUSTIFYLASTLINECENTER }
    },
    textRanges: [{ contents: 'Rich', characterAttributes: { size: 18, textFont: { name: 'Test Sans' }, tracking: 100, leading: 36, underline: true, allCaps: true } }],
  }, [0, 120, 220, 0], directTextElements, 0);
  assert.strictEqual(directTextElements[0].textAlign, 'justified_last_line_center');
  assert.strictEqual(directTextElements[0].textStyle.openTypeFeatures.ligatures, false);
  assert.strictEqual(directTextElements[0].textStyle.baselineShift, 3);
  assert.strictEqual(directTextElements[0].textStyle.horizontalScale, 1.2);
  assert.strictEqual(directTextElements[0].textStyle.verticalScale, 0.9);
  assert.strictEqual(directTextElements[0].textStyle.letterSpacing, 1.8);
  assert.strictEqual(directTextElements[0].textStyle.lineHeight, 2);
  assert.strictEqual(directTextElements[0].textStyle.textDecoration, 'underline');
  assert.strictEqual(directTextElements[0].textStyle.textTransform, 'uppercase');

  const results = [{
    artboard: { name: 'Artboard 1', width: 100, height: 100 },
    elements: [{
      id: 'headline', type: 'text', x: 0, y: 0, w: 90, h: 30, depth: 0,
      text: 'Hello', fill: { r: 255, g: 0, b: 0 }, opacity: 0.5,
      textStyle: { size: 12, weight: 700 }, letterSpacing: 2.4, lineHeight: 1.5,
      textDecoration: 'both', textTransform: 'uppercase', effects: [], notes: []
    }, {
      id: 'opentype', type: 'text', x: 0, y: 40, w: 90, h: 30, depth: 1,
      text: 'office 1st', fill: { r: 0, g: 0, b: 0 }, opacity: 1,
      textStyle: {
        size: 14,
        weight: 400,
        openTypeFeatures: {
          ligatures: false,
          fractions: true,
          ordinals: true,
          stylisticAlternates: true
        },
        baselineShift: 2,
        horizontalScale: 1.1,
        verticalScale: 0.9
      },
      effects: [], notes: []
    }]
  }];
  const exported = plugin.exportFromRawData ? plugin.exportFromRawData(results, { naming: false, sidecar: true, codeOnlyStrict: false }) : null;
  if (exported) {
    const code = exported.files['artboard_1.rs'];
    assert(code.includes('.letter_spacing(2.4)'), 'Tracking should be converted to px letter spacing');
    assert(code.includes('.line_height(1.5)'), 'Leading should be converted to TextBlock line-height multiplier');
    assert(code.includes('with_alpha(tokens::'), 'Text opacity should be applied to token color');
    assert(code.includes('TextDecoration::Both'), 'Underline + strikethrough should map to Both');
    assert(code.includes('TextTransform::Uppercase'), 'All caps should map to Uppercase');
    assert(code.includes('.layout_width(90.0)'), 'Text alignment frame should use layout_width');
    assert(code.includes('.baseline_shift(2.0)'), 'Baseline shift should emit TypeSpec builder');
    assert(code.includes('.horizontal_scale(1.1)'), 'Horizontal scale should emit TypeSpec builder');
    assert(code.includes('.vertical_scale(0.9)'), 'Vertical scale should emit TypeSpec builder');
    assert(code.includes('OpenTypeFeatures'), 'OpenType overrides should emit OpenTypeFeatures');
    assert(code.includes('ligatures: false'), 'Disabled ligatures should be preserved');
    assert(code.includes('fractions: true'), 'Fractions feature should be preserved');
    assert(code.includes('ordinals: true'), 'Ordinals feature should be preserved');
    assert(code.includes('stylistic_alternates: true'), 'Stylistic alternates should be preserved');
    const sidecar = JSON.parse(exported.files['artboard_1.json']);
    const opentype = sidecar.elements.find(el => el.id === 'opentype');
    assert.strictEqual(opentype.parityStatus, 'approximate', 'OpenType metric overrides should be approximate, not unsupported');
    assert(opentype.parityReasons.some(reason => reason.includes('advanced OpenType shaping')), 'OpenType parity reason should be explicit');
    assert.strictEqual(plugin.parityStatusForElement({ id: 'plain_text', type: 'text', openTypeFeatures: { ligatures: true }, horizontalScale: 1, verticalScale: 1 }), 'exact', 'Default OpenType values should not degrade parity');
    assert.strictEqual(plugin.parityStatusForElement({ id: 'ligature_off', type: 'text', openTypeFeatures: { ligatures: false } }, { codeOnlyStrict: false }), 'approximate', 'Non-default OpenType values should mark bounded approximation in non-strict mode');
    assert.strictEqual(plugin.parityStatusForElement({ id: 'ligature_off', type: 'text', openTypeFeatures: { ligatures: false } }, { codeOnlyStrict: true }), 'unsupported', 'Disabling ligatures should require a shaper-backed contract in strict mode');
    assert.strictEqual(plugin.parityStatusForElement({ id: 'stylistic', type: 'text', openTypeFeatures: { stylisticAlternates: true } }), 'unsupported', 'Strict glyph substitution features should fail honestly without a shaper-backed contract');
    assert.strictEqual(plugin.parityStatusForElement({ id: 'stylistic', type: 'text', openTypeFeatures: { stylisticAlternates: true } }, { codeOnlyStrict: false }), 'approximate', 'Non-strict glyph substitution features should remain visibly approximate');
    assert.strictEqual(plugin.parityStatusForElement({ id: 'appearance_expanded', type: 'shape', appearanceExpanded: true, appearanceProbe: { fillCount: 3, strokeCount: 0 }, effects: [], notes: [] }), 'exact', 'Expanded appearances should not remain blocked by the old count mismatch');
  }
}

function testTextShapingContractExportPath() {
  const shapedText = {
    id: 'shaped_text',
    type: 'text',
    x: 12,
    y: 34,
    w: 180,
    h: 40,
    depth: 0,
    text: 'office',
    fill: { r: 0, g: 0, b: 0 },
    opacity: 1,
    textStyle: {
      size: 16,
      openTypeFeatures: { ligatures: false, stylisticAlternates: true }
    },
    shapedGlyphs: [
      { glyphId: 10, cluster: 0, advanceX: 8, advanceY: 0, offsetX: 0, offsetY: 0, contours: [{ points: [[0, 0], [4, 0], [4, 8], [0, 8]], closed: true }] },
      { glyphId: 11, cluster: 1, advanceX: 9, advanceY: 0, offsetX: 0, offsetY: 0, contours: [{ points: [[0, 0], [5, 0], [5, 8], [0, 8]], closed: true }] },
      { glyphId: 12, cluster: 2, advanceX: 7, advanceY: 0, offsetX: 0, offsetY: 0, contours: [{ points: [[0, 0], [3, 0], [3, 8], [0, 8]], closed: true }] }
    ],
    effects: [],
    notes: []
  };

  assert.strictEqual(plugin.parityStatusForElement(shapedText, { codeOnlyStrict: true }), 'exact', 'Explicit shaped glyph data should satisfy strict OpenType export');
  const exported = plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 240, height: 120 }, elements: [shapedText] }], { naming: false, codeOnlyStrict: true, sidecar: true });
  const code = exported.files['artboard_1.rs'];
  assert(code.includes('Text routed through shaped glyph contract'), 'Strict export should use the shaped glyph rendering path');
  assert(code.includes('ShapedGlyphRun {'), 'Strict export should emit a shaped glyph run');
  assert(code.includes('render_shaped_glyph_run(&painter'), 'Strict export should call the shaped glyph renderer');
  const sidecar = JSON.parse(exported.files['artboard_1.json']);
  assert.deepStrictEqual(sidecar.elements[0].shapedGlyphs[0].glyphId, 10);
}

function testOutlinedGlyphsContractExportPath() {
  const outlinedText = {
    id: 'outlined_text',
    type: 'text',
    x: 10,
    y: 20,
    w: 160,
    h: 32,
    depth: 0,
    text: 'AV',
    fill: { r: 0, g: 0, b: 0 },
    opacity: 1,
    textStyle: { size: 18, openTypeFeatures: { kerning: false } },
    shapedGlyphs: [],
    outlinedGlyphs: [
      {
        glyphId: 31,
        cluster: 0,
        advanceX: 7,
        advanceY: 0,
        offsetX: 0,
        offsetY: 0,
        contours: [
          { points: [[0, 0], [6, 0], [6, 10], [0, 10]], closed: true }
        ]
      },
      {
        glyphId: 44,
        cluster: 1,
        advanceX: 8,
        advanceY: 0,
        offsetX: 0,
        offsetY: 0,
        contours: [
          { points: [[0, 0], [4, 0], [4, 8], [0, 8]], closed: true }
        ]
      }
    ],
    effects: [],
    notes: []
  };

  assert.strictEqual(plugin.parityStatusForElement(outlinedText, { codeOnlyStrict: true }), 'exact', 'Explicit outlined glyph data should satisfy strict OpenType export');
  const exported = plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 200, height: 80 }, elements: [outlinedText] }], { naming: false, codeOnlyStrict: true, sidecar: true });
  const code = exported.files['artboard_1.rs'];
  assert(code.includes('Text routed through shaped glyph contract'), 'Outlined glyph contract should use the shaped glyph renderer');
  assert(code.includes('contours: vec!['), 'Outlined glyph contours should be serialized into Rust code');
  assert(code.includes('contours_are_absolute: false'), 'Hand-authored local glyph contours should stay glyph-local');
  assert(code.includes('glyph_id: 31u32'), 'Empty shapedGlyphs should not mask contour-bearing outlinedGlyphs');
  assert(code.includes('path_points(&['), 'Contour points should be emitted as outline geometry');
  const sidecar = JSON.parse(exported.files['artboard_1.json']);
  assert.deepStrictEqual(sidecar.elements[0].outlinedGlyphs[0].glyphId, 31);
  assert.strictEqual(sidecar.elements[0].outlinedGlyphs[0].contours_are_absolute, false);
  assert.strictEqual(sidecar.elements[0].outlinedGlyphs[0].contoursAbsolute, undefined);
  assert.deepStrictEqual(sidecar.elements[0].outlinedGlyphs[0].contours[0].points[0], [0, 0]);
}

function testTextShapingStrictFailsWithoutContract() {
  const unsupportedText = {
    id: 'unsupported_text',
    type: 'text',
    x: 0,
    y: 0,
    w: 100,
    h: 24,
    depth: 0,
    text: 'office',
    fill: { r: 0, g: 0, b: 0 },
    opacity: 1,
    textStyle: {
      size: 16,
      openTypeFeatures: { stylisticAlternates: true }
    },
    effects: [],
    notes: []
  };

  assert.strictEqual(plugin.parityStatusForElement(unsupportedText, { codeOnlyStrict: true }), 'unsupported', 'Missing shaper inputs should fail honestly in strict mode');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 240, height: 120 }, elements: [unsupportedText] }], { naming: false, codeOnlyStrict: true });
  }, /advanced OpenType shaping|unsupported/i);
}

function testTextShapingStrictRejectsFontBytesWithoutOutlines() {
  const fontData = Buffer.from([0, 1, 2, 3]);
  const fontBackedText = {
    id: 'font_backed_text',
    type: 'text',
    x: 0,
    y: 0,
    w: 100,
    h: 24,
    depth: 0,
    text: 'office',
    fill: { r: 0, g: 0, b: 0 },
    opacity: 1,
    textStyle: {
      size: 16,
      openTypeFeatures: { stylisticAlternates: true },
      fontData,
    },
    effects: [],
    notes: []
  };

  assert.strictEqual(plugin.parityStatusForElement(fontBackedText, { codeOnlyStrict: true }), 'unsupported', 'Font bytes without serialized outlines should not satisfy strict advanced shaping');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 240, height: 120 }, elements: [fontBackedText] }], { naming: false, codeOnlyStrict: true, sidecar: true });
  }, /advanced OpenType shaping|unsupported/i);
}

function testTextShapingStrictRejectsStyledRunsWithContours() {
  const styledRunOutlinedText = {
    id: 'styled_run_outlined_text',
    type: 'text',
    x: 10,
    y: 20,
    w: 160,
    h: 32,
    depth: 0,
    text: 'AV',
    fill: { r: 0, g: 0, b: 0 },
    opacity: 1,
    textRuns: [
      { text: 'A', style: { size: 18, openTypeFeatures: { stylisticAlternates: true } } },
      { text: 'V', style: { size: 18, weight: 700 } }
    ],
    outlinedGlyphs: [
      {
        glyphId: 31,
        cluster: 0,
        advanceX: 7,
        advanceY: 0,
        offsetX: 0,
        offsetY: 0,
        contours: [
          { points: [[0, 0], [6, 0], [6, 10], [0, 10]], closed: true }
        ]
      }
    ],
    effects: [],
    notes: []
  };

  assert.strictEqual(plugin.parityStatusForElement(styledRunOutlinedText, { codeOnlyStrict: true }), 'unsupported', 'Styled runs with contour-backed glyphs should not claim exact export until multi-run shaped output is supported');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 240, height: 120 }, elements: [styledRunOutlinedText] }], { naming: false, codeOnlyStrict: true, sidecar: true });
  }, /contour-backed shaped export|unsupported/i);
}

function testTextShapingStrictRejectsEmptyContours() {
  const emptyContourText = {
    id: 'empty_contour_text',
    type: 'text',
    x: 0,
    y: 0,
    w: 100,
    h: 24,
    depth: 0,
    text: 'office',
    fill: { r: 0, g: 0, b: 0 },
    opacity: 1,
    textStyle: {
      size: 16,
      openTypeFeatures: { stylisticAlternates: true }
    },
    shapedGlyphs: [
      { glyphId: 10, cluster: 0, advanceX: 8, advanceY: 0, offsetX: 0, offsetY: 0, contours: [] }
    ],
    effects: [],
    notes: []
  };

  assert.strictEqual(plugin.parityStatusForElement(emptyContourText, { codeOnlyStrict: true }), 'unsupported', 'Empty contour glyphs must not satisfy the strict shaping contract');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 240, height: 120 }, elements: [emptyContourText] }], { naming: false, codeOnlyStrict: true });
  }, /advanced OpenType shaping|unsupported/i);
}

function testTextShapingStrictRejectsPartialContourCoverage() {
  const partialContourText = {
    id: 'partial_contour_text',
    type: 'text',
    x: 0,
    y: 0,
    w: 100,
    h: 24,
    depth: 0,
    text: 'AV',
    fill: { r: 0, g: 0, b: 0 },
    opacity: 1,
    textStyle: {
      size: 16,
      openTypeFeatures: { stylisticAlternates: true }
    },
    shapedGlyphs: [
      { glyphId: 10, cluster: 0, advanceX: 8, advanceY: 0, offsetX: 0, offsetY: 0, contours: [{ points: [[0, 0], [4, 0], [4, 8], [0, 8]], closed: true }] },
      { glyphId: 11, cluster: 1, advanceX: 9, advanceY: 0, offsetX: 0, offsetY: 0, contours: [] }
    ],
    effects: [],
    notes: []
  };

  assert.strictEqual(plugin.parityStatusForElement(partialContourText, { codeOnlyStrict: true }), 'unsupported', 'Every exported shaped glyph must carry contours for exact rendering');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 240, height: 120 }, elements: [partialContourText] }], { naming: false, codeOnlyStrict: true });
  }, /contours for every glyph|unsupported/i);
}

function testTextShapingStrictRejectsMixedContourSpaces() {
  const mixedContourSpaceText = {
    id: 'mixed_contour_space_text',
    type: 'text',
    x: 12,
    y: 34,
    w: 120,
    h: 32,
    depth: 0,
    text: 'AV',
    fill: { r: 0, g: 0, b: 0 },
    opacity: 1,
    textStyle: {
      size: 16,
      openTypeFeatures: { stylisticAlternates: true }
    },
    shapedGlyphs: [
      { glyphId: 10, cluster: 0, advanceX: 8, advanceY: 0, offsetX: 0, offsetY: 0, contoursAbsolute: true, contours: [{ points: [[12, 34], [16, 34], [16, 42], [12, 42]], closed: true }] },
      { glyphId: 11, cluster: 1, advanceX: 9, advanceY: 0, offsetX: 0, offsetY: 0, contours: [{ points: [[0, 0], [5, 0], [5, 8], [0, 8]], closed: true }] }
    ],
    effects: [],
    notes: []
  };

  assert.strictEqual(plugin.parityStatusForElement(mixedContourSpaceText, { codeOnlyStrict: true }), 'unsupported', 'Mixed absolute/local glyph contours must not claim exact shaped export');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 240, height: 120 }, elements: [mixedContourSpaceText] }], { naming: false, codeOnlyStrict: true, sidecar: true });
  }, /one coordinate space per glyph run|unsupported/i);
}

function testSpotTintPatternAndGraphicStyleParity() {
  const tinted = sandbox.colorToRGB({
    typename: 'SpotColor',
    tint: 30,
    spot: { color: { typename: 'RGBColor', red: 100, green: 150, blue: 200 } }
  });
  assert.strictEqual(JSON.stringify(tinted), JSON.stringify({ r: 209, g: 224, b: 239, a: 255 }));

  const patternEl = {
    id: 'pattern_rect', type: 'shape', x: 0, y: 0, w: 20, h: 20, depth: 0,
    gradient: { type: 'pattern', patternName: 'dots' }, effects: [], notes: []
  };
  assert.strictEqual(plugin.parityStatusForElement(patternEl), 'unsupported');
  const patternSidecar = JSON.parse(plugin.generateSidecar({ name: 'Artboard 1', width: 20, height: 20 }, [patternEl], new Map()));
  assert.strictEqual(patternSidecar.elements[0].parityStatus, 'unsupported');
  assert(patternSidecar.elements[0].parityReasons.some(reason => reason.includes('PatternColor fills')));

  const sampledPattern = sandbox.getGradientFromColor({
    typename: 'PatternColor',
    rotation: 0,
    scaleFactor: [100, 100],
    pattern: {
      name: 'Sampled Dots',
      patternItem: {
        pageItems: [
          { typename: 'PathItem', closed: true, filled: true, fillColor: { typename: 'RGBColor', red: 12, green: 34, blue: 56 }, pathPoints: [
            { anchor: [0, 100], leftDirection: [0, 100], rightDirection: [0, 100], pointType: 2 },
            { anchor: [10, 100], leftDirection: [10, 100], rightDirection: [10, 100], pointType: 2 },
            { anchor: [10, 90], leftDirection: [10, 90], rightDirection: [10, 90], pointType: 2 },
            { anchor: [0, 90], leftDirection: [0, 90], rightDirection: [0, 90], pointType: 2 }
          ] },
          { typename: 'PathItem', stroked: true, strokeColor: { typename: 'RGBColor', red: 240, green: 240, blue: 240 } }
        ]
      }
    }
  }, [0, 100, 100, 0]);
  assert.strictEqual(sampledPattern.swatchExtracted, true);
  assert(sampledPattern.tileGeometry.length > 0, 'sampled pattern should preserve swatch tile geometry');
  assert.strictEqual(JSON.stringify(sampledPattern.foreground), JSON.stringify({ r: 12, g: 34, b: 56, a: 255 }));
  assert.strictEqual(JSON.stringify(sampledPattern.background), JSON.stringify({ r: 240, g: 240, b: 240, a: 255 }));
  const sampledPatternEl = { id: 'sampled_pattern', type: 'shape', x: 0, y: 0, w: 20, h: 20, depth: 0, gradient: sampledPattern, effects: [], notes: [] };
  assert.strictEqual(plugin.parityStatusForElement(sampledPatternEl), 'exact');
  const sampledExport = plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 20, height: 20 }, elements: [sampledPatternEl] }], { naming: false, codeOnlyStrict: true });
  assert(sampledExport.files['artboard_1.rs'].includes('from_rgba_unmultiplied(12, 34, 56, 255)'), 'sampled pattern foreground should reach Rust code');
  assert(sampledExport.files['artboard_1.rs'].includes('from_rgba_unmultiplied(240, 240, 240, 255)'), 'sampled pattern background should reach Rust code');
  assert(sampledExport.files['artboard_1.rs'].includes('PatternTileShape'), 'sampled pattern geometry should reach Rust code');

  const truncatedPatternEl = {
    id: 'truncated_pattern',
    type: 'shape',
    x: 0,
    y: 0,
    w: 20,
    h: 20,
    depth: 0,
    gradient: { ...sampledPattern, tileGeometryTruncated: true },
    effects: [],
    notes: []
  };
  assert.strictEqual(plugin.parityStatusForElement(truncatedPatternEl, { codeOnlyStrict: true }), 'unsupported', 'strict pattern export should fail when tile geometry was truncated');
  assert.strictEqual(plugin.parityStatusForElement(truncatedPatternEl, { codeOnlyStrict: false }), 'approximate', 'non-strict pattern export may remain approximate when tile geometry was truncated');

  const manyShapePattern = sandbox.getGradientFromColor({
    typename: 'PatternColor',
    rotation: 0,
    scaleFactor: [100, 100],
    pattern: {
      name: 'Many Shapes',
      patternItem: {
        pageItems: Array.from({ length: 97 }, (_, index) => ({
          typename: 'PathItem',
          closed: true,
          filled: true,
          fillColor: { typename: 'RGBColor', red: 20 + (index % 2), green: 40, blue: 60 },
          pathPoints: [
            { anchor: [index, 100], leftDirection: [index, 100], rightDirection: [index, 100], pointType: 2 },
            { anchor: [index + 1, 100], leftDirection: [index + 1, 100], rightDirection: [index + 1, 100], pointType: 2 },
            { anchor: [index + 1, 99], leftDirection: [index + 1, 99], rightDirection: [index + 1, 99], pointType: 2 },
            { anchor: [index, 99], leftDirection: [index, 99], rightDirection: [index, 99], pointType: 2 }
          ]
        }))
      }
    }
  }, [0, 100, 100, 0]);
  assert.strictEqual(manyShapePattern.tileGeometry.length, 97, 'pattern geometry should not truncate above 96 shapes');
  const manyShapePatternEl = { id: 'many_shape_pattern', type: 'shape', x: 0, y: 0, w: 20, h: 20, depth: 0, gradient: manyShapePattern, effects: [], notes: [] };
  assert.strictEqual(plugin.parityStatusForElement(manyShapePatternEl), 'exact', 'strict pattern export should stay exact when geometry exceeds 96 shapes');
  assert.doesNotThrow(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 20, height: 20 }, elements: [manyShapePatternEl] }], { naming: false, codeOnlyStrict: true });
  });

  const gradientStrokeEl = {
    id: 'gradient_stroke',
    type: 'shape',
    x: 0, y: 0, w: 20, h: 20, depth: 0,
    stroke: { width: 3, gradient: { type: 'linear', angle: 0, stops: [{ position: 0, color: { r: 0, g: 0, b: 0, a: 255 } }, { position: 1, color: { r: 255, g: 255, b: 255, a: 255 } }] } },
    effects: [], notes: []
  };
  assert.strictEqual(plugin.parityStatusForElement(gradientStrokeEl), 'exact', 'deterministic gradient strokes should be exact');

  assert.strictEqual(sandbox.detectThirdPartyEffects({ typename: 'PathItem', graphicStyle: { name: 'Poster Preset' } }).some(effect => effect.type === 'liveEffect'), false);
  assert.strictEqual(plugin.normalizeStrokeAlignment('StrokeAlignment.INSIDE'), 'inside');
  assert.strictEqual(plugin.normalizeStrokeAlignment('outsideAlignment'), 'outside');

  const clipWithText = {
    id: 'clip_text', type: 'group', clipMask: true,
    x: 0, y: 0, w: 100, h: 100, depth: 0,
    children: [{ id: 'clip_text_child', type: 'text', text: 'Hello', textStyle: { size: 14 }, effects: [], notes: [] }],
    effects: [], notes: []
  };
  assert.strictEqual(plugin.parityStatusForElement(clipWithText, { codeOnlyStrict: true }), 'unsupported', 'clip groups with text children should be rejected when the clip path is not rectangular');

  const clipWithRaster = {
    id: 'clip_raster', type: 'group', clipMask: true,
    x: 0, y: 0, w: 100, h: 100, depth: 0,
    children: [
      { id: 'clip_raster_child', type: 'image', x: 0, y: 0, w: 100, h: 100, depth: 1 },
      { id: 'clip_raster_caption', type: 'text', text: 'Caption', textStyle: { size: 14 }, effects: [], notes: [] }
    ],
    effects: [], notes: []
  };
  assert.strictEqual(plugin.parityStatusForElement(clipWithRaster, { codeOnlyStrict: true }), 'unsupported', 'clip groups with unvectorized raster children must remain unsupported in strict export');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [clipWithRaster] }], { naming: false, codeOnlyStrict: true });
  }, /mixed clipping groups containing text are not parity-safe yet|vectorize raster before clip as a preflight requirement|vector tracing/);

  const vectorizedClip = {
    id: 'clip_raster_vectorized', type: 'group', clipMask: true,
    x: 0, y: 0, w: 100, h: 100, depth: 0,
    children: [
      { id: 'clip_raster_vectorized_child', type: 'group', rasterVectorized: true, children: [{ id: 'clip_raster_vectorized_path', type: 'shape', x: 0, y: 0, w: 100, h: 100, depth: 1 }], effects: [], notes: [] },
      { id: 'clip_raster_vectorized_caption', type: 'text', text: 'Caption', textStyle: { size: 14 }, effects: [], notes: [] }
    ],
    effects: [], notes: []
  };
  assert.strictEqual(plugin.parityStatusForElement(vectorizedClip, { codeOnlyStrict: true }), 'unsupported', 'Vectorized raster children plus text remain unsupported until text clipping has an exact non-rect mask path');
}

function testSymbolDefinitionExpansionExtraction() {
  assert.strictEqual(typeof sandbox.extractElements, 'function');
  const symbolItem = {
    typename: 'SymbolItem',
    name: 'logo_symbol',
    geometricBounds: [20, 80, 60, 40],
    visibleBounds: [20, 80, 60, 40],
    opacity: 100,
    symbol: {
      name: 'Logo',
      definition: {
        typename: 'GroupItem',
        pageItems: [{
          typename: 'PathItem',
          name: 'mark',
          geometricBounds: [0, 10, 10, 0],
          visibleBounds: [0, 10, 10, 0],
          filled: true,
          fillColor: { typename: 'RGBColor', red: 12, green: 34, blue: 56 },
          stroked: false,
          closed: true,
          pathPoints: [
            { anchor: [0, 10], leftDirection: [0, 10], rightDirection: [0, 10], pointType: 'corner' },
            { anchor: [10, 10], leftDirection: [10, 10], rightDirection: [10, 10], pointType: 'corner' },
            { anchor: [10, 0], leftDirection: [10, 0], rightDirection: [10, 0], pointType: 'corner' },
            { anchor: [0, 0], leftDirection: [0, 0], rightDirection: [0, 0], pointType: 'corner' }
          ]
        }]
      }
    }
  };
  const elements = sandbox.extractElements([symbolItem], [0, 100, 100, 0]);
  assert.strictEqual(elements.length, 1);
  assert.strictEqual(elements[0].type, 'symbol');
  assert.strictEqual(elements[0].symbolName, 'Logo');
  assert.strictEqual(elements[0].children.length, 1);
  assert.strictEqual(elements[0].symbolExpanded, true);
  assert(elements[0].notes.some(note => note.includes('symbol definition expanded')));
  const child = elements[0].children[0];
  assert.strictEqual(child.id, 'logo_symbol_mark');
  assert.strictEqual(child.x, 20);
  assert.strictEqual(child.y, 20);
  assert.strictEqual(child.w, 40);
  assert.strictEqual(child.h, 40);
  assert.strictEqual(JSON.stringify(child.fill), JSON.stringify({ r: 12, g: 34, b: 56, a: 255 }));
  assert.strictEqual(JSON.stringify(child.pathPoints[0].anchor), JSON.stringify([20, 20]));
  assert.strictEqual(plugin.parityStatusForElement(elements[0]), 'approximate');
  assert.doesNotThrow(() => {
    const result = plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements }], { naming: false, codeOnlyStrict: true });
    assert(result.files['artboard_1.rs'].includes('Symbol instance: "Logo"'));
    assert(result.files['artboard_1.rs'].includes('logo_symbol_mark'));
  });

  const rotatedSymbolItem = {
    typename: 'SymbolItem',
    name: 'rotated_symbol',
    rotation: 30,
    geometricBounds: [10, 90, 30, 70],
    visibleBounds: [10, 90, 30, 70],
    symbol: {
      name: 'RotatedLogo',
      definition: {
        typename: 'GroupItem',
        pageItems: [{
          typename: 'PathItem',
          name: 'rect_child',
          closed: true,
          geometricBounds: [0, 10, 10, 0],
          visibleBounds: [0, 10, 10, 0],
          filled: true,
          fillColor: { typename: 'RGBColor', red: 1, green: 2, blue: 3 },
          stroked: false
        }]
      }
    }
  };
  const rotated = sandbox.extractElements([rotatedSymbolItem], [0, 100, 100, 0])[0];
  assert.strictEqual(rotated.rotation, 0, 'parent symbol rotation should be baked into children for sidecar honesty');
  assert.strictEqual(rotated.children[0].rotation, 30, 'non-path children preserve baked instance rotation');
}

function testHostProgrammaticExpansionFallback() {
  const tempDoc = {
    artboards: [{ artboardRect: [0, 100, 100, 0] }],
    layers: [{ pageItems: [] }],
    selection: [],
    activate() { this.activated = true; },
    close() { this.closed = true; },
  };
  Object.defineProperty(tempDoc, 'pageItems', {
    enumerable: true,
    get() { return this.layers[0].pageItems; }
  });

  const expandedPath = {
    typename: 'PathItem',
    name: 'logo_symbol_mark',
    geometricBounds: [20, 80, 60, 40],
    visibleBounds: [20, 80, 60, 40],
    filled: true,
    fillColor: { typename: 'RGBColor', red: 12, green: 34, blue: 56 },
    stroked: false,
    closed: true,
    pathPoints: [
      { anchor: [20, 80], leftDirection: [20, 80], rightDirection: [20, 80], pointType: 'corner' },
      { anchor: [60, 80], leftDirection: [60, 80], rightDirection: [60, 80], pointType: 'corner' },
      { anchor: [60, 40], leftDirection: [60, 40], rightDirection: [60, 40], pointType: 'corner' },
      { anchor: [20, 40], leftDirection: [20, 40], rightDirection: [20, 40], pointType: 'corner' }
    ]
  };
  const expandedGroup = {
    typename: 'GroupItem',
    name: 'expanded_symbol',
    geometricBounds: [20, 80, 60, 40],
    visibleBounds: [20, 80, 60, 40],
    pageItems: []
  };
  const symbolItem = {
    typename: 'SymbolItem',
    name: 'logo_symbol',
    geometricBounds: [20, 80, 60, 40],
    visibleBounds: [20, 80, 60, 40],
    opacity: 100,
    parent: { typename: 'Layer' },
    symbol: { name: 'Logo', definition: { typename: 'GroupItem', pageItems: [] } },
    duplicate(target) {
      target.pageItems.push(expandedGroup);
      expandedGroup.parent = target;
      return expandedGroup;
    }
  };
  const app = {
    documents: { length: 1, add() { return tempDoc; } },
    activeDocument: { artboards: [{ name: 'Artboard 1', artboardRect: [0, 100, 100, 0] }], pageItems: [symbolItem] },
    commands: [],
    executeMenuCommand(cmd) {
      this.commands.push(cmd);
      if (cmd === 'expandStyle') expandedGroup.pageItems = [expandedPath];
    },
    selection: null,
  };
  const hostSandbox = createHostSandbox({ app });
  const elements = JSON.parse(hostSandbox.extractArtboardDataJSON(JSON.stringify([0])))[0].elements;
  assert.strictEqual(elements.length, 1);
  assert.strictEqual(elements[0].type, 'symbol');
  assert.strictEqual(elements[0].symbolExpanded, true);
  assert(elements[0].children.length > 0);
  assert.strictEqual(elements[0].children[0].type, 'group');
  assert.strictEqual(elements[0].children[0].children.length, 1);
  assert(elements[0].notes.some(note => note.includes('duplicate + Expand Appearance fallback')));
  assert.deepStrictEqual(app.commands, ['expandStyle']);
  assert.doesNotThrow(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements }], { naming: false, codeOnlyStrict: true });
  });

  const failingHostSandbox = createHostSandbox({ app: { documents: { length: 1 }, activeDocument: { artboards: [{ name: 'Artboard 1', artboardRect: [0, 100, 100, 0] }], pageItems: [{
    typename: 'SymbolItem',
    name: 'broken_symbol',
    geometricBounds: [20, 80, 60, 40],
    visibleBounds: [20, 80, 60, 40],
    parent: { typename: 'Layer' },
    symbol: { name: 'Broken', definition: { typename: 'GroupItem', pageItems: [] } },
    duplicate() { throw new Error('duplicate should not be called when menu commands are unavailable'); }
  }] } } });
  const failingExtraction = JSON.parse(failingHostSandbox.extractArtboardDataJSON(JSON.stringify([0])))[0];
  const failingElements = failingExtraction.elements;
  const failingDiagnostics = failingExtraction.hostDiagnostics;
  assert.strictEqual(failingElements[0].children.length, 0);
  assert(failingElements[0].notes.some(note => note.includes('duplicate + Expand Appearance fallback unavailable')));
  assert(failingDiagnostics.some(note => note.note.includes('Expand Appearance fallback unavailable')));
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: failingElements }], { naming: false, codeOnlyStrict: true });
  }, /duplicate \+ Expand Appearance fallback unavailable/);
}

function testHostTextShapingContractExtraction() {
  const tempDoc = {
    artboards: [{ artboardRect: [0, 120, 200, 0] }],
    layers: [{ pageItems: [] }],
    selection: [],
    activate() { this.activated = true; },
    close() { this.closed = true; },
  };
  Object.defineProperty(tempDoc, 'pageItems', {
    enumerable: true,
    get() { return this.layers[0].pageItems; }
  });

  const shapedTextFrame = {
    typename: 'TextFrame',
    name: 'headline',
    contents: 'office',
    geometricBounds: [20, 100, 180, 60],
    visibleBounds: [20, 100, 180, 60],
    parent: { typename: 'Layer' },
    textRange: {
      characterAttributes: {
        size: 24,
        textFont: { name: 'Test Serif Bold' },
        stylisticAlternates: true,
      },
      paragraphAttributes: { justification: sandbox.Justification.LEFT }
    },
    textRanges: [{ contents: 'office', characterAttributes: { size: 24, textFont: { name: 'Test Serif Bold' }, stylisticAlternates: true } }],
    duplicate(target) {
      const duplicate = {
        typename: 'TextFrame',
        name: 'headline_copy',
        contents: this.contents,
        geometricBounds: this.geometricBounds,
        visibleBounds: this.visibleBounds,
        textRange: this.textRange,
        textRanges: this.textRanges,
        parent: target,
      };
      target.pageItems.push(duplicate);
      return duplicate;
    }
  };

  const app = {
    documents: { length: 1, add() { return tempDoc; } },
    activeDocument: { artboards: [{ name: 'Artboard 1', artboardRect: [0, 120, 200, 0] }], pageItems: [shapedTextFrame] },
    executeMenuCommand(cmd) {
      this.commands = (this.commands || []).concat(cmd);
    },
    selection: null,
  };

  const hostSandbox = createHostSandbox({ app });
  const extraction = JSON.parse(hostSandbox.extractArtboardDataJSON(JSON.stringify([0])))[0];
  assert.strictEqual(extraction.elements.length, 1);
  const textElement = extraction.elements[0];
  assert.strictEqual(textElement.shapedGlyphs, null, 'host must not synthesize shapedGlyphs when no real glyph extraction is available');
  assert.strictEqual(textElement.outlinedGlyphs, null, 'host must not synthesize outlinedGlyphs when no real glyph extraction is available');
  assert.strictEqual(plugin.parityStatusForElement(textElement, { codeOnlyStrict: true }), 'unsupported');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 200, height: 120 }, elements: [textElement] }], { naming: false, codeOnlyStrict: true, sidecar: true });
  }, /advanced OpenType shaping|unsupported/i);
}

function testHostTextShapingOutlinedGlyphExtraction() {
  const tempDoc = {
    artboards: [{ artboardRect: [0, 120, 220, 0] }],
    layers: [{ pageItems: [] }],
    selection: [],
    activate() { this.activated = true; },
    close() { this.closed = true; },
  };
  Object.defineProperty(tempDoc, 'pageItems', {
    enumerable: true,
    get() { return this.layers[0].pageItems; }
  });

  const outlinedGlyphOne = {
    typename: 'PathItem',
    name: 'headline_outline_a',
    geometricBounds: [24, 96, 48, 64],
    visibleBounds: [24, 96, 48, 64],
    closed: true,
    pathPoints: [
      { anchor: [24, 96], leftDirection: [24, 96], rightDirection: [24, 96], pointType: 'corner' },
      { anchor: [48, 96], leftDirection: [48, 96], rightDirection: [48, 96], pointType: 'corner' },
      { anchor: [48, 64], leftDirection: [48, 64], rightDirection: [48, 64], pointType: 'corner' },
      { anchor: [24, 64], leftDirection: [24, 64], rightDirection: [24, 64], pointType: 'corner' }
    ]
  };
  const outlinedGlyphTwo = {
    typename: 'PathItem',
    name: 'headline_outline_b',
    geometricBounds: [54, 96, 82, 64],
    visibleBounds: [54, 96, 82, 64],
    closed: true,
    pathPoints: [
      { anchor: [54, 96], leftDirection: [54, 96], rightDirection: [54, 96], pointType: 'corner' },
      { anchor: [82, 96], leftDirection: [82, 96], rightDirection: [82, 96], pointType: 'corner' },
      { anchor: [82, 64], leftDirection: [82, 64], rightDirection: [82, 64], pointType: 'corner' },
      { anchor: [54, 64], leftDirection: [54, 64], rightDirection: [54, 64], pointType: 'corner' }
    ]
  };
  const outlinedGroup = {
    typename: 'GroupItem',
    name: 'headline_outlines',
    geometricBounds: [24, 96, 82, 64],
    visibleBounds: [24, 96, 82, 64],
    pageItems: [outlinedGlyphOne, outlinedGlyphTwo]
  };
  const shapedTextFrame = {
    typename: 'TextFrame',
    name: 'headline',
    contents: 'AV',
    geometricBounds: [20, 100, 90, 60],
    visibleBounds: [20, 100, 90, 60],
    parent: { typename: 'Layer' },
    textRange: {
      characterAttributes: {
        size: 24,
        textFont: { name: 'Test Serif Bold' },
        stylisticAlternates: true,
      },
      paragraphAttributes: { justification: sandbox.Justification.LEFT }
    },
    textRanges: [{ contents: 'AV', characterAttributes: { size: 24, textFont: { name: 'Test Serif Bold' }, stylisticAlternates: true } }],
    duplicate(target) {
      const duplicate = {
        typename: 'TextFrame',
        name: 'headline_copy',
        contents: this.contents,
        geometricBounds: this.geometricBounds,
        visibleBounds: this.visibleBounds,
        textRange: this.textRange,
        textRanges: this.textRanges,
        parent: target,
      };
      target.pageItems.push(duplicate);
      return duplicate;
    }
  };

  const app = {
    documents: { length: 1, add() { return tempDoc; } },
    activeDocument: { artboards: [{ name: 'Artboard 1', artboardRect: [0, 120, 220, 0] }], pageItems: [shapedTextFrame] },
    executeMenuCommand(cmd) {
      this.commands = (this.commands || []).concat(cmd);
      if (cmd === 'createOutlines') {
        tempDoc.layers[0].pageItems = [outlinedGroup];
        tempDoc.selection = [outlinedGroup];
        this.selection = [outlinedGroup];
      }
    },
    selection: null,
  };

  const hostSandbox = createHostSandbox({ app });
  const extraction = JSON.parse(hostSandbox.extractArtboardDataJSON(JSON.stringify([0])))[0];
  assert.strictEqual(extraction.elements.length, 1);
  const textElement = extraction.elements[0];
  assert(textElement.shapedGlyphs == null, 'outlined export should not synthesize shapedGlyphs');
  assert(Array.isArray(textElement.outlinedGlyphs));
  assert.strictEqual(textElement.outlinedGlyphs.length, 2);
  assert.strictEqual(textElement.outlinedGlyphs[0].contours[0].points.length, 4);
  assert.strictEqual(textElement.outlinedGlyphs[0].contoursAbsolute, true, 'host-extracted outline contours are artboard-relative');
  assert(textElement.outlinedGlyphs[0].advanceX > 0);
  assert.strictEqual(plugin.parityStatusForElement(textElement, { codeOnlyStrict: true }), 'exact');

  const exported = plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 220, height: 120 }, elements: [textElement] }], { naming: false, codeOnlyStrict: true, sidecar: true });
  assert(exported.files['artboard_1.rs'].includes('render_shaped_glyph_run(&painter'));
  assert(exported.files['artboard_1.rs'].includes('render_shaped_glyph_run(&painter, origin, &shaped, &spec);'), 'absolute outlined glyph contours should render from artboard origin');
  assert(exported.files['artboard_1.rs'].includes('contours_are_absolute: true'));
  const sidecar = JSON.parse(exported.files['artboard_1.json']);
  assert.strictEqual(sidecar.elements[0].outlinedGlyphs[0].contours[0].points.length, 4);
  assert.strictEqual(sidecar.elements[0].outlinedGlyphs[0].contours_are_absolute, true);
  assert.strictEqual(sidecar.elements[0].outlinedGlyphs[0].contoursAbsolute, undefined);
}

function testHostAllCapsDoesNotForceOutlinedShapingContract() {
  const allCapsTextFrame = {
    typename: 'TextFrame',
    name: 'all_caps_label',
    contents: 'Mode',
    geometricBounds: [20, 100, 90, 60],
    visibleBounds: [20, 100, 90, 60],
    parent: { typename: 'Layer' },
    textRange: {
      characterAttributes: {
        size: 18,
        textFont: { name: 'Test Sans' },
        allCaps: true,
      },
      paragraphAttributes: { justification: sandbox.Justification.LEFT }
    },
    textRanges: [{ contents: 'Mode', characterAttributes: { size: 18, textFont: { name: 'Test Sans' }, allCaps: true } }],
  };
  const app = {
    documents: { length: 1 },
    activeDocument: { artboards: [{ name: 'Artboard 1', artboardRect: [0, 120, 220, 0] }], pageItems: [allCapsTextFrame] },
    commands: [],
    executeMenuCommand(cmd) { this.commands.push(cmd); }
  };

  const hostSandbox = createHostSandbox({ app });
  const extraction = JSON.parse(hostSandbox.extractArtboardDataJSON(JSON.stringify([0])))[0];
  assert.strictEqual(extraction.elements.length, 1);
  const textElement = extraction.elements[0];
  assert.strictEqual(textElement.textTransform, 'uppercase');
  assert.strictEqual(textElement.shapedGlyphs, null);
  assert.strictEqual(textElement.outlinedGlyphs, null);
  assert.deepStrictEqual(app.commands, [], 'plain all-caps should not trigger createOutlines extraction');
  assert.strictEqual(plugin.parityStatusForElement(textElement, { codeOnlyStrict: true }), 'exact');
}

function testHostCanonicalTypographyFieldsExtraction() {
  const richTextFrame = {
    typename: 'TextFrame',
    name: 'rich_text',
    contents: 'Rich',
    geometricBounds: [20, 100, 90, 60],
    visibleBounds: [20, 100, 90, 60],
    parent: { typename: 'Layer' },
    textRange: {
      characterAttributes: {
        size: 18,
        textFont: { name: 'Test Sans' },
        ligatures: false,
        baselineShift: 3,
        horizontalScale: 120,
        verticalScale: 90,
        tracking: 100,
        leading: 36,
        underline: true,
        allCaps: true,
      },
      paragraphAttributes: { justification: 'FULLJUSTIFYLASTLINECENTER' }
    },
    textRanges: [{ contents: 'Rich', characterAttributes: { size: 18, textFont: { name: 'Test Sans' }, ligatures: false, baselineShift: 3, horizontalScale: 120, verticalScale: 90, tracking: 100, leading: 36, underline: true, allCaps: true } }],
  };
  const app = {
    documents: { length: 1 },
    activeDocument: { artboards: [{ name: 'Artboard 1', artboardRect: [0, 120, 220, 0] }], pageItems: [richTextFrame] },
    executeMenuCommand() {}
  };

  const hostSandbox = createHostSandbox({ app });
  const extraction = JSON.parse(hostSandbox.extractArtboardDataJSON(JSON.stringify([0])))[0];
  const textElement = extraction.elements[0];
  assert.strictEqual(textElement.textAlign, 'justified_last_line_center');
  assert.strictEqual(textElement.textStyle.openTypeFeatures.ligatures, false);
  assert.strictEqual(textElement.textStyle.baselineShift, 3);
  assert.strictEqual(textElement.textStyle.horizontalScale, 1.2);
  assert.strictEqual(textElement.textStyle.verticalScale, 0.9);
  assert.strictEqual(textElement.textStyle.letterSpacing, 1.8);
  assert.strictEqual(textElement.textStyle.lineHeight, 2);
  assert.strictEqual(textElement.textStyle.textDecoration, 'underline');
  assert.strictEqual(textElement.textStyle.textTransform, 'uppercase');

  const sidecar = JSON.parse(plugin.generateSidecar({ name: 'Artboard 1', width: 220, height: 120 }, [textElement], new Map(), { codeOnlyStrict: false }));
  assert.strictEqual(sidecar.elements[0].textStyle.openTypeFeatures.ligatures, false);
  assert.strictEqual(sidecar.elements[0].textStyle.baselineShift, 3);
  assert.strictEqual(sidecar.elements[0].textStyle.horizontalScale, 1.2);
  assert.strictEqual(sidecar.elements[0].textStyle.verticalScale, 0.9);
  assert.strictEqual(sidecar.elements[0].textStyle.letterSpacing, 1.8);
  assert.strictEqual(sidecar.elements[0].textStyle.lineHeight, 2);
  assert.strictEqual(sidecar.elements[0].textStyle.textDecoration, 'underline');
  assert.strictEqual(sidecar.elements[0].textStyle.textTransform, 'uppercase');
}

function testParityStatusMarksUnsupportedSubset() {
  const colorMap = new Map();
  const sidecar = JSON.parse(plugin.generateSidecar(
    { name: 'Artboard 1', width: 100, height: 100 },
    [
      { id: 'embedded', type: 'image', x: 0, y: 0, w: 10, h: 10, depth: 0, embeddedRaster: true, effects: [], notes: [] },
      { id: 'smallcaps', type: 'text', x: 0, y: 20, w: 50, h: 10, depth: 1, text: 'Hi', textAlign: 'justified', textTransform: 'small_caps', effects: [], notes: [] },
      { id: 'plugin_item', type: 'plugin', x: 10, y: 30, w: 40, h: 20, depth: 0, fill: { r: 12, g: 34, b: 56 }, effects: [], notes: [] }
    ],
    colorMap,
    { codeOnlyStrict: false }
  ));
  assert.strictEqual(sidecar.artboard.parityStatus, 'unsupported');
  assert.strictEqual(sidecar.elements[0].parityStatus, 'unsupported');
  assert(sidecar.elements[0].parityReasons.some(reason => reason.includes('extractable pixels') && reason.includes('will not be exported as raster')));
  assert.strictEqual(sidecar.elements[1].parityStatus, 'approximate');
  assert(sidecar.elements[1].parityReasons.some(reason => reason.includes('advanced OpenType shaping')));
  assert.strictEqual(sidecar.elements[2].parityStatus, 'unsupported');
  assert(sidecar.elements[2].parityReasons.some(reason => reason.includes('plugin item')));

  const exported = plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [
    { id: 'smallcaps', type: 'text', x: 0, y: 20, w: 50, h: 10, depth: 1, text: 'Hi', textAlign: 'justified', textTransform: 'small_caps', effects: [], notes: [] }
  ] }], { naming: false, codeOnlyStrict: false });
  const code = exported.files['artboard_1.rs'];
  assert(code.includes('TextBlockAlign::Justified'), 'Justified text should emit TextBlockAlign::Justified');
  assert(code.includes('TextTransform::SmallCaps'), 'Small caps should emit TextTransform::SmallCaps');

  const warnings = plugin.collectWarnings(sidecar.elements, {});
  assert(warnings.some(w => w.parityStatus === 'unsupported'));
  const opaqueWarnings = plugin.collectWarnings([
    { id: 'chart_item', type: 'shape', isChart: true, x: 0, y: 0, w: 10, h: 10, effects: [], notes: [] },
    { id: 'mesh_item', type: 'shape', isGradientMesh: true, x: 0, y: 0, w: 10, h: 10, effects: [], notes: [] }
  ], {});
  assert.strictEqual(plugin.parityStatusForElement({ id: 'chart_item', type: 'shape', isChart: true }), 'unsupported');
  assert(plugin.parityFindingsForElement({ id: 'rot_img', type: 'image', imagePath: 'rot.png', rotation: 15 })
    .some(finding => finding.reason.includes('matrix-aware vector tracing')));
  assert(plugin.parityFindingsForElement({ id: 'rot_img_scaled', type: 'image', imagePath: 'rot.png', rotation: 15, rasterScaleX: 1.2, rasterScaleY: 0.8 })
    .some(finding => finding.reason.includes('transform-aware vector tracing')));
  assert(plugin.parityFindingsForElement({ id: 'fx_img', type: 'image', imagePath: 'fx.png', effects: [{ type: 'effect_from_tag' }] })
    .some(finding => finding.reason.includes('effect-aware vector tracing')));
  assert.strictEqual(plugin.parityStatusForElement({ id: 'traced_linked_raster', type: 'group', rasterVectorized: true }), 'approximate');
  assert(opaqueWarnings.some(w => w.note.includes('Chart/graph') && w.note.includes('preserved bounds/metadata')));
  assert(opaqueWarnings.some(w => w.note.includes('Gradient mesh') && w.note.includes('editable mesh patches')));

  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [
      { id: 'plugin_item', type: 'plugin', x: 10, y: 30, w: 40, h: 20, depth: 0, fill: { r: 12, g: 34, b: 56 }, effects: [], notes: [] }
    ] }], { naming: false });
  }, /Cannot export code-only Rust: plugin_item: \[unsupported\] Illustrator plugin item exposes only bounds\/metadata/);

  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [
      { id: 'chart_item', type: 'shape', isChart: true, x: 10, y: 60, w: 40, h: 20, depth: 1, fill: { r: 12, g: 34, b: 56 }, effects: [], notes: [] }
    ] }], { naming: false });
  }, /Cannot export code-only Rust: chart_item: \[unsupported\] Illustrator chart\/graph object exposes only bounds\/metadata/);
}

function testParserAndGradientStrokeParityStatus() {
  const sidecar = JSON.parse(plugin.generateSidecar(
    { name: 'Artboard 1', width: 100, height: 100 },
    [{
      id: 'stroke_gradient', type: 'shape', x: 0, y: 0, w: 20, h: 20, depth: 0,
      stroke: { r: 0, g: 0, b: 0, width: 2, gradient: { type: 'linear', stops: [] } },
      effects: [], notes: []
    }],
    new Map(),
    { parserDiagnostics: [{ id: 'ai-parser', note: 'Bundled ai-parser not found' }] }
  ));
  assert.strictEqual(sidecar.artboard.parityStatus, 'unsupported');
  assert(sidecar.artboard.parityReasons.some(reason => reason.includes('[unsupported] ai-parser enrichment unavailable')));
  assert.strictEqual(plugin.parityStatusForElement({
    id: 'stroke_gradient', type: 'shape', x: 0, y: 0, w: 20, h: 20, depth: 0,
    stroke: { r: 0, g: 0, b: 0, width: 2, gradient: { type: 'linear', stops: [] } },
    effects: [], notes: []
  }), 'exact');
  assert.strictEqual(sidecar.elements[0].parityStatus, 'unsupported');
  assert(sidecar.elements[0].strokeGradient, 'Sidecar should expose stroke gradient metadata');

  const mixedSolidDashAndGradient = {
    id: 'mixed_gradient_strokes', type: 'shape', x: 0, y: 0, w: 20, h: 20, depth: 0,
    appearanceStrokes: [
      { width: 2, color: { r: 0, g: 0, b: 0 }, dash: [2, 2] },
      { width: 3, gradient: { type: 'linear', stops: [{ position: 0, color: '#ff0000' }, { position: 1, color: '#0000ff' }] } }
    ],
    effects: [], notes: []
  };
  assert.strictEqual(plugin.parityStatusForElement(mixedSolidDashAndGradient), 'exact');
  assert.doesNotThrow(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [mixedSolidDashAndGradient] }], { naming: false, codeOnlyStrict: true });
  });

  const mixedSolidDashAndPattern = {
    id: 'mixed_pattern_strokes', type: 'shape', x: 0, y: 0, w: 20, h: 20, depth: 0,
    appearanceStrokes: [
      { width: 2, color: { r: 0, g: 0, b: 0 }, dash: [2, 2] },
      { width: 3, pattern: { patternName: 'dots' } }
    ],
    effects: [], notes: []
  };
  assert.strictEqual(plugin.parityStatusForElement(mixedSolidDashAndPattern), 'unsupported');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [mixedSolidDashAndPattern] }], { naming: false, codeOnlyStrict: true });
  }, /PatternColor strokes use procedural placeholder colors/);

  assert.strictEqual(plugin.parityStatusForElement({
    id: 'dashed_gradient_same_layer', type: 'shape', x: 0, y: 0, w: 20, h: 20, depth: 0,
    appearanceStrokes: [{ width: 3, dash: [2, 2], gradient: { type: 'linear', stops: [{ position: 0, color: '#ff0000' }, { position: 1, color: '#0000ff' }] } }],
    effects: [], notes: []
  }), 'exact');

  // Dashed pattern stroke (same layer) is blocked in strict mode until swatch artwork is sampled.
  assert.strictEqual(plugin.parityStatusForElement({
    id: 'dashed_pattern_same_layer', type: 'shape', x: 0, y: 0, w: 20, h: 20, depth: 0,
    appearanceStrokes: [{ width: 3, dash: [2, 2], pattern: { patternName: 'dots' } }],
    effects: [], notes: []
  }), 'unsupported');

  // Strict export should NOT throw for dashed gradient strokes.
  assert.doesNotThrow(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [
      { id: 'dashed_gradient_same_layer', type: 'shape', x: 0, y: 0, w: 20, h: 20, depth: 0,
        appearanceStrokes: [{ width: 3, dash: [2, 2], gradient: { type: 'linear', stops: [{ position: 0, color: '#ff0000' }, { position: 1, color: '#0000ff' }] } }],
        effects: [], notes: []
      }
    ] }], { naming: false, codeOnlyStrict: true });
  });

  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [
      { id: 'dashed_pattern_same_layer', type: 'shape', x: 0, y: 0, w: 20, h: 20, depth: 0,
        appearanceStrokes: [{ width: 3, dash: [2, 2], pattern: { patternName: 'dots' } }],
        effects: [], notes: []
      }
    ] }], { naming: false, codeOnlyStrict: true });
  }, /PatternColor strokes use procedural placeholder colors/);

  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [] }], {
      parserDiagnostics: [{ id: 'ai-parser', note: 'Bundled ai-parser not found' }]
    });
  }, /ai-parser: \[unsupported\] ai-parser enrichment unavailable/);

  const permissive = plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [] }], {
    codeOnlyStrict: false,
    parserDiagnostics: [{ id: 'ai-parser', note: 'Bundled ai-parser not found' }]
  });
  assert(permissive.files['artboard_1.rs'], 'Non-strict export may continue with an ai-parser diagnostic');
}

function testMultiAppearanceProbeBlocksFlattening() {
  const probe = plugin.appearanceProbeFromMetadataText('appearanceFillCount=3 appearanceStrokeCount=2 [0 0 0 1] XA [1 1 1 1] xa', 'XMPString');
  assert.strictEqual(probe.fillCount, 3);
  assert.strictEqual(probe.strokeCount, 2);

  const flattened = {
    id: 'native_multi_appearance', type: 'shape', x: 0, y: 0, w: 20, h: 20, depth: 0,
    fill: { r: 255, g: 0, b: 0 },
    stroke: { r: 0, g: 0, b: 0, width: 1 },
    appearanceProbe: { fillCount: 2, strokeCount: 2, source: 'XMPString' },
    effects: [], notes: []
  };
  const findings = plugin.parityFindingsForElement(flattened);
  assert(findings.some(finding => finding.status === 'unsupported' && finding.reason.includes('native multi-fill/multi-stroke')));
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [flattened] }], { naming: false, codeOnlyStrict: true });
  }, /native multi-fill\/multi-stroke Appearance panel stack flattened/);

  const permissive = plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [flattened] }], { naming: false, codeOnlyStrict: false, includeSidecar: true });
  const sidecar = JSON.parse(permissive.files['artboard_1.json']);
  assert.strictEqual(sidecar.elements[0].parityStatus, 'approximate');
  assert.strictEqual(sidecar.elements[0].appearanceProbe.fillCount, 2);

  const parserEnriched = {
    ...flattened,
    fill: null,
    stroke: null,
    appearance_fills: [
      { r: 255, g: 0, b: 0, a: 255 },
      { r: 0, g: 255, b: 0, a: 255 }
    ],
    appearance_strokes: [
      { r: 0, g: 0, b: 0, a: 255, width: 1 },
      { r: 0, g: 0, b: 255, a: 255, width: 3 }
    ]
  };
  assert(!plugin.parityFindingsForElement(parserEnriched).some(finding => finding.reason.includes('native multi-fill/multi-stroke')));
  assert.doesNotThrow(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [parserEnriched] }], { naming: false, codeOnlyStrict: true });
  });

  const partialParserMergeMismatch = {
    ...flattened,
    fill: null,
    stroke: null,
    appearanceProbe: { fillCount: 3, strokeCount: 2, source: 'XMPString' },
    appearance_fills: [
      { r: 255, g: 0, b: 0, a: 255 },
      { r: 0, g: 255, b: 0, a: 255 }
    ],
    appearance_strokes: [
      { r: 0, g: 0, b: 0, a: 255, width: 1 }
    ]
  };
  const partialFindings = plugin.parityFindingsForElement(partialParserMergeMismatch);
  assert(partialFindings.some(finding => finding.status === 'unsupported' && finding.reason.includes('native_multi_appearance')));
  assert(partialFindings.some(finding => finding.status === 'unsupported' && finding.reason.includes('recovered 2 fills/1 stroke from XMPString')));
  assert.strictEqual(plugin.parityStatusForElement(partialParserMergeMismatch), 'unsupported');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [partialParserMergeMismatch] }], { naming: false, codeOnlyStrict: true });
  }, /native_multi_appearance: native multi-fill\/multi-stroke Appearance panel stack flattened/);
  assert.strictEqual(plugin.parityStatusForElement(partialParserMergeMismatch, { codeOnlyStrict: false }), 'approximate');
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
  const indexHtml = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
  assert(!hostSource.includes('__eguiHostMaxLogBytes'), 'Host file logging should not be enabled by default');
  assert(!indexHtml.includes('// await appendHostExportLog'), 'Panel should not contain commented-out host logging calls');
  assert(hostSource.includes('center: origin'), 'CEP host gradients should include radial center');
  assert(hostSource.includes('focalPoint: focalPoint'), 'CEP host gradients should include focal point');
  assert(hostSource.includes('getTextAlign(item)'), 'CEP host text extraction should include alignment');
  assert(hostSource.includes('illustratorTrackingToPx'), 'CEP host text extraction should convert tracking to px');
  const writes = [];
  const hostSandbox = {
    Folder: function(path) { this.fsName = path; this.exists = true; this.create = function() { this.exists = true; }; },
    File: function(path) {
      this.fsName = path;
      this.exists = true;
      this.parent = { exists: true };
      this.copy = function() { return true; };
      this.open = function(mode) { this.mode = mode; this.buffer = ''; writes.push(this); return true; };
      this.write = function(content) { this.buffer += String(content); return true; };
      this.writeln = function(content) { this.buffer += String(content) + '\n'; return true; };
      this.close = function() {};
    }
  };
  hostSandbox.Folder.myDocuments = new hostSandbox.Folder('/tmp/Documents');
  hostSandbox.Folder.desktop = new hostSandbox.Folder('/tmp/Desktop');
  hostSandbox.Folder.temp = new hostSandbox.Folder('/tmp');
  hostSandbox.Folder.selectDialog = function() { return new hostSandbox.Folder('/tmp/export'); };

  vm.runInNewContext(hostSource, hostSandbox, { filename: 'host.jsx' });

  assert(!writes.some(w => w.fsName === '/tmp/Documents/egui_expressive_export.log' && w.mode === 'a' && w.buffer.includes('host.jsx loaded')), 'Host log should not be created on load');

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
  hostSandbox.noteHostDiagnostic('embedded raster extraction failed', new Error('/tmp/egui_expressive_raster_trace/raster_1.png'));
  const sanitizedDiags = hostSandbox.consumeHostDiagnostics();
  assert(sanitizedDiags.some(d => d.note.includes('[temporary raster extraction input]')));
  assert(!sanitizedDiags.some(d => d.note.includes('/tmp/egui_expressive_raster_trace') || d.note.includes('raster_1.png')));
  assert(!writes.some(w => w.fsName === '/tmp/Documents/egui_expressive_export.log'), 'Diagnostics should stay in memory without host log file writes');

  assert.strictEqual(hostSandbox.resetHostLogJSON, undefined);
  assert.strictEqual(hostSandbox.appendHostLogJSON, undefined);

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

  const folderResult = JSON.parse(hostSandbox.selectSaveFolderJSON());
  assert.strictEqual(folderResult.success, true);
  assert.strictEqual(folderResult.folder, '/tmp/export');
  const firstChunk = JSON.parse(hostSandbox.writeGeneratedFileChunkJSON(JSON.stringify({
    folder: '/tmp/export', filename: 'chunked.rs', content: 'fn ', mode: 'w'
  })));
  const secondChunk = JSON.parse(hostSandbox.writeGeneratedFileChunkJSON(JSON.stringify({
    folder: '/tmp/export', filename: 'chunked.rs', content: 'main() {}', mode: 'a'
  })));
  assert.strictEqual(firstChunk.success, true);
  assert.strictEqual(secondChunk.success, true);
  const assetCopy = JSON.parse(hostSandbox.copyGeneratedAssetJSON(JSON.stringify({
    folder: '/tmp/export', assetPath: 'assets/123_img.png', sourcePath: '/tmp/img.png'
  })));
  assert.strictEqual(assetCopy.success, true);

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

function testGenerateStateFileDerives() {
  const results = [{
    artboard: { name: "Artboard 1", width: 100, height: 100 },
    elements: [
      { id: "text_1", type: "text", text: "Hello", textStyle: { size: 14 } }
    ]
  }];
  const exported = plugin.exportFromRawData ? plugin.exportFromRawData(results, { naming: false }) : null;
  if (exported) {
    const code = exported.files["state.rs"];
    assert(code.includes("#[derive(Default, Clone)]"), "State struct should derive Default and Clone");
  }
}

function testHostSaveFailureHandling() {
  const hostSource = fs.readFileSync(path.join(__dirname, 'host.jsx'), 'utf8');
  const hostSandbox = {
    Folder: function(path) { this.fsName = path; this.exists = true; this.create = function() { this.exists = true; }; },
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
    appearance_strokes: [{ color: { r: 0, g: 255, b: 0 }, width: 2, opacity: 1, blendMode: "screen", alignment: "outside" }],
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
  assert.strictEqual(sidecar.elements[0].appearanceStrokes[0].alignment, "outside");
  assert.strictEqual(sidecar.elements[0].children[0].text, "Hello");
  assert.strictEqual(sidecar.elements[1].type, "ellipse");

  const stack = sidecar.elements[0].appearanceStack;
  assert(stack.some(e => e.entryType === 'effect' && e.effectType === 'dropShadow'));

  const missingOptionalArrays = JSON.parse(plugin.generateSidecar(ab, [{
    id: "minimal_shape", type: "shape", x: 0, y: 0, w: 1, h: 1, depth: 0,
    fill: { r: 1, g: 2, b: 3 }
  }], colorMap));
  assert.strictEqual(missingOptionalArrays.elements[0].notes, undefined, 'Missing notes array should not crash or emit notes');
  assert.strictEqual(missingOptionalArrays.elements[0].effects, undefined, 'Missing effects array should not crash or emit effects');
}

function testBlendModeUsesSceneBuilder() {
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
    assert(code.includes("with_blend_mode"), "Should emit readable scene blend builder");
    assert(code.includes("scene::render_node"), "Should route blended vector through scene renderer");
    assert(code.includes("egui_expressive::codegen::BlendMode::Multiply"), "Should preserve blend mode in scene primitive");
  }

  const variantExported = plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [{
    id: "el_variant", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0,
    fill: { r: 255, g: 0, b: 0 }, blendMode: "BlendModes.MULTIPLY", effects: [], notes: []
  }] }], { naming: false });
  if (variantExported) {
    assert(variantExported.files["artboard_1.rs"].includes("egui_expressive::codegen::BlendMode::Multiply"), "Should normalize string blend mode variants");
  }

  const ordinalExported = plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [{
    id: "el_ordinal", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0,
    fill: { r: 0, g: 255, b: 0 }, blendMode: 1, effects: [], notes: []
  }] }], { naming: false });
  if (ordinalExported) {
    assert(ordinalExported.files["artboard_1.rs"].includes("egui_expressive::codegen::BlendMode::Multiply"), "Should normalize ordinal blend mode values");
  }

  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [{
      id: "blend_text", type: "text", text: "Hello", x: 0, y: 0, w: 10, h: 10, depth: 0,
      blendMode: "BlendModes.MULTIPLY", effects: [], notes: []
    }] }], { naming: false, codeOnlyStrict: true });
  }, /Cannot export code-only Rust: blend_text: \[unsupported\] non-vector element with blend mode multiply requires scene-routed compositing before strict export/);
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
  const options = { naming: false, codeOnlyStrict: false };
  const exported = plugin.exportFromRawData ? plugin.exportFromRawData(results, options) : null;
  if (exported) {
    const code = exported.files["artboard_1.rs"];
    assert(code.includes("Linked raster/images could not be vectorized and will not be exported as raster"), "Should tell users linked raster/images need tracing");
    assert(!code.includes("paint_image_slot"), "Raster/images should not emit image slots");
    assert.strictEqual(Object.keys(exported.assets).length, 0, "Raster/images should not be copied as export assets");
  }
}

async function testEmbeddedRasterVectorizationUsesExtractedPixels() {
  const embedded = {
    id: "embedded_img",
    type: "image",
    x: 0,
    y: 0,
    w: 10,
    h: 10,
    depth: 0,
    embeddedRaster: true,
    extractedImagePath: "/tmp/egui_expressive_raster_trace/embedded_img.png",
    effects: [],
    notes: ["embedded raster image"]
  };
  assert.strictEqual(plugin.rasterVectorSourcePath(embedded), embedded.extractedImagePath);

  const traced = await plugin.vectorizeRasterElement(embedded, "Artboard 1", async (el, artboardName) => {
    assert.strictEqual(plugin.rasterVectorSourcePath(el), embedded.extractedImagePath);
    assert.strictEqual(artboardName, "Artboard 1");
    return {
      elements: [{
        id: "embedded_img_trace",
        element_type: "path",
        artboard_name: "Artboard 1",
        bounds: [0, 0, 10, 10],
        path_closed: true,
        path_points: [
          { anchor: [0, 0], left_ctrl: [0, 0], right_ctrl: [0, 0] },
          { anchor: [10, 0], left_ctrl: [10, 0], right_ctrl: [10, 0] },
          { anchor: [10, 10], left_ctrl: [10, 10], right_ctrl: [10, 10] },
          { anchor: [0, 10], left_ctrl: [0, 10], right_ctrl: [0, 10] }
        ],
        appearance_fills: [{ r: 240, g: 32, b: 16, a: 255, opacity: 1 }]
      }]
    };
  });

  assert.strictEqual(traced.type, "group");
  assert.strictEqual(traced.originalType, "image");
  assert.strictEqual(traced.rasterVectorized, true);
  assert.strictEqual(traced.rasterSourceOrigin, "embedded");
  assert.strictEqual(traced.embeddedRaster, false);
  assert.strictEqual(traced.imagePath, null);
  assert.strictEqual(traced.extractedImagePath, null);
  assert.strictEqual(traced.children.length, 1);
  assert(traced.notes.some(note => note.includes("embedded raster vectorized")));
  const tracedFindings = plugin.parityFindingsForElement(traced);
  assert(tracedFindings.some(finding => finding.reason.includes("embedded raster traced")));
  assert(!tracedFindings.some(finding => finding.reason.includes("linked raster traced")));

  const exported = plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [traced] }], { naming: false, codeOnlyStrict: true });
  const code = exported.files["artboard_1.rs"];
  assert(code.includes("scene::render_node"), "Embedded raster trace should export vector scene code");
  assert(!code.includes("paint_image_slot"), "Embedded raster traces must not emit image slots");
  assert.strictEqual(Object.keys(exported.assets).length, 0, "Embedded raster temp pixels must not become export assets");

  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [{ ...embedded, extractedImagePath: null }] }], { naming: false, codeOnlyStrict: true });
  }, /Embedded raster\/images need extractable pixels/);

  const emptyTrace = await plugin.vectorizeRasterElement(embedded, "Artboard 1", async () => ({ elements: [] }));
  assert.strictEqual(emptyTrace.type, "image");
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [emptyTrace] }], { naming: false, codeOnlyStrict: true });
  }, /Embedded raster\/images could not be vectorized/);

  const thrownTrace = await plugin.vectorizeRasterElement(embedded, "Artboard 1", async () => {
    throw new Error("trace failed at /tmp/egui_expressive_raster_trace/embedded_img.png");
  });
  const warnings = plugin.collectWarnings([thrownTrace], {
    parserDiagnostics: [{ id: "host", note: "host failed at /tmp/egui_expressive_raster_trace/embedded_img.png" }]
  });
  assert(!warnings.some(w => String(w.note || w.message || "").includes("/tmp/egui_expressive_raster_trace")));
  assert(!warnings.some(w => String(w.note || w.message || "").includes("embedded_img.png")));
  assert(warnings.some(w => String(w.note || "").includes("[temporary raster extraction input]")));
  plugin.resetAiParserStateForTests();
}

function squareTraceResult(artboardName = "Artboard 1") {
  return rectTraceResult({ x: 0, y: 0, w: 10, h: 10 }, artboardName);
}

function rectTraceResult(rect, artboardName = "Artboard 1") {
  const x = rect.x;
  const y = rect.y;
  const w = rect.w;
  const h = rect.h;
  return {
    elements: [{
      id: "raster_trace",
      element_type: "path",
      artboard_name: artboardName,
      bounds: [x, y, w, h],
      path_closed: true,
      path_points: [
        { anchor: [x, y], left_ctrl: [x, y], right_ctrl: [x, y] },
        { anchor: [x + w, y], left_ctrl: [x + w, y], right_ctrl: [x + w, y] },
        { anchor: [x + w, y + h], left_ctrl: [x + w, y + h], right_ctrl: [x + w, y + h] },
        { anchor: [x, y + h], left_ctrl: [x, y + h], right_ctrl: [x, y + h] }
      ],
      appearance_fills: [{ r: 240, g: 32, b: 16, a: 255, opacity: 1 }]
    }]
  };
}

function assertTupleClose(actual, expected, message) {
  assert(Math.abs(actual[0] - expected[0]) < 0.0001 && Math.abs(actual[1] - expected[1]) < 0.0001, message || `${actual} != ${expected}`);
}

async function testRotatedAndEffectedRasterVectorization() {
  const rotated = {
    id: "rot_img",
    type: "image",
    x: 0,
    y: 0,
    w: 50,
    h: 100,
    depth: 0,
    imagePath: "/tmp/rot.png",
    rotation: 90,
    effects: [],
    notes: []
  };
  const rotatedFitRect = { x: -25, y: 25, w: 100, h: 50 };
  const rotatedTrace = await plugin.vectorizeRasterElement(rotated, "Artboard 1", async () => rectTraceResult(rotatedFitRect));
  assert.strictEqual(rotatedTrace.type, "group");
  assert.strictEqual(rotatedTrace.rotation, 0, "Rotation should be baked into traced vector children");
  assert.strictEqual(rotatedTrace.children.length, 1);
  assertTupleClose([rotatedTrace.x, rotatedTrace.y], [0, 0], "Rotated group bounds should stay aligned to original bbox");
  assertTupleClose([rotatedTrace.w, rotatedTrace.h], [50, 100], "Rotated group bounds should be recomputed from children");
  assertTupleClose(rotatedTrace.children[0].pathPoints[0].anchor, [50, 0], "First point should rotate around raster center exactly once");
  assertTupleClose(rotatedTrace.children[0].pathPoints[1].anchor, [50, 100], "Second point should rotate around raster center exactly once");

  let attemptedDiagonalWithoutScale = false;
  const diagonalWithoutScale = { ...rotated, id: "diag_no_scale", w: 141.421356, h: 141.421356, rotation: 45 };
  const diagonalNoScaleResult = await plugin.vectorizeRasterElement(diagonalWithoutScale, "Artboard 1", async () => {
    attemptedDiagonalWithoutScale = true;
    return squareTraceResult();
  });
  assert.strictEqual(attemptedDiagonalWithoutScale, false, "Non-orthogonal rotated linked rasters need transform scale metadata before tracing");
  assert.strictEqual(diagonalNoScaleResult.type, "image");
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 160, height: 160 }, elements: [diagonalNoScaleResult] }], { naming: false, codeOnlyStrict: true });
  }, /matrix-aware vector tracing/);

  const diagonalWithScale = { ...diagonalWithoutScale, id: "diag_scaled", rasterScaleX: 1.5, rasterScaleY: 1 };
  const diagonalCenter = diagonalWithScale.w / 2;
  const diagonalLocal = { x: diagonalCenter - 75, y: diagonalCenter - 25, w: 150, h: 50 };
  const diagonalTrace = await plugin.vectorizeRasterElement(diagonalWithScale, "Artboard 1", async () => rectTraceResult(diagonalLocal));
  assert.strictEqual(diagonalTrace.type, "group");
  assertTupleClose([diagonalTrace.w, diagonalTrace.h], [141.421356, 141.421356], "Stretched diagonal rotation should preserve original bbox when transform scale metadata is present");

  const embeddedRotated = {
    ...rotated,
    id: "embedded_rot",
    imagePath: null,
    embeddedRaster: true,
    extractedImagePath: "/tmp/egui_expressive_raster_trace/embedded_rot.png",
    extractedRasterAlreadyTransformed: true,
  };
  const embeddedTrace = await plugin.vectorizeRasterElement(embeddedRotated, "Artboard 1", async () => rectTraceResult({ x: 0, y: 0, w: 50, h: 100 }));
  assert.strictEqual(embeddedTrace.type, "group");
  assert.strictEqual(embeddedTrace.rotation, 0);
  assertTupleClose(embeddedTrace.children[0].pathPoints[0].anchor, [0, 0], "Transformed embedded extraction must not be rotated twice");
  assertTupleClose([embeddedTrace.x, embeddedTrace.y], [0, 0]);
  assertTupleClose([embeddedTrace.w, embeddedTrace.h], [50, 100]);

  const baseEffectImage = {
    id: "fx_img",
    type: "image",
    x: 0,
    y: 0,
    w: 10,
    h: 10,
    depth: 0,
    imagePath: "/tmp/fx.png",
    notes: []
  };
  const safeEffects = [
    ["dropShadow", "DropShadow", {}],
    ["innerShadow", "InnerShadow", {}],
    ["outerGlow", "OuterGlow", {}],
    ["innerGlow", "InnerGlow", {}],
    ["gaussianBlur", "GaussianBlur", {}],
    ["feather", "Feather", {}],
    ["bevel", "Bevel", { depth: 2, angle: 135, radius: 1, highlight: { r: 255, g: 255, b: 255, a: 0.5 }, shadowColor: { r: 0, g: 0, b: 0, a: 0.5 } }],
    ["noise", "Noise", { amount: 0.2, scale: 2, seed: 42 }],
  ];
  for (const [effectType, rustVariant, extra] of safeEffects) {
    const safeEffect = { ...baseEffectImage, id: `fx_${effectType}`, effects: [{ type: effectType, x: 2, y: 3, blur: 4, radius: 4, color: { r: 0, g: 0, b: 0, a: 1 }, ...extra }] };
    const safeEffectTrace = await plugin.vectorizeRasterElement(safeEffect, "Artboard 1", async () => squareTraceResult());
    assert.strictEqual(safeEffectTrace.type, "group");
    assert.strictEqual(safeEffectTrace.effects.length, 1);
    const exported = plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [safeEffectTrace] }], { naming: false, codeOnlyStrict: true });
    const code = exported.files["artboard_1.rs"];
    assert(code.includes("EffectLayer::new"), `${effectType} should be represented as a scene effect layer`);
    assert(code.includes(`EffectType::${rustVariant}`), `${effectType} should reach codegen`);
    assert(!code.includes("paint_image_slot"), "Effected raster traces must not emit image slots");
    assert.strictEqual(Object.keys(exported.assets).length, 0);
  }

  const clipSourceImage = {
    id: "clip_source_image",
    type: "image",
    x: 0,
    y: 0,
    w: 20,
    h: 20,
    depth: 1,
    vectorSourcePath: "/tmp/clip_source.png",
    effects: [],
    notes: []
  };
  assert.strictEqual(plugin.rasterVectorSourcePath(clipSourceImage), "/tmp/clip_source.png");
  let attemptedClipTrace = false;
  const tracedClipSource = await plugin.vectorizeRasterElement(clipSourceImage, "Artboard 1", async () => {
    attemptedClipTrace = true;
    return squareTraceResult();
  });
  assert.strictEqual(attemptedClipTrace, true, "vector source alias should allow tracing before clip parity checks");
  assert.strictEqual(tracedClipSource.type, "group");
  const clipWithVectorSource = {
    id: "clip_with_vector_source",
    type: "group",
    clipMask: true,
    x: 0,
    y: 0,
    w: 20,
    h: 20,
    depth: 0,
    children: [
      tracedClipSource,
      { id: "clip_with_vector_source_caption", type: "text", text: "Caption", textStyle: { size: 14 }, effects: [], notes: [] }
    ],
    effects: [],
    notes: []
  };
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [clipWithVectorSource] }], { naming: false, codeOnlyStrict: true });
  }, /mixed clipping groups containing text are not parity-safe yet/, "vector-backed raster clip children with text should fail until exact text clipping is implemented");

  let attemptedUnsafeTrace = false;
  const unsafeEffect = { ...baseEffectImage, id: "unsafe_fx", effects: [{ type: "unknown", source: "xmp" }] };
  const unsafeResult = await plugin.vectorizeRasterElement(unsafeEffect, "Artboard 1", async () => {
    attemptedUnsafeTrace = true;
    return squareTraceResult();
  });
  assert.strictEqual(attemptedUnsafeTrace, false, "Unsafe raster effects must not trace silently");
  assert.strictEqual(unsafeResult.type, "image");
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [unsafeResult] }], { naming: false, codeOnlyStrict: true });
  }, /effect-aware vector tracing/);
}

async function testCepRasterEffectsClassifyBeforeVectorization() {
  const safeHostEl = extractHostRasterElementWithXmp('outerGlow feather bevel noise');
  assert(safeHostEl.effects.some(effect => effect.type === 'outerGlow'));
  assert(safeHostEl.effects.some(effect => effect.type === 'feather'));
  assert(safeHostEl.effects.some(effect => effect.type === 'bevel'));
  assert(safeHostEl.effects.some(effect => effect.type === 'noise'));
  const safeTrace = await plugin.vectorizeRasterElement(safeHostEl, 'Artboard 1', async () => squareTraceResult());
  assert.strictEqual(safeTrace.type, 'group');
  const exported = plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [safeTrace] }], { naming: false, codeOnlyStrict: true });
  const code = exported.files['artboard_1.rs'];
  assert(code.includes('EffectType::OuterGlow'));
  assert(code.includes('EffectType::Feather'));
  assert(code.includes('EffectType::Bevel'));
  assert(code.includes('EffectType::Noise'));
  assert(!code.includes('paint_image_slot'));
  assert.strictEqual(Object.keys(exported.assets).length, 0);

  const unknownHostEl = extractHostRasterElementWithXmp('filter=MotionBlur');
  assert(!unknownHostEl.effects.some(effect => effect.type === 'gaussianBlur'));
  assert(unknownHostEl.effects.some(effect => effect.type === 'unknown'));
  let attemptedUnknownHostTrace = false;
  const unknownHostTrace = await plugin.vectorizeRasterElement(unknownHostEl, 'Artboard 1', async () => {
    attemptedUnknownHostTrace = true;
    return squareTraceResult();
  });
  assert.strictEqual(attemptedUnknownHostTrace, false, 'Unknown host raster effects must not trace silently');
  assert.strictEqual(unknownHostTrace.type, 'image');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [unknownHostTrace] }], { naming: false, codeOnlyStrict: true });
  }, /effect-aware vector tracing/);

  const pluginSideEffects = sandbox.extractEffects({ XMPString: 'effect=RadialBlur' });
  assert(!pluginSideEffects.some(effect => effect.type === 'gaussianBlur'));
  assert(pluginSideEffects.some(effect => effect.type === 'unknown'));
  let attemptedUnknownPluginTrace = false;
  const unknownPluginTrace = await plugin.vectorizeRasterElement({
    id: 'plugin_unknown_fx',
    type: 'image',
    x: 0,
    y: 0,
    w: 10,
    h: 10,
    depth: 0,
    imagePath: '/tmp/plugin_unknown.png',
    effects: pluginSideEffects,
    notes: []
  }, 'Artboard 1', async () => {
    attemptedUnknownPluginTrace = true;
    return squareTraceResult();
  });
  assert.strictEqual(attemptedUnknownPluginTrace, false, 'Unknown plugin raster effects must not trace silently');
  assert.strictEqual(unknownPluginTrace.type, 'image');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: [unknownPluginTrace] }], { naming: false, codeOnlyStrict: true });
  }, /effect-aware vector tracing/);
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

function testCompoundPathSceneEmission() {
  const point = (x, y) => ({ anchor: [x, y], leftDir: [x, y], rightDir: [x, y] });
  const outer = [point(0, 0), point(40, 0), point(40, 40), point(0, 40)];
  const inner = [point(10, 10), point(30, 10), point(30, 30), point(10, 30)];
  const results = [{
    artboard: { name: "Artboard 1", width: 100, height: 100 },
    elements: [{
      id: "compound_logo", type: "shape", x: 0, y: 0, w: 40, h: 40, depth: 0,
      isCompoundPath: true,
      fillRule: "evenodd",
      subpaths: [{ points: outer, closed: true }, { points: inner, closed: true }],
      fill: { r: 255, g: 0, b: 0 },
      effects: [], notes: []
    }]
  }];

  const exported = plugin.exportFromRawData(results, { naming: false, codeOnlyStrict: true, includeSidecar: true });
  const code = exported.files["artboard_1.rs"];
  assert(code.includes("SceneNode::compound_path"), "Compound paths should use scene compound geometry");
  assert(code.includes("egui_expressive::scene::PathContour"), "Compound subpaths should emit path contours");
  assert(code.includes("egui_expressive::scene::FillRule::EvenOdd"), "Compound paths should preserve even-odd fill rule");
  assert(code.includes("(10.0, 10.0)"), "Inner contour should be emitted");
  assert(!code.includes("paint_placeholder_slot"), "Compound paths must not fall back to placeholders");

  const sidecar = JSON.parse(exported.files["artboard_1.json"]);
  assert.strictEqual(sidecar.elements[0].fillRule, "evenodd");
  assert.strictEqual(sidecar.elements[0].subpaths.length, 2);
  assert.strictEqual(sidecar.elements[0].parityStatus, "exact");
  assert.strictEqual((sidecar.elements[0].parityReasons || []).length, 0);
  assert(!plugin.parityFindingsForElement(sidecar.elements[0]).some(finding => finding.status === "unsupported"));

  const nonzeroExported = plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [{
      id: "compound_nonzero", type: "shape", x: 0, y: 0, w: 40, h: 40, depth: 0,
      isCompoundPath: true,
      fillRule: "nonzero",
      subpaths: [{ points: outer, closed: true }, { points: inner, closed: true }],
      fill: { r: 255, g: 0, b: 0 },
      effects: [], notes: []
  }] }], { naming: false, codeOnlyStrict: true });
  assert(nonzeroExported.files["artboard_1.rs"].includes("egui_expressive::scene::FillRule::NonZero"));

  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [{
      id: "compound_unknown", type: "shape", x: 0, y: 0, w: 40, h: 40, depth: 0,
      isCompoundPath: true,
      subpaths: [{ points: outer, closed: true }, { points: inner, closed: true }],
      fill: { r: 255, g: 0, b: 0 },
      effects: [], notes: []
    }] }], { naming: false });
  }, /compound path fill rule is unavailable/);

  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [{
      id: "compound_nonzero_pattern", type: "shape", x: 0, y: 0, w: 40, h: 40, depth: 0,
      isCompoundPath: true,
      fillRule: "nonzero",
      subpaths: [{ points: outer, closed: true }, { points: inner, closed: true }],
      gradient: { type: "pattern", patternName: "dots" },
      effects: [], notes: []
    }] }], { naming: false, codeOnlyStrict: true });
  }, /compound path pattern fill with fill rule nonzero/);

  const overlap = [point(20, 20), point(50, 20), point(50, 50), point(20, 50)];
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [{
      id: "compound_overlap", type: "shape", x: 0, y: 0, w: 50, h: 50, depth: 0,
      isCompoundPath: true,
      fillRule: "evenodd",
      subpaths: [{ points: outer, closed: true }, { points: overlap, closed: true }],
      fill: { r: 255, g: 0, b: 0 },
      effects: [], notes: []
    }] }], { naming: false, codeOnlyStrict: true });
  }, /overlapping or intersecting contours/);

  const bowtie = [point(10, 10), point(30, 30), point(10, 30), point(30, 10)];
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [{
      id: "compound_self_intersect", type: "shape", x: 0, y: 0, w: 40, h: 40, depth: 0,
      isCompoundPath: true,
      fillRule: "evenodd",
      subpaths: [{ points: outer, closed: true }, { points: bowtie, closed: true }],
      fill: { r: 255, g: 0, b: 0 },
      effects: [], notes: []
    }] }], { naming: false, codeOnlyStrict: true });
  }, /overlapping or intersecting contours/);
}

function testAiParserFillRulePropagation() {
  const point = (x, y) => ({ anchor: [x, y], leftDir: [x, y], rightDir: [x, y] });
  const outer = [point(0, 0), point(40, 0), point(40, 40), point(0, 40)];
  const inner = [point(10, 10), point(30, 10), point(30, 30), point(10, 30)];
  const parser = {
    elements: [{
      id: 'pdf_path_9_0',
      element_type: 'shape',
      artboard_name: 'Artboard 1',
      bounds: [0, 0, 40, 40],
      fill_rule: 'evenodd',
      path_closed: true,
      path_points: outer,
      subpaths: [{ points: outer, closed: true }, { points: inner, closed: true }],
      appearance_fills: [{ r: 255, g: 0, b: 0, a: 255 }],
      effects: []
    }]
  };

  const merged = plugin.mergeAiParserData([], parser, 'Artboard 1');
  assert.strictEqual(merged.length, 1);
  assert.strictEqual(merged[0].fillRule, 'evenodd');
  assert.strictEqual(merged[0].subpaths.length, 2);

  const exported = plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: merged }], { naming: false, codeOnlyStrict: true, includeSidecar: true });
  const code = exported.files['artboard_1.rs'];
  assert(code.includes('FillRule::EvenOdd'));
  const sidecar = JSON.parse(exported.files['artboard_1.json']);
  assert.strictEqual(sidecar.elements[0].fillRule, 'evenodd');
  assert.notStrictEqual(sidecar.elements[0].parityStatus, 'unsupported');
  assert(!((sidecar.elements[0].parityReasons || []).join(' ')).includes('fill rule is unavailable'));
  assert(!plugin.parityFindingsForElement(merged[0]).some(finding => /fill rule is unavailable/i.test(finding.reason)));
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
    assert(code.includes("scene::render_node"), "Vector strokes should use scene renderer");
    assert(code.includes("egui_expressive::codegen::StrokeCap::Round"), "Circle cap should be preserved");
    assert(code.includes(".dash(vec![2.0, 4.0])"), "Circle dash should be preserved");
    assert(code.includes(".with_rotation(15.0)"), "Rotated stroke should preserve rotation in scene node");
    assert(code.includes("egui_expressive::scene::path_points"), "Scene paths should use compact tuple helper");
    assert(code.includes("(15.0, 50.0)"), "Ellipse should use parser path points");
    assert(code.includes(".with_opacity(0.5)"), "Normal stroke opacity should be emitted");
    assert(code.includes("egui_expressive::codegen::BlendMode::Multiply"), "Stroke blend mode should be emitted");
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
    assert(code.includes("PaintSource::LinearGradient"), "Linear gradient should use scene paint source");
    assert(code.includes("PaintSource::RadialGradient"), "Radial gradient should use scene paint source");
    assert(!code.includes("radial_gradient_rect_stops"), "Should not emit radial_gradient_rect_stops for vector paths");
    assert(!code.includes("linear_gradient_rect"), "Should not emit linear_gradient_rect for vector paths");
    assert(code.includes("center: Some([44.0, 9.0])"), "Radial center should be emitted");
    assert(code.includes("focal_point: Some([46.0, 11.0])"), "Radial focal point should be emitted");
    assert(code.includes("Some(18.0)"), "Radial radius should be emitted");
    assert(code.includes("transform: Some([1.0, 0.0, 0.0, 1.0, 4.0, 5.0])"), "Radial transform should be emitted");
    assert(code.includes('SceneNode::rect("rounded_gradient"') && code.includes(", 6.0)"), "Rounded gradient rect should preserve corner radius");
    assert(code.includes("egui_expressive::scene::path_points"), "Vector paths should use compact tuple helper");
    assert(code.includes("(30.0, 10.0)"), "Ellipse should use parser path points");
    assert(code.includes("(20.0, 30.0)"), "Path should use parser path points");
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
  const exported = plugin.exportFromRawData ? plugin.exportFromRawData(results, { naming: false, includeSidecar: true, codeOnlyStrict: false }) : null;
  if (exported) {
    const code = exported.files["artboard_1.rs"];
    assert(code.includes("PaintSource::Pattern"), "Pattern fill should emit scene pattern primitive");
    assert(code.includes('name: "Diagonal Dots".to_string()'), "Pattern metadata should be preserved in generated code");
    assert(code.includes('name: "conic".to_string()'), "Unknown gradient metadata should be preserved as a vector pattern primitive");
    assert(!code.includes("approximate with solid fill"), "Pattern fill should not fall back to solid fill");
    assert(!code.includes("linear_gradient_rect(rect"), "Pattern fill should not be treated as a linear gradient");
    const seedMatch = code.match(/PatternDef \{ name: "Diagonal Dots"\.to_string\(\), seed: (\d+)u32, foreground: egui::Color32::from_rgba_unmultiplied\([^)]+\), background: egui::Color32::from_rgba_unmultiplied\([^)]+\), cell_size: ([\d.]+), mark_size: ([\d.]+), .*?tile_shapes:/);
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
    assert(code.includes("let painter = ui.painter().clone();"), "Generated scene renderer code should not hold a Ui-borrowed painter");
    assert(code.includes("scene::render_node"), "Blend appearance stack should route through scene renderer");
    assert(code.includes("scene::render_node(ui, &painter"), "Scene renderer should receive painter by reference");
    assert(code.includes("Vector primitive routed through egui_expressive::scene"));
    assert(code.includes("egui_expressive::codegen::BlendMode::Multiply"));
    assert(code.includes("egui_expressive::codegen::BlendMode::Screen"));
    assert(code.includes("PaintSource::LinearGradient"));
    assert(code.includes('SceneNode::ellipse("circle_stack"'), "Circle appearance stack should use scene renderer");
    assert(code.includes('SceneNode::path(') && code.includes('"path_stack"'), "Path appearance stack should use scene renderer");
    assert(code.includes('SceneNode::path(') && code.includes('"open_path_stack"'), "Open 2-point path appearance stack should use scene renderer");
    assert(code.includes('"open_path_stack"') && code.includes('false,'), "Open path should remain open scene path");
    assert(code.includes('SceneNode::rect("shape_effect_stack"'), "Explicit shape effect stack should use scene renderer");
    assert(code.includes('SceneNode::rect("parser_effect_stack"'), "Parser-sourced fill/effect/stroke stack should use scene renderer");
    assert(code.includes("EffectType::DropShadow"), "Appearance-stack effects should be preserved in scene renderer");
    assert(code.includes("SceneNode::path"), "Path appearance stack should preserve path geometry");
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

function testMergeParserDataPromotesOpaqueToGroup() {
  const dom = [{
    id: 'plugin_1', type: 'plugin', x: 10, y: 20, w: 30, h: 40, children: [],
    fill: { r: 1, g: 2, b: 3 }, stroke: { r: 4, g: 5, b: 6, width: 2 },
    effects: [{ type: 'dropShadow', color: { r: 1, g: 2, b: 3, a: 0.5 }, blur: 4, x: 1, y: 1 }]
  }];
  const parser = {
    elements: [{
      id: 'element_8',
      element_type: 'path',
      artboard_name: 'Artboard 1',
      bounds: [15, 25, 20, 30],
      path_closed: true,
      path_points: [
        { anchor: [15, 25], left_ctrl: [15, 25], right_ctrl: [15, 25] },
        { anchor: [35, 25], left_ctrl: [35, 25], right_ctrl: [35, 25] },
        { anchor: [35, 55], left_ctrl: [35, 55], right_ctrl: [35, 55] }
      ],
      appearance_fills: [{ r: 255, g: 0, b: 0, a: 255 }]
    }]
  };
  const merged = plugin.mergeAiParserData(dom, parser, 'Artboard 1');
  assert.strictEqual(merged.length, 1);
  assert.strictEqual(merged[0].type, 'group');
  assert.strictEqual(merged[0].parserRecovered, true);
  assert.strictEqual(merged[0].children.length, 1);
  assert.strictEqual(merged[0].children[0].parserId, 'element_8');
  assert.strictEqual(plugin.parityStatusForElement(merged[0]), 'approximate');

  const exported = plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: merged }], { naming: false });
  const code = exported.files['artboard_1.rs'];
  assert(!code.includes('paint_placeholder_slot'));
  assert(code.includes('scene::render_node'));
  assert(!code.includes('from_rgba_unmultiplied(1, 2, 3'), 'Recovered wrapper must not paint original opaque DOM fill/effects');
  assert(!code.includes('from_rgba_unmultiplied(4, 5, 6'), 'Recovered wrapper must not paint original opaque DOM stroke');
  assert(!code.includes('with_effect_layer'), 'Recovered wrapper must be structural only');

  const unsafeParser = {
    elements: [{
      id: 'open_fill',
      element_type: 'path',
      artboard_name: 'Artboard 1',
      bounds: [15, 25, 20, 30],
      path_points: [
        { anchor: [15, 25], left_ctrl: [15, 25], right_ctrl: [15, 25] },
        { anchor: [35, 55], left_ctrl: [35, 55], right_ctrl: [35, 55] }
      ],
      appearance_fills: [{ r: 255, g: 0, b: 0, a: 255 }]
    }]
  };
  const unsafeMerged = plugin.mergeAiParserData([{ id: 'plugin_2', type: 'plugin', x: 10, y: 20, w: 30, h: 40, children: [], effects: [] }], unsafeParser, 'Artboard 1');
  assert.strictEqual(unsafeMerged[0].type, 'plugin', 'Open two-point filled parser paths must not recover opaque items');
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: 'Artboard 1', width: 100, height: 100 }, elements: unsafeMerged }], { naming: false });
  }, /Cannot export code-only Rust/);

  const otherArtboardParser = {
    elements: [{
      id: 'other_artboard_vector',
      artboard_name: 'Artboard 2',
      element_type: 'path',
      bounds: [15, 25, 20, 30],
      path_closed: true,
      path_points: [
        { anchor: [15, 25], left_ctrl: [15, 25], right_ctrl: [15, 25] },
        { anchor: [35, 25], left_ctrl: [35, 25], right_ctrl: [35, 25] },
        { anchor: [35, 55], left_ctrl: [35, 55], right_ctrl: [35, 55] }
      ],
      appearance_fills: [{ r: 255, g: 0, b: 0, a: 255 }]
    }]
  };
  const crossArtboardMerged = plugin.mergeAiParserData(
    [{ id: 'plugin_3', type: 'plugin', x: 10, y: 20, w: 30, h: 40, children: [], effects: [] }],
    otherArtboardParser,
    'Artboard 1'
  );
  assert.strictEqual(crossArtboardMerged[0].type, 'plugin', 'Opaque recovery must ignore parser vectors assigned to another artboard');
}

function testStrictCodeOnlyExport() {
  if (!plugin.exportFromRawData) return;

  const baseResults = [{
    artboard: { name: "Artboard 1", width: 100, height: 100 },
    elements: []
  }];

  const testCases = [
    {
      name: "Linked image",
      elements: [{ id: "img_1", type: "image", x: 0, y: 0, w: 10, h: 10, depth: 0, imagePath: "test.png" }],
      expectThrow: true
    },
    {
      name: "Embedded image",
      elements: [{ id: "img_2", type: "image", x: 0, y: 0, w: 10, h: 10, depth: 0, embeddedRaster: true }],
      expectThrow: true
    },
    {
      name: "Gradient stroke",
      elements: [{ id: "stroke_1", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, stroke: { width: 2, gradient: { type: "linear", stops: [{ position: 0, color: "#ff0000" }, { position: 1, color: "#0000ff" }] } } }],
      expectThrow: false,
      assertCode: code => {
        assert(code.includes('PaintSource::LinearGradient'), 'Non-dashed gradient stroke should reach scene code');
        assert(code.includes('StrokeLayer'), 'Non-dashed gradient stroke should remain a code-rendered stroke');
      }
    },
    {
      name: "Dashed gradient stroke",
      elements: [{ id: "stroke_dash", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, strokeDash: [2, 2], stroke: { width: 2, gradient: { type: "linear", stops: [{ position: 0, color: "#ff0000" }, { position: 1, color: "#0000ff" }] } } }],
      expectThrow: false,
      assertCode: code => {
        assert(code.includes('PaintSource::LinearGradient'), 'Dashed gradient stroke should reach scene code');
        assert(code.includes('StrokeLayer'), 'Dashed gradient stroke should remain a code-rendered stroke');
      }
    },
    {
      name: "Dashed pattern stroke",
      elements: [{ id: "stroke_pattern_dash", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, strokeDash: [2, 2], stroke: { width: 2, pattern: { patternName: "dots" } } }],
      expectThrow: true
    },
    {
      name: "Inside aligned stroke",
      elements: [{ id: "inside_stroke", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, strokeAlignment: "inside", stroke: { r: 0, g: 0, b: 0, width: 2 } }],
      expectThrow: false,
      assertCode: code => assert(code.includes(".alignment(egui_expressive::scene::StrokeAlignment::Inside)"))
    },
    {
      name: "Complex blend fallback",
      elements: [{ id: "blend_1", type: "text", text: "Hello", blendMode: "multiply" }],
      expectThrow: true
    },
    {
      name: "Opaque Illustrator effect",
      elements: [{ id: "opaque_fx", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, isOpaque: true, fill: { r: 255, g: 0, b: 0 }, thirdPartyEffects: [{ opaque: true }] }],
      expectThrow: true
    },
    {
      name: "Unexpanded symbol",
      elements: [{ id: "symbol_1", type: "symbol", symbolName: "Logo", x: 0, y: 0, w: 10, h: 10, depth: 0, children: [] }],
      expectThrow: true
    },
    {
      name: "Parser-backed symbol expansion",
      elements: [{ id: "symbol_parser", type: "symbol", symbolName: "Logo", x: 0, y: 0, w: 10, h: 10, depth: 0, parserRecovered: true, children: [{ id: "symbol_parser_mark", type: "shape", x: 1, y: 1, w: 8, h: 8, depth: 1, fill: { r: 12, g: 34, b: 56 }, effects: [], notes: [] }] }],
      expectThrow: false,
      assertCode: code => {
        assert(code.includes('Symbol instance: "Logo"'));
        assert(code.includes('symbol_parser_mark'));
      }
    },
    {
      name: "Expanded symbol",
      elements: [{ id: "symbol_2", type: "symbol", symbolName: "Logo", x: 0, y: 0, w: 10, h: 10, depth: 0, children: [{ id: "symbol_2_mark", type: "shape", x: 1, y: 1, w: 8, h: 8, depth: 1, fill: { r: 12, g: 34, b: 56 }, effects: [], notes: [] }] }],
      expectThrow: false,
      assertCode: code => {
        assert(code.includes('Symbol instance: "Logo"'));
        assert(code.includes('symbol_2_mark'));
      }
    },
    {
      name: "Code-rendered bevel effect",
      elements: [{ id: "bevel_fx", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, fill: { r: 1, g: 2, b: 3 }, effects: [{ type: "bevel", depth: 2, angle: 135, radius: 1 }] }],
      expectThrow: false,
      assertCode: code => assert(code.includes("EffectType::Bevel"))
    },
    {
      name: "Code-rendered noise effect",
      elements: [{ id: "noise_fx", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, fill: { r: 1, g: 2, b: 3 }, effects: [{ type: "noise", amount: 0.2, scale: 2, seed: 42 }] }],
      expectThrow: false,
      assertCode: code => assert(code.includes("EffectType::Noise"))
    },
    {
      name: "Unsupported live effect",
      elements: [{ id: "live_fx", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, fill: { r: 1, g: 2, b: 3 }, effects: [{ type: "liveEffect", name: "Twist" }] }],
      expectThrow: true,
      errorIncludes: "parser-expanded vectors missing"
    },
    {
      name: "Parser-backed live effect expansion",
      elements: [{ id: "live_fx_parser", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, fill: { r: 1, g: 2, b: 3 }, parserRecovered: true, effects: [{ type: "liveEffect", name: "Twist" }], children: [{ id: "live_fx_parser_child", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 1, fill: { r: 1, g: 2, b: 3 }, effects: [], notes: [] }] }],
      expectThrow: false,
      assertCode: code => {
        assert(code.includes('live_fx_parser_child'));
        assert(!code.includes('parser-expanded vectors missing'));
      }
    },
    {
      name: "Parser-backed appearance expansion",
      elements: [{ id: "appearance_parser", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, appearanceProbe: { fillCount: 3, strokeCount: 0 }, appearanceExpanded: true, appearanceStack: [{ type: 'fill', color: { r: 1, g: 2, b: 3 }, opacity: 1, blendMode: 'normal' }, { type: 'fill', color: { r: 4, g: 5, b: 6 }, opacity: 1, blendMode: 'normal' }, { type: 'fill', color: { r: 7, g: 8, b: 9 }, opacity: 1, blendMode: 'normal' }], fill: { r: 1, g: 2, b: 3 }, stroke: null }],
      expectThrow: false,
      assertCode: code => assert(code.includes('appearance_parser'))
    },
    {
      name: "Envelope mesh",
      elements: [{ id: "envelope", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, envelope_mesh: { rows: 2, cols: 2, points: [] } }],
      expectThrow: true
    },
    {
      name: "3D appearance",
      elements: [{ id: "three_d", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, three_d: { type: "extrude", depth: 10 } }],
      expectThrow: true
    },
    {
      name: "Justified text",
      elements: [{ id: "justified", type: "text", text: "Hello", x: 0, y: 0, w: 50, h: 10, depth: 0, textAlign: "justified" }],
      expectThrow: false
    },
    {
      name: "Justified last-line center text",
      elements: [{ id: "justified_center", type: "text", text: "Hello", x: 0, y: 0, w: 50, h: 10, depth: 0, textAlign: "justified_last_line_center" }],
      expectThrow: false,
      assertCode: code => assert(code.includes("TextBlockAlign::JustifiedLastLineCenter"))
    },
    {
      name: "Justified last-line right text",
      elements: [{ id: "justified_right", type: "text", text: "Hello", x: 0, y: 0, w: 50, h: 10, depth: 0, textAlign: "justified_last_line_right" }],
      expectThrow: false,
      assertCode: code => assert(code.includes("TextBlockAlign::JustifiedLastLineRight"))
    },
    {
      name: "Justified all-lines text",
      elements: [{ id: "justified_all", type: "text", text: "Hello world", x: 0, y: 0, w: 50, h: 10, depth: 0, textAlign: "justified_all" }],
      expectThrow: false,
      assertCode: code => assert(code.includes("TextBlockAlign::JustifiedAll"))
    },
    {
      name: "Small caps text",
      elements: [{ id: "small_caps", type: "text", text: "Hello", x: 0, y: 0, w: 50, h: 10, depth: 0, textTransform: "small_caps" }],
      expectThrow: true,
      assertCode: code => assert(code.includes("TextTransform::SmallCaps"))
    },
    {
      name: "Allowed vector",
      elements: [{ id: "rect_1", type: "shape", x: 0, y: 0, w: 10, h: 10, depth: 0, fill: { r: 255, g: 0, b: 0 } }],
      expectThrow: false
    },
    {
      name: "Gradient mesh patches",
      elements: [{
        id: "mesh_patch",
        type: "mesh",
        x: 0,
        y: 0,
        w: 20,
        h: 20,
        depth: 0,
        isGradientMesh: true,
        mesh_patches: [{
          corners: [[0, 0], [20, 0], [20, 20], [0, 20]],
          colors: [[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255], [255, 255, 0, 255]]
        }]
      }],
      expectThrow: false,
      assertCode: code => {
        assert(code.includes('mesh_gradient_patch'), 'Gradient mesh patches should emit mesh_gradient_patch call');
        assert(!code.includes('WARNING:'), 'Gradient mesh should not emit WARNING fallbacks');
      }
    },
    {
      name: "Vector clip group",
      elements: [{
        id: "clip_group",
        type: "group",
        clipMask: true,
        x: 0,
        y: 0,
        w: 50,
        h: 50,
        depth: 0,
        children: [{ id: "clip_child", type: "shape", x: 5, y: 5, w: 10, h: 10, depth: 1, fill: { r: 255, g: 0, b: 0 } }]
      }],
      expectThrow: false,
      assertCode: code => {
        assert(code.includes("clip_group"), "Vector clip group should emit clip_group");
        assert(!code.includes("paint_placeholder_slot"), "Clip group must not fall back to placeholders");
        assert(!code.includes("WARNING:"), "Vector clip group should not emit WARNING fallbacks");
      }
    },
    {
      name: "Mixed path clip group with text",
      elements: [{
        id: "mixed_clip_group",
        type: "group",
        clipMask: true,
        x: 0,
        y: 0,
        w: 60,
        h: 40,
        depth: 0,
        pathClosed: true,
        pathPoints: [
          { anchor: [0, 0], leftDir: [0, 0], rightDir: [0, 0] },
          { anchor: [60, 0], leftDir: [60, 0], rightDir: [60, 0] },
          { anchor: [60, 40], leftDir: [60, 40], rightDir: [60, 40] },
          { anchor: [0, 40], leftDir: [0, 40], rightDir: [0, 40] }
        ],
        children: [
          { id: "mixed_clip_shape", type: "shape", x: 5, y: 5, w: 20, h: 20, depth: 1, fill: { r: 255, g: 0, b: 0 } },
          { id: "mixed_clip_text", type: "text", text: "Masked", x: 8, y: 10, w: 40, h: 12, depth: 1 }
        ]
      }],
      expectThrow: true,
      errorIncludes: "mixed clipping groups containing text are not parity-safe yet"
    },
    {
      name: "Mixed path clip group with image",
      elements: [{
        id: "mixed_clip_image_group",
        type: "group",
        clipMask: true,
        x: 0,
        y: 0,
        w: 60,
        h: 40,
        depth: 0,
        pathClosed: true,
        pathPoints: [
          { anchor: [0, 0], leftDir: [0, 0], rightDir: [0, 0] },
          { anchor: [60, 0], leftDir: [60, 0], rightDir: [60, 0] },
          { anchor: [60, 40], leftDir: [60, 40], rightDir: [60, 40] },
          { anchor: [0, 40], leftDir: [0, 40], rightDir: [0, 40] }
        ],
        children: [
          { id: "masked_image", type: "image", x: 0, y: 0, w: 60, h: 40, depth: 1, imagePath: "photo.png" },
          { id: "mixed_clip_caption", type: "text", text: "Caption", x: 8, y: 10, w: 40, h: 12, depth: 1 }
        ]
      }],
      expectThrow: true,
      errorIncludes: "Linked raster/images could not be vectorized and will not be exported as raster"
    },
    {
      name: "Nonzero compound vector",
      elements: [{
        id: "compound_nonzero",
        type: "shape",
        x: 0,
        y: 0,
        w: 20,
        h: 20,
        depth: 0,
        isCompoundPath: true,
        fillRule: "nonzero",
        fill: { r: 255, g: 0, b: 0 },
        subpaths: [
          { closed: true, points: [
            { anchor: [0, 0], leftDir: [0, 0], rightDir: [0, 0] },
            { anchor: [20, 0], leftDir: [20, 0], rightDir: [20, 0] },
            { anchor: [20, 20], leftDir: [20, 20], rightDir: [20, 20] },
            { anchor: [0, 20], leftDir: [0, 20], rightDir: [0, 20] }
          ] },
          { closed: true, points: [
            { anchor: [5, 5], leftDir: [5, 5], rightDir: [5, 5] },
            { anchor: [15, 5], leftDir: [15, 5], rightDir: [15, 5] },
            { anchor: [15, 15], leftDir: [15, 15], rightDir: [15, 15] },
            { anchor: [5, 15], leftDir: [5, 15], rightDir: [5, 15] }
          ] }
        ]
      }],
      expectThrow: false,
      assertCode: code => assert(code.includes("egui_expressive::scene::FillRule::NonZero"))
    },
    {
      name: "Compound hole clip group",
      elements: [{
        id: "compound_clip",
        type: "shape",
        x: 0, y: 0, w: 40, h: 40, depth: 0,
        isCompoundPath: true,
        clipMask: true,
        fillRule: "evenodd",
        subpaths: [
          { closed: true, points: [
            { anchor: [0, 0], leftDir: [0, 0], rightDir: [0, 0] },
            { anchor: [40, 0], leftDir: [40, 0], rightDir: [40, 0] },
            { anchor: [40, 40], leftDir: [40, 40], rightDir: [40, 40] },
            { anchor: [0, 40], leftDir: [0, 40], rightDir: [0, 40] }
          ] },
          { closed: true, points: [
            { anchor: [10, 10], leftDir: [10, 10], rightDir: [10, 10] },
            { anchor: [30, 10], leftDir: [30, 10], rightDir: [30, 10] },
            { anchor: [30, 30], leftDir: [30, 30], rightDir: [30, 30] },
            { anchor: [10, 30], leftDir: [10, 30], rightDir: [10, 30] }
          ] }
        ],
        effects: [], notes: []
      }],
      expectThrow: false,
      assertCode: code => {
        assert(code.includes("SceneNode::compound_path"), "Compound hole clip should use compound_path");
        assert(code.includes("with_clip_children(true)"), "Compound hole clip should set clip_children");
        assert(code.includes("egui_expressive::scene::FillRule::EvenOdd"), "Compound hole clip should preserve even-odd");
        assert(!code.includes("paint_placeholder_slot"), "Compound hole clip must not fall back to placeholders");
      }
    }
  ];

  for (const tc of testCases) {
    const results = [{ ...baseResults[0], elements: tc.elements }];
    let threw = false;
    try {
      const exported = plugin.exportFromRawData(results, { naming: false, codeOnlyStrict: true });
      if (!tc.expectThrow) {
        const code = exported.files["artboard_1.rs"];
        assert(!code.includes("paint_image_slot"), "Strict export should not emit paint_image_slot");
        assert(!code.includes("WARNING:"), "Strict export should not emit WARNING fallbacks");
        if (tc.assertCode) tc.assertCode(code);
      }
    } catch (e) {
      threw = true;
      assert(e.message.includes("Cannot export code-only Rust"), "Should throw strict export error");
      if (tc.name.includes("image")) {
        assert(e.message.includes("vector") && e.message.includes("will not be exported as raster"), "Raster/image strict error should be explicit");
      }
      if (tc.errorIncludes) {
        assert(e.message.includes(tc.errorIncludes), `Strict export error should mention ${tc.errorIncludes}`);
      }
    }
    assert.strictEqual(threw, tc.expectThrow, `Test case ${tc.name} failed strict export expectation`);
  }
}

function testLiveEffectDiagnosticsAreNamedAndHonest() {
  const el = { id: "live_fx", type: "shape", x: 0, y: 0, w: 10, h: 10, effects: [{ type: "liveEffect", name: "Adobe Twist" }] };
  const strictFindings = plugin.parityFindingsForElement(el, { codeOnlyStrict: true });
  assert(strictFindings.some(f => f.status === "unsupported" && f.reason.includes("Adobe Twist") && f.reason.includes("distort") && f.reason.includes("Object > Expand Appearance")));

  const looseFindings = plugin.parityFindingsForElement(el, { codeOnlyStrict: false });
  assert(looseFindings.some(f => f.status === "approximate" && f.reason.includes("Adobe Twist") && f.reason.includes("skipped in code export")));

  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [el] }], { naming: false, codeOnlyStrict: true });
  }, /Adobe Twist.*Object > Expand Appearance/);
}

function testHostCompoundPathExtraction() {
  const hostSource = fs.readFileSync(path.join(__dirname, 'host.jsx'), 'utf8');
  const hostSandbox = {
    Folder: function(path) { this.fsName = path; this.exists = true; this.create = function() { this.exists = true; }; },
    File: function(path) {
      this.fsName = path;
      this.exists = true;
      this.parent = { exists: true };
      this.copy = function() { return true; };
      this.open = function(mode) { this.mode = mode; this.buffer = ''; return true; };
      this.write = function(content) { this.buffer += String(content); return true; };
      this.writeln = function(content) { this.buffer += String(content) + '\n'; return true; };
      this.close = function() {};
    },
    PointType: { SMOOTH: 1, CORNER: 2 }
  };

  vm.runInNewContext(hostSource, hostSandbox, { filename: 'host.jsx' });

  hostSandbox.app = {
    documents: [{}],
    activeDocument: {
      name: "test.ai",
      artboards: [{
        name: "Artboard 1",
        artboardRect: [0, 100, 100, 0]
      }],
      pageItems: [{
        typename: "CompoundPathItem",
        name: "compound_1",
        geometricBounds: [10, 90, 90, 10],
        pathItems: [
          {
            typename: "PathItem",
            evenodd: true,
            closed: true,
            pathPoints: [
              { anchor: [10, 90], leftDirection: [10, 90], rightDirection: [10, 90], pointType: 2 },
              { anchor: [90, 90], leftDirection: [90, 90], rightDirection: [90, 90], pointType: 2 },
              { anchor: [90, 10], leftDirection: [90, 10], rightDirection: [90, 10], pointType: 2 },
              { anchor: [10, 10], leftDirection: [10, 10], rightDirection: [10, 10], pointType: 2 }
            ]
          },
          {
            typename: "PathItem",
            closed: true,
            pathPoints: [
              { anchor: [20, 80], leftDirection: [20, 80], rightDirection: [20, 80], pointType: 2 },
              { anchor: [80, 80], leftDirection: [80, 80], rightDirection: [80, 80], pointType: 2 },
              { anchor: [80, 20], leftDirection: [80, 20], rightDirection: [80, 20], pointType: 2 },
              { anchor: [20, 20], leftDirection: [20, 20], rightDirection: [20, 20], pointType: 2 }
            ]
          }
        ]
      }]
    }
  };

  const resultJSON = hostSandbox.extractArtboardDataJSON(JSON.stringify([0]));
  const results = JSON.parse(resultJSON);

  assert.strictEqual(results.length, 1);
  const elements = results[0].elements;
  assert.strictEqual(elements.length, 1);

  const el = elements[0];
  assert.strictEqual(el.isCompoundPath, true);
  assert.strictEqual(el.fillRule, null);
  assert.strictEqual(el.pathClosed, true);
  assert.strictEqual(el.subpaths.length, 2);

  assert(plugin.parityFindingsForElement(el).some(finding => finding.status === "unsupported" && finding.reason.includes("ai-parser fill_rule")));
  assert.throws(() => {
    plugin.exportFromRawData([{ artboard: { name: "Artboard 1", width: 100, height: 100 }, elements: [el] }], { naming: false, codeOnlyStrict: true });
  }, /ai-parser fill_rule/);

  assert.strictEqual(el.subpaths[0].points.length, 4);
  assert.deepStrictEqual(el.subpaths[0].points[0].anchor, [10, 10]);
  assert.deepStrictEqual(el.subpaths[0].points[1].anchor, [90, 10]);
  assert.deepStrictEqual(el.subpaths[0].points[2].anchor, [90, 90]);
  assert.deepStrictEqual(el.subpaths[0].points[3].anchor, [10, 90]);
  assert.deepStrictEqual(el.pathPoints[0].anchor, [10, 10]);
}

function testPluginCompoundPathFillRuleExtraction() {
  const pathPoint = (x, y) => ({ anchor: [x, y], leftDirection: [x, y], rightDirection: [x, y], pointType: 2 });
  const elements = [];
  sandbox.extractRecursive({
    typename: 'CompoundPathItem',
    name: 'plugin_compound_child_evenodd',
    locked: false,
    hidden: false,
    geometricBounds: [10, 90, 90, 10],
    pathItems: [{
      typename: 'PathItem',
      evenodd: true,
      closed: true,
      pathPoints: [pathPoint(10, 90), pathPoint(90, 90), pathPoint(90, 10), pathPoint(10, 10)]
    }, {
      typename: 'PathItem',
      closed: true,
      pathPoints: [pathPoint(20, 80), pathPoint(80, 80), pathPoint(80, 20), pathPoint(20, 20)]
    }]
  }, [0, 100, 100, 0], elements, 0);

  assert.strictEqual(elements.length, 1);
  assert.strictEqual(elements[0].isCompoundPath, true);
  assert.strictEqual(elements[0].fillRule, null);
  assert.strictEqual(elements[0].subpaths.length, 2);
  assert(plugin.parityFindingsForElement(elements[0]).some(finding => finding.reason.includes('ai-parser fill_rule')));
}

function testHostEmbeddedRasterExtractionPath() {
  const hostSource = fs.readFileSync(path.join(__dirname, 'host.jsx'), 'utf8');
  assert(hostSource.includes('extractEmbeddedRasterToTempPng'), 'Host should expose embedded raster extraction helper');
  assert(hostSource.includes('ExportOptionsPNG24'), 'Host should export embedded rasters through a temporary PNG input');

  const files = [];
  const hostSandbox = {
    Folder: function(folderPath) {
      this.fsName = folderPath;
      this.exists = true;
      this.create = function() { this.exists = true; return true; };
    },
    File: function(filePath) {
      this.fsName = filePath;
      this.exists = false;
      files.push(this);
    },
    ExportOptionsPNG24: function() {},
    ExportType: { PNG24: 'PNG24' },
    SaveOptions: { DONOTSAVECHANGES: 'DONOTSAVECHANGES' },
    DocumentColorSpace: { RGB: 'RGB' },
    ElementPlacement: { PLACEATEND: 'PLACEATEND' },
    PointType: { SMOOTH: 1, CORNER: 2 },
    Date: { now: () => 12345 }
  };
  hostSandbox.Folder.temp = new hostSandbox.Folder('/tmp');

  let exportedFile = null;
  let tempClosed = false;
  const tempDoc = {
    artboards: [{ artboardRect: [0, 20, 20, 0] }],
    layers: [{}],
    exportFile(file, type, options) {
      exportedFile = { file, type, options };
      file.exists = true;
    },
    close() { tempClosed = true; }
  };
  const rasterItem = {
    typename: 'RasterItem',
    name: 'raster 1',
    locked: false,
    hidden: false,
    parent: { typename: 'Layer' },
    geometricBounds: [10, 90, 30, 70],
    duplicate(target, placement) {
      assert.strictEqual(target, tempDoc.layers[0]);
      assert.strictEqual(placement, 'PLACEATEND');
      return {
        geometricBounds: [10, 90, 30, 70],
        translate(dx, dy) {
          assert.strictEqual(dx, -10);
          assert.strictEqual(dy, -70);
        }
      };
    }
  };
  hostSandbox.app = {
    documents: {
      length: 1,
      add(colorSpace, width, height) {
        assert.strictEqual(colorSpace, 'RGB');
        assert.strictEqual(width, 20);
        assert.strictEqual(height, 20);
        return tempDoc;
      }
    },
    activeDocument: {
      name: 'test.ai',
      artboards: [{ name: 'Artboard 1', artboardRect: [0, 100, 100, 0] }],
      pageItems: [rasterItem]
    }
  };

  vm.runInNewContext(hostSource, hostSandbox, { filename: 'host.jsx' });
  const results = JSON.parse(hostSandbox.extractArtboardDataJSON(JSON.stringify([0])));
  assert.strictEqual(results.length, 1);
  assert.strictEqual(results[0].elements.length, 1);
  const el = results[0].elements[0];
  assert.strictEqual(el.type, 'image');
  assert.strictEqual(el.embeddedRaster, true);
  assert(el.extractedImagePath.includes('/tmp/egui_expressive_raster_trace/raster_1_12345.png'));
  assert(el.notes.some(note => note.includes('embedded raster extracted')));
  assert.strictEqual(exportedFile.type, 'PNG24');
  assert.strictEqual(exportedFile.options.transparency, true);
  assert.strictEqual(tempClosed, true);
  assert(files.some(file => file.fsName === el.extractedImagePath));
}

function extractHostRasterElementWithXmp(xmp) {
  const hostSource = fs.readFileSync(path.join(__dirname, 'host.jsx'), 'utf8');
  const hostSandbox = {
    Folder: function(folderPath) { this.fsName = folderPath; this.exists = true; this.create = function() { this.exists = true; return true; }; },
    File: function(filePath) { this.fsName = filePath; this.exists = false; },
    ExportOptionsPNG24: function() {},
    ExportType: { PNG24: 'PNG24' },
    SaveOptions: { DONOTSAVECHANGES: 'DONOTSAVECHANGES' },
    DocumentColorSpace: { RGB: 'RGB' },
    ElementPlacement: { PLACEATEND: 'PLACEATEND' },
    PointType: { SMOOTH: 1, CORNER: 2 },
    Date: { now: () => 22222 }
  };
  hostSandbox.Folder.temp = new hostSandbox.Folder('/tmp');
  const tempDoc = {
    artboards: [{ artboardRect: [0, 20, 20, 0] }],
    layers: [{}],
    exportFile(file) { file.exists = true; },
    close() {}
  };
  const rasterItem = {
    typename: 'RasterItem',
    name: 'raster_xmp',
    locked: false,
    hidden: false,
    parent: { typename: 'Layer' },
    geometricBounds: [10, 90, 30, 70],
    XMPString: xmp,
    duplicate() { return { geometricBounds: [10, 90, 30, 70], translate() {} }; }
  };
  hostSandbox.app = {
    documents: { length: 1, add() { return tempDoc; } },
    activeDocument: {
      name: 'test.ai',
      artboards: [{ name: 'Artboard 1', artboardRect: [0, 100, 100, 0] }],
      pageItems: [rasterItem]
    }
  };
  vm.runInNewContext(hostSource, hostSandbox, { filename: 'host.jsx' });
  const results = JSON.parse(hostSandbox.extractArtboardDataJSON(JSON.stringify([0])));
  return results[0].elements[0];
}

function testExtractionDiagnostics() {
  const itemHiddenThrow = {
    get locked() { throw new Error("locked error"); },
    get hidden() { return false; }
  };
  const elements1 = [];
  sandbox.extractRecursive(itemHiddenThrow, [0, 100, 100, 0], elements1, 0);
  assert.strictEqual(elements1.length, 0);
  const diags1 = sandbox.consumeExtractionDiagnostics();
  assert(diags1.some(d => d.note.includes("skip hidden/locked state error") && d.note.includes("locked error")));

  const itemBoundsThrow = {
    locked: false,
    hidden: false,
    get geometricBounds() { throw new Error("geo error"); },
    get visibleBounds() { throw new Error("vis error"); }
  };
  const elements2 = [];
  sandbox.extractRecursive(itemBoundsThrow, [0, 100, 100, 0], elements2, 0);
  assert.strictEqual(elements2.length, 0);
  const diags2 = sandbox.consumeExtractionDiagnostics();
  assert(diags2.some(d => d.note.includes("skip bounds error") && d.note.includes("vis error")));
}

async function runTests() {
  testExtractionDiagnostics();
  testIllustratorRadialGradientGeometryExtraction();
  testHostCompoundPathExtraction();
  testPluginCompoundPathFillRuleExtraction();
  testHostEmbeddedRasterExtractionPath();
  testMergeParserDataPromotesOpaqueToGroup();
  testGradientOnlyVectorPaths();
  testPatternFillEmitsVectorPrimitive();
  testAppearanceBlendStackUsesSceneRenderer();
  testPortableAssetPath();
  testApplyBlendExpr();
  testGenerateSidecar();
  testBlendModeUsesSceneBuilder();
  testImageOpacityEmission();
  await testEmbeddedRasterVectorizationUsesExtractedPixels();
  await testRotatedAndEffectedRasterVectorization();
  await testCepRasterEffectsClassifyBeforeVectorization();
  testPathRichStrokeAndAppearanceEmission();
  testCompoundPathSceneEmission();
  testAiParserFillRulePropagation();
  testRichCircleAndStrokeOpacityEmission();
  testBundledParserCandidates();
  testMergeParserDataByBounds();
  testMergeParserDataAddsUnmatchedCodeDrawnVectors();
  testMergeParserDataPreservesHierarchyAndAppearance();
  testWarningsUsePortableImagePath();
  await testMixedClipRasterVectorizationRecovery();
  testTextUnitsOpacityAndParityStatus();
  testSpotTintPatternAndGraphicStyleParity();
  testSymbolDefinitionExpansionExtraction();
  testHostProgrammaticExpansionFallback();
  testHostTextShapingContractExtraction();
  testHostTextShapingOutlinedGlyphExtraction();
  testHostAllCapsDoesNotForceOutlinedShapingContract();
  testHostCanonicalTypographyFieldsExtraction();
  testTextShapingContractExportPath();
  testOutlinedGlyphsContractExportPath();
  testTextShapingStrictRejectsFontBytesWithoutOutlines();
  testTextShapingStrictRejectsStyledRunsWithContours();
  testTextShapingStrictRejectsEmptyContours();
  testTextShapingStrictRejectsPartialContourCoverage();
  testTextShapingStrictRejectsMixedContourSpaces();
  testParityStatusMarksUnsupportedSubset();
  testParserAndGradientStrokeParityStatus();
  testMultiAppearanceProbeBlocksFlattening();
  testStaticSecurityChecks();
  testIndexBootstrap();
  testAriaPressedToggle();
  testHostJsx();
  testFileTreeAndCodePreview();
  testHostSaveFailureHandling();
  testGenerateStateFileDerives();
  testStrictCodeOnlyExport();
  testLiveEffectDiagnosticsAreNamedAndHonest();
}

runTests().catch(err => {
  console.error(err);
  process.exit(1);
});
