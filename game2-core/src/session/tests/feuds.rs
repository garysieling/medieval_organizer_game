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
