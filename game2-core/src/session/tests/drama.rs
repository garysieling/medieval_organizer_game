use crate::session::*;
use crate::session::model::*;

/// A campaign for the seat's org, founded today
fn found(g: &mut Game) -> Entity {
    let org = g.org;
    crate::game::campaign::found_campaign(&mut g.world, org,
        &crate::game::campaign::IssueVote { winner: "the bread".into(), ballots: vec![] },
        crate::game::campaign::Framing::SharedGrievance,
        crate::game::campaign::DecisionRule::Majority, g.tick)
}

#[test]
fn phase_follows_campaign() {
    let mut g = Game::new();
    // no cause yet: the arc's still listening
    let d = g.ui_model_at(0).panel.drama;
    assert_eq!(d.phase, CampaignPhaseV::Listening, "no campaign must read listening");
    let c = found(&mut g);
    for (sim, wire) in [
        (crate::game::campaign::CampaignPhase::IssueCut, CampaignPhaseV::IssueCut),
        (crate::game::campaign::CampaignPhase::Research, CampaignPhaseV::Research),
        (crate::game::campaign::CampaignPhase::Action, CampaignPhaseV::Action),
        (crate::game::campaign::CampaignPhase::Evaluate, CampaignPhaseV::Evaluate),
        (crate::game::campaign::CampaignPhase::Listening, CampaignPhaseV::Listening),
    ] {
        g.world.get_component_mut::<crate::game::campaign::Campaign>(c).unwrap().phase = sim;
        assert_eq!(g.ui_model_at(0).panel.drama.phase, wire, "{sim:?} mapped wrong");
    }
}

#[test]
fn next_assembly_counts_down_from_publish_gap() {
    let mut g = Game::new();
    let c = found(&mut g);
    g.world.add_component(c, crate::game::campaign::Assembly {
        cadence_days: 60, last_held: 0, announced: Some(g.tick),
        agenda: vec![("the bread".into(), crate::game::campaign::PledgeCheck::Action { from: 0 })], last_turnout: 0,
        musterered: vec![], held: 0, pledges: vec![],
    });
    let d = g.ui_model_at(0).panel.drama;
    let next = d.next.expect("an announced assembly must be next");
    assert_eq!(next.kind, "assembly");
    assert_eq!(next.at as u64, g.tick + crate::game::campaign::PUBLISH_GAP,
        "the sitting lands at announced + PUBLISH_GAP");
    assert_eq!(next.in_days, crate::game::campaign::PUBLISH_GAP as f32);
}

#[test]
fn next_campaign_meeting_due() {
    let mut g = Game::new();
    let c = found(&mut g);
    // the cadence the organizer ratified: last met today, next in three
    crate::game::campaign::set_cadence(&mut g.world, c, 3, g.tick);
    let (held, cad) = {
        let m = g.world.get_component::<crate::game::campaign::CampaignMeeting>(c).unwrap();
        (m.last_held, m.cadence_days)
    };
    let d = g.ui_model_at(0).panel.drama;
    let next = d.next.expect("a meeting on cadence must be next");
    assert_eq!(next.kind, "campaign meeting", "got {next:?}");
    assert_eq!(next.at as u64, held + cad);
    assert_eq!(next.in_days, (held + cad) as f32 - g.tick as f32);
}

#[test]
fn live_march_progress_rises() {
    let mut g = Game::new();
    g.formation = Some(formation::Formation {
        kind: formation::FormationKind::March, seat: 0, born: g.tick,
        members: vec![g.seats[0].player],
        route: vec![(0.0, 0.0), (10.0, 0.0), (20.0, 0.0), (30.0, 0.0)],
        at: 0, stuck: 0, waited: 0, leg0: 0.0,
        cheered: crate::engine::det::HashSet::default(), dest: None,
    });
    let live = g.ui_model_at(0).panel.drama.live.expect("the column must be live");
    assert_eq!(live.kind, "march");
    assert_eq!(live.progress, 0.0);
    g.formation.as_mut().unwrap().at = 2;
    let live = g.ui_model_at(0).panel.drama.live.unwrap();
    assert_eq!(live.progress, 0.5, "two of four waypoints walked");
}

