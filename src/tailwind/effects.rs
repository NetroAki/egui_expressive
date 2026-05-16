//! Visual effect utilities for `Tw`.

use egui::{Color32, Vec2};

use crate::{
    tailwind::{types::GradientDirection, Tw},
    theme::Elevation,
};

impl Tw {
    pub fn shadow(mut self, elevation: Elevation) -> Self {
        self.elevation = Some(elevation);
        self
    }

    pub fn bg_gradient(mut self, direction: GradientDirection, from: Color32, to: Color32) -> Self {
        self.gradient = Some(crate::tailwind::types::TwGradient::two_stop(
            direction, from, to,
        ));
        self
    }

    pub fn bg_gradient_stops(
        mut self,
        direction: GradientDirection,
        stops: impl IntoIterator<Item = (f32, Color32)>,
    ) -> Self {
        self.gradient = Some(crate::tailwind::types::TwGradient::new(direction, stops));
        self
    }

    pub fn bg_gradient_to_r(self, from: Color32, to: Color32) -> Self {
        self.bg_gradient(GradientDirection::ToRight, from, to)
    }

    pub fn bg_gradient_to_b(self, from: Color32, to: Color32) -> Self {
        self.bg_gradient(GradientDirection::ToBottom, from, to)
    }

    pub fn bg_gradient_angle(self, angle_deg: f32, from: Color32, to: Color32) -> Self {
        self.bg_gradient(GradientDirection::Angle(angle_deg), from, to)
    }

    pub fn bg_gradient_angle_stops(
        self,
        angle_deg: f32,
        stops: impl IntoIterator<Item = (f32, Color32)>,
    ) -> Self {
        self.bg_gradient_stops(GradientDirection::Angle(angle_deg), stops)
    }

    pub fn backdrop_blur(mut self, radius: f32) -> Self {
        self.backdrop_blur = Some(radius.max(0.0));
        self.backdrop_source = crate::tailwind::types::TwBackdropSource::BoundedOverlay;
        self
    }

    pub fn backdrop_blur_app_provided(mut self, radius: f32) -> Self {
        self.backdrop_blur = Some(radius.max(0.0));
        self.backdrop_source = crate::tailwind::types::TwBackdropSource::AppProvidedSnapshot;
        self
    }

    pub fn drop_shadow(mut self, offset: Vec2, blur: u8, color: Color32) -> Self {
        self.drop_shadow = Some(crate::tailwind::types::TwDropShadow {
            offset,
            blur,
            color,
        });
        self
    }
    pub fn aspect_ratio(mut self, ratio: f32) -> Self {
        self.aspect_ratio = (ratio > 0.0).then_some(ratio);
        self
    }

    pub fn ring(mut self, width: f32, color: Color32) -> Self {
        self.ring = Some(crate::tailwind::types::TwRing { width, color });
        self
    }

    pub fn transition(mut self, duration_secs: f32) -> Self {
        self.transition = Some(crate::tailwind::types::TwTransition { duration_secs });
        self
    }

    pub fn selection(mut self, bg: Color32, fg: Color32) -> Self {
        self.selection = Some(crate::tailwind::types::SelectionStyle { bg, fg });
        self
    }
}
