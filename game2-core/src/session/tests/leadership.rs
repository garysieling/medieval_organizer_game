//! Leaders out of followers (#1268): who brought whom, the ladder, reach
use crate::session::*;
use crate::game::leadership::{self, Developed, Reach, Rung};

fn tie(g: &mut Game, a: Entity, b: Entity, strength: f32) {
    relationships::record_interaction(&mut g.world, a, relationships::Interaction {
        other: b, kind: relationships::InteractionKind::Conversation, intensity: 0.3 });
    relationships::find_relationship(&g.world, a, b)
        .and_then(|r| g.world.get_component_mut::<relationships::Relationship>(r))
        .unwrap().strength = strength;
}

fn members(g: &mut Game, ms: &[Entity]) {
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = ms.to_vec();
}

fn join(g: &mut Game, r: Entity) -> bool {
    let o = g.org;
    crate::sim::community::recruit(&mut g.world, o, r, 0.9)
}

/// A stranger with no pull to any cause, for the leader to sit down with
fn free_adult(g: &Game, skip: &[Entity]) -> Entity {
    *g.villagers.iter().find(|&&v| !skip.contains(&v)
        && g.world.get_component::<crate::game::vocation::Kid>(v).is_none()
        && g.world.get_component::<crate::session::country::Outlander>(v).is_none()
        && g.world.get_component::<crate::game::prophetic::Follower>(v).is_none()
        && !crate::game::villager::is_dead(&g.world, v)).unwrap()
}

#[test]
fn test_a_recruit_is_credited_to_the_member_who_knows_them_best() {
    let mut g = Game::new();
    let (a, b, r) = (g.villagers[0], g.villagers[1], g.villagers[2]);
    members(&mut g, &[a, b]);
    tie(&mut g, a, r, 0.5);
    tie(&mut g, b, r, 0.8);
    assert!(join(&mut g, r));
    assert_eq!(g.world.get_component::<Developed>(r).map(|d| d.by), Some(b));
    assert_eq!(leadership::rung(&g.world, g.org, b), Rung::Leader);
    assert_eq!(leadership::rung(&g.world, g.org, a), Rung::Member);
    assert_eq!(leadership::rung(&g.world, g.org, r), Rung::Member);
    let stranger = free_adult(&g, &[a, b, r]);
    assert_eq!(leadership::rung(&g.world, g.org, stranger), Rung::Helpless);
}

#[test]
fn test_a_recruit_no_member_knows_is_the_organizers() {
    let mut g = Game::new();
    let (a, r) = (g.villagers[0], g.villagers[2]);
    members(&mut g, &[a]);
    tie(&mut g, a, r, 0.2);
    assert!(join(&mut g, r));
    let lead = g.world.get_component::<Organization>(g.org).unwrap().leader;
    assert_eq!(g.world.get_component::<Developed>(r).map(|d| d.by), Some(lead));
    // the organizer's own recruits don't count as leaders grown
    assert_eq!(leadership::leaders_grown(&g.world, g.org), 0);
}

#[test]
fn test_the_campaign_and_the_card_show_the_ladder() {
    let mut g = Game::new();
    let (a, r) = (g.villagers[0], g.villagers[2]);
    members(&mut g, &[a]);
    let o = g.org;
    crate::game::campaign::found_for_test(&mut g.world, o);
    tie(&mut g, a, r, 0.7);
    assert!(join(&mut g, r));
    let m = g.ui_model();
    assert_eq!(m.panel.campaign.as_ref().map(|c| c.leaders), Some(1));
    g.seats[0].selected_target = 0;
    g.seats[0].picked = true;
    let card = g.villager_inspector_at(0);
    let text: Vec<_> = card.lines.iter().map(|l| l.text.clone()).collect();
    assert!(text.iter().any(|t| t.contains("helpless -> member -> LEADER (brought in 1)")), "{:?}", text);
}

