# Code Import Policy

This repository allows selective code reuse from AGPL-compatible projects, with strict controls.

## Mandatory Rules

1. Preserve read-only product guarantees (no delete/move/rename operations in v1).
2. Reuse only what is needed; prefer adapter boundaries over bulk imports.
3. Every imported or materially derived file must be listed in `provenance/imported_code.json`.
4. Preserve upstream notices and mention exact source paths/commit references where available.
5. Keep report schema changes additive by default; document any breaking change.
6. Add/adjust tests for safety policy invariants and behavioral regressions.

## Import Checklist

- Confirm upstream license compatibility.
- Record provenance entry (source project/path/license/notes/date).
- Add/update `THIRD_PARTY_NOTICES.md`.
- Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- Run `python scripts/check_compliance.py`.

## Out of Scope for Reuse

- Destructive workflows (delete/move/cleanup execution).
- UI-specific code not required for this CLI core.
- Network-coupled runtime behavior.
