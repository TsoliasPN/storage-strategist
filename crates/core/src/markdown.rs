use crate::model::{Category, Recommendation, Report};

pub fn render_markdown_summary(report: &Report, recommendations: &[Recommendation]) -> String {
    let mut out = String::new();
    out.push_str("# Storage Strategist Summary\n\n");
    out.push_str(&format!(
        "- Report version: `{}`\n- Generated at: `{}`\n- Scan roots: `{}`\n- Backend: `{:?}`\n- Scan elapsed: `{} ms`\n\n",
        report.report_version,
        report.generated_at,
        report.scan.roots.join("`, `"),
        report.scan_metrics.backend,
        report.scan_metrics.elapsed_ms
    ));

    out.push_str("## Disk Inventory\n\n");
    if report.disks.is_empty() {
        out.push_str("No disks detected.\n\n");
    } else {
        for disk in &report.disks {
            out.push_str(&format!(
                "- `{}` (`{}`): total {}, free {}, kind `{:?}`, type `{:?}`, locality `{:?}`, os_drive `{}`, eligible_target `{}`\n",
                disk.name,
                disk.mount_point,
                human_bytes(disk.total_space_bytes),
                human_bytes(disk.free_space_bytes),
                disk.disk_kind,
                disk.storage_type,
                disk.locality_class,
                disk.is_os_drive,
                disk.eligible_for_local_target
            ));
            if !disk.ineligible_reasons.is_empty() {
                out.push_str(&format!(
                    "  - ineligible reasons: {}\n",
                    disk.ineligible_reasons.join("; ")
                ));
            }
        }
        out.push('\n');
    }

    out.push_str("## Path Summaries\n\n");
    for path in &report.paths {
        out.push_str(&format!(
            "### `{}`\n\n- Files: {}\n- Directories: {}\n- Size: {}\n",
            path.root_path,
            path.file_count,
            path.directory_count,
            human_bytes(path.total_size_bytes)
        ));

        if !path.largest_directories.is_empty() {
            out.push_str("- Largest directories:\n");
            for directory in &path.largest_directories {
                out.push_str(&format!(
                    "  - `{}` ({})\n",
                    directory.path,
                    human_bytes(directory.size_bytes)
                ));
            }
        }

        if !path.file_type_summary.top_extensions.is_empty() {
            out.push_str("- Top file types:\n");
            for item in &path.file_type_summary.top_extensions {
                out.push_str(&format!(
                    "  - `.{}`: {} file(s), {}\n",
                    item.extension,
                    item.files,
                    human_bytes(item.bytes)
                ));
            }
        }

        out.push('\n');
    }

    out.push_str("## Category Suggestions\n\n");
    if report.categories.is_empty() {
        out.push_str("No category suggestions generated.\n\n");
    } else {
        for category in &report.categories {
            out.push_str(&format!(
                "- `{}` -> `{}` (confidence {:.2}): {}\n",
                category.target,
                category_label(&category.category),
                category.confidence,
                category.rationale
            ));
        }
        out.push('\n');
    }

    out.push_str("## Duplicate Highlights\n\n");
    if report.duplicates.is_empty() {
        out.push_str("No duplicate groups were detected.\n\n");
    } else {
        for group in report.duplicates.iter().take(20) {
            out.push_str(&format!(
                "- {} duplicate(s), {} each, wasted ~{}, label `{}`\n",
                group.files.len(),
                human_bytes(group.size_bytes),
                human_bytes(group.total_wasted_bytes),
                duplicate_intent_label(&group.intent.label)
            ));
        }
        out.push('\n');
    }

    out.push_str("## Recommendations\n\n");
    if recommendations.is_empty() {
        out.push_str("No recommendations generated.\n");
    } else {
        for recommendation in recommendations {
            out.push_str(&format!(
                "### {}\n\n- Risk: `{:?}`\n- Confidence: `{:.2}`\n- Policy safe: `{}`\n- Rationale: {}\n",
                recommendation.title,
                recommendation.risk_level,
                recommendation.confidence,
                recommendation.policy_safe,
                recommendation.rationale
            ));
            if let Some(target) = &recommendation.target_mount {
                out.push_str(&format!("- Target mount: `{}`\n", target));
            }
            if let Some(space) = recommendation.estimated_impact.space_saving_bytes {
                out.push_str(&format!(
                    "- Estimated space impact: {}\n",
                    human_bytes(space)
                ));
            }
            if let Some(performance) = &recommendation.estimated_impact.performance {
                out.push_str(&format!("- Performance impact: {}\n", performance));
            }
            if let Some(notes) = &recommendation.estimated_impact.risk_notes {
                out.push_str(&format!("- Risk notes: {}\n", notes));
            }
            out.push('\n');
        }
    }

    if !report.policy_decisions.is_empty() {
        out.push_str("## Policy Decisions\n\n");
        for decision in &report.policy_decisions {
            out.push_str(&format!(
                "- `{}` on `{}`: `{:?}` ({})\n",
                decision.policy_id, decision.recommendation_id, decision.action, decision.rationale
            ));
        }
        out.push('\n');
    }

    if !report.rule_traces.is_empty() {
        out.push_str("## Rule Traces\n\n");
        for trace in &report.rule_traces {
            out.push_str(&format!(
                "- `{}`: `{:?}` ({})\n",
                trace.rule_id, trace.status, trace.detail
            ));
        }
        out.push('\n');
    }

    if !report.warnings.is_empty() {
        out.push_str("## Warnings\n\n");
        for warning in &report.warnings {
            out.push_str(&format!("- {}\n", warning));
        }
    }

    out
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

fn duplicate_intent_label(label: &crate::model::DuplicateIntentLabel) -> &'static str {
    match label {
        crate::model::DuplicateIntentLabel::LikelyIntentional => "likely_intentional",
        crate::model::DuplicateIntentLabel::LikelyRedundant => "likely_redundant",
    }
}

fn human_bytes(value: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if value == 0 {
        return "0 B".to_string();
    }
    let mut size = value as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{size:.1} {}", UNITS[unit])
}
