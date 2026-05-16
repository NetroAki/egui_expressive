use egui::{Color32, Response, Ui, Vec2};

#[derive(Clone, Debug, PartialEq)]
pub struct SystemMetric {
    pub label: String,
    pub value: f32,
    pub warning: f32,
}

pub struct SystemMonitor<'a> {
    pub metrics: &'a [SystemMetric],
}

impl<'a> egui::Widget for SystemMonitor<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            for metric in self.metrics {
                let color = if metric.value >= metric.warning {
                    Color32::RED
                } else {
                    Color32::from_rgb(80, 190, 120)
                };
                ui.horizontal(|ui| {
                    ui.label(&metric.label);
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(120.0, 6.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, Color32::from_gray(40));
                    ui.painter()
                        .rect_filled(RectExt::scale_x(rect, metric.value), 2.0, color);
                });
            }
        })
        .response
    }
}

struct RectExt;
impl RectExt {
    fn scale_x(rect: egui::Rect, value: f32) -> egui::Rect {
        egui::Rect::from_min_max(
            rect.min,
            egui::pos2(
                rect.left() + rect.width() * value.clamp(0.0, 1.0),
                rect.bottom(),
            ),
        )
    }
}
