//! Illustrator/AI parser CLI.

#[allow(unused_imports)]
use egui_expressive::codegen::{
    generate_artboard_file, parse_svg_elements, AppearanceFill, AppearanceStroke, ArtboardInfo,
    BlendMode, EffectDef, EffectType, ElementType, GradientDef, GradientStop, GradientType,
    LayoutElement, StrokeCap, StrokeJoin,
};
#[allow(unused_imports)]
use lopdf::Document;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

static ARTBOARD_RE: OnceLock<Regex> = OnceLock::new();
static ARTBOARD_NAME_RE: OnceLock<Regex> = OnceLock::new();

#[path = "ai_parser/convert.rs"]
mod convert;
#[path = "ai_parser/entry.rs"]
mod entry;
#[path = "ai_parser/output.rs"]
mod output;
#[path = "ai_parser/parse_file.rs"]
mod parse_file;
#[path = "ai_parser/parsing.rs"]
mod parsing;
#[path = "ai_parser/pdf.rs"]
mod pdf;
#[path = "ai_parser/types.rs"]
mod types;

#[cfg(test)]
#[path = "ai_parser/tests.rs"]
mod tests;

pub(crate) use convert::*;
pub(crate) use output::*;
pub(crate) use parse_file::*;
pub(crate) use parsing::*;
pub(crate) use pdf::*;
pub use types::*;

fn main() {
    entry::run_main();
}
