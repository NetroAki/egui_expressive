//! FocusScope — keyboard navigation.

/// Direction for deterministic focus traversal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusDirection {
    Forward,
    Backward,
}

/// Resolve the next focused widget in a cyclic tab order.
pub fn next_focus_in_order(
    order: &[egui::Id],
    current: Option<egui::Id>,
    direction: FocusDirection,
) -> Option<egui::Id> {
    if order.is_empty() {
        return None;
    }

    let Some(current) = current else {
        return Some(order[0]);
    };

    let position = order.iter().position(|&id| id == current).unwrap_or(0);
    let next = match direction {
        FocusDirection::Forward => (position + 1) % order.len(),
        FocusDirection::Backward => (position + order.len() - 1) % order.len(),
    };
    Some(order[next])
}

/// Manages keyboard focus across a group of widgets.
pub struct FocusScope {
    id: egui::Id,
}

impl FocusScope {
    pub fn new(id: impl std::hash::Hash) -> Self {
        Self {
            id: egui::Id::new(id),
        }
    }

    /// Register a widget ID in this scope's tab order.
    pub fn register(&self, ctx: &egui::Context, widget_id: egui::Id) {
        let order_id = self.id.with("__fs_order");
        let mut order: Vec<egui::Id> = ctx.data(|d| d.get_temp(order_id)).unwrap_or_default();
        if !order.contains(&widget_id) {
            order.push(widget_id);
            ctx.data_mut(|d| d.insert_temp(order_id, order));
        }
    }

    /// Process Tab/Shift+Tab to advance focus. Call once per frame.
    pub fn handle_tab(&self, ctx: &egui::Context) {
        let tab_pressed = ctx.input(|i| i.key_pressed(egui::Key::Tab));
        if !tab_pressed {
            return;
        }

        let shift = ctx.input(|i| i.modifiers.shift);
        let order_id = self.id.with("__fs_order");
        let focused_id = self.id.with("__fs_focused");

        let order: Vec<egui::Id> = ctx.data(|d| d.get_temp(order_id)).unwrap_or_default();
        if order.is_empty() {
            return;
        }

        let current: Option<egui::Id> = ctx.data(|d| d.get_temp(focused_id));
        let direction = if shift {
            FocusDirection::Backward
        } else {
            FocusDirection::Forward
        };
        if let Some(next) = next_focus_in_order(&order, current, direction) {
            ctx.data_mut(|d| d.insert_temp(focused_id, next));
        }
    }

    /// Returns true if the given widget ID currently has focus in this scope.
    pub fn is_focused(&self, ctx: &egui::Context, widget_id: egui::Id) -> bool {
        let focused_id = self.id.with("__fs_focused");
        ctx.data(|d| d.get_temp::<egui::Id>(focused_id))
            .map(|id| id == widget_id)
            .unwrap_or(false)
    }

    /// Programmatically set focus to a widget.
    pub fn focus(&self, ctx: &egui::Context, widget_id: egui::Id) {
        ctx.data_mut(|d| d.insert_temp(self.id.with("__fs_focused"), widget_id));
    }

    /// Clear focus from all widgets in this scope.
    pub fn clear_focus(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| d.remove::<egui::Id>(self.id.with("__fs_focused")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &'static str) -> egui::Id {
        egui::Id::new(value)
    }

    #[test]
    fn next_focus_starts_at_first_registered_widget() {
        let order = [id("a"), id("b")];
        assert_eq!(
            next_focus_in_order(&order, None, FocusDirection::Forward),
            Some(id("a"))
        );
    }

    #[test]
    fn next_focus_wraps_forward_and_backward() {
        let order = [id("a"), id("b"), id("c")];

        assert_eq!(
            next_focus_in_order(&order, Some(id("c")), FocusDirection::Forward),
            Some(id("a"))
        );
        assert_eq!(
            next_focus_in_order(&order, Some(id("a")), FocusDirection::Backward),
            Some(id("c"))
        );
    }

    #[test]
    fn next_focus_recovers_from_missing_current() {
        let order = [id("a"), id("b")];
        assert_eq!(
            next_focus_in_order(&order, Some(id("missing")), FocusDirection::Forward),
            Some(id("b"))
        );
    }

    #[test]
    fn next_focus_returns_none_for_empty_order() {
        assert_eq!(
            next_focus_in_order(&[], None, FocusDirection::Forward),
            None
        );
    }
}
