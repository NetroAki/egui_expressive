# Migration Guide

This guide tracks compatibility notes for the current pre-1.0 release candidate.
APIs may still move before `1.0`, but avoidable churn should be documented here.

## Current guidance

- Prefer the public modules re-exported from `egui_expressive` rather than deep
  private module paths.
- Treat platform support claims as evidence-scoped. Linux has the strongest
  current runtime evidence; Windows, macOS, and iOS remain planned until their own
  runtime artifacts exist.
- Optional GPU and design-tool paths should be gated behind their documented
  features and runtime checks.
- Registry dependency examples should be used only after a crate version is
  published. Until then, use a Git or path dependency for local validation.

## Compatibility aliases

Compatibility aliases may remain during the pre-1.0 period when removing them
would create unnecessary downstream churn. Prefer new names in fresh code and keep
old names only as documented bridges.
