# Interaction Architecture Guide

Stage 4 interaction primitives live under `src/interaction/*`. They are pure state and dispatch helpers first; widgets and Material 3 surfaces stay thin visual adapters.

## Commands and Actions

- Define stable app behavior with `ActionDef`.
- Store actions in `ActionRegistry`.
- Use `ActionRegistry::dispatch_status(id)` before invoking app code so disabled and unknown actions are consumed deterministically.
- Build menus and palettes from the same registry with `MenuItemDef::from_action`, `MenuDef::actions`, and `CommandPaletteItem::from_registry`.
- Route `TopMenuBar::activated` and `CommandPalette::activated` ids through `ActionRegistry::dispatch_status`; both surfaces emit ids and leave app-specific side effects to the caller.

```rust,no_run
use egui_expressive::interaction::{ActionDef, ActionRegistry};
use egui_expressive::widgets::{CommandPaletteItem, MenuDef, TopMenuBar};

let mut actions = ActionRegistry::new();
actions.register(ActionDef::new("file.save", "Save").description("Save the current document"));

let menu = MenuDef::actions("File", actions.iter().cloned());
let palette_items = CommandPaletteItem::from_registry(&actions);
let mut activated = None;
let menu_bar = TopMenuBar::new(std::slice::from_ref(&menu)).activated(&mut activated);
# let _ = (menu_bar, palette_items);
```

## Scoped Shortcuts

`ScopedShortcutRegistry` resolves shortcuts in this order:

1. active modal
2. active overlay / command palette
3. focused panel
4. app-global

Same key in the same scope for different actions is rejected. Same key in different scopes is allowed; highest active scope with a binding wins. Non-modal higher scopes without a binding fall through. Active modals trap unmatched key events. Disabled matching actions consume the key without falling through. Bindings that reference unknown action ids resolve as `ShortcutResolution::Unknown` so configuration bugs are visible to callers.

```rust,no_run
use egui_expressive::interaction::{
    ScopedShortcutBinding, ScopedShortcutRegistry, ShortcutBinding, ShortcutScope,
};

let mut shortcuts = ScopedShortcutRegistry::new();
shortcuts.bind(ScopedShortcutBinding::new(
    ShortcutScope::Global,
    ShortcutBinding::new("file.save", egui::Key::S, egui::Modifiers::CTRL),
))?;
# Ok::<(), egui_expressive::interaction::ShortcutConflict>(())
```

## Focus Traversal

`FocusScope` owns egui-backed tab handling. `next_focus_in_order` is the pure traversal contract used by tests and by later Stage 5/6 consumers.

```rust,no_run
use egui_expressive::interaction::{next_focus_in_order, FocusDirection};

let order = [egui::Id::new("name"), egui::Id::new("email")];
let next = next_focus_in_order(&order, None, FocusDirection::Forward);
# let _ = next;
```

## Undo / Redo

`UndoStack<T>` is unbounded and snapshot-based for Stage 4. It supports labels, merge keys, redo invalidation after push, merge-key replacement, and clear/reset. Editor object graphs and command closures are deferred to Stage 6.

Stage 6 editor/canvas surfaces use this same stack for snapshot proofs; see `docs/ui-framework/editor-canvas.md` for `CanvasInteraction`, keyboard nudge, drop descriptors, inspector hooks, and editor-specific undo/persistence guidance.

```rust,no_run
use egui_expressive::interaction::{UndoEntry, UndoStack};

let mut history = UndoStack::new("draft".to_owned());
history.push(UndoEntry::new("draft v2".to_owned()).label("Typing"));
let previous = history.undo();
# let _ = previous;
```

## Feedback Dispatch

`FeedbackQueue` is the runtime policy model for feedback:

- one active modal
- FIFO snackbars with one visible snackbar
- bounded toast stack with per-toast TTL
- progress entries coexist with toasts/snackbars but are suppressed while a modal is active
- modal/snackbar dismissal returns optional focus target for caller-side restoration
- notification-center retention is in-memory only

```rust,no_run
use egui_expressive::interaction::{FeedbackMessage, FeedbackQueue, FeedbackToast};

let mut feedback = FeedbackQueue::new();
feedback.push_snackbar(FeedbackMessage::new("saved", "Saved"));
feedback.push_toast(FeedbackToast::new("done", "Export complete", 3.0));
feedback.tick(0.16);
```

Use `Toast::from_feedback` and `ToastLayer` when adapting dispatcher toasts into the existing overlay toast surface.

`ToastLayer::show(ctx)` is the app-level floating overlay path and requests repaint while toasts are visible. `ui.add(ToastLayer::new(...))` renders in the caller's current layout flow and is intended for embedded/demo layouts.

## Stage Deferrals

- Stage 5 owns schema-driven forms, rich input correctness, masked/date/file picker adapters, and inline data/property editing.
- Stage 6 owns editor/canvas command graphs, object-history integration, and deeper canvas interaction semantics.
- Stage 8 owns platform accessibility adapters, live-region guidance, IME/RTL/i18n, clipboard/file dialogs/file drop, and screen-reader claims.
- Stage 9 owns shortcut/history capacity/performance hardening, full visual-regression expansion, and release support boundaries.
