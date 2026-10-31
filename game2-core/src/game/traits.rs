//! Traits: villager personalities that shape behavior.
//! Each villager has traits that emerge in how they act/react.


/// A personality trait
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trait {
    Lazy,        // works less, idles more
    Ambitious,   // seeks advancement, joins orgs eagerly
    Kind,        // helps others, forms bonds faster
    Stingy,      // hoards, less likely to share/trade
    Pious,       // drawn to religion
    Hotheaded,   // quick to feud
    Gregarious,  // seeks social interaction
    Proud,       // resistant to recruitment (high bar)
}

/// A villager's personality - a set of traits
#[derive(Debug)]
pub struct Personality {
    pub traits: Vec<Trait>,
}

/// How traits modify a behavior
pub fn modifies(p: &Personality, behavior: &str) -> f32 {
    let mut modifier = 1.0;
    for t in &p.traits {
        modifier *= match (t, behavior) {
            (Trait::Lazy, "work") => 0.6,
            (Trait::Ambitious, "recruit") => 1.5,
            (Trait::Ambitious, "work") => 1.3,
            (Trait::Kind, "talk") => 1.4,
            (Trait::Kind, "share") => 1.5,
            (Trait::Stingy, "share") => 0.4,
            (Trait::Stingy, "trade") => 0.6,
            (Trait::Pious, "faith") => 1.6,
            (Trait::Hotheaded, "feud") => 1.8,
            (Trait::Gregarious, "social") => 1.5,
            (Trait::Proud, "recruit") => 0.5,
            _ => 1.0,
        };
    }
    modifier
}

/// Pick random-ish traits from seed
pub fn personality_from_seed(seed: u64) -> Personality {
    let all = [Trait::Lazy, Trait::Ambitious, Trait::Kind, Trait::Stingy, Trait::Pious, Trait::Hotheaded, Trait::Gregarious, Trait::Proud];
    let n = ((seed % 3) + 1) as usize; // 1-3 traits
    Personality { traits: all.iter().cycle().skip((seed % 8) as usize).take(n).copied().collect() }
}
