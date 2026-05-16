//! Stack macros: vertical, horizontal, z-stack, spacer, and divider.

#[macro_export]
macro_rules! vstack {
    ($ui:expr, { $($body:tt)* }) => {
        $crate::vstack!($ui, gap: 0.0, { $($body)* })
    };
    ($ui:expr, gap: $gap:expr, { $($body:tt)* }) => {
        $crate::vstack!($ui, gap: $gap, padding: 0.0, { $($body)* })
    };
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, { $($body:tt)* }) => {
        $crate::vstack_impl!($ui, $gap, $padding, { $($body)* }, );
    };
    ($ui:expr, gap: $gap:expr, padding: [$($padding:expr),+], { $($body:tt)* }) => {
        $crate::vstack_impl!($ui, $gap, ($($padding),+), { $($body)* }, );
    };
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, bg: $bg:expr, { $($body:tt)* }) => {
        $crate::vstack_impl!($ui, $gap, $padding, { $($body)* }, bg: $bg);
    };
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, bg: $bg:expr, rounding: $rounding:expr, { $($body:tt)* }) => {
        $crate::vstack_impl!($ui, $gap, $padding, { $($body)* }, bg: $bg, rounding: $rounding);
    };
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, bg: $bg:expr, rounding: $rounding:expr, width: $width:expr, { $($body:tt)* }) => {
        $crate::vstack_impl!($ui, $gap, $padding, { $($body)* }, bg: $bg, rounding: $rounding, width: $width);
    };
}

#[macro_export]
macro_rules! vstack_impl {
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* },) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.y = $gap;
        let __resp = $ui.vertical(|__ui| { $($body)* });
        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* }, bg: $bg:expr) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.y = $gap;
        let __frame = $crate::layout::styled_frame($bg, 0.0, $padding, None);
        let __resp = __frame.show($ui, |__ui| __ui.vertical(|__ui| { $($body)* }));
        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* }, bg: $bg:expr, rounding: $rounding:expr) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.y = $gap;
        let __frame = $crate::layout::styled_frame($bg, $rounding, $padding, None);
        let __resp = __frame.show($ui, |__ui| __ui.vertical(|__ui| { $($body)* }));
        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* }, bg: $bg:expr, rounding: $rounding:expr, width: $width:expr) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.y = $gap;
        $ui.set_width($width);
        let __frame = $crate::layout::styled_frame($bg, $rounding, $padding, None);
        let __resp = __frame.show($ui, |__ui| __ui.vertical(|__ui| { $($body)* }));
        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};
}

#[macro_export]
macro_rules! hstack {
    ($ui:expr, { $($body:tt)* }) => {
        $crate::hstack!($ui, gap: 0.0, { $($body)* })
    };
    ($ui:expr, gap: $gap:expr, { $($body:tt)* }) => {
        $crate::hstack!($ui, gap: $gap, padding: 0.0, { $($body)* })
    };
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, { $($body:tt)* }) => {
        $crate::hstack_impl!($ui, $gap, $padding, { $($body)* }, );
    };
    ($ui:expr, gap: $gap:expr, padding: [$($padding:expr),+], { $($body:tt)* }) => {
        $crate::hstack_impl!($ui, $gap, ($($padding),+), { $($body)* }, );
    };
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, bg: $bg:expr, { $($body:tt)* }) => {
        $crate::hstack_impl!($ui, $gap, $padding, { $($body)* }, bg: $bg);
    };
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, bg: $bg:expr, rounding: $rounding:expr, { $($body:tt)* }) => {
        $crate::hstack_impl!($ui, $gap, $padding, { $($body)* }, bg: $bg, rounding: $rounding);
    };
}

#[macro_export]
macro_rules! hstack_impl {
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* },) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.x = $gap;
        let __resp = $ui.horizontal(|__ui| { $($body)* });
        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* }, bg: $bg:expr) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.x = $gap;
        let __frame = $crate::layout::styled_frame($bg, 0.0, $padding, None);
        let __resp = __frame.show($ui, |__ui| __ui.horizontal(|__ui| { $($body)* }));
        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* }, bg: $bg:expr, rounding: $rounding:expr) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.x = $gap;
        let __frame = $crate::layout::styled_frame($bg, $rounding, $padding, None);
        let __resp = __frame.show($ui, |__ui| __ui.horizontal(|__ui| { $($body)* }));
        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};
}

#[macro_export]
macro_rules! zstack {
    ($ui:expr, size: $size:expr, { $($body:tt)* }) => {{
        let __size = $size;
        let (__rect, __resp) = $ui.allocate_ui(__size, |__ui| { $($body)* });
        __resp
    }};
    ($ui:expr, size: $size:expr, bg: $bg:expr, { $($body:tt)* }) => {{
        let __size = $size;
        $ui.painter().rect_filled($ui.available_rect_before_wrap(), 0.0, $bg);
        let (__rect, __resp) = $ui.allocate_ui(__size, |__ui| { $($body)* });
        __resp
    }};
    ($ui:expr, size: $size:expr, bg: $bg:expr, rounding: $rounding:expr, { $($body:tt)* }) => {{
        let __size = $size;
        $ui.painter().rounded_rect_filled($ui.available_rect_before_wrap(), $rounding, $bg);
        let (__rect, __resp) = $ui.allocate_ui(__size, |__ui| { $($body)* });
        __resp
    }};
}

#[macro_export]
macro_rules! spacer {
    ($ui:expr) => {{
        let __size = $ui.available_size();
        $ui.allocate_space(__size);
    }};
    ($ui:expr, $size:expr) => {{
        $ui.allocate_space(egui::Vec2::splat($size));
    }};
}

#[macro_export]
macro_rules! divider {
    ($ui:expr) => {{
        $ui.separator();
    }};
    ($ui:expr, vertical) => {{
        $crate::layout::vrule($ui, egui::Color32::from_rgb(60, 60, 60), 1.0);
    }};
    ($ui:expr, vertical, $thickness:expr) => {{
        $crate::layout::vrule($ui, egui::Color32::from_rgb(60, 60, 60), $thickness);
    }};
    ($ui:expr, $color:expr) => {{
        $crate::layout::hrule($ui, $color, 1.0);
    }};
    ($ui:expr, vertical, $color:expr) => {{
        $crate::layout::vrule($ui, $color, 1.0);
    }};
    ($ui:expr, vertical, $color:expr, $thickness:expr) => {{
        $crate::layout::vrule($ui, $color, $thickness);
    }};
    ($ui:expr, $color:expr, $thickness:expr) => {{
        $crate::layout::hrule($ui, $color, $thickness);
    }};
}
