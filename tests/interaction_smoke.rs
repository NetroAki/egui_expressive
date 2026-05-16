use std::collections::BTreeMap;

use egui::{pos2, vec2, Id, Rect};
use egui_expressive::interaction::FocusDirection;
use egui_expressive::interaction::{next_focus_in_order, UndoEntry, UndoStack};
use egui_expressive::{
    align_rects, apply_inspector_update, distribute_rects, AccessibilityRole, CanvasInteraction,
    CanvasInteractionEvent, CanvasItem, DataCell, DataColumn, DataColumnFilter, DataGridModel,
    DataGridState, DataRow, DataSortDirection, DataSortState, DependencyEffect, DistributionAxis,
    EditorAlignment, EditorDropItem, EditorDropKind, EditorDropRequest, EditorInspectorField,
    EditorInspectorTarget, EditorInspectorUpdate, FeedbackMessage, FeedbackQueue, FeedbackSeverity,
    FeedbackToast, FieldDependency, FocusScope, FormFieldDef, FormFieldKind, FormFieldValue,
    FormSchema, LaneDef, LaneStack, LiveRegionPoliteness, ResizeEdges, SelectionMode,
    SelectionModel, SnapGrid, ValueLane,
};

fn sample_large_grid() -> DataGridModel {
    let columns = vec![
        DataColumn::new("name", "Name"),
        DataColumn::new("status", "Status"),
        DataColumn::new("priority", "Priority"),
    ];

    let rows = (0..32)
        .map(|index| {
            let status = if index % 4 == 0 {
                "Ready"
            } else if index % 4 == 1 {
                "Paused"
            } else if index % 4 == 2 {
                "Draft"
            } else {
                "Blocked"
            };
            DataRow::new(
                format!("row-{index:02}"),
                vec![
                    DataCell::new(format!("Item {index:02}")),
                    DataCell::new(status),
                    DataCell::new(format!("P{}", index % 3)),
                ],
            )
        })
        .collect::<Vec<_>>();

    DataGridModel::new(columns, rows)
}

#[test]
fn data_grid_large_filter_sort_and_selection_flow_smoke() {
    let model = sample_large_grid();
    let mut state = DataGridState::default();

    state.filter.query = "item".into();
    state
        .filter
        .column_filters
        .push(DataColumnFilter::new("status", "ready"));
    state.sort = Some(DataSortState::new(
        Some("name".into()),
        DataSortDirection::Desc,
    ));
    state.select_row("row-12");
    state.select_column("priority");
    state.hide_column("priority");
    state.recover_selection(model.rows(), model.columns());

    let visible_rows = model.visible_rows(&state);
    assert_eq!(visible_rows.len(), 8);
    assert_eq!(
        visible_rows.first().map(|row| row.id.as_str()),
        Some("row-28")
    );
    assert_eq!(
        visible_rows.last().map(|row| row.id.as_str()),
        Some("row-00")
    );
    assert_eq!(model.visible_column_indices(&state), vec![0, 1]);
    assert_eq!(
        model.selected_row(&state).map(|row| row.id.as_str()),
        Some("row-12")
    );
    assert_eq!(
        model
            .selected_column(&state)
            .map(|column| column.id.as_str()),
        Some("name")
    );
}

#[test]
fn undo_stack_merge_and_redo_invalidation_smoke() {
    let mut stack = UndoStack::new("seed");
    stack.push(UndoEntry::new("draft-1").merge_key("typing"));
    stack.push(UndoEntry::new("draft-2").merge_key("typing"));

    assert_eq!(stack.len(), 2);
    assert_eq!(stack.current(), &"draft-2");

    stack.undo();
    stack.push_snapshot("draft-3");

    assert_eq!(stack.current(), &"draft-3");
    assert!(!stack.can_redo());
    assert_eq!(stack.len(), 2);
}

#[test]
fn focus_traversal_smoke_cycles_and_scope_focus_state() {
    let order = [Id::new("first"), Id::new("second"), Id::new("third")];

    assert_eq!(
        next_focus_in_order(&order, None, FocusDirection::Forward),
        Some(order[0])
    );
    assert_eq!(
        next_focus_in_order(&order, Some(order[1]), FocusDirection::Forward),
        Some(order[2])
    );
    assert_eq!(
        next_focus_in_order(&order, Some(order[0]), FocusDirection::Backward),
        Some(order[2])
    );

    let ctx = egui::Context::default();
    let scope = FocusScope::new("interaction-smoke");
    let first = Id::new("first-widget");

    scope.focus(&ctx, first);
    assert!(scope.is_focused(&ctx, first));
    scope.clear_focus(&ctx);
    assert!(!scope.is_focused(&ctx, first));
}

