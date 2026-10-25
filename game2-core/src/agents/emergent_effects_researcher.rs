//! Emergent Effects Researcher: finds unexpected system interactions.
//! Detects when two systems produce behavior neither was designed for.

/// An emergent effect discovered
#[derive(Debug)]
pub struct EmergentEffect {
    pub systems: Vec<String>,      // which systems interacted
    pub description: String,
    pub novelty: f32,              // how unexpected
    pub example: String,           // what happened
}

/// Emergent Effects Researcher agent
pub struct EmergentEffectsResearcher {
    pub name: String,
}

impl EmergentEffectsResearcher {
    pub fn new() -> Self {
        Self { name: "Emergent Effects Researcher".into() }
    }

    /// Analyze a sim run for emergent interactions
    pub fn analyze(&self, sim_log: &[&str]) -> Vec<EmergentEffect> {
        let mut effects = vec![];
        let log = sim_log.join(" ");
        // Look for known emergent patterns
        if log.contains("guild") && log.contains("influence") {
            effects.push(EmergentEffect {
                systems: vec!["guilds".into(), "organizing".into()],
                description: "Guild forming created organizing hub".into(),
                novelty: 0.8,
                example: "Merchants organized -> became recruitment target".into(),
            });
        }
        if log.contains("illness") && log.contains("faith") {
            effects.push(EmergentEffect {
                systems: vec!["illness".into(), "religion".into()],
                description: "Illness drove faith participation".into(),
                novelty: 0.7,
                example: "Sick villagers sought healing via congregation".into(),
            });
        }
        if log.contains("attachment") && log.contains("feud") {
            effects.push(EmergentEffect {
                systems: vec!["psychology".into(), "feuds".into()],
                description: "Insecure attachment fueled feuds".into(),
                novelty: 0.9,
                example: "Anxious villagers escalated conflicts".into(),
            });
        }
        if log.contains("weather") && log.contains("economy") {
            effects.push(EmergentEffect {
                systems: vec!["weather".into(), "economics".into()],
                description: "Weather shocks moved markets".into(),
                novelty: 0.6,
                example: "Drought spiked food prices".into(),
            });
        }
        effects
    }

    /// Rate how emergent a discovered effect is
    pub fn rate_novelty(&self, systems: &[&str]) -> f32 {
        // More systems interacting = more emergent
        systems.len() as f32 * 0.3
    }
}
