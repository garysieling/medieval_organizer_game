//! Credibility Analyst: validates research sources for credibility and applicability.
//!
//! Infrastructure for source evaluation - credibility ratings, methodology assessment.
//! Intelligence layer runs as Claude Code skill or Ollama.

/// Source credibility rating
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CredibilityRating {
    APlus,  // 9-10/10: Peer-reviewed, widely cited, top journal
    A,      // 7-8/10: Peer-reviewed, credible institution
    B,      // 5-6/10: Academic but limited citations
    C,      // 3-4/10: Industry source, not peer-reviewed
    D,      // 1-2/10: Questionable, avoid
}

impl CredibilityRating {
    pub fn as_str(&self) -> &'static str {
        match self {
            CredibilityRating::APlus => "A+",
            CredibilityRating::A => "A",
            CredibilityRating::B => "B",
            CredibilityRating::C => "C",
            CredibilityRating::D => "D",
        }
    }

    pub fn score(&self) -> f32 {
        match self {
            CredibilityRating::APlus => 9.5,
            CredibilityRating::A => 7.5,
            CredibilityRating::B => 5.5,
            CredibilityRating::C => 3.5,
            CredibilityRating::D => 1.5,
        }
    }
}

/// Source evaluation
#[derive(Debug, Clone)]
pub struct SourceEvaluation {
    pub title: String,
    pub authors: String,
    pub year: Option<u32>,
    pub rating: CredibilityRating,
    pub methodology_quality: MethodologyQuality,
    pub relevance: Relevance,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MethodologyQuality {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Relevance {
    High,    // Directly applicable
    Medium,  // Somewhat applicable
    Low,     // Limited applicability
}

/// Credibility Analyst
pub struct CredibilityAnalyst {
    pub name: String,
}

impl CredibilityAnalyst {
    pub fn new() -> Self {
        Self {
            name: "Credibility Analyst".into(),
        }
    }

    /// Quick heuristic evaluation (skill does deeper analysis)
    pub fn quick_evaluate(&self, source_type: &str, peer_reviewed: bool) -> CredibilityRating {
        match (source_type, peer_reviewed) {
            ("journal", true) => CredibilityRating::APlus,
            ("conference", true) => CredibilityRating::A,
            ("preprint", _) => CredibilityRating::B,
            ("blog", _) => CredibilityRating::C,
            _ => CredibilityRating::D,
        }
    }
}
