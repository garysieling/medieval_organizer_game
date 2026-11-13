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

    // and the Meet button opens the agenda when it's due - the meeting
    // runs on the floor's pick (#1157): the last row's always "just meet"
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    g.seats[0].ctx_actions.push(CtxAct::Meet);
    let i = g.seats[0].ctx_actions.len() - 1;
    g.ctx_act_at(0, i);
    let (slate, _) = g.seats[0].pending_agenda.clone().expect("the agenda's left open");
    let convo = g.ui_model().convo.expect("the slate shows on the box");
    assert!(convo.decides && convo.options.len() == slate.len() + 1, "{:?}", convo.options);
    g.decide_at(0, slate.len()); // "just meet": the resolves, no action
    assert!(g.seats[0].pending_agenda.is_none());
    // the log keeps the outcome; the room's talk is on the TALK panel
    let last = g.log.last().unwrap();
    assert!(last.contains("meet about the campaign") && !last.contains(": \""), "{:?}", last);
    assert!(last.len() < 8 * 43, "the entry must fit the log's eight rows: {:?}", last);
    let rec = g.talk_log.last().expect("the meeting on the TALK panel");
    assert_eq!(rec.kind, dialogue::TalkKind::Meeting);
    assert!(rec.with.contains(&g.player) && rec.lines.len() >= 2, "{:?}", rec.lines);
    assert!(g.ui_model().panel.campaign.unwrap().due);
}

/// A cause keeps its own dates: the cadence meeting gathers on its
/// day without the seat calling it - the room's own pull, not the
/// button (#1159)
#[test]
fn test_the_cause_meets_on_its_own_dates() {
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
    // the founding cadence is three days; nobody presses Meet
    for _ in 0..3 { g.tick += 1; }
    g.campaign_meeting_tick(g.tick);
    {
        let m = g.world.get_component::<crate::game::campaign::CampaignMeeting>(c).unwrap();
        assert_eq!(m.held, 1, "the due meeting never sat");
        assert_eq!(m.last_held, g.tick);
    }
    assert!(g.log.iter().any(|l| l.contains("meet about the campaign")), "{:?}", g.log.last());
    // the same date doesn't sit twice - the seat's button and the
    // tick hold the one meeting
    g.campaign_meeting_tick(g.tick);
    assert_eq!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c).unwrap().held, 1);
    // and the next one comes round on schedule again
    for _ in 0..3 { g.tick += 1; }
    g.campaign_meeting_tick(g.tick);
    assert_eq!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c).unwrap().held, 2);
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
    // the organizer answers once a day - the cycle walks a day a step
    for _ in 0..3 { g.tick += 1; g.ctx_act_at(0, i); }
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
    // the march walks its route out before the street answers
    for _ in 0..12000 { g.update(); if g.formation.is_none() { break; } }
    assert!(g.log.iter().rev().take(8).any(|l| l.contains("march in the street")), "{:?}", g.log.last());
}

