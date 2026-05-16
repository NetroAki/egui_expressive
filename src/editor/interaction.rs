//! Generic stateful interaction controller for editor canvases.

use std::hash::Hash;

use egui::{Pos2, Rect, Vec2};

use crate::editor::{
    CanvasItem, CanvasItemHit, MarqueeSelection, ResizeEdges, SelectionMode, SelectionModel,
    SnapGrid,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasRectMutation<K> {
    pub id: K,
    pub rect: Rect,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CanvasInteractionEvent<K> {
    None,
    Select(K),
    Move(Vec<CanvasRectMutation<K>>),
    Resize(CanvasRectMutation<K>),
    Marquee { rect: Rect, ids: Vec<K> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CanvasInteractionTarget<K> {
    Move { ids: Vec<K> },
    Resize { id: K, edges: ResizeEdges },
    Marquee { mode: SelectionMode },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasInteraction<K> {
    active: Option<CanvasInteractionTarget<K>>,
    marquee: MarqueeSelection,
}

impl<K> Default for CanvasInteraction<K> {
    fn default() -> Self {
        Self {
            active: None,
            marquee: MarqueeSelection::default(),
        }
    }
}

impl<K> CanvasInteraction<K>
where
    K: Clone + Eq + Hash,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active(&self) -> Option<&CanvasInteractionTarget<K>> {
        self.active.as_ref()
    }

    pub fn marquee_rect(&self) -> Option<Rect> {
        self.marquee.rect()
    }

    pub fn begin(
        &mut self,
        pointer: Pos2,
        items: &[CanvasItem<K>],
        selection: &mut SelectionModel<K>,
        mode: SelectionMode,
        tolerance: f32,
    ) -> CanvasInteractionEvent<K> {
        self.marquee.clear();
        if let Some((item, hit)) = topmost_hit(items, pointer, tolerance) {
            if item.selectable
                && (mode != SelectionMode::Replace || !selection.is_selected(&item.id))
            {
                selection.apply(item.id.clone(), mode);
            }
            match hit {
                CanvasItemHit::Body => {
                    let ids = selection.selected().iter().cloned().collect();
                    self.active = Some(CanvasInteractionTarget::Move { ids });
                }
                CanvasItemHit::Edge(edges) => {
                    self.active = Some(CanvasInteractionTarget::Resize {
                        id: item.id.clone(),
                        edges,
                    });
                }
                CanvasItemHit::None => {}
            }
            CanvasInteractionEvent::Select(item.id.clone())
        } else {
            if mode == SelectionMode::Replace {
                selection.clear();
            }
            self.marquee.begin(pointer);
            self.active = Some(CanvasInteractionTarget::Marquee { mode });
            CanvasInteractionEvent::None
        }
    }

    pub fn drag(
        &mut self,
        pointer: Pos2,
        logical_delta: Vec2,
        items: &[CanvasItem<K>],
        snap: &SnapGrid,
    ) -> CanvasInteractionEvent<K> {
        match self.active.as_ref() {
            Some(CanvasInteractionTarget::Move { ids }) => CanvasInteractionEvent::Move(
                items
                    .iter()
                    .filter(|item| ids.iter().any(|id| id == &item.id))
                    .map(|item| CanvasRectMutation {
                        id: item.id.clone(),
                        rect: item.moved_rect(logical_delta, snap),
                    })
                    .collect(),
            ),
            Some(CanvasInteractionTarget::Resize { id, edges }) => items
                .iter()
                .find(|item| &item.id == id)
                .map(|item| {
                    CanvasInteractionEvent::Resize(CanvasRectMutation {
                        id: item.id.clone(),
                        rect: item.resized_rect(*edges, logical_delta, snap),
                    })
                })
                .unwrap_or(CanvasInteractionEvent::None),
            Some(CanvasInteractionTarget::Marquee { .. }) => {
                self.marquee.update(pointer);
                let rect = self
                    .marquee
                    .rect()
                    .unwrap_or_else(|| Rect::from_min_max(pointer, pointer));
                let ids = self
                    .marquee
                    .intersecting_ids(items.iter().map(|item| (item.id.clone(), item.rect)));
                CanvasInteractionEvent::Marquee { rect, ids }
            }
            None => CanvasInteractionEvent::None,
        }
    }

    pub fn finish(&mut self) {
        self.active = None;
        self.marquee.clear();
    }

    pub fn keyboard_nudge(
        items: &[CanvasItem<K>],
        selected_ids: impl IntoIterator<Item = K>,
        delta: Vec2,
        snap: &SnapGrid,
    ) -> Vec<CanvasRectMutation<K>> {
        let selected: Vec<K> = selected_ids.into_iter().collect();
        items
            .iter()
            .filter(|item| selected.iter().any(|id| id == &item.id))
            .map(|item| CanvasRectMutation {
                id: item.id.clone(),
                rect: item.moved_rect(delta, snap),
            })
            .collect()
    }
}

fn topmost_hit<K>(
    items: &[CanvasItem<K>],
    pointer: Pos2,
    tolerance: f32,
) -> Option<(&CanvasItem<K>, CanvasItemHit)> {
    items.iter().rev().find_map(|item| {
        let hit = item.hit_test(pointer, tolerance);
        (hit != CanvasItemHit::None).then_some((item, hit))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    fn item(id: u64, min: Pos2) -> CanvasItem<u64> {
        CanvasItem::rect(id, Rect::from_min_size(min, vec2(2.0, 1.0))).resizable_x(true)
    }

    #[test]
    fn interaction_moves_selected_items_with_snap() {
        let items = vec![item(1, pos2(0.0, 0.0)), item(2, pos2(4.0, 0.0))];
        let mut selection = SelectionModel::new();
        selection.add(1);
        selection.add(2);
        let mut controller = CanvasInteraction::new();

        controller.begin(
            pos2(0.5, 0.5),
            &items,
            &mut selection,
            SelectionMode::Replace,
            0.1,
        );
        let event = controller.drag(
            pos2(1.6, 0.6),
            vec2(1.6, 0.0),
            &items,
            &SnapGrid::uniform(1.0),
        );

        let CanvasInteractionEvent::Move(mutations) = event else {
            panic!("expected move")
        };
        assert_eq!(mutations.len(), 2);
        assert_eq!(mutations[0].rect.min, pos2(2.0, 0.0));
        assert_eq!(mutations[1].rect.min, pos2(6.0, 0.0));
    }

    #[test]
    fn interaction_resizes_edge_hits() {
        let items = vec![item(1, pos2(0.0, 0.0))];
        let mut selection = SelectionModel::new();
        let mut controller = CanvasInteraction::new();

        controller.begin(
            pos2(2.0, 0.5),
            &items,
            &mut selection,
            SelectionMode::Replace,
            0.2,
        );
        let event = controller.drag(
            pos2(3.0, 0.5),
            vec2(1.0, 0.0),
            &items,
            &SnapGrid::uniform(0.5),
        );

        let CanvasInteractionEvent::Resize(mutation) = event else {
            panic!("expected resize")
        };
        assert_eq!(mutation.rect.width(), 3.0);
    }

    #[test]
    fn interaction_marquee_returns_intersecting_ids() {
        let items = vec![item(1, pos2(0.0, 0.0)), item(2, pos2(5.0, 5.0))];
        let mut selection = SelectionModel::new();
        let mut controller = CanvasInteraction::new();

        controller.begin(
            pos2(-1.0, -1.0),
            &items,
            &mut selection,
            SelectionMode::Replace,
            0.1,
        );
        let event = controller.drag(
            pos2(3.0, 2.0),
            vec2(0.0, 0.0),
            &items,
            &SnapGrid::disabled(),
        );

        let CanvasInteractionEvent::Marquee { ids, .. } = event else {
            panic!("expected marquee")
        };
        assert_eq!(ids, vec![1]);
    }

    #[test]
    fn keyboard_nudge_moves_only_selected_items() {
        let items = vec![item(1, pos2(0.0, 0.0)), item(2, pos2(3.0, 0.0))];
        let mutations = CanvasInteraction::keyboard_nudge(
            &items,
            vec![2],
            vec2(0.6, 0.0),
            &SnapGrid::uniform(1.0),
        );

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].id, 2);
        assert_eq!(mutations[0].rect.min, pos2(4.0, 0.0));
    }
}
