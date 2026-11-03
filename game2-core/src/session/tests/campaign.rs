use crate::session::*;

#[test]
fn test_the_harvesters_take_a_vote_and_then_they_meet() {
    let mut g = Game::new();
    // three members sworn in, all taxed, and the issue heard - the
    // vote happens (an org with no stake in the cause can't found one)
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    let msg = g.maybe_found_campaign(0).unwrap();
    assert!(msg.contains("they'll fight for the taxes"), "{}", msg);
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();

    // the panel shows the fight, its seats empty
    let m = g.ui_model();
    let cv = m.panel.campaign.as_ref().unwrap();
    assert_eq!(cv.issue, "the taxes");
    assert_eq!(cv.seats.len(), 3);
    assert!(cv.seats.iter().all(|s| s.ends_with("(empty)")), "{:?}", cv.seats);
    assert!(!cv.due, "just founded - the cadence still has to run");

    // and the Meet button runs the campaign meeting when it's due
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    g.seats[0].ctx_actions.push(CtxAct::Meet);
    let i = g.seats[0].ctx_actions.len() - 1;
    g.ctx_act_at(0, i);
    // the log keeps the outcome; the room's talk is on the TALK panel
    let last = g.log.last().unwrap();
    assert!(last.contains("meet about the campaign") && !last.contains(": \""), "{:?}", last);
    assert!(last.len() < 8 * 43, "the entry must fit the log's eight rows: {:?}", last);
    let rec = g.talk_log.last().expect("the meeting on the TALK panel");
    assert_eq!(rec.kind, dialogue::TalkKind::Meeting);
    assert!(rec.with.contains(&g.player) && rec.lines.len() >= 2, "{:?}", rec.lines);
    assert!(g.ui_model().panel.campaign.unwrap().due);
}

#[test]
fn test_the_organizer_sets_how_often_they_meet() {
    let mut g = Game::new();
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    assert_eq!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days, 3, "the founding cadence");
    // the Cadence button steps the recommendation up, then wraps round
    g.seats[0].ctx_actions.push(CtxAct::Cadence);
    let i = g.seats[0].ctx_actions.len() - 1;
    g.ctx_act_at(0, i);
    assert_eq!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days, 7);
    assert_eq!(g.ui_model().panel.campaign.unwrap().cadence_days, 7);
    for _ in 0..3 { g.ctx_act_at(0, i); }
    assert_eq!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days, 3, "the cycle wraps to the fight's rhythm");
}

#[test]
fn test_a_ctx_button_that_cant_answer_says_why() {
    let mut g = Game::new();
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    g.seats[0].ctx_actions.push(CtxAct::Meet);
    // the campaign just met at founding - Meet shows but can't answer yet
    let mi = g.ui_model().ctx.iter().position(|a| *a == CtxAct::Meet).unwrap();
    assert!(g.ui_model().ctx_blocked[mi].is_some(), "not due: the button should say so");
    let log_len = g.log.len();
    g.ctx_act_at(0, mi);
    assert!(g.log[log_len].contains("Can't"), "{:?}", g.log.last());
    // and when it IS due the button answers again
    g.tick += 4;
    assert!(g.ctx_blocker_at(0, CtxAct::Meet).is_none());
}

#[test]
fn test_the_losers_grumble_as_the_cause_is_named() {
    let mut g = Game::new();
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    let msg = g.maybe_found_campaign(0).unwrap();
    // a tax grievance under a shared-grievance banner loses something in
    // the framing - the log hears the dissent, not just the count
    assert!(msg.contains("mutters"), "{}", msg);
}

#[test]
fn test_the_organizer_names_a_seat_and_takes_the_street() {
    let mut g = Game::new();
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();

    // pick a member and hand them the turnout seat
    g.seats[0].selected_target = 0;
    g.seats[0].picked = true;
    g.seats[0].ctx_actions.push(CtxAct::Delegate);
    g.ctx_act_at(0, 0);
    assert!(g.log.last().unwrap().contains("turnout coordinator"), "{:?}", g.log.last());
    assert_eq!(crate::game::campaign::seat_holder(&g.world, c,
        crate::game::campaign::Role::TurnoutCoordinator), Some(g.villagers[0]));

    // a bonded member answers the march - the street reads the ties
    for _ in 0..3 {
        crate::game::relationships::record_interaction(&mut g.world, g.villagers[1],
            crate::game::relationships::Interaction {
                other: g.villagers[0], kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
    }
    g.seats[0].ctx_actions.push(CtxAct::March);
    g.ctx_act_at(0, 1);
    assert!(g.log.last().unwrap().contains("march in the street"), "{:?}", g.log.last());
}
