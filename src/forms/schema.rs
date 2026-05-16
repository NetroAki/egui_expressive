use std::collections::BTreeMap;

use crate::forms::ValidationRule;

/// Supported schema field kinds for Forms v2.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FormFieldKind {
    Text,
    TextArea,
    Checkbox,
    Switch,
    Select,
    MultiSelect,
    Autocomplete,
    Numeric,
    Date,
    Time,
    Color,
    FilePath,
}

/// Serde-friendly form value model used by schema, validation, and editing helpers.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FormFieldValue {
    Empty,
    Text(String),
    Bool(bool),
    Number(f64),
    Date { year: i32, month: u8, day: u8 },
    Time { hour: u8, minute: u8 },
    Color { r: u8, g: u8, b: u8, a: u8 },
    List(Vec<String>),
    FilePath(String),
}

impl FormFieldValue {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Text(value) | Self::FilePath(value) => value.trim().is_empty(),
            Self::List(values) => values.is_empty(),
            _ => false,
        }
    }
}

/// Effect applied when a field dependency matches another field's value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DependencyEffect {
    Show,
    Hide,
    Enable,
    Disable,
    Require,
    Optional,
}

/// Pure dependency rule between two schema fields.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldDependency {
    pub source_id: String,
    pub expected: FormFieldValue,
    pub effect: DependencyEffect,
}

impl FieldDependency {
    pub fn new(
        source_id: impl Into<String>,
        expected: FormFieldValue,
        effect: DependencyEffect,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            expected,
            effect,
        }
    }

    pub fn matches(&self, values: &BTreeMap<String, FormFieldValue>) -> bool {
        values
            .get(&self.source_id)
            .map(|value| value == &self.expected)
            .unwrap_or(false)
    }
}

/// Derived runtime state for a schema field after dependencies are evaluated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DerivedFieldState {
    pub visible: bool,
    pub enabled: bool,
    pub required: bool,
}

impl Default for DerivedFieldState {
    fn default() -> Self {
        Self {
            visible: true,
            enabled: true,
            required: false,
        }
    }
}

impl DerivedFieldState {
    fn apply(&mut self, effect: DependencyEffect) {
        match effect {
            DependencyEffect::Show => self.visible = true,
            DependencyEffect::Hide => self.visible = false,
            DependencyEffect::Enable => self.enabled = true,
            DependencyEffect::Disable => self.enabled = false,
            DependencyEffect::Require => self.required = true,
            DependencyEffect::Optional => self.required = false,
        }
    }
}

/// Schema-level description of one form field.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FormFieldDef {
    pub id: String,
    pub label: String,
    pub kind: FormFieldKind,
    pub help: Option<String>,
    pub action_id: Option<String>,
    pub focus_id: Option<String>,
    pub validation_rules: Vec<ValidationRule>,
    pub dependencies: Vec<FieldDependency>,
}

impl FormFieldDef {
    pub fn new(id: impl Into<String>, label: impl Into<String>, kind: FormFieldKind) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            help: None,
            action_id: None,
            focus_id: None,
            validation_rules: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn action_id(mut self, action_id: impl Into<String>) -> Self {
        self.action_id = Some(action_id.into());
        self
    }

    pub fn focus_id(mut self, focus_id: impl Into<String>) -> Self {
        self.focus_id = Some(focus_id.into());
        self
    }

    pub fn rule(mut self, rule: ValidationRule) -> Self {
        self.validation_rules.push(rule);
        self
    }

    pub fn dependency(mut self, dependency: FieldDependency) -> Self {
        self.dependencies.push(dependency);
        self
    }
}

/// Ordered schema for rendering, validation, and focus traversal.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FormSchema {
    pub fields: Vec<FormFieldDef>,
}

impl FormSchema {
    pub fn new(fields: impl Into<Vec<FormFieldDef>>) -> Self {
        Self {
            fields: fields.into(),
        }
    }

    pub fn field(&self, id: &str) -> Option<&FormFieldDef> {
        self.fields.iter().find(|field| field.id == id)
    }

    pub fn focus_order(&self) -> Vec<String> {
        self.fields
            .iter()
            .filter_map(|field| field.focus_id.clone())
            .collect()
    }

    pub fn evaluate_dependencies(
        &self,
        values: &BTreeMap<String, FormFieldValue>,
    ) -> BTreeMap<String, DerivedFieldState> {
        let mut states = BTreeMap::new();
        for field in &self.fields {
            let mut state = DerivedFieldState::default();
            for dependency in &field.dependencies {
                if dependency.matches(values) {
                    state.apply(dependency.effect);
                }
            }
            states.insert(field.id.clone(), state);
        }
        states
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_rules_derive_visibility_and_required_state() {
        let schema = FormSchema::new(vec![
            FormFieldDef::new("advanced", "Advanced", FormFieldKind::Switch),
            FormFieldDef::new("token", "Token", FormFieldKind::Text)
                .dependency(FieldDependency::new(
                    "advanced",
                    FormFieldValue::Bool(true),
                    DependencyEffect::Show,
                ))
                .dependency(FieldDependency::new(
                    "advanced",
                    FormFieldValue::Bool(true),
                    DependencyEffect::Require,
                )),
        ]);
        let values = BTreeMap::from([("advanced".to_owned(), FormFieldValue::Bool(true))]);

        let states = schema.evaluate_dependencies(&values);

        assert!(states["token"].visible);
        assert!(states["token"].required);
    }

    #[test]
    fn focus_order_uses_schema_focus_ids() {
        let schema = FormSchema::new(vec![
            FormFieldDef::new("name", "Name", FormFieldKind::Text).focus_id("focus.name"),
            FormFieldDef::new("notes", "Notes", FormFieldKind::TextArea),
        ]);

        assert_eq!(schema.focus_order(), vec!["focus.name".to_owned()]);
    }
}
