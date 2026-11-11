//! The generated people art (assets/rd/people, #690): every posting
//! and trade figure loads, the pack figure stands in when they're
//! missing, and the model names who wears them.
use crate::*;

#[test]
fn test_people_sprites_load_at_villager_height() {
    let art = Art::load_fs("../assets", 0);
    // the postings and trades, plus the rare visitors' slugs (#693)
    assert_eq!(art.people.len(), PEOPLE_SPRITES.len() + game2_core::session::events::VISIT_SLUGS.len(),
        "every generated people figure loads: {:?}", art.people.iter().map(|(n, _)| n).collect::<Vec<_>>());
    for (name, s) in &art.people {
        // visitors came out at walk-figure height (~35px); the village
        // stills were resampled to @24
        let cap = if game2_core::session::events::VISIT_SLUGS.contains(&name.as_str()) { 40 } else { 30 };
        assert!((18..=cap).contains(&s.height), "{} is {}px tall - should read as a person (~24-35)", name, s.height);
        assert!(s.pixels.chunks(4).any(|p| p[3] == 0), "{}: background not keyed out", name);
    }
    // the sexed trades come in pairs, the postings bare
    for v in ["baker", "farmer", "glazier", "knight", "luthier", "mason", "priest", "printmaker", "tailor", "toolmaker"] {
        assert!(art.person(&format!("{}-m", v)).is_some() && art.person(&format!("{}-f", v)).is_some(), "{} pair", v);
    }
    for r in ["overseer", "tax-collector", "trader", "guard-spear", "guard-sword", "guard-shield", "guard-halberd"] {
        assert!(art.person(r).is_some(), "{}", r);
    }
}

#[test]
fn test_people_fall_back_to_pack_figures() {
    // rd/people missing: the pack's villagers and soldiers draw instead
    let fetch = |rel: &str| -> Option<Vec<u8>> {
        if rel.starts_with("rd/people") { return None; }
        std::fs::read(std::path::Path::new("../assets").join(rel)).ok()
    };
    let art = Art::load(&fetch, 0);
    assert!(art.people.is_empty());
    assert!(art.person("baker-m").is_none() && art.person("tax-collector").is_none());
    assert_eq!(art.villager_sprites.len(), 8);
    assert_eq!(art.guards.len(), 4);
}

#[test]
fn test_the_model_names_who_wears_the_art() {
    let game = Game::new_seeded(MAP_SEED);
    let m = game.ui_model();
    // the collector's rounds read as the tax collector figure; whoever
    // holds the covered trades wears theirs
    let collector = m.people.iter().find(|p| matches!(p.kind, PKind::Villager(i) if i == game2_core::session::COLLECTOR_IDX))
        .expect("the tax collector walks the village");
    assert_eq!(collector.person.as_deref(), Some("tax-collector"));
    let keyed: Vec<&str> = m.people.iter().filter_map(|p| p.person.as_deref()).collect();
    assert!(keyed.iter().any(|k| !k.starts_with("guard-") && *k != "tax-collector"),
        "some working trade has a figure too: {:?}", keyed);
    // the watch carries their own stills under the walk sheets
    assert!(m.people.iter().any(|p| matches!(p.kind, PKind::Guard(_))
        && p.person.as_deref().map_or(false, |k| k.starts_with("guard-"))));
    // kids and the uncovered trades keep the generic look
    assert!(m.people.iter().any(|p| matches!(p.kind, PKind::Villager(_)) && p.person.is_none()),
        "uncovered villagers fall back to the pack figure");
}
