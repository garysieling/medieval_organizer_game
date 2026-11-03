use crate::session::*;

#[test]
fn test_a_kid_comes_of_age_and_takes_a_trade() {
    let mut g = Game::new();
    let kid = g.villagers[26];
    assert!(g.world.get_component::<Kid>(kid).is_some());
    g.world.get_component_mut::<KidGrowth>(kid).unwrap().age_days = vocation::COMING_OF_AGE - 1;
    g.lifecycle_tick(100);
    assert!(g.world.get_component::<Kid>(kid).is_none(), "they've grown");
    assert!(g.world.get_component::<Craft>(kid).is_some(), "they take a trade");
}

#[test]
fn test_the_errand_hirer_becomes_the_master() {
    let mut g = Game::new();
    let kid = g.villagers[27];
    let master = g.villagers.iter().copied().find(|&v|
        v != kid && g.world.get_component::<Craft>(v).is_some()).unwrap();
    let mvoc = g.world.get_component::<Craft>(master).unwrap().vocation;
    {
        let grw = g.world.get_component_mut::<KidGrowth>(kid).unwrap();
        grw.age_days = vocation::COMING_OF_AGE - 1;
        grw.errands = vec![(master, 5)];
    }
    g.lifecycle_tick(100);
    let c = g.world.get_component::<Craft>(kid).unwrap();
    assert_eq!(c.vocation, mvoc, "they follow the trade they ran for");
    assert_eq!(c.master, Some(master), "the hirer stands as master");
    assert_eq!(c.tasked_by, Some(master));
}
