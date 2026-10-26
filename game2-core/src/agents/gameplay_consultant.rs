//! Gameplay Consultant: watches gameplay and provides qualitative feedback.
//!
//! Infrastructure for gameplay assessment - feel, pacing, design issues.
//! Intelligence layer runs as Claude Code skill or Ollama.

/// Gameplay consultation result
#[derive(Debug, Clone)]
pub struct GameplayConsultation {
    pub session_id: String,
    pub duration_mins: u32,
    pub overall_assessment: GameplayAssessment,
    pub pacing_score: f32,  // 0-5
    pub feel_score: f32,  // 0-5
    pub clarity_score: f32,  // 0-5
    pub issues: Vec<DesignIssue>,
    pub positive_moments: Vec<GameplayMoment>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameplayAssessment {
    Engaging,
    Decent,
    NeedsWork,
    Poor,
}

/// A design issue observed
#[derive(Debug, Clone)]
pub struct DesignIssue {
    pub severity: IssueSeverity,
    pub timestamp: Option<String>,  // When in gameplay
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IssueSeverity {
    Critical,   // Must fix
    Important,  // Should fix
    Polish,     // Nice to have
}

/// A notable gameplay moment
#[derive(Debug, Clone)]
pub struct GameplayMoment {
    pub timestamp: Option<String>,
    pub description: String,
    pub why_it_worked: String,
}

/// Gameplay Consultant
pub struct GameplayConsultant {
    pub name: String,
}

impl GameplayConsultant {
    pub fn new() -> Self {
        Self {
            name: "Gameplay Consultant".into(),
        }
    }

    /// Quick assessment (skill does detailed analysis)
    pub fn quick_assess(&self, session_id: &str) -> GameplayConsultation {
        // Placeholder - actual consultation via skill
        GameplayConsultation {
            session_id: session_id.into(),
            duration_mins: 0,
            overall_assessment: GameplayAssessment::Decent,
            pacing_score: 3.5,
            feel_score: 3.5,
            clarity_score: 4.0,
            issues: vec![],
            positive_moments: vec![],
            recommendations: vec!["Run full gameplay consultation via skill".into()],
        }
    }
}
