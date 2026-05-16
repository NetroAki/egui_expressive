//! Scene graph model, appearance stack, and egui renderer.

use crate::codegen::{BlendMode, EffectDef, EffectType, GradientDef, StrokeCap, StrokeJoin};

#[path = "scene/effects_geom.rs"]
mod effects_geom;
#[path = "scene/fill.rs"]
mod fill;
#[path = "scene/model.rs"]
mod model;
#[path = "scene/render.rs"]
mod render;
#[path = "scene/stroke.rs"]
mod stroke;

#[cfg(test)]
#[path = "scene/tests.rs"]
mod tests;

pub(crate) use self::effects_geom::*;
pub(crate) use self::fill::*;
pub use self::model::*;
pub use self::render::*;
pub(crate) use self::stroke::*;
