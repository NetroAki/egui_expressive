//! Generic inspector descriptors for selected editor objects.

use crate::forms::FormFieldValue;

#[derive(Debug, Clone, PartialEq)]
pub struct EditorInspectorField {
    pub id: String,
    pub label: String,
    pub value: FormFieldValue,
    pub read_only: bool,
}

impl EditorInspectorField {
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: FormFieldValue) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value,
            read_only: false,
        }
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorInspectorTarget<K> {
    pub id: K,
    pub label: String,
    pub fields: Vec<EditorInspectorField>,
}

impl<K> EditorInspectorTarget<K> {
    pub fn new(
        id: K,
        label: impl Into<String>,
        fields: impl IntoIterator<Item = EditorInspectorField>,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            fields: fields.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorInspectorUpdate<K> {
    pub target_id: K,
    pub field_id: String,
    pub value: FormFieldValue,
}

impl<K> EditorInspectorUpdate<K> {
    pub fn new(target_id: K, field_id: impl Into<String>, value: FormFieldValue) -> Self {
        Self {
            target_id,
            field_id: field_id.into(),
            value,
        }
    }
}

pub fn apply_inspector_update<K: PartialEq>(
    target: &mut EditorInspectorTarget<K>,
    update: EditorInspectorUpdate<K>,
) -> bool {
    if target.id != update.target_id {
        return false;
    }
    let Some(field) = target
        .fields
        .iter_mut()
        .find(|field| field.id == update.field_id && !field.read_only)
    else {
        return false;
    };
    field.value = update.value;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspector_update_changes_editable_field_only() {
        let mut target = EditorInspectorTarget::new(
            7,
            "Node",
            [
                EditorInspectorField::new("x", "X", FormFieldValue::Number(1.0)),
                EditorInspectorField::new("id", "Id", FormFieldValue::Text("7".into()))
                    .read_only(true),
            ],
        );

        assert!(apply_inspector_update(
            &mut target,
            EditorInspectorUpdate::new(7, "x", FormFieldValue::Number(2.0))
        ));
        assert!(!apply_inspector_update(
            &mut target,
            EditorInspectorUpdate::new(7, "id", FormFieldValue::Text("8".into()))
        ));
        assert_eq!(target.fields[0].value, FormFieldValue::Number(2.0));
    }
}
