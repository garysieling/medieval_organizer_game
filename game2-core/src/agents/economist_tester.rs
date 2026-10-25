//! Economist Tester: analyzes economy for balance/exploits.
//! Detects: inflation, scarcity, unfairness, exploit strategies.

/// Economic analysis finding
#[derive(Debug)]
pub struct EconFinding {
    pub kind: EconIssue,
    pub description: String,
    pub severity: f32,   // 0-1
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EconIssue {
    Inflation,      // prices spiraling up
    Scarcity,       // critical good too rare
    Unfairness,     // one group exploits another
    Exploit,        // strategy that breaks economy
    Stagnation,     // economy frozen, no growth
}

/// Economist Tester agent
pub struct EconomistTester {
    pub name: String,
}

impl EconomistTester {
    pub fn new() -> Self {
        Self { name: "Economist Tester".into() }
    }

    /// Analyze market data for issues
    pub fn analyze(&self, prices: &[(&str, f32)], supply: &[(&str, u32)], wealth_dist: &[f32]) -> Vec<EconFinding> {
        let mut findings = vec![];
        // Check for inflation (avg price too high)
        let avg_price = prices.iter().map(|(_, p)| p).sum::<f32>() / prices.len().max(1) as f32;
        if avg_price > 50.0 {
            findings.push(EconFinding {
                kind: EconIssue::Inflation,
                description: format!("Prices high (avg {:.0})", avg_price),
                severity: 0.6,
                suggestion: "Increase supply or cap prices".into(),
            });
        }
        // Check for scarcity
        for (good, qty) in supply {
            if *qty < 10 {
                findings.push(EconFinding {
                    kind: EconIssue::Scarcity,
                    description: format!("'{}' scarce ({})", good, qty),
                    severity: 0.7,
                    suggestion: "Boost production or import".into(),
                });
            }
        }
        // Check for inequality (gini-like)
        let total: f32 = wealth_dist.iter().sum();
        let top20: f32 = wealth_dist.iter().take((wealth_dist.len() / 5).max(1)).sum();
        if total > 0.0 && top20 / total > 0.6 {
            findings.push(EconFinding {
                kind: EconIssue::Unfairness,
                description: "Top 20% hold >60% of wealth".into(),
                severity: 0.8,
                suggestion: "Redistribute or progressive policies".into(),
            });
        }
        findings
    }
}
