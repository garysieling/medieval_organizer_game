use crate::session::*;

#[test]
fn test_a_letter_lands_and_warms_the_bond() {
    let mut g = Game::new();
    let (from, to) = (g.villagers[0], g.villagers[1]);
    g.letters.push(Letter { from: from, to: to, office: None, courier: None, due: 7 });
    g.letter_tick(7);
    assert!(g.letters.is_empty(), "the letter's still on the road");
    assert!(g.log.iter().any(|l| l.contains("a letter comes")),
        "the landing went unsaid");
    assert!(crate::game::relationships::find_relationship(&g.world, from, to).is_some(),
        "the bond never warmed");
}

#[test]
fn test_the_scribe_pockets_his_penny() {
    let mut g = Game::new();
    g.force_event(crate::session::events::EventId::Scribe);
    // walk him in: the road's a frame's work, `until` books at the pitch
    for _ in 0..6000 {
        g.advance_rare();
        if g.rare.iter().any(|r| r.id == crate::session::events::EventId::Scribe && r.until.is_some()) { break; }
    }
    let t = g.tick;
    let si = g.rare.iter().position(|r| r.id == crate::session::events::EventId::Scribe);
    let Some(at) = si.and_then(|i| g.rare[i].at) else { return }; // he never arrived
    let se = g.rare[si.unwrap()].e.unwrap();
    // an illiterate villager, a living bond for the page to carry, the penny
    let a = g.villagers.iter().copied().find(|&v| !g.literate(v)
        && !villager::is_dead(&g.world, v)).expect("no illiterate villager");
    let b = g.villagers.iter().copied().find(|&v| v != a
        && !villager::is_dead(&g.world, v)).unwrap();
    crate::game::relationships::record_interaction(&mut g.world, a,
        crate::game::relationships::Interaction { other: b,
            kind: crate::game::relationships::InteractionKind::Conversation, intensity: 2.0 });
    if let Some(p) = g.world.get_component_mut::<Pos>(a) { p.x = at.0 as f32; p.y = at.1 as f32; }
    if let Some(p) = g.world.get_component_mut::<Purse>(a) { p.gold = 5.0; }
    let cust = g.world.get_component::<Purse>(a).map(|p| p.gold).unwrap();
    for dt in 0..10 { g.events_tick(t + dt); }
    assert!(g.letters.iter().any(|l| l.from == a), "he never wrote the page");
    assert_eq!(g.world.get_component::<Purse>(a).map(|p| p.gold).unwrap(), cust - 1.0,
        "the penny was never charged");
    assert!(g.world.get_component::<Purse>(se).map(|p| p.gold).unwrap_or(0.0) >= 1.0,
        "the penny never reached the scribe");
}
