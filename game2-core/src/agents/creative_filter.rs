//! Creative Filter: generates wild ideas, filters down to keepers.
//! From spec: diverse ideas (10x) -> filter to practical implementations (1x).

/// A wild idea for a mechanic/system
#[derive(Debug)]
pub struct WildIdea {
    pub name: String,
    pub category: String,        // mechanic, theme, emergent, social, magic
    pub description: String,
    pub wildness: f32,           // 0-1, how out-there
    pub feasibility: f32,        // 0-1, how buildable
    pub vision_fit: f32,         // 0-1, matches Gary's vision
}

/// A filtered idea (practical implementation)
#[derive(Debug)]
pub struct KeeperIdea {
    pub name: String,
    pub description: String,
    pub effort: u32,            // hours estimate
    pub score: f32,             // wildness * feasibility * vision_fit
}

/// Creative Filter agent
pub struct CreativeFilter {
    pub name: String,
    pub generation_rate: usize,   // ideas per run
    pub filter_ratio: f32,        // fraction that survive
}

impl CreativeFilter {
    pub fn new() -> Self {
        Self { name: "Creative Filter".into(), generation_rate: 20, filter_ratio: 0.15 }
    }

    /// Generate wild ideas (diverse, unusual)
    pub fn generate(&self, count: usize) -> Vec<WildIdea> {
        let templates = vec![
            ("villagers gossip about the player", "social", 0.4),
            ("pets get sick too", "emergent", 0.3),
            ("weather changes villager mood", "emergent", 0.4),
            ("contracts can be inherited", "mechanic", 0.5),
            ("textiles have texture-sensitive weavers", "mechanic", 0.6),
            ("organizing a guild triggers merchant rivalry", "emergent", 0.5),
            ("the player's reputation travels ahead of them", "social", 0.5),
            ("magic emerges from reading books", "magic", 0.7),
            ("illness spreads through gossip", "emergent", 0.6),
            ("aging villagers gain wisdom but lose strength", "mechanic", 0.4),
        ];
        templates.iter().take(count).map(|(name, cat, wild)| WildIdea {
            name: name.to_string(),
            category: cat.to_string(),
            description: format!("Implement {}", name),
            wildness: *wild,
            feasibility: 1.0 - wild,
            vision_fit: if *wild > 0.5 { 0.7 } else { 0.5 },
        }).collect()
    }

    /// Filter wild ideas to keepers (highest score)
    pub fn filter(&self, ideas: Vec<WildIdea>) -> Vec<KeeperIdea> {
        let mut scored: Vec<_> = ideas.iter().map(|i| (i.name.clone(), i.description.clone(),
            i.wildness * i.feasibility * i.vision_fit)).collect();
        scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        let keep_count = (scored.len() as f32 * self.filter_ratio) as usize;
        scored.into_iter().take(keep_count.max(1)).map(|(name, desc, score)| KeeperIdea {
            name, description: desc, effort: 4, score,
        }).collect()
    }
}
