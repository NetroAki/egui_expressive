use std::{fs, path::Path};

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative)).unwrap_or_else(|err| {
        panic!("failed to read {relative}: {err}");
    })
}

#[test]
fn illustrator_plugin_package_version_tracks_crate_version() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let version = env!("CARGO_PKG_VERSION");
    let linux_zxp = format!("egui_expressive_export-{version}-linux.zxp");
    let win_zxp = format!("egui_expressive_export-{version}-win32.zxp");

    let manifest = read(root, "illustrator-plugin/manifest.json");
    assert!(
        manifest.contains(&format!("\"version\": \"{version}\"")),
        "illustrator UXP-reference manifest version must track Cargo package version {version}"
    );
    assert!(
        manifest.contains("\"domains\": []"),
        "reference UXP manifest must keep network domains empty"
    );
    assert!(
        !manifest.contains("readAndWrite"),
        "reference UXP manifest must not request broad clipboard read/write permission"
    );

    let csxs_manifest = read(root, "illustrator-plugin/CSXS/manifest.xml");
    assert!(
        csxs_manifest.contains(&format!("ExtensionBundleVersion=\"{version}\"")),
        "CEP ExtensionBundleVersion must track Cargo package version {version}"
    );
    assert!(
        csxs_manifest.contains(&format!(
            "<Extension Id=\"com.egui-expressive.illustrator-exporter.panel\" Version=\"{version}\" />"
        )),
        "CEP panel Version must track Cargo package version {version}"
    );

    let build_sh = read(root, "illustrator-plugin/installer/build_zxp.sh");
    assert!(
        build_sh.contains(&format!("VERSION=\"{version}\"")),
        "Linux ZXP builder VERSION must track Cargo package version {version}"
    );
    assert!(
        build_sh.contains("EGUI_EXPRESSIVE_ALLOW_UNSIGNED_ZXP")
            && build_sh.contains("Refusing to create an unsigned ZXP")
            && build_sh.contains("internal smoke diagnostics"),
        "Linux ZXP builder must fail closed on unsigned packages unless explicitly opted in for internal smoke diagnostics"
    );
    assert!(
        build_sh.contains("ai-parser-integrity.json")
            && build_sh.contains("compute_sha256")
            && build_sh.contains("bin/$platform/$binary"),
        "Linux ZXP builder must write the bundled ai-parser SHA-256 integrity manifest required by the CEP plugin"
    );
    assert!(
        build_sh.contains("command -v zxp-sign-cmd")
            && build_sh.contains("not a trustworthy signer probe")
            && !build_sh.contains("npx zxp-sign-cmd"),
        "Linux ZXP builder must not treat npx alone as a production signer"
    );

    let build_bat = read(root, "illustrator-plugin/installer/build_zxp.bat");
    assert!(
        build_bat.contains(&format!("set \"VERSION={version}\"")),
        "Windows ZXP builder VERSION must track Cargo package version {version}"
    );
    assert!(
        build_bat.contains("ai-parser-integrity.json")
            && build_bat.contains("certutil -hashfile")
            && build_bat.contains("bin/win32/ai-parser.exe"),
        "Windows ZXP builder must write the bundled ai-parser SHA-256 integrity manifest required by the CEP plugin"
    );

    let nsi = read(
        root,
        "illustrator-plugin/installer/egui_expressive_plugin.nsi",
    );
    assert!(
        nsi.contains(&format!("!define PRODUCT_VERSION \"{version}\"")),
        "NSIS installer PRODUCT_VERSION must track Cargo package version {version}"
    );
    assert!(
        nsi.contains("ai-parser-integrity.json")
            && nsi.contains("Delete \"$INSTDIR\\ai-parser-integrity.json\""),
        "NSIS installer/uninstaller must account for the ai-parser integrity manifest"
    );

    let install_bat = read(root, "illustrator-plugin/install.bat");
    assert!(
        install_bat.contains(&win_zxp),
        "Windows installer helper must look for the Cargo-versioned package {win_zxp}"
    );
    assert!(
        install_bat.contains("EGUI_EXPRESSIVE_ENABLE_CEP_DEBUG")
            && install_bat.contains(":enable_debug_modes"),
        "Windows installer helper must keep CEP debug-mode writes behind an explicit opt-in"
    );

    let install_zxp_bat = read(root, "illustrator-plugin/installer/install_zxp.bat");
    assert!(
        install_zxp_bat.contains("EGUI_EXPRESSIVE_ENABLE_CEP_DEBUG")
            && install_zxp_bat.contains(":enable_debug_modes"),
        "Legacy Windows ZXP installer helper must keep CEP debug-mode writes behind an explicit opt-in"
    );

    let readme = read(root, "illustrator-plugin/README.md");
    assert!(
        readme.contains(&linux_zxp) && readme.contains(&win_zxp),
        "Illustrator plugin README must document Cargo-versioned package names {linux_zxp} and {win_zxp}"
    );
}
