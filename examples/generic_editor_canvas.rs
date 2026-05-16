//! Generic editor canvas demo.
//!
//! This example intentionally avoids DAW-specific modules. The rectangles could be
//! notes, clips, automation handles, designer parts, or any other editor item.

use eframe::egui;
use egui_expressive::interaction::{UndoEntry, UndoStack};
use egui_expressive::{
    align_rects, apply_inspector_update, distribute_rects, Axis, CanvasInteraction,
    CanvasInteractionEvent, CanvasItem, DistributionAxis, EditorAlignment, EditorCanvas,
    EditorDropItem, EditorDropKind, EditorDropRequest, EditorInspectorField, EditorInspectorTarget,
    EditorInspectorUpdate, FormFieldValue, ResizeEdges, SelectionMode, SelectionModel, SnapGrid,
};

#[derive(Clone)]
struct EditorItem {
    id: u64,
    label: String,
    rect: egui::Rect,
}

struct GenericEditorCanvasApp {
    items: Vec<EditorItem>,
    selection: SelectionModel<u64>,
    interaction: CanvasInteraction<u64>,
    history: UndoStack<Vec<(u64, egui::Rect)>>,
    last_feedback: String,
    drop_request: Option<EditorDropRequest>,
    drag_baseline: Option<(Vec<EditorItem>, egui::Pos2)>,
}

impl Default for GenericEditorCanvasApp {
    fn default() -> Self {
        let items = vec![
            EditorItem::new(1, "Panel", egui::pos2(1.0, 1.0), egui::vec2(2.0, 1.0)),
            EditorItem::new(2, "Card", egui::pos2(4.0, 2.0), egui::vec2(3.0, 1.0)),
            EditorItem::new(3, "Badge", egui::pos2(8.0, 4.0), egui::vec2(1.5, 1.0)),
        ];
        Self {
            history: UndoStack::new(snapshot(&items)),
            items,
            selection: SelectionModel::default(),
            interaction: CanvasInteraction::default(),
            last_feedback: "Use the buttons to run pure Stage 6 editor interactions.".to_owned(),
            drop_request: None,
            drag_baseline: None,
        }
    }
}

impl eframe::App for GenericEditorCanvasApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Generic Editor Canvas");
        ui.separator();

        self.toolbar(ui);
        ui.label(&self.last_feedback);
        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| self.canvas(ui));
            ui.separator();
            ui.vertical(|ui| self.inspector(ui));
        });
    }
}

