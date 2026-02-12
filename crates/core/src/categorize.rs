use std::collections::HashMap;

use crate::model::{Category, CategorySuggestion, DiskInfo, FileTypeSummary, PathStats};

#[derive(Default)]
struct ScoreState {
    score: f32,
    evidence: Vec<String>,
}

pub fn categorize_paths(paths: &[PathStats]) -> Vec<CategorySuggestion> {
    let mut suggestions = Vec::new();
    for path in paths {
        suggestions.extend(categorize_path(path));
    }
    suggestions
}

pub fn categorize_disks(disks: &[DiskInfo]) -> Vec<CategorySuggestion> {
    let mut output = Vec::new();
    for disk in disks {
        let mut scores: HashMap<Category, ScoreState> = HashMap::new();
        let mut signal = disk.name.to_lowercase();
        if let Some(model) = &disk.model {
            if !model.is_empty() {
                signal.push(' ');
                signal.push_str(&model.to_lowercase());
            }
        }

        if contains_any(
            &signal,
            &["photos", "pictures", "media", "dcim", "video", "videos"],
        ) {
            bump(
                &mut scores,
                Category::Media,
                0.82,
                "Disk label/model indicates media/photo usage.",
            );
        }

        if contains_any(
            &signal,
            &[
                "steam",
                "epic",
                "gog",
                "game",
                "games",
                "apps",
                "application",
            ],
        ) {
            bump(
                &mut scores,
                Category::Games,
                0.85,
                "Disk label/model indicates games or application library usage.",
            );
            bump(
                &mut scores,
                Category::Work,
                0.35,
                "Application-oriented disk labels suggest active workload usage.",
            );
        }

        if contains_any(
            &signal,
            &["backup", "time machine", "history", "snapshot", "restore"],
        ) {
            bump(
                &mut scores,
                Category::Backup,
                0.9,
                "Disk label/model indicates backup retention usage.",
            );
        }

        if contains_any(
            &signal,
            &["work", "project", "projects", "repo", "dev", "docs"],
        ) {
            bump(
                &mut scores,
                Category::Work,
                0.75,
                "Disk label/model indicates work/project usage.",
            );
        }

        if contains_any(&signal, &["archive", "cold", "old", "long-term"]) {
            bump(
                &mut scores,
                Category::Archive,
                0.7,
                "Disk label/model indicates archival usage.",
            );
        }

        for (category, state) in scores {
            if state.score < 0.35 {
                continue;
            }
            let rationale = if state.evidence.is_empty() {
                "Disk-level category inferred from weak signals.".to_string()
            } else {
                format!("Signals: {}", state.evidence.join("; "))
            };
            output.push(CategorySuggestion {
                target: disk.mount_point.clone(),
                disk_mount: Some(disk.mount_point.clone()),
                category,
                confidence: state.score.min(1.0),
                rationale,
                evidence: state.evidence,
            });
        }
    }

    output.sort_by(|a, b| {
        b.confidence
            .total_cmp(&a.confidence)
            .then_with(|| category_label(&a.category).cmp(category_label(&b.category)))
            .then_with(|| a.target.cmp(&b.target))
    });
    output
}

pub fn aggregate_categories_by_disk(suggestions: &[CategorySuggestion]) -> Vec<CategorySuggestion> {
    let mut grouped: HashMap<(String, Category), ScoreState> = HashMap::new();
    let mut counts: HashMap<(String, Category), usize> = HashMap::new();

    for suggestion in suggestions {
        let Some(mount) = &suggestion.disk_mount else {
            continue;
        };
        let key = (mount.clone(), suggestion.category.clone());
        let state = grouped.entry(key.clone()).or_default();
        state.score += suggestion.confidence;
        state.evidence.extend(suggestion.evidence.clone());
        *counts.entry(key).or_insert(0) += 1;
    }

    let mut output = grouped
        .into_iter()
        .filter_map(|((mount, category), mut state)| {
            let count = counts
                .get(&(mount.clone(), category.clone()))
                .copied()
                .unwrap_or(1);
            let confidence = (state.score / count as f32).min(1.0);
            if confidence < 0.35 {
                return None;
            }
            state.evidence.sort();
            state.evidence.dedup();
            Some(CategorySuggestion {
                target: mount.clone(),
                disk_mount: Some(mount),
                category,
                confidence,
                rationale: format!(
                    "Aggregated from {} path-level signal(s) on this disk.",
                    count
                ),
                evidence: state.evidence,
            })
        })
        .collect::<Vec<_>>();

    output.sort_by(|a, b| {
        b.confidence
            .total_cmp(&a.confidence)
            .then_with(|| category_label(&a.category).cmp(category_label(&b.category)))
            .then_with(|| a.target.cmp(&b.target))
    });
    output
}

