//! Git worktree management for parallel agent work
//!
//! Each agent gets an isolated worktree so parallel work doesn't conflict.
//! Uses `git worktree` under the hood.

use std::path::Path;
use std::process::Command;

/// Manages git worktrees for agent isolation
pub struct WorktreeManager {
    /// Root directory containing the main repo
    repo_root: String,
    /// Directory where worktrees are created
    worktree_root: String,
}

/// Represents a single worktree assigned to an agent
#[derive(Debug, Clone)]
pub struct Worktree {
    /// Agent this worktree belongs to
    pub agent: String,
    /// Branch name checked out in this worktree
    pub branch: String,
    /// Filesystem path of the worktree
    pub path: String,
}

impl WorktreeManager {
    /// Create a new worktree manager
    pub fn new(repo_root: &str, worktree_root: &str) -> Self {
        Self {
            repo_root: repo_root.to_string(),
            worktree_root: worktree_root.to_string(),
        }
    }

    /// Run a git command in the repo root
    fn git(&self, args: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_root)
            .output()
            .map_err(|e| format!("Failed to run git: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Create a worktree for an agent on a new branch
    pub fn create_worktree(&self, agent: &str, task_id: &str) -> Result<Worktree, String> {
        let branch = format!("agent/{}/{}", agent, task_id);
        let path = format!("{}/{}-{}", self.worktree_root, agent, task_id);

        if Path::new(&path).exists() {
            return Err(format!("Worktree already exists: {}", path));
        }

        self.git(&["worktree", "add", "-b", &branch, &path])?;

        Ok(Worktree {
            agent: agent.to_string(),
            branch,
            path,
        })
    }

    /// Remove a worktree (and optionally delete its branch)
    pub fn remove_worktree(&self, worktree: &Worktree, delete_branch: bool) -> Result<(), String> {
        self.git(&["worktree", "remove", "--force", &worktree.path])?;
        if delete_branch {
            self.git(&["branch", "-D", &worktree.branch])?;
        }
        Ok(())
    }

    /// List all active worktrees
    pub fn list_worktrees(&self) -> Result<Vec<Worktree>, String> {
        let out = self.git(&["worktree", "list", "--porcelain"])?;
        let mut worktrees = Vec::new();
        let mut path = String::new();
        let mut branch = String::new();

        for line in out.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = p.to_string();
            } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
                branch = b.to_string();
            } else if line.is_empty() && !path.is_empty() {
                let agent = branch
                    .strip_prefix("agent/")
                    .and_then(|s| s.split('/').next())
                    .unwrap_or("unknown")
                    .to_string();
                worktrees.push(Worktree {
                    agent,
                    branch: branch.clone(),
                    path: path.clone(),
                });
                path.clear();
                branch.clear();
            }
        }
        if !path.is_empty() {
            worktrees.push(Worktree {
                agent: "main".to_string(),
                branch,
                path,
            });
        }
        Ok(worktrees)
    }

    /// Check if two worktrees have conflicting changes to the same files
    pub fn detect_conflicts(&self, worktree_a: &Worktree, worktree_b: &Worktree) -> Result<Vec<String>, String> {
        // Files changed on branch A
        let files_a = self.git(&[
            "diff", "--name-only", "master", &worktree_a.branch,
        ])?;
        // Files changed on branch B
        let files_b = self.git(&[
            "diff", "--name-only", "master", &worktree_b.branch,
        ])?;

        let set_a: std::collections::HashSet<&str> = files_a.lines().collect();
        let set_b: std::collections::HashSet<&str> = files_b.lines().collect();

        Ok(set_a.intersection(&set_b).map(|s| s.to_string()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_manager_creation() {
        let mgr = WorktreeManager::new(".", ".worktrees");
        assert_eq!(mgr.repo_root, ".");
        assert_eq!(mgr.worktree_root, ".worktrees");
    }
}
