use crate::render::RenderQuality;
use crate::scene::PathContour;
use egui::{Color32, Context, FontFamily, FontId, RichText};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextDecoration {
    #[default]
    None,
    Underline,
    Strikethrough,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextOverflow {
    #[default]
    Visible,
    Ellipsis,
    Clip,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextTransform {
    #[default]
    None,
    Uppercase,
    Lowercase,
    Capitalize,
    SmallCaps,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpenTypeFeatures {
    pub ligatures: bool,
    pub contextual_ligatures: bool,
    pub discretionary_ligatures: bool,
    pub fractions: bool,
    pub ordinals: bool,
    pub swash: bool,
    pub titling_alternates: bool,
    pub stylistic_alternates: bool,
    pub kerning: bool,
}

impl OpenTypeFeatures {
    pub fn all_off() -> Self {
        Self {
            ligatures: false,
            contextual_ligatures: false,
            discretionary_ligatures: false,
            fractions: false,
            ordinals: false,
            swash: false,
            titling_alternates: false,
            stylistic_alternates: false,
            kerning: false,
        }
    }

    pub fn can_use_fast_path(&self) -> bool {
        *self == Self::default()
    }
}

impl Default for OpenTypeFeatures {
    fn default() -> Self {
        Self {
            ligatures: true,
            contextual_ligatures: true,
            discretionary_ligatures: false,
            fractions: false,
            ordinals: false,
            swash: false,
            titling_alternates: false,
            stylistic_alternates: false,
            kerning: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShapedGlyph {
    #[serde(alias = "glyphId")]
    pub glyph_id: u32,
    pub cluster: u32,
    #[serde(alias = "advanceX")]
    pub advance_x: f32,
    #[serde(alias = "advanceY")]
    pub advance_y: f32,
    #[serde(alias = "offsetX")]
    pub offset_x: f32,
    #[serde(alias = "offsetY")]
    pub offset_y: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contours: Vec<PathContour>,
    #[serde(default, alias = "contoursAbsolute")]
    pub contours_are_absolute: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapedGlyphRun {
    pub text: String,
    pub glyphs: Vec<ShapedGlyph>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontFaceId(String);

impl FontFaceId {
    pub fn new(id: impl Into<String>) -> Result<Self, FontSelectionIssue> {
        let id = trimmed_non_empty(id, FontSelectionIssueKind::InvalidMetadata, "font face id")?;
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum FontStyleKind {
    #[default]
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontStretch(u16);

impl FontStretch {
    pub fn new(value: u16) -> Result<Self, FontSelectionIssue> {
        if value == 0 {
            return Err(FontSelectionIssue::new(
                FontSelectionIssueKind::InvalidMetadata,
                "font stretch must be positive",
            ));
        }
        Ok(Self(value))
    }

    pub fn value(self) -> u16 {
        self.0
    }
}

impl Default for FontStretch {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontCoverageRange {
    pub start: char,
    pub end: char,
}

impl FontCoverageRange {
    pub fn new(start: char, end: char) -> Result<Self, FontSelectionIssue> {
        if start > end {
            return Err(FontSelectionIssue::new(
                FontSelectionIssueKind::InvalidMetadata,
                "font coverage range start must not exceed end",
            ));
        }
        Ok(Self { start, end })
    }

    fn contains(self, ch: char) -> bool {
        self.start <= ch && ch <= self.end
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontFaceRecord {
    pub family_name: String,
    pub face_id: FontFaceId,
    pub weight: u16,
    pub style: FontStyleKind,
    pub stretch: FontStretch,
    pub fallback_order: u16,
    pub license_id: String,
    pub provenance_id: String,
    pub coverage: Vec<FontCoverageRange>,
    pub font_bytes: Option<Arc<[u8]>>,
    pub bytes_sha256: Option<String>,
}

impl FontFaceRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        family_name: impl Into<String>,
        face_id: FontFaceId,
        weight: u16,
        style: FontStyleKind,
        stretch: FontStretch,
        fallback_order: u16,
        license_id: impl Into<String>,
        provenance_id: impl Into<String>,
        coverage: Vec<FontCoverageRange>,
        font_bytes: Option<Arc<[u8]>>,
        bytes_sha256: Option<String>,
    ) -> Result<Self, FontSelectionIssue> {
        let family_name = trimmed_non_empty(
            family_name,
            FontSelectionIssueKind::InvalidMetadata,
            "font family name",
        )?;
        let license_id = trimmed_non_empty(
            license_id,
            FontSelectionIssueKind::MissingLicense,
            "font license id",
        )?;
        let provenance_id = trimmed_non_empty(
            provenance_id,
            FontSelectionIssueKind::InvalidMetadata,
            "font provenance id",
        )?;

        Ok(Self {
            family_name,
            face_id,
            weight: weight.clamp(1, 1000),
            style,
            stretch,
            fallback_order,
            license_id,
            provenance_id,
            coverage,
            font_bytes,
            bytes_sha256: bytes_sha256.filter(|hash| !hash.trim().is_empty()),
        })
    }

    fn covers_text(&self, text: &str) -> bool {
        if self.coverage.is_empty() {
            return false;
        }
        text.chars()
            .all(|ch| self.coverage.iter().any(|range| range.contains(ch)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontFamilyRecord {
    pub family_name: String,
    pub aliases: Vec<String>,
    pub fallback_families: Vec<String>,
    pub faces: Vec<FontFaceRecord>,
}

impl FontFamilyRecord {
    pub fn new(
        family_name: impl Into<String>,
        aliases: Vec<String>,
        fallback_families: Vec<String>,
        faces: Vec<FontFaceRecord>,
    ) -> Result<Self, FontSelectionIssue> {
        let family_name = trimmed_non_empty(
            family_name,
            FontSelectionIssueKind::InvalidMetadata,
            "font family name",
        )?;
        for face in &faces {
            if normalize_family(&face.family_name) != normalize_family(&family_name) {
                return Err(FontSelectionIssue::new(
                    FontSelectionIssueKind::InvalidMetadata,
                    "font face family must match family record",
                ));
            }
        }
        Ok(Self {
            family_name,
            aliases: aliases.into_iter().filter_map(non_empty_string).collect(),
            fallback_families: fallback_families
                .into_iter()
                .filter_map(non_empty_string)
                .collect(),
            faces,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontSelectionIssueKind {
    MissingFamily,
    MissingFace,
    MissingGlyph,
    MissingLicense,
    LicenseUnapproved,
    InvalidMetadata,
    FallbackUsed,
    WeightSubstituted,
    NoFontBytes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontSelectionIssue {
    pub kind: FontSelectionIssueKind,
    pub message: &'static str,
}

impl FontSelectionIssue {
    pub fn new(kind: FontSelectionIssueKind, message: &'static str) -> Self {
        Self { kind, message }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontSelectionReport {
    pub requested_family: Option<String>,
    pub requested_weight: u16,
    pub requested_style: FontStyleKind,
    pub requested_stretch: FontStretch,
    pub requested_text: String,
    pub selected_family: Option<String>,
    pub selected_face_id: Option<FontFaceId>,
    pub selected_weight: Option<u16>,
    pub selected_style: Option<FontStyleKind>,
    pub selected_stretch: Option<FontStretch>,
    pub actual_quality: RenderQuality,
    pub issues: Vec<FontSelectionIssue>,
    pub fallback_chain: Vec<String>,
}

impl FontSelectionReport {
    pub fn is_exact(&self) -> bool {
        self.actual_quality == RenderQuality::Exact && self.issues.is_empty()
    }

    fn empty(
        requested_family: Option<String>,
        requested_weight: u16,
        requested_style: FontStyleKind,
        requested_stretch: FontStretch,
        requested_text: &str,
    ) -> Self {
        Self {
            requested_family,
            requested_weight,
            requested_style,
            requested_stretch,
            requested_text: requested_text.to_owned(),
            selected_family: None,
            selected_face_id: None,
            selected_weight: None,
            selected_style: None,
            selected_stretch: None,
            actual_quality: RenderQuality::Unsupported,
            issues: Vec::new(),
            fallback_chain: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontRegistry {
    families: BTreeMap<String, FontFamilyRecord>,
    aliases: BTreeMap<String, String>,
    approved_license_ids: BTreeSet<String>,
}

impl FontRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_approved_license(mut self, license_id: impl Into<String>) -> Self {
        if let Some(license_id) = non_empty_string(license_id.into()) {
            self.approved_license_ids.insert(license_id);
        }
        self
    }

    pub fn add_family(mut self, family: FontFamilyRecord) -> Self {
        let key = normalize_family(&family.family_name);
        for alias in &family.aliases {
            self.aliases.insert(normalize_family(alias), key.clone());
        }
        self.aliases.insert(key.clone(), key.clone());
        self.families.insert(key, family);
        self
    }

    pub fn resolve(
        &self,
        requested_family: Option<&str>,
        requested_weight: u16,
        requested_style: FontStyleKind,
        requested_stretch: FontStretch,
        text: &str,
    ) -> FontSelectionReport {
        let requested_family = requested_family.and_then(non_empty_string);
        let mut report = FontSelectionReport::empty(
            requested_family.clone(),
            requested_weight,
            requested_style,
            requested_stretch,
            text,
        );
        let Some(request_key) = requested_family
            .as_deref()
            .and_then(|family| self.resolve_family_key(family))
        else {
            report.issues.push(FontSelectionIssue::new(
                FontSelectionIssueKind::MissingFamily,
                "requested font family is not registered",
            ));
            return report;
        };

        let mut family_keys = vec![request_key.clone()];
        if let Some(family) = self.families.get(&request_key) {
            for fallback in &family.fallback_families {
                if let Some(key) = self.resolve_family_key(fallback) {
                    if !family_keys.contains(&key) {
                        family_keys.push(key);
                    }
                }
            }
        }

        let mut saw_family_face = false;
        let mut saw_missing_glyph = false;
        for family_key in family_keys {
            let Some(family) = self.families.get(&family_key) else {
                continue;
            };
            report.fallback_chain.push(family.family_name.clone());
            let Some(face) = best_face(
                &family.faces,
                requested_weight,
                requested_style,
                requested_stretch,
            ) else {
                continue;
            };
            saw_family_face = true;
            if !face.covers_text(text) {
                saw_missing_glyph = true;
                continue;
            }

            report.selected_family = Some(family.family_name.clone());
            report.selected_face_id = Some(face.face_id.clone());
            report.selected_weight = Some(face.weight);
            report.selected_style = Some(face.style);
            report.selected_stretch = Some(face.stretch);
            if normalize_family(&family.family_name) != request_key {
                report.issues.push(FontSelectionIssue::new(
                    FontSelectionIssueKind::FallbackUsed,
                    "font fallback family was selected",
                ));
            }
            if face.weight != requested_weight {
                report.issues.push(FontSelectionIssue::new(
                    FontSelectionIssueKind::WeightSubstituted,
                    "nearest registered font weight was selected",
                ));
            }
            if face.license_id.trim().is_empty() {
                report.issues.push(FontSelectionIssue::new(
                    FontSelectionIssueKind::MissingLicense,
                    "font face has no license id",
                ));
            } else if !self.approved_license_ids.contains(&face.license_id) {
                report.issues.push(FontSelectionIssue::new(
                    FontSelectionIssueKind::LicenseUnapproved,
                    "font license id is not in the approved registry set",
                ));
            }
            if face.provenance_id.trim().is_empty() {
                report.issues.push(FontSelectionIssue::new(
                    FontSelectionIssueKind::InvalidMetadata,
                    "font face has no provenance id",
                ));
            }
            if face.font_bytes.is_none() {
                report.issues.push(FontSelectionIssue::new(
                    FontSelectionIssueKind::NoFontBytes,
                    "font face has no app-provided bytes",
                ));
            }
            report.actual_quality = font_report_quality(&report.issues);
            return report;
        }

        report.issues.push(FontSelectionIssue::new(
            if saw_family_face {
                FontSelectionIssueKind::MissingGlyph
            } else {
                FontSelectionIssueKind::MissingFace
            },
            if saw_missing_glyph {
                "registered font faces do not cover the requested text"
            } else {
                "no registered font face matches the requested style and stretch"
            },
        ));
        report
    }

    fn resolve_family_key(&self, family: &str) -> Option<String> {
        let normalized = normalize_family(family);
        self.aliases.get(&normalized).cloned().or_else(|| {
            self.families
                .contains_key(&normalized)
                .then_some(normalized)
        })
    }
}

fn best_face(
    faces: &[FontFaceRecord],
    requested_weight: u16,
    requested_style: FontStyleKind,
    requested_stretch: FontStretch,
) -> Option<&FontFaceRecord> {
    faces
        .iter()
        .filter(|face| face.style == requested_style && face.stretch == requested_stretch)
        .min_by_key(|face| {
            let distance = face.weight.abs_diff(requested_weight);
            (
                distance,
                face.weight,
                face.fallback_order,
                face.face_id.as_str(),
            )
        })
}

fn font_report_quality(issues: &[FontSelectionIssue]) -> RenderQuality {
    if issues.iter().any(|issue| {
        matches!(
            issue.kind,
            FontSelectionIssueKind::MissingFamily
                | FontSelectionIssueKind::MissingFace
                | FontSelectionIssueKind::MissingGlyph
                | FontSelectionIssueKind::MissingLicense
                | FontSelectionIssueKind::LicenseUnapproved
                | FontSelectionIssueKind::InvalidMetadata
                | FontSelectionIssueKind::NoFontBytes
        )
    }) {
        RenderQuality::Unsupported
    } else if issues.is_empty() {
        RenderQuality::Exact
    } else {
        RenderQuality::Approximate
    }
}

fn trimmed_non_empty(
    value: impl Into<String>,
    kind: FontSelectionIssueKind,
    field: &'static str,
) -> Result<String, FontSelectionIssue> {
    non_empty_string(value.into()).ok_or_else(|| FontSelectionIssue::new(kind, field))
}

fn non_empty_string(value: impl AsRef<str>) -> Option<String> {
    let value = value.as_ref().trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn normalize_family(family: &str) -> String {
    family.trim().to_ascii_lowercase()
}

pub fn shaped_glyph_run_advance_width(run: &ShapedGlyphRun, spec: &TypeSpec) -> f32 {
    run.glyphs
        .iter()
        .map(|glyph| glyph.advance_x.max(0.0) * spec.horizontal_scale)
        .sum()
}

#[derive(Clone, Debug)]
pub struct TypeSpec {
    pub size: f32,
    pub weight: u16,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub color: Option<Color32>,
    pub font_family: Option<String>,
    pub decoration: TextDecoration,
    pub overflow: TextOverflow,
    pub text_transform: TextTransform,
    pub open_type_features: OpenTypeFeatures,
    pub baseline_shift: f32,
    pub horizontal_scale: f32,
    pub vertical_scale: f32,
}

impl TypeSpec {
    pub fn new(size: f32) -> Self {
        Self {
            size,
            weight: 400,
            line_height: 1.4,
            letter_spacing: 0.0,
            color: None,
            font_family: None,
            decoration: TextDecoration::None,
            overflow: TextOverflow::Visible,
            text_transform: TextTransform::None,
            open_type_features: OpenTypeFeatures::default(),
            baseline_shift: 0.0,
            horizontal_scale: 1.0,
            vertical_scale: 1.0,
        }
    }

    pub fn micro_label() -> Self {
        Self::new(9.0)
            .weight(700)
            .letter_spacing(1.2)
            .line_height(1.0)
            .text_transform(TextTransform::Uppercase)
    }

    pub fn mono_readout(size: f32) -> Self {
        Self::new(size)
            .weight(600)
            .letter_spacing(0.2)
            .font_family("mono")
    }

    pub fn weight(mut self, w: u16) -> Self {
        self.weight = w;
        self
    }

    pub fn line_height(mut self, lh: f32) -> Self {
        self.line_height = lh;
        self
    }

    pub fn letter_spacing(mut self, ls: f32) -> Self {
        self.letter_spacing = ls;
        self
    }

    pub fn color(mut self, c: Color32) -> Self {
        self.color = Some(c);
        self
    }

    pub fn font_family(mut self, f: impl Into<String>) -> Self {
        self.font_family = Some(f.into());
        self
    }

    pub fn decoration(mut self, d: TextDecoration) -> Self {
        self.decoration = d;
        self
    }

    pub fn overflow(mut self, o: TextOverflow) -> Self {
        self.overflow = o;
        self
    }

    pub fn text_transform(mut self, t: TextTransform) -> Self {
        self.text_transform = t;
        self
    }

    pub fn open_type_features(mut self, otf: OpenTypeFeatures) -> Self {
        self.open_type_features = otf;
        self
    }

    pub fn ligatures(mut self, on: bool) -> Self {
        self.open_type_features.ligatures = on;
        self
    }

    pub fn baseline_shift(mut self, px: f32) -> Self {
        self.baseline_shift = px;
        self
    }

    pub fn horizontal_scale(mut self, s: f32) -> Self {
        self.horizontal_scale = s;
        self
    }

    pub fn vertical_scale(mut self, s: f32) -> Self {
        self.vertical_scale = s;
        self
    }

    pub fn effective_size(&self) -> f32 {
        self.size * self.vertical_scale
    }

    pub fn requires_full_shaper(&self) -> bool {
        !self.open_type_features.can_use_fast_path()
            || self.text_transform == TextTransform::SmallCaps
            || self.horizontal_scale != 1.0
            || self.vertical_scale != 1.0
            || self.baseline_shift != 0.0
            || self.letter_spacing != 0.0
    }

    pub(super) fn can_use_fast_path(&self) -> bool {
        self.letter_spacing == 0.0
            && self.text_transform != TextTransform::SmallCaps
            && self.open_type_features.can_use_fast_path()
            && self.horizontal_scale == 1.0
            && self.vertical_scale == 1.0
            && self.baseline_shift == 0.0
    }

    pub fn to_font_id(&self) -> FontId {
        let family = self
            .font_family
            .as_deref()
            .map(font_family_from_alias)
            .unwrap_or(FontFamily::Proportional);
        FontId::new(self.effective_size(), family)
    }

    pub fn resolve_font(&self, registry: &FontRegistry, text: &str) -> FontSelectionReport {
        registry.resolve(
            self.font_family.as_deref(),
            self.weight,
            FontStyleKind::Normal,
            FontStretch::default(),
            text,
        )
    }

    pub fn to_rich_text(&self, text: &str) -> RichText {
        let rich_text = RichText::new(text).size(self.effective_size());
        let rich_text = match &self.font_family {
            Some(f) => rich_text.font(FontId::new(
                self.effective_size(),
                font_family_from_alias(f),
            )),
            None => rich_text,
        };

        let rich_text = match self.weight {
            100..=300 => rich_text.weak(),
            400..=500 => rich_text,
            600..=900 => rich_text.strong(),
            _ => rich_text,
        };

        match self.color {
            Some(c) => rich_text.color(c),
            None => rich_text,
        }
    }
}

fn font_family_from_alias(name: &str) -> FontFamily {
    match name.trim().to_ascii_lowercase().as_str() {
        "mono" | "monospace" => FontFamily::Monospace,
        "sans" | "proportional" => FontFamily::Proportional,
        _ => FontFamily::Name(name.to_owned().into()),
    }
}

#[cfg(test)]
mod phase8_tests {
    use super::*;
    use std::sync::Arc;

    fn latin_range() -> Vec<FontCoverageRange> {
        vec![FontCoverageRange::new(' ', '~').unwrap()]
    }

    fn face(weight: u16, id: &str, order: u16) -> FontFaceRecord {
        FontFaceRecord::new(
            "Inter",
            FontFaceId::new(id).unwrap(),
            weight,
            FontStyleKind::Normal,
            FontStretch::default(),
            order,
            "OFL-1.1",
            "stage5-fixture",
            latin_range(),
            Some(Arc::from([1_u8, 2, 3])),
            Some(format!("sha-{id}")),
        )
        .unwrap()
    }

    fn inter_registry(faces: Vec<FontFaceRecord>) -> FontRegistry {
        FontRegistry::new()
            .with_approved_license("OFL-1.1")
            .add_family(
                FontFamilyRecord::new("Inter", vec!["ui".into()], Vec::new(), faces).unwrap(),
            )
    }

    #[test]
    fn builtin_family_aliases_map_to_egui_builtin_families() {
        assert!(matches!(
            TypeSpec::new(13.0).font_family("mono").to_font_id().family,
            FontFamily::Monospace
        ));
        assert!(matches!(
            TypeSpec::new(13.0)
                .font_family("monospace")
                .to_font_id()
                .family,
            FontFamily::Monospace
        ));
        assert!(matches!(
            TypeSpec::new(13.0).font_family("sans").to_font_id().family,
            FontFamily::Proportional
        ));
        assert!(matches!(
            TypeSpec::new(13.0)
                .font_family("proportional")
                .to_font_id()
                .family,
            FontFamily::Proportional
        ));
    }

    #[test]
    fn custom_family_names_stay_named_families() {
        let id = TypeSpec::new(13.0).font_family("Custom UI").to_font_id();
        assert!(matches!(id.family, FontFamily::Name(_)));
    }

    #[test]
    fn mono_readout_uses_builtin_monospace_alias() {
        assert!(matches!(
            TypeSpec::mono_readout(12.0).to_font_id().family,
            FontFamily::Monospace
        ));
    }

    #[test]
    fn r100_005a_type_spec_rich_text_uses_bounded_weight_emphasis() {
        let weak = format!("{:?}", TypeSpec::new(14.0).weight(300).to_rich_text("thin"));
        assert!(weak.contains("weak: true"));

        let normal = format!(
            "{:?}",
            TypeSpec::new(14.0).weight(500).to_rich_text("normal")
        );
        assert!(!normal.contains("weak: true"));
        assert!(!normal.contains("strong: true"));

        let strong = format!(
            "{:?}",
            TypeSpec::new(14.0).weight(600).to_rich_text("strong")
        );
        assert!(strong.contains("strong: true"));
    }

    #[test]
    fn r100_005a_type_spec_font_id_remains_weight_agnostic() {
        let light = TypeSpec::new(13.0).weight(300).to_font_id();
        let bold = TypeSpec::new(13.0).weight(800).to_font_id();

        assert_eq!(light.size, bold.size);
        assert_eq!(light.family, bold.family);
    }

    #[test]
    fn stage5_font_registry_selects_exact_registered_weight() {
        let registry = inter_registry(vec![face(400, "regular", 0), face(700, "bold", 1)]);

        let report = TypeSpec::new(14.0)
            .font_family("Inter")
            .weight(700)
            .resolve_font(&registry, "Hello");

        assert!(report.is_exact());
        assert_eq!(report.selected_face_id.unwrap().as_str(), "bold");
        assert_eq!(report.selected_weight, Some(700));
    }

    #[test]
    fn stage5_font_registry_selects_nearest_weight_with_lower_tie_break() {
        let registry = inter_registry(vec![face(400, "regular", 0), face(600, "semi", 1)]);

        let report = TypeSpec::new(14.0)
            .font_family("Inter")
            .weight(500)
            .resolve_font(&registry, "Hello");

        assert_eq!(report.actual_quality, RenderQuality::Approximate);
        assert_eq!(report.selected_face_id.unwrap().as_str(), "regular");
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.kind == FontSelectionIssueKind::WeightSubstituted));
    }

    #[test]
    fn stage5_font_registry_reports_missing_family_and_face() {
        let registry = inter_registry(vec![face(400, "regular", 0)]);
        let missing_family = TypeSpec::new(14.0)
            .font_family("Missing")
            .resolve_font(&registry, "Hello");
        assert_eq!(missing_family.actual_quality, RenderQuality::Unsupported);
        assert_eq!(
            missing_family.issues[0].kind,
            FontSelectionIssueKind::MissingFamily
        );

        let italic = registry.resolve(
            Some("Inter"),
            400,
            FontStyleKind::Italic,
            FontStretch::default(),
            "Hello",
        );
        assert_eq!(italic.actual_quality, RenderQuality::Unsupported);
        assert_eq!(italic.issues[0].kind, FontSelectionIssueKind::MissingFace);
    }

    #[test]
    fn stage5_font_registry_reports_missing_glyph_license_and_bytes() {
        let glyph_limited = FontFaceRecord::new(
            "Inter",
            FontFaceId::new("limited").unwrap(),
            400,
            FontStyleKind::Normal,
            FontStretch::default(),
            0,
            "OFL-1.1",
            "stage5-fixture",
            vec![FontCoverageRange::new('A', 'Z').unwrap()],
            Some(Arc::from([1_u8])),
            None,
        )
        .unwrap();
        let registry = inter_registry(vec![glyph_limited]);
        let missing_glyph = TypeSpec::new(14.0)
            .font_family("Inter")
            .resolve_font(&registry, "hello");
        assert_eq!(
            missing_glyph.issues[0].kind,
            FontSelectionIssueKind::MissingGlyph
        );

        let missing_license = FontFaceRecord::new(
            "Inter",
            FontFaceId::new("bad").unwrap(),
            400,
            FontStyleKind::Normal,
            FontStretch::default(),
            0,
            " ",
            "stage5-fixture",
            latin_range(),
            Some(Arc::from([1_u8])),
            None,
        )
        .unwrap_err();
        assert_eq!(missing_license.kind, FontSelectionIssueKind::MissingLicense);

        let no_bytes = FontFaceRecord::new(
            "Inter",
            FontFaceId::new("metadata-only").unwrap(),
            400,
            FontStyleKind::Normal,
            FontStretch::default(),
            0,
            "OFL-1.1",
            "stage5-fixture",
            latin_range(),
            None,
            None,
        )
        .unwrap();
        let no_bytes_report = inter_registry(vec![no_bytes]).resolve(
            Some("Inter"),
            400,
            FontStyleKind::Normal,
            FontStretch::default(),
            "Hello",
        );
        assert_eq!(no_bytes_report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(
            no_bytes_report.issues[0].kind,
            FontSelectionIssueKind::NoFontBytes
        );

        let unknown_coverage = FontFaceRecord::new(
            "Inter",
            FontFaceId::new("unknown-coverage").unwrap(),
            400,
            FontStyleKind::Normal,
            FontStretch::default(),
            0,
            "OFL-1.1",
            "stage5-fixture",
            Vec::new(),
            Some(Arc::from([1_u8])),
            None,
        )
        .unwrap();
        let unknown_coverage_report = inter_registry(vec![unknown_coverage]).resolve(
            Some("Inter"),
            400,
            FontStyleKind::Normal,
            FontStretch::default(),
            "",
        );
        assert_eq!(
            unknown_coverage_report.actual_quality,
            RenderQuality::Unsupported
        );
        assert_eq!(
            unknown_coverage_report.issues[0].kind,
            FontSelectionIssueKind::MissingGlyph
        );
    }

    #[test]
    fn stage5_font_registry_reports_manually_invalid_public_metadata() {
        let mut face = face(400, "manual-invalid", 0);
        face.license_id.clear();
        face.provenance_id.clear();

        let report = inter_registry(vec![face]).resolve(
            Some("Inter"),
            400,
            FontStyleKind::Normal,
            FontStretch::default(),
            "Hello",
        );

        assert_eq!(report.actual_quality, RenderQuality::Unsupported);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.kind == FontSelectionIssueKind::MissingLicense));
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.kind == FontSelectionIssueKind::InvalidMetadata));
    }

    #[test]
    fn stage5_font_registry_reports_unapproved_license_and_fallback_order() {
        let unapproved = FontFaceRecord::new(
            "Inter",
            FontFaceId::new("commercial").unwrap(),
            400,
            FontStyleKind::Normal,
            FontStretch::default(),
            0,
            "Commercial-Internal",
            "stage5-fixture",
            latin_range(),
            Some(Arc::from([1_u8])),
            None,
        )
        .unwrap();
        let unapproved_report = FontRegistry::new()
            .with_approved_license("OFL-1.1")
            .add_family(
                FontFamilyRecord::new("Inter", Vec::new(), Vec::new(), vec![unapproved]).unwrap(),
            )
            .resolve(
                Some("Inter"),
                400,
                FontStyleKind::Normal,
                FontStretch::default(),
                "Hello",
            );
        assert_eq!(unapproved_report.actual_quality, RenderQuality::Unsupported);
        assert_eq!(
            unapproved_report.issues[0].kind,
            FontSelectionIssueKind::LicenseUnapproved
        );

        let fallback_registry = FontRegistry::new()
            .with_approved_license("OFL-1.1")
            .add_family(
                FontFamilyRecord::new("Display", Vec::new(), vec!["Inter".into()], Vec::new())
                    .unwrap(),
            )
            .add_family(
                FontFamilyRecord::new(
                    "Inter",
                    Vec::new(),
                    Vec::new(),
                    vec![face(400, "regular", 0)],
                )
                .unwrap(),
            );
        let fallback = TypeSpec::new(14.0)
            .font_family("Display")
            .resolve_font(&fallback_registry, "Hello");
        assert_eq!(fallback.actual_quality, RenderQuality::Approximate);
        assert_eq!(fallback.selected_family.as_deref(), Some("Inter"));
        assert_eq!(fallback.fallback_chain, vec!["Display", "Inter"]);
        assert!(fallback
            .issues
            .iter()
            .any(|issue| issue.kind == FontSelectionIssueKind::FallbackUsed));
    }
}

impl Default for TypeSpec {
    fn default() -> Self {
        TypeSpec::new(14.0)
    }
}

/// A type scale with named presets matching common design-system conventions.
#[derive(Clone, Debug)]
pub struct TypeScale {
    pub display: TypeSpec,
    pub headline: TypeSpec,
    pub title_lg: TypeSpec,
    pub title_md: TypeSpec,
    pub title_sm: TypeSpec,
    pub body_lg: TypeSpec,
    pub body_md: TypeSpec,
    pub body_sm: TypeSpec,
    pub label_lg: TypeSpec,
    pub label_md: TypeSpec,
    pub label_sm: TypeSpec,
    pub mono: TypeSpec,
}

impl Default for TypeScale {
    fn default() -> Self {
        Self {
            display: TypeSpec::new(57.0),
            headline: TypeSpec::new(32.0),
            title_lg: TypeSpec::new(22.0),
            title_md: TypeSpec::new(16.0).weight(500),
            title_sm: TypeSpec::new(14.0).weight(500),
            body_lg: TypeSpec::new(16.0),
            body_md: TypeSpec::new(14.0),
            body_sm: TypeSpec::new(12.0),
            label_lg: TypeSpec::new(14.0).weight(500),
            label_md: TypeSpec::new(12.0).weight(500),
            label_sm: TypeSpec::new(11.0).weight(500),
            mono: TypeSpec::new(13.0).font_family("mono"),
        }
    }
}

impl TypeScale {
    const STORE_ID: &'static str = "egui_expressive_type_scale";

    /// Stores this type scale in egui's context.
    pub fn store(&self, ctx: &Context) {
        ctx.data_mut(|data| data.insert_temp(egui::Id::new(Self::STORE_ID), self.clone()));
    }

    /// Loads the type scale from egui's context, falling back to the default scale.
    pub fn load(ctx: &Context) -> Self {
        ctx.data(|data| {
            data.get_temp(egui::Id::new(Self::STORE_ID))
                .unwrap_or_else(Self::default)
        })
    }
}
