//! Runtime visual property editor — "browser DevTools for egui".
//!
//! Register named properties at runtime, edit them live in an egui panel,
//! then export the current values as Rust source code.
//!
//! ## Feature
//!
//! `DevToolsPanel::show()` is a no-op in release builds (`#[cfg(debug_assertions)]`).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[allow(unused_imports)]
use egui::{color_picker, CollapsingHeader, Ui};

/// A named editable property
#[derive(Clone, Debug)]
pub enum PropValue {
    Color(egui::Color32),
    Float(f32),
    Vec2(egui::Vec2),
    Rounding(f32),
    Bool(bool),
}

/// A registered property with metadata
#[derive(Clone, Debug)]
pub struct Prop {
    pub name: String,
    pub group: String,
    pub value: PropValue,
    pub default: PropValue,
    pub min: Option<f32>,
    pub max: Option<f32>,
}

/// The property registry — stored in egui memory
#[derive(Clone, Default)]
pub struct PropRegistry {
    props: HashMap<String, Prop>,
    order: Vec<String>,
}

impl PropRegistry {
    /// Get or create the registry from egui context
    pub fn get(ctx: &egui::Context) -> Arc<Mutex<PropRegistry>> {
        let id = egui::Id::new("egui_expressive::devtools::PropRegistry");

        ctx.data_mut(|data| {
            if let Some(reg) = data.get_temp::<Arc<Mutex<PropRegistry>>>(id) {
                reg
            } else {
                let reg: Arc<Mutex<PropRegistry>> = Arc::new(Mutex::new(PropRegistry::default()));
                data.insert_temp(id, reg.clone());
                reg
            }
        })
    }

    fn key(name: &str) -> String {
        name.to_owned()
    }

    /// Register a color property (idempotent — only sets default on first call)
    pub fn register_color(&mut self, group: &str, name: &str, default: egui::Color32) {
        let key = Self::key(name);
        if !self.props.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.props.entry(key).or_insert_with(|| Prop {
            name: name.to_owned(),
            group: group.to_owned(),
            value: PropValue::Color(default),
            default: PropValue::Color(default),
            min: None,
            max: None,
        });
    }

    /// Register a float property
    pub fn register_float(&mut self, group: &str, name: &str, default: f32, min: f32, max: f32) {
        let key = Self::key(name);
        if !self.props.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.props.entry(key).or_insert_with(|| Prop {
            name: name.to_owned(),
            group: group.to_owned(),
            value: PropValue::Float(default),
            default: PropValue::Float(default),
            min: Some(min),
            max: Some(max),
        });
    }

    /// Register a bool property
    pub fn register_bool(&mut self, group: &str, name: &str, default: bool) {
        let key = Self::key(name);
        if !self.props.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.props.entry(key).or_insert_with(|| Prop {
            name: name.to_owned(),
            group: group.to_owned(),
            value: PropValue::Bool(default),
            default: PropValue::Bool(default),
            min: None,
            max: None,
        });
    }

    /// Register a Vec2 property
    pub fn register_vec2(&mut self, group: &str, name: &str, default: egui::Vec2) {
        let key = Self::key(name);
        if !self.props.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.props.entry(key).or_insert_with(|| Prop {
            name: name.to_owned(),
            group: group.to_owned(),
            value: PropValue::Vec2(default),
            default: PropValue::Vec2(default),
            min: None,
            max: None,
        });
    }

    /// Register a rounding property (uniform rounding)
    pub fn register_rounding(&mut self, group: &str, name: &str, default: f32, min: f32, max: f32) {
        let key = Self::key(name);
        if !self.props.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.props.entry(key).or_insert_with(|| Prop {
            name: name.to_owned(),
            group: group.to_owned(),
            value: PropValue::Rounding(default),
            default: PropValue::Rounding(default),
            min: Some(min),
            max: Some(max),
        });
    }

    /// Get current value of a color property (returns default if not registered)
    pub fn color(&self, name: &str) -> egui::Color32 {
        if let Some(prop) = self.props.get(name) {
            if let PropValue::Color(c) = &prop.value {
                return *c;
            }
        }
        egui::Color32::TRANSPARENT
    }

