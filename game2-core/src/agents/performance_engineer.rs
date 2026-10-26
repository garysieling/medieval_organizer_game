//! Performance Engineer: monitors performance, identifies bottlenecks.
//!
//! Infrastructure for performance monitoring - telemetry analysis, anomaly detection.
//! Intelligence layer runs as Claude Code skill or Ollama.

use std::collections::HashMap;

/// A performance metric
#[derive(Debug, Clone)]
pub struct PerfMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,  // "ms", "fps", "MB", etc.
    pub budget: Option<f64>,  // Optional budget/target
}

/// A performance issue detected
#[derive(Debug, Clone)]
pub struct PerfIssue {
    pub severity: PerfSeverity,
    pub metric: String,
    pub current: f64,
    pub budget: f64,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerfSeverity {
    Critical,  // Way over budget
    Warning,   // Approaching budget
    Info,      // Notable but not concerning
}

/// Performance analysis result
#[derive(Debug, Clone)]
pub struct PerfAnalysis {
    pub metrics: Vec<PerfMetric>,
    pub issues: Vec<PerfIssue>,
    pub bottlenecks: Vec<String>,  // Identified performance bottlenecks
    pub suggestions: Vec<String>,
}

/// Performance Engineer
pub struct PerformanceEngineer {
    pub name: String,
    pub budgets: HashMap<String, f64>,  // Metric name -> budget
}

impl PerformanceEngineer {
    pub fn new() -> Self {
        let mut budgets = HashMap::new();
        // Default budgets
        budgets.insert("frame_time_ms".into(), 16.67);  // 60fps
        budgets.insert("tick_time_ms".into(), 10.0);
        budgets.insert("memory_mb".into(), 500.0);

        Self {
            name: "Performance Engineer".into(),
            budgets,
        }
    }

    /// Analyze metrics against budgets
    pub fn analyze(&self, metrics: Vec<PerfMetric>) -> PerfAnalysis {
        let mut issues = vec![];
        let mut bottlenecks = vec![];

        for metric in &metrics {
            if let Some(budget) = metric.budget.or_else(|| self.budgets.get(&metric.name).copied()) {
                let ratio = metric.value / budget;

                if ratio > 2.0 {
                    issues.push(PerfIssue {
                        severity: PerfSeverity::Critical,
                        metric: metric.name.clone(),
                        current: metric.value,
                        budget,
                        description: format!("{} is {:.0}% over budget", metric.name, (ratio - 1.0) * 100.0),
                        suggestion: format!("Optimize {}: current {:.1}{}, budget {:.1}{}",
                            metric.name, metric.value, metric.unit, budget, metric.unit),
                    });
                    bottlenecks.push(metric.name.clone());
                } else if ratio > 1.2 {
                    issues.push(PerfIssue {
                        severity: PerfSeverity::Warning,
                        metric: metric.name.clone(),
                        current: metric.value,
                        budget,
                        description: format!("{} approaching budget", metric.name),
                        suggestion: format!("Monitor {} closely", metric.name),
                    });
                }
            }
        }

        PerfAnalysis {
            metrics,
            issues,
            bottlenecks,
            suggestions: vec!["Profile hot paths".into(), "Check for unnecessary allocations".into()],
        }
    }

    /// Set performance budget for a metric
    pub fn set_budget(&mut self, metric: &str, budget: f64) {
        self.budgets.insert(metric.into(), budget);
    }
}
