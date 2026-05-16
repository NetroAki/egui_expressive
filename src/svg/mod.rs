//! SVG path/document parsing and ASE palette parsing.

use egui::{epaint::PathStroke, Color32, Pos2, Rect, Shape, Stroke};

mod ase;
mod document;
mod path;

pub use ase::*;
pub use document::*;
pub use path::*;
