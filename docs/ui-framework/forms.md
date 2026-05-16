# Forms

`src/forms` provides readable wrappers and pure data models for settings,
preferences, inspectors, and inline-edit flows. It wraps egui native inputs; it
does **not** reimplement low-level text editing, focus, IME, bidi, selection, or
platform dialogs.

## Modules

- `field.rs` — label/message shell and field state.
- `text.rs` — `TextField`, `TextAreaField` wrappers around `egui::TextEdit`.
- `select.rs` — `SelectField` wrapper around `egui::ComboBox`.
- `check.rs` — `CheckboxField`, `SwitchField` wrappers around egui boolean controls.
- `schema.rs` — `FormSchema`, field definitions, values, focus/action IDs, dependencies.
- `validation.rs` — messages, sync validation rules, deferred validation descriptors, summaries.
- `input.rs` — masks, numeric constraints, selection metadata, input correctness contract.
- `rich_inputs.rs` — autocomplete/multi-select/date/time/file/color value descriptors.
- `editing.rs` — inline edit sessions, commits, cancels, and single-session controller.

`mod.rs` remains docs/re-exports only.

## Forms v1 wrappers

Existing wrappers remain compatible:

```rust
TextField::new("Track name", &mut name)
    .hint("Lead synth")
    .message(ValidationMessage::help("Shown in the mixer."))
    .show(ui);

SelectField::new("Lane type", &mut mode)
    .options([SelectOption::new(FormMode::Audio, "Audio")])
    .show(ui);
```

## Schema and dependencies

Forms v2 schemas are plain descriptors, not renderer-specific widget trees.

```rust
let schema = FormSchema::new(vec![
    FormFieldDef::new("advanced", "Advanced", FormFieldKind::Switch),
    FormFieldDef::new("token", "Token", FormFieldKind::Text)
        .focus_id("settings.token")
        .dependency(FieldDependency::new(
            "advanced",
            FormFieldValue::Bool(true),
            DependencyEffect::Require,
        )),
]);
let states = schema.evaluate_dependencies(&values);
```

Fields can carry stable IDs, labels, help text, kind, action ID, focus ID,
validation rules, and dependency rules. App code chooses how to render fields and
how to route action IDs into Stage 4 command/focus systems.

## Validation

Synchronous validation is pure and deterministic:

```rust
let rules = vec![ValidationRule::new(
    "name",
    ValidationRuleKind::Required,
    ValidationMessage::error("Name required"),
)];
let summary = ValidationSummary::from_rules(&values, &rules);
```

Rule kinds are value-kind specific. If a rule is paired with a mismatched
`FormFieldValue` variant, it is treated as not applicable rather than invalid;
schema tests should pair text rules with text values and numeric rules with
numeric values.

Deferred validation is represented by `DeferredValidationRequest`. Apps own any
worker, timer, network, or async runtime they choose to use; this crate only
defines the request/result data boundary.

## Input correctness contract

- `TextMask` sanitizes paste/input text before validation. Mask slots: `#` for
  ASCII digits and `A` for ASCII alphanumeric characters.
- `NumericConstraint` parses and clamps text-backed numeric values.
- `TextSelectionRange` stores deterministic selection metadata and can normalize
  reversed ranges.
- `InputTextContract` documents that egui/platform code owns IME composition and
  that full RTL/bidi behavior is platform-limited here.

Stage 5 intentionally does not claim platform IME/RTL certification. Stage 8 owns
broader accessibility/i18n/platform guidance.

## Rich input descriptors

- `ChoiceOption` + `AutocompleteState` filter autocomplete choices without a
  platform service or virtualized popup requirement.
- `MultiSelectState` tracks ordered selected IDs and converts to `FormFieldValue::List`.
- `DateParts` / `TimeParts` validate dependency-free date/time values.
- `FilePickerRequest` describes a requested native file selection but performs no
  OS dialog, filesystem read/write, or path validation.
- `RgbaColorValue` bridges `egui::Color32` and `FormFieldValue::Color`.

## Inline editing

`InlineEditSession`, `InlineEditTarget`, `InlineEditCommit`, and
`InlineEditController` provide pure edit descriptors for forms, property grids,
and data cells. They do not mutate app models automatically and do not create a
Stage 6 editor/canvas history layer.

Use `src/widgets/data/editing.rs` adapters to apply commits to data/property edit
descriptors while keeping Stage 3 `DataTable` and `PropertyGrid` read-only.

## Example

- `examples/forms_gallery.rs` demonstrates schemas, validation summaries, masks,
  numeric parsing, autocomplete, multi-select, color values, file request
  descriptors, and inline edit commit descriptors.

## Deferrals

- Stage 6: editor/canvas object graph, object/file drag/drop, DAW namespace cleanup.
- Stage 8: platform accessibility, screen-reader/live-region patterns, full IME/RTL guidance.
- Stage 9: release-scale benchmarks, CI/release harness, advanced data-grid hardening.
