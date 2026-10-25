//! Magic: "the world is written in an ancient language we don't know."
//! Ingest text sources (books, sagas, gutenberg) -> discover magic -> behaviors/mechanics emerge.

use std::collections::HashMap;

/// A magical discovery from text
#[derive(Debug, Clone)]
pub struct MagicDiscovery {
    pub source: String,           // which text revealed it
    pub name: String,             // what the magic is called
    pub domain: MagicDomain,
    pub description: String,
    pub strength: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MagicDomain {
    Healing,      // cures, regeneration
    Protection,   // wards, shields
    Prophecy,     // foresight, omens
    Elemental,    // fire, water, earth, air
    Spirit,       // souls, ancestors
    Transformation, // shape, form
    Binding,      // oaths, contracts
    Weather,      // storms, seasons
}

/// A grimoire: collected magical knowledge
#[derive(Debug)]
pub struct Grimoire {
    pub discoveries: Vec<MagicDiscovery>,
    pub domains_known: HashMap<String, usize>, // domain -> count
    pub total_power: f32,
}

/// Text sources that reveal magic
pub fn ancient_sources() -> Vec<(&'static str, MagicDomain, f32)> {
    vec![
        ("Icelandic sagas - binding oaths", MagicDomain::Binding, 0.6),
        ("Egyptian Book of the Dead - souls", MagicDomain::Spirit, 0.8),
        ("Hesiod Works and Days - seasons", MagicDomain::Weather, 0.5),
        ("Paracelsus - transformation", MagicDomain::Transformation, 0.7),
        ("Dante's Inferno - protection through descent", MagicDomain::Protection, 0.6),
        ("Medieval herbals - healing", MagicDomain::Healing, 0.5),
        ("Rumi - spirit and longing", MagicDomain::Spirit, 0.7),
        ("Tao Te Ching - elemental balance", MagicDomain::Elemental, 0.6),
    ]
}

/// Ingest a text source, discover magic
pub fn ingest_text(grimoire: &mut Grimoire, source: &str, domain: MagicDomain, strength: f32) -> MagicDiscovery {
    let name = format!("{:?} from {}", domain, source.split(" - ").next().unwrap_or(source));
    let d = MagicDiscovery {
        source: source.into(), name, domain, strength,
        description: format!("A {:?} practice discovered in {}", domain, source),
    };
    grimoire.discoveries.push(d.clone());
    *grimoire.domains_known.entry(format!("{:?}", domain)).or_insert(0) += 1;
    grimoire.total_power += strength;
    d
}

/// Magic affects the world: which mechanics it enables
pub fn magic_effects(domain: MagicDomain) -> Vec<&'static str> {
    match domain {
        MagicDomain::Healing => vec!["cure illness", "restore health", "regeneration"],
        MagicDomain::Protection => vec!["ward enemies", "prevent disease", "resist weather"],
        MagicDomain::Prophecy => vec!["see future events", "warn of danger", "guide decisions"],
        MagicDomain::Elemental => vec!["control elements", "weather manipulation", "resource creation"],
        MagicDomain::Spirit => vec!["contact ancestors", "gain wisdom", "influence souls"],
        MagicDomain::Transformation => vec!["change form", "alter materials", "adapt"],
        MagicDomain::Binding => vec!["enforce oaths", "guarantee contracts", "bind promises"],
        MagicDomain::Weather => vec!["call storms", "bring rain", "change seasons"],
    }
}
