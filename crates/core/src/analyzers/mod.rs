use crate::model::{Recommendation, Report, RuleTrace};

pub mod dev_artifacts;
pub mod system_caches;
pub mod trend_analyzer;

/// A trait for application-specific or pattern-specific analysis.
pub trait Analyzer {
    fn id(&self) -> &'static str;
    fn analyze(&self, report: &Report) -> AnalyzerResult;
}

/// The output of a single analyzer.
#[derive(Default)]
pub struct AnalyzerResult {
    pub recommendations: Vec<Recommendation>,
    pub traces: Vec<RuleTrace>,
}

/// Runs all registered analyzers and returns their combined results.
pub fn run_analyzers(report: &Report) -> Vec<AnalyzerResult> {
    let analyzers: Vec<Box<dyn Analyzer>> = vec![
        Box::new(dev_artifacts::DevArtifactsAnalyzer),
        Box::new(system_caches::SystemCachesAnalyzer),
        Box::new(trend_analyzer::TrendAnalyzer),
    ];

    analyzers.iter().map(|a| a.analyze(report)).collect()
}
