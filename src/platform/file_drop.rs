//! File-drop descriptors for egui raw dropped-file input.

use crate::editor::{EditorDropItem, EditorDropKind, EditorDropRequest};

/// Dependency-free dropped-file summary. Paths/bytes are intentionally app-owned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DroppedFileDescriptor {
    pub id: String,
    pub label: String,
    pub mime_type: Option<String>,
}

impl DroppedFileDescriptor {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            mime_type: None,
        }
    }

    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    pub fn to_editor_item(&self) -> EditorDropItem {
        let item = EditorDropItem::new(&self.id, &self.label, EditorDropKind::FilePath);
        match &self.mime_type {
            Some(mime_type) => item.mime_type(mime_type.clone()),
            None => item,
        }
    }
}

/// Batch of platform drop descriptors destined for one UI target.
#[derive(Clone, Debug, PartialEq)]
pub struct PlatformDropBatch {
    pub target: egui::Pos2,
    pub files: Vec<DroppedFileDescriptor>,
}

impl PlatformDropBatch {
    pub fn new(target: egui::Pos2, files: impl IntoIterator<Item = DroppedFileDescriptor>) -> Self {
        Self {
            target,
            files: files.into_iter().collect(),
        }
    }

    pub fn to_editor_request(&self) -> EditorDropRequest {
        EditorDropRequest::new(
            self.target,
            self.files.iter().map(DroppedFileDescriptor::to_editor_item),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_batch_maps_to_editor_drop_request() {
        let batch = PlatformDropBatch::new(
            egui::pos2(4.0, 8.0),
            [DroppedFileDescriptor::new("file-1", "clip.wav").mime_type("audio/wav")],
        );

        let request = batch.to_editor_request();

        assert!(request.accepts_kind(EditorDropKind::FilePath));
        assert_eq!(request.items[0].mime_type.as_deref(), Some("audio/wav"));
    }
}
