use egui::{Pos2, Response, Sense, Ui, Vec2};

pub struct RoutingCable {
    pub points: [Pos2; 4],
}
pub struct DesignerPart {
    pub id: String,
    pub pos: Pos2,
}
pub struct DesignerCanvas<'a> {
    pub parts: &'a mut [DesignerPart],
}

impl RoutingCable {
    pub fn new(start: Pos2, end: Pos2) -> Self {
        let dx = (end.x - start.x).abs() * 0.35;
        Self {
            points: [
                start,
                Pos2::new(start.x + dx, start.y),
                Pos2::new(end.x - dx, end.y),
                end,
            ],
        }
    }
}
impl<'a> egui::Widget for DesignerCanvas<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let available_width = ui.available_width();
        let available_height = ui.available_height();
        let width = if available_width.is_finite() {
            available_width.max(1.0)
        } else {
            240.0
        };
        let height = if available_height.is_finite() {
            available_height.max(1.0)
        } else {
            120.0
        };
        let resp = ui.allocate_response(Vec2::new(width, height), Sense::click_and_drag());
        for part in self.parts.iter() {
            ui.painter()
                .circle_filled(part.pos, 6.0, egui::Color32::from_rgb(180, 180, 220));
        }
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routing_cable_generates_four_control_points() {
        let cable = RoutingCable::new(Pos2::new(0.0, 0.0), Pos2::new(10.0, 5.0));

        assert_eq!(cable.points.len(), 4);
    }
}
