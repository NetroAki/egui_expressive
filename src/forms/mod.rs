//! Form primitives built as readable wrappers around egui native input widgets.
//!
//! Each file owns one form concern: field shell, text input, selection, boolean
//! input, or validation metadata.

mod check;
mod editing;
mod field;
mod input;
mod rich_inputs;
mod schema;
mod select;
mod text;
mod validation;

pub use check::{CheckboxField, SwitchField};
pub use editing::{
    InlineEditCancel, InlineEditCommit, InlineEditController, InlineEditSession, InlineEditStart,
    InlineEditTarget,
};
pub use field::{FieldShell, FieldState};
pub use input::{
    InputTextContract, NumericConstraint, TextDirection, TextMask, TextSelectionRange,
};
pub use rich_inputs::{
    AutocompleteState, ChoiceOption, DateParts, FilePickerRequest, MultiSelectState,
    RgbaColorValue, TimeParts,
};
pub use schema::{
    DependencyEffect, DerivedFieldState, FieldDependency, FormFieldDef, FormFieldKind,
    FormFieldValue, FormSchema,
};
pub use select::{SelectField, SelectOption};
pub use text::{TextAreaField, TextField};
pub use validation::{
    DeferredValidationRequest, ValidationMessage, ValidationRule, ValidationRuleKind,
    ValidationSeverity, ValidationSummary,
};
