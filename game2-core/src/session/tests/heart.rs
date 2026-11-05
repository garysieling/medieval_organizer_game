use crate::session::*;

#[test]
fn test_the_heart_helper_is_one() {
    let g = Game::new();
    let mm = g.matchmaker.expect("a heart-helper watches the square");
    assert_eq!(g.world.get_component::<TownRole>(mm), Some(&TownRole::Matchmaker));
    assert_eq!(g.villagers.iter().filter(|&&v|
        g.world.get_component::<TownRole>(v) == Some(&TownRole::Matchmaker)).count(), 1);
}

#[test]
fn test_lonely_hearts_excludes_the_spoken_for() {
    let mut g = Game::new();
    // nobody starts lonely - wants are sparse - so none qualify by default
    // give two villagers unmet love wants and they join the pool
    let (a, b) = (g.villagers[0], g.villagers[1]);
    for v in [a, b] {
        g.world.get_component_mut::<InnerLife>(v).unwrap().wants.push(desires::Want::Love);
    }
    let pool = g.lonely_hearts();
    assert!(pool.contains(&a) && pool.contains(&b), "the lonely are known");
    // a kid can't hold a heart to mend (no craft)
    assert!(!g.lonely_hearts().iter().any(|&v| g.world.get_component::<Craft>(v).is_none()));
}

#[test]
fn test_a_match_kindles_a_lover_bond() {
    let mut g = Game::new();
    let (a, b) = (g.villagers[0], g.villagers[1]);
    for v in [a, b] {
        g.world.get_component_mut::<InnerLife>(v).unwrap().wants.push(desires::Want::Love);
    }
    // keep trying across the years until the spark catches
    for t in (0..900).step_by(31) { g.match_attempt(a, t, g.matchmaker, "the heart-helper"); }
    assert!(g.has_rel(relationships::RelKind::Lover, a), "a match was kindled");
    assert!(g.world.get_component::<InnerLife>(a).unwrap().satisfied.iter()
        .any(|(w, _)| *w == desires::Want::Love), "the ache is mended");
    assert!(g.log.iter().any(|l| l.contains("together")), "the village hears of it");
}

#[test]
fn test_kin_are_not_paired() {
    let mut g = Game::new();
    let (a, b) = (g.villagers[0], g.villagers[1]);
    for v in [a, b] {
        g.world.get_component_mut::<InnerLife>(v).unwrap().wants.push(desires::Want::Love);
    }
    // a and b are kin: no match between them, ever
    let rel = g.world.create_entity();
    g.world.add_component(rel, relationships::Relationship {
        from: a, other: b, kind: relationships::RelKind::Family, strength: 0.8, interactions: 5 });
    for t in (0..40).step_by(31) { g.match_attempt(a, t, g.matchmaker, "the heart-helper"); }
    assert_eq!(g.rel_kind(a, b), Some(relationships::RelKind::Family), "kin stay kin");
}

#[test]
fn test_the_find_box_knows_her_by_her_posting() {
    let g = Game::new();
    let mm = g.matchmaker.expect("a heart-helper watches the square");
    let m = g.ui_model();
    let pv = m.people.iter().find(|p| p.e == mm).expect("she's on the map");
    // "heart helper" finds her: the posting folds into the find text
    // alongside her name and trade
    assert!(pv.find.contains("heart helper"), "find text: {}", pv.find);
    assert!(pv.find.contains(&find_key(&vill_name(g.villagers.iter().position(|&v| v == mm).unwrap()))),
        "her name's in there too: {}", pv.find);
}
