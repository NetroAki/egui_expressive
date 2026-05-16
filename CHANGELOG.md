# Changelog

All notable user-facing changes to `egui_expressive` should be recorded here.

This project follows the versioning policy in `docs/versioning-policy.md`. Until the first public release, the changelog groups changes by staged readiness rather than published crate versions.

## Unreleased

### Added

- Release-readiness validation for Stage 9: interaction smoke tests, deterministic performance-smoke tests, expanded CI gates, and release documentation.
- Documentation index, migration guide, versioning policy, and release checklist.

### Changed

- Data-widget advanced column interactions are documented as explicit unsupported release scope unless a later hardening stage opts into them.

### Removed

- Historical orphan split files under `src/{codegen,draw,scene}/legacy_parts`, inactive scene split leftovers, and orphan `src/bin/ai_parser_parts` files after reference audit.
