use crate::session::*;

#[test]
fn test_the_feasts_fall_on_the_celestial_anchors() {
    let mut g = Game::new();
    // the wheel marks the year: Midsummer's its own day, not a %90 turn
    g.festival_tick(172);
    assert!(g.log.iter().any(|l| l.contains("Midsummer")),
        "Midsummer's anchor never fired");
    // and an in-between day falls silent
    let g2 = {
        let mut g2 = Game::new();
        g2.festival_tick(100);
        g2
    };
    assert!(!g2.log.iter().any(|l| l.contains("feast") || l.contains("Midsummer")
        || l.contains("Yule") || l.contains("Harvest")),
        "a plain day fired a feast");
}

#[test]
fn test_the_pious_and_the_old_faith_read_the_sky_their_own_ways() {
    let g = Game::new();
    // the read keys on the heart: Pious reads God's sign, the old faith
    // the stones' word - the lines exist for both
    assert!(g.villagers.iter().any(|&v|
        g.world.get_component::<crate::game::traits::Personality>(v)
            .map(|p| p.traits.contains(&crate::game::traits::Trait::Pious)).unwrap_or(false)
    ), "no pious villager to read the sky");
    assert!(g.villagers.iter().any(|&v|
        crate::session::routine::keeps_old_ways(&g.world, v)
    ), "no old-faith villager to read the sky");
}
