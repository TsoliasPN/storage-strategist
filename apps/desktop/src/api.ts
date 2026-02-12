import { invoke } from "@tauri-apps/api/core";

import type {
  DiagnosticsBundle,
  DoctorInfo,
  RecommendationBundle,
  Report,
  ScenarioPlan,
  ScanProgressEvent,
  ScanRequest,
  ScanSessionSnapshot,
} from "./types";

export async function startScan(request: ScanRequest): Promise<string> {
  return invoke<string>("start_scan", { request });
}

export async function pollScanEvents(
  scanId: string,
  fromSeq: number
): Promise<ScanProgressEvent[]> {
  return invoke<ScanProgressEvent[]>("poll_scan_events", {
    scanId,
    fromSeq,
  });
}

export async function getScanSession(
  scanId: string
): Promise<ScanSessionSnapshot> {
  return invoke<ScanSessionSnapshot>("get_scan_session", { scanId });
}

export async function cancelScan(scanId: string): Promise<void> {
  await invoke("cancel_scan", { scanId });
}

export async function loadReport(path: string): Promise<Report> {
  return invoke<Report>("load_report", { path });
}

export async function generateRecommendations(
  report: Report
): Promise<RecommendationBundle> {
  return invoke<RecommendationBundle>("generate_recommendations", { report });
}

export async function planScenarios(report: Report): Promise<ScenarioPlan> {
  return invoke<ScenarioPlan>("plan_scenarios", { report });
}

export async function exportDiagnosticsBundle(
  report: Report,
  outputPath: string,
  sourceReportPath?: string
): Promise<DiagnosticsBundle> {
  return invoke<DiagnosticsBundle>("export_diagnostics_bundle", {
    report,
    outputPath,
    sourceReportPath,
  });
}

export async function doctor(): Promise<DoctorInfo> {
  return invoke<DoctorInfo>("doctor");
}
