use super::*;

#[test]
fn modal_is_exclusive_and_suppresses_progress() {
    let mut queue = FeedbackQueue::new();
    let focus_id = Id::new("return_to_button");
    queue.push_progress(FeedbackProgress::new("load", "Loading", Some(0.5)));

    queue.push_modal(FeedbackMessage::new("confirm", "Confirm").focus_return(focus_id));

    assert!(queue.active_modal().is_some());
    assert!(queue.progress_suppressed_by_modal());
    assert!(queue.visible_progress().is_empty());
    assert_eq!(queue.dismiss_modal(), Some(focus_id));
    assert_eq!(queue.visible_progress().len(), 1);
}

#[test]
fn snackbars_are_fifo_with_one_visible() {
    let mut queue = FeedbackQueue::new();
    queue.push_snackbar(FeedbackMessage::new("one", "Saved"));
    queue.push_snackbar(FeedbackMessage::new("two", "Synced"));

    assert_eq!(queue.visible_snackbar().unwrap().id, "one");
    assert_eq!(queue.queued_snackbars(), 1);

    queue.dismiss_snackbar();

    assert_eq!(queue.visible_snackbar().unwrap().id, "two");
    assert_eq!(queue.queued_snackbars(), 0);
}

#[test]
fn toasts_are_bounded_and_expire() {
    let mut queue = FeedbackQueue::with_max_toasts(2);
    queue.push_toast(FeedbackToast::new("one", "One", 1.0));
    queue.push_toast(FeedbackToast::new("two", "Two", 2.0));
    queue.push_toast(FeedbackToast::new("three", "Three", 2.0));

    assert_eq!(queue.toasts().len(), 2);
    assert_eq!(queue.toasts()[0].message.id, "two");

    queue.tick(1.5);

    assert_eq!(queue.toasts().len(), 2);
    assert!(queue.toasts().iter().all(|toast| toast.seconds_left > 0.0));

    queue.tick(0.6);

    assert!(queue.toasts().is_empty());
}

#[test]
fn notifications_retain_feedback_history() {
    let mut queue = FeedbackQueue::new();
    queue.push_modal(FeedbackMessage::new("modal", "Needs attention"));
    queue.push_snackbar(FeedbackMessage::new("snackbar", "Saved"));
    queue.push_toast(FeedbackToast::new("toast", "Done", 1.0));

    let ids: Vec<_> = queue
        .notifications()
        .iter()
        .map(|message| message.id.as_str())
        .collect();
    assert_eq!(ids, vec!["modal", "snackbar", "toast"]);
}

#[test]
fn progress_updates_replace_matching_id() {
    let mut queue = FeedbackQueue::new();
    queue.push_progress(FeedbackProgress::new("load", "Loading", Some(0.1)));
    queue.push_progress(FeedbackProgress::new("load", "Loading", Some(0.9)));

    let visible = queue.visible_progress();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].fraction, Some(0.9));
}

#[test]
fn toasts_can_be_dismissed_by_id() {
    let mut queue = FeedbackQueue::new();
    queue.push_toast(FeedbackToast::new("keep", "Keep", 2.0));
    queue.push_toast(FeedbackToast::new("remove", "Remove", 2.0));

    let removed = queue.dismiss_toast("remove").unwrap();

    assert_eq!(removed.message.id, "remove");
    assert_eq!(queue.toasts().len(), 1);
    assert_eq!(queue.toasts()[0].message.id, "keep");
    assert!(queue.dismiss_toast("missing").is_none());
}

#[test]
fn progress_can_be_finished_by_id() {
    let mut queue = FeedbackQueue::new();
    queue.push_progress(FeedbackProgress::new("sync", "Sync", Some(0.25)));
    queue.push_progress(FeedbackProgress::new("export", "Export", None));

    let finished = queue.finish_progress("sync").unwrap();

    assert_eq!(finished.id, "sync");
    assert_eq!(queue.visible_progress().len(), 1);
    assert_eq!(queue.visible_progress()[0].id, "export");
    assert!(queue.finish_progress("missing").is_none());
}

#[test]
fn feedback_severity_maps_to_live_region_politeness() {
    let info = FeedbackMessage::new("saved", "Saved");
    let error = FeedbackMessage::new("failed", "Export failed").severity(FeedbackSeverity::Error);

    assert_eq!(info.live_region().politeness, LiveRegionPoliteness::Polite);
    assert_eq!(
        error.live_region().politeness,
        LiveRegionPoliteness::Assertive
    );
    assert_eq!(
        error
            .accessibility_meta(AccessibilityRole::Alert)
            .role
            .as_str(),
        "alert"
    );
}

#[test]
fn progress_metadata_carries_percent_value() {
    let progress = FeedbackProgress::new("export", "Exporting", Some(0.42));

    let meta = progress.accessibility_meta();

    assert_eq!(meta.role.as_str(), "progressbar");
    assert_eq!(meta.value.as_deref(), Some("42%"));
    assert_eq!(
        meta.live_region.unwrap().politeness,
        LiveRegionPoliteness::Polite
    );
}

#[test]
fn progress_metadata_omits_value_when_indeterminate() {
    let progress = FeedbackProgress::new("sync", "Syncing", None);

    let meta = progress.accessibility_meta();

    assert_eq!(meta.role.as_str(), "progressbar");
    assert!(meta.value.is_none());
    assert_eq!(
        meta.live_region.unwrap().politeness,
        LiveRegionPoliteness::Polite
    );
}
