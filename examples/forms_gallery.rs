use std::collections::BTreeMap;

use eframe::egui;
use egui_expressive::widgets::{
    ColorSwatch, ColorWheel, ColorWheelState, DataCellEditSpec, PropertyEditSpec,
};
use egui_expressive::{
    AutocompleteState, CheckboxField, ChoiceOption, DateParts, DependencyEffect, FieldDependency,
    FilePickerRequest, FormFieldDef, FormFieldKind, FormFieldValue, FormSchema,
    InlineEditController, InlineEditSession, InlineEditStart, MultiSelectState, NumericConstraint,
    RgbaColorValue, SelectField, SelectOption, SwitchField, TextAreaField, TextField, TextMask,
    TimeParts, ValidationMessage, ValidationRule, ValidationRuleKind, ValidationSummary,
};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Forms Gallery",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(FormsGallery::default()))),
    )
}

struct FormsGallery {
    name: String,
    notes: String,
    serial: String,
    gain: String,
    mode: FormMode,
    enabled: bool,
    sync: bool,
    autocomplete: AutocompleteState,
    tags: MultiSelectState,
    accent: egui::Color32,
    color_wheel: ColorWheelState,
    edit_controller: InlineEditController,
    property_edit: PropertyEditSpec,
    cell_edit: DataCellEditSpec,
    last_edit_feedback: String,
}

impl Default for FormsGallery {
    fn default() -> Self {
        Self {
            name: String::new(),
            notes: String::new(),
            serial: String::new(),
            gain: "0".to_owned(),
            mode: FormMode::Audio,
            enabled: false,
            sync: false,
            autocomplete: AutocompleteState::default(),
            tags: MultiSelectState::default(),
            accent: egui::Color32::from_rgb(120, 180, 255),
            color_wheel: ColorWheelState {
                hue: 0.58,
                saturation: 0.8,
                value: 0.95,
            },
            edit_controller: InlineEditController::default(),
            property_edit: PropertyEditSpec::new(
                "Layout",
                "Width",
                FormFieldKind::Numeric,
                FormFieldValue::Number(128.0),
            ),
            cell_edit: DataCellEditSpec::new(
                "row-1",
                "gain",
                FormFieldKind::Text,
                FormFieldValue::Text("0 dB".to_owned()),
            ),
            last_edit_feedback: "No edit committed yet".to_owned(),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum FormMode {
    #[default]
    Audio,
    Midi,
    Control,
}

impl eframe::App for FormsGallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.render(ui);
    }
}

impl FormsGallery {
    fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("egui_expressive forms v2");
        ui.label("Schema metadata, validation summaries, rich input descriptors, masks, and inline edit commits layered over native egui controls.");
        ui.add_space(12.0);

        let schema = self.schema();
        ui.collapsing("Schema focus order", |ui| {
            ui.monospace(format!("{:?}", schema.focus_order()));
        });

        let summary = self.validation_summary();
        self.show_summary(ui, &summary);

        TextField::new("Track name", &mut self.name)
            .hint("Lead synth")
            .message(message_or_help(
                &summary,
                "track_name",
                "Used in browser and mixer labels.",
            ))
            .show(ui);

        ui.add_space(8.0);
        SelectField::new("Lane type", &mut self.mode)
            .options([
                SelectOption::new(FormMode::Audio, "Audio"),
                SelectOption::new(FormMode::Midi, "MIDI"),
                SelectOption::new(FormMode::Control, "Control"),
            ])
            .show(ui);

        ui.add_space(8.0);
        CheckboxField::new("Enabled", &mut self.enabled).show(ui);
        SwitchField::new("Sync to project tempo", &mut self.sync).show(ui);

        let states = schema.evaluate_dependencies(&self.form_values());
        ui.collapsing("Derived dependency state", |ui| {
            ui.monospace(format!("gain: {:?}", states.get("gain")));
        });

        ui.separator();
        ui.heading("Input correctness");
        let mask = TextMask::new("AAA-###");
        if TextField::new("Masked serial", &mut self.serial)
            .hint("OSC-001")
            .message(ValidationMessage::help(
                "Paste is sanitized before validation.",
            ))
            .show(ui)
            .changed()
        {
            self.serial = mask.format(&self.serial);
        }

        let gain_constraint = NumericConstraint::new(-60.0, 12.0).step(0.5);
        let gain_preview = gain_constraint.parse_clamped(&self.gain);
        let gain_state = states.get("gain").copied().unwrap_or_default();
        TextField::new("Gain", &mut self.gain)
            .hint("-6.0")
            .message(ValidationMessage::help(format!(
                "Parsed/clamped preview: {:?}; required by sync: {}",
                gain_preview, gain_state.required
            )))
            .enabled(gain_state.enabled)
            .show(ui);

        ui.separator();
        ui.heading("Rich inputs");
        TextField::new("Autocomplete", &mut self.autocomplete.query)
            .hint("filter destinations")
            .show(ui);
        for option in self.autocomplete.filtered_options(&routing_options()) {
            ui.label(format!("{} ({})", option.label, option.id));
        }

