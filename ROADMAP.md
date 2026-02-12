# Roadmap Update Plan: PDU-Informed Core + Tauri Review UI (Read-Only)

## Summary

This roadmap track updates `storage-strategist` with concrete patterns from `F:\repos\oss\parallel-disk-usage` and adds a full desktop UI path.

Chosen defaults:
- UI shell: **Tauri + React**
- First UI release: **read-only review UI**
- Scan UX: **guided path selection first**

Key external findings from `parallel-disk-usage`:
- Mature parallel traversal and event/reporting abstractions.
- Strict JSON/schema-oriented design and compatibility handling.
- Strong quality baseline (about **98 tests**) and benchmark workflows.
- Apache-2.0 license compatible with this AGPL project.

## 90-Day Focus (UI + Core)

### P0 (Weeks 1-4)

1. [x] **PDU library backend integration (real library usage)**
   - Added `ScanBackendKind::PduLibrary`.
   - Uses `parallel-disk-usage` tree summarization APIs (`FsTreeBuilder`) with native detailed traversal fallback.
2. [x] **Scan progress/event system**
   - Added structured `ScanProgressEvent`, `ScanPhase`, `ScanProgressSummary`, and `scan_id`.
   - Added service-layer event polling with session IDs.
3. [x] **Recommendation safety hardening**
   - Added role-aware policy checks blocking active-placement recommendations into media/archive/backup targets.
4. [x] **Benchmark regression gate**
   - Added `.github/workflows/bench.yml`.
   - Added `scripts/check_benchmark_regression.py` (initial threshold 15%).
5. [x] **UI foundation (Tauri + React)**
   - Added `apps/desktop` scaffold with setup, scanning, results, and doctor screens.
6. [x] **Quality uplift (first tranche)**
   - Expanded tests around scan excludes/progress, role inference, and policy safety.

### P1 (Weeks 5-8)

7. [ ] **UI results workbench depth**
   - richer recommendation inspector and policy trace views
   - duplicate/group drill-down interactions
8. [ ] **Rule calibration loop**
   - fixture expansion + KPI thresholds in CI (`precision@3`, contradiction rate, unsafe target count)
9. [ ] **Device intelligence providers**
   - OS-specific enrichers for model/interface/rotational confidence

### P2 (Weeks 9-12)

10. [ ] **Incremental scan cache**
11. [ ] **Scenario planner (read-only what-if simulation)**
12. [ ] **Release hardening**
    - installer/signing, diagnostics bundle, desktop packaging

## Known Limitations

- `pdu_library` currently powers tree/summary integration and backend parity checks; full parity/perf validation still required before making it default.
- UI scaffold is intentionally read-only and focuses on review/explainability.
- Hardware metadata remains best-effort on platform APIs that do not expose low-level fields.

## Important Public API / Type Changes

### Core (`crates/core`)

- `ScanOptions` additions:
  - `scan_id`, `emit_progress_events`, `progress_interval_ms`, `backend: ScanBackendKind`
- New event model:
  - `ScanProgressEvent`, `ScanPhase`, `ScanProgressSummary`
- `DiskInfo` additions:
  - `role_hint`, `target_role_eligibility`
- New role model:
  - `DiskRole`, `DiskRoleHint`
- `Report` additions:
  - `scan_id`, `scan_progress_summary`, `backend_parity`
- `Recommendation` additions:
  - `policy_rules_applied`, `policy_rules_blocked`

### Service layer (`crates/service`)

Implemented APIs:
- `start_scan(request) -> scan_id`
- `poll_scan_events(scan_id, from_seq) -> Vec<ScanProgressEvent>`
- `get_scan_session(scan_id) -> ScanSessionSnapshot`
- `cancel_scan(scan_id) -> CancelScanResponse`
- `load_report(path) -> Report`
- `generate_recommendations_from_report(report) -> RecommendationBundle`
- `doctor() -> DoctorInfo`

### Desktop UI (`apps/desktop`)

- Tauri command bridge wired to `crates/service`.
- React state machine:
  - `setup -> scanning -> results -> doctor`

## UI Plan (Decision Complete)

Information architecture:
1. Setup screen (guided path selection first)
2. Scanning screen (phase/events/counters + cancel)
3. Results screen tabs (`Disks`, `Usage`, `Categories`, `Duplicates`, `Recommendations`, `Rule Trace`)
4. Doctor screen (diagnostics + eligibility context)

UX constraints:
- No destructive controls.
- Advisory recommendation wording only.
- Cloud/virtual/network/OS destination constraints preserved in policy/UI messaging.

## Implementation Sequence (Current)

1. [x] Add PDU dependency and implement `PduLibraryBackend`
2. [x] Add backend parity function (`compare_backends`)
3. [x] Add progress event bus + session IDs
4. [x] Add `DiskRole` inference module
5. [x] Add role-aware recommendation policy constraints
6. [x] Add benchmark CI regression gate (15% initial threshold)
7. [x] Add `crates/service`
8. [x] Scaffold Tauri + React app and connect setup/scan flow
9. [ ] Expand UI recommendation inspector and trace drilldowns
10. [ ] Add UI e2e smoke tests and packaging jobs

## Test Scenarios (Implemented + Planned)

Implemented:
- disk role classification heuristics
- policy blocking for cloud targets
- policy blocking for active-placement into media-role targets
- scan exclude matching (glob + substring fallback)
- fixture-based recommendation evaluation

Planned next:
- backend parity fixture assertions (`native` vs `pdu_library`)
- permission continuation stress fixtures
- UI smoke tests for setup/scanning/results rendering

## Assumptions and Defaults

- Strictly read-only behavior across CLI/service/UI.
- Guided path selection is default UI entrypoint.
- `native` remains default scanner until parity/perf criteria are tightened.
- Performance regression threshold starts at 15% and can be tightened later.
- First UI release prioritizes trust/explainability over automation.
