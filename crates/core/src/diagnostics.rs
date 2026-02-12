use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::doctor::collect_doctor_info;
use crate::doctor::DoctorInfo;
use crate::model::Report;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsBundle {
    pub generated_at: String,
    pub source_report_path: Option<String>,
    pub report: Report,
    pub doctor: DoctorInfo,
    pub environment: DiagnosticsEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsEnvironment {
    pub os: String,
    pub arch: String,
    pub current_dir: Option<String>,
    pub os_mount: Option<String>,
    pub read_only_mode: bool,
    pub app_version: String,
}

pub fn build_diagnostics_bundle(
    report: &Report,
    source_report_path: Option<&Path>,
) -> DiagnosticsBundle {
    let doctor = collect_doctor_info();
    DiagnosticsBundle {
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        source_report_path: source_report_path.map(|path| path.to_string_lossy().to_string()),
        report: report.clone(),
        environment: DiagnosticsEnvironment {
            os: doctor.os.clone(),
            arch: doctor.arch.clone(),
            current_dir: doctor.current_dir.clone(),
            os_mount: doctor.os_mount.clone(),
            read_only_mode: doctor.read_only_mode,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        },
        doctor,
    }
}

pub fn write_diagnostics_bundle(
    bundle: &DiagnosticsBundle,
    output_path: impl AsRef<Path>,
) -> Result<()> {
    let path = output_path.as_ref();
    let payload =
        serde_json::to_string_pretty(bundle).context("failed to serialize diagnostics bundle")?;
    fs::write(path, payload)
        .with_context(|| format!("failed to write diagnostics bundle to {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::build_diagnostics_bundle;
    use crate::model::Report;

    #[test]
    fn bundle_embeds_report_and_source_path() {
        let report: Report =
            serde_json::from_str(include_str!("../../../fixtures/sample-report.json"))
                .expect("fixture report parses");
        let bundle = build_diagnostics_bundle(&report, Some(Path::new("sample-report.json")));

        assert_eq!(bundle.report.scan_id, report.scan_id);
        assert_eq!(
            bundle.source_report_path,
            Some("sample-report.json".to_string())
        );
        assert!(bundle.environment.read_only_mode);
    }
}
