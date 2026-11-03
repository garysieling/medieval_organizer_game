use crate::session::*;

#[test]
fn test_knights_and_priests_stay_off_the_claim_list() {
    let mut g = Game::new();
    // villagers[0] is always a working adult with a Craft vocation
    let knight = g.villagers[0];
    g.world.get_component_mut::<Craft>(knight).unwrap().vocation = Vocation::Knight;
    let claimed = g.free_villager().map(|i| g.villagers[i]);
    assert_ne!(claimed, Some(knight), "a knight keeps their post");
}

#[test]
fn test_kids_stay_off_the_claim_list() {
    let g = Game::new();
    // villagers[26..29] are the three seeded children
    let kids: Vec<Entity> = g.villagers[26..29].to_vec();
    for &k in &kids {
        assert!(g.world.get_component::<Kid>(k).is_some(), "setup: expected a child here");
    }
    let claimed = g.free_villager().map(|i| g.villagers[i]);
    assert!(claimed.map(|c| !kids.contains(&c)).unwrap_or(true), "a child can't take agent commands");
}
