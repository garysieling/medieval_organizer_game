//! Full Game Playtester: runs complete games via backend API.
//!
//! Infrastructure for long-term stability testing - 1000+ turn runs, mass simulations.
//! Intelligence layer runs as Claude Code skill or Ollama.

/// Configuration for a full game run
#[derive(Debug, Clone)]
pub struct GameRunConfig {
    pub max_ticks: u64,
    pub world_seed: u64,
    pub ai_players: usize,
    pub fast_mode: bool,  // Skip rendering, maximum speed
}

/// Result of a full game run
#[derive(Debug, Clone)]
pub struct GameRunResult {
    pub run_id: String,
    pub config: GameRunConfig,
    pub ticks_completed: u64,
    pub outcome: GameOutcome,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub final_state: String,  // JSON or description
    pub duration_secs: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameOutcome {
    Completed,      // Reached max ticks
    Victory,        // Player/AI won
    Defeat,         // Player/AI lost
    Crashed,        // Game crashed
    Timeout,        // Exceeded time limit
}

/// Aggregated results from mass simulation
#[derive(Debug, Clone)]
pub struct MassSimulationResult {
    pub total_runs: usize,
    pub completed: usize,
    pub crashed: usize,
    pub avg_duration_secs: f64,
    pub common_errors: Vec<(String, usize)>,  // Error message, count
}

/// Full Game Playtester
pub struct FullGamePlaytester {
    pub name: String,
    pub use_backend_api: bool,  // true = no UI, faster
}

impl FullGamePlaytester {
    pub fn new() -> Self {
        Self {
            name: "Full Game Playtester".into(),
            use_backend_api: true,
        }
    }

    /// Run a single full game
    pub fn run_game(&self, config: GameRunConfig) -> GameRunResult {
        // Placeholder: actual implementation runs game via backend API
        GameRunResult {
            run_id: format!("run_{}", config.world_seed),
            config: config.clone(),
            ticks_completed: config.max_ticks,
            outcome: GameOutcome::Completed,
            errors: vec![],
            warnings: vec![],
            final_state: "Placeholder final state".into(),
            duration_secs: 0.0,
        }
    }

    /// Run mass simulation (e.g., 1000 games overnight)
    pub fn run_mass_simulation(&self, configs: Vec<GameRunConfig>) -> MassSimulationResult {
        let results: Vec<GameRunResult> = configs.into_iter().map(|c| self.run_game(c)).collect();

        let total = results.len();
        let completed = results.iter().filter(|r| r.outcome == GameOutcome::Completed).count();
        let crashed = results.iter().filter(|r| r.outcome == GameOutcome::Crashed).count();
        let avg_duration = results.iter().map(|r| r.duration_secs).sum::<f64>() / total.max(1) as f64;

        MassSimulationResult {
            total_runs: total,
            completed,
            crashed,
            avg_duration_secs: avg_duration,
            common_errors: vec![],  // TODO: aggregate error counts
        }
    }
}
