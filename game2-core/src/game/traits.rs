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
    Steady,      // slow to anger, keeps at the work (#1239)
    Wary,        // watches who's asking before they answer
    Stolid,      // keeps to themselves, unmoved by talk
    Cruel,       // hard hand, quick to wound
    SelfAbsorbed, // the conversation hog: talks at you, never asks back (#1252)
}

/// Every trait, in a fixed order (ties over "most common" settle on it)
pub const TRAITS: [Trait; 13] = [Trait::Lazy, Trait::Ambitious, Trait::Kind, Trait::Stingy,
    Trait::Pious, Trait::Hotheaded, Trait::Gregarious, Trait::Proud,
    Trait::Steady, Trait::Wary, Trait::Stolid, Trait::Cruel, Trait::SelfAbsorbed];

impl Trait {
    /// The trait that can't sit beside this one: a taught trait drives
    /// its opposite out
    pub fn opposite(self) -> Option<Trait> {
        match self {
            Trait::Steady => Some(Trait::Hotheaded),
            Trait::Hotheaded => Some(Trait::Steady),
            Trait::Kind => Some(Trait::Cruel),
            Trait::Cruel => Some(Trait::Kind),
            Trait::Lazy => Some(Trait::Ambitious),
            Trait::Ambitious => Some(Trait::Lazy),
            Trait::Gregarious => Some(Trait::Stolid),
            Trait::Stolid => Some(Trait::Gregarious),
            _ => None,
        }
    }
    /// What a pupil takes from a teacher with this trait - mostly a copy,
    /// but a stolid master's pupil grows steady, and a cruel one's wary
    pub fn taught(self) -> Trait {
        match self {
            Trait::Stolid => Trait::Steady,
            Trait::Cruel => Trait::Wary,
            t => t,
        }
    }
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
            (Trait::Steady, "work") => 1.15,
            (Trait::Steady, "feud") => 0.6,
            (Trait::Wary, "recruit") => 0.7,
            (Trait::Wary, "trade") => 0.8,
            (Trait::Stolid, "social") => 0.6,
            (Trait::Stolid, "talk") => 0.8,
            (Trait::Cruel, "feud") => 1.5,
            (Trait::Cruel, "share") => 0.5,
            (Trait::Cruel, "talk") => 0.7,
            (Trait::SelfAbsorbed, "talk") => 1.3,
            (Trait::SelfAbsorbed, "social") => 1.2,
            _ => 1.0,
        };
    }
    modifier
}

/// Pick random-ish traits from seed
pub fn personality_from_seed(seed: u64) -> Personality {
    let all = [Trait::Lazy, Trait::Ambitious, Trait::Kind, Trait::Stingy, Trait::Pious, Trait::Hotheaded, Trait::Gregarious, Trait::Proud];
    let n = ((seed % 3) + 1) as usize; // 1-3 traits
    let mut traits: Vec<Trait> = all.iter().cycle().skip((seed % 8) as usize).take(n).copied().collect();
    // the later temperaments roll apart, so the first eight stay as they were
    let extra = match seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 59 {
        0 | 1 => Some(Trait::Steady),
        2 | 3 => Some(Trait::Wary),
        4 => Some(Trait::Stolid),
        5 => Some(Trait::Cruel),
        6 | 7 => Some(Trait::SelfAbsorbed),
        _ => None,
    };
    if let Some(t) = extra.filter(|t| t.opposite().map(|o| !traits.contains(&o)).unwrap_or(true)) {
        traits.push(t);
    }
    Personality { traits }
}
