use egui_expressive::{diff_image_paths, diff_image_paths_with_heatmap, VisualDiffConfig};
use image::{Rgba, RgbaImage};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const REAL_FIXTURE_MANIFEST: &str = "tests/visual_diff/fixtures/manifest.tsv";

fn fixture_dir() -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "egui_expressive_visual_diff_{}_{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    fs::create_dir_all(&dir).expect("visual diff fixture dir");
    dir
}

fn save_fixture(dir: &Path, name: &str, pixels: &[Rgba<u8>]) -> PathBuf {
    assert_eq!(pixels.len(), 4, "2x2 fixture expects four pixels");
    let mut image = RgbaImage::new(2, 2);
    for (idx, pixel) in pixels.iter().enumerate() {
        image.put_pixel((idx % 2) as u32, (idx / 2) as u32, *pixel);
    }
    let path = dir.join(name);
    image.save(&path).expect("write png fixture");
    path
}

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "required"
    )
}

fn tolerance_requires_justification(config: &VisualDiffConfig) -> bool {
    config.max_channel_delta > 16
        || config.max_mean_delta > 1.0
        || config.max_bad_pixel_ratio > 0.001
}

fn has_tolerance_justification(lines: &[&str], line_idx: usize, case_name: &str) -> bool {
    has_manifest_metadata(lines, line_idx, case_name, "tolerance-justification:")
}

fn has_manifest_metadata(lines: &[&str], line_idx: usize, case_name: &str, label: &str) -> bool {
    lines[..line_idx]
        .iter()
        .rev()
        .map(|line| line.trim())
        .take_while(|line| line.is_empty() || line.starts_with('#'))
        .any(|line| line.contains(label) && line.contains(case_name))
}

fn score_class_for_case<'a>(
    lines: &'a [&'a str],
    line_idx: usize,
    case_name: &str,
) -> Option<&'a str> {
    for raw_line in lines[..line_idx].iter().rev() {
        let line = raw_line.trim();
        if !(line.is_empty() || line.starts_with('#')) {
            break;
        }

        let Some(rest) = line.strip_prefix('#') else {
            continue;
        };
        let Some(rest) = rest.trim().strip_prefix("score-class:") else {
            continue;
        };

        let mut parts = rest.split_whitespace();
        let tagged_case = parts.next().expect("score-class comment names a case");
        assert_eq!(
            tagged_case, case_name,
            "score-class comment before {case_name} must name that case"
        );

        let score_class = parts
            .next()
            .expect("score-class comment includes exact, bounded, or plumbing");
        assert!(
            matches!(score_class, "exact" | "bounded" | "plumbing"),
            "score-class for {case_name} must be exact, bounded, or plumbing, got {score_class}"
        );
        assert!(
            parts.next().is_none(),
            "score-class for {case_name} must contain only case name and class"
        );
        return Some(score_class);
    }

    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CropRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn crop_rect_for_case(lines: &[&str], line_idx: usize, case_name: &str) -> Option<CropRect> {
    for raw_line in lines[..line_idx].iter().rev() {
        let line = raw_line.trim();
        if !(line.is_empty() || line.starts_with('#')) {
            break;
        }
        let Some(rest) = line.strip_prefix('#') else {
            continue;
        };
        let Some(rest) = rest.trim().strip_prefix("crop-rect:") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let tagged_case = parts.next().expect("crop-rect comment names a case");
        assert_eq!(
            tagged_case, case_name,
            "crop-rect comment before {case_name} must name that case"
        );
        let values = parts
            .map(|value| value.parse::<u32>().expect("crop-rect values are integers"))
            .collect::<Vec<_>>();
        assert_eq!(
            values.len(),
            4,
            "crop-rect for {case_name} must contain x y width height"
        );
        assert!(
            values[2] > 0 && values[3] > 0,
            "crop-rect for {case_name} must use positive width and height"
        );
        return Some(CropRect {
            x: values[0],
            y: values[1],
            width: values[2],
            height: values[3],
        });
    }

    None
}

fn cropped_fixture_path(
    case_name: &str,
    role: &str,
    source_path: &Path,
    crop: CropRect,
    output_dir: &Path,
) -> PathBuf {
    let crop_dir = output_dir.join("crops");
    fs::create_dir_all(&crop_dir).expect("visual diff crop output dir");
    let image = image::open(source_path)
        .unwrap_or_else(|err| panic!("visual fixture {case_name} crop source failed: {err}"));
    assert!(
        crop.x + crop.width <= image.width() && crop.y + crop.height <= image.height(),
        "crop-rect for {case_name} exceeds {role} image bounds"
    );
    let crop_image = image.crop_imm(crop.x, crop.y, crop.width, crop.height);
    let path = crop_dir.join(format!("{case_name}-{role}.png"));
    crop_image
        .save(&path)
        .unwrap_or_else(|err| panic!("save cropped visual fixture {case_name}: {err}"));
    path
}