/// The cause's flyers: the hat comes round the members, the treasury
/// buys pamphlets off the print shop, and they go up on doors and the
/// market cross - folk who read them wake to the issue
#[test]
fn test_the_cause_passes_the_hat_and_posts_flyers() {
    let mut g = Game::new();
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    // a print shop with pamphlets on the shelf, and a printer minding it
    let shop = household::spawn_building(&mut g.world,
        household::BuildingKind::Shop(Vocation::Printmaker), "the print shop", 20.0, 20.0, 20);
    g.world.get_component_mut::<Building>(shop).unwrap().stock.insert("pamphlets".into(), 6);
    g.building_entities.push(shop);
    let printer = g.villagers[3..].iter().copied().find(|&v| {
        g.world.get_component::<Kid>(v).is_none()
            && g.world.get_component::<Craft>(v).is_some()
            && !villager::is_dead(&g.world, v)
    }).unwrap();
    {
        let c = g.world.get_component_mut::<Craft>(printer).unwrap();
        c.vocation = Vocation::Printmaker;
        c.workplace = Some(shop);
    }
    g.world.get_component_mut::<Building>(shop).unwrap().worker = Some(printer);
    let treasury = g.world.get_component::<Organization>(g.org).unwrap().resources;
    let gold: Vec<f32> = g.villagers[..3].iter()
        .map(|&v| g.world.get_component::<Purse>(v).unwrap().gold).collect();
    let printer_gold = g.world.get_component::<Purse>(printer).unwrap().gold;
    g.seats[0].ctx_actions.push(CtxAct::Flyers);
    g.ctx_act_at(0, 0);
    assert!(g.log.last().unwrap().contains("flyers go"), "{:?}", g.log.last());
    let qty = g.flyers.len() as u32;
    assert!(qty > 0 && qty <= 3, "one flyer a member at most: {qty}");
    assert_eq!(g.ui_model().flyers.len(), qty as usize, "the model carries the postings");
    // the hat came round: each member gave what they could spare
    for (i, &v) in g.villagers[..3].iter().enumerate() {
        let now = g.world.get_component::<Purse>(v).unwrap().gold;
        assert!((now - (gold[i] - gold[i].min(2.0))).abs() < 0.01, "member {i} gave");
    }
    // the treasury paid the printer for the pamphlets
    let price = crate::game::goods::worth_at(&g.world, "pamphlets");
    let raised: f32 = gold.iter().map(|x| x.min(2.0)).sum();
    let after = g.world.get_component::<Organization>(g.org).unwrap().resources;
    assert!((after - (treasury + raised - qty as f32 * price)).abs() < 0.01,
        "treasury {treasury} + {raised} - {qty}x{price} = {}", treasury + raised - qty as f32 * price);
    assert!((g.world.get_component::<Purse>(printer).unwrap().gold - printer_gold - qty as f32 * price).abs() < 0.01,
        "the printer was paid");
    // and somebody who read a posting woke to the issue
    let woken = g.villagers.iter().any(|&v| {
        !g.villagers[..3].contains(&v)
            && g.world.get_component::<crate::game::reactions::Sympathized>(v)
                .map(|s| s.issues.iter().any(|(i, _)| i == "the taxes")).unwrap_or(false)
    });
    assert!(woken || !g.log.last().unwrap().contains("wake"), "sympathies where folk read them");
}

#[test]
fn test_the_card_names_the_odds_before_the_ask() {
    let mut g = Game::new();
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let v = g.villagers[7];
    let rel = g.world.create_entity();
    g.world.add_component(rel, crate::game::relationships::Relationship {
        from: g.seats[0].player, other: v,
        kind: crate::game::relationships::RelKind::Friend,
        strength: 0.95, interactions: 5 });
    g.seats[0].selected_target = 7;
    let ins = g.villager_inspector_at(0);
    let stage = ins.lines.iter().find(|l| l.x == 150 && l.y == 330).map(|l| l.text.clone()).unwrap_or_default();
    for w in ["sure to sign on", "a stranger's ask", "likely to join", "a cold ask"] {
        if stage.contains(w) { return; }
    }
    panic!("no odds word rode the stage: {}", stage);
}

#[test]
fn test_the_panel_names_who_pulls_the_room() {
    let mut g = Game::new();
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    // Bram's pull outdraws the leader's: tie him to the other members
    let elite = g.villagers[0];
    for &m in &g.villagers[1..3] {
        let rel = g.world.create_entity();
        g.world.add_component(rel, crate::game::relationships::Relationship {
            from: elite, other: m,
            kind: crate::game::relationships::RelKind::Friend,
            strength: 0.95, interactions: 5 });
    }
    // the pull's only named once the room's observably followed them -
    // a drifted meeting marks its driver
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c).unwrap()
        .driven_by = Some(elite);
    let m = g.ui_model();
    let cv = m.panel.campaign.as_ref().unwrap();
    assert_eq!(cv.shadow.as_deref(), Some(crate::session::vill_name(0)),
        "the pull wasn't named: {:?}", cv.shadow);
}

#[test]
fn test_the_panel_names_the_cause_s_posture() {
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
    // the room banked its consent last meeting - the cause's word shows
    crate::game::campaign_arc::record_consent(&mut g.world, c, true);
    let m = g.ui_model();
    let cv = m.panel.campaign.as_ref().unwrap();
    assert_eq!(cv.state, "the room consents", "posture read: {}", cv.state);
}

