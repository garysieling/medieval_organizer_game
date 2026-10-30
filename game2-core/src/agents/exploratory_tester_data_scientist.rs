//! Exploratory Tester (Data Scientist): analyzes logs, telemetry, data patterns.
//!
//! Infrastructure for data-driven testing - anomaly detection, pattern analysis.
//! Intelligence layer runs as Claude Code skill or Ollama.

use std::collections::HashMap;

/// An anomaly detected in telemetry/logs
#[derive(Debug, Clone)]
pub struct Anomaly {
    pub tick: u64,
    pub metric: String,
    pub expected: f32,
    pub actual: f32,
    pub severity: Severity,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Critical,  // Game-breaking
    Warning,   // Suspicious but playable
    Info,      // Notable but not concerning
}

/// A pattern detected in data
#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: String,
    pub description: String,
    pub frequency: f32,  // How often this pattern appears
    pub confidence: f32, // 0-1, confidence in detection
}

/// Test result from data analysis
#[derive(Debug, Clone)]
pub struct DataTestResult {
    pub test_name: String,
    pub passed: bool,
    pub anomalies: Vec<Anomaly>,
    pub patterns: Vec<Pattern>,
    pub notes: String,
}

/// Exploratory Tester (Data Scientist mode)
pub struct ExploratoryTesterData {
    pub name: String,
}

impl ExploratoryTesterData {
    pub fn new() -> Self {
        Self {
            name: "Exploratory Tester (Data Scientist)".into(),
        }
    }

    /// Detect anomalies in time-series data
    pub fn detect_anomalies(&self, metric: &str, data: &[(u64, f32)]) -> Vec<Anomaly> {
        let mut anomalies = vec![];

        // Simple heuristic: detect sudden spikes (>3x mean change)
        if data.len() < 2 { return anomalies; }

        let values: Vec<f32> = data.iter().map(|(_, v)| *v).collect();
        let mean = values.iter().sum::<f32>() / values.len() as f32;

        for window in data.windows(2) {
            let (_tick1, val1) = window[0];
            let (tick2, val2) = window[1];
            let change = (val2 - val1).abs();

            if change > mean * 3.0 {
                anomalies.push(Anomaly {
                    tick: tick2,
                    metric: metric.into(),
                    expected: val1,
                    actual: val2,
                    severity: if change > mean * 10.0 { Severity::Critical } else { Severity::Warning },
                    description: format!("Spike detected: {} → {} (change: {:.1})", val1, val2, change),
                });
            }
        }

        anomalies
    }

    /// Detect patterns in categorical data (e.g., repeated errors)
    pub fn detect_patterns(&self, events: &[String]) -> Vec<Pattern> {
        let mut patterns = vec![];
        let mut counts: HashMap<String, usize> = HashMap::new();

        for event in events {
            *counts.entry(event.clone()).or_insert(0) += 1;
        }

        let total = events.len() as f32;
        for (event, count) in counts {
            let freq = count as f32 / total;
            if freq > 0.1 {  // Pattern if appears in >10% of events
                patterns.push(Pattern {
                    name: event.clone(),
                    description: format!("Repeats {} times ({:.0}%)", count, freq * 100.0),
                    frequency: freq,
                    confidence: if freq > 0.3 { 0.9 } else { 0.7 },
                });
            }
        }

        patterns
    }

    /// Run exploratory test on telemetry data
    pub fn test(&self, test_name: &str, metrics: &[(&str, Vec<(u64, f32)>)], events: &[String]) -> DataTestResult {
        let mut all_anomalies = vec![];
        for (metric, data) in metrics {
            all_anomalies.extend(self.detect_anomalies(metric, data));
        }

        let patterns = self.detect_patterns(events);
        let passed = all_anomalies.iter().all(|a| matches!(a.severity, Severity::Info | Severity::Warning));

        DataTestResult {
            test_name: test_name.into(),
            passed,
            anomalies: all_anomalies,
            patterns,
            notes: if passed { "No critical anomalies".into() } else { "Critical anomalies detected".into() },
        }
    }
}
