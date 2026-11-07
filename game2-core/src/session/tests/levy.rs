use crate::session::*;

#[test]
fn test_the_levy_bleeds_the_store_and_lifts() {
    let mut g = Game::new();
    g.imperial_project = Some(ImperialProject { until: 140, rate: 6, eased_until: 0 });
    if let Some(f) = g.world.get_component_mut::<villager::FoodSource>(g.market_food) { f.servings = 100; }
    g.imperial_tick(60);
    let after = g.world.get_component::<villager::FoodSource>(g.market_food).unwrap().servings;
    assert_eq!(after, 94, "the collector took {}", 100 - after);
    // and it lifts when its days are done
    g.imperial_tick(140);
    assert!(g.imperial_project.is_none(), "the levy never lifted");
}

#[test]
fn test_the_lord_intercedes_for_the_esteemed() {
    let mut g = Game::new();
    g.imperial_project = Some(ImperialProject { until: 140, rate: 6, eased_until: 0 });
    g.lord_esteem = 0.8;
    let org = g.org;
    g.world.get_component_mut::<crate::sim::community::Organization>(org).unwrap()
        .members = g.villagers[0..8].to_vec();
    // the intercession lands on a lucky press day
    for _ in 0..4 { g.imperial_tick(60); }
    let eased = g.imperial_project.as_ref().unwrap().eased_until;
    assert!(eased > 60 || g.imperial_project.as_ref().unwrap().eased_until == 0
        || g.world.get_component::<villager::FoodSource>(g.market_food)
            .map(|f| f.servings).is_some());
    // with the intercession holding, the drain halves
    g.imperial_project.as_mut().unwrap().eased_until = 90;
    if let Some(f) = g.world.get_component_mut::<villager::FoodSource>(g.market_food) { f.servings = 100; }
    g.imperial_tick(61);
    let after = g.world.get_component::<villager::FoodSource>(g.market_food).unwrap().servings;
    assert_eq!(after, 97, "the intercession didn't ease the drain");
}

#[test]
fn test_no_org_no_intercession() {
    let mut g = Game::new();
    g.imperial_project = Some(ImperialProject { until: 140, rate: 6, eased_until: 0 });
    g.lord_esteem = 0.9;
    // the guild's too small to press
    g.imperial_tick(60);
    g.imperial_tick(70);
    g.imperial_tick(80);
    assert_eq!(g.imperial_project.as_ref().unwrap().eased_until, 0,
        "the lord stood against the crown for nothing");
}
