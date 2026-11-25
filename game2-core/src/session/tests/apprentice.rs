use crate::session::*;

/// A grown junior (no master yet) and a free master of the same trade:
/// the indices into `villagers` for each
fn junior_and_master(g: &mut Game) -> (usize, usize) {
    let grown: Vec<usize> = (0..g.villagers.len())
        .filter(|&i| {
            let v = g.villagers[i];
            g.world.get_component::<Kid>(v).is_none()
                && g.world.get_component::<Craft>(v).is_some()
                && !villager::is_dead(&g.world, v)
                && !crate::game::dungeon::unavailable(&g.world, v)
        })
        .collect();
    let (ji, mi) = (grown[0], grown[1]);
    let voc = g.world.get_component::<Craft>(g.villagers[ji]).unwrap().vocation;
    if let Some(c) = g.world.get_component_mut::<Craft>(g.villagers[ji]) {
        c.skill = 0.5; c.master = None;
    }
    if let Some(c) = g.world.get_component_mut::<Craft>(g.villagers[mi]) {
        c.vocation = voc; c.skill = 0.9; c.master = None;
    }
    (ji, mi)
}

/// The Pair up button's gone (#1598): a picked junior with a master of
/// their trade to hand still draws no button on the ctx row
#[test]
fn test_no_pair_up_button_whatever_the_junior() {
    let mut g = Game::new();
    let (ji, _) = junior_and_master(&mut g);
    g.seats[0].selected_target = ji;
    g.seats[0].selected_building = None;
    g.update_ctx_actions_at(0);
    assert!(g.seats[0].ctx_actions.iter().all(|&a| ctx_button(a).label != "Pair up"),
        "the Pair up button's back");
}

/// Agents keep the brokering (#1598): a villager seat asking pairs the
/// junior with a free master of their trade - and refuses where there's
/// no one to pair them with
#[test]
fn test_an_agent_pairs_a_junior_with_a_master() {
    let mut g = Game::new();
    let (ji, mi) = junior_and_master(&mut g);
    let junior = g.villagers[ji];
    let master = g.villagers[mi];
    let who = g.villagers.iter().copied()
        .find(|&v| v != junior && v != master
            && g.world.get_component::<Kid>(v).is_none()
            && !villager::is_dead(&g.world, v)
            && !crate::game::dungeon::unavailable(&g.world, v))
        .expect("a grown asker");
    let seat = g.add_agent_seat(who);
    g.agent_command(seat, AgentCmd::Context {
        action: AgentAction::PairUp, target: Some(junior),
    }).unwrap();
    assert_eq!(g.world.get_component::<Craft>(junior).unwrap().master, Some(master),
        "the pairing never sealed");
    // no master free for the trade, and the ask is refused
    if let Some(c) = g.world.get_component_mut::<Craft>(master) { c.skill = 0.2; }
    if let Some(c) = g.world.get_component_mut::<Craft>(junior) { c.master = None; }
    let err = g.agent_command(seat, AgentCmd::Context {
        action: AgentAction::PairUp, target: Some(junior),
    }).unwrap_err();
    assert!(!err.is_empty(), "a masterless trade should refuse");
}

#[test]
fn test_the_term_accrues_under_a_master() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let master = g.villagers[1];
    if let Some(c) = g.world.get_component_mut::<crate::game::vocation::Craft>(m) {
        c.master = Some(master);
        c.apprentice_years = 3;
        c.skill = 0.2;
    }
    g.apprentice_tick(54);
    let years = g.world.get_component::<crate::game::vocation::Craft>(m)
        .map(|c| c.apprentice_years).unwrap_or(3);
    assert!(years > 3, "the term never accrued: {}", years);
}

#[test]
fn test_the_graduation_lifts_the_skill() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let master = g.villagers[1];
    if let Some(c) = g.world.get_component_mut::<crate::game::vocation::Craft>(m) {
        c.master = Some(master);
        c.apprentice_years = 6;
        c.skill = 0.2;
    }
    g.apprentice_tick(54);
    let skill = g.world.get_component::<crate::game::vocation::Craft>(m)
        .map(|c| c.skill).unwrap_or(0.2);
    assert!(skill >= 0.4, "the graduation never lifted: {}", skill);
    assert!(g.log.iter().any(|l| l.contains("the journeyman's own ceremony")),
        "the ceremony never logged");
}
