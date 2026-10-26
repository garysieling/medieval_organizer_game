//! Exploratory Tester (UI Clicker): explores UI by clicking around.
//!
//! Infrastructure for UI testing - click sequences, visual bugs, flow validation.
//! Intelligence layer runs as Claude Code skill or Ollama.

/// A UI action (click, input, etc.)
#[derive(Debug, Clone)]
pub struct UIAction {
    pub action_type: ActionType,
    pub target: String,  // UI element ID or description
    pub input: Option<String>,  // For text input actions
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    Click,
    Input,
    Scroll,
    Wait,
}

/// A UI bug found during exploration
#[derive(Debug, Clone)]
pub struct UIBug {
    pub severity: BugSeverity,
    pub location: String,  // Where in UI
    pub description: String,
    pub reproduction: Vec<UIAction>,  // Steps to reproduce
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BugSeverity {
    Blocker,   // Prevents gameplay
    Major,     // Major UX issue
    Minor,     // Small visual bug
}

/// Result of UI exploration
#[derive(Debug, Clone)]
pub struct UITestResult {
    pub session_id: String,
    pub actions_taken: Vec<UIAction>,
    pub bugs_found: Vec<UIBug>,
    pub flow_completed: bool,
    pub notes: String,
}

/// Exploratory Tester (UI Clicker mode)
pub struct ExploratoryTesterUI {
    pub name: String,
}

impl ExploratoryTesterUI {
    pub fn new() -> Self {
        Self {
            name: "Exploratory Tester (UI Clicker)".into(),
        }
    }

    /// Generate random click sequence
    pub fn generate_click_sequence(&self, num_actions: usize) -> Vec<UIAction> {
        // Placeholder: real implementation would actually interact with UI
        // For now, generate sample sequence
        vec![
            UIAction {
                action_type: ActionType::Click,
                target: "start_button".into(),
                input: None,
            },
        ]
    }

    /// Execute UI test (placeholder - actual execution via UI framework)
    pub fn test(&self, session_id: &str, actions: Vec<UIAction>) -> UITestResult {
        // Placeholder: actual implementation would execute actions and capture results
        UITestResult {
            session_id: session_id.into(),
            actions_taken: actions,
            bugs_found: vec![],
            flow_completed: true,
            notes: "UI test placeholder - actual execution via skill/Ollama".into(),
        }
    }
}
