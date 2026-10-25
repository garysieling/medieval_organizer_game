//! PR + conflict resolution flows.
//! PR Agent: creates/reviews/merges PRs. Conflict Agent: resolves blocked merges.
//! Works on worktrees created by WorktreeManager.

use crate::infra::worktree::{Worktree, WorktreeManager};

/// A pull request from an agent's worktree
#[derive(Debug)]
pub struct PullRequest {
    pub id: String,
    pub agent: String,
    pub branch: String,
    pub title: String,
    pub status: PrStatus,
    pub checklist: Vec<(String, bool)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrStatus {
    Open,
    UnderReview,
    Approved,
    ChangesRequested,
    Merged,
    Closed,
}

/// Review checklist items (from PR Agent spec)
pub fn review_checklist() -> Vec<(String, bool)> {
    vec![
        ("builds clean".into(), false),
        ("tests pass".into(), false),
        ("docs updated".into(), false),
        ("no secrets".into(), false),
        ("commit style ok".into(), false),
        ("diff reviewable".into(), false),
    ]
}

/// PR Agent: create a PR for an agent's work
pub fn create_pr(agent: &str, branch: &str, title: &str) -> PullRequest {
    PullRequest {
        id: format!("pr_{}", title.replace(' ', "_").to_lowercase()),
        agent: agent.into(),
        branch: branch.into(),
        title: title.into(),
        status: PrStatus::Open,
        checklist: review_checklist(),
    }
}

/// PR Agent: review a PR (mark checklist items)
pub fn review_pr(pr: &mut PullRequest, results: &[(usize, bool)]) {
    for (i, passed) in results {
        if let Some(item) = pr.checklist.get_mut(*i) {
            item.1 = *passed;
        }
    }
    pr.status = if pr.checklist.iter().all(|(_, ok)| *ok) {
        PrStatus::Approved
    } else {
        PrStatus::ChangesRequested
    };
}

/// PR Agent: merge if approved
pub fn merge_pr(pr: &mut PullRequest) -> bool {
    if pr.status == PrStatus::Approved {
        pr.status = PrStatus::Merged;
        return true;
    }
    false
}

/// Conflict Agent: detect overlapping file changes between two worktrees
pub fn find_conflicts(mgr: &WorktreeManager, a: &Worktree, b: &Worktree) -> Vec<String> {
    mgr.detect_conflicts(a, b).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_lifecycle() {
        let mut pr = create_pr("impl-eng", "agent/impl-eng/task_1", "Add feature");
        assert_eq!(pr.status, PrStatus::Open);
        review_pr(&mut pr, &[(0, true), (1, true), (2, true), (3, true), (4, true), (5, true)]);
        assert_eq!(pr.status, PrStatus::Approved);
        assert!(merge_pr(&mut pr));
        assert_eq!(pr.status, PrStatus::Merged);
    }

    #[test]
    fn test_pr_rejected() {
        let mut pr = create_pr("impl-eng", "agent/impl-eng/task_2", "Bad change");
        review_pr(&mut pr, &[(0, false), (1, true), (2, true), (3, true), (4, true), (5, true)]);
        assert_eq!(pr.status, PrStatus::ChangesRequested);
        assert!(!merge_pr(&mut pr)); // can't merge unapproved
    }
}