#[test]
fn visual_parity_corpus_smoke_test() {
    let dir = fixture_dir();
    let expected_pixels = [
        Rgba([18, 52, 86, 255]),
        Rgba([240, 200, 32, 255]),
        Rgba([64, 128, 192, 255]),
        Rgba([0, 0, 0, 0]),
    ];

    let expected = save_fixture(&dir, "ai-reference.png", &expected_pixels);
    let exact = save_fixture(&dir, "egui-exact.png", &expected_pixels);
    let tolerated = save_fixture(
        &dir,
        "egui-tolerated.png",
        &[
            Rgba([19, 53, 87, 255]),
            expected_pixels[1],
            expected_pixels[2],
            expected_pixels[3],
        ],
    );
    let mismatch = save_fixture(
        &dir,
        "egui-mismatch.png",
        &[
            Rgba([255, 0, 0, 255]),
            expected_pixels[1],
            expected_pixels[2],
            expected_pixels[3],
        ],
    );

    let config = VisualDiffConfig {
        max_channel_delta: 2,
        max_mean_delta: 1.0,
        max_bad_pixel_ratio: 0.0,
        compare_alpha: true,
    };

    let exact_report = diff_image_paths(&expected, &exact, config).expect("exact fixture diff");
    assert!(exact_report.passed, "{}", exact_report.summary());

    let tolerated_report =
        diff_image_paths(&expected, &tolerated, config).expect("tolerated fixture diff");
    assert!(tolerated_report.passed, "{}", tolerated_report.summary());

    let mismatch_report =
        diff_image_paths(&expected, &mismatch, config).expect("mismatch fixture diff");
    assert!(
        !mismatch_report.passed,
        "mismatch should fail: {}",
        mismatch_report.summary()
    );
}

#[test]
fn real_visual_fixture_manifest_is_wired() {
    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let fixture_root = manifest_path.parent().expect("fixture manifest parent");
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();
    let output_dir = repo_path("test-results/visual-diff");
    fs::create_dir_all(&output_dir).expect("visual diff output dir");

    let mut cases = 0;
    let mut compared = 0;
    let mut skipped_optional = 0;
    for (line_idx, raw_line) in manifest_lines.iter().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            columns.len(),
            8,
            "{}:{} must have 8 tab-separated columns",
            REAL_FIXTURE_MANIFEST,
            line_idx + 1
        );
        cases += 1;

        let case_name = columns[0];
        assert!(
            case_name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'),
            "fixture case names must be filesystem-safe: {case_name}"
        );
        let expected_path = fixture_root.join(columns[1]);
        let actual_path = fixture_root.join(columns[2]);
        let config = VisualDiffConfig {
            max_channel_delta: columns[3].parse().expect("max_channel_delta"),
            max_mean_delta: columns[4].parse().expect("max_mean_delta"),
            max_bad_pixel_ratio: columns[5].parse().expect("max_bad_pixel_ratio"),
            compare_alpha: parse_bool(columns[6]),
        };
        assert!(
            !tolerance_requires_justification(&config)
                || has_tolerance_justification(&manifest_lines, line_idx, case_name),
            "fixture case {case_name} uses broad visual-diff tolerance without a preceding tolerance-justification comment"
        );
        let required = parse_bool(columns[7]);

        if !expected_path.exists() || !actual_path.exists() {
            if required {
                panic!(
                    "required visual fixture case {case_name} is incomplete: expected={} actual={}",
                    expected_path.display(),
                    actual_path.display()
                );
            }
            skipped_optional += 1;
            continue;
        }

        let crop_rect = crop_rect_for_case(&manifest_lines, line_idx, case_name);
        let (compare_expected, compare_actual) = if let Some(crop) = crop_rect {
            (
                cropped_fixture_path(case_name, "expected", &expected_path, crop, &output_dir),
                cropped_fixture_path(case_name, "actual", &actual_path, crop, &output_dir),
            )
        } else {
            (expected_path, actual_path)
        };

        let heatmap_path = output_dir.join(format!("{case_name}-heatmap.png"));
        let report = diff_image_paths_with_heatmap(
            &compare_expected,
            &compare_actual,
            &heatmap_path,
            config,
        )
        .unwrap_or_else(|err| panic!("visual fixture case {case_name} failed to load: {err}"));
        assert!(
            report.passed,
            "fixture {case_name} failed: {} heatmap={}",
            report.summary(),
            heatmap_path.display()
        );
        compared += 1;
    }

    assert!(
        cases > 0,
        "visual fixture manifest must contain at least one case"
    );
    assert!(
        compared > 0 || skipped_optional > 0,
        "visual fixture manifest must compare or explicitly skip an optional real fixture"
    );
}

