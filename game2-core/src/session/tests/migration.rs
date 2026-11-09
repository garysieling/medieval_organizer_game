use crate::session::*;

#[test]
fn test_the_hunger_pushes_a_household() {
    let mut g = Game::new();
    for i in 0..8 {
        let v = g.villagers[i % g.villagers.len()];
        g.world.add_component(v, crate::game::villager::Hunger { value: 0.9 });
    }
    let before = g.villagers.len();
    g.migration_tick(57);
    assert!(g.villagers.len() < before, "the hunger never pushed: {} -> {}", before, g.villagers.len());
    assert!(g.emigration > 0, "the leaving never counted");
    assert!(g.log.iter().any(|l| l.contains("walks off the map")),
        "the leaving never logged");
}

#[test]
fn test_the_plenty_draws_a_stranger() {
    let mut g = Game::new();
    g.soil_quality = 0.8;
    // no hungry villagers - the seeded ones carry the field too
    for i in 0..g.villagers.len() {
        let v = g.villagers[i];
        if let Some(h) = g.world.get_component_mut::<crate::game::villager::Hunger>(v) {
            h.value = 0.0;
        }
    }
    g.emigration = 3;
    g.migration_tick(117);
    // the plenty's own pull: the emigration's own count falls - the
    // swell's the mirror
    assert!(g.emigration < 3, "the plenty never drew: {}", g.emigration);
    assert!(g.log.iter().any(|l| l.contains("a stranger comes")),
        "the arrival never logged");
}
