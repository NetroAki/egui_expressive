use super::*;

#[test]
fn test_easing_linear() {
    assert!((Easing::Linear.apply(0.0) - 0.0).abs() < 1e-6);
    assert!((Easing::Linear.apply(0.5) - 0.5).abs() < 1e-6);
    assert!((Easing::Linear.apply(1.0) - 1.0).abs() < 1e-6);
}

#[test]
fn test_easing_ease_in() {
    let t = 0.5;
    let result = Easing::EaseIn.apply(t);
    let expected = t * t * t;
    assert!((result - expected).abs() < 1e-6);
}

#[test]
fn test_easing_ease_out() {
    let t = 0.5;
    let result = Easing::EaseOut.apply(t);
    let expected = 1.0 - (1.0 - t).powi(3);
    assert!((result - expected).abs() < 1e-6);
}

#[test]
fn test_easing_ease_in_out() {
    let t = 0.25;
    let result = Easing::EaseInOut.apply(t);
    let expected = 4.0 * t * t * t;
    assert!((result - expected).abs() < 1e-6);

    let t = 0.75;
    let result = Easing::EaseInOut.apply(t);
    let expected = 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0;
    assert!((result - expected).abs() < 1e-6);
}

#[test]
fn test_easing_in_out_bounce() {
    assert!((Easing::EaseOutBounce.apply(0.0) - 0.0).abs() < 1e-4);
    assert!((Easing::EaseOutBounce.apply(1.0) - 1.0).abs() < 1e-4);
    assert!((Easing::EaseInBounce.apply(0.0) - 0.0).abs() < 1e-4);
    assert!((Easing::EaseInBounce.apply(1.0) - 1.0).abs() < 1e-4);
}

#[test]
fn test_easing_cubic_bezier() {
    let result = Easing::CubicBezier(0.0, 0.0, 1.0, 1.0).apply(0.5);
    assert!((result - 0.5).abs() < 0.15);

    let ease_in = Easing::CubicBezier(0.42, 0.0, 1.0, 1.0).apply(0.5);
    assert!(ease_in < 0.5);

    let ease_out = Easing::CubicBezier(0.0, 0.0, 0.58, 1.0).apply(0.5);
    assert!(ease_out > 0.5);
}

#[test]
fn test_color_roundtrip() {
    let c = Color32::from_rgba_unmultiplied(255, 255, 255, 255);
    let f = color_to_f32(c);
    for component in f {
        assert!((0.0..=255.0).contains(&component));
    }
}
