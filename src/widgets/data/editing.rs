use crate::forms::{FormFieldKind, FormFieldValue, InlineEditCommit, InlineEditTarget};

/// Editable data-cell descriptor that keeps Stage 3 grid models read-only.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DataCellEditSpec {
    pub row_id: String,
    pub column_id: String,
    pub kind: FormFieldKind,
    pub value: FormFieldValue,
}

impl DataCellEditSpec {
    pub fn new(
        row_id: impl Into<String>,
        column_id: impl Into<String>,
        kind: FormFieldKind,
        value: FormFieldValue,
    ) -> Self {
        Self {
            row_id: row_id.into(),
            column_id: column_id.into(),
            kind,
            value,
        }
    }

    pub fn target(&self) -> InlineEditTarget {
        InlineEditTarget::data_cell(self.row_id.clone(), self.column_id.clone())
    }

    pub fn apply_commit(&mut self, commit: &InlineEditCommit) -> bool {
        if commit.target == self.target() {
            self.value = commit.value.clone();
            true
        } else {
            false
        }
    }
}

/// Editable property descriptor layered over `PropertyGridEntry` metadata.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PropertyEditSpec {
    pub category: String,
    pub name: String,
    pub kind: FormFieldKind,
    pub value: FormFieldValue,
}

impl PropertyEditSpec {
    pub fn new(
        category: impl Into<String>,
        name: impl Into<String>,
        kind: FormFieldKind,
        value: FormFieldValue,
    ) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
            kind,
            value,
        }
    }

    pub fn target(&self) -> InlineEditTarget {
        InlineEditTarget::property(self.category.clone(), self.name.clone())
    }

    pub fn apply_commit(&mut self, commit: &InlineEditCommit) -> bool {
        if commit.target == self.target() {
            self.value = commit.value.clone();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_cell_edit_spec_accepts_matching_commit() {
        let mut spec = DataCellEditSpec::new(
            "row-1",
            "gain",
            FormFieldKind::Text,
            FormFieldValue::Text("0 dB".to_owned()),
        );
        let commit = InlineEditCommit {
            target: InlineEditTarget::data_cell("row-1", "gain"),
            value: FormFieldValue::Text("-6 dB".to_owned()),
        };

        assert!(spec.apply_commit(&commit));
        assert_eq!(spec.value, FormFieldValue::Text("-6 dB".to_owned()));
    }

    #[test]
    fn property_edit_spec_ignores_other_targets() {
        let mut spec = PropertyEditSpec::new(
            "Layout",
            "Width",
            FormFieldKind::Numeric,
            FormFieldValue::Number(128.0),
        );
        let commit = InlineEditCommit {
            target: InlineEditTarget::property("Layout", "Height"),
            value: FormFieldValue::Number(64.0),
        };

        assert!(!spec.apply_commit(&commit));
        assert_eq!(spec.value, FormFieldValue::Number(128.0));
    }
}
