# Desktop UI (Tauri + React)

This app is the read-only review UI track for `storage-strategist`.

## Scope

- Guided path selection before scan start.
- Live scan progress (events + session polling).
- Results workbench:
  - Disks
  - Usage
  - Categories
  - Duplicates
  - Recommendations
  - Rule Trace
- Doctor diagnostics panel.
- No destructive operations.

## Local run

```bash
cd apps/desktop
npm install
npm run tauri dev
```

## Notes

- Uses `crates/service` for scan/recommend/report APIs.
- UI is intentionally advisory and read-only in all current phases.
