#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use storage_strategist_core::{
    DiagnosticsBundle, DoctorInfo, RecommendationBundle, Report, ScanProgressEvent, ScenarioPlan,
};
use storage_strategist_service::{
    cancel_scan as service_cancel_scan, doctor as service_doctor, export_diagnostics_bundle as service_export_diagnostics_bundle,
    generate_recommendations_from_report, get_scan_session as service_get_scan_session,
    load_report as service_load_report, poll_scan_events as service_poll_scan_events, start_scan as service_start_scan,
    plan_scenarios_from_report as service_plan_scenarios_from_report, CancelScanResponse, ScanRequest,
    ScanSessionSnapshot,
};

#[tauri::command]
fn start_scan(request: ScanRequest) -> Result<String, String> {
    service_start_scan(request).map_err(|err| err.to_string())
}

#[tauri::command]
fn poll_scan_events(scan_id: String, from_seq: u64) -> Result<Vec<ScanProgressEvent>, String> {
    service_poll_scan_events(&scan_id, from_seq).map_err(|err| err.to_string())
}

#[tauri::command]
fn get_scan_session(scan_id: String) -> Result<ScanSessionSnapshot, String> {
    service_get_scan_session(&scan_id).map_err(|err| err.to_string())
}

#[tauri::command]
fn cancel_scan(scan_id: String) -> Result<CancelScanResponse, String> {
    service_cancel_scan(&scan_id).map_err(|err| err.to_string())
}

#[tauri::command]
fn load_report(path: String) -> Result<Report, String> {
    service_load_report(path).map_err(|err| err.to_string())
}

#[tauri::command]
fn generate_recommendations(report: Report) -> Result<RecommendationBundle, String> {
    Ok(generate_recommendations_from_report(&report))
}

#[tauri::command]
fn plan_scenarios(report: Report) -> Result<ScenarioPlan, String> {
    Ok(service_plan_scenarios_from_report(&report))
}

#[tauri::command]
fn export_diagnostics_bundle(
    report: Report,
    output_path: String,
    source_report_path: Option<String>,
) -> Result<DiagnosticsBundle, String> {
    service_export_diagnostics_bundle(
        &report,
        output_path,
        source_report_path.map(PathBuf::from),
    )
    .map_err(|err| err.to_string())
}

#[tauri::command]
fn doctor() -> DoctorInfo {
    service_doctor()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_scan,
            poll_scan_events,
            get_scan_session,
            cancel_scan,
            load_report,
            generate_recommendations,
            plan_scenarios,
            export_diagnostics_bundle,
            doctor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
