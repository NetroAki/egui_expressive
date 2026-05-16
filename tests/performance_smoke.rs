use egui::{Color32, Pos2, Rect, Vec2};
use egui_expressive::interaction::{next_focus_in_order, FocusDirection, UndoEntry, UndoStack};
use egui_expressive::{
    align_rects, bounded_visible_range, distribute_rects, BreakpointName, CanvasItem,
    CanvasItemHit, DataCell, DataColumn, DataGridModel, DataGridState, DataRow, DataSortDirection,
    DataSortState, DistributionAxis, EditorAlignment, FormFieldDef, FormFieldKind, FormSchema,
    PanZoom, Tw, ViewportCuller,
};

fn large_grid_model(row_count: usize) -> DataGridModel {
    DataGridModel::new(
        vec![
            DataColumn::new("name", "Name"),
            DataColumn::new("status", "Status"),
        ],
        (0..row_count)
            .map(|index| {
                let status = if index % 4 == 0 { "Ready" } else { "Busy" };
                DataRow::new(
                    format!("row-{index:04}"),
                    vec![
                        DataCell::new(format!("Item {index:04}")),
                        DataCell::new(status),
                    ],
                )
            })
            .collect::<Vec<_>>(),
    )
}

#[test]
fn release_smoke_large_data_grid_visible_rows_stay_count_bounded() {
    let model = large_grid_model(4096);
    let mut state = DataGridState::default();

    state.filter.query = "ready".into();
    state.sort = Some(DataSortState::new(
        Some("name".into()),
        DataSortDirection::Desc,
    ));

    let filtered = model.filtered_sorted_row_indices(&state);
    let visible = model.visible_rows(&state);
    let window = bounded_visible_range(model.rows().len(), 1_000.0, 1_240.0, 20.0, 2);

    assert_eq!(filtered.len(), 1024);
    assert_eq!(visible.len(), 1024);
    assert_eq!(
        filtered.first().map(|idx| model.rows()[*idx].id.as_str()),
        Some("row-4092")
    );
    assert_eq!(
        filtered.last().map(|idx| model.rows()[*idx].id.as_str()),
        Some("row-0000")
    );
    assert_eq!(window, 48..64);
}

#[test]
fn release_smoke_focus_order_scales_with_many_fields() {
    let schema = FormSchema::new(
        (0..512)
            .map(|index| {
                FormFieldDef::new(
                    format!("field-{index:03}"),
                    format!("Field {index:03}"),
                    FormFieldKind::Text,
                )
                .focus_id(format!("focus.{index:03}"))
            })
            .collect::<Vec<_>>(),
    );

    let focus_order = schema.focus_order();
    let order = focus_order.iter().map(egui::Id::new).collect::<Vec<_>>();

    assert_eq!(focus_order.len(), 512);
    assert_eq!(focus_order.first().map(String::as_str), Some("focus.000"));
    assert_eq!(focus_order.last().map(String::as_str), Some("focus.511"));
    assert_eq!(
        next_focus_in_order(&order, None, FocusDirection::Forward),
        Some(order[0])
    );
    assert_eq!(
        next_focus_in_order(&order, Some(order[511]), FocusDirection::Forward),
        Some(order[0])
    );
    assert_eq!(
        next_focus_in_order(&order, Some(order[0]), FocusDirection::Backward),
        Some(order[511])
    );
}

#[test]
fn release_smoke_undo_stack_merge_collapses_repeated_edits() {
    let mut stack = UndoStack::new(String::from("seed"));

    for index in 0..256 {
        stack.push(UndoEntry::new(format!("draft-{index}")).merge_key("typing"));
    }

    assert_eq!(stack.len(), 2);
    assert_eq!(stack.current(), &String::from("draft-255"));
    assert!(stack.can_undo());
    assert!(!stack.can_redo());
}

#[test]
fn release_smoke_undo_stack_redo_branch_is_dropped_after_new_edit() {
    let mut stack = UndoStack::new(0usize);

    for value in 1..=1024 {
        stack.push_snapshot(value);
    }
    for _ in 0..512 {
        stack.undo();
    }

    assert_eq!(stack.current(), &512);
    assert!(stack.can_redo());

    stack.push(UndoEntry::new(9_999).merge_key("typing"));

    assert_eq!(stack.current(), &9_999);
    assert_eq!(stack.len(), 514);
    assert!(!stack.can_redo());
}

#[test]
fn release_smoke_viewport_culler_limits_visible_rows_and_columns() {
    let culler = ViewportCuller::new(
        Rect::from_min_size(Pos2::ZERO, Vec2::new(1_000.0, 2_000.0)),
        PanZoom::new(),
        Pos2::ZERO,
    );

    assert_eq!(culler.visible_rows(50.0, 10_000).len(), 40);
    assert_eq!(culler.visible_cols(100.0, 10_000).len(), 10);
    assert!(culler.is_visible(Rect::from_min_size(
        Pos2::new(950.0, 1_950.0),
        Vec2::splat(100.0),
    )));
    assert!(!culler.is_visible(Rect::from_min_size(
        Pos2::new(2_000.0, 2_000.0),
        Vec2::splat(100.0),
    )));
}

#[test]
fn release_smoke_editor_hit_test_with_many_items() {
    let items = (0..1_024)
        .map(|index| {
            CanvasItem::rect(
                index,
                Rect::from_min_size(Pos2::new(index as f32 * 3.0, 0.0), Vec2::new(2.0, 2.0)),
            )
        })
        .collect::<Vec<_>>();
    let pointer = Pos2::new(1_023.0 * 3.0 + 1.0, 1.0);

    let hit = items
        .iter()
        .rev()
        .find(|item| item.hit_test(pointer, 0.0) != CanvasItemHit::None)
        .map(|item| item.id);

    assert_eq!(hit, Some(1_023));
}

#[test]
fn release_smoke_editor_alignment_many_rects() {
    let rects = (0..512)
        .map(|index| {
            (
                index,
                Rect::from_min_size(
                    Pos2::new(index as f32 * 3.0, (index % 7) as f32),
                    Vec2::new(2.0, 1.0),
                ),
            )
        })
        .collect::<Vec<_>>();
    let ids = (0..512).collect::<Vec<_>>();

    let aligned = align_rects(&rects, ids.clone(), EditorAlignment::Top);
    let distributed = distribute_rects(&rects, ids, DistributionAxis::Horizontal);

    assert_eq!(aligned.len(), 512);
    assert!(aligned.iter().all(|mutation| mutation.rect.top() == 0.0));
    assert_eq!(distributed.len(), 512);
    assert_eq!(distributed.first().map(|mutation| mutation.id), Some(0));
    assert_eq!(distributed.last().map(|mutation| mutation.id), Some(511));
}

#[test]
fn release_smoke_visual_style_resolution_many_variants() {
    let styles = (0..1_024)
        .map(|index| {
            Tw::new()
                .p(2.0)
                .bg_alpha(Color32::from_rgb(20, 40, 80), 0.5)
                .opacity_75()
                .md(Tw::new()
                    .p((index % 8) as f32 + 4.0)
                    .rounded((index % 12) as f32)
                    .ring(1.0, Color32::WHITE)
                    .backdrop_blur(4.0))
        })
        .collect::<Vec<_>>();

    let mut padding_total = 0.0;
    for style in &styles {
        let resolved = style.resolve(BreakpointName::Lg);
        padding_total += resolved.padding.top;
        assert!(resolved.ring.is_some());
        assert_eq!(resolved.backdrop_blur, Some(4.0));
    }

    assert_eq!(styles.len(), 1_024);
    assert_eq!(padding_total, 7_680.0);
}
