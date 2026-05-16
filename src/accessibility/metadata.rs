//! Semantic metadata for custom egui widgets.

use crate::accessibility::LiveRegion;

/// Common semantic roles used by custom-painted controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccessibilityRole {
    Alert,
    AlertDialog,
    Button,
    Checkbox,
    Dialog,
    Grid,
    Heading,
    Image,
    Link,
    List,
    ListItem,
    Log,
    Menu,
    MenuItem,
    ProgressBar,
    Radio,
    Slider,
    Status,
    Switch,
    Tab,
    Table,
    TextField,
    Timer,
    Toolbar,
    Tooltip,
    Tree,
    TreeGrid,
    Custom(&'static str),
}

impl AccessibilityRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Alert => "alert",
            Self::AlertDialog => "alertdialog",
            Self::Button => "button",
            Self::Checkbox => "checkbox",
            Self::Dialog => "dialog",
            Self::Grid => "grid",
            Self::Heading => "heading",
            Self::Image => "image",
            Self::Link => "link",
            Self::List => "list",
            Self::ListItem => "listitem",
            Self::Log => "log",
            Self::Menu => "menu",
            Self::MenuItem => "menuitem",
            Self::ProgressBar => "progressbar",
            Self::Radio => "radio",
            Self::Slider => "slider",
            Self::Status => "status",
            Self::Switch => "switch",
            Self::Tab => "tab",
            Self::Table => "table",
            Self::TextField => "textfield",
            Self::Timer => "timer",
            Self::Toolbar => "toolbar",
            Self::Tooltip => "tooltip",
            Self::Tree => "tree",
            Self::TreeGrid => "treegrid",
            Self::Custom(role) => role,
        }
    }
}

/// Accessibility metadata carried by custom-painted primitives and form wrappers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibilityMeta {
    pub role: AccessibilityRole,
    pub label: String,
    pub description: Option<String>,
    pub value: Option<String>,
    pub disabled: bool,
    pub live_region: Option<LiveRegion>,
}

impl AccessibilityMeta {
    pub fn new(role: AccessibilityRole, label: impl Into<String>) -> Self {
        Self {
            role,
            label: label.into(),
            description: None,
            value: None,
            disabled: false,
            live_region: None,
        }
    }

    pub fn status(label: impl Into<String>) -> Self {
        Self::new(AccessibilityRole::Status, label)
    }

    pub fn alert(label: impl Into<String>) -> Self {
        Self::new(AccessibilityRole::Alert, label)
    }

    pub fn button(label: impl Into<String>) -> Self {
        Self::new(AccessibilityRole::Button, label)
    }

    pub fn slider(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(AccessibilityRole::Slider, label).value(value)
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn live_region(mut self, live_region: LiveRegion) -> Self {
        self.live_region = Some(live_region);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_builder_records_role_label_and_state() {
        let meta = AccessibilityMeta::button("Render")
            .description("Start offline export")
            .disabled(true);
        assert_eq!(meta.role.as_str(), "button");
        assert_eq!(meta.label, "Render");
        assert!(meta.disabled);
        assert_eq!(meta.description.as_deref(), Some("Start offline export"));
    }

    #[test]
    fn metadata_supports_live_region_roles() {
        let meta = AccessibilityMeta::alert("Export failed")
            .live_region(LiveRegion::assertive("Export failed"));

        assert_eq!(meta.role.as_str(), "alert");
        assert_eq!(meta.live_region.unwrap().politeness.as_str(), "assertive");
    }
}