    /// Get current value of a float property
    pub fn float(&self, name: &str) -> f32 {
        if let Some(prop) = self.props.get(name) {
            if let PropValue::Float(v) = &prop.value {
                return *v;
            }
        }
        0.0
    }

    /// Get current value of a bool property
    pub fn bool_val(&self, name: &str) -> bool {
        if let Some(prop) = self.props.get(name) {
            if let PropValue::Bool(b) = &prop.value {
                return *b;
            }
        }
        false
    }

    /// Get current value of a Vec2 property
    pub fn vec2(&self, name: &str) -> egui::Vec2 {
        if let Some(prop) = self.props.get(name) {
            if let PropValue::Vec2(v) = &prop.value {
                return *v;
            }
        }
        egui::Vec2::ZERO
    }

    /// Get current value of a rounding property
    pub fn rounding(&self, name: &str) -> f32 {
        if let Some(prop) = self.props.get(name) {
            if let PropValue::Rounding(r) = &prop.value {
                return *r;
            }
        }
        0.0
    }

    /// Reset all properties to their default values
    pub fn reset_all(&mut self) {
        for prop in self.props.values_mut() {
            prop.value = prop.default.clone();
        }
    }

    /// Export all current values as Rust source code
    pub fn export_rust(&self) -> String {
        use std::time::SystemTime;

        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| format!("{}", d.as_secs()))
            .unwrap_or_else(|_| "unknown".to_owned());

        let mut colors = Vec::new();
        let mut floats = Vec::new();
        let mut bools = Vec::new();

        for key in &self.order {
            let prop = match self.props.get(key) {
                Some(p) => p,
                None => continue,
            };

            let const_name = to_screaming_snake(&prop.group, &prop.name);

            match &prop.value {
                PropValue::Color(c) => {
                    let (r, g, b, a) = (c.r(), c.g(), c.b(), c.a());
                    colors.push(format!(
                        "const {}: egui::Color32 = egui::Color32::from_rgba_unmultiplied({}, {}, {}, {});",
                        const_name, r, g, b, a
                    ));
                }
                PropValue::Float(v) | PropValue::Rounding(v) => {
                    floats.push(format!("const {}: f32 = {};", const_name, v));
                }
                PropValue::Vec2(v) => {
                    floats.push(format!(
                        "const {}: egui::Vec2 = egui::Vec2::new({}, {});",
                        const_name, v.x, v.y
                    ));
                }
                PropValue::Bool(b) => {
                    bools.push(format!("const {}: bool = {};", const_name, b));
                }
            }
        }

        let mut output = format!(
            "// egui_expressive dev tools export\n// Generated: {}\n\n",
            timestamp
        );

        if !colors.is_empty() {
            output.push_str("// Colors\n");
            for c in colors {
                output.push_str(&c);
                output.push('\n');
            }
            output.push('\n');
        }

        if !floats.is_empty() {
            output.push_str("// Floats\n");
            for f in floats {
                output.push_str(&f);
                output.push('\n');
            }
            output.push('\n');
        }

        if !bools.is_empty() {
            output.push_str("// Bools\n");
            for b in bools {
                output.push_str(&b);
                output.push('\n');
            }
            output.push('\n');
        }

