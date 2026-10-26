//! PR Agent: creates, reviews, and merges pull requests.
//!
//! Infrastructure for PR management - quality gates, reviews, merge decisions.
//! Intelligence layer runs as Claude Code skill or Ollama.

/// PR quality assessment
#[derive(Debug, Clone)]
pub struct PRQualityCheck {
    pub builds: bool,
    pub tests_pass: bool,
    pub has_tests: bool,
    pub has_docs: bool,
    pub code_quality: CodeQuality,
    pub recommendation: PRDecision,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CodeQuality {
    Excellent,
    Good,
    Acceptable,
    NeedsWork,
    Poor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PRDecision {
    Approve,           // Ready to merge
    ApproveWithNotes,  // Merge but note improvements
    RequestChanges,    // Block until fixed
    Reject,            // Don't merge
}

/// PR metadata
#[derive(Debug, Clone)]
pub struct PRMetadata {
    pub title: String,
    pub description: String,
    pub agent: String,
    pub files_changed: Vec<String>,
    pub task_id: Option<String>,
}

/// PR Agent
pub struct PRAgent {
    pub name: String,
}

impl PRAgent {
    pub fn new() -> Self {
        Self {
            name: "PR Agent".into(),
        }
    }

    /// Quick quality check (skill does detailed review)
    pub fn quick_check(&self, builds: bool, tests_pass: bool) -> PRQualityCheck {
        let recommendation = match (builds, tests_pass) {
            (true, true) => PRDecision::Approve,
            (true, false) => PRDecision::RequestChanges,
            (false, _) => PRDecision::Reject,
        };

        PRQualityCheck {
            builds,
            tests_pass,
            has_tests: false,  // Would need deeper analysis
            has_docs: false,
            code_quality: CodeQuality::Acceptable,
            recommendation,
        }
    }
}
