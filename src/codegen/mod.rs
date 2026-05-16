//! SVG layout inference and Rust scaffold code generation.

use egui::Color32;
use std::collections::HashMap;
use std::f32;

mod dims;
mod effect_emit;
mod generate;
mod inference;
mod multi_file;
mod naming;
mod node_emit;
mod node_emit_layout;
mod scaffold;
mod scene_codegen;
mod sidecar;
mod sidecar_values;
mod svg_helpers;
mod svg_parser;
mod types;

#[cfg(test)]
mod tests;

pub(crate) use dims::*;
pub use generate::*;
pub use inference::*;
pub use multi_file::*;
pub use naming::*;
pub(crate) use node_emit::*;
pub(crate) use node_emit_layout::*;
pub use scaffold::*;
pub(crate) use scene_codegen::*;
pub use sidecar::*;
pub(crate) use sidecar_values::*;
pub(crate) use svg_helpers::*;
pub use svg_parser::*;
pub use types::*;