#[test]
fn visual_fixture_manifest_covers_stage7_and_release_targets() {
    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let cases = manifest
        .lines()
        .filter_map(|raw_line| {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                None
            } else {
                line.split('\t').next()
            }
        })
        .collect::<BTreeSet<_>>();

    for required_case in [
        "stage7-tailwind-effects",
        "tailwind-layout-bounds",
        "tailwind-soft-shadow",
        "tailwind-backdrop-layered",
        "tailwind-supported-drop-shadow-wgpu",
        "tailwind-supported-backdrop-snapshot-blur",
        "backdrop-supported-app-snapshot-blur",
        "scene-supported-gaussian-blur",
        "scene-supported-feather",
        "scene-supported-drop-shadow",
        "scene-supported-outer-glow",
        "scene-supported-rounded-rect-blur",
        "scene-supported-ellipse-drop-shadow",
        "scene-supported-path-feather",
        "scene-supported-rotated-rect-drop-shadow",
        "editor-canvas-selection-states",
        "clip-layered-background",
        "typography-token-panel",
        "m3-component-states",
        "icon-button-token-states",
        "animation-transition-frames",
        "form-data-widget-polish",
        "compositing-blend-boundary",
        "design-tool-stroke-boundary",
        "phase5-supported-gradient",
        "phase6-supported-gradient-angle",
        "phase6-supported-rounded-stroke",
        "phase7-supported-polygon-clip-gradient",
        "phase7-supported-compound-hole-fill",
        "phase7-supported-multiply-stack",
        "typography-supported-family-selection",
        "tailwind-supported-gradient-card",
        "tailwind-supported-state-endpoints",
        "typography-supported-ascii-panel",
        "typography-supported-decoration-overflow",
        "m3-top-app-bar-states",
        "m3-button-card-states",
        "m3-input-control-states",
        "m3-text-field-states",
        "m3-navigation-list-states",
        "figma-token-placeholder-boundary",
        "ui-assets-page1",
        "ui-assets-page1-el3-fill",
        "ui-assets-page1-el4-fill",
        "gradient-mesh-quad",
        "vector-clip-nested",
        "compound-clip-hole",
    ] {
        assert!(
            cases.contains(required_case),
            "visual fixture manifest must keep {required_case} wired"
        );
    }
}

#[test]
fn visual_fixture_manifest_tolerances_are_governed() {
    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();

    for (line_idx, raw_line) in manifest_lines.iter().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(columns.len(), 8);
        let case_name = columns[0];
        let config = VisualDiffConfig {
            max_channel_delta: columns[3].parse().expect("max_channel_delta"),
            max_mean_delta: columns[4].parse().expect("max_mean_delta"),
            max_bad_pixel_ratio: columns[5].parse().expect("max_bad_pixel_ratio"),
            compare_alpha: parse_bool(columns[6]),
        };

        assert!(
            !tolerance_requires_justification(&config)
                || has_tolerance_justification(&manifest_lines, line_idx, case_name),
            "broad fixture tolerance for {case_name} must be justified in manifest comments"
        );
    }
}

#[test]
fn required_visual_fixture_manifest_cases_have_metadata() {
    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();

    for (line_idx, raw_line) in manifest_lines.iter().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(columns.len(), 8);
        let case_name = columns[0];
        let required = parse_bool(columns[7]);
        if !required {
            continue;
        }

        for label in ["fixture-intent:", "fixture-source:", "fixture-backend:"] {
            assert!(
                has_manifest_metadata(&manifest_lines, line_idx, case_name, label),
                "required visual fixture {case_name} must have preceding {label} metadata comment naming the case"
            );
        }
    }
}

