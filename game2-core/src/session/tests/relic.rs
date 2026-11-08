use crate::session::*;

#[test]
fn test_the_saint_consents_or_the_bones_are_just_bones() {
    let mut g = Game::new();
    let church = g.congregation;
    assert!(g.world.get_component::<crate::game::relic::Relic>(church).is_some(),
        "the parish never held its relic");
    // the theft carries it to a raider - the saint consents on the roll
    let raider = g.villagers[0];
    let consented = crate::game::relic::raid_steals(&mut g.world, church, raider, 0);
    assert!(consented, "the story never held");
    assert!(g.world.get_component::<crate::game::relic::Relic>(raider)
        .map(|r| r.prestige > 0.0).unwrap_or(false), "the relic lost its pull");
    // a failed story's just a sack of bones - the fake's exposed
    let back = crate::game::relic::raid_steals(&mut g.world, raider, church, 1);
    assert!(!back, "the story held when the saint refused");
}

#[test]
fn test_the_expose_crashes_the_whole_market() {
    let mut g = Game::new();
    crate::game::relic::expose_all(&mut g.world);
    let all_fakes = g.world.query::<crate::game::relic::Relic>().iter().all(|&e|
        g.world.get_component::<crate::game::relic::Relic>(e)
            .map(|r| r.exposed && r.prestige < 0.2).unwrap_or(true));
    assert!(all_fakes, "a relic kept its pull through the denunciation");
}
