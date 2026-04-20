use egui::{Color32, FontFamily, FontId, Response, Ui, Vec2, Widget};

pub mod chars {
    // Playback
    pub const PLAY: char = '\u{E037}';
    pub const PAUSE: char = '\u{E034}';
    pub const STOP: char = '\u{E047}';
    pub const SKIP_NEXT: char = '\u{E044}';
    pub const SKIP_PREV: char = '\u{E045}';
    pub const REPLAY: char = '\u{E042}';
    pub const LOOP: char = '\u{E040}';
    pub const SHUFFLE: char = '\u{E043}';
    pub const RECORD: char = '\u{E061}'; // fiber_manual_record
    pub const MIC: char = '\u{E029}';
    pub const MIC_OFF: char = '\u{E02A}';
    pub const VOLUME_UP: char = '\u{E050}';
    pub const VOLUME_DOWN: char = '\u{E04F}';
    pub const VOLUME_MUTE: char = '\u{E04E}';
    pub const VOLUME_OFF: char = '\u{E04D}';
    // Navigation
    pub const ARROW_BACK: char = '\u{E5C4}';
    pub const ARROW_FORWARD: char = '\u{E5C8}';
    pub const ARROW_UP: char = '\u{E5D8}';
    pub const ARROW_DOWN: char = '\u{E5DB}';
    pub const CLOSE: char = '\u{E5CD}';
    pub const MENU: char = '\u{E5D2}';
    pub const MORE_VERT: char = '\u{E5D4}';
    pub const MORE_HORIZ: char = '\u{E5D3}';
    pub const EXPAND_MORE: char = '\u{E5CF}';
    pub const EXPAND_LESS: char = '\u{E5CE}';
    pub const CHEVRON_RIGHT: char = '\u{E5CC}';
    pub const CHEVRON_LEFT: char = '\u{E5CB}';
    // Actions
    pub const ADD: char = '\u{E145}';
    pub const REMOVE: char = '\u{E15B}';
    pub const EDIT: char = '\u{E3C9}';
    pub const DELETE: char = '\u{E872}';
    pub const SAVE: char = '\u{E161}';
    pub const COPY: char = '\u{E14D}';
    pub const PASTE: char = '\u{E14F}';
    pub const CUT: char = '\u{E14E}';
    pub const UNDO: char = '\u{E166}';
    pub const REDO: char = '\u{E15A}';
    pub const SEARCH: char = '\u{E8B6}';
    pub const FILTER: char = '\u{EF4F}';
    pub const SORT: char = '\u{E164}';
    pub const DOWNLOAD: char = '\u{F090}';
    pub const UPLOAD: char = '\u{F09B}';
    pub const SHARE: char = '\u{E80D}';
    pub const LINK: char = '\u{E157}';
    pub const OPEN_IN_NEW: char = '\u{E89E}';
    // Status
    pub const CHECK: char = '\u{E876}';
    pub const CHECK_CIRCLE: char = '\u{E86C}';
    pub const ERROR: char = '\u{E000}';
    pub const WARNING: char = '\u{E002}';
    pub const INFO: char = '\u{E88E}';
    pub const HELP: char = '\u{E887}';
    pub const STAR: char = '\u{E838}';
    pub const STAR_OUTLINE: char = '\u{E83A}';
    pub const FAVORITE: char = '\u{E87D}';
    pub const BOOKMARK: char = '\u{E866}';
    // Files/Media
    pub const FOLDER: char = '\u{E2C7}';
    pub const FILE: char = '\u{E24D}';
    pub const IMAGE: char = '\u{E3F4}';
    pub const AUDIO: char = '\u{E3A1}';
    pub const VIDEO: char = '\u{E04B}';
    pub const SETTINGS: char = '\u{E8B8}';
    pub const TUNE: char = '\u{E429}';
    pub const PALETTE: char = '\u{E40A}';
    pub const BRUSH: char = '\u{E3AE}';
    pub const LAYERS: char = '\u{E53B}';
}

/// Deprecated: use `chars` instead.
#[deprecated(note = "use `egui_expressive::icons::chars` instead")]
pub use chars as icons;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IconSize {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

impl IconSize {
    pub fn to_px(self) -> f32 {
        match self {
            IconSize::Xs => 12.0,
            IconSize::Sm => 16.0,
            IconSize::Md => 20.0,
            IconSize::Lg => 24.0,
            IconSize::Xl => 32.0,
        }
    }
}

pub struct Icon {
    codepoint: char,
    size: f32,
    color: Option<Color32>,
}

impl Icon {
    pub fn new(codepoint: char) -> Self {
        Self {
            codepoint,
            size: 20.0,
            color: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn icon_size(mut self, size: IconSize) -> Self {
        self.size = size.to_px();
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

impl Widget for Icon {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = Vec2::splat(self.size);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());

        let color = self.color.unwrap_or_else(|| ui.visuals().text_color());

        let font_id = FontId::new(self.size, FontFamily::Name("icons".into()));

        let painter = ui.painter();
        let galley = painter.layout(self.codepoint.to_string(), font_id, color, f32::INFINITY);

        let pos = rect.center() - Vec2::new(galley.size().x / 2.0, galley.size().y / 2.0);

        painter.add(egui::epaint::TextShape::new(pos, galley, color));

        response
    }
}

/// A clickable icon with hover highlight.
pub struct IconButton {
    codepoint: char,
    size: f32,
    color: Option<Color32>,
    active: bool,
}

impl IconButton {
    pub fn new(codepoint: char) -> Self {
        Self {
            codepoint,
            size: 24.0,
            color: None,
            active: false,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

impl Widget for IconButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = Vec2::splat(self.size);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        if response.hovered() || self.active {
            let circle_rect = rect.expand(4.0);
            let color = if self.active {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().widgets.hovered.bg_fill
            };
            ui.painter().add(egui::Shape::circle_filled(
                circle_rect.center(),
                self.size / 2.0 + 2.0,
                color,
            ));
        }

        let color = self.color.unwrap_or_else(|| ui.visuals().text_color());

        let font_id = FontId::new(self.size, FontFamily::Name("icons".into()));

        let painter = ui.painter();
        let galley = painter.layout(self.codepoint.to_string(), font_id, color, f32::INFINITY);

        let pos = rect.center() - Vec2::new(galley.size().x / 2.0, galley.size().y / 2.0);

        painter.add(egui::epaint::TextShape::new(pos, galley, color));

        response
    }
}
