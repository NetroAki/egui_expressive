//! Clipboard command descriptors.

/// Pure clipboard-write intent. The host app decides how to fulfill it through egui/native APIs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipboardCommand {
    pub text: String,
    pub contains_sensitive_text: bool,
}

impl ClipboardCommand {
    pub fn copy_text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            contains_sensitive_text: false,
        }
    }

    pub fn sensitive(mut self, contains_sensitive_text: bool) -> Self {
        self.contains_sensitive_text = contains_sensitive_text;
        self
    }

    pub fn should_log_value(&self) -> bool {
        !self.contains_sensitive_text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_clipboard_commands_suppress_value_logging() {
        let command = ClipboardCommand::copy_text("token").sensitive(true);

        assert!(!command.should_log_value());
    }
}
