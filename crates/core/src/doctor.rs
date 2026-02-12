use std::env;

use serde::{Deserialize, Serialize};
use sysinfo::{DiskKind as SysDiskKind, Disks};

use crate::device::{detect_os_mount, enrich_disks, DiskProbe};
use crate::model::{DiskInfo, DiskKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorInfo {
    pub os: String,
    pub arch: String,
    pub current_dir: Option<String>,
    pub os_mount: Option<String>,
    pub read_only_mode: bool,
    pub disks: Vec<DiskInfo>,
    pub notes: Vec<String>,
}

pub fn collect_doctor_info() -> DoctorInfo {
    let current_dir = env::current_dir()
        .ok()
        .map(|path| path.to_string_lossy().to_string());
    let os_mount = detect_os_mount();

    let disks = enumerate_disks();
    let mut notes = vec![
        "v1 operates in read-only mode; no file mutations are performed.".to_string(),
        "Network access is not used by the runtime scanner.".to_string(),
        "Cloud/network/virtual mounts are excluded as local placement targets.".to_string(),
    ];
    if disks.is_empty() {
        notes.push("No disks detected by sysinfo; consider passing explicit --paths.".to_string());
    }

    DoctorInfo {
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
        current_dir,
        os_mount,
        read_only_mode: true,
        disks,
        notes,
    }
}

fn enumerate_disks() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let probes = disks
        .list()
        .iter()
        .map(|disk| {
            let disk_kind = match disk.kind() {
                SysDiskKind::HDD => DiskKind::Hdd,
                SysDiskKind::SSD => DiskKind::Ssd,
                _ => DiskKind::Unknown,
            };

            DiskProbe {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space_bytes: disk.total_space(),
                free_space_bytes: disk.available_space(),
                disk_kind,
                file_system: Some(disk.file_system().to_string_lossy().to_string()),
                is_removable: disk.is_removable(),
            }
        })
        .collect::<Vec<_>>();
    enrich_disks(probes)
}