pub fn categorize_path(path: &PathStats) -> Vec<CategorySuggestion> {
    let mut scores: HashMap<Category, ScoreState> = HashMap::new();
    let lowered_root = path.root_path.to_lowercase();

    score_name_patterns(path, &lowered_root, &mut scores);
    score_extension_distribution(&path.file_type_summary, &mut scores);
    score_activity(path, &mut scores);

    let mut output = scores
        .into_iter()
        .filter(|(_, state)| state.score >= 0.35)
        .map(|(category, state)| {
            let rationale = if state.evidence.is_empty() {
                "Category inferred from weak aggregate signals.".to_string()
            } else {
                format!("Signals: {}", state.evidence.join("; "))
            };
            CategorySuggestion {
                target: path.root_path.clone(),
                disk_mount: path.disk_mount.clone(),
                category,
                confidence: state.score.min(1.0),
                rationale,
                evidence: state.evidence,
            }
        })
        .collect::<Vec<_>>();

    output.sort_by(|a, b| {
        b.confidence
            .total_cmp(&a.confidence)
            .then_with(|| category_label(&a.category).cmp(category_label(&b.category)))
    });
    output
}

fn score_name_patterns(
    path: &PathStats,
    lowered_root: &str,
    scores: &mut HashMap<Category, ScoreState>,
) {
    let directory_names = path
        .largest_directories
        .iter()
        .map(|entry| entry.path.to_lowercase())
        .collect::<Vec<_>>();

    if contains_any(lowered_root, &["steam", "epic", "gog"])
        || directory_names
            .iter()
            .any(|name| contains_any(name, &["steamapps", "epic", "gog"]))
    {
        bump(
            scores,
            Category::Games,
            0.8,
            "Folder naming matches common game library paths.",
        );
    }

    if contains_any(lowered_root, &["dcim", "photos", "pictures", "videos"])
        || directory_names
            .iter()
            .any(|name| contains_any(name, &["dcim", "photos", "pictures", "videos"]))
    {
        bump(
            scores,
            Category::Media,
            0.75,
            "Folder naming indicates media/photo storage.",
        );
    }

    if contains_any(
        lowered_root,
        &["backup", "time machine", "history", "snapshot"],
    ) || directory_names
        .iter()
        .any(|name| contains_any(name, &["backup", "time machine", "history", "snapshot"]))
    {
        bump(
            scores,
            Category::Backup,
            0.9,
            "Folder naming indicates backup/snapshot usage.",
        );
    }

    if contains_any(lowered_root, &["projects", "work", "documents", "repos"])
        || directory_names
            .iter()
            .any(|name| contains_any(name, &["projects", "work", "documents", "repos"]))
    {
        bump(
            scores,
            Category::Work,
            0.7,
            "Folder naming indicates active work/project content.",
        );
    }

    if contains_any(lowered_root, &["archive", "old", "cold"])
        || directory_names
            .iter()
            .any(|name| contains_any(name, &["archive", "old", "cold"]))
    {
        bump(
            scores,
            Category::Archive,
            0.65,
            "Folder naming suggests archival storage.",
        );
    }
}

fn score_extension_distribution(
    summary: &FileTypeSummary,
    scores: &mut HashMap<Category, ScoreState>,
) {
    let total_bytes = summary.total_bytes.max(1) as f32;

    for ext in &summary.top_extensions {
        let ratio = ext.bytes as f32 / total_bytes;
        let ext_name = ext.extension.as_str();
        if is_media_extension(ext_name) && ratio >= 0.1 {
            bump(
                scores,
                Category::Media,
                0.8 * ratio,
                &format!("High media extension share: .{}", ext_name),
            );
        }
        if is_work_extension(ext_name) && ratio >= 0.08 {
            bump(
                scores,
                Category::Work,
                0.75 * ratio,
                &format!("High work/document extension share: .{}", ext_name),
            );
        }
        if is_archive_extension(ext_name) && ratio >= 0.08 {
            bump(
                scores,
                Category::Archive,
                0.7 * ratio,
                &format!("High archive/compressed extension share: .{}", ext_name),
            );
        }
    }
}

fn score_activity(path: &PathStats, scores: &mut HashMap<Category, ScoreState>) {
    let total_files = path.file_count.max(1) as f32;
    let stale_ratio = path.activity.stale_files as f32 / total_files;
    let recent_ratio = path.activity.recent_files as f32 / total_files;

    if stale_ratio > 0.6 {
        bump(
            scores,
            Category::Archive,
            0.4,
            "Large share of stale files indicates colder storage.",
        );
    }

    if recent_ratio > 0.35 {
        bump(
            scores,
            Category::Work,
            0.3,
            "Frequent recent modifications indicate active usage.",
        );
    }

    if stale_ratio > 0.7 && recent_ratio < 0.1 {
        bump(
            scores,
            Category::Backup,
            0.2,
            "Mostly stale files with little activity can indicate backup retention.",
        );
    }
}

fn bump(
    scores: &mut HashMap<Category, ScoreState>,
    category: Category,
    delta: f32,
    evidence: &str,
) {
    let state = scores.entry(category).or_default();
    state.score += delta;
    state.evidence.push(evidence.to_string());
}

fn contains_any(value: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| value.contains(pattern))
}

