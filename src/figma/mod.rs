//! Figma token parsing and Rust token code generation.

use crate::style::DesignTokens;
use serde::Deserialize;
mod codegen;
mod parse;
mod runtime;

#[cfg(test)]
mod tests;

pub use codegen::*;
pub use parse::*;
pub use runtime::*;
