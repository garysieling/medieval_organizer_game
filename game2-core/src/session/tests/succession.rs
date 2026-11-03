use crate::session::*;

#[test]
fn test_the_bench_passes_to_the_apprentice() {
    let mut g = Game::new();
    // a shop master and a junior bound to them
    let master = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v)
            .map(|c| c.workplace.is_some() && c.vocation != Vocation::Farmer).unwrap_or(false)).unwrap();
    let shop = g.world.get_component::<Craft>(master).unwrap().workplace.unwrap();
    let junior = g.villagers.iter().copied().find(|&v|
        v != master && g.world.get_component::<Kid>(v).is_none()
        && g.world.get_component::<Craft>(v).is_some()).unwrap();
    g.world.get_component_mut::<Craft>(junior).unwrap().master = Some(master);
    g.world.get_component_mut::<Craft>(junior).unwrap().workplace = None;
    g.world.get_component_mut::<Purse>(master).unwrap().gold = 10.0;
    let before = g.world.get_component::<Purse>(junior).unwrap().gold;
    g.world.add_component(master, villager::Dead);
    g.succession_tick(1);
    assert_eq!(g.world.get_component::<Craft>(junior).unwrap().workplace, Some(shop),
        "the apprentice takes the bench");
    assert_eq!(g.world.get_component::<Building>(shop).unwrap().worker, Some(junior));
    let after = g.world.get_component::<Purse>(junior).unwrap().gold;
    assert!((after - before - 8.0).abs() < 0.01, "four parts pass, the fifth is the lord's");
    assert!(desires::problems_of(&g.world, junior, 1).contains(&desires::Problem::DeathDuty),
        "the death-duty is grievance");
}

#[test]
fn test_a_shop_with_no_heir_stands_shuttered() {
    let mut g = Game::new();
    // a master with no apprentice, no kin at home, and no journeyman
    let master = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v)
            .map(|c| c.workplace.is_some() && c.vocation != Vocation::Farmer).unwrap_or(false)).unwrap();
    let shop = g.world.get_component::<Craft>(master).unwrap().workplace.unwrap();
    // clear every possible heir path
    let mvoc = g.world.get_component::<Craft>(master).unwrap().vocation;
    for &v in g.villagers.clone().iter() {
        if let Some(c) = g.world.get_component_mut::<Craft>(v) {
            if c.master == Some(master) { c.master = None; }
            if c.vocation == mvoc && v != master { c.vocation = Vocation::Farmer; }
        }
    }
    if let Some(home) = household::home_of(&g.world, master) {
        for &o in g.world.get_component::<Building>(home).unwrap().occupants.clone().iter() {
            if o != master && g.world.get_component::<Kid>(o).is_none() {
                g.world.add_component(o, villager::Dead);
            }
        }
    }
    g.world.add_component(master, villager::Dead);
    g.succession_tick(1);
    assert_eq!(g.world.get_component::<Building>(shop).unwrap().worker, None,
        "no heir - the bench stands empty");
}
