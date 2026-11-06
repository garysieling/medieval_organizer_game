use crate::session::*;
use crate::game::relationships::*;

#[test]
fn test_a_trusted_ear_can_be_won_and_the_rolls_count() {
    let mut g = Game::new();
    let v = g.villagers[0];
    let oe = g.seats[0].org;
    let pe = g.seats[0].player;
    // a thin tie won't win them
    assert!(!crate::sim::community::recruit(&mut g.world, oe, v, 0.3));
    // real trust does
    for _ in 0..12 {
        record_interaction(&mut g.world, pe, Interaction {
            other: v, kind: InteractionKind::Conversation, intensity: 1.0 });
    }
    let trust = find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component::<Relationship>(r)).map(|r| r.strength).unwrap_or(0.0);
    assert!(trust >= crate::sim::community::RECRUIT_MIN_TRUST, "the tie never deepened: {}", trust);
    assert!(crate::sim::community::recruit(&mut g.world, oe, v, trust), "they wouldn't sign on");
    // the rolls pool their sway, and the dead leave them
    g.world.add_component(v, Citizen { influence: 0.7, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
    g.influence_tick();
    let pool = g.world.get_component::<Organization>(oe).unwrap().influence_pool;
    assert!(pool > 0.0, "the roll holds no sway");
    g.world.get_component_mut::<Health>(v).unwrap().condition = 0.0;
    g.influence_tick();
    let o = g.world.get_component::<Organization>(oe).unwrap();
    assert!(!o.members.contains(&v), "the dead kept their seat on the roll");
}
