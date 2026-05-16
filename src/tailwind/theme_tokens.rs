//! Theme-token utilities for Tailwind-like semantic colors and `dark:` variants.

use egui::Context;

use crate::tailwind::builder::Tw;
use crate::theme::{SemanticColors, Theme};

/// Surface token family, matching `bg-surface-*` / `text-surface-*` intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceLevel {
    Base,
    Dim,
    Bright,
    Container,
    On,
    OnVariant,
    Outline,
    OutlineVariant,
}

/// Accent token family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccentKind {
    Primary,
    OnPrimary,
    Secondary,
    OnSecondary,
    Error,
    OnError,
    Scrim,
}

/// Semantic color token stored on `Tw` until render time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorToken {
    Surface(SurfaceLevel),
    Accent(AccentKind),
}

impl ColorToken {
    pub fn resolve(self, colors: &SemanticColors) -> egui::Color32 {
        match self {
            ColorToken::Surface(level) => match level {
                SurfaceLevel::Base => colors.surface,
                SurfaceLevel::Dim => colors.surface_dim,
                SurfaceLevel::Bright => colors.surface_bright,
                SurfaceLevel::Container => colors.surface_container,
                SurfaceLevel::On => colors.on_surface,
                SurfaceLevel::OnVariant => colors.on_surface_variant,
                SurfaceLevel::Outline => colors.outline,
                SurfaceLevel::OutlineVariant => colors.outline_variant,
            },
            ColorToken::Accent(kind) => match kind {
                AccentKind::Primary => colors.primary,
                AccentKind::OnPrimary => colors.on_primary,
                AccentKind::Secondary => colors.secondary,
                AccentKind::OnSecondary => colors.on_secondary,
                AccentKind::Error => colors.error,
                AccentKind::OnError => colors.on_error,
                AccentKind::Scrim => colors.scrim,
            },
        }
    }
}

/// Tailwind-like `dark:` resolver.
#[derive(Clone, Debug)]
pub struct TwThemeVariants {
    pub base: Tw,
    pub dark: Option<Tw>,
    pub light: Option<Tw>,
}

impl TwThemeVariants {
    pub fn new(base: Tw) -> Self {
        Self {
            base,
            dark: None,
            light: None,
        }
    }

    pub fn dark(mut self, style: Tw) -> Self {
        self.dark = Some(style);
        self
    }

    pub fn light(mut self, style: Tw) -> Self {
        self.light = Some(style);
        self
    }

    pub fn resolve_theme(&self, theme: &Theme) -> &Tw {
        if theme.is_dark {
            self.dark.as_ref().unwrap_or(&self.base)
        } else {
            self.light.as_ref().unwrap_or(&self.base)
        }
    }

    pub fn resolve_ctx(&self, ctx: &Context) -> &Tw {
        let theme = Theme::load(ctx);
        if theme.is_dark {
            self.dark.as_ref().unwrap_or(&self.base)
        } else {
            self.light.as_ref().unwrap_or(&self.base)
        }
    }

    pub fn show(self, ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) -> egui::Response {
        self.resolve_ctx(ui.ctx()).clone().show(ui, content)
    }
}

impl Tw {
    pub fn bg_surface(mut self, level: SurfaceLevel) -> Self {
        self.bg_token = Some(ColorToken::Surface(level));
        self
    }

    pub fn text_surface(mut self, level: SurfaceLevel) -> Self {
        self.fg_token = Some(ColorToken::Surface(level));
        self
    }

    pub fn bg_accent(mut self, kind: AccentKind) -> Self {
        self.bg_token = Some(ColorToken::Accent(kind));
        self
    }

    pub fn bg_accent_alpha(self, kind: AccentKind, alpha: f32, ctx: &egui::Context) -> Self {
        let theme = Theme::load(ctx);
        let color = ColorToken::Accent(kind).resolve(&theme.colors);
        self.bg_alpha(color, alpha)
    }

    pub fn bg_surface_alpha(self, level: SurfaceLevel, alpha: f32, ctx: &egui::Context) -> Self {
        let theme = Theme::load(ctx);
        let color = ColorToken::Surface(level).resolve(&theme.colors);
        self.bg_alpha(color, alpha)
    }

    pub fn text_accent(mut self, kind: AccentKind) -> Self {
        self.fg_token = Some(ColorToken::Accent(kind));
        self
    }

    pub fn dark(self, style: Self) -> TwThemeVariants {
        TwThemeVariants::new(self).dark(style)
    }

    pub fn light(self, style: Self) -> TwThemeVariants {
        TwThemeVariants::new(self).light(style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_tokens_resolve_against_theme() {
        let theme = Theme::neutraudio_dark();
        assert_eq!(
            ColorToken::Surface(SurfaceLevel::Base).resolve(&theme.colors),
            theme.colors.surface
        );
        assert_eq!(
            ColorToken::Accent(AccentKind::Primary).resolve(&theme.colors),
            theme.colors.primary
        );
    }

    #[test]
    fn dark_variant_resolves_dark_style() {
        let variants = Tw::new().p(4.0).dark(Tw::new().p(12.0));
        assert_eq!(variants.resolve_theme(&Theme::dark()).padding.top, 12.0);
        assert_eq!(variants.resolve_theme(&Theme::light()).padding.top, 4.0);
    }
}
