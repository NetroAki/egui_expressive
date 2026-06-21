# Changelog

This project follows the versioning policy in `docs/versioning-policy.md`.

## Unreleased

### Added

- Linux virtual-desktop smoke scripts for X11/Xvfb/Openbox and Wayland/Sway.
- Bounded Web and Android showcase harnesses and platform-smoke docs.
- Additional claim-boundary tests for public docs and platform support wording.
- Illustrator plugin integrity/signing checks and stricter parity sidecar behavior.

### Changed

- Public docs now describe platform support as evidence-scoped and pre-1.0.
- README install guidance points to the Git repository until a registry release is
  explicitly published.
- Release checklist now distinguishes release-candidate validation from registry
  publish approval.

### Fixed

- Hardened non-finite coordinate handling in layout/codegen inference.
- Tightened Illustrator plugin parser discovery and unsigned package behavior.
- Improved Linux runtime smoke coverage for normal/high-DPI render paths.

### Not yet claimed

- Windows, macOS, and iOS runtime support require their own runtime artifacts.
- Full design-tool parity and every platform/renderer path remain outside the
  bounded release-candidate claims.