#[test]
fn feedback_queue_severity_live_region_and_toast_smoke() {
    let return_focus = Id::new("return-to-toolbar");
    let info = FeedbackMessage::new("info", "Saved").severity(FeedbackSeverity::Info);
    let warning = FeedbackMessage::new("warn", "Check input").severity(FeedbackSeverity::Warning);
    let error = FeedbackMessage::new("err", "Failed").severity(FeedbackSeverity::Error);

    assert_eq!(info.live_region().politeness, LiveRegionPoliteness::Polite);
    assert_eq!(
        warning.live_region().politeness,
        LiveRegionPoliteness::Assertive
    );
    assert_eq!(
        error.live_region().politeness,
        LiveRegionPoliteness::Assertive
    );

    let meta = error.accessibility_meta(AccessibilityRole::Status);
    assert_eq!(
        meta.live_region.as_ref().map(|region| region.politeness),
        Some(LiveRegionPoliteness::Assertive)
    );

    let mut queue = FeedbackQueue::with_max_toasts(2);
    queue.push_modal(FeedbackMessage::new("modal", "Confirm").focus_return(return_focus));
    queue
        .push_snackbar(FeedbackMessage::new("snackbar-1", "Queued one").focus_return(return_focus));
    queue.push_snackbar(FeedbackMessage::new("snackbar-2", "Queued two"));
    queue.push_toast(
        FeedbackToast::new("toast-1", "Toast one", 3.0).severity(FeedbackSeverity::Success),
    );
    queue.push_toast(FeedbackToast::new("toast-2", "Toast two", 3.0));
    queue.push_toast(FeedbackToast::new("toast-3", "Toast three", 3.0));

    assert!(queue.active_modal().is_some());
    assert_eq!(
        queue.visible_snackbar().map(|message| message.id.as_str()),
        Some("snackbar-1")
    );
    assert_eq!(queue.queued_snackbars(), 1);
    assert_eq!(queue.toasts().len(), 2);
    assert_eq!(queue.toasts()[0].message.id, "toast-2");
    assert_eq!(queue.toasts()[1].message.id, "toast-3");
    assert_eq!(queue.dismiss_modal(), Some(return_focus));
    assert_eq!(queue.dismiss_snackbar(), Some(return_focus));
    assert_eq!(
        queue.visible_snackbar().map(|message| message.id.as_str()),
        Some("snackbar-2")
    );
}

#[test]
fn form_schema_dependency_and_focus_order_smoke() {
    let schema = FormSchema::new(vec![
        FormFieldDef::new("advanced", "Advanced", FormFieldKind::Switch).focus_id("focus.advanced"),
        FormFieldDef::new("token", "Token", FormFieldKind::Text)
            .focus_id("focus.token")
            .dependency(FieldDependency::new(
                "advanced",
                FormFieldValue::Bool(true),
                DependencyEffect::Show,
            ))
            .dependency(FieldDependency::new(
                "advanced",
                FormFieldValue::Bool(true),
                DependencyEffect::Require,
            )),
        FormFieldDef::new("notes", "Notes", FormFieldKind::TextArea)
            .focus_id("focus.notes")
            .dependency(FieldDependency::new(
                "advanced",
                FormFieldValue::Bool(false),
                DependencyEffect::Hide,
            )),
    ]);

    let enabled_values = BTreeMap::from([(String::from("advanced"), FormFieldValue::Bool(true))]);
    let disabled_values = BTreeMap::from([(String::from("advanced"), FormFieldValue::Bool(false))]);

    let enabled_states = schema.evaluate_dependencies(&enabled_values);
    let disabled_states = schema.evaluate_dependencies(&disabled_values);

    assert_eq!(
        schema.focus_order(),
        vec!["focus.advanced", "focus.token", "focus.notes"]
    );
    assert!(enabled_states["token"].visible);
    assert!(enabled_states["token"].required);
    assert!(!disabled_states["notes"].visible);
}

