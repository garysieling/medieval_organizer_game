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
    g.influence_tick(1);
    let pool = g.world.get_component::<Organization>(oe).unwrap().influence_pool;
    assert!(pool > 0.0, "the roll holds no sway");
    g.world.get_component_mut::<Health>(v).unwrap().condition = 0.0;
    g.influence_tick(1);
    let o = g.world.get_component::<Organization>(oe).unwrap();
    assert!(!o.members.contains(&v), "the dead kept their seat on the roll");
}

#[test]
fn test_a_muscle_unused_goes_soft() {
    let mut g = Game::new();
    let oe = g.seats[0].org;
    g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(oe)
        .unwrap().capacity = 0.3;
    // a season of doing nothing with it
    for t in 1..40 { g.influence_tick(t); }
    let p = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(oe).unwrap();
    assert!(p.capacity < 0.3, "the muscle kept its strength unused");
    assert!(g.log.iter().any(|l| l.contains("gone soft")), "the softening went unremarked");
    // using it keeps it - a town of action stays fit
    g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(oe)
        .unwrap().capacity = 0.3;
    for t in 41..46 {
        crate::sim::organized_power::record_action(&mut g.world, oe,
            crate::sim::organized_power::ActionQuality::Transactional);
        g.influence_tick(t);
    }
    let p2 = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(oe).unwrap();
    assert!(p2.capacity >= 0.3, "a used muscle still lost it: {}", p2.capacity);
}

#[test]
fn test_the_panel_reads_the_muscle() {
    let mut g = Game::new();
    // a used muscle and a turnout craft to name
    if let Some(p) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
        p.capacity = 0.4;
        p.idle_ticks = 0;
    }
    let m = g.ui_model_at(0);
    let pw = m.panel.power.as_deref().unwrap_or("");
    // a word, not the gauge: the exact numbers stay the probe's
    assert!(pw.contains("muscle") && pw.contains("call") && pw.contains("fresh"),
        "the muscle's read is wrong: {:?}", pw);
    assert!(!pw.contains("0.4"), "the gauge leaked: {:?}", pw);
}
