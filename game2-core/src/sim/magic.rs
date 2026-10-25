//! Magic system: dynamic spells from ingested texts.
//! From backlog: "magic/story system that evolves based on books"
//! Icelandic sagas -> new behaviors/mechanics at runtime.

use crate::engine::ecs::{Entity, World};

/// A spell/ability learned from text
#[derive(Debug)]
pub struct Spell {
    pub name: String,
    pub source_text: String,   // book/saga it came from
    pub effect: SpellEffect,
    pub power: f32,
}

#[derive(Debug)]
pub enum SpellEffect {
    Buff(String, f32),       // boost a stat
    Debuff(String, f32),     // reduce a stat
    Summon(String),          // create an entity
    Transform(String),       // change something
    Vision(String),          // reveal information
}

/// A grimoire/spellbook - grows from ingested texts
#[derive(Debug)]
pub struct Grimoire {
    pub spells: Vec<Spell>,
    pub texts_read: Vec<String>,
}

/// Ingest a text - extract potential spells/mechanics
pub fn ingest_text(grim: &mut Grimoire, title: &str, content: &str) -> Vec<String> {
    grim.texts_read.push(title.to_string());
    // Simple heuristic: look for magic/ritual/omen keywords
    let mut found = vec![];
    for kw in ["curse", "omen", "prophecy", "ritual", "spirit", "ghost", "vision"] {
        if content.to_lowercase().contains(kw) {
            found.push(format!("{}:{}", title, kw));
        }
    }
    found
}

/// Spawn a grimoire
pub fn spawn_grimoire(world: &mut World) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Grimoire { spells: vec![], texts_read: vec![] });
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingest_finds_magic() {
        let mut g = Grimoire { spells: vec![], texts_read: vec![] };
        let found = ingest_text(&mut g, "Saga", "The witch spoke a curse and the omen came true");
        assert!(found.iter().any(|f| f.contains("curse")));
        assert!(found.iter().any(|f| f.contains("omen")));
    }
}