#[test]
fn required_visual_fixture_manifest_cases_have_score_class() {
    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();

    for (line_idx, raw_line) in manifest_lines.iter().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(columns.len(), 8);
        let case_name = columns[0];
        let required = parse_bool(columns[7]);
        if !required {
            continue;
        }

        assert!(
            score_class_for_case(&manifest_lines, line_idx, case_name).is_some(),
            "required visual fixture {case_name} must have preceding score-class metadata comment naming the case"
        );
    }
}

#[test]
fn exact_visual_fixture_rows_use_strict_zero_tolerance() {
    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();

    for (line_idx, raw_line) in manifest_lines.iter().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(columns.len(), 8);
        let case_name = columns[0];
        if score_class_for_case(&manifest_lines, line_idx, case_name) != Some("exact") {
            continue;
        }

        let config = VisualDiffConfig {
            max_channel_delta: columns[3].parse().expect("max_channel_delta"),
            max_mean_delta: columns[4].parse().expect("max_mean_delta"),
            max_bad_pixel_ratio: columns[5].parse().expect("max_bad_pixel_ratio"),
            compare_alpha: parse_bool(columns[6]),
        };

        assert_eq!(
            config.max_channel_delta, 0,
            "exact visual fixture {case_name} must use zero max_channel_delta"
        );
        assert_eq!(
            config.max_mean_delta, 0.0,
            "exact visual fixture {case_name} must use zero max_mean_delta"
        );
        assert_eq!(
            config.max_bad_pixel_ratio, 0.0,
            "exact visual fixture {case_name} must use zero max_bad_pixel_ratio"
        );
    }
}

#[test]
fn broad_tolerance_visual_fixture_rows_are_not_exact() {
    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();

    for (line_idx, raw_line) in manifest_lines.iter().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(columns.len(), 8);
        let case_name = columns[0];
        let config = VisualDiffConfig {
            max_channel_delta: columns[3].parse().expect("max_channel_delta"),
            max_mean_delta: columns[4].parse().expect("max_mean_delta"),
            max_bad_pixel_ratio: columns[5].parse().expect("max_bad_pixel_ratio"),
            compare_alpha: parse_bool(columns[6]),
        };

        if config.max_channel_delta == 0
            && config.max_mean_delta == 0.0
            && config.max_bad_pixel_ratio == 0.0
        {
            continue;
        }

        assert_ne!(
            score_class_for_case(&manifest_lines, line_idx, case_name),
            Some("exact"),
            "non-zero tolerance fixture {case_name} must not be exact-score evidence"
        );
    }
}

#[test]
fn placeholder_gradient_mesh_quad_stays_non_exact() {
    let reference = repo_path("tests/visual_diff/fixtures/illustrator/gradient-mesh-quad.png");
    let image = image::open(reference)
        .expect("Illustrator gradient mesh reference fixture should be committed")
        .to_rgba8();
    assert_eq!([image.width(), image.height()], [2, 2]);

    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();
    let (line_idx, _) = manifest_lines
        .iter()
        .enumerate()
        .find(|(_, line)| line.starts_with("gradient-mesh-quad\t"))
        .expect("gradient-mesh-quad manifest row exists");

    assert_ne!(
        score_class_for_case(&manifest_lines, line_idx, "gradient-mesh-quad"),
        Some("exact"),
        "2x2 gradient-mesh-quad placeholder must not count as exact-score evidence"
    );
}

#[test]
fn phase5_supported_gradient_fixture_is_committed() {
    let vector_source = repo_path("tests/visual_diff/fixtures/egui/phase5-supported-gradient.svg");
    assert!(
        vector_source.exists(),
        "Phase 5 supported gradient fixture should keep its vector source"
    );

    let reference =
        repo_path("tests/visual_diff/fixtures/illustrator/phase5-supported-gradient.png");
    let actual = repo_path("tests/visual_diff/fixtures/egui/phase5-supported-gradient.png");
    for path in [&reference, &actual] {
        let image = image::open(path)
            .unwrap_or_else(|err| panic!("phase5-supported-gradient PNG exists: {err}"))
            .to_rgba8();
        assert_eq!([image.width(), image.height()], [64, 64]);
    }

    let added_asset_bytes = fs::metadata(vector_source)
        .expect("phase5-supported-gradient SVG metadata")
        .len()
        + fs::metadata(reference)
            .expect("phase5-supported-gradient reference metadata")
            .len()
        + fs::metadata(actual)
            .expect("phase5-supported-gradient actual metadata")
            .len();
    assert!(
        added_asset_bytes < 100 * 1024,
        "Phase 5 supported-gradient assets must stay below 100 KiB, got {added_asset_bytes} bytes"
    );
}

