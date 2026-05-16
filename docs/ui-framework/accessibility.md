# Accessibility, Keyboard, and Internationalization Guide

Stage 8 makes accessibility semantics explicit and auditable while leaving native
screen-reader integration to egui/the host platform. This is not a browser DOM or
ARIA tree; the framework provides metadata, focus, motion, and live-region
contracts so custom-painted widgets do not bury important semantics in draw code.

## Source Map

- `src/accessibility/metadata.rs` — `AccessibilityMeta`, roles, labels, values,
  disabled state, and optional live-region metadata.
- `src/accessibility/live_region.rs` — `LiveRegion`, politeness, atomicity, and
  relevant-change descriptors.
- `src/accessibility/focus.rs` — `FocusRing`, `ModalFocusTrap`, and
  `RovingFocusGroup` for tabs/radios/toolbars.
- `src/accessibility/motion.rs` — `MotionPolicy` and reduced-motion preference.
- `src/interaction/focus.rs` — `FocusScope` and cyclic Tab/Shift+Tab traversal.
- `src/interaction/feedback.rs` — feedback severity to live-region mapping.
- `src/forms/input.rs` — paste, selection, IME, locale, and text-direction
  contract values.

## Metadata Pattern

```rust,no_run
use egui_expressive::{AccessibilityMeta, AccessibilityRole, LiveRegion};

let render = AccessibilityMeta::new(AccessibilityRole::Button, "Render")
    .description("Start offline export")
    .disabled(false);

let status = AccessibilityMeta::status("Export complete")
    .live_region(LiveRegion::polite("Export complete"));
# let _ = (render, status);
```

Use metadata for every custom-painted control that behaves like a native widget.
At minimum provide role, label, disabled state, and value for range/progress/text
controls.

## Keyboard Conventions

- **Tab / Shift+Tab**: move between major focusable controls through
  `FocusScope`.
- **Arrow keys**: move inside one composite widget using `RovingFocusGroup`.
  This is the right pattern for tabs, radio groups, segmented controls,
  toolbars, tree rows, and grid-like option pickers.
- **Home / End**: jump to first/last enabled item in roving groups.
- **Enter / Space**: activate focused buttons, menu items, checkboxes, and tabs.
- **Escape**: dismiss modal/dialog/popover scopes when safe; `ModalFocusTrap`
  exposes the close request without closing for the app.
- **Disabled items**: keep visible when useful, but skip during roving traversal.

```rust,no_run
use egui_expressive::{RovingFocusDirection, RovingFocusGroup, RovingFocusItem};

let one = egui::Id::new("one");
let two = egui::Id::new("two");
let group = RovingFocusGroup::new()
    .item(RovingFocusItem::new(one))
    .item(RovingFocusItem::new(two).disabled(true));

let next = group.resolve(Some(one), RovingFocusDirection::Next);
# let _ = next;
```

## Live-Region Feedback Pattern

Feedback queues keep UI policy pure and provide announcement descriptors:

- `Info` / `Success` → polite live region.
- `Warning` / `Error` → assertive live region.
- Progress → `progressbar` metadata with polite updates and optional percent
  value.
- Modal alerts should use `AlertDialog`; non-modal critical messages should use
  `Alert`; routine state should use `Status`.

```rust,no_run
use egui_expressive::{AccessibilityRole, FeedbackMessage, FeedbackSeverity};

let failed = FeedbackMessage::new("export_failed", "Export failed")
    .severity(FeedbackSeverity::Error);
let meta = failed.accessibility_meta(AccessibilityRole::Alert);
assert_eq!(meta.live_region.unwrap().politeness.as_str(), "assertive");
```

## Screen-Reader Audit Checklist

For each custom widget or example:

- Is the role specific enough (`button`, `tab`, `progressbar`, `treegrid`, etc.)?
- Is the label stable and user-facing, not a debug ID?
- Does value text exist for sliders, progress, meters, selected tabs, and editable
  text?
- Are disabled/read-only states surfaced in metadata or visible copy?
- Does focus order match visual and task order?
- Do composite widgets use roving focus instead of trapping Tab on every child?
- Are transient updates mapped to polite/assertive live regions?
- Are animations gated by `MotionPolicy` when they are non-essential?
- Are platform limits documented instead of claimed as certified behavior?

## Internationalization and RTL

Stage 8 does not add a localization framework or custom bidi renderer. App code
owns string catalogs, locale-specific formatting, and final text direction. The
library exposes `InputTextContract` and `TextDirection` so forms/examples can
state their assumptions clearly.

Guidance:

- Keep labels and descriptions as app-owned strings; avoid concatenating grammar
  fragments inside widgets.
- Store machine IDs separately from localized labels.
- Use `TextDirection::RightToLeft` for app surfaces that intentionally mirror
  labels/help text; leave `LocaleDefault` when direction comes from app locale.
- Treat IME preedit/composition as egui/platform-owned.
- Treat complex bidi shaping and cursor movement as platform-limited unless the
  host app validates it on target OS/input methods.
- Sanitize paste before validation for masked/numeric fields, then validate the
  sanitized text.

## Example Proof

- `examples/state_accessibility_gallery.rs` — Stage 7/8 baseline for metadata,
  focus rings, state variants, and reduced motion.
- `examples/accessibility_platform_gallery.rs` — Stage 8 walkthrough for roving
  focus, live-region feedback, RTL/input contract, clipboard/drop/system/DPI
  descriptors.