#[test]
fn editor_canvas_interaction_select_move_resize_marquee_smoke() {
    let items = vec![
        CanvasItem::rect(
            "node-a",
            Rect::from_min_size(pos2(0.0, 0.0), vec2(2.0, 1.0)),
        )
        .resizable_x(true)
        .resizable_y(true),
        CanvasItem::rect(
            "node-b",
            Rect::from_min_size(pos2(4.0, 0.0), vec2(2.0, 1.0)),
        )
        .resizable_x(true),
    ];
    let mut selection = SelectionModel::new();
    let mut controller = CanvasInteraction::new();

    assert_eq!(
        controller.begin(
            pos2(0.5, 0.5),
            &items,
            &mut selection,
            SelectionMode::Replace,
            0.1,
        ),
        CanvasInteractionEvent::Select("node-a")
    );
    let move_event = controller.drag(
        pos2(1.5, 0.5),
        vec2(1.6, 0.0),
        &items,
        &SnapGrid::uniform(1.0),
    );
    let CanvasInteractionEvent::Move(mutations) = move_event else {
        panic!("expected move event")
    };
    assert_eq!(mutations.len(), 1);
    assert_eq!(mutations[0].id, "node-a");
    assert_eq!(mutations[0].rect.min, pos2(2.0, 0.0));
    controller.finish();

    controller.begin(
        pos2(2.0, 0.5),
        &items,
        &mut selection,
        SelectionMode::Replace,
        0.2,
    );
    let resize_event = controller.drag(
        pos2(3.0, 0.5),
        vec2(1.0, 0.0),
        &items,
        &SnapGrid::uniform(0.5),
    );
    let CanvasInteractionEvent::Resize(resize) = resize_event else {
        panic!("expected resize event")
    };
    assert_eq!(resize.id, "node-a");
    assert_eq!(resize.rect.width(), 3.0);
    controller.finish();

    controller.begin(
        pos2(-1.0, -1.0),
        &items,
        &mut selection,
        SelectionMode::Replace,
        0.1,
    );
    let marquee_event = controller.drag(
        pos2(3.0, 2.0),
        vec2(0.0, 0.0),
        &items,
        &SnapGrid::disabled(),
    );
    let CanvasInteractionEvent::Marquee { ids, .. } = marquee_event else {
        panic!("expected marquee event")
    };
    assert_eq!(ids, vec!["node-a"]);
}

#[test]
fn editor_alignment_drop_inspector_lane_smoke() {
    let rects = vec![
        ("a", Rect::from_min_size(pos2(3.0, 0.0), vec2(1.0, 1.0))),
        ("b", Rect::from_min_size(pos2(8.0, 2.0), vec2(1.0, 1.0))),
        ("c", Rect::from_min_size(pos2(12.0, 0.0), vec2(1.0, 1.0))),
    ];
    let aligned = align_rects(&rects, vec!["a", "b", "c"], EditorAlignment::Left);
    assert_eq!(aligned.len(), 3);
    assert!(aligned.iter().all(|mutation| mutation.rect.left() == 3.0));

    let distributed = distribute_rects(&rects, vec!["a", "b", "c"], DistributionAxis::Horizontal);
    assert_eq!(distributed[1].rect.left(), 7.5);

    let drop_request = EditorDropRequest::new(
        pos2(8.0, 4.0),
        [
            EditorDropItem::new("file", "sprite.png", EditorDropKind::FilePath)
                .mime_type("image/png"),
            EditorDropItem::new("object", "Rectangle", EditorDropKind::Object),
        ],
    );
    assert_eq!(
        drop_request.accepted_items(&[EditorDropKind::Object]).len(),
        1
    );
    assert!(drop_request.accepts_kind(EditorDropKind::FilePath));

    let mut target = EditorInspectorTarget::new(
        "node-a",
        "Node A",
        [
            EditorInspectorField::new("x", "X", FormFieldValue::Number(1.0)),
            EditorInspectorField::new("id", "Id", FormFieldValue::Text("node-a".into()))
                .read_only(true),
        ],
    );
    assert!(apply_inspector_update(
        &mut target,
        EditorInspectorUpdate::new("node-a", "x", FormFieldValue::Number(2.0)),
    ));
    assert!(!apply_inspector_update(
        &mut target,
        EditorInspectorUpdate::new("node-a", "id", FormFieldValue::Text("node-b".into())),
    ));
    assert_eq!(target.fields[0].value, FormFieldValue::Number(2.0));

    let lanes = LaneStack::new()
        .gap(2.0)
        .lane(LaneDef::new("notes", "Notes", 10.0))
        .lane(LaneDef::new("velocity", "Velocity", 12.0));
    assert_eq!(lanes.total_height(), 24.0);

    let value_lane = ValueLane::new(0.0..=100.0).inverted(false);
    assert_eq!(value_lane.normalize(25.0), 0.25);
    assert_eq!(value_lane.denormalize(0.75), 75.0);
    assert!(!ResizeEdges::HORIZONTAL.is_empty());
}
