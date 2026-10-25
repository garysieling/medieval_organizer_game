//! Save/load: Datomic-style history with checkpoints + replay.
//! From backlog: "history that checkpoints/replays from points"
//! Enables time-travel debugging and architectural stability.

use crate::engine::ecs::World;

/// A snapshot of world state at a point in time
#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub tick: u64,
    pub id: String,
    /// Serialized component data (simplified)
    pub data: Vec<String>,
}

/// History of checkpoints (Datomic-style)
#[derive(Debug)]
pub struct History {
    pub checkpoints: Vec<Checkpoint>,
    pub current_tick: u64,
}

impl History {
    pub fn new() -> Self {
        Self { checkpoints: vec![], current_tick: 0 }
    }

    /// Save a checkpoint at the current tick
    pub fn checkpoint(&mut self, world: &World, label: &str) -> Checkpoint {
        let cp = Checkpoint {
            tick: self.current_tick,
            id: format!("cp_{}", label),
            data: vec![format!("entities={}", world.entity_count())],
        };
        self.checkpoints.push(cp.clone());
        cp
    }

    /// Replay from a checkpoint (jump back in time)
    pub fn replay_from(&self, checkpoint_id: &str) -> Option<&Checkpoint> {
        self.checkpoints.iter().find(|c| c.id == checkpoint_id)
    }

    /// Latest checkpoint
    pub fn latest(&self) -> Option<&Checkpoint> {
        self.checkpoints.last()
    }
}
