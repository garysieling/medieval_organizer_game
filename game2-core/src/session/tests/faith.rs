use crate::session::*;
use crate::engine::ecs::System;

fn world_add_routine(g: &mut Game, v: Entity, holy: u64) {
    g.world.add_component(v, routine::Routine {
        day: holy, mode: routine::Mode::Resting, spot: routine::Spot::Home,
        anchor: (0.0, 0.0), face: None, drift: 0.0, leads: false, reached: false });
}

#[test]
fn test_the_office_feeds_devotion_and_the_hungry() {
    let mut g = Game::new();
    let holy = routine::HOLY_EVERY;
    // two worshippers, one of them hungry - the office bonds them and
    // the poor box passes food down
    let (a, b) = (g.villagers[0], g.villagers[1]);
    for v in [a, b] {
        if g.world.get_component::<routine::Routine>(v).is_none() {
            world_add_routine(&mut g, v, holy);
        }
        let r = g.world.get_component_mut::<routine::Routine>(v).unwrap();
        r.mode = routine::Mode::Worshipping;
        r.day = holy;
    }
    g.world.get_component_mut::<Hunger>(a).unwrap().value = 80.0;
    g.world.get_component_mut::<Hunger>(b).unwrap().value = 10.0;
    let d0 = g.world.get_component::<crate::sim::religion::Faith>(a).unwrap().devotion;
    g.service_tick(holy); // the office runs after the day's plans are laid
    let f = g.world.get_component::<crate::sim::religion::Faith>(a).unwrap();
    assert!(f.devotion > d0, "the office grew no devotion: {} -> {}", d0, f.devotion);
    assert!(g.world.get_component::<Hunger>(b).unwrap().value > 10.0,
        "the poor box passed nothing to the hungry");
    // and unkept it fades
    crate::sim::religion::DevotionSystem.run(&mut g.world);
    assert!(g.world.get_component::<crate::sim::religion::Faith>(a).unwrap().devotion
        < g.world.get_component::<crate::sim::religion::Faith>(a).map(|f| f.devotion + 1.0).unwrap_or(0.0));
}
