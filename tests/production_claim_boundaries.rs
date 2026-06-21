use std::{fs, path::Path};

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative)).unwrap_or_else(|err| {
        panic!("failed to read {relative}: {err}");
    })
}

#[test]
fn public_docs_keep_release_and_support_claims_scoped() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = read(root, "README.md");
    assert!(
        readme.contains("pre-1.0") && readme.contains("evidence-scoped"),
        "README must keep the pre-1.0 / evidence-scoped boundary visible"
    );
    assert!(
        readme.contains("artifact-gated") || readme.contains("support evidence"),
        "README must keep platform support evidence-gated"
    );
    assert!(
        !readme.contains("Full Material Design 3 component set"),
        "README must not overclaim full M3 support while API stability remains evidence-scoped"
    );
    assert!(
        !readme.contains("Full feature/effect coverage dashboard"),
        "README examples must not imply full feature/effect coverage from a diagnostic dashboard"
    );

    let release_checklist = read(root, "docs/release-checklist.md");
    assert!(
        release_checklist.contains("release candidate only")
            && release_checklist.contains("registry publish")
            && release_checklist.contains("support claims must stay")
            && release_checklist.contains("within those documented bounds"),
        "release checklist must keep release/platform gates explicit"
    );
}

#[test]
fn web_artifact_stays_bounded() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = read(root, "README.md");
    assert!(
        readme.contains("Web | validated-bounded"),
        "README must reflect bounded Web proof without upgrading broad support claims"
    );

    let web = read(root, "docs/platform-smoke/web.md");
    assert!(
        web.contains("Status: `validated-bounded`")
            && web.contains("Chromium loopback")
            && web.contains("not every browser"),
        "Web artifact must record bounded browser-smoke scope"
    );
}

#[test]
fn linux_artifact_stays_bounded() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = read(root, "README.md");
    assert!(
        readme.contains("Linux | validated"),
        "README must reflect Linux proof without upgrading every platform"
    );

    let linux = read(root, "docs/platform-smoke/linux.md");
    assert!(
        linux.contains("Status: `validated`")
            && linux.contains("tools/linux_cross_platform_smoke.sh")
            && linux.contains("tools/linux_wayland_sway_smoke.sh")
            && linux.contains("X11/Xvfb/Openbox")
            && linux.contains("Wayland/Sway"),
        "Linux artifact must record reproducible X11/Wayland smoke harnesses"
    );
    assert!(
        linux.contains("not a guarantee for every Linux")
            && linux
                .contains("Publishing the crate does not imply a blanket production-ready claim"),
        "Linux artifact must not bypass production support boundaries"
    );
}

#[test]
fn android_artifact_stays_bounded() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = read(root, "README.md");
    assert!(
        readme.contains("Android | validated-bounded"),
        "README must reflect bounded Android proof without upgrading broad support claims"
    );

    let android = read(root, "docs/platform-smoke/android.md");
    assert!(
        android.contains("Status: `validated-bounded`")
            && android.contains("emulator evidence")
            && android.contains("not\na blanket guarantee"),
        "Android artifact must record bounded emulator-smoke scope"
    );
}

#[test]
fn visual_fixture_docs_do_not_promote_gradient_mesh_plumbing_to_exactness() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_readme = read(root, "tests/visual_diff/fixtures/README.md");
    assert!(
        fixture_readme.contains("pipeline/plumbing evidence only"),
        "gradient-mesh fixture must be labeled as plumbing evidence only"
    );
    assert!(
        fixture_readme.contains("does not prove exact gradient-mesh rendering"),
        "gradient-mesh fixture must not be exactness proof"
    );
    assert!(
        !fixture_readme.contains("gates strict-code export of gradient mesh patches"),
        "gradient-mesh fixture must not overclaim strict export gating/exactness"
    );
}

#[test]
fn release_checklist_keeps_initial_release_scope_bounded() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let checklist = read(root, "docs/release-checklist.md");
    assert!(
        checklist.contains("Initial Release Scope")
            && checklist.contains("Linux")
            && checklist.contains("Compile-only checks are not support claims"),
        "release checklist must keep the initial release scope bounded"
    );
    assert!(
        !checklist.contains("every platform is supported"),
        "release checklist must not promote broad platform support"
    );
}
