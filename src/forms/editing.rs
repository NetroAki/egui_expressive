use crate::forms::FormFieldValue;

/// Target addressed by an inline form/data editing session.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InlineEditTarget {
    Field { field_id: String },
    Property { category: String, name: String },
    DataCell { row_id: String, column_id: String },
}

impl InlineEditTarget {
    pub fn field(field_id: impl Into<String>) -> Self {
        Self::Field {
            field_id: field_id.into(),
        }
    }

    pub fn property(category: impl Into<String>, name: impl Into<String>) -> Self {
        Self::Property {
            category: category.into(),
            name: name.into(),
        }
    }

    pub fn data_cell(row_id: impl Into<String>, column_id: impl Into<String>) -> Self {
        Self::DataCell {
            row_id: row_id.into(),
            column_id: column_id.into(),
        }
    }
}

/// What started an edit session; useful for focus restoration and undo labels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InlineEditStart {
    Keyboard,
    Pointer,
    Programmatic,
}

/// Pure inline edit session. UI owns rendering; app decides how to persist commits.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InlineEditSession {
    pub target: InlineEditTarget,
    pub draft: FormFieldValue,
    pub start: InlineEditStart,
}

impl InlineEditSession {
    pub fn new(target: InlineEditTarget, draft: FormFieldValue, start: InlineEditStart) -> Self {
        Self {
            target,
            draft,
            start,
        }
    }

    pub fn commit(self) -> InlineEditCommit {
        InlineEditCommit {
            target: self.target,
            value: self.draft,
        }
    }

    pub fn cancel(self) -> InlineEditCancel {
        InlineEditCancel {
            target: self.target,
        }
    }
}

/// App-level commit descriptor emitted by an inline editor.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InlineEditCommit {
    pub target: InlineEditTarget,
    pub value: FormFieldValue,
}

/// App-level cancellation descriptor emitted by an inline editor.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InlineEditCancel {
    pub target: InlineEditTarget,
}

/// Small controller that keeps at most one active inline edit session.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InlineEditController {
    active: Option<InlineEditSession>,
}

impl InlineEditController {
    pub fn begin(&mut self, session: InlineEditSession) {
        self.active = Some(session);
    }

    pub fn active(&self) -> Option<&InlineEditSession> {
        self.active.as_ref()
    }

    pub fn active_mut(&mut self) -> Option<&mut InlineEditSession> {
        self.active.as_mut()
    }

    pub fn commit(&mut self) -> Option<InlineEditCommit> {
        self.active.take().map(InlineEditSession::commit)
    }

    pub fn cancel(&mut self) -> Option<InlineEditCancel> {
        self.active.take().map(InlineEditSession::cancel)
    }

    pub fn is_editing(&self, target: &InlineEditTarget) -> bool {
        self.active
            .as_ref()
            .map(|session| &session.target == target)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controller_commits_active_session_once() {
        let mut controller = InlineEditController::default();
        let target = InlineEditTarget::data_cell("row-1", "gain");
        controller.begin(InlineEditSession::new(
            target.clone(),
            FormFieldValue::Text("-6 dB".to_owned()),
            InlineEditStart::Keyboard,
        ));

        let commit = controller.commit().unwrap();

        assert_eq!(commit.target, target);
        assert!(controller.active().is_none());
        assert!(controller.commit().is_none());
    }

    #[test]
    fn controller_reports_matching_target() {
        let mut controller = InlineEditController::default();
        let target = InlineEditTarget::property("Layout", "Width");
        controller.begin(InlineEditSession::new(
            target.clone(),
            FormFieldValue::Number(128.0),
            InlineEditStart::Pointer,
        ));

        assert!(controller.is_editing(&target));
    }
}
