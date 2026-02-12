use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::model::Report;
use crate::recommend::generate_recommendation_bundle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSuite {
    pub cases: Vec<EvaluationCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationCase {
    pub name: String,
    pub report: String,
    #[serde(default)]
    pub expected_top_ids: Vec<String>,
    #[serde(default)]
    pub forbidden_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub total_cases: usize,
    pub passed_cases: usize,
    pub precision_at_3: f32,
    pub contradiction_rate: f32,
    pub unsafe_recommendations: u64,
    pub case_results: Vec<EvaluationCaseResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationCaseResult {
    pub name: String,
    pub passed: bool,
    pub observed_ids: Vec<String>,
    pub expected_top_ids: Vec<String>,
    pub forbidden_hits: Vec<String>,
    pub precision_at_3: f32,
    pub contradiction_count: u64,
}

pub fn evaluate_suite_file(path: &Path) -> Result<EvaluationResult> {
    let suite_text = fs::read_to_string(path)
        .with_context(|| format!("failed to read evaluation suite {}", path.display()))?;
    let suite: EvaluationSuite =
        serde_json::from_str(&suite_text).context("failed to parse evaluation suite JSON")?;
    evaluate_suite(path, &suite)
}

pub fn evaluate_suite(suite_path: &Path, suite: &EvaluationSuite) -> Result<EvaluationResult> {
    let suite_dir = suite_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut case_results = Vec::new();
    let mut passed_cases = 0_usize;
    let mut precision_total = 0.0_f32;
    let mut contradiction_cases = 0_u64;
    let mut unsafe_recommendations = 0_u64;

    for case in &suite.cases {
        let report_path = suite_dir.join(&case.report);
        let report_text = fs::read_to_string(&report_path)
            .with_context(|| format!("failed to read report fixture {}", report_path.display()))?;
        let report: Report = serde_json::from_str(&report_text)
            .with_context(|| format!("failed to parse fixture {}", report_path.display()))?;

        let bundle = generate_recommendation_bundle(&report);
        let observed_ids = bundle
            .recommendations
            .iter()
            .map(|r| r.id.clone())
            .collect::<Vec<_>>();
        let forbidden_hits = case
            .forbidden_ids
            .iter()
            .filter(|id| observed_ids.iter().any(|observed| observed == *id))
            .cloned()
            .collect::<Vec<_>>();

        let expected = case
            .expected_top_ids
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let top3_len = observed_ids.len().min(3);
        let hit_count = observed_ids
            .iter()
            .take(3)
            .filter(|id| expected.contains(id.as_str()))
            .count() as f32;
        let precision_at_3 = if top3_len == 0 {
            0.0
        } else {
            hit_count / top3_len as f32
        };
        precision_total += precision_at_3;

        if bundle.contradiction_count > 0 {
            contradiction_cases = contradiction_cases.saturating_add(1);
        }

        unsafe_recommendations = unsafe_recommendations.saturating_add(
            bundle
                .recommendations
                .iter()
                .filter(|rec| !rec.policy_safe)
                .count() as u64,
        );

        let passed =
            forbidden_hits.is_empty() && (case.expected_top_ids.is_empty() || hit_count > 0.0);
        if passed {
            passed_cases += 1;
        }

        case_results.push(EvaluationCaseResult {
            name: case.name.clone(),
            passed,
            observed_ids,
            expected_top_ids: case.expected_top_ids.clone(),
            forbidden_hits,
            precision_at_3,
            contradiction_count: bundle.contradiction_count,
        });
    }

    let total_cases = suite.cases.len();
    let contradiction_rate = if total_cases == 0 {
        0.0
    } else {
        contradiction_cases as f32 / total_cases as f32
    };

    Ok(EvaluationResult {
        total_cases,
        passed_cases,
        precision_at_3: if total_cases == 0 {
            0.0
        } else {
            precision_total / total_cases as f32
        },
        contradiction_rate,
        unsafe_recommendations,
        case_results,
    })
}

#[cfg(test)]
mod tests {
    use super::{evaluate_suite, EvaluationCase, EvaluationSuite};
    use std::path::Path;

    #[test]
    fn evaluates_fixture_suite() {
        let suite = EvaluationSuite {
            cases: vec![EvaluationCase {
                name: "sample".to_string(),
                report: "sample-report.json".to_string(),
                expected_top_ids: vec!["backup-gap".to_string()],
                forbidden_ids: vec!["consolidation-opportunity".to_string()],
            }],
        };

        let result = evaluate_suite(Path::new("../../fixtures/eval-suite.json"), &suite)
            .expect("evaluation should run");
        assert_eq!(result.total_cases, 1);
        assert!(result.precision_at_3 >= 0.0);
    }
}