#[test]
fn test_the_panel_only_offers_what_can_fire() {
    let mut g = Game::new();
    // before any cause exists the campaign verbs aren't on the card
    g.update_ctx_actions_at(0);
    for a in [CtxAct::Meet, CtxAct::Cadence, CtxAct::March, CtxAct::Flyers,
              CtxAct::Delegate, CtxAct::Strike, CtxAct::Townhall] {
        assert!(!g.seats[0].ctx_actions.contains(&a), "{a:?}: {:?}", g.seats[0].ctx_actions);
    }
    // a founded cause offers its street verbs - but not a Meet that
    // isn't due, nor flyers with no print shop to buy from
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    g.update_ctx_actions_at(0);
    {
        let ctx = &g.seats[0].ctx_actions;
        assert!(ctx.contains(&CtxAct::Cadence), "{ctx:?}");
        assert!(!ctx.contains(&CtxAct::Meet), "just founded - not due yet: {ctx:?}");
        // the street verbs aren't on the card any more - they're motions
        // the meeting floor votes (#1157)
        for a in [CtxAct::March, CtxAct::Flyers, CtxAct::Strike, CtxAct::Townhall] {
            assert!(!ctx.contains(&a), "{a:?} is a motion now: {ctx:?}");
        }
    }
    // due is due: Meet joins the card
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    g.update_ctx_actions_at(0);
    assert!(g.seats[0].ctx_actions.contains(&CtxAct::Meet), "{:?}", g.seats[0].ctx_actions);
    // a picked member is offered a seat while one stands empty
    g.seats[0].selected_target = 0;
    g.seats[0].picked = true;
    g.update_ctx_actions_at(0);
    assert!(g.seats[0].ctx_actions.contains(&CtxAct::Delegate), "{:?}", g.seats[0].ctx_actions);
    // every seat spoken for: Delegate has nothing left to offer
    for (role, holder) in [
        (crate::game::campaign::Role::TurnoutCoordinator, g.villagers[1]),
        (crate::game::campaign::Role::NoteTaker, g.villagers[2]),
        (crate::game::campaign::Role::Chair, g.villagers[0]),
    ] {
        crate::game::campaign::delegate_role(&mut g.world, c, role, holder);
    }
    g.update_ctx_actions_at(0);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::Delegate),
        "a full roster offers no seat: {:?}", g.seats[0].ctx_actions);
    // the demands live on the slate now: four grown members can put them,
    // but a full grant the Reeve's honouring takes them off it - the same
    // door that used to grey the button (#1157)
    let grown: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .take(4).collect();
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = grown;
    g.world.get_component_mut::<crate::game::campaign::Campaign>(c).unwrap()
        .target = g.collector;
    g.seats[0].ctx_actions.push(CtxAct::Meet);
    let mi = g.seats[0].ctx_actions.len() - 1;
    g.ctx_act_at(0, mi);
    let slate = &g.seats[0].pending_agenda.as_ref().unwrap().0;
    assert!(slate.contains(&crate::game::campaign::Motion::Townhall), "{slate:?}");
    g.world.add_component(g.org, crate::game::reactions::LordGrant::full(
        "the taxes", g.tick, g.tick + 100));
    g.ctx_act_at(0, mi);
    let slate = &g.seats[0].pending_agenda.as_ref().unwrap().0;
    assert!(!slate.contains(&crate::game::campaign::Motion::Townhall),
        "the grant stands: {slate:?}");
    // and a march is a motion the floor votes now: the slate's pick walks
    // the column out, and the street still answers once a day
    g.ctx_act_at(0, mi);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|&m| m == crate::game::campaign::Motion::March)
        .expect("a cause this size can march");
    g.decide_at(0, pick);
    assert!(g.formation.is_some(), "the vote marches them out");
    // the march walks its route out before the street answers
    for _ in 0..12000 { g.update(); if g.formation.is_none() { break; } }
    assert!(g.ctx_blocker_at(0, CtxAct::March).is_some(), "same-day march refused");
    g.tick += 1;
    assert!(g.ctx_blocker_at(0, CtxAct::March).is_none(), "tomorrow's march answers");
}

