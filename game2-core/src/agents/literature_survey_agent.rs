//! Literature Survey Agent: academic research, domain surveys.
//!
//! Infrastructure for literature surveys - data structures and KB utilities.
//! The intelligence layer runs as a Claude Code skill (.claude/skills/literature-survey/)
//! or via Ollama for free execution.

use std::fs;
use std::path::PathBuf;

/// A completed literature survey
#[derive(Debug, Clone)]
pub struct Survey {
    pub domain: String,
    pub date: String,
    pub sources: Vec<Source>,
    pub mechanics: Vec<MechanicProposal>,
    pub confidence: Confidence,
    pub wiki_path: String,
}

/// A source found during research
#[derive(Debug, Clone)]
pub struct Source {
    pub title: String,
    pub author: String,
    pub year: Option<u32>,
    pub rating: SourceRating,  // A+ to D
    pub relevance: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceRating {
    APlus,  // Peer-reviewed, widely cited, foundational
    A,      // Peer-reviewed, well-cited
    B,      // Academic but less cited, or well-regarded industry
    C,      // Industry source or reputable blog
    D,      // Unverified or low-quality
}

impl SourceRating {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceRating::APlus => "A+",
            SourceRating::A => "A",
            SourceRating::B => "B",
            SourceRating::C => "C",
            SourceRating::D => "D",
        }
    }
}

/// A proposed game mechanic from academic research
#[derive(Debug, Clone)]
pub struct MechanicProposal {
    pub name: String,
    pub academic_basis: String,
    pub variables: Vec<String>,
    pub description: String,
    pub educational_value: String,
    pub fun_factor: String,
    pub implementation_suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Confidence::High => "High",
            Confidence::Medium => "Medium",
            Confidence::Low => "Low",
        }
    }
}

/// KB file utilities
pub struct LiteratureSurveyKB {
    kb_path: PathBuf,
}

impl LiteratureSurveyKB {
    pub fn new() -> Self {
        Self {
            kb_path: PathBuf::from(".agents/literature-survey-agent/kb"),
        }
    }

    /// Read a KB file
    pub fn read_kb(&self, filename: &str) -> Result<String, String> {
        let path = self.kb_path.join(filename);
        fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))
    }

    /// Write a KB file
    pub fn write_kb(&self, filename: &str, content: &str) -> Result<(), String> {
        let path = self.kb_path.join(filename);
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    /// Append to a KB file
    pub fn append_kb(&self, filename: &str, content: &str) -> Result<(), String> {
        let path = self.kb_path.join(filename);
        let mut existing = fs::read_to_string(&path).unwrap_or_default();
        existing.push_str("\n");
        existing.push_str(content);
        fs::write(&path, existing)
            .map_err(|e| format!("Failed to append to {}: {}", path.display(), e))
    }

    /// Add a survey entry to surveys.md
    pub fn add_survey(&self, survey: &Survey) -> Result<(), String> {
        let entry = format!(
            r#"
### {}: {}
**Date Completed:** {}
**Domain:** {}
**Output:** {}
**Yield:**
{}

**Key Sources:**
{}

**Confidence:** {}
**Tags:** {}

---
"#,
            survey.domain,
            survey.domain,
            survey.date,
            survey.domain,
            survey.wiki_path,
            survey.mechanics.iter()
                .map(|m| format!("- {}", m.name))
                .collect::<Vec<_>>()
                .join("\n"),
            survey.sources.iter()
                .map(|s| format!("- {} ({}) - {}", s.title, s.author, s.rating.as_str()))
                .collect::<Vec<_>>()
                .join("\n"),
            survey.confidence.as_str(),
            survey.domain.to_lowercase().replace(" ", "-")
        );
        self.append_kb("surveys.md", &entry)
    }

    /// Add a source to source-log.md
    pub fn add_source(&self, source: &Source) -> Result<(), String> {
        let entry = format!(
            r#"
#### {} - {} ({})
**Type:** Academic/Historical/Industry
**Domain:** [To be categorized]
**Rating:** {}
**Year:** {}
**Relevance:** {}
**URL:** {}
**Notes:** [To be filled]

---
"#,
            source.title,
            source.author,
            source.year.map(|y| y.to_string()).unwrap_or("Unknown".into()),
            source.rating.as_str(),
            source.year.map(|y| y.to_string()).unwrap_or("Unknown".into()),
            source.relevance,
            source.url.as_ref().map(|u| u.as_str()).unwrap_or("N/A")
        );
        self.append_kb("source-log.md", &entry)
    }

    /// Add a mechanic translation to mechanism-translations.md
    pub fn add_mechanic_translation(&self, mechanic: &MechanicProposal) -> Result<(), String> {
        let entry = format!(
            r#"
### {}: {}
**Academic Source:** {}
**Game Mechanic:** {}
**Implementation:** {}
**Simplification:** {}
**Educational Value:** {}
**Fun Factor:** {}
**Tags:** [To be categorized]

---
"#,
            "Pattern",  // TODO: categorize pattern type
            mechanic.name,
            mechanic.academic_basis,
            mechanic.description,
            mechanic.implementation_suggestion.as_ref().unwrap_or(&"TBD".to_string()),
            mechanic.variables.join(", "),
            mechanic.educational_value,
            mechanic.fun_factor
        );
        self.append_kb("mechanism-translations.md", &entry)
    }

    /// Get next domain from rotation schedule
    pub fn get_next_domain(&self) -> Result<String, String> {
        let coverage = self.read_kb("domain-coverage.md")?;
        // Simple heuristic: look for "Cycle 2 (Proposed):" and extract first item
        if let Some(cycle2_pos) = coverage.find("Cycle 2 (Proposed):") {
            let after_cycle2 = &coverage[cycle2_pos..];
            if let Some(first_item) = after_cycle2.lines()
                .skip(1)
                .find(|line| line.trim().starts_with("1.")) {
                // Extract domain name from "1. **Domain Name** (PRIORITY - description)"
                if let Some(start) = first_item.find("**") {
                    if let Some(end) = first_item[start+2..].find("**") {
                        return Ok(first_item[start+2..start+2+end].to_string());
                    }
                }
            }
        }
        Ok("Agriculture & Food Systems".to_string())  // Fallback to first priority
    }
}

/// Literature Survey Agent - delegates to skill or Ollama
pub struct LiteratureSurveyAgent {
    pub kb: LiteratureSurveyKB,
}

impl LiteratureSurveyAgent {
    pub fn new() -> Self {
        Self {
            kb: LiteratureSurveyKB::new(),
        }
    }

    /// Run a survey (skill-invoked or Ollama-delegated)
    /// This is a placeholder - actual execution happens via skill or Ollama
    pub fn survey_domain(&self, domain: &str) -> Result<String, String> {
        Ok(format!(
            "Survey for '{}' should be run via /literature-survey skill or Ollama delegation",
            domain
        ))
    }
}
