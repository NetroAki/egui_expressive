use egui::Color32;

/// Complete M3 color scheme — all 30 semantic color roles.
#[derive(Clone, Debug)]
pub struct M3ColorScheme {
    // Primary
    pub primary: Color32,
    pub on_primary: Color32,
    pub primary_container: Color32,
    pub on_primary_container: Color32,
    pub primary_fixed: Color32,
    pub primary_fixed_dim: Color32,
    pub on_primary_fixed: Color32,
    pub on_primary_fixed_variant: Color32,

    // Secondary
    pub secondary: Color32,
    pub on_secondary: Color32,
    pub secondary_container: Color32,
    pub on_secondary_container: Color32,

    // Tertiary
    pub tertiary: Color32,
    pub on_tertiary: Color32,
    pub tertiary_container: Color32,
    pub on_tertiary_container: Color32,

    // Error
    pub error: Color32,
    pub on_error: Color32,
    pub error_container: Color32,
    pub on_error_container: Color32,

    // Surface
    pub surface: Color32,
    pub on_surface: Color32,
    pub surface_variant: Color32,
    pub on_surface_variant: Color32,
    pub surface_dim: Color32,
    pub surface_bright: Color32,
    pub surface_container_lowest: Color32,
    pub surface_container_low: Color32,
    pub surface_container: Color32,
    pub surface_container_high: Color32,
    pub surface_container_highest: Color32,

    // Other
    pub outline: Color32,
    pub outline_variant: Color32,
    pub inverse_surface: Color32,
    pub inverse_on_surface: Color32,
    pub inverse_primary: Color32,
    pub scrim: Color32,
    pub shadow: Color32,
}

impl M3ColorScheme {
    /// M3 baseline dark scheme (purple seed — matches Material You defaults).
    pub fn baseline_dark() -> Self {
        Self {
            primary: Color32::from_rgb(208, 188, 255),
            on_primary: Color32::from_rgb(56, 30, 114),
            primary_container: Color32::from_rgb(79, 55, 139),
            on_primary_container: Color32::from_rgb(234, 221, 255),
            primary_fixed: Color32::from_rgb(234, 221, 255),
            primary_fixed_dim: Color32::from_rgb(208, 188, 255),
            on_primary_fixed: Color32::from_rgb(33, 0, 93),
            on_primary_fixed_variant: Color32::from_rgb(79, 55, 139),

            secondary: Color32::from_rgb(204, 194, 220),
            on_secondary: Color32::from_rgb(51, 45, 65),
            secondary_container: Color32::from_rgb(74, 68, 88),
            on_secondary_container: Color32::from_rgb(232, 222, 248),

            tertiary: Color32::from_rgb(239, 184, 200),
            on_tertiary: Color32::from_rgb(73, 37, 50),
            tertiary_container: Color32::from_rgb(99, 59, 72),
            on_tertiary_container: Color32::from_rgb(255, 216, 228),

            error: Color32::from_rgb(242, 184, 181),
            on_error: Color32::from_rgb(96, 20, 16),
            error_container: Color32::from_rgb(140, 29, 24),
            on_error_container: Color32::from_rgb(255, 218, 214),

            surface: Color32::from_rgb(20, 18, 24),
            on_surface: Color32::from_rgb(230, 225, 229),
            surface_variant: Color32::from_rgb(73, 69, 79),
            on_surface_variant: Color32::from_rgb(202, 196, 208),
            surface_dim: Color32::from_rgb(20, 18, 24),
            surface_bright: Color32::from_rgb(59, 56, 62),
            surface_container_lowest: Color32::from_rgb(15, 13, 19),
            surface_container_low: Color32::from_rgb(29, 27, 32),
            surface_container: Color32::from_rgb(33, 31, 38),
            surface_container_high: Color32::from_rgb(43, 41, 48),
            surface_container_highest: Color32::from_rgb(54, 52, 59),

            outline: Color32::from_rgb(150, 144, 156),
            outline_variant: Color32::from_rgb(73, 69, 79),
            inverse_surface: Color32::from_rgb(230, 225, 229),
            inverse_on_surface: Color32::from_rgb(50, 47, 53),
            inverse_primary: Color32::from_rgb(103, 80, 164),
            scrim: Color32::from_rgb(0, 0, 0),
            shadow: Color32::from_rgb(0, 0, 0),
        }
    }

