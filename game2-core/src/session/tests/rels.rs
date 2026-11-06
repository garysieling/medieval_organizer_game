use crate::session::*;
use crate::game::relationships::*;

#[test]
fn test_bad_blood_settles_and_the_label_comes_off() {
    let mut g = Game::new();
    let (a, b) = (g.villagers[0], g.villagers[1]);
    // enough quarrels to make them rivals
    for _ in 0..6 {
        record_interaction(&mut g.world, a, Interaction {
            other: b, kind: InteractionKind::Conflict, intensity: 0.8 });
    }
    let rel = find_relationship(&g.world, a, b).expect("no tie");
    assert_eq!(g.world.get_component::<Relationship>(rel).unwrap().kind, RelKind::Rival);
    // a year and a half of quiet days settles it
    for t in 0..560 { g.rel_tick(); let _ = t; }
    let r = g.world.get_component::<Relationship>(rel).unwrap();
    assert_ne!(r.kind, RelKind::Rival, "the rivalry outlived the bad blood");
    assert!(r.strength > -0.3, "the strength held at {}", r.strength);
    // and a warm tie cools toward acquaintance too
    for _ in 0..9 {
        record_interaction(&mut g.world, a, Interaction {
            other: b, kind: InteractionKind::SharedMeal, intensity: 0.8 });
    }
    let s0 = g.world.get_component::<Relationship>(rel).unwrap().strength;
    for _ in 0..30 { g.rel_tick(); }
    let s1 = g.world.get_component::<Relationship>(rel).unwrap().strength;
    assert!(s1 < s0, "a warm tie didn't cool: {} -> {}", s0, s1);
}
