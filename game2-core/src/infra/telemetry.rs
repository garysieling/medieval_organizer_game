//! Telemetry: KPI recording (simplified OTel-style metrics/spans).
//! From backlog: "open telemetry to record KPIs" for playtest observation.

use std::time::Instant;

/// A metric point
#[derive(Debug)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub timestamp: u64,
    pub labels: Vec<(String, String)>,
}

/// A span (operation timing)
#[derive(Debug)]
pub struct Span {
    pub name: String,
    pub start: Instant,
    pub end: Option<Instant>,
    pub attributes: Vec<(String, String)>,
}

/// Telemetry collector
pub struct Telemetry {
    pub metrics: Vec<Metric>,
    pub spans: Vec<Span>,
    epoch: u64,
}

impl Telemetry {
    pub fn new() -> Self {
        Self { metrics: vec![], spans: vec![], epoch: 0 }
    }

    /// Record a metric value
    pub fn gauge(&mut self, name: &str, value: f64) {
        self.metrics.push(Metric {
            name: name.into(), value, timestamp: self.epoch,
            labels: vec![],
        });
        self.epoch += 1;
    }

    /// Start a span
    pub fn span(&mut self, name: &str) -> usize {
        self.spans.push(Span {
            name: name.into(), start: Instant::now(), end: None,
            attributes: vec![],
        });
        self.spans.len() - 1
    }

    /// End a span
    pub fn end_span(&mut self, idx: usize) {
        if let Some(s) = self.spans.get_mut(idx) {
            s.end = Some(Instant::now());
        }
    }

    /// Export metrics as text lines
    pub fn export_metrics(&self) -> Vec<String> {
        self.metrics.iter()
            .map(|m| format!("{}={} @{}", m.name, m.value, m.timestamp))
            .collect()
    }
}
