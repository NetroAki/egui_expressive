// egui_expressive Token Exporter — Figma Plugin
// Exports Figma styles as JSON compatible with figma_tokens_to_rust

figma.showUI(__html__, { width: 480, height: 600, title: "egui_expressive Token Exporter" });

function colorToHex(color, opacity) {
  const r = Math.round(color.r * 255);
  const g = Math.round(color.g * 255);
  const b = Math.round(color.b * 255);
  const a = Math.round((opacity !== undefined ? opacity : 1.0) * 255);
  if (a === 255) {
    return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
  }
  return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}${a.toString(16).padStart(2, '0')}`;
}

function slugify(name) {
  return name.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_|_$/g, '');
}

function extractTokens() {
  const tokens = { global: {} };

  // --- Paint styles (colors) ---
  const paintStyles = figma.getLocalPaintStyles();
  for (const style of paintStyles) {
    const paint = style.paints[0];
    if (!paint || paint.type !== 'SOLID') continue;

    const nameParts = style.name.split('/').map(slugify);
    const group = nameParts.length > 1 ? nameParts.slice(0, -1).join('_') : 'colors';
    const key = nameParts[nameParts.length - 1];

    if (!tokens.global[group]) tokens.global[group] = {};
    tokens.global[group][key] = {
      value: colorToHex(paint.color, paint.opacity),
      type: 'color',
      description: style.description || ''
    };
  }

  // --- Text styles (typography) ---
  const textStyles = figma.getLocalTextStyles();
  if (textStyles.length > 0) {
    tokens.global['typography'] = {};
    for (const style of textStyles) {
      const key = slugify(style.name);
      tokens.global['typography'][key] = {
        value: {
          fontFamily: style.fontName.family,
          fontWeight: style.fontName.style,
          fontSize: `${style.fontSize}px`,
          lineHeight: style.lineHeight.unit === 'PIXELS'
            ? `${style.lineHeight.value}px`
            : style.lineHeight.unit === 'PERCENT'
            ? `${style.lineHeight.value}%`
            : 'normal',
          letterSpacing: style.letterSpacing.unit === 'PIXELS'
            ? `${style.letterSpacing.value}px`
            : `${style.letterSpacing.value}%`
        },
        type: 'typography',
        description: style.description || ''
      };
    }
  }

  // --- Effect styles (shadows) ---
  const effectStyles = figma.getLocalEffectStyles();
  if (effectStyles.length > 0) {
    tokens.global['effects'] = {};
    for (const style of effectStyles) {
      const key = slugify(style.name);
      const effects = style.effects.map(e => {
        if (e.type === 'DROP_SHADOW' || e.type === 'INNER_SHADOW') {
          return {
            type: e.type === 'DROP_SHADOW' ? 'dropShadow' : 'innerShadow',
            color: colorToHex(e.color, e.color.a),
            offsetX: `${e.offset.x}px`,
            offsetY: `${e.offset.y}px`,
            blur: `${e.radius}px`,
            spread: `${e.spread || 0}px`
          };
        }
        return null;
      }).filter(Boolean);

      if (effects.length > 0) {
        tokens.global['effects'][key] = {
          value: effects.length === 1 ? effects[0] : effects,
          type: 'boxShadow',
          description: style.description || ''
        };
      }
    }
  }

  // --- Spacing from local variables (if any) ---
  // Try to extract spacing variables
  try {
    const collections = figma.variables.getLocalVariableCollections();
    for (const collection of collections) {
      const collectionKey = slugify(collection.name);
      for (const varId of collection.variableIds) {
        const variable = figma.variables.getVariableById(varId);
        if (!variable) continue;
        if (variable.resolvedType !== 'FLOAT' && variable.resolvedType !== 'COLOR') continue;

        const nameParts = variable.name.split('/').map(slugify);
        const group = nameParts.length > 1
          ? `${collectionKey}_${nameParts.slice(0, -1).join('_')}`
          : collectionKey;
        const key = nameParts[nameParts.length - 1];

        if (!tokens.global[group]) tokens.global[group] = {};

        // Get value from default mode
        const modeId = collection.defaultModeId;
        const rawValue = variable.valuesByMode[modeId];

        if (variable.resolvedType === 'FLOAT' && typeof rawValue === 'number') {
          tokens.global[group][key] = {
            value: `${rawValue}px`,
            type: 'spacing',
            description: variable.description || ''
          };
        } else if (variable.resolvedType === 'COLOR' && rawValue && typeof rawValue === 'object' && 'r' in rawValue) {
          tokens.global[group][key] = {
            value: colorToHex(rawValue, rawValue.a),
            type: 'color',
            description: variable.description || ''
          };
        }
      }
    }
  } catch (e) {
    // Variables API may not be available in all contexts
  }

  return tokens;
}

figma.ui.onmessage = (msg) => {
  if (msg.type === 'export') {
    try {
      const tokens = extractTokens();
      const json = JSON.stringify(tokens, null, 2);
      figma.ui.postMessage({ type: 'result', json, error: null });
    } catch (e) {
      figma.ui.postMessage({ type: 'result', json: null, error: String(e) });
    }
  } else if (msg.type === 'close') {
    figma.closePlugin();
  }
};

// Auto-export on open
const tokens = extractTokens();
const json = JSON.stringify(tokens, null, 2);
figma.ui.postMessage({ type: 'result', json, error: null });
