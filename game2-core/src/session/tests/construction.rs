use crate::session::*;

#[test]
fn test_the_hands_labor_it_up() {
    let mut g = Game::new();
    g.construction_sites.push(ConstructionSite {
        kind: household::BuildingKind::Home, tile: (10, 10),
        progress: 0, need: 4, crew: vec![], stalled_days: 0 });
    // put two hands on it
    for v in g.villagers.iter().take(2).copied().collect::<Vec<_>>() {
        g.world.get_component_mut::<Pos>(v).map(|p| { p.x = 10.0; p.y = 10.0; });
        g.construction_sites.last_mut().unwrap().crew.push(v);
    }
    for d in 0..4 { g.site_tick(100 + d); }
    assert!(g.construction_sites.is_empty(), "the work never stood");
    assert!(g.building_entities.iter().any(|&b|
        g.world.get_component::<Pos>(b).map(|p| p.x.round() as i32 == 10).unwrap_or(false)),
        "the building never registered");
}

#[test]
fn test_no_hands_and_it_stands_unfinished() {
    let mut g = Game::new();
    g.construction_sites.push(ConstructionSite {
        kind: household::BuildingKind::Home, tile: (10, 10),
        progress: 0, need: 8, crew: vec![], stalled_days: 0 });
    for d in 0..10 { g.site_tick(100 + d); }
    assert_eq!(g.construction_sites.len(), 1, "the weather took the site");
    assert!(g.log.iter().any(|l| l.contains("unfinished")), "the stall went unlogged");
}
