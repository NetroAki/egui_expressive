use super::*;
#[test]
fn test_parse_hex_color_6() {
    let color = parse_hex_color("#ff8040").unwrap();
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 128);
    assert_eq!(color.b, 64);
    assert_eq!(color.a, 255);
}

#[test]
fn test_parse_hex_color_8() {
    let color = parse_hex_color("#ff804080").unwrap();
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 128);
    assert_eq!(color.b, 64);
    assert_eq!(color.a, 128);
}

#[test]
fn test_parse_color_value_rgb() {
    let color = parse_color_value("rgb(255, 128, 64)").unwrap();
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 128);
    assert_eq!(color.b, 64);
    assert_eq!(color.a, 255);
}

#[test]
fn test_parse_color_value_rgba() {
    let color = parse_color_value("rgba(255, 128, 64, 0.5)").unwrap();
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 128);
    assert_eq!(color.b, 64);
    assert_eq!(color.a, 127); // 0.5 * 255 = 127.5 truncates to 127
}

#[test]
fn test_figma_tokens_basic() {
    let json = r##"{
        "global": {
            "surface": {
                "50": { "value": "#f8f8f8", "type": "color" },
                "950": { "value": "#0a0a0a", "type": "color" }
            },
            "accent": {
                "glow": { "value": "#7c3aed", "type": "color" }
            },
            "spacing": {
                "md": { "value": "8", "type": "spacing" }
            },
            "rounding": {
                "md": { "value": "4", "type": "borderRadius" }
            }
        }
    }"##;

    let result = figma_tokens_to_rust(json);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let code = result.unwrap();
    assert!(code.contains("pub fn design_tokens()"));
    assert!(code.contains("SurfacePalette"));
    assert!(code.contains("AccentColors"));
}

#[test]
fn test_figma_tokens_no_wrapper() {
    let json = r##"{
        "surface": {
            "50": { "value": "#ffffff", "type": "color" }
        },
        "accent": {
            "glow": { "value": "#000000", "type": "color" }
        }
    }"##;

    let result = figma_tokens_to_rust(json);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
}