#[test]
fn test_the_slate_grows_with_the_cause_and_sends_an_organizer() {
    let mut g = Game::new();
    use crate::game::campaign::Motion;
    // a cause of eight grown members carries the road work too - and a
    // small one doesn't see the big doors (#1157)
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    let grown: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .take(8).collect();
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = grown.clone();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    g.seats[0].ctx_actions.push(CtxAct::Meet);
    g.ctx_act_at(0, g.seats[0].ctx_actions.len() - 1);
    let (slate, _) = g.seats[0].pending_agenda.clone().expect("the agenda's left open");
    assert!(slate.contains(&Motion::TwoFronts) && slate.contains(&Motion::SendOrganizer),
        "eight hands carry two fronts and the road: {slate:?}");
    assert!(!slate.iter().any(|m| matches!(m, Motion::ParallelEvent(_))),
        "no post warms yet: {slate:?}");
    // shrink the roster and the big motions fall off the slate
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        grown[..3].to_vec();
    g.ctx_act_at(0, g.seats[0].ctx_actions.len() - 1);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    assert!(!slate.contains(&Motion::TwoFronts) && !slate.contains(&Motion::SendOrganizer),
        "three hands can't hold two fronts or the road: {slate:?}");
    // back to eight: the vote sends an organizer out to raise a town
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = grown;
    g.ctx_act_at(0, g.seats[0].ctx_actions.len() - 1);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|&m| m == Motion::SendOrganizer).unwrap();
    g.decide_at(0, pick);
    assert!(g.seats[0].pending_agenda.is_none(), "the pick settles the slate");
    let (site, who) = g.world.get_component::<crate::game::campaign::Outreach>(c)
        .and_then(|o| o.posts.first().map(|(s, w, _)| (*s, *w)))
        .expect("the vote posts someone");
    assert_eq!(g.sites[site].kind, crate::session::country::SiteKind::Town);
    assert_eq!(g.world.get_component::<crate::game::campaign::FieldPost>(who).unwrap().site, site);
    assert!(g.travels.iter().any(|t| t.who == who && matches!(t.goal, Goal::FieldPost(_))),
        "the organizer rides out");
    // the post warms the town a measure a day - three days and the town
    // can stage the cause's day itself
    for _ in 0..20000 {
        g.update();
        let warm = g.world.get_component::<crate::game::campaign::Outreach>(c)
            .and_then(|o| o.posts.first().map(|(_, _, s)| *s)).unwrap_or(0.0);
        if warm >= crate::game::campaign::FIELD_WARM { break; }
    }
    let warm = g.world.get_component::<crate::game::campaign::Outreach>(c)
        .and_then(|o| o.posts.first().map(|(_, _, s)| *s)).unwrap_or(0.0);
    assert!(warm >= crate::game::campaign::FIELD_WARM, "the post warms by the day: {warm}");
    // the warm town goes on the slate: the floor stages its day there
    g.seats[0].ctx_actions.push(CtxAct::Meet);
    g.ctx_act_at(0, g.seats[0].ctx_actions.len() - 1);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|&m| matches!(m, Motion::ParallelEvent(s) if s == site))
        .expect("the warm town can stage: {slate:?}");
    g.decide_at(0, pick);
    // the town's own folk wake to the issue, the post's work done, and
    // the organizer's road home begins
    assert!(g.world.get_component::<crate::game::campaign::FieldPost>(who).is_none());
    assert!(g.world.get_component::<crate::game::campaign::Outreach>(c)
        .map(|o| o.posts.is_empty()).unwrap_or(true));
    let woke = g.villagers.iter().filter(|&&v|
        g.world.get_component::<crate::session::country::Outlander>(v)
            .map(|o| o.site == site).unwrap_or(false)
        && g.world.get_component::<crate::game::reactions::Sympathized>(v)
            .map(|s| s.issues.iter().any(|(i, _)| *i == "the taxes")).unwrap_or(false)).count();
    assert!(woke > 0, "the town's folk wake to the issue");
}

