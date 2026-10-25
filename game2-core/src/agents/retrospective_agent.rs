//! Retrospective Agent: analyzes past work, proposes systemic improvements.
//! Looks at sprint history, task outcomes, agent KBs for patterns.

/// A retrospective finding
#[derive(Debug)]
pub struct Finding {
    pub category: String,   // "went well", "didn't", "learned", "action"
    pub description: String,
    pub evidence: String,   // log/task that shows it
}

/// An action item proposed to other agents
#[derive(Debug)]
pub struct ActionItem {
    pub target_agent: String,
    pub proposal: String,
    pub motivation: String,  // what incident prompted this
}

/// Retrospective Agent
pub struct RetrospectiveAgent {
    pub name: String,
}

impl RetrospectiveAgent {
    pub fn new() -> Self {
        Self { name: "Retrospective Agent".into() }
    }

    /// Analyze a sprint history and produce findings
    pub fn analyze(&self, sprint_log: &str) -> Vec<Finding> {
        let mut findings = vec![];
        // Simple heuristic: look for repeated issues
        if sprint_log.contains("blocked") || sprint_log.contains("error") {
            findings.push(Finding {
                category: "didn't".into(),
                description: "Environment/exe blocks recurring".into(),
                evidence: "multiple blocked executions".into(),
            });
        }
        findings
    }

    /// Propose an action for another agent
    pub fn propose_action(&self, target: &str, proposal: &str, motivation: &str) -> ActionItem {
        ActionItem {
            target_agent: target.into(),
            proposal: proposal.into(),
            motivation: motivation.into(),
        }
    }
}