impl GenericEditorCanvasApp {
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Select all").clicked() {
                self.selection
                    .replace_all(self.items.iter().map(|item| item.id));
                self.last_feedback = "Selected all items".to_owned();
            }
            if ui.button("Nudge right").clicked() {
                let mutations = CanvasInteraction::keyboard_nudge(
                    &canvas_items_from(&self.items),
                    self.selected_ids(),
                    egui::vec2(1.0, 0.0),
                    &SnapGrid::uniform(1.0),
                );
                self.apply_mutations(mutations, "Nudge right");
            }
            if ui.button("Inspector X +1").clicked() {
                self.apply_inspector_x_delta(1.0);
            }
            if ui.button("Align left").clicked() {
                let mutations = align_rects(
                    &self.rect_pairs(),
                    self.selected_ids(),
                    EditorAlignment::Left,
                );
                self.apply_mutations(mutations, "Align left");
            }
            if ui.button("Distribute X").clicked() {
                let mutations = distribute_rects(
                    &self.rect_pairs(),
                    self.selected_ids(),
                    DistributionAxis::Horizontal,
                );
                self.apply_mutations(mutations, "Distribute X");
            }
            if ui.button("Drop object").clicked() {
                self.drop_request = Some(EditorDropRequest::new(
                    egui::pos2(6.0, 1.0),
                    [EditorDropItem::new(
                        "shape.rect",
                        "Dropped rectangle",
                        EditorDropKind::Object,
                    )],
                ));
                self.last_feedback =
                    "Drop descriptor recorded without filesystem or platform side effects"
                        .to_owned();
            }
            if ui.button("Undo").clicked() {
                if let Some(snapshot) = self.history.undo().cloned() {
                    self.restore(snapshot);
                    self.last_feedback = "Undo snapshot restored".to_owned();
                }
            }
            if ui.button("Redo").clicked() {
                if let Some(snapshot) = self.history.redo().cloned() {
                    self.restore(snapshot);
                    self.last_feedback = "Redo snapshot restored".to_owned();
                }
            }
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        let snap = SnapGrid::uniform(1.0);
        let x_axis = Axis::time(0.0..=16.0, 4.0).unit("b").minor_step(1.0);

        let mut pointer = None;
        let response = EditorCanvas::new(
            ui.id().with("generic_editor_canvas"),
            egui::vec2(1600.0, 800.0),
        )
        .snap_grid(snap)
        .x_axis(x_axis)
        .zoom_range(20.0, 160.0)
        .show(ui, |canvas| {
            let painter = canvas.ui.painter();
            pointer = canvas.ui.input(|input| {
                input
                    .pointer
                    .interact_pos()
                    .map(|pos| canvas.culler.to_logical(pos))
            });

            for item in &self.items {
                let canvas_item = CanvasItem::rect(item.id, item.rect)
                    .resizable_x(true)
                    .min_size(egui::vec2(0.5, 0.5));
                let screen_rect = canvas.rect_to_screen(canvas_item.rect);
                let selected = self.selection.is_selected(&item.id);
                let fill = if selected {
                    egui::Color32::from_rgb(120, 170, 255)
                } else {
                    egui::Color32::from_rgb(80, 120, 190)
                };

                painter.rect_filled(screen_rect, 4.0, fill.linear_multiply(0.7));
                painter.rect_stroke(
                    screen_rect,
                    4.0,
                    egui::Stroke::new(1.5, fill),
                    egui::StrokeKind::Outside,
                );
                painter.text(
                    screen_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &item.label,
                    egui::TextStyle::Small.resolve(canvas.ui.style()),
                    egui::Color32::WHITE,
                );

                let handle_rect = egui::Rect::from_min_max(
                    egui::pos2(screen_rect.right() - 5.0, screen_rect.top()),
                    screen_rect.right_bottom(),
                );
                painter.rect_filled(handle_rect, 2.0, egui::Color32::from_white_alpha(60));
            }

            if let Some(drop_request) = &self.drop_request {
                let drop_pos = canvas.to_screen(drop_request.target);
                painter.circle_filled(drop_pos, 5.0, egui::Color32::LIGHT_GREEN);
                painter.text(
                    drop_pos + egui::vec2(8.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    "object drop descriptor",
                    egui::TextStyle::Small.resolve(canvas.ui.style()),
                    egui::Color32::LIGHT_GREEN,
                );
            }
        });
        self.handle_canvas_pointer(&response, pointer);
    }

    fn inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Inspector hooks");
        let targets = self.inspector_targets();
        if targets.is_empty() {
            ui.label("No selected item");
        }
        for target in targets {
            ui.group(|ui| {
                ui.strong(target.label);
                for field in target.fields {
                    ui.label(format!("{}: {:?}", field.label, field.value));
                }
            });
        }
    }

    fn apply_mutations(
        &mut self,
        mutations: Vec<egui_expressive::CanvasRectMutation<u64>>,
        label: &str,
    ) {
        if mutations.is_empty() {
            self.last_feedback = format!("{label}: nothing selected");
            return;
        }
        for mutation in mutations {
            if let Some(item) = self.items.iter_mut().find(|item| item.id == mutation.id) {
                item.rect = mutation.rect;
            }
        }
        self.history
            .push(UndoEntry::new(snapshot(&self.items)).label(label));
        self.last_feedback = label.to_owned();
    }

    fn apply_mutations_without_history(
        &mut self,
        mutations: Vec<egui_expressive::CanvasRectMutation<u64>>,
    ) {
        for mutation in mutations {
            if let Some(item) = self.items.iter_mut().find(|item| item.id == mutation.id) {
                item.rect = mutation.rect;
            }
        }
    }

    fn handle_canvas_pointer(&mut self, response: &egui::Response, pointer: Option<egui::Pos2>) {
        let Some(pointer) = pointer else { return };
        let snap = SnapGrid::uniform(1.0);
        if response.drag_started() {
            let baseline = self.items.clone();
            self.interaction.begin(
                pointer,
                &canvas_items_from(&baseline),
                &mut self.selection,
                SelectionMode::Replace,
                0.15,
            );
            self.drag_baseline = Some((baseline, pointer));
        }
        let event = self.drag_baseline.as_ref().and_then(|(baseline, start)| {
            response.dragged().then(|| {
                self.interaction.drag(
                    pointer,
                    pointer - *start,
                    &canvas_items_from(baseline),
                    &snap,
                )
            })
        });
        match event {
            Some(CanvasInteractionEvent::Move(mutations)) => {
                self.apply_mutations_without_history(mutations)
            }
            Some(CanvasInteractionEvent::Resize(mutation)) => {
                self.apply_mutations_without_history(vec![mutation])
            }
            Some(CanvasInteractionEvent::Marquee { ids, .. }) => self.selection.replace_all(ids),
            _ => {}
        }
        if response.drag_stopped() {
            if let Some((baseline, _)) = self.drag_baseline.take() {
                if snapshot(&self.items) != snapshot(&baseline) {
                    self.history
                        .push(UndoEntry::new(snapshot(&self.items)).label("Pointer drag"));
                    self.last_feedback = "Pointer-driven canvas interaction committed".to_owned();
                }
            }
            self.interaction.finish();
        }
    }

    fn apply_inspector_x_delta(&mut self, delta: f64) {
        let Some(id) = self.selected_ids().first().copied() else {
            self.last_feedback = "Inspector update: no selected item".to_owned();
            return;
        };
        let Some(index) = self.items.iter().position(|item| item.id == id) else {
            return;
        };
        let mut target = inspector_target_for(&self.items[index]);
        let next_x = self.items[index].rect.min.x as f64 + delta;
        if apply_inspector_update(
            &mut target,
            EditorInspectorUpdate::new(id, "x", FormFieldValue::Number(next_x)),
        ) {
            let width = self.items[index].rect.width();
            let height = self.items[index].rect.height();
            self.items[index].rect = egui::Rect::from_min_size(
                egui::pos2(next_x as f32, self.items[index].rect.min.y),
                egui::vec2(width, height),
            );
            self.history
                .push(UndoEntry::new(snapshot(&self.items)).label("Inspector X +1"));
            self.last_feedback = "Inspector update descriptor applied to selected item".to_owned();
        }
    }

    fn restore(&mut self, snapshot: Vec<(u64, egui::Rect)>) {
        for (id, rect) in snapshot {
            if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
                item.rect = rect;
            }
        }
    }

    fn selected_ids(&self) -> Vec<u64> {
        self.selection.selected().iter().copied().collect()
    }

    fn rect_pairs(&self) -> Vec<(u64, egui::Rect)> {
        self.items.iter().map(|item| (item.id, item.rect)).collect()
    }

    fn inspector_targets(&self) -> Vec<EditorInspectorTarget<u64>> {
        self.items
            .iter()
            .filter(|item| self.selection.is_selected(&item.id))
            .map(inspector_target_for)
            .collect()
    }
}

