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
    pub(crate) props: HashMap<String, Prop>,
    pub(crate) order: Vec<String>,
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

    pub fn color(&self, name: &str) -> egui::Color32 {
        if let Some(prop) = self.props.get(name) {
            if let PropValue::Color(c) = &prop.value {
                return *c;
            }
        }
        egui::Color32::TRANSPARENT
    }

    pub fn float(&self, name: &str) -> f32 {
        if let Some(prop) = self.props.get(name) {
            if let PropValue::Float(v) = &prop.value {
                return *v;
            }
        }
        0.0
    }

    pub fn bool_val(&self, name: &str) -> bool {
        if let Some(prop) = self.props.get(name) {
            if let PropValue::Bool(b) = &prop.value {
                return *b;
            }
        }
        false
    }

    pub fn vec2(&self, name: &str) -> egui::Vec2 {
        if let Some(prop) = self.props.get(name) {
            if let PropValue::Vec2(v) = &prop.value {
                return *v;
            }
        }
        egui::Vec2::ZERO
    }

    pub fn rounding(&self, name: &str) -> f32 {
        if let Some(prop) = self.props.get(name) {
            if let PropValue::Rounding(r) = &prop.value {
                return *r;
            }
        }
        0.0
    }

    pub fn reset_all(&mut self) {
        for prop in self.props.values_mut() {
            prop.value = prop.default.clone();
        }
    }

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
                    colors.push(format!("const {}: egui::Color32 = egui::Color32::from_rgba_unmultiplied({}, {}, {}, {});", const_name, r, g, b, a));
                }
                PropValue::Float(v) | PropValue::Rounding(v) => {
                    floats.push(format!("const {}: f32 = {};", const_name, v))
                }
                PropValue::Vec2(v) => floats.push(format!(
                    "const {}: egui::Vec2 = egui::Vec2::new({}, {});",
                    const_name, v.x, v.y
                )),
                PropValue::Bool(b) => bools.push(format!("const {}: bool = {};", const_name, b)),
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

    #[allow(dead_code)]
    pub(crate) fn edit_prop_ui(ui: &mut Ui, prop: &mut Prop) {
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

    #[allow(dead_code)]
    pub(crate) fn groups(&self) -> Vec<String> {
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

    #[allow(dead_code)]
    pub(crate) fn keys_in_group(&self, group: &str) -> Vec<String> {
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

pub(crate) fn to_screaming_snake(group: &str, name: &str) -> String {
    let group_upper = group.to_uppercase().replace([' ', '/'], "_");
    let name_upper = name.to_uppercase().replace([' ', '/'], "_");
    if group_upper.is_empty() {
        name_upper
    } else {
        format!("{}_{}", group_upper, name_upper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_and_lookup() {
        let mut registry = PropRegistry::default();
        registry.register_bool("A", "flag", true);
        assert!(registry.bool_val("flag"));
        let exported = registry.export_rust();
        assert!(exported.contains("const A_FLAG: bool = true;"));
    }
}
