use crate::session::*;

#[test]
fn test_the_season_call_brings_the_hands() {
    let mut g = Game::new();
    g.laborer_tick(77); // SEASON_AT
    assert!(g.villagers.iter().any(|&v|
        g.world.get_component::<villager::Laborer>(v).is_some()),
        "no hands answered the call");
    assert!(g.villagers.iter().any(|&v|
        g.world.get_component::<villager::Laborer>(v).is_some()
        && g.travels.iter().any(|t| t.who == v)),
        "the hands never walked in");
}

#[test]
fn test_the_season_ends_and_the_hands_go_home() {
    let mut g = Game::new();
    let hand = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<country::Outlander>(v).is_some())
        .expect("no country folk");
    g.world.add_component(hand, villager::Laborer { until: 80 });
    g.laborer_tick(81);
    assert!(g.world.get_component::<villager::Laborer>(hand).is_none(),
        "the season's over and he's still marked");
    assert!(g.travels.iter().any(|t| t.who == hand), "he never started home");
}

#[test]
fn test_a_drunk_hands_night_ends_in_the_mud() {
    let mut g = Game::new();
    let hand = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<country::Outlander>(v).is_some())
        .expect("no country folk");
    // stand a witness near him
    let wit = g.villagers.iter().copied().find(|&w| w != hand
        && g.world.get_component::<country::Outlander>(w).is_none()).expect("no villager");
    *g.world.get_component_mut::<Pos>(wit).unwrap() =
        g.world.get_component::<Pos>(hand).unwrap().clone();
    g.world.add_component(hand, villager::Laborer { until: 999 });
    g.world.add_component(hand, villager::Drunk { until: 999 });
    // run the check until the brawl lands - the odds are 1-in-4 a check
    for d in 0..60 {
        g.laborer_tick(100 + d);
        if g.world.get_component::<crate::game::feuds::Feud>(wit).is_some()
            || g.world.get_component::<crate::game::desires::InnerLife>(wit)
                .map(|l| l.grievances.iter().any(|g2| g2.0 == crate::game::desires::Problem::Rowdy))
                .unwrap_or(false) { break; }
    }
    assert!(g.world.get_component::<crate::game::desires::InnerLife>(wit)
        .map(|l| l.grievances.iter().any(|g2| g2.0 == crate::game::desires::Problem::Rowdy))
        .unwrap_or(false), "sixty checks and nobody saw the fists");
}