impl EditorItem {
    fn new(id: u64, label: impl Into<String>, min: egui::Pos2, size: egui::Vec2) -> Self {
        Self {
            id,
            label: label.into(),
            rect: egui::Rect::from_min_size(min, size),
        }
    }
}

fn snapshot(items: &[EditorItem]) -> Vec<(u64, egui::Rect)> {
    items.iter().map(|item| (item.id, item.rect)).collect()
}

fn canvas_items_from(items: &[EditorItem]) -> Vec<CanvasItem<u64>> {
    items
        .iter()
        .map(|item| {
            CanvasItem::rect(item.id, item.rect)
                .resize_edges(ResizeEdges::HORIZONTAL)
                .min_size(egui::vec2(0.5, 0.5))
        })
        .collect()
}

fn inspector_target_for(item: &EditorItem) -> EditorInspectorTarget<u64> {
    EditorInspectorTarget::new(
        item.id,
        item.label.clone(),
        [
            EditorInspectorField::new("x", "X", FormFieldValue::Number(item.rect.min.x as f64)),
            EditorInspectorField::new("y", "Y", FormFieldValue::Number(item.rect.min.y as f64)),
            EditorInspectorField::new(
                "width",
                "Width",
                FormFieldValue::Number(item.rect.width() as f64),
            ),
        ],
    )
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Generic Editor Canvas",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(GenericEditorCanvasApp::default()))),
    )
}
