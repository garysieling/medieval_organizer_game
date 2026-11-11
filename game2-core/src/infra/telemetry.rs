//! Telemetry: KPI recording (simplified OTel-style metrics/spans).
//! From backlog: "open telemetry to record KPIs" for playtest observation.

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

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
    tick: u64,
}

impl Telemetry {
    pub fn new() -> Self {
        Self { metrics: vec![], spans: vec![], tick: 0 }
    }

    /// Set the simulation tick that subsequent metrics are stamped with
    pub fn set_tick(&mut self, tick: u64) {
        self.tick = tick;
    }

    /// Record a metric value at the current tick
    pub fn gauge(&mut self, name: &str, value: f64) {
        self.metrics.push(Metric {
            name: name.into(), value, timestamp: self.tick,
            labels: vec![],
        });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gauges_in_same_tick_share_timestamp() {
        let mut t = Telemetry::new();
        t.set_tick(50);
        t.gauge("a", 1.0);
        t.gauge("b", 2.0);
        t.set_tick(100);
        t.gauge("a", 3.0);
        assert_eq!(t.export_metrics(), vec!["a=1 @50", "b=2 @50", "a=3 @100"]);
    }
}
