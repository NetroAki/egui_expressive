//! Shared geometry/util helpers for draw modules.
use egui::*;

pub(crate) fn cross2(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

pub(crate) fn bounds_from_points(points: &[Pos2]) -> Option<Rect> {
    let first = points.first()?;
    let mut min = *first;
    let mut max = *first;
    for p in &points[1..] {
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
    }
    Some(Rect::from_min_max(min, max))
}

pub(crate) fn valid_bounds(rect: Rect) -> Option<Rect> {
    if rect.is_finite() && rect.is_positive() {
        Some(rect)
    } else {
        None
    }
}

pub(crate) fn point_in_polygon(p: Pos2, polygon: &[Pos2]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let pi = polygon[i];
        let pj = polygon[j];
        if ((pi.y > p.y) != (pj.y > p.y))
            && (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub(crate) fn winding_number(p: Pos2, polygon: &[Pos2]) -> i32 {
    let mut wn = 0i32;
    let n = polygon.len();
    for i in 0..n {
        let a = polygon[i];
        let b = polygon[(i + 1) % n];
        if a.y <= p.y {
            if b.y > p.y && cross2(b - a, p - a) > 0.0 {
                wn += 1;
            }
        } else if b.y <= p.y && cross2(b - a, p - a) < 0.0 {
            wn -= 1;
        }
    }
    wn
}

pub(crate) fn hash_noise(seed: u32, x: u32, y: u32) -> u8 {
    let mut n = seed ^ x.wrapping_mul(0x9E37_79B9) ^ y.wrapping_mul(0x85EB_CA6B);
    n ^= n >> 16;
    n = n.wrapping_mul(0x7FEB_352D);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846C_A68B);
    n ^= n >> 16;
    (n & 0xFF) as u8
}

pub(crate) fn nearest_bbox_corner(a: Pos2, b: Pos2, bbox: Rect) -> Option<Pos2> {
    let mid = pos2((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    let corners = [
        bbox.min,
        pos2(bbox.max.x, bbox.min.y),
        bbox.max,
        pos2(bbox.min.x, bbox.max.y),
    ];
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let nx = dy;
    let ny = -dx;
    corners
        .iter()
        .filter(|&&c| {
            let dot = (c.x - mid.x) * nx + (c.y - mid.y) * ny;
            dot > 1.0
        })
        .copied()
        .next()
}
