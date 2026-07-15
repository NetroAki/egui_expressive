use eframe::egui;

pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "workspace-sans".to_owned(),
        egui::FontData::from_static(include_bytes!("assets/AdwaitaSans-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        "workspace-mono".to_owned(),
        egui::FontData::from_static(include_bytes!("assets/AdwaitaMono-Regular.ttf")).into(),
    );

    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.insert(0, "workspace-sans".to_owned());
    }
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        family.insert(0, "workspace-mono".to_owned());
    }
    ctx.set_fonts(fonts);
}
