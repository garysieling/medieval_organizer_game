use crate::session::*;
use crate::game::villager::*;
use crate::sim::illness::*;

#[test]
fn test_starving_hurts_and_eating_well_mends() {
    let mut g = Game::new();
    let v = g.villagers[0];
    // a starving villager wastes a little each day
    g.world.get_component_mut::<Hunger>(v).unwrap().value = 0.0;
    let h0 = g.world.get_component::<Health>(v).unwrap().condition;
    for _ in 0..3 { g.nourish_tick(); }
    let h1 = g.world.get_component::<Health>(v).unwrap().condition;
    assert!(h1 < h0, "starving didn't hurt: {} -> {}", h0, h1);
    // fed and unsick, the body mends toward the nourished mark
    g.world.get_component_mut::<Hunger>(v).unwrap().value = 100.0;
    for _ in 0..40 { g.nourish_tick(); }
    let h2 = g.world.get_component::<Health>(v).unwrap().condition;
    assert!(h2 >= NOURISHED_HEALTH - 0.01, "eating well didn't mend: {}", h2);
    assert!(h2 <= NOURISHED_HEALTH + 0.01, "healed past the mark: {}", h2);
    // but the sick don't mend for a full belly
    let d = g.world.create_entity();
    g.world.add_component(d, Disease { kind: DiseaseKind::Influenza,
        name: "la grippe".into(), patient: v, stage: Stage::Mild,
        severity: 0.4, contagious: true, incubation: 0 });
    let h3 = g.world.get_component::<Health>(v).unwrap().condition;
    for _ in 0..10 { g.nourish_tick(); }
    let h4 = g.world.get_component::<Health>(v).unwrap().condition;
    assert!((h4 - h3).abs() < 0.05, "the sick healed by eating: {} -> {}", h3, h4);
}
