//! Tailwind-style breakpoint names and thresholds.

/// Tailwind-recognizable breakpoint names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BreakpointName {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
}

/// Configurable breakpoint thresholds in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Breakpoints {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

impl Default for Breakpoints {
    fn default() -> Self {
        Self::tailwind()
    }
}

impl Breakpoints {
    /// Tailwind CSS defaults: `sm=640`, `md=768`, `lg=1024`, `xl=1280`, `2xl=1536`.
    pub const fn tailwind() -> Self {
        Self {
            sm: 640.0,
            md: 768.0,
            lg: 1024.0,
            xl: 1280.0,
            xxl: 1536.0,
        }
    }

    /// Classify a width into a breakpoint bucket.
    pub fn classify(self, width: f32) -> BreakpointName {
        if width >= self.xxl {
            BreakpointName::Xxl
        } else if width >= self.xl {
            BreakpointName::Xl
        } else if width >= self.lg {
            BreakpointName::Lg
        } else if width >= self.md {
            BreakpointName::Md
        } else if width >= self.sm {
            BreakpointName::Sm
        } else {
            BreakpointName::Xs
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_tailwind_breakpoints() {
        let bp = Breakpoints::tailwind();
        assert_eq!(bp.classify(320.0), BreakpointName::Xs);
        assert_eq!(bp.classify(640.0), BreakpointName::Sm);
        assert_eq!(bp.classify(800.0), BreakpointName::Md);
        assert_eq!(bp.classify(1100.0), BreakpointName::Lg);
        assert_eq!(bp.classify(1300.0), BreakpointName::Xl);
        assert_eq!(bp.classify(1600.0), BreakpointName::Xxl);
    }
}