#[test]
fn phase6_supported_subset_fixtures_are_committed() {
    let external_cases = [
        ("phase6-supported-gradient-angle", 64, 64),
        ("phase6-supported-rounded-stroke", 96, 64),
    ];
    let mut external_asset_bytes = 0;

    for (case, width, height) in external_cases {
        let vector_source = repo_path(&format!("tests/visual_diff/fixtures/egui/{case}.svg"));
        assert!(
            vector_source.exists(),
            "{case} should keep its vector source beside rendered PNGs"
        );
        external_asset_bytes += fs::metadata(&vector_source)
            .unwrap_or_else(|err| panic!("{case} SVG metadata: {err}"))
            .len();

        for folder in ["illustrator", "egui"] {
            let path = repo_path(&format!("tests/visual_diff/fixtures/{folder}/{case}.png"));
            let image = image::open(&path)
                .unwrap_or_else(|err| panic!("{case} {folder} PNG exists: {err}"))
                .to_rgba8();
            assert_eq!([image.width(), image.height()], [width, height]);
            external_asset_bytes += fs::metadata(&path)
                .unwrap_or_else(|err| panic!("{case} {folder} PNG metadata: {err}"))
                .len();
        }
    }

    assert!(
        external_asset_bytes < 200 * 1024,
        "Phase 6 external exact-subset assets must stay below 200 KiB, got {external_asset_bytes} bytes"
    );

    for (case, width, height) in [
        ("tailwind-supported-gradient-card", 96, 64),
        ("typography-supported-ascii-panel", 96, 64),
        ("m3-top-app-bar-states", 96, 64),
    ] {
        for suffix in ["expected", "actual"] {
            let path = repo_path(&format!(
                "tests/visual_diff/fixtures/headless/{case}-{suffix}.png"
            ));
            let image = image::open(&path)
                .unwrap_or_else(|err| panic!("{case} {suffix} PNG exists: {err}"))
                .to_rgba8();
            assert_eq!([image.width(), image.height()], [width, height]);
        }
    }
}

#[test]
fn phase7_exact_core_slice_fixtures_are_committed() {
    let external_cases = [
        ("phase7-supported-polygon-clip-gradient", 96, 64),
        ("phase7-supported-compound-hole-fill", 96, 64),
        ("phase7-supported-multiply-stack", 96, 64),
    ];
    let mut external_asset_bytes = 0;

    for (case, width, height) in external_cases {
        let vector_source = repo_path(&format!("tests/visual_diff/fixtures/egui/{case}.svg"));
        assert!(
            vector_source.exists(),
            "{case} should keep its vector source beside rendered PNGs"
        );
        external_asset_bytes += fs::metadata(&vector_source)
            .unwrap_or_else(|err| panic!("{case} SVG metadata: {err}"))
            .len();

        for folder in ["illustrator", "egui"] {
            let path = repo_path(&format!("tests/visual_diff/fixtures/{folder}/{case}.png"));
            let image = image::open(&path)
                .unwrap_or_else(|err| panic!("{case} {folder} PNG exists: {err}"))
                .to_rgba8();
            assert_eq!([image.width(), image.height()], [width, height]);
            external_asset_bytes += fs::metadata(&path)
                .unwrap_or_else(|err| panic!("{case} {folder} PNG metadata: {err}"))
                .len();
        }
    }

    assert!(
        external_asset_bytes < 300 * 1024,
        "Phase 7 external exact-slice assets must stay below 300 KiB, got {external_asset_bytes} bytes"
    );

    for (case, width, height) in [
        ("tailwind-supported-state-endpoints", 96, 64),
        ("typography-supported-decoration-overflow", 96, 64),
        ("m3-button-card-states", 96, 64),
    ] {
        for suffix in ["expected", "actual"] {
            let path = repo_path(&format!(
                "tests/visual_diff/fixtures/headless/{case}-{suffix}.png"
            ));
            let image = image::open(&path)
                .unwrap_or_else(|err| panic!("{case} {suffix} PNG exists: {err}"))
                .to_rgba8();
            assert_eq!([image.width(), image.height()], [width, height]);
        }
    }
}

