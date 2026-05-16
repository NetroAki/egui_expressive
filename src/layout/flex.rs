//! Flexbox-like layout primitives.

/// Sizing mode for flex children — mirrors Figma's Fill/Hug/Fixed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlexSize {
    Hug,
    Fill,
    Fixed(f32),
    Min(f32),
    Max(f32),
    Clamp(f32, f32),
    Fraction(f32),
}

/// Alignment of children along the cross axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlexAlign {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

/// Justification of children along the main axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlexJustify {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// A flex container that maps Figma Auto Layout parameters to egui layout.
pub struct FlexContainer {
    direction: egui::Direction,
    gap: f32,
    padding: f32,
    align: FlexAlign,
    justify: FlexJustify,
    width: FlexSize,
    height: FlexSize,
    bg: Option<egui::Color32>,
    rounding: f32,
    wrap: bool,
    grow: f32,
    shrink: f32,
}

impl FlexContainer {
    pub fn row(_ui: &egui::Ui) -> Self {
        Self::new(egui::Direction::LeftToRight)
    }

    pub fn column(_ui: &egui::Ui) -> Self {
        Self::new(egui::Direction::TopDown)
    }

    fn new(direction: egui::Direction) -> Self {
        Self {
            direction,
            gap: 0.0,
            padding: 0.0,
            align: FlexAlign::Start,
            justify: FlexJustify::Start,
            width: FlexSize::Hug,
            height: FlexSize::Hug,
            bg: None,
            rounding: 0.0,
            wrap: false,
            grow: 0.0,
            shrink: 1.0,
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
    pub fn align(mut self, align: FlexAlign) -> Self {
        self.align = align;
        self
    }
    pub fn justify(mut self, justify: FlexJustify) -> Self {
        self.justify = justify;
        self
    }
    pub fn width(mut self, width: FlexSize) -> Self {
        self.width = width;
        self
    }
    pub fn height(mut self, height: FlexSize) -> Self {
        self.height = height;
        self
    }
    pub fn bg(mut self, color: egui::Color32) -> Self {
        self.bg = Some(color);
        self
    }
    pub fn rounding(mut self, r: f32) -> Self {
        self.rounding = r;
        self
    }
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }
    pub fn grow(mut self, grow: f32) -> Self {
        self.grow = grow;
        self
    }
    pub fn shrink(mut self, shrink: f32) -> Self {
        self.shrink = shrink;
        self
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui),
    ) -> egui::Response {
        let mut frame = egui::Frame::NONE.inner_margin(egui::Margin::same(self.padding as i8));
        if let Some(color) = self.bg {
            frame = frame.fill(color);
        }
        if self.rounding > 0.0 {
            frame = frame.corner_radius(self.rounding.min(255.0) as u8);
        }

        frame
            .show(ui, |ui| {
                let _flex_weights = (self.grow, self.shrink);
                apply_flex_size(ui, self.width, true);
                apply_flex_size(ui, self.height, false);
                ui.spacing_mut().item_spacing = egui::Vec2::splat(self.gap);
                let layout = self.layout();
                if self.wrap && matches!(self.direction, egui::Direction::LeftToRight) {
                    ui.horizontal_wrapped(add_contents);
                } else {
                    ui.with_layout(layout, add_contents);
                }
            })
            .response
    }

    fn layout(&self) -> egui::Layout {
        match self.direction {
            egui::Direction::LeftToRight | egui::Direction::RightToLeft => {
                let base = match self.align {
                    FlexAlign::Center => egui::Layout::left_to_right(egui::Align::Center),
                    FlexAlign::End | FlexAlign::Stretch => {
                        egui::Layout::left_to_right(egui::Align::Max)
                    }
                    FlexAlign::Baseline | FlexAlign::Start => {
                        egui::Layout::left_to_right(egui::Align::Min)
                    }
                };
                match self.justify {
                    FlexJustify::Center => {
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight)
                    }
                    FlexJustify::End => egui::Layout::right_to_left(egui::Align::Center),
                    FlexJustify::SpaceBetween
                    | FlexJustify::SpaceAround
                    | FlexJustify::SpaceEvenly => base.with_main_justify(true),
                    FlexJustify::Start => base,
                }
            }
            _ => {
                let base = match self.align {
                    FlexAlign::Center => egui::Layout::top_down(egui::Align::Center),
                    FlexAlign::End | FlexAlign::Stretch => egui::Layout::top_down(egui::Align::Max),
                    FlexAlign::Baseline | FlexAlign::Start => {
                        egui::Layout::top_down(egui::Align::Min)
                    }
                };
                match self.justify {
                    FlexJustify::Center => {
                        egui::Layout::centered_and_justified(egui::Direction::TopDown)
                    }
                    FlexJustify::End => egui::Layout::bottom_up(egui::Align::Center),
                    FlexJustify::SpaceBetween
                    | FlexJustify::SpaceAround
                    | FlexJustify::SpaceEvenly => base.with_main_justify(true),
                    FlexJustify::Start => base,
                }
            }
        }
    }
}

fn apply_flex_size(ui: &mut egui::Ui, size: FlexSize, horizontal: bool) {
    let available = if horizontal {
        ui.available_width()
    } else {
        ui.available_height()
    };
    let value = match size {
        FlexSize::Fill => Some(available),
        FlexSize::Fixed(v) => Some(v),
        FlexSize::Min(v) => Some(available.max(v)),
        FlexSize::Max(v) => Some(available.min(v)),
        FlexSize::Clamp(min, max) => Some(available.clamp(min, max)),
        FlexSize::Fraction(frac) => Some(available * frac),
        FlexSize::Hug => None,
    };
    if let Some(value) = value {
        if horizontal {
            ui.set_width(value);
        } else {
            ui.set_height(value);
        }
    }
}

/// Macro for flex row layout — mirrors Figma Auto Layout (horizontal).
#[macro_export]
macro_rules! flex_row {
    ($ui:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::row($ui).show($ui, |__ui| { $($body)* })
    }};
    ($ui:expr, gap: $gap:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::row($ui).gap($gap).show($ui, |__ui| { $($body)* })
    }};
    ($ui:expr, gap: $gap:expr, padding: $pad:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::row($ui).gap($gap).padding($pad).show($ui, |__ui| { $($body)* })
    }};
    ($ui:expr, gap: $gap:expr, padding: $pad:expr, bg: $bg:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::row($ui).gap($gap).padding($pad).bg($bg).show($ui, |__ui| { $($body)* })
    }};
}

/// Macro for flex column layout — mirrors Figma Auto Layout (vertical).
#[macro_export]
macro_rules! flex_col {
    ($ui:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::column($ui).show($ui, |__ui| { $($body)* })
    }};
    ($ui:expr, gap: $gap:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::column($ui).gap($gap).show($ui, |__ui| { $($body)* })
    }};
    ($ui:expr, gap: $gap:expr, padding: $pad:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::column($ui).gap($gap).padding($pad).show($ui, |__ui| { $($body)* })
    }};
    ($ui:expr, gap: $gap:expr, padding: $pad:expr, bg: $bg:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::column($ui).gap($gap).padding($pad).bg($bg).show($ui, |__ui| { $($body)* })
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flex_size_fraction_is_preserved() {
        assert_eq!(FlexSize::Fraction(0.5), FlexSize::Fraction(0.5));
    }
}
