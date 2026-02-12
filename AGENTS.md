# AGENTS

## Project Guidance

- v1 is strictly read-only for user data:
  - Do not delete, move, rename, or modify scanned user files.
  - Emit recommendations only.
- Project license is `AGPL-3.0-or-later`.
- Selective code reuse from AGPL projects is allowed, but every reused/derived file must be
  recorded in `provenance/imported_code.json` and reflected in `THIRD_PARTY_NOTICES.md`.
- Prefer small, reviewable PR-like commits and incremental changes.
- Keep `Report` schema backward compatible within major version:
  - Additive fields are preferred.
  - Do not repurpose existing field semantics.
  - Bump `report_version` when breaking schema changes are required.
- Handle permission or IO failures as warnings; continue best-effort scanning.

## Local Quality Checks

Run before opening a PR:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python scripts/check_compliance.py
```
