use crate::session::*;

#[test]
fn test_the_purse_buys_what_the_village_lacks() {
    let mut g = Game::new();
    let castle = g.sites.iter().position(|s| s.kind == crate::session::country::SiteKind::Castle).unwrap_or(0);
    g.sites[castle].treasury = 200.0;
    // the season's check day
    g.lord_works_tick(15);
    assert!(g.sites[castle].treasury < 200.0, "the purse never opened");
    assert!(!g.lord_works.is_empty(), "no commission queued");
    // the work stands on its day
    g.lord_works_tick(21);
    assert!(g.lord_works.is_empty(), "the crews never finished");
    assert!(g.building_entities.iter().any(|&b|
        g.world.get_component::<household::Building>(b)
            .map(|bb| bb.kind == household::BuildingKind::Dungeon
                || bb.kind == household::BuildingKind::Plantation).unwrap_or(false)),
        "the lord's work never stood");
}

#[test]
fn test_the_empty_purse_stays_shut() {
    let mut g = Game::new();
    g.lord_works_tick(15);
    assert!(g.lord_works.is_empty(), "a broke lord still commissioned");
}

#[test]
fn test_one_commission_a_season() {
    let mut g = Game::new();
    let castle = g.sites.iter().position(|s| s.kind == crate::session::country::SiteKind::Castle).unwrap_or(0);
    g.sites[castle].treasury = 500.0;
    g.lord_works_tick(15);
    g.lord_works_tick(15);
    assert_eq!(g.lord_works.len(), 1, "the purse paid twice in a day");
}
