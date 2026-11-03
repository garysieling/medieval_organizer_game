use crate::session::*;

#[test]
fn test_the_fever_year_opens_in_three_beds() {
    let g = Game::new_scenario(Scenario::FeverYear);
    let fevered = g.world.query::<Disease>().iter()
        .filter(|&&d| g.world.get_component::<Disease>(d)
            .map(|dd| dd.name == "scarlatina").unwrap_or(false)).count();
    assert_eq!(fevered, 3, "the scarlatina's already in three beds");
    assert!(g.log.iter().any(|l| l.contains("the scarlatina year")));
}

#[test]
fn test_the_circus_winter_opens_harsh() {
    let mut g = Game::new_scenario(Scenario::CircusWinter);
    assert!(g.circus.is_some(), "the tents are already pitched");
    g.winter_tick(g.tick);
    assert!(g.harsh_winter, "a hard winter");
}

#[test]
fn test_the_widows_shop_stands_derelict() {
    let g = Game::new_scenario(Scenario::WidowsShop);
    assert!(g.log.iter().any(|l| l.contains("the widow's shop")));
    // and someone's weighing the keeping of it
    let any_derelict = g.building_entities.iter().any(|&b|
        g.world.get_component::<Building>(b).map(|bb| household::derelict(bb)).unwrap_or(false));
    assert!(any_derelict, "a shop stands derelict");
}
