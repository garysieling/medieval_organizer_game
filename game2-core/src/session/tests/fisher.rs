use crate::session::*;

#[test]
fn test_the_fisher_works_the_river() {
    let mut g = Game::new();
    let fisher = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Fisher).unwrap_or(false));
    let Some(f) = fisher else { panic!("no fisher") };
    let hut = g.world.get_component::<Craft>(f).unwrap().workplace.unwrap();
    g.fisher_tick(4);
    let fish = g.world.get_component::<Building>(hut).unwrap()
        .stock.get("fish").copied().unwrap_or(0);
    assert!(fish > 0, "the river gives up its catch");
}

#[test]
fn test_the_water_brings_up_something_strange() {
    let mut g = Game::new();
    g.fisher_tick(60);
    assert!(g.log.iter().any(|l| l.contains("pale stone")),
        "a pale stone, faintly lit");
}

#[test]
fn test_the_gone_fisher_lands_nothing() {
    let mut g = Game::new();
    let fisher = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Fisher).unwrap_or(false));
    let Some(f) = fisher else { panic!("no fisher") };
    let hut = g.world.get_component::<Craft>(f).unwrap().workplace.unwrap();
    g.world.get_component_mut::<Building>(hut).unwrap()
        .stock.insert("fish".into(), 0);
    g.world.add_component(f, villager::Dead);
    g.fisher_tick(4);
    let fish = g.world.get_component::<Building>(hut).unwrap()
        .stock.get("fish").copied().unwrap_or(0);
    assert_eq!(fish, 0, "the river gives the dead no catch");
}

#[test]
fn test_the_heir_works_the_river() {
    let mut g = Game::new();
    let fisher = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Fisher).unwrap_or(false));
    let Some(f) = fisher else { panic!("no fisher") };
    let hut = g.world.get_component::<Craft>(f).unwrap().workplace.unwrap();
    g.world.get_component_mut::<Building>(hut).unwrap()
        .stock.insert("fish".into(), 0);
    // the bench passes to living hands - the heir needn't carry
    // the vocation, the river only asks who's behind the bench
    let heir = g.villagers.iter().copied().find(|&v| v != f
        && !villager::is_dead(&g.world, v)).unwrap();
    g.world.get_component_mut::<Building>(hut).unwrap().worker = Some(heir);
    g.world.add_component(f, villager::Dead);
    g.fisher_tick(4);
    let fish = g.world.get_component::<Building>(hut).unwrap()
        .stock.get("fish").copied().unwrap_or(0);
    assert!(fish > 0, "the heir works the river");
}
