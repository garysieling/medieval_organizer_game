//! Conflict Resolution Agent: resolves merge conflicts and integration issues.
//!
//! Infrastructure for conflict resolution - conflict detection, resolution strategies.
//! Intelligence layer runs as Claude Code skill or Ollama.

/// A merge conflict detected
#[derive(Debug, Clone)]
pub struct MergeConflict {
    pub file_path: String,
    pub conflict_type: ConflictType,
    pub agents_involved: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictType {
    Simple,       // Non-overlapping, easy to resolve
    Complex,      // Overlapping logic, needs analysis
    Structural,   // Architectural disagreement
}

/// Resolution strategy
#[derive(Debug, Clone)]
pub struct ConflictResolution {
    pub strategy: ResolutionStrategy,
    pub description: String,
    pub requires_testing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolutionStrategy {
    KeepBoth,     // Both changes are compatible
    MergeLogic,   // Combine both approaches
    ChooseBest,   // Pick one version
    Redesign,     // Need architectural change
    Escalate,     // Unclear, need manager/human input
}

/// Conflict Resolution Agent
pub struct ConflictResolutionAgent {
    pub name: String,
}

impl ConflictResolutionAgent {
    pub fn new() -> Self {
        Self {
            name: "Conflict Resolution Agent".into(),
        }
    }

    /// Classify conflict type (skill does actual resolution)
    pub fn classify_conflict(&self, overlapping_lines: usize) -> ConflictType {
        match overlapping_lines {
            0..=5 => ConflictType::Simple,
            6..=20 => ConflictType::Complex,
            _ => ConflictType::Structural,
        }
    }

    /// Suggest resolution strategy
    pub fn suggest_strategy(&self, conflict_type: ConflictType) -> ResolutionStrategy {
        match conflict_type {
            ConflictType::Simple => ResolutionStrategy::KeepBoth,
            ConflictType::Complex => ResolutionStrategy::MergeLogic,
            ConflictType::Structural => ResolutionStrategy::Escalate,
        }
    }
}
