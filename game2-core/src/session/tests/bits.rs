use crate::session::*;

#[test]
fn test_two_who_were_there_carry_the_story_between_them() {
    let mut g = Game::new();
    let (a, b, subject) = (g.villagers[10], g.villagers[11], g.villagers[12]);
    // both were there: the same memory lands in both their histories
    for v in [a, b] {
        crate::sim::psychology::record_episode(&mut g.world, v,
            crate::sim::psychology::Episode { with: subject,
                kind: crate::sim::psychology::EpisodeKind::Loss });
    }
    g.bits_tick(1);
    let carries = |e: Entity, g: &Game| g.world
        .get_component::<crate::sim::psychology::SharedBits>(e)
        .map(|s| s.bits.iter().any(|bit| bit.subject == subject)).unwrap_or(false);
    assert!(carries(a, &g) && carries(b, &g), "the shared memory never became a bit");
    // and when next they meet they trade it - the bit is told
    g.bits_tick(2);
    let told = |e: Entity, g: &Game| g.world
        .get_component::<crate::sim::psychology::SharedBits>(e)
        .map(|s| s.bits.iter().all(|bit| bit.told || bit.subject != subject)).unwrap_or(false);
    assert!(told(a, &g) || told(b, &g), "the retelling never happened");
}