#[test]
fn phase8_fixtures_and_crop_slices_are_committed() {
    for (case, width, height) in [
        ("typography-supported-family-selection", 96, 64),
        ("m3-input-control-states", 96, 64),
        ("m3-text-field-states", 96, 64),
        ("m3-navigation-list-states", 96, 64),
    ] {
        for suffix in ["expected", "actual"] {
            let path = repo_path(&format!(
                "tests/visual_diff/fixtures/headless/{case}-{suffix}.png"
            ));
            let image = image::open(&path)
                .unwrap_or_else(|err| panic!("{case} {suffix} PNG exists: {err}"))
                .to_rgba8();
            assert_eq!([image.width(), image.height()], [width, height]);
        }
    }

    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();
    for (case, expected_crop) in [
        (
            "ui-assets-page1-el3-fill",
            CropRect {
                x: 3056,
                y: 512,
                width: 64,
                height: 64,
            },
        ),
        (
            "ui-assets-page1-el4-fill",
            CropRect {
                x: 512,
                y: 592,
                width: 64,
                height: 64,
            },
        ),
    ] {
        let (line_idx, _) = manifest_lines
            .iter()
            .enumerate()
            .find(|(_, line)| line.starts_with(&format!("{case}\t")))
            .unwrap_or_else(|| panic!("{case} manifest row exists"));
        assert_eq!(
            crop_rect_for_case(&manifest_lines, line_idx, case),
            Some(expected_crop)
        );
        assert_eq!(
            score_class_for_case(&manifest_lines, line_idx, case),
            Some("exact")
        );
    }
}

#[test]
fn phase9a_effect_blur_fixtures_are_committed() {
    for case in ["scene-supported-gaussian-blur", "scene-supported-feather"] {
        for suffix in ["expected", "actual"] {
            let path = repo_path(&format!(
                "tests/visual_diff/fixtures/headless/{case}-{suffix}.png"
            ));
            let image = image::open(&path)
                .unwrap_or_else(|err| panic!("{case} {suffix} PNG exists: {err}"))
                .to_rgba8();
            assert_eq!([image.width(), image.height()], [96, 64]);
        }
    }

    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();
    for case in ["scene-supported-gaussian-blur", "scene-supported-feather"] {
        let (line_idx, _) = manifest_lines
            .iter()
            .enumerate()
            .find(|(_, line)| line.starts_with(&format!("{case}\t")))
            .unwrap_or_else(|| panic!("{case} manifest row exists"));
        assert_eq!(
            score_class_for_case(&manifest_lines, line_idx, case),
            Some("exact")
        );
    }
}

#[test]
fn phase9b_effect_shadow_fixtures_are_committed() {
    for case in ["scene-supported-drop-shadow", "scene-supported-outer-glow"] {
        for suffix in ["expected", "actual"] {
            let path = repo_path(&format!(
                "tests/visual_diff/fixtures/headless/{case}-{suffix}.png"
            ));
            let image = image::open(&path)
                .unwrap_or_else(|err| panic!("{case} {suffix} PNG exists: {err}"))
                .to_rgba8();
            assert_eq!([image.width(), image.height()], [96, 64]);
        }
    }

    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();
    for case in ["scene-supported-drop-shadow", "scene-supported-outer-glow"] {
        let (line_idx, _) = manifest_lines
            .iter()
            .enumerate()
            .find(|(_, line)| line.starts_with(&format!("{case}\t")))
            .unwrap_or_else(|| panic!("{case} manifest row exists"));
        assert_eq!(
            score_class_for_case(&manifest_lines, line_idx, case),
            Some("exact")
        );
    }
}

