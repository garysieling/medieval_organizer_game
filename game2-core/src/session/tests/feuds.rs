use crate::session::*;
use crate::game::feuds::{self, FeudStatus};
use crate::engine::ecs::System;

#[test]
fn test_bad_blood_burns_then_settles_under_the_priests_word() {
    let mut g = Game::new();
    let (a, b) = (g.villagers[0], g.villagers[1]);
    feuds::spawn_feud(&mut g.world, b, a, "a turned-down suit");
    // left alone it worsens till it burns, and the village hears of it
    for _ in 0..20 { feuds::FeudSystem.run(&mut g.world); }
    g.feud_tick(g.tick);
    assert!(g.log.iter().any(|l| l.contains("bad blood runs hot")), "{}", g.log.last().unwrap_or(&"".into()));
    // and who it's with is no secret: the card says so
    g.seats[0].selected_target = 0;
    g.seats[0].picked = true;
    let insp = g.villager_inspector_at(0);
    assert!(insp.lines.iter().any(|l| l.text.contains("feuds with")),
        "{:?}", insp.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());
    // the priest's word on the holy day puts it down over the weeks
    let fe = g.world.query::<feuds::Feud>()[0];
    for _ in 0..40 {
        feuds::FeudSystem.run(&mut g.world);
        g.tick += 1;
        if routine::is_holy(g.tick) { g.feud_tick(g.tick); }
        if g.world.get_component::<feuds::Feud>(fe).unwrap().status == FeudStatus::Resolved { break; }
    }
    assert_eq!(g.world.get_component::<feuds::Feud>(fe).unwrap().status, FeudStatus::Resolved,
        "forty days of arbitration never settled it");
    g.feud_tick(g.tick);
    assert!(g.log.iter().any(|l| l.contains("put to rest")), "no one heard it settled");
    // and settled blood shows no feud on the card
    let insp = g.villager_inspector_at(0);
    assert!(!insp.lines.iter().any(|l| l.text.contains("feuds with")));
}

#[test]
fn test_the_card_names_every_live_feud() {
    let mut g = Game::new();
    let (a, b, c) = (g.villagers[0], g.villagers[1], g.villagers[2]);
    feuds::spawn_feud(&mut g.world, b, a, "a turned-down suit");
    feuds::spawn_feud(&mut g.world, c, a, "a slight");
    // make the second one burn: it's the worse and mustn't hide behind the first
    for _ in 0..20 { feuds::FeudSystem.run(&mut g.world); }
    g.seats[0].selected_target = 0;
    g.seats[0].picked = true;
    let insp = g.villager_inspector_at(0);
    let feud_lines: Vec<String> = insp.lines.iter().filter(|l| l.text.contains("feuds with"))
        .map(|l| l.text.clone()).collect();
    assert_eq!(feud_lines.len(), 1, "the card's feud line: {:?}", feud_lines);
    assert!(feud_lines[0].contains("Aldric") || feud_lines[0].contains(","), "both live feuds named: {:?}", feud_lines);
    assert!(feud_lines.iter().any(|l| l.contains("it burns")), "the burning one says so: {:?}", feud_lines);
}

#[test]
fn test_a_held_priest_settles_no_feuds() {
    let mut g = Game::new();
    let (a, b) = (g.villagers[0], g.villagers[1]);
    let fe = feuds::spawn_feud(&mut g.world, b, a, "a turned-down suit");
    // hold every priest: no one is free to arbitrate
    for &v in &g.villagers {
        if g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false) {
            g.world.add_component(v, crate::game::dungeon::Imprisoned { since: 0, charge: "held".into() });
        }
    }
    for _ in 0..30 {
        feuds::FeudSystem.run(&mut g.world);
        g.tick += 1;
        if routine::is_holy(g.tick) { g.feud_tick(g.tick); }
    }
    assert_ne!(g.world.get_component::<feuds::Feud>(fe).unwrap().status, FeudStatus::Resolved,
        "a priest who can't act settled a feud");
}
