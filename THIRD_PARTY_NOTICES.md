# Third-Party Notices

This project is licensed under **AGPL-3.0-or-later**.

## Imported or Derived Code

### SquirrelDisk
- Upstream project: `squirreldisk`
- Upstream path: `F:\repos\oss\squirreldisk`
- Upstream license: `AGPL-3.0` (compatible with this repository's `AGPL-3.0-or-later`)
- Imported/derived areas in this repository:
  - `crates/core/src/scan.rs` (pseudo/system mount filtering and progress-compatible scan backend behavior inspired by `src-tauri/src/scan.rs`)
  - `crates/core/src/model.rs` (scan backend and metrics fields aligned with scanner progress patterns)
- Reuse scope:
  - scanner/mount-filter/progress patterns only
  - no destructive file-management UX/actions were reused

## Additional Dependencies

Third-party Rust crates and their licenses are declared in `Cargo.lock` and crate metadata.

## Compliance Process

- All imported AGPL-derived files must be recorded in `provenance/imported_code.json`.
- CI runs `scripts/check_compliance.py` to verify licensing and provenance metadata.
