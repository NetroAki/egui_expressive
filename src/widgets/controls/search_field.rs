/// Dense search/filter field primitive.
pub struct SearchField<'a> {
    query: &'a mut String,
    hint: String,
    width: f32,
}

impl<'a> SearchField<'a> {
    pub fn new(query: &'a mut String) -> Self {
        Self {
            query,
            hint: "Search…".to_owned(),
            width: 180.0,
        }
    }
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn matches(query: &str, candidate: &str) -> bool {
        let q = query.trim().to_lowercase();
        q.is_empty() || candidate.to_lowercase().contains(&q)
    }
}

impl<'a> egui::Widget for SearchField<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(
            egui::TextEdit::singleline(self.query)
                .hint_text(self.hint)
                .desired_width(self.width),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_case_insensitive_substrings() {
        assert!(SearchField::matches("foo", "The Foo Bar"));
    }
}