    /// M3 baseline light scheme.
    pub fn baseline_light() -> Self {
        Self {
            primary: Color32::from_rgb(103, 80, 164),
            on_primary: Color32::from_rgb(255, 255, 255),
            primary_container: Color32::from_rgb(234, 221, 255),
            on_primary_container: Color32::from_rgb(33, 0, 93),
            primary_fixed: Color32::from_rgb(234, 221, 255),
            primary_fixed_dim: Color32::from_rgb(208, 188, 255),
            on_primary_fixed: Color32::from_rgb(33, 0, 93),
            on_primary_fixed_variant: Color32::from_rgb(79, 55, 139),

            secondary: Color32::from_rgb(98, 91, 113),
            on_secondary: Color32::from_rgb(255, 255, 255),
            secondary_container: Color32::from_rgb(232, 222, 248),
            on_secondary_container: Color32::from_rgb(30, 25, 43),

            tertiary: Color32::from_rgb(125, 82, 96),
            on_tertiary: Color32::from_rgb(255, 255, 255),
            tertiary_container: Color32::from_rgb(255, 216, 228),
            on_tertiary_container: Color32::from_rgb(55, 11, 30),

            error: Color32::from_rgb(179, 38, 30),
            on_error: Color32::from_rgb(255, 255, 255),
            error_container: Color32::from_rgb(255, 218, 214),
            on_error_container: Color32::from_rgb(65, 14, 11),

            surface: Color32::from_rgb(255, 251, 254),
            on_surface: Color32::from_rgb(28, 27, 31),
            surface_variant: Color32::from_rgb(231, 224, 236),
            on_surface_variant: Color32::from_rgb(73, 69, 79),
            surface_dim: Color32::from_rgb(222, 216, 225),
            surface_bright: Color32::from_rgb(255, 251, 254),
            surface_container_lowest: Color32::from_rgb(255, 255, 255),
            surface_container_low: Color32::from_rgb(247, 242, 250),
            surface_container: Color32::from_rgb(243, 237, 247),
            surface_container_high: Color32::from_rgb(236, 230, 240),
            surface_container_highest: Color32::from_rgb(230, 224, 233),

            outline: Color32::from_rgb(121, 116, 126),
            outline_variant: Color32::from_rgb(202, 196, 208),
            inverse_surface: Color32::from_rgb(49, 48, 51),
            inverse_on_surface: Color32::from_rgb(244, 239, 244),
            inverse_primary: Color32::from_rgb(208, 188, 255),
            scrim: Color32::from_rgb(0, 0, 0),
            shadow: Color32::from_rgb(0, 0, 0),
        }
    }

