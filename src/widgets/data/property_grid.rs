use serde::{Deserialize, Serialize};

/// A single read-only property entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyGridEntry {
    pub name: String,
    pub value: String,
    pub category: String,
    pub group: Option<String>,
    pub description: Option<String>,
}

impl PropertyGridEntry {
    /// Creates a property entry with a category and no group/description.
    pub fn new(
        name: impl Into<String>,
        value: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            category: category.into(),
            group: None,
            description: None,
        }
    }

    /// Adds the entry to a named group.
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
    /// Adds a small helper description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// A property-group bucket inside a category.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyGridGroup {
    pub name: String,
    pub entries: Vec<PropertyGridEntry>,
}

/// A property-grid category containing groups of entries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyGridCategory {
    pub name: String,
    pub groups: Vec<PropertyGridGroup>,
}

/// In-memory model for `PropertyGrid`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyGridModel {
    categories: Vec<PropertyGridCategory>,
}

impl PropertyGridModel {
    /// Groups entries into categories and groups for read-only display.
    pub fn new(entries: impl Into<Vec<PropertyGridEntry>>) -> Self {
        let entries = entries.into();
        Self {
            categories: group_entries(entries),
        }
    }

    /// Returns the grouped categories.
    pub fn categories(&self) -> &[PropertyGridCategory] {
        &self.categories
    }
}

fn group_entries(entries: Vec<PropertyGridEntry>) -> Vec<PropertyGridCategory> {
    let mut categories: Vec<PropertyGridCategory> = Vec::new();
    for entry in entries {
        let category = categories
            .iter_mut()
            .find(|category| category.name == entry.category);
        let category = match category {
            Some(category) => category,
            None => {
                categories.push(PropertyGridCategory {
                    name: entry.category.clone(),
                    groups: Vec::new(),
                });
                categories.last_mut().expect("category just pushed")
            }
        };
        let group_name = entry.group.clone().unwrap_or_else(|| "General".to_owned());
        let group = category
            .groups
            .iter_mut()
            .find(|group| group.name == group_name);
        let group = match group {
            Some(group) => group,
            None => {
                category.groups.push(PropertyGridGroup {
                    name: group_name.clone(),
                    entries: Vec::new(),
                });
                category.groups.last_mut().expect("group just pushed")
            }
        };
        group.entries.push(entry);
    }
    categories
}

/// Read-only, grouped property inspector grid.
pub struct PropertyGrid<'a> {
    model: &'a PropertyGridModel,
}

impl<'a> PropertyGrid<'a> {
    /// Creates a property grid widget for a model.
    pub fn new(model: &'a PropertyGridModel) -> Self {
        Self { model }
    }
}

impl<'a> egui::Widget for PropertyGrid<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let output = egui::ScrollArea::vertical().show(ui, |ui| {
            for category in self.model.categories() {
                egui::CollapsingHeader::new(&category.name)
                    .default_open(true)
                    .show(ui, |ui| {
                        for group in &category.groups {
                            ui.label(egui::RichText::new(&group.name).strong());
                            for entry in &group.entries {
                                ui.horizontal(|ui| {
                                    ui.label(&entry.name);
                                    ui.label(&entry.value);
                                });
                                if let Some(description) = &entry.description {
                                    ui.small(description);
                                }
                            }
                        }
                    });
            }
        });
        ui.interact(
            output.inner_rect,
            ui.id().with("property_grid"),
            egui::Sense::hover(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_grid_groups_by_category_and_group() {
        let model = PropertyGridModel::new(vec![
            PropertyGridEntry::new("Width", "128", "Layout").group("Geometry"),
            PropertyGridEntry::new("Height", "64", "Layout").group("Geometry"),
            PropertyGridEntry::new("Title", "Dashboard", "Content"),
        ]);

        assert_eq!(model.categories().len(), 2);
        assert_eq!(model.categories()[0].groups[0].entries.len(), 2);
        assert_eq!(model.categories()[1].name, "Content");
    }
}
