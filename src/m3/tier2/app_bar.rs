use crate::m3::{M3Elevation, M3Theme};
use egui::{Align2, CornerRadius, FontId, Pos2, Rect, Response, Sense, Ui, Vec2};

#[derive(Clone, Copy, Default, Debug)]
pub enum M3TopAppBarVariant {
    #[default]
    Small,
    CenterAligned,
    Medium,
    Large,
}

pub struct M3TopAppBar<'a> {
    title: &'a str,
    variant: M3TopAppBarVariant,
    navigation_icon: Option<char>,
    actions: Vec<(char, &'a str)>,
    scrolled: bool,
}

fn top_app_bar_height(variant: M3TopAppBarVariant) -> f32 {
    match variant {
        M3TopAppBarVariant::Small | M3TopAppBarVariant::CenterAligned => 64.0,
        M3TopAppBarVariant::Medium => 112.0,
        M3TopAppBarVariant::Large => 152.0,
    }
}

fn top_app_bar_title_font(variant: M3TopAppBarVariant) -> FontId {
    match variant {
        M3TopAppBarVariant::Small | M3TopAppBarVariant::CenterAligned => FontId::proportional(22.0),
        M3TopAppBarVariant::Medium => FontId::proportional(24.0),
        M3TopAppBarVariant::Large => FontId::proportional(28.0),
    }
}

fn top_app_bar_title_y(rect: Rect, variant: M3TopAppBarVariant) -> f32 {
    match variant {
        M3TopAppBarVariant::Small | M3TopAppBarVariant::CenterAligned => rect.top() + 32.0,
        M3TopAppBarVariant::Medium => rect.bottom() - 24.0,
        M3TopAppBarVariant::Large => rect.bottom() - 28.0,
    }
}

fn top_app_bar_title_layout(
    rect: Rect,
    variant: M3TopAppBarVariant,
    leading_x: f32,
) -> (Pos2, Align2) {
    let title_y = top_app_bar_title_y(rect, variant);
    match variant {
        M3TopAppBarVariant::CenterAligned => {
            (Pos2::new(rect.center().x, title_y), Align2::CENTER_CENTER)
        }
        _ => (Pos2::new(leading_x, title_y), Align2::LEFT_CENTER),
    }
}

impl<'a> M3TopAppBar<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            variant: M3TopAppBarVariant::Small,
            navigation_icon: None,
            actions: Vec::new(),
            scrolled: false,
        }
    }
    pub fn center_aligned(mut self) -> Self {
        self.variant = M3TopAppBarVariant::CenterAligned;
        self
    }
    pub fn medium(mut self) -> Self {
        self.variant = M3TopAppBarVariant::Medium;
        self
    }
    pub fn large(mut self) -> Self {
        self.variant = M3TopAppBarVariant::Large;
        self
    }
    pub fn navigation_icon(mut self, icon: char) -> Self {
        self.navigation_icon = Some(icon);
        self
    }
    pub fn action(mut self, icon: char, tooltip: &'a str) -> Self {
        self.actions.push((icon, tooltip));
        self
    }
    pub fn scrolled(mut self, s: bool) -> Self {
        self.scrolled = s;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let height = top_app_bar_height(self.variant);
        let bg = if self.scrolled {
            M3Elevation::Level2.surface_tint(c.surface, c.primary)
        } else {
            c.surface
        };
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect_filled(rect, CornerRadius::ZERO, bg);
            let icon_size = 24.0_f32;
            let icon_pad = 12.0_f32;
            let mut x = rect.left() + icon_pad;
            if let Some(nav_icon) = self.navigation_icon {
                painter.text(
                    Pos2::new(x + icon_size / 2.0, rect.top() + 32.0),
                    egui::Align2::CENTER_CENTER,
                    nav_icon.to_string(),
                    egui::FontId::proportional(icon_size),
                    c.on_surface,
                );
                x += icon_size + icon_pad;
            }

            let title_font = top_app_bar_title_font(self.variant);
            let (title_pos, title_align) = top_app_bar_title_layout(rect, self.variant, x);
            painter.text(title_pos, title_align, self.title, title_font, c.on_surface);

            let mut ax = rect.right() - icon_pad;
            for (icon, _tooltip) in self.actions.iter().rev() {
                ax -= icon_size;
                painter.text(
                    Pos2::new(ax + icon_size / 2.0, rect.top() + 32.0),
                    egui::Align2::CENTER_CENTER,
                    icon.to_string(),
                    egui::FontId::proportional(icon_size),
                    c.on_surface_variant,
                );
                ax -= icon_pad;
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_app_bar_variant_heights_match_material_contract() {
        assert_eq!(top_app_bar_height(M3TopAppBarVariant::Small), 64.0);
        assert_eq!(top_app_bar_height(M3TopAppBarVariant::CenterAligned), 64.0);
        assert_eq!(top_app_bar_height(M3TopAppBarVariant::Medium), 112.0);
        assert_eq!(top_app_bar_height(M3TopAppBarVariant::Large), 152.0);
    }

    #[test]
    fn top_app_bar_title_alignment_is_centered_only_for_center_variant() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(320.0, 64.0));
        let leading_x = 48.0;

        let (center_pos, center_align) =
            top_app_bar_title_layout(rect, M3TopAppBarVariant::CenterAligned, leading_x);
        assert_eq!(center_pos, Pos2::new(160.0, 32.0));
        assert_eq!(center_align, Align2::CENTER_CENTER);

        let (small_pos, small_align) =
            top_app_bar_title_layout(rect, M3TopAppBarVariant::Small, leading_x);
        assert_eq!(small_pos, Pos2::new(48.0, 32.0));
        assert_eq!(small_align, Align2::LEFT_CENTER);
    }

    #[test]
    fn top_app_bar_medium_and_large_title_baselines_are_bottom_anchored() {
        let medium = Rect::from_min_size(Pos2::ZERO, Vec2::new(320.0, 112.0));
        let large = Rect::from_min_size(Pos2::ZERO, Vec2::new(320.0, 152.0));

        assert_eq!(
            top_app_bar_title_y(medium, M3TopAppBarVariant::Medium),
            88.0
        );
        assert_eq!(top_app_bar_title_y(large, M3TopAppBarVariant::Large), 124.0);
        assert_eq!(top_app_bar_title_font(M3TopAppBarVariant::Large).size, 28.0);
    }

    #[test]
    fn top_app_bar_scrolled_state_keeps_builder_intent() {
        let app_bar = M3TopAppBar::new("Title")
            .navigation_icon('<')
            .action('S', "Search")
            .scrolled(true);
        assert_eq!(app_bar.navigation_icon, Some('<'));
        assert_eq!(app_bar.actions, vec![('S', "Search")]);
        assert!(app_bar.scrolled);
    }
}