#[test]
fn test_a_leader_sent_out_brings_in_a_warm_ask_on_their_own() {
    let mut g = Game::new();
    let (a, r) = (g.villagers[0], g.villagers[2]);
    members(&mut g, &[a]);
    tie(&mut g, a, r, 0.7);
    assert!(join(&mut g, r));
    let t = free_adult(&g, &[a, r, g.player]);
    tie(&mut g, a, t, 0.9);
    // only a leader gets the button
    g.seats[0].picked = true;
    g.seats[0].selected_target = g.villagers.iter().position(|&v| v == r).unwrap();
    g.update_ctx_actions();
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::Reach), "a plain member isn't sent out");
    g.seats[0].selected_target = 0;
    g.update_ctx_actions();
    let i = g.seats[0].ctx_actions.iter().position(|&c| c == CtxAct::Reach).expect("a leader can be sent out");
    g.ctx_act(i);
    let reach = *g.world.get_component::<Reach>(a).expect("sent out");
    assert_eq!(reach.target, t);
    // nothing happens before the day comes
    assert!(leadership::run_reach(&mut g.world, a, reach.due - 1).is_none());
    assert_eq!(leadership::run_reach(&mut g.world, a, reach.due).map(|x| x.1),
        Some(leadership::ReachOutcome::Joined));
    assert!(g.world.get_component::<Organization>(g.org).unwrap().members.contains(&t));
    assert_eq!(g.world.get_component::<Developed>(t).map(|d| d.by), Some(a));
    assert_eq!(leadership::brought(&g.world, g.org, a), 2);
    assert!(g.world.get_component::<Reach>(a).is_none(), "the errand's done");
}

#[test]
fn test_a_cold_ask_from_a_leader_is_heard_but_not_won() {
    let mut g = Game::new();
    let (a, r) = (g.villagers[0], g.villagers[2]);
    members(&mut g, &[a]);
    tie(&mut g, a, r, 0.7);
    assert!(join(&mut g, r));
    let t = free_adult(&g, &[a, r, g.player]);
    let org = g.org;
    g.world.add_component(a, Reach { org, target: t, due: g.tick });
    let tick = g.tick;
    assert_eq!(leadership::run_reach(&mut g.world, a, tick).map(|x| x.1),
        Some(leadership::ReachOutcome::Heard));
    assert!(!g.world.get_component::<Organization>(g.org).unwrap().members.contains(&t));
}

#[test]
fn test_the_daily_loop_runs_a_due_reach_and_logs_it() {
    let mut g = Game::new();
    let (a, r) = (g.villagers[0], g.villagers[2]);
    members(&mut g, &[a]);
    tie(&mut g, a, r, 0.7);
    assert!(join(&mut g, r));
    let t = free_adult(&g, &[a, r, g.player]);
    tie(&mut g, a, t, 0.9);
    let (org, tick) = (g.org, g.tick);
    g.world.add_component(a, Reach { org, target: t, due: tick + 1 });
    g.livelihood_tick(tick + 1);
    assert!(g.world.get_component::<Reach>(a).is_none());
    assert!(g.log.iter().any(|l| l.contains("on their own")), "the log hears of it");
}

#[test]
fn test_a_jailed_target_cancels_the_reach() {
    let mut g = Game::new();
    let (a, r) = (g.villagers[0], g.villagers[2]);
    members(&mut g, &[a]);
    tie(&mut g, a, r, 0.7);
    assert!(join(&mut g, r));
    let t = free_adult(&g, &[a, r, g.player]);
    tie(&mut g, a, t, 0.9);
    let (org, tick) = (g.org, g.tick);
    g.world.add_component(a, Reach { org, target: t, due: tick });
    g.world.add_component(t, crate::game::dungeon::Imprisoned { since: tick, charge: "sedition".into() });
    assert!(leadership::run_reach(&mut g.world, a, tick).is_none());
    assert!(g.world.get_component::<Reach>(a).is_none(), "the errand's off");
    assert!(!g.world.get_component::<Organization>(g.org).unwrap().members.contains(&t));
    assert_ne!(leadership::reach_target(&g.world, g.org, a, &g.villagers.clone()), Some(t), "a prisoner isn't asked");
}

#[test]
fn test_a_reach_never_takes_a_first_developers_credit() {
    let mut g = Game::new();
    let (a, r) = (g.villagers[0], g.villagers[2]);
    members(&mut g, &[a]);
    tie(&mut g, a, r, 0.7);
    assert!(join(&mut g, r));
    let t = free_adult(&g, &[a, r, g.player]);
    tie(&mut g, a, t, 0.9);
    // credited once already, by someone in another org
    let other = g.world.create_entity();
    let first = g.villagers[3];
    g.world.add_component(t, Developed { org: other, by: first });
    let (org, tick) = (g.org, g.tick);
    g.world.add_component(a, Reach { org, target: t, due: tick });
    assert_eq!(leadership::run_reach(&mut g.world, a, tick).map(|x| x.1), Some(leadership::ReachOutcome::Joined));
    assert_eq!(g.world.get_component::<Developed>(t).map(|d| d.by), Some(first));
    assert_eq!(leadership::brought(&g.world, g.org, first), 0, "another org's credit isn't this one's");
}
