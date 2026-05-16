//! Responsive `Tw` variants, mirroring `sm:`/`md:`/`lg:` utility overrides.

use egui::Ui;

use crate::responsive::{container_breakpoint, BreakpointName, Breakpoints};
use crate::tailwind::builder::Tw;

/// Mobile-first responsive style resolver.
#[derive(Clone, Debug)]
pub struct ResponsiveTw {
    pub base: Tw,
    pub sm: Option<Tw>,
    pub md: Option<Tw>,
    pub lg: Option<Tw>,
    pub xl: Option<Tw>,
    pub xxl: Option<Tw>,
}

impl ResponsiveTw {
    pub fn new(base: Tw) -> Self {
        Self {
            base,
            sm: None,
            md: None,
            lg: None,
            xl: None,
            xxl: None,
        }
    }

    pub fn sm(mut self, style: Tw) -> Self {
        self.sm = Some(style);
        self
    }

    pub fn md(mut self, style: Tw) -> Self {
        self.md = Some(style);
        self
    }

    pub fn lg(mut self, style: Tw) -> Self {
        self.lg = Some(style);
        self
    }

    pub fn xl(mut self, style: Tw) -> Self {
        self.xl = Some(style);
        self
    }

    pub fn xxl(mut self, style: Tw) -> Self {
        self.xxl = Some(style);
        self
    }

    pub fn resolve(&self, breakpoint: BreakpointName) -> &Tw {
        match breakpoint {
            BreakpointName::Xs => &self.base,
            BreakpointName::Sm => self.sm.as_ref().unwrap_or(&self.base),
            BreakpointName::Md => self.md.as_ref().or(self.sm.as_ref()).unwrap_or(&self.base),
            BreakpointName::Lg => self
                .lg
                .as_ref()
                .or(self.md.as_ref())
                .or(self.sm.as_ref())
                .unwrap_or(&self.base),
            BreakpointName::Xl => self
                .xl
                .as_ref()
                .or(self.lg.as_ref())
                .or(self.md.as_ref())
                .or(self.sm.as_ref())
                .unwrap_or(&self.base),
            BreakpointName::Xxl => self
                .xxl
                .as_ref()
                .or(self.xl.as_ref())
                .or(self.lg.as_ref())
                .or(self.md.as_ref())
                .or(self.sm.as_ref())
                .unwrap_or(&self.base),
        }
    }

    pub fn resolve_ui(&self, ui: &Ui) -> &Tw {
        self.resolve(container_breakpoint(ui, Breakpoints::tailwind()))
    }

    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> egui::Response {
        self.resolve_ui(ui).clone().show(ui, content)
    }
}

impl Tw {
    pub fn sm(self, style: Self) -> ResponsiveTw {
        ResponsiveTw::new(self).sm(style)
    }

    pub fn md(self, style: Self) -> ResponsiveTw {
        ResponsiveTw::new(self).md(style)
    }

    pub fn lg(self, style: Self) -> ResponsiveTw {
        ResponsiveTw::new(self).lg(style)
    }

    pub fn xl(self, style: Self) -> ResponsiveTw {
        ResponsiveTw::new(self).xl(style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responsive_tw_uses_mobile_first_fallbacks() {
        let styles = Tw::new().p(4.0).md(Tw::new().p(12.0));
        assert_eq!(styles.resolve(BreakpointName::Xs).padding.top, 4.0);
        assert_eq!(styles.resolve(BreakpointName::Lg).padding.top, 12.0);
    }

    #[test]
    fn responsive_tw_has_direct_show_entrypoint_type() {
        let styles = Tw::new().p(4.0).sm(Tw::new().p(8.0));
        assert_eq!(styles.resolve(BreakpointName::Sm).padding.top, 8.0);
    }
}