#[test]
fn r100_001a_backdrop_snapshot_fixtures_are_committed() {
    let case = "backdrop-supported-app-snapshot-blur";
    let source_path = repo_path(&format!(
        "tests/visual_diff/fixtures/headless/{case}-source.png"
    ));
    let expected_path = repo_path(&format!(
        "tests/visual_diff/fixtures/headless/{case}-expected.png"
    ));
    let actual_path = repo_path(&format!(
        "tests/visual_diff/fixtures/headless/{case}-actual.png"
    ));

    let source = image::open(&source_path)
        .unwrap_or_else(|err| panic!("{case} source PNG exists: {err}"))
        .to_rgba8();
    let expected = image::open(&expected_path)
        .unwrap_or_else(|err| panic!("{case} expected PNG exists: {err}"))
        .to_rgba8();
    let actual = image::open(&actual_path)
        .unwrap_or_else(|err| panic!("{case} actual PNG exists: {err}"))
        .to_rgba8();

    assert_eq!([source.width(), source.height()], [96, 64]);
    assert_eq!([expected.width(), expected.height()], [96, 64]);
    assert_eq!([actual.width(), actual.height()], [96, 64]);

    let first_source_color = *source
        .pixels()
        .next()
        .expect("source image contains pixels");
    let has_multiple_source_colors = source.pixels().any(|pixel| *pixel != first_source_color);
    assert!(
        has_multiple_source_colors,
        "{case} source must be nontrivial and high-contrast"
    );
    assert_ne!(
        source.as_raw(),
        expected.as_raw(),
        "{case} expected blur must differ from source snapshot"
    );
    assert_eq!(
        expected.as_raw(),
        actual.as_raw(),
        "{case} expected and actual exact fixtures must match"
    );

    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();
    let (line_idx, row) = manifest_lines
        .iter()
        .enumerate()
        .find(|(_, line)| line.starts_with(&format!("{case}\t")))
        .unwrap_or_else(|| panic!("{case} manifest row exists"));
    let columns: Vec<&str> = row.split('\t').collect();
    assert_eq!(columns.len(), 8);
    assert_eq!(
        score_class_for_case(&manifest_lines, line_idx, case),
        Some("exact")
    );
    assert!(parse_bool(columns[7]), "{case} must be required=true");
}

#[test]
fn r100_002_tailwind_exact_effect_fixtures_are_committed() {
    let drop_case = "tailwind-supported-drop-shadow-wgpu";
    let drop_expected = image::open(repo_path(&format!(
        "tests/visual_diff/fixtures/headless/{drop_case}-expected.png"
    )))
    .unwrap_or_else(|err| panic!("{drop_case} expected PNG exists: {err}"))
    .to_rgba8();
    let drop_actual = image::open(repo_path(&format!(
        "tests/visual_diff/fixtures/headless/{drop_case}-actual.png"
    )))
    .unwrap_or_else(|err| panic!("{drop_case} actual PNG exists: {err}"))
    .to_rgba8();

    assert_eq!([drop_expected.width(), drop_expected.height()], [96, 64]);
    assert_eq!([drop_actual.width(), drop_actual.height()], [96, 64]);
    assert_eq!(
        drop_expected.as_raw(),
        drop_actual.as_raw(),
        "{drop_case} expected and actual exact fixtures must match"
    );

    let backdrop_case = "tailwind-supported-backdrop-snapshot-blur";
    let backdrop_source = image::open(repo_path(&format!(
        "tests/visual_diff/fixtures/headless/{backdrop_case}-source.png"
    )))
    .unwrap_or_else(|err| panic!("{backdrop_case} source PNG exists: {err}"))
    .to_rgba8();
    let backdrop_expected = image::open(repo_path(&format!(
        "tests/visual_diff/fixtures/headless/{backdrop_case}-expected.png"
    )))
    .unwrap_or_else(|err| panic!("{backdrop_case} expected PNG exists: {err}"))
    .to_rgba8();
    let backdrop_actual = image::open(repo_path(&format!(
        "tests/visual_diff/fixtures/headless/{backdrop_case}-actual.png"
    )))
    .unwrap_or_else(|err| panic!("{backdrop_case} actual PNG exists: {err}"))
    .to_rgba8();

    assert_eq!(
        [backdrop_source.width(), backdrop_source.height()],
        [96, 64]
    );
    assert_eq!(
        [backdrop_expected.width(), backdrop_expected.height()],
        [96, 64]
    );
    assert_eq!(
        [backdrop_actual.width(), backdrop_actual.height()],
        [96, 64]
    );
    let first_source_color = *backdrop_source
        .pixels()
        .next()
        .expect("source image contains pixels");
    assert!(
        backdrop_source
            .pixels()
            .any(|pixel| *pixel != first_source_color),
        "{backdrop_case} source must be nontrivial and high-contrast"
    );
    assert_ne!(
        backdrop_source.as_raw(),
        backdrop_expected.as_raw(),
        "{backdrop_case} expected blur must differ from source snapshot"
    );
    assert_eq!(
        backdrop_expected.as_raw(),
        backdrop_actual.as_raw(),
        "{backdrop_case} expected and actual exact fixtures must match"
    );

    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();
    for case in [drop_case, backdrop_case] {
        let (line_idx, row) = manifest_lines
            .iter()
            .enumerate()
            .find(|(_, line)| line.starts_with(&format!("{case}\t")))
            .unwrap_or_else(|| panic!("{case} manifest row exists"));
        let columns: Vec<&str> = row.split('\t').collect();
        assert_eq!(columns.len(), 8);
        assert_eq!(
            score_class_for_case(&manifest_lines, line_idx, case),
            Some("exact")
        );
        assert!(parse_bool(columns[7]), "{case} must be required=true");
    }
}

