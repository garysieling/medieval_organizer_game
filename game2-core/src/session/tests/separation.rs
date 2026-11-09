use crate::session::*;

#[test]
fn test_a_worn_marriage_separates() {
    let mut g = Game::new();
    let a = g.villagers[0];
    let b = g.villagers[1];
    // the union stands, the bond's worn thin
    g.world.add_component(a, crate::game::relationships::Married { spouse: b, form: 0 });
    g.world.add_component(b, crate::game::relationships::Married { spouse: a, form: 0 });
    g.world.add_component(a, crate::sim::psychology::Bond { target: b, security: 0.1 });
    g.separation_tick(47);
    let form = g.world.get_component::<crate::game::relationships::Married>(a)
        .map(|m| m.form).unwrap_or(0);
    assert!(form > 0, "the union never ended");
}

#[test]
fn test_the_deserter_forfeits_to_the_stayer() {
    let mut g = Game::new();
    let a = g.villagers[0];
    let b = g.villagers[1];
    g.world.add_component(a, crate::game::relationships::Married { spouse: b, form: 0 });
    g.world.add_component(b, crate::game::relationships::Married { spouse: a, form: 0 });
    g.world.add_component(a, crate::sim::psychology::Bond { target: b, security: 0.1 });
    // the deserter's purse drains to the stayer
    if let Some(p) = g.world.get_component_mut::<Purse>(a) { p.gold = 10.0; }
    if let Some(p) = g.world.get_component_mut::<Purse>(b) { p.gold = 0.0; }
    g.separation_tick(47);
    let form = g.world.get_component::<crate::game::relationships::Married>(a)
        .map(|m| m.form).unwrap_or(0);
    if form == 2 { // desertion: the claim fell to the stayer
        let stayer = g.world.get_component::<Purse>(b).map(|p| p.gold).unwrap_or(0.0);
        assert!(stayer > 0.0, "the deserter's claim never fell");
    }
}
