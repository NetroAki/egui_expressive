//! Color palette matching the Neutraudio UI mockup exactly.

use egui::Color32;

// Surface palette (from Tailwind slate)
pub const SURFACE_50: Color32 = Color32::from_rgb(248, 250, 252);
pub const SURFACE_100: Color32 = Color32::from_rgb(241, 245, 249);
pub const SURFACE_200: Color32 = Color32::from_rgb(226, 232, 240);
pub const SURFACE_300: Color32 = Color32::from_rgb(203, 213, 225);
pub const SURFACE_400: Color32 = Color32::from_rgb(148, 163, 184);
pub const SURFACE_500: Color32 = Color32::from_rgb(100, 116, 139);
pub const SURFACE_600: Color32 = Color32::from_rgb(71, 85, 105);
pub const SURFACE_700: Color32 = Color32::from_rgb(51, 65, 85);
pub const SURFACE_800: Color32 = Color32::from_rgb(30, 41, 59);
pub const SURFACE_900: Color32 = Color32::from_rgb(15, 23, 42);
pub const SURFACE_950: Color32 = Color32::from_rgb(2, 6, 23);

// Accent colors (from CSS custom properties)
pub const ACCENT_GLOW: Color32 = Color32::from_rgb(239, 68, 68); // red
pub const ACCENT_MIDI: Color32 = Color32::from_rgb(16, 185, 129); // green
pub const ACCENT_AUDIO: Color32 = Color32::from_rgb(139, 92, 246); // purple
pub const ACCENT_WARN: Color32 = Color32::from_rgb(245, 158, 11); // amber

// Mixer channel colors (per channel index)
pub const MIXER_COLORS: [Color32; 8] = [
    Color32::from_rgb(239, 68, 68),   // ch1 - red
    Color32::from_rgb(249, 115, 22),  // ch2 - orange
    Color32::from_rgb(34, 197, 94),   // ch3 - green
    Color32::from_rgb(6, 182, 212),   // ch4 - cyan
    Color32::from_rgb(59, 130, 246),  // ch5 - blue
    Color32::from_rgb(139, 92, 246),  // ch6 - purple
    Color32::from_rgb(236, 72, 153),  // ch7 - pink
    Color32::from_rgb(20, 184, 166),  // ch8 - teal
];
pub const MASTER_COLOR: Color32 = Color32::from_rgb(245, 158, 11); // amber

/// Background for panel title bars
pub const TITLE_BAR_BG: Color32 = Color32::from_rgba_premultiplied(30, 41, 59, 128); // surface-800/50
