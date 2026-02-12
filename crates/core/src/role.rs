use std::collections::HashMap;

use crate::model::{Category, CategorySuggestion, DiskInfo, DiskRole, DiskRoleHint};

pub fn infer_disk_roles(disks: &mut [DiskInfo], categories: &[CategorySuggestion]) {
    let mut score_by_mount: HashMap<String, HashMap<Category, f32>> = HashMap::new();
    let mut evidence_by_mount: HashMap<String, Vec<String>> = HashMap::new();

    for suggestion in categories {
        let Some(mount) = &suggestion.disk_mount else {
            continue;
        };
        let scores = score_by_mount.entry(mount.clone()).or_default();
        *scores.entry(suggestion.category.clone()).or_insert(0.0) += suggestion.confidence;
        let evidence = evidence_by_mount.entry(mount.clone()).or_default();
        evidence.push(format!(
            "category:{} conf:{:.2}",
            category_label(&suggestion.category),
            suggestion.confidence
        ));
    }

    for disk in disks {
        let mount_scores = score_by_mount.entry(disk.mount_point.clone()).or_default();
        let mount_evidence = evidence_by_mount
            .entry(disk.mount_point.clone())
            .or_default();

        let mut label_signal = disk.name.to_lowercase();
        if let Some(model) = &disk.model {
            if !model.is_empty() {
                label_signal.push(' ');
                label_signal.push_str(&model.to_lowercase());
            }
        }

        if contains_any(
            &label_signal,
            &[
                "games",
                "game",
                "steam",
                "epic",
                "gog",
                "apps",
                "application",
            ],
        ) {
            *mount_scores.entry(Category::Games).or_insert(0.0) += 0.85;
            *mount_scores.entry(Category::Work).or_insert(0.0) += 0.35;
            mount_evidence.push("label:games_or_apps".to_string());
        }

        if contains_any(
            &label_signal,
            &[
                "photos", "photo", "pictures", "media", "dcim", "video", "videos",
            ],
        ) {
            *mount_scores.entry(Category::Media).or_insert(0.0) += 0.82;
            mount_evidence.push("label:media_or_photos".to_string());
        }

        if contains_any(
            &label_signal,
            &["backup", "time machine", "snapshot", "history", "restore"],
        ) {
            *mount_scores.entry(Category::Backup).or_insert(0.0) += 0.9;
            mount_evidence.push("label:backup".to_string());
        }

        if contains_any(&label_signal, &["archive", "cold", "old", "long-term"]) {
            *mount_scores.entry(Category::Archive).or_insert(0.0) += 0.75;
            mount_evidence.push("label:archive".to_string());
        }

        let games = *mount_scores.get(&Category::Games).unwrap_or(&0.0);
        let work = *mount_scores.get(&Category::Work).unwrap_or(&0.0);
        let media = *mount_scores.get(&Category::Media).unwrap_or(&0.0);
        let archive = *mount_scores.get(&Category::Archive).unwrap_or(&0.0);
        let backup = *mount_scores.get(&Category::Backup).unwrap_or(&0.0);
        let active = games + work;
        let cold = media + archive + backup;

        let role = if backup >= 0.85 && backup > active + 0.2 {
            DiskRole::BackupTarget
        } else if media >= 0.8 && media > active + 0.2 {
            DiskRole::MediaLibrary
        } else if games >= 0.85 && games >= work {
            DiskRole::GamesLibrary
        } else if active >= 0.9 && active > cold + 0.2 {
            DiskRole::ActiveWorkload
        } else if archive >= 0.75 && archive > active {
            DiskRole::Archive
        } else {
            let significant = [games, work, media, archive, backup]
                .into_iter()
                .filter(|score| *score >= 0.5)
                .count();
            if significant >= 2 {
                DiskRole::Mixed
            } else {
                DiskRole::Unknown
            }
        };

        let confidence = [games, work, media, archive, backup]
            .into_iter()
            .fold(0.0_f32, f32::max)
            .min(1.0);

        mount_evidence.sort();
        mount_evidence.dedup();

        disk.role_hint = DiskRoleHint {
            role: role.clone(),
            confidence,
            evidence: mount_evidence.clone(),
        };
        disk.target_role_eligibility = eligibility_for_role(role);
    }
}

