//! Simple Manager Bot prototype
//!
//! This module provides a basic Manager Bot that can:
//! - Read tasks from the task queue
//! - Assign tasks to agents
//! - Track task completion
//! - Log assignments and completions

use crate::infra::task_queue::{Task, TaskQueue, TaskStatus};
use std::fs::OpenOptions;
use std::io::Write;

/// Agent type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentType {
    ImplementationEngineer,
    PedanticSeniorEngineer,
    PerformanceEngineer,
    ExploratoryTesterDataScientist,
    ExploratoryTesterUIClicker,
    SimplePlaytester,
    FullGamePlaytester,
    FunInspector,
    RetrospectiveAgent,
    EconomistTester,
    CredibilityAnalyst,
    CreativeFilter,
    EmergentEffectsResearcher,
    AestheticConsultant,
    GameplayConsultant,
    ManagerBot,
    PRAgent,
    ConflictResolutionAgent,
    LiteratureSurveyAgent,
    CreativityGenerator,
}

impl AgentType {
    /// Get agent name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentType::ImplementationEngineer => "implementation-engineer",
            AgentType::PedanticSeniorEngineer => "pedantic-senior-engineer",
            AgentType::PerformanceEngineer => "performance-engineer",
            AgentType::ExploratoryTesterDataScientist => "exploratory-tester-data-scientist",
            AgentType::ExploratoryTesterUIClicker => "exploratory-tester-ui-clicker",
            AgentType::SimplePlaytester => "simple-playtester",
            AgentType::FullGamePlaytester => "full-game-playtester",
            AgentType::FunInspector => "fun-inspector",
            AgentType::RetrospectiveAgent => "retrospective-agent",
            AgentType::EconomistTester => "economist-tester",
            AgentType::CredibilityAnalyst => "credibility-analyst",
            AgentType::CreativeFilter => "creative-filter",
            AgentType::EmergentEffectsResearcher => "emergent-effects-researcher",
            AgentType::AestheticConsultant => "aesthetic-consultant",
            AgentType::GameplayConsultant => "gameplay-consultant",
            AgentType::ManagerBot => "manager-bot",
            AgentType::PRAgent => "pr-agent",
            AgentType::ConflictResolutionAgent => "conflict-resolution-agent",
            AgentType::LiteratureSurveyAgent => "literature-survey-agent",
            AgentType::CreativityGenerator => "creativity-generator",
        }
    }
}

/// Manager Bot for task coordination
pub struct ManagerBot {
    task_queue: TaskQueue,
    log_file: String,
    /// Agent capability tracking - which agents can do which task types
    agent_capabilities: std::collections::HashMap<AgentType, Vec<String>>,
}