fn is_media_extension(ext: &str) -> bool {
    matches!(
        ext,
        "jpg" | "jpeg" | "png" | "heic" | "gif" | "mp4" | "mov" | "mkv" | "avi" | "mp3" | "flac"
    )
}

fn is_work_extension(ext: &str) -> bool {
    matches!(
        ext,
        "doc"
            | "docx"
            | "xls"
            | "xlsx"
            | "ppt"
            | "pptx"
            | "pdf"
            | "md"
            | "txt"
            | "rs"
            | "py"
            | "ts"
            | "js"
            | "java"
    )
}

fn is_archive_extension(ext: &str) -> bool {
    matches!(ext, "zip" | "7z" | "rar" | "tar" | "gz" | "bak")
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
    use crate::model::{
        ActivitySignals, DirectoryUsage, DiskInfo, DiskKind, DiskStorageType, ExtensionUsage,
        LargestFiles, LocalityClass, PerformanceClass,
    };

    use super::{categorize_disks, categorize_path, Category, FileTypeSummary, PathStats};

    fn build_path(root: &str, extensions: Vec<ExtensionUsage>) -> PathStats {
        PathStats {
            root_path: root.to_string(),
            disk_mount: Some("D:\\".to_string()),
            total_size_bytes: 10_000,
            file_count: 100,
            directory_count: 5,
            largest_files: LargestFiles {
                entries: Vec::new(),
            },
            largest_directories: vec![DirectoryUsage {
                path: format!("{root}/SteamLibrary"),
                size_bytes: 5_000,
            }],
            file_type_summary: FileTypeSummary {
                top_extensions: extensions,
                other_files: 0,
                other_bytes: 0,
                total_files: 100,
                total_bytes: 10_000,
            },
            activity: ActivitySignals {
                recent_files: 60,
                stale_files: 20,
                unknown_modified_files: 20,
            },
        }
    }

    #[test]
    fn scores_games_from_path_names() {
        let path = build_path(
            "D:/Games/Steam",
            vec![ExtensionUsage {
                extension: "pak".to_string(),
                files: 50,
                bytes: 8_000,
            }],
        );
        let categories = categorize_path(&path);
        assert!(categories
            .iter()
            .any(|item| item.category == Category::Games));
    }

    #[test]
    fn scores_media_from_extensions() {
        let path = build_path(
            "E:/Photos",
            vec![
                ExtensionUsage {
                    extension: "jpg".to_string(),
                    files: 70,
                    bytes: 7_000,
                },
                ExtensionUsage {
                    extension: "png".to_string(),
                    files: 10,
                    bytes: 1_000,
                },
            ],
        );
        let categories = categorize_path(&path);
        let media = categories
            .iter()
            .find(|item| item.category == Category::Media)
            .expect("media category");
        assert!(media.confidence >= 0.35);
    }

    #[test]
    fn scores_disk_purpose_from_labels() {
        let disks = vec![
            DiskInfo {
                name: "RED (Photos)".to_string(),
                mount_point: "G:\\".to_string(),
                total_space_bytes: 1,
                free_space_bytes: 1,
                disk_kind: DiskKind::Hdd,
                file_system: Some("ntfs".to_string()),
                storage_type: DiskStorageType::Hdd,
                locality_class: LocalityClass::LocalPhysical,
                locality_confidence: 0.8,
                locality_rationale: "test".to_string(),
                is_os_drive: false,
                is_removable: false,
                vendor: None,
                model: Some("WD Red Photos".to_string()),
                interface: None,
                rotational: Some(true),
                hybrid: Some(false),
                performance_class: PerformanceClass::Slow,
                performance_confidence: 0.8,
                performance_rationale: "test".to_string(),
                eligible_for_local_target: true,
                ineligible_reasons: Vec::new(),
                metadata_notes: Vec::new(),
                role_hint: Default::default(),
                target_role_eligibility: Vec::new(),
            },
            DiskInfo {
                name: "Black Rider (Games and Apps)".to_string(),
                mount_point: "D:\\".to_string(),
                total_space_bytes: 1,
                free_space_bytes: 1,
                disk_kind: DiskKind::Ssd,
                file_system: Some("ntfs".to_string()),
                storage_type: DiskStorageType::Ssd,
                locality_class: LocalityClass::LocalPhysical,
                locality_confidence: 0.8,
                locality_rationale: "test".to_string(),
                is_os_drive: false,
                is_removable: false,
                vendor: None,
                model: Some("Gaming Apps".to_string()),
                interface: None,
                rotational: Some(false),
                hybrid: Some(false),
                performance_class: PerformanceClass::Fast,
                performance_confidence: 0.9,
                performance_rationale: "test".to_string(),
                eligible_for_local_target: true,
                ineligible_reasons: Vec::new(),
                metadata_notes: Vec::new(),
                role_hint: Default::default(),
                target_role_eligibility: Vec::new(),
            },
        ];

        let categories = categorize_disks(&disks);
        assert!(categories
            .iter()
            .any(|item| item.target == "G:\\" && item.category == Category::Media));
        assert!(categories
            .iter()
            .any(|item| item.target == "D:\\" && item.category == Category::Games));
    }
}
