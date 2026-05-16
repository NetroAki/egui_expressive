//! Responsive values with Tailwind-style fallback resolution.

use super::{BreakpointName, Breakpoints};

/// A value that can change across breakpoints.
#[derive(Clone, Debug, PartialEq)]
pub struct Responsive<T> {
    pub base: T,
    pub sm: Option<T>,
    pub md: Option<T>,
    pub lg: Option<T>,
    pub xl: Option<T>,
    pub xxl: Option<T>,
}

impl<T> Responsive<T> {
    pub fn new(base: T) -> Self {
        Self {
            base,
            sm: None,
            md: None,
            lg: None,
            xl: None,
            xxl: None,
        }
    }

    pub fn sm(mut self, value: T) -> Self {
        self.sm = Some(value);
        self
    }

    pub fn md(mut self, value: T) -> Self {
        self.md = Some(value);
        self
    }

    pub fn lg(mut self, value: T) -> Self {
        self.lg = Some(value);
        self
    }

    pub fn xl(mut self, value: T) -> Self {
        self.xl = Some(value);
        self
    }

    pub fn xxl(mut self, value: T) -> Self {
        self.xxl = Some(value);
        self
    }

    pub fn resolve(&self, breakpoint: BreakpointName) -> &T {
        match breakpoint {
            BreakpointName::Xxl => self
                .xxl
                .as_ref()
                .or(self.xl.as_ref())
                .or(self.lg.as_ref())
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
            BreakpointName::Lg => self
                .lg
                .as_ref()
                .or(self.md.as_ref())
                .or(self.sm.as_ref())
                .unwrap_or(&self.base),
            BreakpointName::Md => self.md.as_ref().or(self.sm.as_ref()).unwrap_or(&self.base),
            BreakpointName::Sm => self.sm.as_ref().unwrap_or(&self.base),
            BreakpointName::Xs => &self.base,
        }
    }

    pub fn resolve_width(&self, width: f32, breakpoints: Breakpoints) -> &T {
        self.resolve(breakpoints.classify(width))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responsive_values_fall_back_to_lower_breakpoints() {
        let value = Responsive::new(1).md(3).xl(5);
        assert_eq!(*value.resolve(BreakpointName::Xs), 1);
        assert_eq!(*value.resolve(BreakpointName::Sm), 1);
        assert_eq!(*value.resolve(BreakpointName::Md), 3);
        assert_eq!(*value.resolve(BreakpointName::Lg), 3);
        assert_eq!(*value.resolve(BreakpointName::Xl), 5);
        assert_eq!(*value.resolve(BreakpointName::Xxl), 5);
    }
}