fn eligibility_for_role(role: DiskRole) -> Vec<String> {
    match role {
        DiskRole::ActiveWorkload | DiskRole::GamesLibrary => vec![
            "active_workload".to_string(),
            "games_library".to_string(),
            "mixed".to_string(),
        ],
        DiskRole::MediaLibrary => vec![
            "media_library".to_string(),
            "archive".to_string(),
            "backup_target".to_string(),
        ],
        DiskRole::BackupTarget => vec!["backup_target".to_string(), "archive".to_string()],
        DiskRole::Archive => vec![
            "archive".to_string(),
            "backup_target".to_string(),
            "media_library".to_string(),
        ],
        DiskRole::Mixed | DiskRole::Unknown => vec![
            "active_workload".to_string(),
            "games_library".to_string(),
            "media_library".to_string(),
            "archive".to_string(),
            "backup_target".to_string(),
            "mixed".to_string(),
            "unknown".to_string(),
        ],
    }
}

fn contains_any(value: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| value.contains(pattern))
}

fn category_label(category: &Category) -> &'static str {
    match category {
        Category::Backup => "backup",
        Category::Games => "games",
        Category::Work => "work",
        Category::Media => "media",
        Category::Archive => "archive",
    }
}

#[cfg(test)]
mod tests {
    use super::infer_disk_roles;
    use crate::model::{
        Category, CategorySuggestion, DiskInfo, DiskKind, DiskRole, DiskStorageType, LocalityClass,
        PerformanceClass,
    };

    #[test]
    fn detects_games_and_media_roles_from_labels() {
        let mut disks = vec![
            disk("Black Rider (Games and Apps)", "D:\\"),
            disk("RED (Photos)", "G:\\"),
        ];

        let categories = vec![
            CategorySuggestion {
                target: "D:\\".to_string(),
                disk_mount: Some("D:\\".to_string()),
                category: Category::Games,
                confidence: 0.9,
                rationale: "test".to_string(),
                evidence: vec!["games".to_string()],
            },
            CategorySuggestion {
                target: "G:\\".to_string(),
                disk_mount: Some("G:\\".to_string()),
                category: Category::Media,
                confidence: 0.8,
                rationale: "test".to_string(),
                evidence: vec!["photos".to_string()],
            },
        ];

        infer_disk_roles(&mut disks, &categories);

        let d = disks.iter().find(|d| d.mount_point == "D:\\").expect("D");
        let g = disks.iter().find(|d| d.mount_point == "G:\\").expect("G");
        assert!(matches!(
            d.role_hint.role,
            DiskRole::GamesLibrary | DiskRole::ActiveWorkload
        ));
        assert_eq!(g.role_hint.role, DiskRole::MediaLibrary);
    }

    fn disk(name: &str, mount: &str) -> DiskInfo {
        DiskInfo {
            name: name.to_string(),
            mount_point: mount.to_string(),
            total_space_bytes: 1,
            free_space_bytes: 1,
            disk_kind: DiskKind::Unknown,
            file_system: Some("ntfs".to_string()),
            storage_type: DiskStorageType::Unknown,
            locality_class: LocalityClass::LocalPhysical,
            locality_confidence: 0.7,
            locality_rationale: "test".to_string(),
            is_os_drive: false,
            is_removable: false,
            vendor: None,
            model: Some(name.to_string()),
            interface: None,
            rotational: None,
            hybrid: None,
            performance_class: PerformanceClass::Unknown,
            performance_confidence: 0.4,
            performance_rationale: "test".to_string(),
            eligible_for_local_target: true,
            ineligible_reasons: Vec::new(),
            metadata_notes: Vec::new(),
            role_hint: Default::default(),
            target_role_eligibility: Vec::new(),
        }
    }
}
