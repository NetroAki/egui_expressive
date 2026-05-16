use crate::interaction::{ActionDef, ActionRegistry};
use crate::widgets::controls::SearchField;
use egui::{Key, Response, ScrollArea, Ui};

#[derive(Clone, Debug)]
pub struct CommandPaletteItem {
    pub id: String,
    pub label: String,
    pub hint: String,
}

impl CommandPaletteItem {
    pub fn from_action(action: &ActionDef) -> Self {
        Self {
            id: action.id.clone(),
            label: action.label.clone(),
            hint: action.description.clone().unwrap_or_default(),
        }
    }

    pub fn from_registry(registry: &ActionRegistry) -> Vec<Self> {
        registry.iter().map(Self::from_action).collect()
    }
}

pub struct CommandPalette<'a> {
    pub query: &'a mut String,
    pub items: &'a [CommandPaletteItem],
    pub selected: Option<&'a mut usize>,
    pub activated: Option<&'a mut Option<String>>,
}

impl<'a> CommandPalette<'a> {
    pub fn new(query: &'a mut String, items: &'a [CommandPaletteItem]) -> Self {
        Self {
            query,
            items,
            selected: None,
            activated: None,
        }
    }
    pub fn selected(mut self, selected: &'a mut usize) -> Self {
        self.selected = Some(selected);
        self
    }
    pub fn activated(mut self, activated: &'a mut Option<String>) -> Self {
        // The palette emits an action id only. Callers should route the id
        // through ActionRegistry::dispatch_status so disabled and unknown
        // actions are consumed consistently with shortcuts and menus.
        self.activated = Some(activated);
        self
    }
    pub fn fuzzy_score(query: &str, candidate: &str) -> Option<usize> {
        let mut score = 0;
        let mut pos = 0;
        let candidate = candidate.to_lowercase();
        for ch in query.trim().to_lowercase().chars() {
            let found = candidate[pos..].find(ch)?;
            score += found;
            pos += found + ch.len_utf8();
        }
        Some(score)
    }
}

impl<'a> egui::Widget for CommandPalette<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.add(SearchField::new(self.query));
        let mut rows: Vec<_> = self
            .items
            .iter()
            .filter_map(|item| {
                Self::fuzzy_score(self.query, &item.label).map(|score| (score, item))
            })
            .collect();
        rows.sort_by_key(|(score, _)| *score);
        let mut fallback_selected = 0usize;
        let selected = self.selected.unwrap_or(&mut fallback_selected);
        if ui.input(|i| i.key_pressed(Key::ArrowDown)) && !rows.is_empty() {
            *selected = (*selected + 1).min(rows.len() - 1);
        }
        if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
            *selected = selected.saturating_sub(1);
        }
        if ui.input(|i| i.key_pressed(Key::Enter)) {
            if let (Some((_, item)), Some(activated)) = (rows.get(*selected), self.activated) {
                *activated = Some(item.id.clone());
            }
        }
        ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
            for (index, (_, item)) in rows.iter().enumerate() {
                let label = format!("{} — {}", item.label, item.hint);
                if ui.selectable_label(index == *selected, label).clicked() {
                    *selected = index;
                }
            }
        });
        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_score_matches_ordered_letters() {
        assert!(CommandPalette::fuzzy_score("plg", "Plugin Manager").is_some());
        assert!(CommandPalette::fuzzy_score("zzz", "Plugin Manager").is_none());
    }

    #[test]
    fn palette_items_can_be_built_from_action_registry() {
        let mut registry = ActionRegistry::new();
        registry.register(ActionDef::new("open", "Open File").description("Open a file"));

        let items = CommandPaletteItem::from_registry(&registry);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "open");
        assert_eq!(items[0].hint, "Open a file");
    }
}
