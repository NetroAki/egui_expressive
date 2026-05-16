//! Focus-ring and modal focus helpers.

use egui::{Color32, CornerRadius, Id, Rect, Response, Stroke, StrokeKind, Ui};

use crate::interaction::FocusScope;
use crate::theme::Theme;

/// Standard visible focus ring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusRing {
    pub stroke: Stroke,
    pub padding: f32,
    pub rounding: f32,
}

impl FocusRing {
    pub fn new(stroke: Stroke) -> Self {
        Self {
            stroke,
            padding: 3.0,
            rounding: 6.0,
        }
    }

    pub fn themed(ctx: &egui::Context) -> Self {
        let theme = Theme::load(ctx);
        Self::new(Stroke::new(2.0, theme.colors.primary))
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn rounding(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self
    }

    pub fn paint(&self, ui: &Ui, rect: Rect) {
        ui.painter().rect_stroke(
            rect.expand(self.padding),
            CornerRadius::same(self.rounding.min(255.0) as u8),
            self.stroke,
            StrokeKind::Outside,
        );
    }

    pub fn paint_if(&self, ui: &Ui, response: &Response) {
        if response.has_focus() || response.highlighted() {
            self.paint(ui, response.rect);
        }
    }
}

impl Default for FocusRing {
    fn default() -> Self {
        Self::new(Stroke::new(2.0, Color32::from_rgb(96, 165, 250)))
    }
}

/// Arrow-key direction for roving focus groups such as tabs, radio groups, and toolbars.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RovingFocusDirection {
    Next,
    Previous,
    First,
    Last,
}

/// Registered item for pure roving-focus resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RovingFocusItem {
    pub id: Id,
    pub disabled: bool,
}

impl RovingFocusItem {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Dependency-free roving-focus model for one-active-descendant widget groups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RovingFocusGroup {
    items: Vec<RovingFocusItem>,
    wrap: bool,
}

impl Default for RovingFocusGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl RovingFocusGroup {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            wrap: true,
        }
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn item(mut self, item: RovingFocusItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn resolve(&self, current: Option<Id>, direction: RovingFocusDirection) -> Option<Id> {
        let enabled: Vec<_> = self.items.iter().filter(|item| !item.disabled).collect();
        if enabled.is_empty() {
            return None;
        }

        let position = current
            .and_then(|id| enabled.iter().position(|item| item.id == id))
            .unwrap_or(0);

        let next = match direction {
            RovingFocusDirection::First => 0,
            RovingFocusDirection::Last => enabled.len() - 1,
            RovingFocusDirection::Next if position + 1 < enabled.len() => position + 1,
            RovingFocusDirection::Next if self.wrap => 0,
            RovingFocusDirection::Next => position,
            RovingFocusDirection::Previous if position > 0 => position - 1,
            RovingFocusDirection::Previous if self.wrap => enabled.len() - 1,
            RovingFocusDirection::Previous => position,
        };
        Some(enabled[next].id)
    }
}

/// Result of modal keyboard handling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalTrapAction {
    None,
    CloseRequested,
}

/// Minimal focus-trap wrapper for modal/dialog scopes.
pub struct ModalFocusTrap {
    scope: FocusScope,
    close_on_escape: bool,
}

impl ModalFocusTrap {
    pub fn new(id: impl std::hash::Hash) -> Self {
        Self {
            scope: FocusScope::new(id),
            close_on_escape: true,
        }
    }

    pub fn close_on_escape(mut self, close_on_escape: bool) -> Self {
        self.close_on_escape = close_on_escape;
        self
    }

    pub fn register(&self, ctx: &egui::Context, widget_id: Id) {
        self.scope.register(ctx, widget_id);
    }

    pub fn focus(&self, ctx: &egui::Context, widget_id: Id) {
        self.scope.focus(ctx, widget_id);
    }

    pub fn is_focused(&self, ctx: &egui::Context, widget_id: Id) -> bool {
        self.scope.is_focused(ctx, widget_id)
    }

    pub fn handle_keyboard(&self, ctx: &egui::Context) -> ModalTrapAction {
        self.scope.handle_tab(ctx);
        if self.close_on_escape && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ModalTrapAction::CloseRequested
        } else {
            ModalTrapAction::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &'static str) -> Id {
        Id::new(value)
    }

    #[test]
    fn roving_focus_skips_disabled_and_wraps() {
        let group = RovingFocusGroup::new()
            .item(RovingFocusItem::new(id("one")))
            .item(RovingFocusItem::new(id("two")).disabled(true))
            .item(RovingFocusItem::new(id("three")));

        assert_eq!(
            group.resolve(Some(id("one")), RovingFocusDirection::Next),
            Some(id("three"))
        );
        assert_eq!(
            group.resolve(Some(id("three")), RovingFocusDirection::Next),
            Some(id("one"))
        );
    }

    #[test]
    fn roving_focus_can_clamp_at_edges() {
        let group = RovingFocusGroup::new()
            .wrap(false)
            .item(RovingFocusItem::new(id("one")))
            .item(RovingFocusItem::new(id("two")));

        assert_eq!(
            group.resolve(Some(id("two")), RovingFocusDirection::Next),
            Some(id("two"))
        );
        assert_eq!(
            group.resolve(Some(id("two")), RovingFocusDirection::First),
            Some(id("one"))
        );
    }
}