#[test]
fn last_outcome_verdict_mapping() {
    use crate::game::campaign_arc::{record_outcome, OutcomeKind};
    use crate::game::actions::TargetResponse;
    use crate::game::reactions::Reaction;
    let mut g = Game::new();
    let c = found(&mut g);
    // one per Verdict: the design's table, newest stamp wins
    for (kind, want, want_kind) in [
        (OutcomeKind::Townhall(TargetResponse::Carries), Verdict::Win, "townhall"),
        (OutcomeKind::Townhall(TargetResponse::Concedes), Verdict::Partial, "townhall"),
        (OutcomeKind::Townhall(TargetResponse::Deflects), Verdict::Stall, "townhall"),
        (OutcomeKind::Townhall(TargetResponse::Refuses), Verdict::Loss, "townhall"),
        (OutcomeKind::March(Reaction::Crackdown), Verdict::Crackdown, "march"),
    ] {
        record_outcome(&mut g.world, c, kind, g.tick);
        let last = g.ui_model_at(0).panel.drama.last.expect("a stamp must surface");
        assert_eq!(last.verdict, want, "{kind:?} mapped wrong");
        assert_eq!(last.kind, want_kind);
        assert_eq!(last.day as u64, g.tick);
    }
    // a newer stamp beats an older one whatever it says
    record_outcome(&mut g.world, c, OutcomeKind::March(Reaction::Ignored), g.tick + 1);
    g.tick += 1;
    assert_eq!(g.ui_model_at(0).panel.drama.last.unwrap().verdict, Verdict::Stall);
}

#[test]
fn pressure_matches_org_status() {
    let mut g = Game::new();
    let oe = g.org;
    g.world.add_component(oe, crate::game::reactions::GuardsPosted { knights: 3, since: g.tick });
    g.world.add_component(oe, crate::game::reactions::AssemblyBan { cap: 2 });
    let (member, leader) = {
        let org = g.world.get_component::<Organization>(oe).unwrap();
        (g.villagers.iter().copied().find(|&v| v != org.leader).unwrap(), org.leader)
    };
    g.world.get_component_mut::<Organization>(oe).unwrap().members.push(member);
    g.world.add_component(member, crate::game::dungeon::Imprisoned {
        since: g.tick, charge: "sedition".into() });
    g.world.add_component(leader, crate::game::dungeon::Imprisoned {
        since: g.tick, charge: "sedition".into() });
    let m = g.ui_model_at(0);
    let p = m.panel.drama.pressure;
    assert!(p.guards_posted && p.assembly_ban && p.leader_taken,
        "the strings show it, the fields must too: {p:?}");
    let held_members = m.panel.members.iter().filter(|s| s.contains("(held)")).count() as u32;
    assert_eq!(p.members_held, held_members,
        "members_held must count the same folk the panel marks (held)");
    let status = m.panel.org_status.join(" ");
    assert!(status.contains("knights posted"), "{status}");
    assert!(status.contains("assembly ban"), "{status}");
}

#[test]
fn drama_does_not_leak_hidden_state() {
    // two worlds identical in public; the hidden knobs differ: the
    // offer's binding flag, the plant, the true organized power
    let mut a = Game::new();
    let mut b = Game::new();
    for g in [&mut a, &mut b] {
        let c = found(g);
        g.world.add_component(c, crate::game::campaign::Assembly {
            cadence_days: 60, last_held: 0, announced: Some(g.tick),
            agenda: vec![("the bread".into(), crate::game::campaign::PledgeCheck::Action { from: 0 })], last_turnout: 0,
            musterered: vec![], held: 0, pledges: vec![],
        });
        crate::game::campaign_arc::record_outcome(&mut g.world, c,
            crate::game::campaign_arc::OutcomeKind::Pledge { kept: true }, g.tick);
    }
    let offer = |g: &Game, binding: bool| crate::game::campaign::SeatOffer {
        from: g.villagers[1], role: crate::game::campaign::Role::TurnoutCoordinator,
        binding, offered: g.tick };
    a.seat_offer = Some(offer(&a, false));
    b.seat_offer = Some(offer(&b, true));
    b.plant = Some(b.villagers[2]);
    b.world.add_component(b.org, crate::sim::organized_power::OrganizedPower {
        capacity: 0.9, idle_ticks: 0 });
    for _ in 0..30 { a.update(); b.update(); }
    let da = serde_json::to_string(&a.ui_model_at(0).panel.drama).unwrap();
    let db = serde_json::to_string(&b.ui_model_at(0).panel.drama).unwrap();
    assert_eq!(da, db, "the director's channel must not leak hidden state");
}
