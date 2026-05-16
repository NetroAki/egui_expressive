//! Validation metadata for form fields.

use std::collections::BTreeMap;

use crate::forms::FormFieldValue;

/// Severity for help/warning/error text under a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValidationSeverity {
    Help,
    Warning,
    Error,
}

/// Human-readable validation or helper message.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValidationMessage {
    pub severity: ValidationSeverity,
    pub text: String,
}

impl ValidationMessage {
    pub fn help(text: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Help,
            text: text.into(),
        }
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            text: text.into(),
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            text: text.into(),
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self.severity {
            ValidationSeverity::Help => egui::Color32::from_gray(150),
            ValidationSeverity::Warning => egui::Color32::from_rgb(220, 165, 55),
            ValidationSeverity::Error => egui::Color32::from_rgb(230, 90, 90),
        }
    }
}

/// Pure validation rule kinds supported by Forms v2.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ValidationRuleKind {
    Required,
    MinLength(usize),
    MaxLength(usize),
    NumericRange { min: f64, max: f64 },
    Contains(String),
}

/// Validation rule tied to one schema field id.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidationRule {
    pub field_id: String,
    pub kind: ValidationRuleKind,
    pub message: ValidationMessage,
}

impl ValidationRule {
    pub fn new(
        field_id: impl Into<String>,
        kind: ValidationRuleKind,
        message: ValidationMessage,
    ) -> Self {
        Self {
            field_id: field_id.into(),
            kind,
            message,
        }
    }

    /// Evaluates this rule against a field value.
    ///
    /// Rules are value-kind specific. A mismatched value variant is treated as
    /// not applicable rather than invalid so heterogeneous schemas can keep one
    /// rule list and rely on schema construction/tests to pair rule kinds with
    /// compatible `FormFieldValue` variants.
    pub fn evaluate(&self, value: Option<&FormFieldValue>) -> Option<ValidationMessage> {
        let valid = match (&self.kind, value) {
            (ValidationRuleKind::Required, Some(value)) => !value.is_empty(),
            (ValidationRuleKind::Required, None) => false,
            (ValidationRuleKind::MinLength(min), Some(FormFieldValue::Text(value))) => {
                value.chars().count() >= *min
            }
            (ValidationRuleKind::MaxLength(max), Some(FormFieldValue::Text(value))) => {
                value.chars().count() <= *max
            }
            (
                ValidationRuleKind::NumericRange { min, max },
                Some(FormFieldValue::Number(value)),
            ) => value >= min && value <= max,
            (ValidationRuleKind::Contains(needle), Some(FormFieldValue::Text(value))) => {
                value.contains(needle)
            }
            (ValidationRuleKind::MinLength(_) | ValidationRuleKind::MaxLength(_), _) => true,
            (ValidationRuleKind::NumericRange { .. }, _) => true,
            (ValidationRuleKind::Contains(_), _) => true,
        };

        (!valid).then(|| self.message.clone())
    }
}

/// Runtime-free deferred validation request for app-owned async/work queue execution.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeferredValidationRequest {
    pub field_id: String,
    pub validator_id: String,
    pub value: FormFieldValue,
}

impl DeferredValidationRequest {
    pub fn new(
        field_id: impl Into<String>,
        validator_id: impl Into<String>,
        value: FormFieldValue,
    ) -> Self {
        Self {
            field_id: field_id.into(),
            validator_id: validator_id.into(),
            value,
        }
    }
}

/// Grouped validation output for summary panels and field shells.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidationSummary {
    messages: BTreeMap<String, Vec<ValidationMessage>>,
}

impl ValidationSummary {
    pub fn from_rules(values: &BTreeMap<String, FormFieldValue>, rules: &[ValidationRule]) -> Self {
        let mut summary = Self::default();
        for rule in rules {
            if let Some(message) = rule.evaluate(values.get(&rule.field_id)) {
                summary.push(rule.field_id.clone(), message);
            }
        }
        summary
    }

    pub fn push(&mut self, field_id: impl Into<String>, message: ValidationMessage) {
        self.messages
            .entry(field_id.into())
            .or_default()
            .push(message);
    }

    pub fn messages_for(&self, field_id: &str) -> &[ValidationMessage] {
        self.messages
            .get(field_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn has_errors(&self) -> bool {
        self.messages
            .values()
            .flatten()
            .any(|message| matches!(message.severity, ValidationSeverity::Error))
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

#[cfg(test)]
mod forms_v2_tests {
    use super::*;

    #[test]
    fn validation_summary_groups_rule_failures() {
        let values = BTreeMap::from([("name".to_owned(), FormFieldValue::Text(String::new()))]);
        let rules = vec![ValidationRule::new(
            "name",
            ValidationRuleKind::Required,
            ValidationMessage::error("Name required"),
        )];

        let summary = ValidationSummary::from_rules(&values, &rules);

        assert!(summary.has_errors());
        assert_eq!(summary.messages_for("name")[0].text, "Name required");
    }

    #[test]
    fn deferred_validation_request_is_plain_data() {
        let request = DeferredValidationRequest::new(
            "slug",
            "unique-slug",
            FormFieldValue::Text("demo".to_owned()),
        );

        assert_eq!(request.validator_id, "unique-slug");
    }

    #[test]
    fn value_variant_mismatch_is_not_applicable() {
        let rule = ValidationRule::new(
            "gain",
            ValidationRuleKind::MinLength(3),
            ValidationMessage::error("Too short"),
        );

        assert_eq!(rule.evaluate(Some(&FormFieldValue::Number(1.0))), None);
    }
}
