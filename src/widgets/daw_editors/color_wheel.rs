use egui::{Color32, Response, Sense, Ui, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorWheelState {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
}

pub struct ColorWheel<'a> {
    pub state: &'a mut ColorWheelState,
    pub radius: f32,
}

impl<'a> ColorWheel<'a> {
    pub fn new(state: &'a mut ColorWheelState) -> Self {
        Self {
            state,
            radius: 56.0,
        }
    }
    pub fn color(&self) -> Color32 {
        hsv_to_rgb(self.state.hue, self.state.saturation, self.state.value)
    }
}

pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Color32 {
    let hue = hue.rem_euclid(1.0) * 6.0;
    let saturation = saturation.clamp(0.0, 1.0);
    let value = value.clamp(0.0, 1.0);
    let chroma = value * saturation;
    let secondary = chroma * (1.0 - (hue % 2.0 - 1.0).abs());
    let match_value = value - chroma;
    let (r1, g1, b1) = match hue as u8 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };
    Color32::from_rgb(
        channel_to_u8(r1 + match_value),
        channel_to_u8(g1 + match_value),
        channel_to_u8(b1 + match_value),
    )
}

fn channel_to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl<'a> egui::Widget for ColorWheel<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::splat(self.radius * 2.0), Sense::click_and_drag());
        ui.painter()
            .circle_filled(rect.center(), self.radius, self.color());
        if let Some(pos) = response.interact_pointer_pos() {
            let delta = pos - rect.center();
            self.state.hue = (delta.angle() / std::f32::consts::TAU).rem_euclid(1.0);
            self.state.saturation = (delta.length() / self.radius).clamp(0.0, 1.0);
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_to_rgb_returns_primary_colors() {
        assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), Color32::from_rgb(255, 0, 0));
        assert_eq!(
            hsv_to_rgb(1.0 / 3.0, 1.0, 1.0),
            Color32::from_rgb(0, 255, 0)
        );
        assert_eq!(
            hsv_to_rgb(2.0 / 3.0, 1.0, 1.0),
            Color32::from_rgb(0, 0, 255)
        );
    }

    #[test]
    fn hsv_to_rgb_handles_grayscale_and_clamping() {
        assert_eq!(hsv_to_rgb(0.25, 0.0, 0.5), Color32::from_rgb(128, 128, 128));
        assert_eq!(hsv_to_rgb(-1.0, 2.0, 2.0), Color32::from_rgb(255, 0, 0));
    }

    #[test]
    fn color_wheel_color_uses_hsv_conversion() {
        let mut state = ColorWheelState {
            hue: 1.0 / 6.0,
            saturation: 1.0,
            value: 1.0,
        };
        let wheel = ColorWheel::new(&mut state);

        assert_eq!(wheel.color(), Color32::from_rgb(255, 255, 0));
    }
}
