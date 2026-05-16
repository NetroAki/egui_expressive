/// Transport control button kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportKind {
    Play,
    Stop,
    Record,
    Metronome,
    Loop,
}

/// A transport control button.
pub struct TransportButton<'a> {
    kind: TransportKind,
    active: &'a mut bool,
    size: f32,
}

impl<'a> TransportButton<'a> {
    pub fn new(kind: TransportKind, active: &'a mut bool) -> Self {
        Self {
            kind,
            active,
            size: 28.0,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
}

impl<'a> egui::Widget for TransportButton<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(self.size), egui::Sense::click());
        if response.clicked() {
            *self.active = !*self.active;
        }
        let painter = ui.painter();
        let center = rect.center();
        let r = self.size * 0.5;
        let visuals = ui.visuals();
        let bg = if *self.active {
            visuals.selection.bg_fill
        } else if response.hovered() {
            visuals.widgets.hovered.bg_fill
        } else {
            egui::Color32::TRANSPARENT
        };
        painter.rect_filled(rect, egui::CornerRadius::same(4), bg);
        let icon_color = if *self.active {
            visuals.widgets.active.fg_stroke.color
        } else {
            visuals.widgets.inactive.fg_stroke.color
        };
        match self.kind {
            TransportKind::Play => {
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        egui::Pos2::new(center.x - r * 0.3, center.y - r * 0.45),
                        egui::Pos2::new(center.x + r * 0.45, center.y),
                        egui::Pos2::new(center.x - r * 0.3, center.y + r * 0.45),
                    ],
                    icon_color,
                    egui::Stroke::NONE,
                ));
            }
            TransportKind::Stop => {
                painter.rect_filled(
                    egui::Rect::from_center_size(center, egui::Vec2::splat(r * 1.1)),
                    egui::CornerRadius::ZERO,
                    icon_color,
                );
            }
            TransportKind::Record => {
                painter.circle_filled(center, r * 0.4, egui::Color32::from_rgb(220, 70, 70));
            }
            TransportKind::Metronome => {
                painter.line_segment(
                    [
                        egui::Pos2::new(center.x, center.y - r * 0.5),
                        egui::Pos2::new(center.x, center.y + r * 0.5),
                    ],
                    egui::Stroke::new(2.0, icon_color),
                );
                painter.line_segment(
                    [
                        center,
                        egui::Pos2::new(center.x + r * 0.3, center.y - r * 0.1),
                    ],
                    egui::Stroke::new(2.0, icon_color),
                );
            }
            TransportKind::Loop => {
                painter.circle_stroke(center, r * 0.4, egui::Stroke::new(2.0, icon_color));
                let tip = egui::Pos2::new(center.x + r * 0.4, center.y);
                let a1 = egui::Pos2::new(tip.x - r * 0.15, tip.y - r * 0.15);
                let a2 = egui::Pos2::new(tip.x + r * 0.15, tip.y - r * 0.15);
                painter.add(egui::Shape::convex_polygon(
                    vec![tip, a1, a2],
                    icon_color,
                    egui::Stroke::NONE,
                ));
            }
        }
        response
    }
}
