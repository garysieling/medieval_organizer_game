use crate::session::*;

#[test]
fn test_about_one_in_twelve_carry_a_difference() {
    let g = Game::new();
    let carriers = g.villagers.iter().filter(|&&v|
        g.world.get_component::<villager::Condition>(v).is_some()).count();
    assert!((1..=6).contains(&carriers), "{} carry a difference", carriers);
    for &v in &g.villagers {
        if let Some(c) = g.world.get_component::<villager::Condition>(v) {
            assert!(["a lame leg", "failing sight", "a mind that needs quiet",
                "no sight", "no hearing"].contains(&c.kind), "known kinds only: {}", c.kind);
        }
    }
}

#[test]
fn test_the_deaf_miss_the_call() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v| !villager::is_dead(&g.world, v)).unwrap();
    g.world.add_component(v, villager::Condition { kind: "no hearing", until: u64::MAX });
    if let Some(p) = g.world.get_component_mut::<crate::game::traits::Personality>(v) {
        if !p.traits.contains(&crate::game::traits::Trait::Pious) { p.traits.push(crate::game::traits::Trait::Pious); }
    }
    g.livelihood_tick(7); // a holy day - the bell rings
    let worshipped = g.world.get_component::<routine::Routine>(v)
        .map(|r| r.mode == routine::Mode::Worshipping && r.day == 7).unwrap_or(false);
    assert!(!worshipped, "no bell reaches them");
    // and the call did go out - somebody else stood in it
    let others = g.villagers.iter().any(|&u| g.world.get_component::<routine::Routine>(u)
        .map(|r| r.mode == routine::Mode::Worshipping && r.day == 7).unwrap_or(false));
    assert!(others, "the faithful still gather");
}

#[test]
fn test_the_blind_skip_the_stalls() {
    let mut g = Game::new();
    // stock a shelf so there's somewhere to browse
    let shop = g.world.query::<Building>().into_iter().find(|&b|
        g.world.get_component::<Building>(b)
            .map(|bb| household::is_workplace(bb.kind)).unwrap_or(false)).unwrap();
    household::add_stock(g.world.get_component_mut::<Building>(shop).unwrap(), "bread", 5);
    let setup = |g: &mut Game, v: Entity| {
        if let Some(l) = g.world.get_component_mut::<desires::InnerLife>(v) {
            l.wants = vec![Want::GoodFood]; l.satisfied.clear();
        }
        if let Some(p) = g.world.get_component_mut::<Purse>(v) { p.gold = 50.0; }
        else { g.world.add_component(v, Purse { gold: 50.0 }); }
    };
    let sighted = g.villagers.iter().copied().find(|&v| !villager::is_dead(&g.world, v)).unwrap();
    setup(&mut g, sighted);
    assert!(g.want_shopping(sighted, 1).is_some(), "a shopper finds the shelf");
    let blind = g.villagers.iter().copied().find(|&v| v != sighted && !villager::is_dead(&g.world, v)).unwrap();
    setup(&mut g, blind);
    g.world.add_component(blind, villager::Condition { kind: "no sight", until: u64::MAX });
    assert!(g.want_shopping(blind, 1).is_none(), "browsing takes eyes");
}

#[test]
fn test_care_costs_a_morning() {
    let mut g = Game::new();
    g.care_tick(2);
    let marked = g.villagers.iter().any(|&c|
        g.world.get_component::<villager::CareVisit>(c).is_some());
    assert!(marked, "a check-in costs somebody a morning");
}
