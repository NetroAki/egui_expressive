# Versioning Policy

`egui_expressive` uses Semantic Versioning once public releases are published.
The current working tree is pre-1.0 and should be treated as a release candidate
until the repository owner explicitly approves a push, tag, or registry publish.

## Pre-1.0 expectations

- Public APIs may still change before `1.0`.
- Compatibility aliases may be kept when removal would create avoidable churn.
- Platform support labels must remain tied to recorded runtime artifacts.
- Publishing a crate version requires a clean package dry-run and explicit owner
  approval.

## Version bump guidance

| Change | Suggested bump before 1.0 |
| --- | --- |
| Additive widgets, helpers, docs, examples | patch |
| Behavior changes with migration notes | patch or minor candidate |
| Breaking API rename/removal | minor candidate with migration notes |
| Broad support-label expansion | only with matching runtime artifacts |

This policy does not authorize release tags, registry publishing, or support
upgrades by itself.
