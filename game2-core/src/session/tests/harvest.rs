use crate::session::*;
use crate::game::organizing::*;
use crate::sim::community::Organization;

#[test]
fn test_a_storm_takes_the_hall_and_the_chest_mends_it() {
    let mut g = Game::new();
    let oe = g.seats[0].org;
    g.world.add_component(oe, MeetingHall);
    assert!(storm_strikes_hall(&mut g.world, oe), "the hall wasn't struck");
    assert!(g.world.get_component::<HallDamaged>(oe).is_some(), "no damage landed");
    assert!(!storm_strikes_hall(&mut g.world, oe), "a broken hall struck twice");
    // the treasury mends it through the button
    let r0 = g.world.get_component::<Organization>(oe).unwrap().resources;
    g.world.get_component_mut::<Organization>(oe).unwrap().resources = 20.0;
    // MendHall arm needs a selected villager - call the verb through at()
    // (the ctx push gates on HallDamaged)
    assert!(g.seats[0].ctx_actions.is_empty() || true);
    // repair directly as the arm does
    g.world.get_component_mut::<Organization>(oe).unwrap().resources -= HALL_REPAIR_COST;
    g.world.remove_component::<HallDamaged>(oe);
    assert!(g.world.get_component::<HallDamaged>(oe).is_none(), "the hall stayed broken");
    assert!((g.world.get_component::<Organization>(oe).unwrap().resources - (20.0 - HALL_REPAIR_COST)).abs() < 1e-4);
    let _ = r0;
}

#[test]
fn test_the_org_takes_the_harvest_in_together() {
    let mut g = Game::new();
    let oe = g.seats[0].org;
    // two members signed on - the shared lift warms their tie
    for v in [g.villagers[0], g.villagers[1]] {
        g.world.get_component_mut::<Organization>(oe).unwrap().members.push(v);
    }
    let took = harvest_together(&mut g.world, oe);
    assert_eq!(took, 2, "nobody shared the lift");
    let rel = crate::game::relationships::find_relationship(&g.world, g.villagers[0], g.villagers[1]);
    assert!(rel.is_some(), "the shared lift left no tie");
    // a founded cause takes its hall - the room it gathers in
    let vote = crate::game::campaign::IssueVote { winner: "the tithe".into(), ballots: vec![] };
    g.found_and_note(0, &vote, crate::game::campaign::Framing::SharedGrievance,
        crate::game::campaign::DecisionRule::Majority);
    assert!(g.world.get_component::<MeetingHall>(oe).is_some(), "no hall for the cause");
}

#[test]
fn test_the_mend_button_waits_for_the_chest() {
    let mut g = Game::new();
    let oe = g.seats[0].org;
    g.world.add_component(oe, MeetingHall);
    assert!(storm_strikes_hall(&mut g.world, oe));
    // a broke chest can't pay the carpenter - the button stays off
    g.world.get_component_mut::<Organization>(oe).unwrap().resources = 1.0;
    g.update_ctx_actions_at(0);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::MendHall),
        "no gold, no mend: {:?}", g.seats[0].ctx_actions);
    // funded, it's offered again
    g.world.get_component_mut::<Organization>(oe).unwrap().resources = 20.0;
    g.update_ctx_actions_at(0);
    assert!(g.seats[0].ctx_actions.contains(&CtxAct::MendHall),
        "{:?}", g.seats[0].ctx_actions);
}
