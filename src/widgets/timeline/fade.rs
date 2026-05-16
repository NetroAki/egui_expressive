#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FadeSide {
    In,
    Out,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FadeHandle {
    pub amount: f32,
    pub side: FadeSide,
}

impl FadeHandle {
    pub fn handle_rect(&self, rect: egui::Rect) -> egui::Rect {
        let w = (self.amount.abs() * 20.0).max(6.0);
        if self.side == FadeSide::In {
            egui::Rect::from_min_size(rect.min, egui::Vec2::new(w, rect.height()))
        } else {
            egui::Rect::from_min_size(
                egui::Pos2::new(rect.max.x - w, rect.min.y),
                egui::Vec2::new(w, rect.height()),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_rect_keeps_minimum_width() {
        let handle = FadeHandle {
            amount: 0.1,
            side: FadeSide::In,
        };

        assert!(
            handle
                .handle_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2::new(100.0, 20.0)
                ))
                .width()
                >= 6.0
        );
    }
}
