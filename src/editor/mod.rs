//! Generic editor/canvas primitives.
//!
//! This module is split by one concern per file: interaction controller,
//! alignment/distribution, drop descriptors, inspector descriptors, snap grids,
//! axes, selection, canvas item interaction, marquee selection, lane stacks,
//! value lanes, persistence snapshots, and the canvas adapter.

mod alignment;
mod axis;
mod canvas;
mod drop;
mod inspector;
mod interaction;
mod item_interaction;
mod lane_stack;
mod marquee;
mod persistence;
mod selection;
mod snap;
mod value_lane;

pub use alignment::{align_rects, distribute_rects, DistributionAxis, EditorAlignment};
pub use axis::{Axis, AxisKind, AxisTick};
pub use canvas::{EditorCanvas, EditorCanvasContext};
pub use drop::{EditorDropItem, EditorDropKind, EditorDropRequest};
pub use inspector::{
    apply_inspector_update, EditorInspectorField, EditorInspectorTarget, EditorInspectorUpdate,
};
pub use interaction::{
    CanvasInteraction, CanvasInteractionEvent, CanvasInteractionTarget, CanvasRectMutation,
};
pub use item_interaction::{CanvasItem, CanvasItemHit, ResizeEdges};
pub use lane_stack::{LaneDef, LaneStack};
pub use marquee::MarqueeSelection;
pub use persistence::{EditorInteractionSnapshot, EditorViewSnapshot};
pub use selection::{SelectionMode, SelectionModel};
pub use snap::SnapGrid;
pub use value_lane::ValueLane;
