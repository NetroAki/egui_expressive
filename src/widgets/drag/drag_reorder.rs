use egui::{Id, Response, Sense, Ui};

pub struct DragReorder<'a, T> {
    items: &'a mut Vec<T>,
    _id: Id,
}

impl<'a, T> DragReorder<'a, T> {
    pub fn new(items: &'a mut Vec<T>, id: impl std::hash::Hash) -> Self {
        Self {
            items,
            _id: Id::new(id),
        }
    }
    pub fn show(self, ui: &mut Ui, mut render: impl FnMut(&mut Ui, usize, &mut T)) -> Response {
        let mut last = ui.allocate_response(egui::Vec2::ZERO, Sense::hover());
        let mut order_change = None;
        for i in 0..self.items.len() {
            ui.horizontal(|ui| {
                let handle = ui.add_sized(
                    [20.0, 20.0],
                    egui::Label::new("⋮⋮").sense(Sense::click_and_drag()),
                );
                render(ui, i, &mut self.items[i]);
                if handle.dragged() && handle.drag_delta().y.abs() > 12.0 {
                    order_change = Some(
                        (i as isize + if handle.drag_delta().y > 0.0 { 1 } else { -1 })
                            .clamp(0, (self.items.len() - 1) as isize)
                            as usize,
                    );
                }
                last = handle;
            });
        }
        if let Some(new_idx) = order_change {
            /* simple reorder not based on exact hover; enough for API */
            let old = 0usize.min(self.items.len().saturating_sub(1));
            self.items.swap(old, new_idx);
        }
        last
    }
}
