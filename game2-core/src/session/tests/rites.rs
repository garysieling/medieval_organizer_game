use crate::session::*;

/// A village child the old ways don't claim - the church gets them
fn church_kid(g: &Game) -> Entity {
    g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Kid>(v).is_some()
            && g.site_of(v).is_none()
            && !routine::keeps_old_ways(&g.world, v))
        .expect("a church-going child")
}

/// A child of the old ways: the first two kids may both keep the new
/// faith, so spawn until the stones claim one (the hash is on the entity)
fn old_kid(g: &mut Game) -> Entity {
    if let Some(v) = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Kid>(v).is_some()
            && g.site_of(v).is_none()
            && routine::keeps_old_ways(&g.world, v)) {
        return v;
    }
    let home = household::home_of(&g.world, g.villagers[0]).expect("a home");
    for _ in 0..12 {
        let before = g.villagers.len();
        g.spawn_kid(home, 1);
        let v = g.villagers[before];
        if routine::keeps_old_ways(&g.world, v) { return v; }
    }
    panic!("the old ways claim no child");
}

/// Walk every pending rite/wedding to its site and let the day scan hold it
fn gather_rites(g: &mut Game, tick: u64, rounds: usize) {
    for _ in 0..rounds {
        g.advance_travels();
        g.rites_tick(tick);
    }
}

#[test]
fn test_a_new_adult_is_initiated_at_the_church_before_the_party() {
    let mut g = Game::new();
    let kid = church_kid(&g);
    g.world.get_component_mut::<KidGrowth>(kid).unwrap().age_days = vocation::COMING_OF_AGE - 1;
    g.lifecycle_tick(100);
    assert!(g.world.get_component::<Kid>(kid).is_none(), "they've grown");
    let p = g.world.get_component::<villager::PendingRite>(kid)
        .expect("the rite was never called");
    assert!(!p.old, "a church parishioner keeps the old ways");
    assert!(g.world.get_component::<villager::Initiated>(kid).is_none(),
        "marked before the rite was held");
    // the officiant is a priest when the parish has one
    if g.villagers.iter().any(|&v| g.world.get_component::<Craft>(v)
        .map(|c| c.vocation == Vocation::Priest).unwrap_or(false)) {
        assert_eq!(g.world.get_component::<Craft>(p.by).unwrap().vocation, Vocation::Priest,
            "a priest stands for the church");
    }
    gather_rites(&mut g, 100, 20000);
    assert!(g.world.get_component::<villager::Initiated>(kid).is_some(),
        "never made a Christian of the parish");
    assert!(g.world.get_component::<villager::PendingRite>(kid).is_none());
    // and the rite comes before the party in the telling: the rite's own
    // day gathered at the church, the household party followed on the next
    let rite_at = g.log.iter().position(|l| l.contains("rite of initiation"));
    g.rites_tick(101);
    assert!(g.world.get_component::<villager::DueParty>(kid).is_none(), "the party never came");
    let party_at = g.log.iter().position(|l| l.contains("comes of age"));
    assert!(rite_at.is_some() && party_at.is_some() && rite_at < party_at,
        "the party was told before the rite: {:?}", g.log.iter().rev().take(6).collect::<Vec<_>>());
}

#[test]
fn test_the_old_ways_keep_their_rite_at_the_stones() {
    let mut g = Game::new();
    let kid = old_kid(&mut g);
    g.world.get_component_mut::<KidGrowth>(kid).unwrap().age_days = vocation::COMING_OF_AGE - 1;
    g.lifecycle_tick(100);
    let p = g.world.get_component::<villager::PendingRite>(kid)
        .expect("the old rite was never called");
    assert!(p.old, "the old ways were denied their rite");
    // the site is the stones or the grove, not the church door
    let near_stone = g.stones.iter().any(|s|
        (s.0 - p.site.0).abs() + (s.1 - p.site.1).abs() <= 2);
    let near_tree = g.map.trees.iter().any(|t|
        (t.0 - p.site.0).abs() + (t.1 - p.site.1).abs() <= 2);
    assert!(near_stone || near_tree, "the old rite wandered off the stones");
    gather_rites(&mut g, 100, 20000);
    assert!(g.world.get_component::<villager::Initiated>(kid).is_some(),
        "never sworn to the old stones");
    assert!(g.log.iter().any(|l| l.contains("old stones")),
        "the rite went untold: {:?}", g.log.iter().rev().take(8).collect::<Vec<_>>());
}

/// Two grown villagers under one roof each, made lovers the way the
/// heart-helper makes them
fn court_a_couple(g: &mut Game) -> (Entity, Entity) {
    let mut folk: Vec<Entity> = g.villagers.iter().copied().filter(|&v|
        g.site_of(v).is_none()
            && !villager::is_dead(&g.world, v)
            && g.world.get_component::<Craft>(v).is_some()
            && household::home_of(&g.world, v).is_some())
        .collect();
    folk.sort();
    let (a, b) = (folk[0], folk[1]);
    for (x, y) in [(a, b), (b, a)] {
        relationships::add_relationship(&mut g.world, relationships::Relationship {
            from: x, other: y, kind: relationships::RelKind::Lover,
            strength: 0.6, interactions: 1 });
    }
    g.courtships.push((a, b, 0));
    (a, b)
}

#[test]
fn test_lovers_wed_and_keep_one_roof() {
    let mut g = Game::new();
    let (a, b) = court_a_couple(&mut g);
    // the courtship runs its month plus the pair's stagger
    g.rites_tick(45);
    assert!(g.world.get_component::<villager::Betrothed>(a).is_some(), "never betrothed");
    assert_eq!(g.world.get_component::<villager::Betrothed>(a).unwrap().partner, b);
    assert_eq!(g.world.get_component::<villager::Betrothed>(b).unwrap().partner, a);
    gather_rites(&mut g, 45, 20000);
    assert!(g.world.get_component::<villager::Betrothed>(a).is_none(), "still betrothed");
    assert_eq!(g.rel_kind(a, b), Some(relationships::RelKind::Family),
        "lovers didn't become family");
    assert_eq!(g.rel_kind(b, a), Some(relationships::RelKind::Family),
        "the bond ran one way only");
    assert!(g.log.iter().any(|l| l.contains("are wed")),
        "the wedding went untold: {:?}", g.log.iter().rev().take(8).collect::<Vec<_>>());
    assert_eq!(household::home_of(&g.world, a), household::home_of(&g.world, b),
        "the wed keep two roofs");
    assert!(g.courtships.iter().all(|&(x, y, _)| !((x == a && y == b) || (x == b && y == a))),
        "the spent courtship stayed on the books");
}

#[test]
fn test_the_wed_house_can_bring_a_child() {
    let mut g = Game::new();
    let (a, _) = court_a_couple(&mut g);
    g.rites_tick(45);
    gather_rites(&mut g, 45, 20000);
    let home = household::home_of(&g.world, a).expect("the wed have no roof");
    let grown = g.world.get_component::<Building>(home).unwrap().occupants.iter()
        .filter(|&&o| g.world.get_component::<Kid>(o).is_none()
            && !crate::game::dungeon::unavailable(&g.world, o)
            && (g.villagers.contains(&o) || g.seats.iter().any(|s| s.player == o)))
        .count();
    assert!(grown >= 2, "the wed house holds {} grown folk, not a pair", grown);
    let before = g.villagers.len();
    for t in 0..30 { g.maybe_birth(t); }
    assert!(g.villagers.len() > before, "the full house never brought a child");
}
