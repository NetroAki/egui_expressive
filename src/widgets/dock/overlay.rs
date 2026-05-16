use super::split::DockZone;
use egui::{Color32, Pos2, Rect, Response, Sense, Ui};

#[derive(Clone, Copy, Debug, PartialEq)]
/// Drop target geometry for dock overlays.
///
/// `rect` is stored in local coordinates relative to the parent UI allocation origin.
pub struct DockDropZone {
    /// Target dock zone for this drop rect.
    pub zone: DockZone,
    /// Local rect relative to the overlay allocation origin.
    pub rect: Rect,
}

impl DockDropZone {
    /// Creates a new drop zone with local coordinates.
    pub fn new(zone: DockZone, rect: Rect) -> Self {
        Self { zone, rect }
    }
}

/// Overlay widget that paints dock drop targets in local allocation space.
pub struct DockOverlay<'a> {
    zones: &'a [DockDropZone],
    pointer: Option<Pos2>,
}

impl<'a> DockOverlay<'a> {
    /// Creates an overlay from local-coordinate drop zones.
    pub fn new(zones: &'a [DockDropZone]) -> Self {
        Self {
            zones,
            pointer: None,
        }
    }

    /// Sets the pointer position in the same local coordinates as the zones.
    pub fn pointer(mut self, pointer: Pos2) -> Self {
        self.pointer = Some(pointer);
        self
    }

    /// Returns the dock zone at a local point, if any.
    pub fn zone_at(&self, point: Pos2) -> Option<DockZone> {
        self.zones
            .iter()
            .find(|zone| zone.rect.contains(point))
            .map(|zone| zone.zone)
    }
}

fn painted_zone_rect(zone_rect: Rect, allocation_rect: Rect) -> Option<Rect> {
    let local_rect = zone_rect.intersect(Rect::from_min_size(Pos2::ZERO, allocation_rect.size()));
    if local_rect.is_negative() {
        None
    } else {
        Some(local_rect.translate(allocation_rect.min.to_vec2()))
    }
}

impl<'a> egui::Widget for DockOverlay<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let resp = ui.allocate_response(
            egui::Vec2::new(ui.available_width(), ui.available_height()),
            Sense::hover(),
        );
        for zone in self.zones {
            let Some(paint_rect) = painted_zone_rect(zone.rect, resp.rect) else {
                continue;
            };
            let local_rect = paint_rect.translate(-resp.rect.min.to_vec2());
            let active = self.pointer.is_some_and(|point| local_rect.contains(point));
            let col = if active {
                Color32::from_rgba_unmultiplied(80, 160, 255, 90)
            } else {
                Color32::from_rgba_unmultiplied(60, 120, 220, 36)
            };
            ui.painter().rect_filled(paint_rect, 6.0, col);
        }
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, Rect};

    #[test]
    fn dock_overlay_resolves_zone_from_geometry() {
        let zones = [DockDropZone::new(
            DockZone::Left,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(20.0, 20.0)),
        )];
        let overlay = DockOverlay::new(&zones);
        assert_eq!(overlay.zone_at(pos2(10.0, 10.0)), Some(DockZone::Left));
        assert_eq!(overlay.zone_at(pos2(30.0, 10.0)), None);
    }

    #[test]
    fn dock_overlay_translates_local_rect_to_nonzero_allocation_origin() {
        let zone = DockDropZone::new(
            DockZone::Right,
            Rect::from_min_max(pos2(5.0, 6.0), pos2(15.0, 16.0)),
        );
        let allocation = Rect::from_min_max(pos2(100.0, 200.0), pos2(180.0, 260.0));

        let painted = painted_zone_rect(zone.rect, allocation).expect("paint rect");

        assert_eq!(painted.min, pos2(105.0, 206.0));
        assert_eq!(painted.max, pos2(115.0, 216.0));
    }
}
