use crate::session::*;

#[test]
fn test_a_full_house_gets_a_child() {
    let mut g = Game::new();
    // a house with two grown folk: room fills out from there
    let home = g.building_entities.iter().copied().find(|&b|
        g.world.get_component::<Building>(b).map(|bb| bb.kind == BuildingKind::Home).unwrap_or(false)).unwrap();
    let two: Vec<Entity> = g.villagers[..2].to_vec();
    g.world.get_component_mut::<Building>(home).unwrap().occupants = two;
    let n = g.villagers.len();
    g.maybe_birth(40); // may or may not, it's a roll - keep calling until the roll lands
    for t in (40..400).step_by(40) { if g.villagers.len() > n { break; } g.maybe_birth(t); }
    assert!(g.villagers.len() > n, "a full house gets a child");
    let baby = *g.villagers.last().unwrap();
    assert!(g.world.get_component::<Kid>(baby).is_some());
    assert!(g.world.get_component::<KidGrowth>(baby).is_some());
    assert!(g.log.iter().any(|l| l.contains("born")), "{:?}", g.log.last());
}

#[test]
fn test_a_stranger_settles_at_the_gate() {
    let mut g = Game::new();
    let n = g.villagers.len();
    g.maybe_migrant(30);
    assert_eq!(g.villagers.len(), n + 1);
    let m = *g.villagers.last().unwrap();
    // trade in hand, and a roof if one's going (a bed want otherwise)
    assert!(g.world.get_component::<Craft>(m).is_some());
    let housed = household::home_of(&g.world, m).is_some();
    let wants_bed = desires::problems_of(&g.world, m, g.tick).contains(&desires::Problem::NoHome);
    assert!(housed || wants_bed, "a spare bed or the want of one");
    assert!(g.log.iter().any(|l| l.contains("settles")), "{:?}", g.log.last());
}
