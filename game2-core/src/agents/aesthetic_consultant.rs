//! Aesthetic Consultant: reviews visuals for aesthetic quality and consistency.
//!
//! Infrastructure for aesthetic assessment - visual quality, style consistency.
//! Intelligence layer runs as Claude Code skill or Ollama.

/// Aesthetic review of a screenshot or visual
#[derive(Debug, Clone)]
pub struct AestheticReview {
    pub screenshot_id: String,
    pub overall_rating: f32,  // 0-10
    pub consistency_score: f32,  // 0-5
    pub readability_score: f32,  // 0-5
    pub style_score: f32,  // 0-5
    pub bugs: Vec<VisualBug>,
    pub recommendations: Vec<String>,
}

/// A visual bug or issue
#[derive(Debug, Clone)]
pub struct VisualBug {
    pub severity: VisualSeverity,
    pub location: String,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualSeverity {
    Critical,   // Blocks gameplay or unreadable
    Important,  // Major UX issue
    Minor,      // Polish item
}

/// Aesthetic Consultant
pub struct AestheticConsultant {
    pub name: String,
}

impl AestheticConsultant {
    pub fn new() -> Self {
        Self {
            name: "Aesthetic Consultant".into(),
        }
    }

    /// Quick review (skill does detailed analysis)
    pub fn quick_review(&self, screenshot_id: &str) -> AestheticReview {
        // Placeholder - actual review via skill
        AestheticReview {
            screenshot_id: screenshot_id.into(),
            overall_rating: 7.0,
            consistency_score: 4.0,
            readability_score: 4.0,
            style_score: 3.5,
            bugs: vec![],
            recommendations: vec!["Run full aesthetic review via skill".into()],
        }
    }
}
