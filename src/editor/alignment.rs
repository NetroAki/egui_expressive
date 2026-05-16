//! Alignment and distribution helpers for editor items.

use std::collections::HashSet;
use std::hash::Hash;

use egui::Rect;

use crate::editor::CanvasRectMutation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorAlignment {
    Left,
    CenterX,
    Right,
    Top,
    CenterY,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributionAxis {
    Horizontal,
    Vertical,
}

pub fn align_rects<K>(
    items: &[(K, Rect)],
    selected_ids: impl IntoIterator<Item = K>,
    alignment: EditorAlignment,
) -> Vec<CanvasRectMutation<K>>
where
    K: Clone + Eq + Hash,
{
    let selected: HashSet<K> = selected_ids.into_iter().collect();
    let selected_rects: Vec<&(K, Rect)> = items
        .iter()
        .filter(|(id, _)| selected.contains(id))
        .collect();
    if selected_rects.is_empty() {
        return Vec::new();
    }

    let target = match alignment {
        EditorAlignment::Left => selected_rects
            .iter()
            .map(|(_, rect)| rect.left())
            .fold(f32::INFINITY, f32::min),
        EditorAlignment::CenterX => average(selected_rects.iter().map(|(_, rect)| rect.center().x)),
        EditorAlignment::Right => selected_rects
            .iter()
            .map(|(_, rect)| rect.right())
            .fold(f32::NEG_INFINITY, f32::max),
        EditorAlignment::Top => selected_rects
            .iter()
            .map(|(_, rect)| rect.top())
            .fold(f32::INFINITY, f32::min),
        EditorAlignment::CenterY => average(selected_rects.iter().map(|(_, rect)| rect.center().y)),
        EditorAlignment::Bottom => selected_rects
            .iter()
            .map(|(_, rect)| rect.bottom())
            .fold(f32::NEG_INFINITY, f32::max),
    };

    selected_rects
        .into_iter()
        .map(|(id, rect)| CanvasRectMutation {
            id: id.clone(),
            rect: align_one(*rect, alignment, target),
        })
        .collect()
}

pub fn distribute_rects<K>(
    items: &[(K, Rect)],
    selected_ids: impl IntoIterator<Item = K>,
    axis: DistributionAxis,
) -> Vec<CanvasRectMutation<K>>
where
    K: Clone + Eq + Hash,
{
    let selected: HashSet<K> = selected_ids.into_iter().collect();
    let mut selected_rects: Vec<(K, Rect)> = items
        .iter()
        .filter(|(id, _)| selected.contains(id))
        .cloned()
        .collect();
    if selected_rects.len() < 3 {
        return selected_rects
            .into_iter()
            .map(|(id, rect)| CanvasRectMutation { id, rect })
            .collect();
    }

    selected_rects.sort_by(|(_, a), (_, b)| start(*a, axis).total_cmp(&start(*b, axis)));
    let first = *selected_rects.first().map(|(_, rect)| rect).unwrap();
    let last = *selected_rects.last().map(|(_, rect)| rect).unwrap();
    let span_start = start(first, axis);
    let span_end = end(last, axis);
    let total_size: f32 = selected_rects
        .iter()
        .map(|(_, rect)| size(*rect, axis))
        .sum();
    let gap = ((span_end - span_start) - total_size) / (selected_rects.len() as f32 - 1.0);

    let mut cursor = span_start;
    selected_rects
        .into_iter()
        .map(|(id, rect)| {
            let placed = place(rect, axis, cursor);
            cursor += size(rect, axis) + gap;
            CanvasRectMutation { id, rect: placed }
        })
        .collect()
}

fn align_one(rect: Rect, alignment: EditorAlignment, target: f32) -> Rect {
    match alignment {
        EditorAlignment::Left => rect.translate(egui::vec2(target - rect.left(), 0.0)),
        EditorAlignment::CenterX => rect.translate(egui::vec2(target - rect.center().x, 0.0)),
        EditorAlignment::Right => rect.translate(egui::vec2(target - rect.right(), 0.0)),
        EditorAlignment::Top => rect.translate(egui::vec2(0.0, target - rect.top())),
        EditorAlignment::CenterY => rect.translate(egui::vec2(0.0, target - rect.center().y)),
        EditorAlignment::Bottom => rect.translate(egui::vec2(0.0, target - rect.bottom())),
    }
}

fn average(values: impl Iterator<Item = f32>) -> f32 {
    let (sum, count) = values.fold((0.0, 0usize), |(sum, count), value| {
        (sum + value, count + 1)
    });
    sum / count.max(1) as f32
}

fn start(rect: Rect, axis: DistributionAxis) -> f32 {
    match axis {
        DistributionAxis::Horizontal => rect.left(),
        DistributionAxis::Vertical => rect.top(),
    }
}

fn end(rect: Rect, axis: DistributionAxis) -> f32 {
    match axis {
        DistributionAxis::Horizontal => rect.right(),
        DistributionAxis::Vertical => rect.bottom(),
    }
}

fn size(rect: Rect, axis: DistributionAxis) -> f32 {
    match axis {
        DistributionAxis::Horizontal => rect.width(),
        DistributionAxis::Vertical => rect.height(),
    }
}

fn place(rect: Rect, axis: DistributionAxis, value: f32) -> Rect {
    match axis {
        DistributionAxis::Horizontal => rect.translate(egui::vec2(value - rect.left(), 0.0)),
        DistributionAxis::Vertical => rect.translate(egui::vec2(0.0, value - rect.top())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(pos2(x, y), vec2(w, h))
    }

    #[test]
    fn align_rects_matches_left_edge() {
        let items = vec![(1, rect(3.0, 0.0, 1.0, 1.0)), (2, rect(6.0, 1.0, 2.0, 1.0))];
        let mutations = align_rects(&items, vec![1, 2], EditorAlignment::Left);
        assert_eq!(mutations[0].rect.left(), 3.0);
        assert_eq!(mutations[1].rect.left(), 3.0);
    }

    #[test]
    fn distribute_rects_places_even_gaps() {
        let items = vec![
            (1, rect(0.0, 0.0, 1.0, 1.0)),
            (2, rect(2.0, 0.0, 1.0, 1.0)),
            (3, rect(6.0, 0.0, 1.0, 1.0)),
        ];
        let mutations = distribute_rects(&items, vec![1, 2, 3], DistributionAxis::Horizontal);
        assert!((mutations[1].rect.left() - 3.0).abs() < 0.0001);
    }

    #[test]
    fn align_rects_covers_remaining_edges_and_centers() {
        let items = vec![(1, rect(0.0, 0.0, 2.0, 2.0)), (2, rect(4.0, 6.0, 2.0, 2.0))];
        let cases = [
            (EditorAlignment::Right, 4.0, 4.0),
            (EditorAlignment::CenterX, 2.0, 2.0),
            (EditorAlignment::Top, 0.0, 0.0),
            (EditorAlignment::CenterY, 3.0, 3.0),
            (EditorAlignment::Bottom, 6.0, 6.0),
        ];

        for (alignment, first, second) in cases {
            let mutations = align_rects(&items, vec![1, 2], alignment);
            let values = match alignment {
                EditorAlignment::Right => [mutations[0].rect.left(), mutations[1].rect.left()],
                EditorAlignment::CenterX => [mutations[0].rect.left(), mutations[1].rect.left()],
                EditorAlignment::Top => [mutations[0].rect.top(), mutations[1].rect.top()],
                EditorAlignment::CenterY => [mutations[0].rect.top(), mutations[1].rect.top()],
                EditorAlignment::Bottom => [mutations[0].rect.top(), mutations[1].rect.top()],
                EditorAlignment::Left => unreachable!(),
            };
            assert_eq!(values, [first, second]);
        }
    }

    #[test]
    fn distribute_rects_places_vertical_even_gaps() {
        let items = vec![
            (1, rect(0.0, 0.0, 1.0, 1.0)),
            (2, rect(0.0, 2.0, 1.0, 1.0)),
            (3, rect(0.0, 6.0, 1.0, 1.0)),
        ];
        let mutations = distribute_rects(&items, vec![1, 2, 3], DistributionAxis::Vertical);
        assert!((mutations[1].rect.top() - 3.0).abs() < 0.0001);
    }
}
