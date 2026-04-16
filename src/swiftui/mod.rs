//! # SwiftUI-style helpers for egui
//!
//! This module provides SwiftUI-inspired patterns built on top of egui and `Tw`:
//!
//! - [`Navigator`] — push/pop screen navigation state machine
//! - [`ScrollList`] — virtualized scrollable list with viewport culling
//! - [`GeometryProxy`] — available size reader (like SwiftUI's GeometryReader)
//! - [`ViewModifier`] — composable style modifier trait
//!
//! ## Example
//!
//! ```rust,ignore
//! let mut nav = Navigator::new("main_nav");
//! ScrollList::new(items).row_height(40.0).show(ui, |ui, item| {
//!     // render row
//! });
//! ```

use egui::{Context, Id, Response, ScrollArea, Ui, Vec2};

// ---------------------------------------------------------------------------
// Navigator
// ---------------------------------------------------------------------------

/// A screen navigation state machine stored in egui memory.
/// Provides push/pop navigation like SwiftUI's NavigationView.
///
/// The navigator tracks stack depth; the user manages their own screen state
/// in a [`StateSlot`](crate::state::StateSlot) or similar.
///
/// ## Example
///
/// ```rust,ignore
/// let mut nav = Navigator::new("my_nav");
///
/// // Show current screen
/// match nav.current::<Screen>() {
///     Screen::Home => home_ui(ui),
///     Screen::Detail(id) => detail_ui(ui, id),
/// }
///
/// // Navigate
/// nav.push(ctx);
/// ```
pub struct Navigator {
    id: Id,
}

impl Navigator {
    /// Create a new navigator with the given ID.
    pub fn new(id: impl std::hash::Hash) -> Self {
        Self { id: Id::new(id) }
    }

    /// Create or load a navigator (alias for [`new`]).
    #[inline]
    pub fn load(_ctx: &Context, id: impl std::hash::Hash) -> Self {
        Self::new(id)
    }

    /// Current stack depth. 0 = root (no screen pushed), 1 = root screen, 2+ = pushed screens.
    pub fn depth(&self, ctx: &Context) -> usize {
        ctx.data(|d| d.get_temp::<usize>(self.id).unwrap_or(0))
    }

    /// Push a new screen: increment depth.
    pub fn push(&self, ctx: &Context) {
        let d = self.depth(ctx);
        ctx.data_mut(|data| data.insert_temp(self.id, d + 1));
    }

    /// Pop the top screen: decrement depth. Returns true if there was something to pop.
    pub fn pop(&self, ctx: &Context) -> bool {
        let d = self.depth(ctx);
        if d > 0 {
            ctx.data_mut(|data| data.insert_temp(self.id, d - 1));
            true
        } else {
            false
        }
    }

    /// Returns true if there is a screen to pop (depth > 0).
    pub fn can_go_back(&self, ctx: &Context) -> bool {
        self.depth(ctx) > 0
    }

    /// Reset to root: set depth to 0.
    pub fn reset(&self, ctx: &Context) {
        ctx.data_mut(|data| data.insert_temp(self.id, 0usize));
    }
}

// ---------------------------------------------------------------------------
// ScrollList
// ---------------------------------------------------------------------------

/// A virtualized scrollable list, like SwiftUI's List.
/// Only renders visible rows (viewport culling via ScrollArea).
///
/// ## Example
///
/// ```rust,ignore
/// let items = vec!["Alice", "Bob", "Charlie"];
/// ScrollList::new(items).row_height(40.0).show(ui, |ui, name| {
///     ui.label(*name);
/// });
/// ```
pub struct ScrollList<T> {
    items: Vec<T>,
    row_height: f32,
    id_salt: Id,
}

impl<T: Clone> ScrollList<T> {
    /// Create a new ScrollList with the given items.
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            row_height: 32.0,
            id_salt: Id::new("scroll_list"),
        }
    }

    /// Set the row height in points.
    pub fn row_height(mut self, h: f32) -> Self {
        self.row_height = h;
        self
    }

    /// Set the id salt for ScrollArea persistence.
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id_salt = Id::new(id);
        self
    }

    /// Render the list. The closure receives a mutable Ui reference and the current item.
    pub fn show(self, ui: &mut Ui, mut row_fn: impl FnMut(&mut Ui, &T)) {
        let row_height = self.row_height;
        let total_rows = self.items.len();
        let items = self.items;

        ScrollArea::vertical().id_salt(self.id_salt).show_rows(
            ui,
            row_height,
            total_rows,
            |ui, row_range| {
                for i in row_range {
                    if let Some(item) = items.get(i) {
                        row_fn(ui, item);
                    }
                }
            },
        );
    }
}

// ---------------------------------------------------------------------------
// GeometryProxy
// ---------------------------------------------------------------------------

/// Like SwiftUI's GeometryReader — provides the available size to the content closure.
///
/// ## Example
///
/// ```rust,ignore
/// GeometryProxy::read(ui, |ui, size| {
///     ui.label(format!("Width: {:.1}", size.x));
/// });
/// ```
pub struct GeometryProxy;

impl GeometryProxy {
    /// Render content with access to the available size of the parent UI.
    #[inline]
    pub fn read(ui: &mut Ui, content: impl FnOnce(&mut Ui, Vec2)) {
        let size = ui.available_size();
        content(ui, size);
    }
}

// ---------------------------------------------------------------------------
// ViewModifier
// ---------------------------------------------------------------------------

/// A reusable style modifier, like SwiftUI's ViewModifier protocol.
/// Implement this to create named, composable style presets.
///
/// ## Example
///
/// ```rust,ignore
/// struct CardStyle;
/// impl ViewModifier for CardStyle {
///     fn body(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Response {
///         ui.vertical(|ui| {
///             ui.style_mut().spacing.item_spacing = Vec2::new(8.0, 8.0);
///             content(ui);
///         }).response
///     }
/// }
/// ```
pub trait ViewModifier {
    /// The response from applying this modifier.
    fn body(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Response;
}

// ---------------------------------------------------------------------------
// Convenience macros
// ---------------------------------------------------------------------------

/// SwiftUI-style conditional view rendering.
///
/// ## Example
///
/// ```rust,ignore
/// if_view!(show_details, ui, {
///     ui.label("Details");
/// });
/// ```
#[macro_export]
macro_rules! if_view {
    ($cond:expr, $ui:expr, { $($body:tt)* }) => {
        if $cond {
            $($body)*
        }
    };
}

/// SwiftUI-style ForEach — iterates over items and renders content for each.
///
/// ## Example
///
/// ```rust,ignore
/// for_each!(items, ui, |item| {
///     ui.label(item);
/// });
/// ```
#[macro_export]
macro_rules! for_each {
    ($items:expr, $ui:expr, |$item:ident| { $($body:tt)* }) => {
        for $item in $items.iter() {
            $($body)*
        }
    };
}
