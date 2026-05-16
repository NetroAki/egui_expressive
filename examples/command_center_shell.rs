use eframe::egui;
use egui_expressive::interaction::{
    next_focus_in_order, ActionDef, ActionRegistry, FeedbackMessage, FeedbackProgress,
    FeedbackQueue, FeedbackToast, FocusDirection, ScopedShortcutBinding, ScopedShortcutRegistry,
    ShortcutBinding, ShortcutResolution, ShortcutScope, UndoStack,
};
use egui_expressive::widgets::{
    CommandPalette, CommandPaletteItem, MenuDef, Toast, ToastLayer, TopMenuBar,
};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Command Center Shell",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(CommandCenterShell::default()))),
    )
}

struct CommandCenterShell {
    actions: ActionRegistry,
    shortcuts: ScopedShortcutRegistry,
    palette_items: Vec<CommandPaletteItem>,
    menus: Vec<MenuDef>,
    palette_open: bool,
    palette_query: String,
    palette_selected: usize,
    palette_activated: Option<String>,
    menu_activated: Option<String>,
    modal_open: bool,
    focused_widget: Option<egui::Id>,
    history: UndoStack<usize>,
    feedback: FeedbackQueue,
    visual_toasts: Vec<Toast>,
    last_event: String,
}

impl Default for CommandCenterShell {
    fn default() -> Self {
        let actions = build_actions();
        let shortcuts = build_shortcuts();
        let palette_items = CommandPaletteItem::from_registry(&actions);
        let menus = vec![MenuDef::actions("Command", actions.iter().cloned())];
        Self {
            actions,
            shortcuts,
            palette_items,
            menus,
            palette_open: false,
            palette_query: String::new(),
            palette_selected: 0,
            palette_activated: None,
            menu_activated: None,
            modal_open: false,
            focused_widget: None,
            history: UndoStack::new(0),
            feedback: FeedbackQueue::new(),
            visual_toasts: Vec::new(),
            last_event: "Ready".to_owned(),
        }
    }
}

impl eframe::App for CommandCenterShell {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_shortcuts(&ctx);
        self.render_modal_feedback(&ctx);
        self.render_palette(&ctx);
        self.render_shell(ui);
        ToastLayer::new(&mut self.visual_toasts).show(&ctx);
    }
}

