use crate::session::*;

#[test]
fn test_the_sign_rises_on_the_channels() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let mut o = crate::game::org_skills::Organizing::default();
    o.symbol = 0.8;
    o.signal_net = 0.8;
    g.world.add_component(m, o);
    if let Some(org) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !org.members.contains(&m) { org.members.push(m); }
    }
    g.brand_saturation = 0.0;
    g.brand_tick(34);
    assert!(g.brand_saturation > 0.0, "the sign never spread");
}

#[test]
fn test_the_sign_fades_unreinforced() {
    let mut g = Game::new();
    g.brand_saturation = 0.5;
    g.brand_tick(34);
    assert!(g.brand_saturation < 0.5, "the sign never faded: {}", g.brand_saturation);
}
