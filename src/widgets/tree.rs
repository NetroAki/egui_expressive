use egui::{Key, Response, Sense, TextEdit, Ui};

pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub icon: Option<char>,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub selected: bool,
    pub renaming: bool,
    pub draggable: bool,
}

pub struct TreeView<'a> {
    nodes: &'a mut [TreeNode],
    pub selected_id: Option<&'a mut Option<String>>,
    pub dragged_id: Option<&'a mut Option<String>>,
}

impl TreeNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            children: Vec::new(),
            expanded: false,
            selected: false,
            renaming: false,
            draggable: true,
        }
    }
    pub fn icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn children(mut self, children: Vec<TreeNode>) -> Self {
        self.children = children;
        self
    }
}

impl<'a> TreeView<'a> {
    pub fn new(nodes: &'a mut [TreeNode]) -> Self {
        Self {
            nodes,
            selected_id: None,
            dragged_id: None,
        }
    }
    pub fn selected_id(mut self, selected_id: &'a mut Option<String>) -> Self {
        self.selected_id = Some(selected_id);
        self
    }
    pub fn dragged_id(mut self, dragged_id: &'a mut Option<String>) -> Self {
        self.dragged_id = Some(dragged_id);
        self
    }
}

impl<'a> egui::Widget for TreeView<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let mut response = ui.allocate_response(egui::Vec2::ZERO, Sense::hover());
        for node in self.nodes.iter_mut() {
            show_node(
                ui,
                node,
                &mut self.selected_id,
                &mut self.dragged_id,
                &mut response,
            );
        }
        response
    }
}

fn show_node(
    ui: &mut Ui,
    node: &mut TreeNode,
    selected_id: &mut Option<&mut Option<String>>,
    dragged_id: &mut Option<&mut Option<String>>,
    response: &mut Response,
) {
    let title = format!(
        "{}{}",
        node.icon.map(|i| format!("{} ", i)).unwrap_or_default(),
        node.label
    );
    egui::CollapsingHeader::new(title)
        .id_salt(&node.id)
        .default_open(node.expanded)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if node.renaming {
                    let resp = ui.add(TextEdit::singleline(&mut node.label));
                    if resp.lost_focus() || ui.input(|i| i.key_pressed(Key::Enter)) {
                        node.renaming = false;
                    }
                } else if ui.selectable_label(node.selected, &node.label).clicked() {
                    node.selected = true;
                    if let Some(selected) = selected_id.as_deref_mut() {
                        *selected = Some(node.id.clone());
                    }
                    response.mark_changed();
                }
                if node.draggable {
                    let drag = ui.allocate_response(egui::Vec2::splat(10.0), Sense::drag());
                    if drag.drag_started() {
                        if let Some(dragged) = dragged_id.as_deref_mut() {
                            *dragged = Some(node.id.clone());
                        }
                    }
                }
            });
            for child in node.children.iter_mut() {
                show_node(ui, child, selected_id, dragged_id, response);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_node_tracks_icons_selection_and_rename() {
        let node = TreeNode::new("samples", "Samples").icon('F');
        assert_eq!(node.icon, Some('F'));
        assert!(node.draggable);
    }
}
