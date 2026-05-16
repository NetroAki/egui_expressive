use std::ops::Range;

/// Returns the bounded visible index range for a scroll window.
///
/// `start`/`end` are scroll-space pixel bounds, `item_extent` is the per-row
/// extent, and `overscan` expands the window by whole items on both sides.
/// Invalid or empty inputs return `0..0`.
pub fn bounded_visible_range(
    total: usize,
    start: f32,
    end: f32,
    item_extent: f32,
    overscan: usize,
) -> Range<usize> {
    if total == 0 || item_extent <= 0.0 || end <= start {
        return 0..0;
    }

    let first = (start / item_extent).floor().max(0.0) as usize;
    let last = (end / item_extent).ceil().max(0.0) as usize;
    let begin = first.saturating_sub(overscan).min(total);
    let finish = last.saturating_add(overscan).min(total);
    begin.min(finish)..finish
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_visible_range_clamps_large_data() {
        let range = bounded_visible_range(10_000, 1_000.0, 1_240.0, 20.0, 2);
        assert_eq!(range, 48..64);
    }

    #[test]
    fn bounded_visible_range_handles_invalid_inputs() {
        assert_eq!(bounded_visible_range(0, 0.0, 1.0, 10.0, 1), 0..0);
        assert_eq!(bounded_visible_range(10, 10.0, 9.0, 10.0, 1), 0..0);
        assert_eq!(bounded_visible_range(10, 0.0, 1.0, 0.0, 1), 0..0);
    }
}
