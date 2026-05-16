use super::registry::PropRegistry;

/// DevTools floating panel for property editing
pub struct DevToolsPanel;

impl DevToolsPanel {
    /// Show the dev tools panel as a floating egui window.
    /// `open` controls visibility.
    pub fn show(ctx: &egui::Context, open: &mut bool) {
        #[cfg(debug_assertions)]
        {
            let registry = PropRegistry::get(ctx);
            let mut reg = registry.lock().unwrap();

            if !*open {
                return;
            }

            egui::Window::new("Dev Tools — Properties")
                .open(open)
                .resizable(true)
                .default_width(320.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Reset All").clicked() {
                            reg.reset_all();
                        }
                        if ui.button("Export Rust").clicked() {
                            let exported = reg.export_rust();
                            println!("{}", exported);
                            ctx.copy_text(exported);
                        }
                    });

                    ui.separator();

                    let groups = reg.groups();
                    for group in groups {
                        let keys = reg.keys_in_group(&group);
                        if keys.is_empty() {
                            continue;
                        }
                        egui::CollapsingHeader::new(&group)
                            .default_open(true)
                            .show(ui, |ui| {
                                for key in keys {
                                    let prop = reg.props.get_mut(&key);
                                    if let Some(prop) = prop {
                                        let name = prop.name.clone();
                                        ui.horizontal(|ui| {
                                            ui.label(&name);
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    let prop = reg.props.get_mut(&key).unwrap();
                                                    PropRegistry::edit_prop_ui(ui, prop);
                                                },
                                            );
                                        });
                                    }
                                }
                            });
                    }
                });
        }

        #[cfg(not(debug_assertions))]
        {
            let _ = (ctx, open);
        }
    }
}
