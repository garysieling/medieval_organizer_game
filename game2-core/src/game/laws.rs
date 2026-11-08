//! The realm's statutes and the lord's own - a `LawBook` reads the
//! crown's law at the base and the local lord's override on top, so a
//! village under a kind liege pays the fifth while a harsh one's pays
//! more. A won campaign writes its issue's scope into the book: a realm
//! grievance changes the crown's line, a local one the lord's override.

use crate::engine::det::HashMap;

/// What the law says about the things the village does: the share the
/// lord takes, how many may assemble, what the drink levy is, and the
/// writ's own weight. The realm's `realm` is the base; a jurisdiction's
/// override sits over it - a field the override doesn't set keeps the
/// realm's word.
#[derive(Clone, Copy, Debug)]
pub struct Law {
    /// The lord's cut of what's made: the fifth is the crown's old word
    pub tax_share: f32,
    /// How many may assemble before the law calls it a mob
    pub assembly_cap: u32,
    /// The castle's share of every barrel that pours
    pub drink_levy: f32,
    /// How hard a writ lands: 1.0 is the crown's ordinary weight
    pub writ_mult: f32,
}

impl Default for Law {
    fn default() -> Self {
        Self { tax_share: 0.20, assembly_cap: 2, drink_levy: 0.25, writ_mult: 1.0 }
    }
}

/// The law: the crown's realm-wide word, and the lords' overrides where
/// a lord holds a different line. Jurisdictions key to site index - the
/// castle the village answers to is its liege, and its law is the
/// realm's merged with that lord's own.
#[derive(Debug, Default)]
pub struct LawBook {
    pub realm: Law,
    pub overrides: HashMap<usize, Law>,
}

impl LawBook {
    pub fn new() -> Self { Self::default() }

    /// The law a jurisdiction stands under: the realm's word, and the
    /// lord's own where he's set one
    pub fn law_for(&self, site: usize) -> Law {
        self.overrides.get(&site).copied().unwrap_or(self.realm)
    }
}
