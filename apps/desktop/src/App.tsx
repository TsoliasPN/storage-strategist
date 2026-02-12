import { useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import {
  cancelScan,
  doctor,
  generateRecommendations,
  getScanSession,
  loadReport,
  pollScanEvents,
  startScan,
} from "./api";
import type {
  DoctorInfo,
  Report,
  ScanProgressEvent,
  ScanRequest,
  ScanSessionSnapshot,
} from "./types";

type Screen = "setup" | "scanning" | "results" | "doctor";
type ResultTab =
  | "disks"
  | "usage"
  | "categories"
  | "duplicates"
  | "recommendations"
  | "rule-trace";

const DEFAULT_OUTPUT = "storage-strategist-report.json";

function App() {
  const [screen, setScreen] = useState<Screen>("setup");
  const [tab, setTab] = useState<ResultTab>("recommendations");

  const [paths, setPaths] = useState<string[]>([]);
  const [pathInput, setPathInput] = useState("");
  const [excludeInput, setExcludeInput] = useState("");
  const [excludes, setExcludes] = useState<string[]>([]);
  const [output, setOutput] = useState(DEFAULT_OUTPUT);
  const [maxDepth, setMaxDepth] = useState<number | undefined>(undefined);
  const [backend, setBackend] = useState<"native" | "pdu_library">("native");
  const [dedupe, setDedupe] = useState(true);

  const [scanId, setScanId] = useState<string | null>(null);
  const [session, setSession] = useState<ScanSessionSnapshot | null>(null);
  const [events, setEvents] = useState<ScanProgressEvent[]>([]);
  const [report, setReport] = useState<Report | null>(null);
  const [doctorInfo, setDoctorInfo] = useState<DoctorInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (screen !== "scanning" || !scanId) {
      return;
    }

    let isActive = true;
    let fromSeq = 0;

    const poll = async () => {
      try {
        const [snapshot, nextEvents] = await Promise.all([
          getScanSession(scanId),
          pollScanEvents(scanId, fromSeq),
        ]);

        if (!isActive) {
          return;
        }

        setSession(snapshot);

        if (nextEvents.length > 0) {
          fromSeq = nextEvents[nextEvents.length - 1].seq;
          setEvents((prev) => [...prev, ...nextEvents]);
        }

        if (snapshot.status === "completed") {
          const reportPath = snapshot.report_path ?? output;
          const loaded = await loadReport(reportPath);
          const bundle = await generateRecommendations(loaded);

          if (!isActive) {
            return;
          }

          setReport({ ...loaded, recommendations: bundle.recommendations });
          setTab("recommendations");
          setScreen("results");
        }

        if (snapshot.status === "failed") {
          setError(snapshot.error ?? "scan failed");
          setScreen("setup");
        }

        if (snapshot.status === "cancelled") {
          setError("Scan cancelled.");
          setScreen("setup");
        }
      } catch (pollError) {
        setError(String(pollError));
        setScreen("setup");
      }
    };

    const timer = window.setInterval(poll, 500);
    void poll();

    return () => {
      isActive = false;
      window.clearInterval(timer);
    };
  }, [scanId, screen, output]);

  const latestEvent = useMemo(
    () => (events.length > 0 ? events[events.length - 1] : null),
    [events]
  );

  const start = async () => {
    setError(null);
    if (paths.length === 0) {
      setError("Select at least one path before starting a scan.");
      return;
    }

    const request: ScanRequest = {
      paths,
      output: output.trim() || undefined,
      max_depth: maxDepth,
      excludes,
      dedupe,
      dedupe_min_size: 1_048_576,
      backend,
      progress: true,
      min_ratio: undefined,
      emit_progress_events: true,
      progress_interval_ms: 250,
    };

    try {
      setEvents([]);
      setSession(null);
      setReport(null);
      const id = await startScan(request);
      setScanId(id);
      setScreen("scanning");
    } catch (startError) {
      setError(String(startError));
    }
  };

  const cancel = async () => {
    if (!scanId) {
      return;
    }
    await cancelScan(scanId);
  };

  const loadDoctor = async () => {
    setError(null);
    try {
      const info = await doctor();
      setDoctorInfo(info);
      setScreen("doctor");
    } catch (doctorError) {
      setError(String(doctorError));
    }
  };

  const addPath = (value: string) => {
    const normalized = value.trim();
    if (!normalized || paths.includes(normalized)) {
      return;
    }
    setPaths((prev) => [...prev, normalized]);
    setPathInput("");
  };

  const browsePaths = async () => {
    try {
      const selection = await open({ directory: true, multiple: true });
      if (!selection) {
        return;
      }
      if (Array.isArray(selection)) {
        selection.forEach((entry) => addPath(entry));
      } else {
        addPath(selection);
      }
    } catch {
      setError(
        "Directory picker unavailable in this environment. Enter paths manually."
      );
    }
  };

  return (
    <main className="app-shell">
      <header className="header">
        <div>
          <h1>Storage Strategist Desktop</h1>
          <p className="sub">Read-only review UI. No delete/move/rename actions are available.</p>
        </div>
        <nav className="header-actions">
          <button onClick={() => setScreen("setup")}>Setup</button>
          <button onClick={loadDoctor}>Doctor</button>
        </nav>
      </header>

      {error ? <p className="error">{error}</p> : null}

      {screen === "setup" ? (
        <section className="panel">
          <h2>Guided Path Selection</h2>
          <p>Select local paths first. Cloud/network/virtual targets are excluded from placement recommendations.</p>

          <div className="row">
            <input
              value={pathInput}
              onChange={(event) => setPathInput(event.target.value)}
              placeholder="Add path (e.g. D:\\Games)"
            />
            <button onClick={() => addPath(pathInput)}>Add Path</button>
            <button onClick={browsePaths}>Browse...</button>
          </div>

          <ul className="list">
            {paths.map((path) => (
              <li key={path}>
                <span>{path}</span>
                <button onClick={() => setPaths(paths.filter((item) => item !== path))}>Remove</button>
              </li>
            ))}
            {paths.length === 0 ? <li className="muted">No paths selected.</li> : null}
          </ul>

          <div className="row two-col">
            <label>
              Output report path
              <input
                value={output}
                onChange={(event) => setOutput(event.target.value)}
                placeholder={DEFAULT_OUTPUT}
              />
            </label>
            <label>
              Max depth (optional)
              <input
                type="number"
                min={1}
                value={maxDepth ?? ""}
                onChange={(event) => {
                  const value = event.target.value;
                  setMaxDepth(value ? Number(value) : undefined);
                }}
              />
            </label>
          </div>

          <div className="row two-col">
            <label>
              Exclude pattern
              <input
                value={excludeInput}
                onChange={(event) => setExcludeInput(event.target.value)}
                placeholder="node_modules or **/*.tmp"
              />
            </label>
            <label>
              Backend
              <select
                value={backend}
                onChange={(event) => setBackend(event.target.value as "native" | "pdu_library")}
              >
                <option value="native">native</option>
                <option value="pdu_library">pdu_library</option>
              </select>
            </label>
          </div>

          <div className="row">
            <button onClick={() => {
              const next = excludeInput.trim();
              if (!next || excludes.includes(next)) {
                return;
              }
              setExcludes((prev) => [...prev, next]);
              setExcludeInput("");
            }}>
              Add Exclude
            </button>
            <label className="inline-toggle">
              <input type="checkbox" checked={dedupe} onChange={(event) => setDedupe(event.target.checked)} />
              Enable dedupe scan
            </label>
          </div>

          <ul className="list compact">
            {excludes.map((item) => (
              <li key={item}>
                <span>{item}</span>
                <button onClick={() => setExcludes(excludes.filter((x) => x !== item))}>Remove</button>
              </li>
            ))}
          </ul>

          <div className="row end">
            <button className="primary" onClick={start} disabled={paths.length === 0}>
              Start Read-Only Scan
            </button>
          </div>
        </section>
      ) : null}

      {screen === "scanning" ? (
        <section className="panel">
          <h2>Scanning</h2>
          <p>
            Scan ID: <code>{scanId}</code>
          </p>
          <p>
            Status: <strong>{session?.status ?? "running"}</strong>
          </p>
          <p>
            Phase: <strong>{latestEvent?.phase ?? "starting"}</strong>
          </p>
          <p>
            Files: {latestEvent?.scanned_files ?? 0} | Bytes: {latestEvent?.scanned_bytes ?? 0} |
            Errors: {latestEvent?.errors ?? 0}
          </p>
          <div className="progress-log">
            {events.slice(-12).map((event) => (
              <p key={event.seq}>
                #{event.seq} {event.phase} {event.current_path ? `(${event.current_path})` : ""}
              </p>
            ))}
          </div>
          <div className="row end">
            <button onClick={cancel}>Cancel</button>
          </div>
        </section>
      ) : null}

      {screen === "results" && report ? (
        <section className="panel">
          <h2>Results Workbench</h2>
          <p>
            Report {report.report_version} generated at {report.generated_at}
          </p>
          <div className="tabs">
            {(["disks", "usage", "categories", "duplicates", "recommendations", "rule-trace"] as ResultTab[]).map(
              (item) => (
                <button
                  key={item}
                  className={tab === item ? "active" : ""}
                  onClick={() => setTab(item)}
                >
                  {item}
                </button>
              )
            )}
          </div>

          {tab === "disks" ? (
            <table>
              <thead>
                <tr>
                  <th>Disk</th>
                  <th>Mount</th>
                  <th>Role</th>
                  <th>Locality</th>
                  <th>Perf</th>
                  <th>OS</th>
                  <th>Eligible</th>
                </tr>
              </thead>
              <tbody>
                {report.disks.map((disk) => (
                  <tr key={disk.mount_point}>
                    <td>{disk.name}</td>
                    <td>{disk.mount_point}</td>
                    <td>{disk.role_hint?.role ?? "unknown"}</td>
                    <td>{disk.locality_class}</td>
                    <td>{disk.performance_class}</td>
                    <td>{disk.is_os_drive ? "yes" : "no"}</td>
                    <td>{disk.eligible_for_local_target ? "yes" : "no"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : null}

          {tab === "usage" ? (
            <div className="scroll-block">
              {(report.paths ?? []).map((path) => (
                <article key={path.root_path} className="card">
                  <h3>{path.root_path}</h3>
                  <p>
                    files {path.file_count} | directories {path.directory_count} | bytes {path.total_size_bytes}
                  </p>
                </article>
              ))}
            </div>
          ) : null}

          {tab === "categories" ? (
            <div className="scroll-block">
              {(report.categories ?? []).map((category, index) => (
                <article key={`${category.target}-${index}`} className="card">
                  <h3>{category.category}</h3>
                  <p>{category.target}</p>
                  <p>confidence {(category.confidence * 100).toFixed(1)}%</p>
                  <p>{category.rationale}</p>
                </article>
              ))}
            </div>
          ) : null}

          {tab === "duplicates" ? (
            <div className="scroll-block">
              {(report.duplicates ?? []).map((dup) => (
                <article key={`${dup.hash}-${dup.size_bytes}`} className="card">
                  <h3>{dup.intent?.label ?? "duplicate"}</h3>
                  <p>
                    size {dup.size_bytes} | files {dup.files.length} | wasted {dup.total_wasted_bytes}
                  </p>
                  <p>{dup.intent?.rationale}</p>
                </article>
              ))}
            </div>
          ) : null}

          {tab === "recommendations" ? (
            <div className="scroll-block">
              {report.recommendations.map((recommendation) => (
                <article key={recommendation.id} className="card">
                  <h3>{recommendation.title}</h3>
                  <p>{recommendation.rationale}</p>
                  <p>
                    risk {recommendation.risk_level} | confidence {(recommendation.confidence * 100).toFixed(1)}%
                  </p>
                  <p>target {recommendation.target_mount ?? "none"}</p>
                  <p>policy applied: {recommendation.policy_rules_applied.join(", ") || "none"}</p>
                  <p>policy blocked: {recommendation.policy_rules_blocked.join(", ") || "none"}</p>
                </article>
              ))}
            </div>
          ) : null}

          {tab === "rule-trace" ? (
            <div className="scroll-block">
              {(report.rule_traces ?? []).map((trace, index) => (
                <article key={`${trace.rule_id}-${index}`} className="card">
                  <h3>{trace.rule_id}</h3>
                  <p>{trace.status}</p>
                  <p>{trace.detail}</p>
                </article>
              ))}
            </div>
          ) : null}
        </section>
      ) : null}

      {screen === "doctor" ? (
        <section className="panel">
          <h2>Doctor</h2>
          {!doctorInfo ? <p>Loading doctor data...</p> : null}
          {doctorInfo ? (
            <>
              <p>
                OS {doctorInfo.os} ({doctorInfo.arch}) | read-only {doctorInfo.read_only_mode ? "yes" : "no"}
              </p>
              <p>Detected disks: {doctorInfo.disks.length}</p>
              <ul className="list compact">
                {doctorInfo.disks.map((disk) => (
                  <li key={disk.mount_point}>
                    {disk.mount_point} {disk.name} | locality {disk.locality_class} | role {disk.role_hint.role}
                  </li>
                ))}
              </ul>
            </>
          ) : null}
        </section>
      ) : null}
    </main>
  );
}

export default App;
