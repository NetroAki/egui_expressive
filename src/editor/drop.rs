//! Pure file/object drop descriptors for editor surfaces.

use egui::Pos2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorDropKind {
    FilePath,
    Object,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDropItem {
    pub id: String,
    pub label: String,
    pub kind: EditorDropKind,
    pub mime_type: Option<String>,
}

impl EditorDropItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>, kind: EditorDropKind) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            mime_type: None,
        }
    }

    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorDropRequest {
    pub target: Pos2,
    pub items: Vec<EditorDropItem>,
}

impl EditorDropRequest {
    pub fn new(target: Pos2, items: impl IntoIterator<Item = EditorDropItem>) -> Self {
        Self {
            target,
            items: items.into_iter().collect(),
        }
    }

    pub fn accepted_items(&self, accepted: &[EditorDropKind]) -> Vec<&EditorDropItem> {
        self.items
            .iter()
            .filter(|item| accepted.iter().any(|kind| kind == &item.kind))
            .collect()
    }

    pub fn accepts_kind(&self, kind: EditorDropKind) -> bool {
        self.items.iter().any(|item| item.kind == kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn drop_request_filters_without_side_effects() {
        let request = EditorDropRequest::new(
            pos2(2.0, 3.0),
            [
                EditorDropItem::new("file", "clip.wav", EditorDropKind::FilePath),
                EditorDropItem::new("obj", "Rectangle", EditorDropKind::Object),
            ],
        );

        let accepted = request.accepted_items(&[EditorDropKind::Object]);
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].id, "obj");
        assert!(request.accepts_kind(EditorDropKind::FilePath));
    }
}