#[test]
fn r100_003a_shaped_scene_effect_fixtures_are_committed() {
    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();

    for case in [
        "scene-supported-rounded-rect-blur",
        "scene-supported-ellipse-drop-shadow",
        "scene-supported-path-feather",
        "scene-supported-rotated-rect-drop-shadow",
    ] {
        let expected = image::open(repo_path(&format!(
            "tests/visual_diff/fixtures/headless/{case}-expected.png"
        )))
        .unwrap_or_else(|err| panic!("{case} expected PNG exists: {err}"))
        .to_rgba8();
        let actual = image::open(repo_path(&format!(
            "tests/visual_diff/fixtures/headless/{case}-actual.png"
        )))
        .unwrap_or_else(|err| panic!("{case} actual PNG exists: {err}"))
        .to_rgba8();

        assert_eq!([expected.width(), expected.height()], [96, 64]);
        assert_eq!([actual.width(), actual.height()], [96, 64]);
        assert_eq!(
            expected.as_raw(),
            actual.as_raw(),
            "{case} expected and actual exact fixtures must match"
        );

        let (line_idx, row) = manifest_lines
            .iter()
            .enumerate()
            .find(|(_, line)| line.starts_with(&format!("{case}\t")))
            .unwrap_or_else(|| panic!("{case} manifest row exists"));
        let columns: Vec<&str> = row.split('\t').collect();
        assert_eq!(columns.len(), 8);
        assert_eq!(
            score_class_for_case(&manifest_lines, line_idx, case),
            Some("exact")
        );
        assert!(parse_bool(columns[7]), "{case} must be required=true");
    }
}

#[test]
fn r100_001a_tailwind_backdrop_fixture_stays_bounded() {
    let manifest_path = repo_path(REAL_FIXTURE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).expect("visual fixture manifest exists");
    let manifest_lines = manifest.lines().collect::<Vec<_>>();
    let case = "tailwind-backdrop-layered";
    let (line_idx, _) = manifest_lines
        .iter()
        .enumerate()
        .find(|(_, line)| line.starts_with(&format!("{case}\t")))
        .unwrap_or_else(|| panic!("{case} manifest row exists"));

    assert_eq!(
        score_class_for_case(&manifest_lines, line_idx, case),
        Some("bounded"),
        "R100-001A must not promote default Tailwind backdrop overlay to exact"
    );
}

#[test]
fn real_illustrator_reference_fixture_is_committed() {
    let reference = repo_path("tests/visual_diff/fixtures/illustrator/ui-assets-page1.png");
    let image = image::open(reference)
        .expect("Illustrator reference fixture should be committed")
        .to_rgba8();
    assert_eq!([image.width(), image.height()], [5102, 3608]);
}

#[test]
fn real_gradient_mesh_fixture_is_committed() {
    let reference = repo_path("tests/visual_diff/fixtures/illustrator/gradient-mesh-quad.png");
    let image = image::open(reference)
        .expect("Illustrator gradient mesh reference fixture should be committed")
        .to_rgba8();
    assert_eq!([image.width(), image.height()], [2, 2]);

    let actual = repo_path("tests/visual_diff/fixtures/egui/gradient-mesh-quad.png");
    let egui_image = image::open(actual)
        .expect("egui gradient mesh fixture should be committed")
        .to_rgba8();
    assert_eq!([egui_image.width(), egui_image.height()], [2, 2]);
}

#[test]
fn real_gradient_mesh_fixture_has_vector_source() {
    let vector_source = repo_path("tests/visual_diff/fixtures/egui/gradient-mesh-quad.svg");
    assert!(
        vector_source.exists(),
        "egui gradient mesh fixture should keep its vector source beside the rendered PNG"
    );
}

#[test]
fn real_egui_fixture_is_committed_with_vector_source() {
    let vector_source = repo_path("tests/visual_diff/fixtures/egui/ui-assets-page1.svg");
    assert!(
        vector_source.exists(),
        "egui visual fixture should keep its vector source beside the rendered PNG"
    );

    let actual = repo_path("tests/visual_diff/fixtures/egui/ui-assets-page1.png");
    let image = image::open(actual)
        .expect("egui visual fixture should be committed")
        .to_rgba8();
    assert_eq!([image.width(), image.height()], [5102, 3608]);
}
