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
    // the ladder wraps to a second card row at the manuscript metric
    let joined = text.join(" ");
    assert!(joined.contains("helpless -> member -> LEADER (brought in 1)"), "{:?}", text);
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

/// The meeting's work leaves its marks (#1345): a kept part greens the
/// keeper and yellows everyone who watched it done
#[test]
fn test_a_kept_part_marks_the_holder_and_the_room() {
    use crate::game::leadership::{job_score, Job, JobMarks};
    let mut g = Game::new();
    let (holder, watcher, host) = (g.villagers[0], g.villagers[1], g.villagers[2]);
    let mut m = Meeting {
        label: "Bram's house".into(), venue: (10, 10), born: g.tick,
        invited: vec![holder, watcher, host],
        spots: vec![(10, 10); 3], paths: vec![None; 3],
        frames: 0, talk_from: None,
        lines: vec![(host, "Friends - you came.".into())],
        talk_id: 0, seat: 0,
        roles: (false, false, true, false, false),
        role_people: [None, None, Some(holder), None, None],
        bought: vec![], probe: false, testimony: None,
        agenda: None, steer_at: None, steer_wait: 0, steered: false, promised: vec![],
        marks: vec![], impromptu: false, pledged: vec![],
    };
    g.meeting_beats(&mut m);
    let hm = g.world.get_component::<JobMarks>(holder).expect("the keeper's marked");
    assert_eq!(hm.done[Job::Opener.ix()], 1, "they held the opener");
    let wm = g.world.get_component::<JobMarks>(watcher).expect("a witness is marked");
    assert_eq!((wm.seen[Job::Opener.ix()], wm.done[Job::Opener.ix()]), (1, 0),
        "they watched it done");
    assert!(job_score(&g.world, holder) > job_score(&g.world, watcher),
        "holding outweighs watching");
}

/// The marks steady a leader's own ask (#1345): on the same cold tie,
/// the member who's held the work brings the sit-down in where the
/// unmarked one is only heard
#[test]
fn test_the_marks_steady_a_leaders_own_ask() {
    let outcome = |marked: bool| {
        let mut g = Game::new();
        let (a, r) = (g.villagers[0], g.villagers[2]);
        members(&mut g, &[a]);
        tie(&mut g, a, r, 0.7);
        assert!(join(&mut g, r));
        let t = free_adult(&g, &[a, r, g.player]);
        // under the ask: the sit-down's own warmth doesn't reach it alone
        tie(&mut g, a, t, 0.3);
        if marked {
            for _ in 0..5 {
                leadership::mark_performed(&mut g.world, a, leadership::Job::Chair);
            }
        }
        let org = g.org;
        g.world.add_component(a, Reach { org, target: t, due: g.tick });
        let tick = g.tick;
        leadership::run_reach(&mut g.world, a, tick).map(|x| x.1)
    };
    assert_eq!(outcome(false), Some(leadership::ReachOutcome::Heard));
    assert_eq!(outcome(true), Some(leadership::ReachOutcome::Joined));
}

/// A member marked enough stops waiting to be sent (#1345): the daily
/// tick hands them their own sit-down, and the unmarked keep waiting
#[test]
fn test_a_marked_member_starts_their_own_reach() {
    let mut g = Game::new();
    let (a, first, gone) = (g.villagers[0], g.villagers[2], g.villagers[3]);
    members(&mut g, &[first]);
    assert!(join(&mut g, a), "a joins the cause - developed by the member who knew them");
    assert!(g.world.get_component::<Developed>(a).is_some());
    for _ in 0..5 {
        leadership::mark_performed(&mut g.world, a, leadership::Job::Turnout);
        leadership::mark_performed(&mut g.world, gone, leadership::Job::Turnout);
    }
    // a member who dropped off the roll doesn't get sent out either
    let tick = g.tick;
    g.livelihood_tick(tick);
    let reach = g.world.get_component::<Reach>(a).expect("marked enough, they pick their own");
    assert!(!g.world.get_component::<crate::sim::community::Organization>(g.org)
        .unwrap().members.contains(&reach.target), "they aim past the fold");
    assert!(g.world.get_component::<Reach>(first).is_none(), "the unmarked still wait to be sent");
    assert!(g.world.get_component::<Reach>(gone).is_none(), "off the roll, no reach");
}

/// A founding-vote member never got a `Developed` (#1345): the marks
/// still find their org by the roll they're on
#[test]
fn test_a_founding_member_starts_their_own_reach() {
    let mut g = Game::new();
    let first = g.villagers[2];
    members(&mut g, &[first]);
    assert!(g.world.get_component::<Developed>(first).is_none(), "enrolled, not developed");
    for _ in 0..5 {
        leadership::mark_performed(&mut g.world, first, leadership::Job::Chair);
    }
    let tick = g.tick;
    g.livelihood_tick(tick);
    assert!(g.world.get_component::<Reach>(first).is_some(), "the roll is the proof");
}

/// The card carries the marks (#1345): watched jobs yellow, held green
#[test]
fn test_the_card_rows_show_seen_and_held() {
    let mut g = Game::new();
    let a = g.villagers[0];
    leadership::mark_observed(&mut g.world, a, leadership::Job::Song);
    leadership::mark_performed(&mut g.world, a, leadership::Job::Chair);
    g.seats[0].selected_target = 0;
    g.seats[0].picked = true;
    let card = g.villager_inspector_at(0);
    let seen = card.lines.iter().find(|l| l.text == "the song").expect("a seen row");
    let held = card.lines.iter().find(|l| l.text == "the chair").expect("a held row");
    assert_eq!(seen.rgb, (215, 190, 90), "yellow for seen it");
    assert_eq!(held.rgb, (110, 185, 90), "green for done it");
}
