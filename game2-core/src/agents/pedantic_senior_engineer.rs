//! Pedantic Senior Engineer: periodic complexity refactors.
//! Runs every N turns, finds smells, refactors along seams, proposes patterns.

/// A code smell finding
#[derive(Debug)]
pub struct Finding {
    pub severity: Severity,
    pub location: String,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Critical,
    Should,
    Nitpick,
}

/// A refactor proposal
#[derive(Debug)]
pub struct Refactor {
    pub name: String,
    pub motivation: String,     // problem it solves
    pub seam: String,           // natural boundary to split at
    pub estimated_effort: u32,  // hours
    pub pattern: Option<String>,// design pattern introduced
}

/// Pedantic Senior Engineer agent
pub struct PedanticEngineer {
    pub name: String,
    pub review_cadence: u32,    // runs every N turns
    pub turns_since_review: u32,
}

impl PedanticEngineer {
    pub fn new() -> Self {
        Self {
            name: "Pedantic Senior Engineer".into(),
            review_cadence: 10,
            turns_since_review: 0,
        }
    }

    /// Should this agent run this turn?
    pub fn should_review(&mut self) -> bool {
        self.turns_since_review += 1;
        if self.turns_since_review >= self.review_cadence {
            self.turns_since_review = 0;
            return true;
        }
        false
    }

    /// Analyze a file for smells (simple heuristics)
    pub fn review(&self, filename: &str, lines: usize, complexity: f32) -> Vec<Finding> {
        let mut findings = vec![];
        if lines > 300 {
            findings.push(Finding {
                severity: Severity::Should,
                location: filename.into(),
                description: format!("File is {} lines - too long", lines),
                suggestion: "Split along natural module boundaries".into(),
            });
        }
        if complexity > 0.7 {
            findings.push(Finding {
                severity: Severity::Critical,
                location: filename.into(),
                description: format!("High complexity ({:.1})", complexity),
                suggestion: "Extract functions, reduce nesting, introduce pattern".into(),
            });
        }
        findings
    }

    /// Propose a refactor
    pub fn propose(&self, name: &str, motivation: &str, seam: &str, effort: u32, pattern: Option<&str>) -> Refactor {
        Refactor {
            name: name.into(),
            motivation: motivation.into(),
            seam: seam.into(),
            estimated_effort: effort,
            pattern: pattern.map(|p| p.to_string()),
        }
    }
}