        output
    }

    /// Show the property editor UI for a single property, returning the modified prop if changed
    #[allow(dead_code)]
    fn edit_prop_ui(ui: &mut Ui, prop: &mut Prop) {
        match &mut prop.value {
            PropValue::Color(c) => {
                color_picker::color_edit_button_srgba(ui, c, egui::color_picker::Alpha::Opaque);
            }
            PropValue::Float(v) | PropValue::Rounding(v) => {
                let range = prop.min.unwrap_or(0.0)..=prop.max.unwrap_or(100.0);
                ui.add(egui::Slider::new(v, range).text(&prop.name));
            }
            PropValue::Vec2(v) => {
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut v.x, -1000.0..=1000.0).text("x"));
                    ui.add(egui::Slider::new(&mut v.y, -1000.0..=1000.0).text("y"));
                });
            }
            PropValue::Bool(b) => {
                ui.checkbox(b, &prop.name);
            }
        }
    }

    /// Collect all groups in insertion order
    #[allow(dead_code)]
    fn groups(&self) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        let mut groups = Vec::new();
        for key in &self.order {
            if let Some(prop) = self.props.get(key) {
                if seen.insert(prop.group.clone()) {
                    groups.push(prop.group.clone());
                }
            }
        }
        groups
    }

    /// Get keys of props belonging to a group, in insertion order
    #[allow(dead_code)]
    fn keys_in_group(&self, group: &str) -> Vec<String> {
        self.order
            .iter()
            .filter(|key| {
                self.props
                    .get(*key)
                    .map(|p| p.group == group)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
}

/// Convert "GroupName" + "property_name" to "GROUP_NAME_PROPERTY_NAME"
fn to_screaming_snake(group: &str, name: &str) -> String {
    let group_upper = group.to_uppercase().replace([' ', '/'], "_");
    let name_upper = name.to_uppercase().replace([' ', '/'], "_");
    if group_upper.is_empty() {
        name_upper
    } else {
        format!("{}_{}", group_upper, name_upper)
    }
}

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

                        CollapsingHeader::new(&group)
                            .default_open(true)
                            .show(ui, |ui| {
                                // For each key, we scope the mutable borrow
                                for key in keys {
                                    let prop = reg.props.get_mut(&key);
                                    if let Some(prop) = prop {
                                        let name = prop.name.clone();
                                        ui.horizontal(|ui| {
                                            ui.label(&name);
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    // Scope the mutable borrow to just this call
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

// ---------------------------------------------------------------------------
// Convenience macros
// ---------------------------------------------------------------------------

/// Register and read a color property in one call.
///
/// Usage:
/// ```ignore
/// let color = prop_color!(ctx, "Panel", "background", Color32::from_rgb(30, 30, 30));
/// ```
#[macro_export]
macro_rules! prop_color {
    ($ctx:expr, $group:expr, $name:expr, $default:expr) => {{
        let registry = $crate::devtools::PropRegistry::get($ctx);
        let mut reg = registry.lock().unwrap();
        reg.register_color($group, $name, $default);
        reg.color($name)
    }};
}

/// Register and read a float property in one call.
///
/// Usage:
/// ```ignore
/// let value = prop_float!(ctx, "Panel", "rounding", 8.0, 0.0, 24.0);
/// ```
#[macro_export]
macro_rules! prop_float {
    ($ctx:expr, $group:expr, $name:expr, $default:expr, $min:expr, $max:expr) => {{
        let registry = $crate::devtools::PropRegistry::get($ctx);
        let mut reg = registry.lock().unwrap();
        reg.register_float($group, $name, $default, $min, $max);
        reg.float($name)
    }};
}

/// Register and read a bool property in one call.
///
/// Usage:
/// ```ignore
/// let enabled = prop_bool!(ctx, "Panel", "show_shadow", true);
/// ```
#[macro_export]
macro_rules! prop_bool {
    ($ctx:expr, $group:expr, $name:expr, $default:expr) => {{
        let registry = $crate::devtools::PropRegistry::get($ctx);
        let mut reg = registry.lock().unwrap();
        reg.register_bool($group, $name, $default);
        reg.bool_val($name)
    }};
}

/// Register and read a rounding property in one call.
///
/// Usage:
/// ```ignore
/// let rounding = prop_rounding!(ctx, "Panel", "corner_radius", 8.0, 0.0, 24.0);
/// ```
#[macro_export]
macro_rules! prop_rounding {
    ($ctx:expr, $group:expr, $name:expr, $default:expr, $min:expr, $max:expr) => {{
        let registry = $crate::devtools::PropRegistry::get($ctx);
        let mut reg = registry.lock().unwrap();
        reg.register_rounding($group, $name, $default, $min, $max);
        reg.rounding($name)
    }};
}

/// Register and read a Vec2 property in one call.
///
/// Usage:
/// ```ignore
/// let padding = prop_vec2!(ctx, "Panel", "padding", Vec2::new(12.0, 8.0));
/// ```
#[macro_export]
macro_rules! prop_vec2 {
    ($ctx:expr, $group:expr, $name:expr, $default:expr) => {{
        let registry = $crate::devtools::PropRegistry::get($ctx);
        let mut reg = registry.lock().unwrap();
        reg.register_vec2($group, $name, $default);
        reg.vec2($name)
    }};
}