    /// Generate a color scheme from a seed color using a simplified HCT-inspired algorithm.
    /// This is an approximation — for exact M3 dynamic color, use the full HCT algorithm.
    pub fn from_seed(seed: Color32, dark: bool) -> Self {
        // Extract hue from seed color using HSL
        let (h, s, _l) = rgb_to_hsl(seed.r(), seed.g(), seed.b());

        // Generate tonal palette by rotating hue and adjusting lightness
        let primary = hsl_to_rgb(h, s.max(0.48), if dark { 0.80 } else { 0.40 });
        let on_primary = hsl_to_rgb(h, s.max(0.48), if dark { 0.20 } else { 1.0 });
        let primary_container = hsl_to_rgb(h, s.max(0.48), if dark { 0.30 } else { 0.90 });
        let on_primary_container = hsl_to_rgb(h, s.max(0.48), if dark { 0.90 } else { 0.10 });

        let sec_h = (h + 30.0) % 360.0;
        let secondary = hsl_to_rgb(sec_h, 0.25, if dark { 0.80 } else { 0.40 });
        let on_secondary = hsl_to_rgb(sec_h, 0.25, if dark { 0.20 } else { 1.0 });
        let secondary_container = hsl_to_rgb(sec_h, 0.25, if dark { 0.30 } else { 0.90 });
        let on_secondary_container = hsl_to_rgb(sec_h, 0.25, if dark { 0.90 } else { 0.10 });

        let ter_h = (h + 60.0) % 360.0;
        let tertiary = hsl_to_rgb(ter_h, 0.35, if dark { 0.80 } else { 0.40 });
        let on_tertiary = hsl_to_rgb(ter_h, 0.35, if dark { 0.20 } else { 1.0 });
        let tertiary_container = hsl_to_rgb(ter_h, 0.35, if dark { 0.30 } else { 0.90 });
        let on_tertiary_container = hsl_to_rgb(ter_h, 0.35, if dark { 0.90 } else { 0.10 });

        let surface_l = if dark { 0.08 } else { 0.99 };
        let surface = hsl_to_rgb(h, 0.04, surface_l);
        let on_surface = hsl_to_rgb(h, 0.04, if dark { 0.90 } else { 0.10 });
        let surface_variant = hsl_to_rgb(h, 0.12, if dark { 0.30 } else { 0.90 });
        let on_surface_variant = hsl_to_rgb(h, 0.12, if dark { 0.80 } else { 0.30 });

        Self {
            primary,
            on_primary,
            primary_container,
            on_primary_container,
            primary_fixed: primary_container,
            primary_fixed_dim: primary,
            on_primary_fixed: on_primary_container,
            on_primary_fixed_variant: primary_container,

            secondary,
            on_secondary,
            secondary_container,
            on_secondary_container,

            tertiary,
            on_tertiary,
            tertiary_container,
            on_tertiary_container,

            error: Color32::from_rgb(
                if dark { 242 } else { 179 },
                if dark { 184 } else { 38 },
                if dark { 181 } else { 30 },
            ),
            on_error: Color32::from_rgb(
                if dark { 96 } else { 255 },
                if dark { 20 } else { 255 },
                if dark { 16 } else { 255 },
            ),
            error_container: Color32::from_rgb(
                if dark { 140 } else { 255 },
                if dark { 29 } else { 218 },
                if dark { 24 } else { 214 },
            ),
            on_error_container: Color32::from_rgb(
                if dark { 255 } else { 65 },
                if dark { 218 } else { 14 },
                if dark { 214 } else { 11 },
            ),

            surface,
            on_surface,
            surface_variant,
            on_surface_variant,
            surface_dim: hsl_to_rgb(h, 0.04, if dark { 0.06 } else { 0.87 }),
            surface_bright: hsl_to_rgb(h, 0.04, if dark { 0.24 } else { 0.99 }),
            surface_container_lowest: hsl_to_rgb(h, 0.04, if dark { 0.04 } else { 1.0 }),
            surface_container_low: hsl_to_rgb(h, 0.04, if dark { 0.10 } else { 0.96 }),
            surface_container: hsl_to_rgb(h, 0.04, if dark { 0.12 } else { 0.94 }),
            surface_container_high: hsl_to_rgb(h, 0.04, if dark { 0.17 } else { 0.92 }),
            surface_container_highest: hsl_to_rgb(h, 0.04, if dark { 0.22 } else { 0.90 }),

            outline: hsl_to_rgb(h, 0.08, if dark { 0.60 } else { 0.50 }),
            outline_variant: hsl_to_rgb(h, 0.08, if dark { 0.30 } else { 0.80 }),
            inverse_surface: hsl_to_rgb(h, 0.04, if dark { 0.90 } else { 0.20 }),
            inverse_on_surface: hsl_to_rgb(h, 0.04, if dark { 0.20 } else { 0.95 }),
            inverse_primary: hsl_to_rgb(h, s.max(0.48), if dark { 0.40 } else { 0.80 }),
            scrim: Color32::BLACK,
            shadow: Color32::BLACK,
        }
    }
}

// ── Color math helpers ────────────────────────────────────────────────────────

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < 1e-6 {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    (h * 360.0, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color32 {
    let h = h / 360.0;
    if s < 1e-6 {
        let v = (l * 255.0) as u8;
        return Color32::from_rgb(v, v, v);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// Blend two colors: result = base * (1 - alpha) + overlay * alpha
pub fn blend_overlay(base: Color32, overlay: Color32, alpha: f32) -> Color32 {
    let a = alpha.clamp(0.0, 1.0);
    Color32::from_rgb(
        (base.r() as f32 * (1.0 - a) + overlay.r() as f32 * a) as u8,
        (base.g() as f32 * (1.0 - a) + overlay.g() as f32 * a) as u8,
        (base.b() as f32 * (1.0 - a) + overlay.b() as f32 * a) as u8,
    )
}
