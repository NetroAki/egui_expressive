use std::collections::VecDeque;

use egui::Id;

use crate::accessibility::{
    AccessibilityMeta, AccessibilityRole, LiveRegion, LiveRegionPoliteness,
};

/// Severity hint shared by feedback adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackSeverity {
    Info,
    Success,
    Warning,
    Error,
}

/// User-facing feedback payload with optional focus restoration target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackMessage {
    pub id: String,
    pub message: String,
    pub severity: FeedbackSeverity,
    pub focus_return: Option<Id>,
}

impl FeedbackMessage {
    pub fn new(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
            severity: FeedbackSeverity::Info,
            focus_return: None,
        }
    }

    pub fn severity(mut self, severity: FeedbackSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn focus_return(mut self, id: Id) -> Self {
        self.focus_return = Some(id);
        self
    }

    pub fn live_region(&self) -> LiveRegion {
        let politeness = match self.severity {
            FeedbackSeverity::Info | FeedbackSeverity::Success => LiveRegionPoliteness::Polite,
            FeedbackSeverity::Warning | FeedbackSeverity::Error => LiveRegionPoliteness::Assertive,
        };
        LiveRegion::new(politeness, self.message.clone())
    }

    pub fn accessibility_meta(&self, role: AccessibilityRole) -> AccessibilityMeta {
        AccessibilityMeta::new(role, self.message.clone()).live_region(self.live_region())
    }
}

/// Expiring toast state stored by the dispatcher.
#[derive(Clone, Debug, PartialEq)]
pub struct FeedbackToast {
    pub message: FeedbackMessage,
    pub seconds_left: f32,
}

impl FeedbackToast {
    pub fn new(id: impl Into<String>, message: impl Into<String>, seconds_left: f32) -> Self {
        Self {
            message: FeedbackMessage::new(id, message),
            seconds_left,
        }
    }

    pub fn severity(mut self, severity: FeedbackSeverity) -> Self {
        self.message.severity = severity;
        self
    }
}

/// Long-running operation feedback; `None` means indeterminate progress.
#[derive(Clone, Debug, PartialEq)]
pub struct FeedbackProgress {
    pub id: String,
    pub label: String,
    pub fraction: Option<f32>,
}

impl FeedbackProgress {
    pub fn new(id: impl Into<String>, label: impl Into<String>, fraction: Option<f32>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            fraction,
        }
    }

    pub fn live_region(&self) -> LiveRegion {
        LiveRegion::polite(self.label.clone())
    }

    pub fn accessibility_meta(&self) -> AccessibilityMeta {
        let value = self
            .fraction
            .map(|fraction| format!("{:.0}%", fraction.clamp(0.0, 1.0) * 100.0));
        let mut meta = AccessibilityMeta::new(AccessibilityRole::ProgressBar, self.label.clone())
            .live_region(self.live_region());
        if let Some(value) = value {
            meta = meta.value(value);
        }
        meta
    }
}

/// Pure runtime feedback policy for modals, snackbars, toasts, progress, and notifications.
#[derive(Clone, Debug, PartialEq)]
pub struct FeedbackQueue {
    modal: Option<FeedbackMessage>,
    visible_snackbar: Option<FeedbackMessage>,
    snackbar_queue: VecDeque<FeedbackMessage>,
    toasts: Vec<FeedbackToast>,
    progress: Vec<FeedbackProgress>,
    notifications: VecDeque<FeedbackMessage>,
    max_toasts: usize,
}

impl Default for FeedbackQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedbackQueue {
    pub fn new() -> Self {
        Self::with_max_toasts(3)
    }

    pub fn with_max_toasts(max_toasts: usize) -> Self {
        Self {
            modal: None,
            visible_snackbar: None,
            snackbar_queue: VecDeque::new(),
            toasts: Vec::new(),
            progress: Vec::new(),
            notifications: VecDeque::new(),
            max_toasts,
        }
    }

    pub fn push_modal(&mut self, message: FeedbackMessage) -> Option<FeedbackMessage> {
        self.record(message.clone());
        self.modal.replace(message)
    }

    pub fn push_snackbar(&mut self, message: FeedbackMessage) {
        self.record(message.clone());
        if self.visible_snackbar.is_none() {
            self.visible_snackbar = Some(message);
        } else {
            self.snackbar_queue.push_back(message);
        }
    }

    pub fn push_toast(&mut self, toast: FeedbackToast) {
        self.record(toast.message.clone());
        if self.max_toasts == 0 {
            return;
        }
        if self.toasts.len() == self.max_toasts {
            self.toasts.remove(0);
        }
        self.toasts.push(toast);
    }

    pub fn push_progress(&mut self, progress: FeedbackProgress) {
        self.record(FeedbackMessage::new(
            progress.id.clone(),
            progress.label.clone(),
        ));
        if let Some(existing) = self.progress.iter_mut().find(|p| p.id == progress.id) {
            *existing = progress;
        } else {
            self.progress.push(progress);
        }
    }

    pub fn active_modal(&self) -> Option<&FeedbackMessage> {
        self.modal.as_ref()
    }

    pub fn visible_snackbar(&self) -> Option<&FeedbackMessage> {
        self.visible_snackbar.as_ref()
    }

    pub fn queued_snackbars(&self) -> usize {
        self.snackbar_queue.len()
    }

    pub fn toasts(&self) -> &[FeedbackToast] {
        &self.toasts
    }

    pub fn notifications(&self) -> &VecDeque<FeedbackMessage> {
        &self.notifications
    }

    pub fn progress_suppressed_by_modal(&self) -> bool {
        self.modal.is_some() && !self.progress.is_empty()
    }

    pub fn visible_progress(&self) -> Vec<&FeedbackProgress> {
        if self.modal.is_some() {
            Vec::new()
        } else {
            self.progress.iter().collect()
        }
    }

    pub fn dismiss_modal(&mut self) -> Option<Id> {
        self.modal.take().and_then(|message| message.focus_return)
    }

    pub fn dismiss_snackbar(&mut self) -> Option<Id> {
        let focus_return = self
            .visible_snackbar
            .take()
            .and_then(|message| message.focus_return);
        self.visible_snackbar = self.snackbar_queue.pop_front();
        focus_return
    }

    pub fn dismiss_toast(&mut self, id: &str) -> Option<FeedbackToast> {
        let index = self
            .toasts
            .iter()
            .position(|toast| toast.message.id == id)?;
        Some(self.toasts.remove(index))
    }

    pub fn finish_progress(&mut self, id: &str) -> Option<FeedbackProgress> {
        let index = self
            .progress
            .iter()
            .position(|progress| progress.id == id)?;
        Some(self.progress.remove(index))
    }

    pub fn tick(&mut self, seconds: f32) {
        for toast in &mut self.toasts {
            toast.seconds_left -= seconds;
        }
        self.toasts.retain(|toast| toast.seconds_left > 0.0);
    }

    fn record(&mut self, message: FeedbackMessage) {
        self.notifications.push_back(message);
    }
}

#[cfg(test)]
#[path = "feedback_tests.rs"]
mod feedback_tests;
