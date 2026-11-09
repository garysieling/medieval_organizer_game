use crate::session::*;

#[test]
fn test_the_weavers_skill_scales_the_cloth() {
    let mut g = Game::new();
    g.cloth_quality = 0.5;
    let m = g.villagers[0];
    if let Some(c) = g.world.get_component_mut::<crate::game::vocation::Craft>(m) {
        c.vocation = crate::game::vocation::Vocation::Weaver;
        c.skill = 0.8;
    }
    g.textile_tick(52);
    assert!(g.cloth_quality > 0.5, "the skill never scaled: {}", g.cloth_quality);
}

#[test]
fn test_the_good_cloth_comforts() {
    let mut g = Game::new();
    g.cloth_quality = 0.7;
    let m = g.villagers[0];
    g.world.add_component(m, crate::sim::illness::Health {
        condition: 0.5, immunity: 0.5 });
    g.textile_tick(52);
    let after = g.world.get_component::<crate::sim::illness::Health>(m)
        .map(|h| h.condition).unwrap_or(0.5);
    assert!(after > 0.5, "the good cloth never comforted: {}", after);
}