        ui.horizontal_wrapped(|ui| {
            for option in tag_options() {
                let selected = self.tags.is_selected(&option.id);
                if ui.selectable_label(selected, &option.label).clicked() {
                    self.tags.toggle(option.id);
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Accent");
            ui.add(ColorSwatch::new(&mut self.accent).label("Color field adapter"));
            let rgba = RgbaColorValue::from(self.accent);
            ui.monospace(format!("{:?}", rgba.to_field_value()));
        });
        ui.horizontal(|ui| {
            ui.label("HSV wheel");
            ui.add(ColorWheel::new(&mut self.color_wheel));
            ui.monospace(format!(
                "{:?}",
                ColorWheel::new(&mut self.color_wheel).color()
            ));
        });

        ui.label(format!(
            "Date/time descriptors: {:?} / {:?}",
            DateParts::new(2026, 5, 7).to_field_value(),
            TimeParts::new(14, 30).to_field_value()
        ));
        ui.label(format!(
            "File picker request descriptor: {:?}",
            FilePickerRequest::new("preset", "Choose preset").allow_extension("json")
        ));

        ui.separator();
        ui.heading("Inline editing descriptors");
        if ui.button("Begin property edit").clicked() {
            self.edit_controller.begin(InlineEditSession::new(
                self.property_edit.target(),
                FormFieldValue::Number(192.0),
                InlineEditStart::Pointer,
            ));
        }
        if ui.button("Begin data-cell edit").clicked() {
            self.edit_controller.begin(InlineEditSession::new(
                self.cell_edit.target(),
                FormFieldValue::Text("-6 dB".to_owned()),
                InlineEditStart::Keyboard,
            ));
        }
        ui.horizontal(|ui| {
            if ui.button("Commit edit").clicked() {
                self.commit_active_edit();
            }
            if ui.button("Cancel edit").clicked() {
                self.edit_controller.cancel();
                self.last_edit_feedback = "Edit cancelled".to_owned();
            }
        });
        ui.monospace(format!("Active edit: {:?}", self.edit_controller.active()));
        ui.monospace(format!(
            "Property edit value: {:?}",
            self.property_edit.value
        ));
        ui.monospace(format!("Cell edit value: {:?}", self.cell_edit.value));
        ui.label(&self.last_edit_feedback);

        ui.add_space(8.0);
        TextAreaField::new("Notes", &mut self.notes)
            .rows(3)
            .message(ValidationMessage::warning(
                "Keep plugin notes short for compact modals.",
            ))
            .show(ui);
    }

    fn schema(&self) -> FormSchema {
        FormSchema::new(vec![
            FormFieldDef::new("track_name", "Track name", FormFieldKind::Text)
                .focus_id("forms.track_name")
                .rule(ValidationRule::new(
                    "track_name",
                    ValidationRuleKind::Required,
                    ValidationMessage::error("Track name is required"),
                )),
            FormFieldDef::new("serial", "Masked serial", FormFieldKind::Text)
                .focus_id("forms.serial"),
            FormFieldDef::new("sync", "Sync to project tempo", FormFieldKind::Switch)
                .focus_id("forms.sync"),
            FormFieldDef::new("gain", "Gain", FormFieldKind::Numeric)
                .focus_id("forms.gain")
                .dependency(FieldDependency::new(
                    "sync",
                    FormFieldValue::Bool(true),
                    DependencyEffect::Require,
                ))
                .dependency(FieldDependency::new(
                    "enabled",
                    FormFieldValue::Bool(false),
                    DependencyEffect::Disable,
                )),
        ])
    }

    fn form_values(&self) -> BTreeMap<String, FormFieldValue> {
        BTreeMap::from([
            (
                "track_name".to_owned(),
                FormFieldValue::Text(self.name.clone()),
            ),
            ("sync".to_owned(), FormFieldValue::Bool(self.sync)),
            ("enabled".to_owned(), FormFieldValue::Bool(self.enabled)),
        ])
    }

    fn validation_summary(&self) -> ValidationSummary {
        let values = self.form_values();
        let rules = self
            .schema()
            .fields
            .iter()
            .flat_map(|field| field.validation_rules.clone())
            .collect::<Vec<_>>();
        ValidationSummary::from_rules(&values, &rules)
    }

    fn commit_active_edit(&mut self) {
        let Some(commit) = self.edit_controller.commit() else {
            self.last_edit_feedback = "No active edit to commit".to_owned();
            return;
        };
        let property_applied = self.property_edit.apply_commit(&commit);
        let cell_applied = self.cell_edit.apply_commit(&commit);
        self.last_edit_feedback =
            format!("Commit applied — property: {property_applied}, data cell: {cell_applied}");
    }

    fn show_summary(&self, ui: &mut egui::Ui, summary: &ValidationSummary) {
        if summary.is_empty() {
            return;
        }
        ui.group(|ui| {
            ui.label(egui::RichText::new("Validation summary").strong());
            for message in summary.messages_for("track_name") {
                ui.colored_label(message.color(), &message.text);
            }
        });
        ui.add_space(8.0);
    }
}

fn message_or_help(summary: &ValidationSummary, field_id: &str, help: &str) -> ValidationMessage {
    summary
        .messages_for(field_id)
        .first()
        .cloned()
        .unwrap_or_else(|| ValidationMessage::help(help))
}

fn routing_options() -> Vec<ChoiceOption> {
    vec![
        ChoiceOption::new("bus-a", "Bus A").keyword("send"),
        ChoiceOption::new("bus-b", "Bus B").keyword("sidechain"),
        ChoiceOption::new("master", "Master").keyword("output"),
    ]
}

fn tag_options() -> Vec<ChoiceOption> {
    vec![
        ChoiceOption::new("favorite", "Favorite"),
        ChoiceOption::new("needs-mix", "Needs mix"),
        ChoiceOption::new("printed", "Printed"),
    ]
}