impl ManagerBot {
    /// Create a new Manager Bot
    pub fn new(task_queue_path: &str, log_file: &str) -> Self {
        let mut agent_capabilities = std::collections::HashMap::new();

        // Define agent capabilities based on their types
        agent_capabilities.insert(
            AgentType::ImplementationEngineer,
            vec![
                "code-implementation".to_string(),
                "bug-fix".to_string(),
                "feature-development".to_string(),
                "testing".to_string(),
                "refactoring".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::PedanticSeniorEngineer,
            vec![
                "code-review".to_string(),
                "refactoring".to_string(),
                "architectural-analysis".to_string(),
                "complexity-analysis".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::PerformanceEngineer,
            vec![
                "performance-analysis".to_string(),
                "optimization".to_string(),
                "profiling".to_string(),
                "resource-management".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::ExploratoryTesterDataScientist,
            vec![
                "data-analysis".to_string(),
                "log-analysis".to_string(),
                "telemetry-analysis".to_string(),
                "pattern-detection".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::ExploratoryTesterUIClicker,
            vec![
                "ui-testing".to_string(),
                "ux-testing".to_string(),
                "edge-case-testing".to_string(),
                "user-flow-testing".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::SimplePlaytester,
            vec![
                "goal-testing".to_string(),
                "regression-testing".to_string(),
                "functional-testing".to_string(),
                "feature-testing".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::FullGamePlaytester,
            vec![
                "simulation-testing".to_string(),
                "balance-testing".to_string(),
                "long-term-testing".to_string(),
                "end-to-end-testing".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::FunInspector,
            vec![
                "fun-analysis".to_string(),
                "playability-assessment".to_string(),
                "engagement-analysis".to_string(),
                "pacing-analysis".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::RetrospectiveAgent,
            vec![
                "retrospective".to_string(),
                "issue-analysis".to_string(),
                "systemic-issue-detection".to_string(),
                "improvement-proposal".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::EconomistTester,
            vec![
                "economic-testing".to_string(),
                "balance-testing".to_string(),
                "economic-simulation".to_string(),
                "resource-analysis".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::CredibilityAnalyst,
            vec![
                "research-validation".to_string(),
                "source-evaluation".to_string(),
                "credibility-assessment".to_string(),
                "academic-analysis".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::CreativeFilter,
            vec![
                "creative-application".to_string(),
                "cross-domain-analysis".to_string(),
                "mechanic-borrowing".to_string(),
                "innovation-proposal".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::EmergentEffectsResearcher,
            vec![
                "emergent-system-research".to_string(),
                "complexity-analysis".to_string(),
                "emergent-mechanic-proposal".to_string(),
                "system-architecture".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::AestheticConsultant,
            vec![
                "aesthetic-analysis".to_string(),
                "visual-feedback".to_string(),
                "design-consultation".to_string(),
                "art-direction".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::GameplayConsultant,
            vec![
                "gameplay-analysis".to_string(),
                "game-design-feedback".to_string(),
                "player-experience".to_string(),
                "mechanic-evaluation".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::ManagerBot,
            vec![
                "orchestration".to_string(),
                "task-delegation".to_string(),
                "sprint-planning".to_string(),
                "agent-coordination".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::PRAgent,
            vec![
                "pr-management".to_string(),
                "code-review".to_string(),
                "merge-management".to_string(),
                "workflow-coordination".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::ConflictResolutionAgent,
            vec![
                "conflict-resolution".to_string(),
                "merge-conflict-handling".to_string(),
                "integration-management".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::LiteratureSurveyAgent,
            vec![
                "literature-review".to_string(),
                "research-analysis".to_string(),
                "academic-sourcing".to_string(),
                "domain-research".to_string(),
            ],
        );
        agent_capabilities.insert(
            AgentType::CreativityGenerator,
            vec![
                "idea-generation".to_string(),
                "name-generation".to_string(),
                "plot-generation".to_string(),
                "worldbuilding-ideas".to_string(),
            ],
        );

        Self {
            task_queue: TaskQueue::new(task_queue_path),
            log_file: log_file.to_string(),
            agent_capabilities,
        }
    }

    /// Log a message to the log file
    fn log(&self, message: &str) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&self.log_file)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        writeln!(file, "{}", message)
            .map_err(|e| format!("Failed to write to log: {}", e))?;

        Ok(())
    }

    /// Read pending tasks from the queue
    pub fn read_pending_tasks(&self) -> Result<Vec<Task>, String> {
        let tasks = self.task_queue.list_pending()?;
        self.log(&format!("Read {} pending tasks", tasks.len()))?;
        Ok(tasks)
    }

    /// Read ready tasks (pending tasks whose dependencies are all complete)
    pub fn read_ready_tasks(&self) -> Result<Vec<Task>, String> {
        let tasks = self.task_queue.list_ready()?;
        self.log(&format!("Read {} ready tasks", tasks.len()))?;
        Ok(tasks)
    }

    /// Assign a task to an agent
    pub fn assign_task(&self, task_id: &str, agent: &str) -> Result<(), String> {
        self.task_queue.assign_task(task_id, agent)?;
        self.log(&format!("Assigned task {} to agent {}", task_id, agent))?;
        Ok(())
    }

    /// Determine which agent can handle a task type.
    /// Specialist agents are checked first so a task like "Write tests"
    /// goes to a tester, not implementation-engineer (which also lists "testing").
    fn find_agent_for_task(&self, task_type: &str) -> Option<AgentType> {
        // Priority order: specialists first, generalist impl-engineer last
        let priority = [
            AgentType::ExploratoryTesterDataScientist,
            AgentType::ExploratoryTesterUIClicker,
            AgentType::SimplePlaytester,
            AgentType::FullGamePlaytester,
            AgentType::FunInspector,
            AgentType::EconomistTester,
            AgentType::PerformanceEngineer,
            AgentType::PedanticSeniorEngineer,
            AgentType::CredibilityAnalyst,
            AgentType::CreativeFilter,
            AgentType::CreativityGenerator,
            AgentType::EmergentEffectsResearcher,
            AgentType::AestheticConsultant,
            AgentType::GameplayConsultant,
            AgentType::RetrospectiveAgent,
            AgentType::LiteratureSurveyAgent,
            AgentType::PRAgent,
            AgentType::ConflictResolutionAgent,
            AgentType::ImplementationEngineer, // generalist fallback, checked last
        ];

        for agent_type in &priority {
            if let Some(caps) = self.agent_capabilities.get(agent_type) {
                // Fuzzy match: "testing" matches "goal-testing", "regression-testing", etc.
                if caps.iter().any(|c| c.contains(task_type) || task_type.contains(c.as_str())) {
                    return Some(agent_type.clone());
                }
            }
        }
        None
    }

    /// Auto-assign all ready tasks to appropriate agents based on capabilities
    /// Uses capability matching and respects task dependencies
    pub fn auto_assign_tasks(&self, _agents: &[&str]) -> Result<Vec<String>, String> {
        let pending = self.read_ready_tasks()?;
        let mut assignments = Vec::new();

        for task in pending {
            // Infer task type from description (simple heuristic)
            let task_type = self.infer_task_type(&task.description);

            // Find appropriate agent for this task type
            match self.find_agent_for_task(&task_type) {
                Some(agent_type) => {
                    let agent_name = agent_type.as_str();
                    self.assign_task(&task.id, agent_name)?;
                    assignments.push(format!("{} -> {} ({})", task.id, agent_name, task_type));
                }
                None => {
                    // Fallback to Implementation Engineer for unknown task types
                    self.assign_task(&task.id, AgentType::ImplementationEngineer.as_str())?;
                    assignments.push(format!("{} -> implementation-engineer (fallback)", task.id));
                }
            }
        }

        Ok(assignments)
    }

    /// Infer task type from description (simple heuristic)
    fn infer_task_type(&self, description: &str) -> String {
        let desc = description.to_lowercase();

        // Simple keyword matching for task type inference
        if desc.contains("test") || desc.contains("verify") {
            "testing".to_string()
        } else if desc.contains("implement") || desc.contains("build") || desc.contains("create") {
            "code-implementation".to_string()
        } else if desc.contains("refactor") || desc.contains("clean") || desc.contains("improve") {
            "refactoring".to_string()
        } else if desc.contains("research") || desc.contains("analyze") || desc.contains("review") {
            "research-validation".to_string()
        } else if desc.contains("design") || desc.contains("architect") {
            "architectural-analysis".to_string()
        } else if desc.contains("bug") || desc.contains("fix") {
            "bug-fix".to_string()
        } else if desc.contains("feature") {
            "feature-development".to_string()
        } else if desc.contains("performance") || desc.contains("optimize") {
            "performance-analysis".to_string()
        } else if desc.contains("data") || desc.contains("log") || desc.contains("telemetry") {
            "data-analysis".to_string()
        } else if desc.contains("ui") || desc.contains("ux") || desc.contains("interface") {
            "ui-testing".to_string()
        } else if desc.contains("play") || desc.contains("simulate") || desc.contains("balance") {
            "simulation-testing".to_string()
        } else {
            "code-implementation".to_string() // Default fallback
        }
    }

    /// Mark a task as complete
    pub fn mark_task_complete(&self, task_id: &str) -> Result<(), String> {
        self.task_queue.mark_complete(task_id)?;
        self.log(&format!("Marked task {} as complete", task_id))?;
        Ok(())
    }

    /// Get all tasks (any status)
    pub fn get_all_tasks(&self) -> Result<Vec<Task>, String> {
        self.task_queue.list_all()
    }

    /// Print task summary
    pub fn print_summary(&self) -> Result<(), String> {
        let tasks = self.get_all_tasks()?;

        println!("=== Task Summary ===");
        println!("Total tasks: {}", tasks.len());

        let pending = tasks.iter().filter(|t| t.status == TaskStatus::Pending).count();
        let in_progress = tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count();
        let complete = tasks.iter().filter(|t| t.status == TaskStatus::Complete).count();

        println!("Pending: {}", pending);
        println!("In Progress: {}", in_progress);
        println!("Complete: {}", complete);
        println!("==================");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_paths(name: &str) -> (String, String, std::path::PathBuf, std::path::PathBuf) {
        let t = std::env::temp_dir().join(format!("mb_{}_tasks.txt", name));
        let l = std::env::temp_dir().join(format!("mb_{}.log", name));
        let _ = fs::remove_file(&t);
        let _ = fs::remove_file(&l);
        (t.to_str().unwrap().to_string(), l.to_str().unwrap().to_string(), t, l)
    }

    #[test]
    fn test_manager_bot_creation() {
        let (t, l, tp, lp) = tmp_paths("create");
        let _bot = ManagerBot::new(&t, &l);
        let _ = fs::remove_file(&tp);
        let _ = fs::remove_file(&lp);
    }

    #[test]
    fn test_read_pending_tasks() {
        let (t, l, tp, lp) = tmp_paths("pending");
        let queue = TaskQueue::new(&t);
        queue.add_task("Test task").unwrap();
        let bot = ManagerBot::new(&t, &l);
        let pending = bot.read_pending_tasks().unwrap();
        assert_eq!(pending.len(), 1);
        let _ = fs::remove_file(&tp);
        let _ = fs::remove_file(&lp);
    }

    #[test]
    fn test_assign_task() {
        let (t, l, tp, lp) = tmp_paths("assign");
        let queue = TaskQueue::new(&t);
        let task = queue.add_task("Test task").unwrap();
        let bot = ManagerBot::new(&t, &l);
        bot.assign_task(&task.id, "Test Agent").unwrap();
        let tasks = bot.get_all_tasks().unwrap();
        let updated_task = tasks.iter().find(|t| t.id == task.id).unwrap();
        assert_eq!(updated_task.assigned_to, "Test Agent");
        assert_eq!(updated_task.status, TaskStatus::InProgress);
        let _ = fs::remove_file(&tp);
        let _ = fs::remove_file(&lp);
    }

    #[test]
    fn test_auto_assign_tasks() {
        let (t, l, tp, lp) = tmp_paths("auto");
        let queue = TaskQueue::new(&t);
        queue.add_task("Task 1").unwrap();
        queue.add_task("Task 2").unwrap();
        queue.add_task("Task 3").unwrap();
        let bot = ManagerBot::new(&t, &l);
        let agents = vec!["Agent A", "Agent B"];
        let assignments = bot.auto_assign_tasks(&agents).unwrap();
        assert_eq!(assignments.len(), 3);
        let _ = fs::remove_file(&tp);
        let _ = fs::remove_file(&lp);
    }

    #[test]
    fn test_creativity_generator_routing() {
        let (t, l, tp, lp) = tmp_paths("creativity");
        let bot = ManagerBot::new(&t, &l);
        assert_eq!(bot.find_agent_for_task("name-generation"), Some(AgentType::CreativityGenerator));
        assert_eq!(bot.find_agent_for_task("plot-generation"), Some(AgentType::CreativityGenerator));
        assert_eq!(AgentType::CreativityGenerator.as_str(), "creativity-generator");
        let _ = fs::remove_file(&tp);
        let _ = fs::remove_file(&lp);
    }
}

/// A sprint plan the Manager Bot builds and tracks
#[derive(Debug)]
pub struct SprintPlan {
    pub name: String,
    pub goal: String,
    pub tasks: Vec<SprintTask>,
    pub exit_criteria: Vec<String>,
    pub approved: bool,
}

#[derive(Debug)]
pub struct SprintTask {
    pub id: String,
    pub description: String,
    pub workstream: String,
    pub completed: bool,
}

impl SprintPlan {
    /// Check if all exit criteria are met (simple heuristic: all tasks done)
    pub fn exit_criteria_met(&self) -> bool {
        self.tasks.iter().all(|t| t.completed) && self.approved
    }

    /// Progress as percentage
    pub fn progress(&self) -> f32 {
        if self.tasks.is_empty() { return 0.0; }
        let done = self.tasks.iter().filter(|t| t.completed).count() as f32;
        done / self.tasks.len() as f32 * 100.0
    }

    /// Mark a task complete by id
    pub fn complete_task(&mut self, id: &str) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            t.completed = true;
        }
    }
}

/// Build a sprint plan from workstreams and tasks
pub fn build_sprint_plan(name: &str, goal: &str) -> SprintPlan {
    SprintPlan {
        name: name.into(),
        goal: goal.into(),
        tasks: vec![],
        exit_criteria: vec![],
        approved: false,
    }
}
