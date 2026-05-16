//! View-only piano-roll note renderer.
//!
//! Stage 6 keeps this primitive as a compatibility view. Interactive create, move,
//! resize, and marquee behavior lives in the generic `editor` canvas primitives so
//! apps can build piano-roll, timeline, and designer tools without DAW coupling.

use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

#[derive(Clone, Debug, PartialEq)]
pub struct PianoRollNote {
    pub pitch: u8,
    pub beat: f32,
    pub length: f32,
    pub velocity: f32,
    pub selected: bool,
}

pub struct PianoRollView<'a> {
    pub notes: &'a [PianoRollNote],
    pub beats: f32,
    pub key_count: usize,
    pub beat_width: f32,
    pub key_height: f32,
}

/// Backwards-compatible alias for the Stage 6 view-only primitive.
pub type PianoRoll<'a> = PianoRollView<'a>;

impl<'a> PianoRollView<'a> {
    pub fn new(notes: &'a mut [PianoRollNote]) -> Self {
        Self::new_view(notes)
    }

    pub fn new_view(notes: &'a [PianoRollNote]) -> Self {
        Self {
            notes,
            beats: 16.0,
            key_count: 48,
            beat_width: 32.0,
            key_height: 12.0,
        }
    }
    pub fn note_rect(&self, rect: Rect, note: &PianoRollNote) -> Rect {
        let x = rect.left() + note.beat * self.beat_width;
        let y = rect.bottom() - (note.pitch as f32 % self.key_count as f32 + 1.0) * self.key_height;
        Rect::from_min_size(
            Pos2::new(x, y),
            Vec2::new(note.length * self.beat_width, self.key_height),
        )
    }

    pub fn note_at_pos(&self, rect: Rect, pos: Pos2) -> Option<usize> {
        self.notes
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, note)| self.note_rect(rect, note).contains(pos).then_some(index))
    }
}

impl<'a> egui::Widget for PianoRollView<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = Vec2::new(
            self.beats * self.beat_width,
            self.key_count as f32 * self.key_height,
        );
        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        let painter = ui.painter();
        for beat in 0..=self.beats as usize {
            let x = rect.left() + beat as f32 * self.beat_width;
            painter.vline(x, rect.y_range(), Stroke::new(1.0, Color32::from_gray(45)));
        }
        for key in 0..=self.key_count {
            let y = rect.top() + key as f32 * self.key_height;
            painter.hline(rect.x_range(), y, Stroke::new(1.0, Color32::from_gray(36)));
        }
        for note in self.notes.iter() {
            let note_rect = self.note_rect(rect, note).shrink(1.0);
            let color = if note.selected {
                Color32::YELLOW
            } else {
                Color32::from_rgb(80, 170, 240)
            };
            painter.rect_filled(note_rect, 2.0, color.gamma_multiply(note.velocity.max(0.2)));
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn piano_roll_maps_notes_to_rects() {
        let mut notes = [PianoRollNote {
            pitch: 60,
            beat: 1.0,
            length: 2.0,
            velocity: 1.0,
            selected: false,
        }];
        let roll = PianoRollView::new(&mut notes);
        let rect = roll.note_rect(
            Rect::from_min_size(Pos2::ZERO, Vec2::new(512.0, 256.0)),
            &roll.notes[0],
        );
        assert_eq!(rect.width(), 64.0);
    }

    #[test]
    fn piano_roll_view_reports_note_hit_without_mutating_notes() {
        let mut notes = [PianoRollNote {
            pitch: 60,
            beat: 1.0,
            length: 2.0,
            velocity: 1.0,
            selected: false,
        }];
        let roll = PianoRollView::new(&mut notes);
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(512.0, 576.0));
        let note_rect = roll.note_rect(rect, &roll.notes[0]);

        assert_eq!(roll.note_at_pos(rect, note_rect.center()), Some(0));
        assert!(!roll.notes[0].selected);
    }

    #[test]
    fn piano_roll_view_accepts_immutable_notes() {
        let notes = [PianoRollNote {
            pitch: 64,
            beat: 2.0,
            length: 1.0,
            velocity: 0.8,
            selected: false,
        }];
        let roll = PianoRollView::new_view(&notes);
        assert_eq!(roll.notes[0].pitch, 64);
    }
}
