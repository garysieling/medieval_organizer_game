use crate::session::*;

#[test]
fn test_the_castles_mood_reaches_the_village() {
    let mut g = Game::new();
    let oe = g.org;
    // a cause with real muscle: the estimate climbs past the tiers
    g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(oe)
        .unwrap().capacity = 1.0;
    for t in (3..=400).step_by(7) { g.power_tick(t); }
    assert!(g.lord_esteem > 0.5, "the estimate never climbed: {}", g.lord_esteem);
    assert!(g.log.iter().any(|l| l.contains("the castle's heard the name")
        || l.contains("the lord keeps an ear on it")), "no word came down: {:?}",
        g.log.iter().filter(|l| l.contains("castle") || l.contains("lord")).collect::<Vec<_>>());
    // and it eases back the other way when the muscle's spent
    g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(oe)
        .unwrap().capacity = 0.0;
    for t in (402..=1500).step_by(7) { g.power_tick(t); }
    assert!(g.log.iter().any(|l| l.contains("eases") || l.contains("slips their minds")
        || l.contains("forgot")), "the easing never reached the village");
}
