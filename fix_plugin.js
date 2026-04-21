const fs = require('fs');
let content = fs.readFileSync('illustrator-plugin/plugin.js', 'utf8');

let startIdx = content.indexOf('function generateElementCode(el, indent, colorMap, comps) {');
let endIdx = content.indexOf('function isHorizontal(children) {');

let before = content.substring(0, startIdx);
let func = content.substring(startIdx, endIdx);
let after = content.substring(endIdx);

func = func.replace(/fmtF32\(Math\.round\((el\.[xywh])\)\)/g, 'fmtF32($1)');
func = func.replace(/Math\.round\(el\.x \+ el\.w \/ 2\)/g, 'el.x + el.w / 2');
func = func.replace(/Math\.round\(el\.y \+ el\.h \/ 2\)/g, 'el.y + el.h / 2');
func = func.replace(/Math\.round\(Math\.min\(el\.w, el\.h\) \/ 2\)/g, 'Math.min(el.w, el.h) / 2');
func = func.replace(/Math\.round\(p0\[0\]\)/g, 'p0[0]');
func = func.replace(/Math\.round\(p0\[1\]\)/g, 'p0[1]');
func = func.replace(/Math\.round\(p1\[0\]\)/g, 'p1[0]');
func = func.replace(/Math\.round\(p1\[1\]\)/g, 'p1[1]');
func = func.replace(/Math\.round\(p\.anchor\[0\]\)/g, 'p.anchor[0]');
func = func.replace(/Math\.round\(p\.anchor\[1\]\)/g, 'p.anchor[1]');
func = func.replace(/Math\.round\(el\.y \+ el\.h\/2\)/g, 'el.y + el.h/2');
func = func.replace(/Math\.round\(el\.x \+ el\.w\)/g, 'el.x + el.w');

fs.writeFileSync('illustrator-plugin/plugin.js', before + func + after);