impl CommandCenterShell {
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        match self
            .shortcuts
            .resolve_pressed(ctx, &self.active_scopes(), &self.actions)
        {
            ShortcutResolution::Dispatched { action_id, .. } => self.dispatch(&action_id),
            ShortcutResolution::Disabled { action_id, .. } => {
                self.last_event = format!("Shortcut consumed by disabled action: {action_id}");
            }
            ShortcutResolution::Unknown { action_id, .. } => {
                self.last_event = format!("Shortcut consumed by unknown action: {action_id}");
            }
            ShortcutResolution::Trapped { scope } => {
                self.last_event = format!("Shortcut trapped by {scope:?}");
            }
            ShortcutResolution::NoMatch => {}
        }
    }

    fn active_scopes(&self) -> Vec<ShortcutScope> {
        let mut scopes = vec![
            ShortcutScope::Global,
            ShortcutScope::FocusedPanel("main".to_owned()),
        ];
        if self.palette_open {
            scopes.push(ShortcutScope::Overlay("command_palette".to_owned()));
        }
        if self.modal_open || self.feedback.active_modal().is_some() {
            scopes.push(ShortcutScope::Modal("feedback".to_owned()));
        }
        scopes
    }

    fn dispatch(&mut self, action_id: &str) {
        match action_id {
            "app.save" => self.save_snapshot(),
            "app.undo" => self.undo(),
            "app.redo" => self.redo(),
            "app.palette" => self.palette_open = true,
            "app.progress" => self.show_progress(),
            "app.modal" => self.show_modal(),
            _ => self.last_event = format!("Unknown action: {action_id}"),
        }
    }

    fn save_snapshot(&mut self) {
        let next = *self.history.current() + 1;
        self.history
            .push(egui_expressive::interaction::UndoEntry::new(next).label("Save snapshot"));
        self.push_toast("save", format!("Saved snapshot {next}"));
        self.last_event = "Dispatched Save from command spine".to_owned();
    }

    fn undo(&mut self) {
        let message = match self.history.undo() {
            Some(snapshot) => format!("Undo → snapshot {snapshot}"),
            None => "Nothing to undo".to_owned(),
        };
        self.push_snackbar("undo", message.clone());
        self.last_event = message;
    }

    fn redo(&mut self) {
        let message = match self.history.redo() {
            Some(snapshot) => format!("Redo → snapshot {snapshot}"),
            None => "Nothing to redo".to_owned(),
        };
        self.push_snackbar("redo", message.clone());
        self.last_event = message;
    }

    fn show_progress(&mut self) {
        self.feedback.push_progress(FeedbackProgress::new(
            "sync",
            "Syncing command center",
            Some(0.66),
        ));
        self.last_event = "Progress feedback dispatched".to_owned();
    }

    fn show_modal(&mut self) {
        self.modal_open = true;
        self.feedback.push_modal(
            FeedbackMessage::new("confirm", "Modal feedback traps unmatched shortcuts")
                .focus_return(egui::Id::new("open_modal")),
        );
        self.last_event = "Modal feedback dispatched".to_owned();
    }

    fn push_toast(&mut self, id: &str, message: String) {
        let toast = FeedbackToast::new(id, message, 3.0);
        self.visual_toasts.push(Toast::from_feedback(&toast));
        self.feedback.push_toast(toast);
    }

    fn push_snackbar(&mut self, id: &str, message: String) {
        self.feedback
            .push_snackbar(FeedbackMessage::new(id, message));
    }

    fn render_shell(&mut self, ui: &mut egui::Ui) {
        ui.add(TopMenuBar::new(&self.menus).activated(&mut self.menu_activated));
        if let Some(action_id) = self.menu_activated.take() {
            self.dispatch(&action_id);
        }
        ui.separator();
        ui.heading("Stage 4 command center shell");
        ui.label("Menus, palette, shortcuts, focus traversal, undo/redo, and feedback share interaction-layer primitives.");
        ui.separator();
        self.render_command_buttons(ui);
        ui.separator();
        self.render_focus_demo(ui);
        ui.separator();
        self.render_feedback_summary(ui);
    }

    fn render_command_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Save snapshot (Ctrl+S)").clicked() {
                self.dispatch("app.save");
            }
            if ui.button("Undo (Ctrl+Z)").clicked() {
                self.dispatch("app.undo");
            }
            if ui.button("Redo (Ctrl+Y)").clicked() {
                self.dispatch("app.redo");
            }
            if ui.button("Palette (Ctrl+K)").clicked() {
                self.dispatch("app.palette");
            }
            if ui.button("Progress").clicked() {
                self.dispatch("app.progress");
            }
            if ui.button("Modal").clicked() {
                self.dispatch("app.modal");
            }
        });
        ui.label(format!("Current undo snapshot: {}", self.history.current()));
        ui.label(format!("Last event: {}", self.last_event));
    }

    fn render_focus_demo(&mut self, ui: &mut egui::Ui) {
        let focus_order = vec![
            egui::Id::new("save_button"),
            egui::Id::new("palette_button"),
            egui::Id::new("feedback_button"),
        ];
        ui.horizontal(|ui| {
            if ui.button("Next focus target").clicked() {
                self.focused_widget =
                    next_focus_in_order(&focus_order, self.focused_widget, FocusDirection::Forward);
            }
            if ui.button("Previous focus target").clicked() {
                self.focused_widget = next_focus_in_order(
                    &focus_order,
                    self.focused_widget,
                    FocusDirection::Backward,
                );
            }
        });
        ui.label(format!(
            "Pure focus traversal target: {:?}",
            self.focused_widget
        ));
    }

    fn render_feedback_summary(&mut self, ui: &mut egui::Ui) {
        if let Some(snackbar) = self.feedback.visible_snackbar().cloned() {
            ui.horizontal(|ui| {
                ui.label(format!("Snackbar: {}", snackbar.message));
                if ui.button("Dismiss").clicked() {
                    self.feedback.dismiss_snackbar();
                }
            });
        }
        for progress in self.feedback.visible_progress() {
            ui.label(&progress.label);
            ui.add(egui::ProgressBar::new(progress.fraction.unwrap_or(0.0)));
        }
        ui.label(format!(
            "Notification center entries: {}",
            self.feedback.notifications().len()
        ));
    }

    fn render_palette(&mut self, ctx: &egui::Context) {
        if !self.palette_open {
            return;
        }
        egui::Window::new("Command palette")
            .collapsible(false)
            .show(ctx, |ui| {
                ui.add(
                    CommandPalette::new(&mut self.palette_query, &self.palette_items)
                        .selected(&mut self.palette_selected)
                        .activated(&mut self.palette_activated),
                );
                if ui.button("Close palette").clicked() {
                    self.palette_open = false;
                }
            });
        if let Some(action_id) = self.palette_activated.take() {
            self.palette_open = false;
            self.dispatch(&action_id);
        }
    }

    fn render_modal_feedback(&mut self, ctx: &egui::Context) {
        let Some(modal) = self.feedback.active_modal().cloned() else {
            return;
        };
        egui::Window::new("Feedback modal")
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(&modal.message);
                ui.label("Try an unmatched shortcut: the modal scope traps it.");
                if ui.button("Dismiss modal").clicked() {
                    self.modal_open = false;
                    let focus_return = self.feedback.dismiss_modal();
                    self.last_event = format!("Modal dismissed, focus return: {focus_return:?}");
                }
            });
    }
}

fn build_actions() -> ActionRegistry {
    let mut actions = ActionRegistry::new();
    for action in [
        ActionDef::new("app.save", "Save snapshot").description("Push a history entry"),
        ActionDef::new("app.undo", "Undo").description("Move back in history"),
        ActionDef::new("app.redo", "Redo").description("Move forward in history"),
        ActionDef::new("app.palette", "Open palette").description("Show command discovery"),
        ActionDef::new("app.progress", "Show progress").description("Dispatch progress feedback"),
        ActionDef::new("app.modal", "Show modal").description("Dispatch modal feedback"),
    ] {
        actions.register(action);
    }
    actions
}

fn build_shortcuts() -> ScopedShortcutRegistry {
    let mut shortcuts = ScopedShortcutRegistry::new();
    for (scope, action, key, modifiers) in [
        (
            ShortcutScope::Global,
            "app.save",
            egui::Key::S,
            egui::Modifiers::CTRL,
        ),
        (
            ShortcutScope::Global,
            "app.undo",
            egui::Key::Z,
            egui::Modifiers::CTRL,
        ),
        (
            ShortcutScope::Global,
            "app.redo",
            egui::Key::Y,
            egui::Modifiers::CTRL,
        ),
        (
            ShortcutScope::Global,
            "app.palette",
            egui::Key::K,
            egui::Modifiers::CTRL,
        ),
        (
            ShortcutScope::Overlay("command_palette".to_owned()),
            "app.palette",
            egui::Key::K,
            egui::Modifiers::CTRL,
        ),
    ] {
        shortcuts
            .bind(ScopedShortcutBinding::new(
                scope,
                ShortcutBinding::new(action, key, modifiers),
            ))
            .expect("example shortcuts should not conflict");
    }
    shortcuts
}
