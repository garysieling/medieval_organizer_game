use crate::session::*;
use super::sit_floor;

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

    // and the cadence's own date opens the agenda when it's due - the
    // meeting runs on the floor's pick (#1337): "just meet" then the
    // adjourn sit past the slate (#1627)
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().expect("the agenda's left open");
    let convo = g.ui_model().convo.expect("the slate shows on the box");
    assert!(convo.decides && convo.options.len() == slate.len() + 2, "{:?}", convo.options);
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
/// day: the cadence's own date sits the floor on the seat's box, and
/// the seat's vote is the meeting (#1337)
#[test]
fn test_the_cause_sits_the_floor_on_its_own_dates() {
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
    // the founding cadence is three days: on its day the cause convenes
    // on the seat (#1615) - the slate opens once the room's resolved
    for _ in 0..3 { g.tick += 1; }
    g.campaign_meeting_tick(g.tick);
    assert!(g.meeting.is_some(), "the cause's folk convene on the seat");
    assert!(g.seats[0].pending_agenda.is_none(), "no floating slate mid-meeting");
    while g.meeting.is_some() { g.advance_meeting(); }
    let (slate, _) = g.seats[0].pending_agenda.clone().expect("the floor sits");
    // the same date doesn't sit twice
    g.campaign_meeting_tick(g.tick);
    assert!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c).unwrap().held == 0,
        "the floor waits on the seat");
    g.decide_at(0, slate.len()); // "just meet": the resolves alone
    {
        let m = g.world.get_component::<crate::game::campaign::CampaignMeeting>(c).unwrap();
        assert_eq!(m.held, 1, "the vote held the meeting");
        assert_eq!(m.last_held, g.tick);
    }
    assert!(g.log.iter().any(|l| l.contains("meet about the campaign")), "{:?}", g.log.last());
    // and the next one comes round on schedule again - convening then
    // sitting the floor the same way
    for _ in 0..3 { g.tick += 1; }
    g.campaign_meeting_tick(g.tick);
    while g.meeting.is_some() { g.advance_meeting(); }
    assert!(g.seats[0].pending_agenda.is_some(), "the next floor sits");
    g.decide_at(0, g.seats[0].pending_agenda.as_ref().unwrap().0.len());
    assert_eq!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c).unwrap().held, 2);
}

/// A floor nobody votes doesn't stall the cause (#1337): the slate
/// waits the week, then the room's own date holds the meeting without
/// the seat - the same "you never came" the tick files
#[test]
fn test_a_floor_left_sitting_meets_without_the_seat() {
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
    sit_floor(&mut g, 0);
    assert!(g.seats[0].pending_agenda.is_some());
    // put the rhythm back: the lapsed floor holds the one meeting, the
    // tick doesn't sit a fresh slate on the same day
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 3;
    // the slate outlives the week: the day's work ages it out and the
    // room meets anyway - "we looked for you and you never came"
    g.tick += FOUNDING_OFFER_EXPIRY_DAYS;
    g.livelihood_tick(g.tick);
    assert!(g.seats[0].pending_agenda.is_none(), "the floor moved on");
    let m = g.world.get_component::<crate::game::campaign::CampaignMeeting>(c).unwrap();
    assert!(m.held >= 1 && m.last_held == g.tick, "the room met without the seat");
    assert!(g.log.iter().any(|l| l.contains("meet about the campaign")), "{:?}", g.log.last());
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
    // the cadence motion steps the recommendation up, then wraps round
    g.cadence_at(c);
    assert_eq!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days, 7);
    assert_eq!(g.ui_model().panel.campaign.unwrap().cadence_days, 7);
    // the room's call counts once a day - the cycle walks a day a step
    for _ in 0..3 { g.tick += 1; g.cadence_at(c); }
    assert_eq!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days, 3, "the cycle wraps to the fight's rhythm");
}

#[test]
fn test_the_floor_sits_only_when_the_cadence_due() {
    let mut g = Game::new();
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    // the cause just met at founding - the day's tick doesn't sit the
    // floor until the cadence's run (#1337)
    g.campaign_meeting_tick(g.tick);
    assert!(g.seats[0].pending_agenda.is_none(), "not due: no floor sits");
    // and when it IS due the cause convenes on the seat (#1615) - the
    // floor opens on the seat's box once the room's resolved
    g.tick += 4;
    g.campaign_meeting_tick(g.tick);
    while g.meeting.is_some() { g.advance_meeting(); }
    assert!(g.seats[0].pending_agenda.is_some(), "due: the floor sits");
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
    g.march_at(0);
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
    let msg = g.flyers_at(0);
    assert!(msg.contains("flyers go"), "{msg}");
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
        strength: 0.95, interactions: 5, ..Default::default() });
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
            strength: 0.95, interactions: 5, ..Default::default() });
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
fn test_the_panel_and_the_slate_only_offer_what_can_fire() {
    let mut g = Game::new();
    // the cause's verbs left the card for good (#1337): before any cause
    // exists there's nothing of the campaign on the panel at all
    g.update_ctx_actions_at(0);
    for a in [CtxAct::Delegate, CtxAct::Strike, CtxAct::Townhall] {
        assert!(!g.seats[0].ctx_actions.contains(&a), "{a:?}: {:?}", g.seats[0].ctx_actions);
    }
    // a founded cause's day sits the floor - the slate's the only street
    // left, sized by what the room can carry (#1157)
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    // just founded: the cadence hasn't run, so the floor doesn't sit
    g.campaign_meeting_tick(g.tick);
    assert!(g.seats[0].pending_agenda.is_none(), "not due: no floor sits");
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    sit_floor(&mut g, 0);
    let slate = g.seats[0].pending_agenda.as_ref().unwrap().0.clone();
    assert!(slate.contains(&crate::game::campaign::Motion::Cadence), "{slate:?}");
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
    // the demands live on the slate: four grown members can put them,
    // but a full grant the Reeve's honouring takes them off it - the same
    // door that used to grey the button (#1157)
    let oe = g.org;
    crate::game::campaign::mark_legwork(&mut g.world, oe, "the taxes".into());
    let grown: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .take(4).collect();
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = grown;
    g.world.get_component_mut::<crate::game::campaign::Campaign>(c).unwrap()
        .target = g.collector;
    assert!(g.agenda_motions(0, c).contains(&crate::game::campaign::Motion::Townhall));
    g.world.add_component(g.org, crate::game::reactions::LordGrant::full(
        "the taxes", g.tick, g.tick + 100));
    assert!(!g.agenda_motions(0, c).contains(&crate::game::campaign::Motion::Townhall),
        "the grant stands");
    // and a march is a motion the floor votes: the slate's pick walks the
    // column out, and the street still answers once a day
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|&m| m == crate::game::campaign::Motion::March)
        .expect("a cause this size can march");
    g.decide_at(0, pick);
    assert!(g.formation.is_some(), "the vote marches them out");
    // the march walks its route out before the street answers - and the
    // same day can't be asked for a second column
    for _ in 0..12000 { g.update(); if g.formation.is_none() { break; } }
    assert!(!g.agenda_motions(0, c).contains(&crate::game::campaign::Motion::March),
        "same-day march refused");
    g.tick += 1;
    assert!(g.agenda_motions(0, c).contains(&crate::game::campaign::Motion::March),
        "tomorrow's march answers");
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
    // the asking's already done on this trouble - the loud work opens
    let go = g.org;
    crate::game::campaign::mark_legwork(&mut g.world, go, "the taxes".into());
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().expect("the agenda's left open");
    assert!(slate.contains(&Motion::TwoFronts) && slate.contains(&Motion::SendOrganizer),
        "eight hands carry two fronts and the road: {slate:?}");
    assert!(!slate.iter().any(|m| matches!(m, Motion::ParallelEvent(_))),
        "no post warms yet: {slate:?}");
    // shrink the roster and the big motions fall off the slate
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        grown[..3].to_vec();
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    assert!(!slate.contains(&Motion::TwoFronts) && !slate.contains(&Motion::SendOrganizer),
        "three hands can't hold two fronts or the road: {slate:?}");
    // back to eight: the vote sends an organizer out to raise a town
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = grown;
    sit_floor(&mut g, 0);
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
    sit_floor(&mut g, 0);
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


/// A bond strong enough for a door to answer (#1270)
fn survey_bond(g: &mut Game, a: Entity, b: Entity) {
    let r = g.world.create_entity();
    g.world.add_component(r, crate::game::relationships::Relationship {
        from: a, other: b, kind: crate::game::relationships::RelKind::Friend,
        strength: 0.5, interactions: 1, ..Default::default() });
}

/// A four-hand cause on "the taxes" with the meeting pulled
fn tax_cause(g: &mut Game, hands: usize) -> Entity {
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 400);
    }
    let grown: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .take(hands).collect();
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = grown;
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    c
}

#[test]
fn test_the_survey_motion_sends_the_doors_asking() {
    let mut g = Game::new();
    use crate::game::campaign::Motion;
    let c = tax_cause(&mut g, 4);
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().expect("the agenda's left open");
    let pick = slate.iter().position(|&m| m == Motion::Survey)
        .expect("four hands carry the asking: {slate:?}");
    g.decide_at(0, pick);
    // the cause's own question's the only one on the floor, so the
    // asking starts at once - no card, hands named, three days out
    let sv = g.org_survey.clone().expect("the asking's under way");
    assert_eq!(sv.campaign, c);
    assert_eq!(sv.issue, "the taxes");
    assert!(!sv.canvassers.is_empty() && sv.canvassers.len() <= 6);
    assert_eq!(sv.days_left, 3);
    assert!(g.survey_pick.is_none() && g.dilemma.is_none());
    // and one round of asking at a time: the motion's off the slate
    // while the count's out
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    assert!(!slate.contains(&Motion::Survey), "the asking's still out: {slate:?}");
}

#[test]
fn test_the_street_waits_until_the_asking_comes_back() {
    let mut g = Game::new();
    use crate::game::campaign::Motion;
    // a fresh cause, no asking done on its trouble yet
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 400);
    }
    let grown: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .take(4).collect();
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = grown.clone();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    // the floor's first slate is the quiet work - the street's loud
    // picks wait until the cause has asked around (#1335)
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    assert!(!slate.contains(&Motion::March) && !slate.contains(&Motion::Strike)
        && !slate.contains(&Motion::Townhall),
        "a fresh cause can't jump to action: {slate:?}");
    let pick = slate.iter().position(|&m| m == Motion::Survey)
        .expect("the asking's always on the floor: {slate:?}");
    g.decide_at(0, pick);
    // three days of knocking and the count lands - the legwork's done
    let t = g.tick;
    for _ in 0..3 { g.survey_tick(t); }
    assert!(g.org_survey.is_none(), "the count's back");
    assert!(crate::game::campaign::legwork_done(&g.world, c));
    // the vote to ask left the room reading patient, and a patient room
    // turns down the street by its own lights (#1310) - clear the reads
    // so it's the legwork gate alone on show here
    for &m in &grown {
        g.world.remove_component::<crate::game::rhetoric::RhetoricMemory>(m);
    }
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    assert!(slate.contains(&Motion::March) && slate.contains(&Motion::Strike)
        && slate.contains(&Motion::Townhall),
        "the asking done, the street opens: {slate:?}");
}

#[test]
fn test_the_floor_picks_the_surveys_question() {
    let mut g = Game::new();
    let c = tax_cause(&mut g, 4);
    // a second question the organizer's heard ledger knows
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the wolves".into());
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|&m| m == crate::game::campaign::Motion::Survey).unwrap();
    g.decide_at(0, pick);
    // two candidate questions on the floor: the card asks which rides
    let (cc, issues) = g.survey_pick.clone().expect("the floor's asked");
    assert_eq!(cc, c);
    assert_eq!(issues.len(), 2);
    let k = g.dilemma.clone().expect("the card's out");
    assert_eq!(k.branches.len(), 2);
    // the second question it is - the asking starts on it
    g.decide_at(0, 1);
    let sv = g.org_survey.clone().expect("the asking's under way");
    assert_eq!(sv.issue, "the wolves");
    assert!(g.survey_pick.is_none() && g.dilemma.is_none());
}

#[test]
fn test_the_count_comes_back_fuzzed_and_the_asking_feeds_the_eyes() {
    let mut g = Game::new();
    let c = tax_cause(&mut g, 4);
    g.world.get_component_mut::<crate::game::campaign::Campaign>(c).unwrap()
        .target = g.collector;
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|&m| m == crate::game::campaign::Motion::Survey).unwrap();
    g.decide_at(0, pick);
    let canvassers = g.org_survey.as_ref().unwrap().canvassers.clone();
    // doors the ties can reach: three souls the canvassers know, two
    // carrying the taxed worry themselves
    let outside: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none()
            && !g.world.get_component::<Organization>(g.org).unwrap().members.contains(&v))
        .take(3).collect();
    for &v in &outside {
        survey_bond(&mut g, canvassers[0], v);
    }
    desires::add_grievance(&mut g.world, outside[0], desires::Problem::Taxed, g.tick + 400);
    desires::add_grievance(&mut g.world, outside[1], desires::Problem::Taxed, g.tick + 400);
    // and the canvasser carries the worry too: a door that doesn't
    // share it still answers "a friend has it" (#1270)
    desires::add_grievance(&mut g.world, canvassers[0], desires::Problem::Taxed, g.tick + 400);
    let reach = crate::game::campaign::survey_respondents(
        &g.world, g.org, &canvassers, &g.villagers);
    assert!(reach.len() >= 3, "the bonds carry the asking's reach: {reach:?}");
    assert_eq!(crate::game::campaign::survey_ring(&g.world, outside[0], &"the taxes".into(), g.tick), 3,
        "their own house has it");
    assert_eq!(crate::game::campaign::survey_ring(&g.world, outside[2], &"the taxes".into(), g.tick), 1,
        "a friend of theirs has it");
    let pen_before = g.penetration;
    // three days' asking, then the count lands
    for _ in 0..30000 {
        g.update();
        if g.org_survey.is_none() { break; }
    }
    assert!(g.org_survey.is_none(), "the count came back");
    assert!(g.log.iter().any(|l| l.contains("the count's back") && l.contains("answered")),
        "the tally lands in the log: {:?}", g.log.iter().last());
    // fuzzed, never named: no villager's name rides the count
    let tally = g.log.iter().find(|l| l.contains("the count's back")).unwrap();
    for &v in &outside {
        if let Some(i) = g.villagers.iter().position(|&x| x == v) {
            assert!(!tally.contains(vill_name(i)), "the count keeps no names: {tally}");
        }
    }
    // and the asking fed the eyes inside - canvassing is what
    // agitators do
    assert!(g.penetration > pen_before, "the asking fed the eyes: {}", g.penetration);
    // the doors themselves got walked: canvassers took the road out
    assert!(g.log.iter().any(|l| l.contains("three days' asking")),
        "the going-out was noted");
}

/// The motion the floor's handed is the line the room hears (#1310):
/// the march witnesses the firebrand on every member standing there
#[test]
fn test_the_picked_motion_witnesses_its_line_on_the_room() {
    use crate::game::campaign::Motion;
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
    use crate::game::rhetoric::{self, Line};
    let mut g = Game::new();
    let grown: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .take(4).collect();
    for &v in &grown {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = grown.clone();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    // the asking's already done on this trouble - the loud work opens
    let go = g.org;
    crate::game::campaign::mark_legwork(&mut g.world, go, "the taxes".into());
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    // ties to the driver pull them into the room; the fourth isn't a
    // member at all - the cause's convening doesn't call them (#1615)
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = grown[..3].to_vec();
    let pe = g.player;
    for &m in &grown[..3] {
        for _ in 0..2 {
            record_interaction(&mut g.world, m,
                Interaction { other: pe, kind: InteractionKind::Aid, intensity: 1.0 });
        }
    }
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().expect("the agenda's left open");
    let pick = slate.iter().position(|&m| m == Motion::March).expect("four hands march");
    g.decide_at(0, pick);
    // the sworn lean in on the line they heard - the stay-home heard nothing
    for &m in &grown[..3] {
        let w = rhetoric::hardened(&g.world, m, g.org);
        let mem = g.world.get_component::<rhetoric::RhetoricMemory>(m).unwrap();
        let got = mem.by_org[&g.org].weight[Line::Firebrand as usize];
        assert!(got >= 1.0 || w.is_empty(), "{m:?} heard the march: {got}");
        assert!(got >= 1.0, "attendee {m:?} read the firebrand line");
    }
    assert_eq!(
        g.world.get_component::<rhetoric::RhetoricMemory>(grown[3])
            .map(|mm| mm.by_org[&g.org].weight[Line::Firebrand as usize]).unwrap_or(0.0),
        0.0, "the stay-home heard nothing");
}

/// A member set fully against the line walks when it's taken again
/// under their nose (#1310): the march costs the patient one, seat
/// and all
#[test]
fn test_a_member_set_against_the_motion_storms_out() {
    use crate::game::campaign::{delegate_role, Motion, Role};
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
    use crate::game::rhetoric::{OrgRead, RhetoricMemory};
    let mut g = Game::new();
    let grown: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .take(4).collect();
    for &v in &grown {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = grown.clone();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    // the asking's already done on this trouble - the loud work opens
    let go = g.org;
    crate::game::campaign::mark_legwork(&mut g.world, go, "the taxes".into());
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    let pe = g.player;
    for &m in &grown {
        for _ in 0..2 {
            record_interaction(&mut g.world, m,
                Interaction { other: pe, kind: InteractionKind::Aid, intensity: 1.0 });
        }
    }
    // the note taker leans patient to the edge: one more street pick
    // sets them fully against it
    let waver = grown[0];
    delegate_role(&mut g.world, c, Role::NoteTaker, waver);
    let mut mem = RhetoricMemory::default();
    mem.by_org.insert(g.org, OrgRead { weight: [2.5, 0.0, 0.0, 0.0], ..Default::default() });
    g.world.add_component(waver, mem);
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|&m| m == Motion::March).unwrap();
    g.decide_at(0, pick);
    assert!(!g.world.get_component::<Organization>(g.org).unwrap().members.contains(&waver),
        "the street pick walked the patient one out");
    assert!(g.log.iter().any(|l| l.contains("storms out")), "the walk-out's in the log");
    assert!(g.world.get_component::<crate::game::campaign::Campaign>(c).unwrap()
        .seats.iter().all(|s| s.holder != waver), "their seat passed with them");
    assert_eq!(crate::game::campaign::seat_holder(&g.world, c, Role::NoteTaker), None);
}

/// The room's settled word is heard as the cause's line (#1310): the
/// debrief's tally witnesses its direction on every member standing there
#[test]
fn test_the_debriefs_settled_word_is_heard_as_the_line() {
    use crate::game::rhetoric::{self, Line};
    let mut g = Game::new();
    let grown: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .take(4).collect();
    for &v in &grown {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = grown.clone();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .map(|m| m.held = 2);
    // stand them all in earshot so the whole room witnesses
    let pp = g.world.get_component::<Pos>(g.player).copied().unwrap();
    for &m in &grown {
        if let Some(mp) = g.world.get_component_mut::<Pos>(m) { mp.x = pp.x + 1.0; mp.y = pp.y; }
    }
    // the floor's debrief motion tallies the room standing there (#1337)
    g.debrief_at(0, &grown);
    let d = g.world.get_component::<crate::game::campaign::Campaign>(c)
        .and_then(|cc| cc.directive.clone()).expect("the room settled nothing");
    let line = match d.as_str() {
        "the petition" => Line::Patient,
        "the march" | "the strike" | "a town hall" => Line::Firebrand,
        other => panic!("an unlined directive: {other}"),
    };
    for &m in &grown {
        let got = g.world.get_component::<rhetoric::RhetoricMemory>(m)
            .map(|mm| mm.by_org[&g.org].weight[line as usize]).unwrap_or(0.0);
        assert!(got >= 1.0, "{m:?} never heard the settled word: {got}");
    }
}

#[test]
fn test_the_founding_call_enrolls_everyone_who_came() {
    let mut g = Game::new();
    // three sworn members hold the issue; a fourth came to listen but
    // never swore - the vote that founds binds the room as a body
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    let came = g.villagers[..4].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    g.resolve_meeting(&came, 0, id, (false, false, false, false, false), &[], false);
    assert!(g.seats[0].pending_founding.is_some(), "the founding call's on the table");
    let listener = g.villagers[3];
    let was = g.world.get_component::<Organization>(g.org).unwrap()
        .members.contains(&listener);
    g.decide_at(0, 0); // a shared grievance carries
    assert!(crate::game::campaign::campaign_of(&g.world, g.org).is_some());
    let members = &g.world.get_component::<Organization>(g.org).unwrap().members;
    assert!(members.contains(&listener),
        "the room joins as a body (was sworn by the meeting: {was}): {members:?}");
    // and the roll-call's grammar agrees: the one new name joins,
    // never "join" (#1626)
    assert!(g.log.iter().any(|l| l.contains("joins the rolls")),
        "the enrollment never logged: {:?}", g.log.last());
    assert!(!g.log.iter().any(|l| l.contains(" join the rolls")),
        "a lone name took the plural verb");
    // and the dues book opens with the cause - the flat rate the vote set
    let dues = g.world.get_component::<crate::game::campaign::Dues>(g.org)
        .expect("the founding opened the roll");
    assert_eq!(dues.rate, crate::game::campaign::DUES_RATE);
}

#[test]
fn test_the_floor_carries_the_pooled_buy() {
    let mut g = Game::new();
    use crate::game::campaign::Motion;
    let c = tax_cause(&mut g, 4);
    // the roll wants bread and carries coin
    let members = g.world.get_component::<Organization>(g.org).unwrap().members.clone();
    for &m in &members {
        g.world.get_component_mut::<desires::InnerLife>(m).unwrap()
            .wants = vec![desires::Want::GoodFood];
        g.world.get_component_mut::<crate::game::livelihood::Purse>(m)
            .map(|p| p.gold = 20.0);
    }
    // a shop keeps it on the shelf
    let b = crate::game::household::spawn_building(&mut g.world,
        crate::game::household::BuildingKind::Shop(crate::game::vocation::Vocation::Baker),
        "the bakery", 0.0, 0.0, 20);
    {
        let bb = g.world.get_component_mut::<crate::game::household::Building>(b).unwrap();
        bb.worker = Some(g.villagers[10]);
        bb.stock.insert("bread".into(), 20);
    }
    // the floor reads whose coin's behind it
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|m| matches!(m, Motion::GroupBuy(0, _)))
        .expect("the pledges put the pooled buy on the floor: {slate:?}");
    g.decide_at(0, pick);
    // shares came home, the rest went out the door below street
    assert!(members.iter().all(|&m| g.world.get_component::<crate::game::livelihood::Satchel>(m)
        .map(|s| s.goods.get("bread").copied().unwrap_or(0) >= 1).unwrap_or(false)),
        "every pledging member takes a share home");
    assert!(g.world.get_component::<Organization>(g.org).unwrap().resources > 0.0,
        "the surplus' coin is the cause's");
    let _ = c;
}

#[test]
fn test_the_party_walks_out_and_the_take_fills_the_chest() {
    let mut g = Game::new();
    use crate::game::campaign::{Motion, PartyKind};
    let _c = tax_cause(&mut g, 4);
    // a good patch flushed and waiting on the pickers
    g.mush_patches[0].kind = crate::game::mycology::MushKind::FieldMushroom;
    for p in g.mush_patches.iter_mut() { p.next_ready = 0; }
    // the first member has a friend outside the cause - the party's
    // soft ask brings them along
    let members = g.world.get_component::<Organization>(g.org).unwrap().members.clone();
    let guest = g.villagers.iter().copied()
        .find(|&v| !members.contains(&v)
            && g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .unwrap();
    survey_bond(&mut g, members[0], guest);
    let warm0 = crate::game::relationships::find_relationship(&g.world, guest, members[0])
        .and_then(|r| g.world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    let res0 = g.world.get_component::<Organization>(g.org).unwrap().resources;

    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|m| matches!(m, Motion::WorkParty(PartyKind::Forage)))
        .expect("four hands carry a picking party: {slate:?}");
    g.decide_at(0, pick);

    // the party's out: members and the guest walking, the organizer home
    assert_eq!(g.work_parties.len(), 1, "the party's under way");
    let pi = g.work_parties[0].patch.expect("a patch party");
    assert!(g.work_parties[0].hands.contains(&guest), "the guest came along");
    assert!(!g.work_parties[0].hands.contains(&g.player), "the organizer's hands stay clean");
    for _ in 0..12000 { g.update(); if g.work_parties.is_empty() { break; } }
    assert!(g.work_parties.is_empty(), "the party resolves once the hands are in");

    // the take pooled to the chest, the patch flushed, the shared day warmed
    let res1 = g.world.get_component::<Organization>(g.org).unwrap().resources;
    assert!(res1 > res0, "the caps sell into the chest");
    assert!(g.mush_patches[pi].next_ready > g.tick, "the patch is picked");
    let warm1 = crate::game::relationships::find_relationship(&g.world, guest, members[0])
        .and_then(|r| g.world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    assert!(warm1 > warm0, "the guest walks home warmer to their host: {warm0} -> {warm1}");
    let m_warm = crate::game::relationships::find_relationship(&g.world, members[0], members[1])
        .and_then(|r| g.world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    assert!(m_warm > 0.0, "the hands warmed to each other over the work");
}

#[test]
fn test_the_raising_turns_the_village_out() {
    let mut g = Game::new();
    use crate::game::campaign::{Motion, PartyKind};
    let _c = tax_cause(&mut g, 5);
    let members = g.world.get_component::<Organization>(g.org).unwrap().members.clone();
    // the folk nearest the plot answer the open call: one bystander bonded
    // so the host-warmth lands on a tie we can read
    let guest = g.villagers.iter().copied()
        .find(|&v| !members.contains(&v)
            && g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .unwrap();
    survey_bond(&mut g, members[0], guest);
    // the open call reads the site: put the friend in reach of wherever
    // the roof goes - beside the player, where the plot scan centers
    let pp = g.world.get_component::<Pos>(g.player).copied().unwrap();
    g.world.get_component_mut::<Pos>(guest).unwrap().x = pp.x + 3.0;
    g.world.get_component_mut::<Pos>(guest).unwrap().y = pp.y;
    let n0 = g.buildings.len();

    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|m| matches!(m, Motion::WorkParty(PartyKind::Raise)))
        .expect("five hands carry a raising: {slate:?}");
    g.decide_at(0, pick);
    assert_eq!(g.work_parties.len(), 1);
    let tile = g.work_parties[0].tile;
    let _ = tile;
    for _ in 0..12000 { g.update(); if g.work_parties.is_empty() { break; } }
    assert!(g.work_parties.is_empty(), "the party resolves");
    assert!(g.buildings.len() > n0, "the roof went up");
}

/// The called meeting IS the cadence (#1337): a cause come due on the
/// day a talk-invited gathering resolves takes its floor on the
/// resolve - and the day's own tick stands down for it
#[test]
fn test_the_gathered_meeting_holds_the_cadence() {
    use super::fixtures::plan_ready_meeting;
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
    // the cause is due today and the word's out: the gathering lands
    // today, so the tick stands down instead of sitting a shadow
    // meeting beside it
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    let host = g.villagers[4];
    let coming: Vec<Entity> = g.villagers[..3].to_vec();
    plan_ready_meeting(&mut g, host, (16.0, 16.0), &coming);
    let (oe, today) = (g.org, g.tick);
    crate::game::organizing::plan_for(&mut g.world, oe).for_day = Some(today);
    // the day's own tick fires the call before it runs the campaign's
    // (#1547 review) - a real gathering is already under way by the
    // time campaign_meeting_tick sees it, not just a dated plan that's
    // already been consumed
    g.livelihood_tick(today);
    assert!(g.meeting.is_some(), "the day fires the call: {:?}", g.log.last());
    assert_eq!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c).unwrap().held, 0,
        "the tick held a meeting beside the gathering");
    // the gathering resolves: the room's own call shows first, the
    // cause's floor sits behind it. The resolve itself is modeled
    // through the same record/resolve pair the rest of this file uses
    // rather than walking frames to the live gathering's own finish
    g.meeting = None;
    let came = g.villagers[..3].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    let msg = g.resolve_meeting(&came, 0, id, (false, false, false, false, false), &[], false);
    assert!(msg.contains("the floor's yours"), "{}", msg);
    assert!(g.seats[0].pending_agenda.is_some(), "the slate never sat");
    assert!(g.seats[0].pending_founding.is_some(), "the room's call should queue ahead");
    // answer the room's call first (the last row rests it) - the floor
    // slate then shows on the box
    let call = g.convo_panel_at(0).expect("the call on the box");
    g.decide_at(0, call.options.len() - 1);
    let convo = g.convo_panel_at(0).expect("the floor's slate shows next");
    assert!(convo.options.iter().any(|o| o.contains("cadence")), "{:?}", convo.options);
    assert!(convo.options.iter().any(|o| o.contains("Just meet")), "{:?}", convo.options);
}

/// A rhythm verb on the floor (#1337): the slate carries the cadence
/// motion and the carried pick retimes the meetings
#[test]
fn test_the_floor_retimes_the_meetings() {
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
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|m| matches!(m, crate::game::campaign::Motion::Cadence))
        .expect("the rhythm always sits on the slate: {slate:?}");
    g.decide_at(0, pick);
    let m = g.world.get_component::<crate::game::campaign::CampaignMeeting>(c).unwrap();
    assert_eq!(m.cadence_days, 7, "the carried pick steps the cadence a notch");
    assert_eq!(m.held, 1, "the vote's own meeting counts");
}

/// The hall-mending rides the slate (#1337): a storm-hit hall puts the
/// spend on the floor, and the carried pick pays the carpenter's bill
#[test]
fn test_the_floor_mends_the_hall() {
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
    g.world.add_component(g.org, crate::game::organizing::HallDamaged);
    g.world.get_component_mut::<Organization>(g.org).unwrap().resources =
        crate::game::organizing::HALL_REPAIR_COST + 5.0;
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|m| matches!(m, crate::game::campaign::Motion::MendHall))
        .expect("a roofless hall puts the mending on the slate: {slate:?}");
    let purse_before = g.world.get_component::<Organization>(g.org).unwrap().resources;
    g.decide_at(0, pick);
    assert!(g.world.get_component::<crate::game::organizing::HallDamaged>(g.org).is_none(),
        "the carried spend never landed");
    let purse_after = g.world.get_component::<Organization>(g.org).unwrap().resources;
    // the hat can pass at the same sitting - the bill's still bigger
    // than the dues, so the chest ends lighter than it started
    assert!(purse_after < purse_before, "the purse never paid the bill: {purse_before} -> {purse_after}");
}

/// A hall the button already mended doesn't bill the floor's pick too
/// (#1337 review): a slate drawn before the repair can still offer
/// MendHall - carrying it once the roof's already fixed costs nothing
#[test]
fn test_a_mended_hall_is_not_billed_twice() {
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
    g.world.add_component(g.org, crate::game::organizing::HallDamaged);
    g.world.get_component_mut::<Organization>(g.org).unwrap().resources =
        crate::game::organizing::HALL_REPAIR_COST + 5.0;
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|m| matches!(m, crate::game::campaign::Motion::MendHall))
        .expect("a roofless hall puts the mending on the slate: {slate:?}");
    // the other path beat the slate to it: the roof's already fixed
    g.world.remove_component::<crate::game::organizing::HallDamaged>(g.org);
    let purse_before = g.world.get_component::<Organization>(g.org).unwrap().resources;
    g.decide_at(0, pick);
    let purse_after = g.world.get_component::<Organization>(g.org).unwrap().resources;
    // the sitting's own dues can still nudge the purse up a little (as
    // the sibling test notes) - the repair's bill is what must not
    // land twice, and that bill dwarfs any dues this sitting collects
    assert!(purse_after >= purse_before,
        "a stale pick on an already-mended hall still billed: {purse_before} -> {purse_after}");
}

/// The room's tally rides the slate once the cause has met (#1337):
/// the debrief motion carries the attendees' next step
#[test]
fn test_the_floor_tallies_the_next_step() {
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
    // a cause that's never met gets no tally on the slate
    let slate0 = g.agenda_motions(0, c);
    assert!(!slate0.iter().any(|m| matches!(m, crate::game::campaign::Motion::Debrief)),
        "no tally before a meeting's held: {slate0:?}");
    // hold one, then the slate offers it
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().held = 1;
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|m| matches!(m, crate::game::campaign::Motion::Debrief))
        .expect("a met cause can tally: {slate:?}");
    g.decide_at(0, pick);
    let directive = g.world.get_component::<crate::game::campaign::Campaign>(c)
        .and_then(|cc| cc.directive.clone());
    assert!(directive.is_some(), "the tally never settled a step");
}

/// A motion can fail on the floor (#1337): the street pick that costs
/// the cause its room - the ones against the line outnumber the ones
/// who stay - falls unseconded
#[test]
fn test_a_motion_the_room_walks_out_on_falls() {
    use crate::game::campaign::Motion;
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
    use crate::game::rhetoric::{OrgRead, RhetoricMemory};
    let mut g = Game::new();
    let grown: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none())
        .take(3).collect();
    for &v in &grown {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = grown.clone();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let go = g.org;
    crate::game::campaign::mark_legwork(&mut g.world, go, "the taxes".into());
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    let pe = g.player;
    // the convening warms the whole room in (#1615): two members walk
    // in already hardened against the street - read enough to storm
    // over the pick, not so deep the room's lean narrows the slate
    // first (#1310's floor gate reads the weight, the walk-out the mark)
    for &m in &grown[..2] {
        for _ in 0..2 {
            record_interaction(&mut g.world, m,
                Interaction { other: pe, kind: InteractionKind::Aid, intensity: 1.0 });
        }
        let mut mem = RhetoricMemory::default();
        mem.by_org.insert(g.org, OrgRead {
            weight: [1.4, 0.0, 0.0, 0.0], hardened: [Some(0), None, None, None],
            ..Default::default() });
        g.world.add_component(m, mem);
    }
    sit_floor(&mut g, 0);
    let (slate, _) = g.seats[0].pending_agenda.clone().unwrap();
    let pick = slate.iter().position(|&m| m == Motion::March).unwrap();
    g.decide_at(0, pick);
    assert!(g.formation.is_none(), "the column formed off an emptied floor");
    assert!(g.log.iter().any(|l| l.contains("storms out")), "{:?}", g.log.last());
}

/// The due date doesn't float a slate wherever the seat stands (#1615):
/// the cause convenes on them - its folk walk to where the organizer
/// is, talk it over, and the slate opens when the room resolves
#[test]
fn test_the_due_floor_convenes_the_cause_on_the_seat() {
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
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    let (px, py) = g.player_pos_at(0);
    g.campaign_meeting_tick(g.tick);
    // no floating slate: the floor's a gathering on the seat's own ground
    assert!(g.seats[0].pending_agenda.is_none(), "no slate out of a meeting");
    let m = g.meeting.as_ref().expect("the cause convenes on the seat");
    assert_eq!(m.seat, 0);
    assert!(m.invited.iter().all(|v| g.villagers[..3].contains(v)),
        "the cause's own folk are called: {:?}", m.invited);
    let near = (m.venue.0 as f32 - px).abs() + (m.venue.1 as f32 - py).abs();
    assert!(near <= 1.5, "the room gathers where the organizer stands: {:?}", m.venue);
    assert!(m.paths.iter().all(|p| p.is_some()), "every called soul has a way");
    // the room talks and resolves - and the slate opens there, in context
    while g.meeting.is_some() { g.advance_meeting(); }
    assert!(g.seats[0].pending_agenda.is_some(), "the floor sits once the room's done");
    let rec = g.talk_log.last().expect("the convening on the TALK panel");
    assert_eq!(rec.kind, dialogue::TalkKind::Meeting);
}

/// A room that can't gather sits the slate the old way (#1615): too
/// few of the cause's folk able to come and the day falls back to the
/// bare slate rather than calling an empty meeting
#[test]
fn test_a_cause_with_no_room_sits_the_slate_directly() {
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
    // the cause's folk are gone but the seat's: no room to call
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = vec![];
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .unwrap().cadence_days = 0;
    g.campaign_meeting_tick(g.tick);
    assert!(g.meeting.is_none(), "nobody to convene: no meeting stands");
    assert!(g.seats[0].pending_agenda.is_some(), "the slate sits the old way");
}

/// A thin room offers the turnout round (#1713): under five souls at
/// the door and the meet-again option becomes each naming a count -
/// and the organizer's held to one of their own
#[test]
fn test_a_thin_room_offers_the_turnout_round() {
    let mut g = Game::new();
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    // two souls at the door is a thin room
    let came = g.villagers[..2].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    g.resolve_meeting(&came, 0, id, (false, false, false, false, false), &[], false);
    let d = g.seats[0].pending_founding.as_ref().expect("the call's on the table");
    assert!(d.vote.is_none(), "a founded cause leaves no vote");
    assert!(d.turnout, "a thin room offers the turnout round");
    let p = g.convo_panel_at(0).expect("the call renders");
    assert!(p.options[0].contains("turnout"),
        "the first option works the turnout: {:?}", p.options);
    // a fuller room keeps the bare meet-again
    g.seats[0].pending_founding = None;
    let came = g.villagers[..6].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    g.resolve_meeting(&came, 0, id, (false, false, false, false, false), &[], false);
    let d = g.seats[0].pending_founding.as_ref().expect("the call's on the table");
    assert!(!d.turnout, "six at the door is no thin room");
    let p = g.convo_panel_at(0).expect("the call renders");
    assert_eq!(p.options[0], "Meet again - they'll come back");
}

/// The pledged count is their own word (#1713): each attendee names a
/// count bounded by the reach they have, the organizer names theirs
/// on the next card, and the plan holds them all until the door -
/// where each word pulls the friends it can actually reach
#[test]
fn test_the_turnout_round_puts_the_words_on_the_plan() {
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
    let mut g = Game::new();
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    // a thin room of two; the organizer knows the third by name
    let (pe, v5) = (g.player, g.villagers[5]);
    record_interaction(&mut g.world, pe,
        Interaction { other: v5, kind: InteractionKind::Aid, intensity: 1.0 });
    let came = g.villagers[..2].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    g.resolve_meeting(&came, 0, id, (false, false, false, false, false), &[], false);
    // the first card option is the turnout round
    g.decide_at(0, 0);
    let plan = g.world.get_component::<crate::game::organizing::MeetingPlan>(g.org)
        .expect("the plan's made");
    assert_eq!(plan.pledges.len(), 2, "each who came gave a count: {:?}", plan.pledges);
    for &(e, n) in &plan.pledges {
        assert!(came.contains(&e), "a pledge belongs to the room");
        assert!((1..=3).contains(&n), "their word's a workable count: {}", n);
    }
    assert!(g.seats[0].pledge_offer.is_some(), "the organizer's count is asked next");
    // the organizer's own count: "I'll bring two"
    let card = g.convo_panel_at(0).expect("the count card renders");
    assert_eq!(card.title, "THE CALL - YOUR OWN COUNT");
    g.decide_at(0, 1);
    let plan = g.world.get_component::<crate::game::organizing::MeetingPlan>(g.org).unwrap();
    assert_eq!(plan.pledges.len(), 3, "the organizer's word rides with theirs");
    assert_eq!(plan.pledges[2], (g.player, 2));
    // the day, then the question - the call's made
    g.decide_at(0, 0);
    g.decide_at(0, 0);
    // at the door each word pulls the friends it can reach
    let msg = g.call_meeting_at(0);
    assert!(msg.contains("gather"), "{}", msg);
    let m = g.meeting.as_ref().expect("the room convenes");
    assert_eq!(m.pledged.len(), 3, "every word's snapshotted for the report");
    for &(p, n, ref pulled) in &m.pledged {
        assert!(pulled.len() <= n as usize, "a pull can't outrun the word");
        let friends = crate::game::organizing::friends_of(&g.world, p, usize::MAX);
        assert!(pulled.iter().all(|f| friends.contains(f)),
            "a word only pulls the friends they have");
    }
    // the room talks and resolves - and the log reads the tally
    while g.meeting.is_some() { g.advance_meeting(); }
    let tally = g.log.iter().find(|l| l.contains(" promised ")).cloned()
        .expect("the turnout report lands in the log");
    assert!(tally.contains(" brought "), "promised reads against brought: {}", tally);
    assert!(tally.contains("you promised 2"), "the organizer's word reads the same: {}", tally);
}

/// The report reads promised against brought (#1713): a kept count
/// hears PromiseKept, a shortfall confirms "promises are for those
/// who can keep them", and the organizer's own word reads the same
#[test]
fn test_the_turnout_report_reads_promised_against_brought() {
    use crate::game::persona::{Belief, EventId, EventKind, Fear, Ghost, Need, Persona};
    // the seeded cast deals each villager a persona the ghost can ride
    let mut g = Game::new_seeded_cast(77);
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let (v0, v1, v3, v4, v5, v6) =
        (g.villagers[0], g.villagers[1], g.villagers[3], g.villagers[4], g.villagers[5], g.villagers[6]);
    // both carry the ghost a pledged word tests
    let ghost = |lie| Ghost { event: EventId(0), kind: EventKind::Levy, hook: None,
        lie, strength: 0.6, fear: Fear::BailiffKnowsMyFace, need: Need::ToBeSafe, turned: false };
    g.world.get_component_mut::<Persona>(v0).unwrap().ghosts.push(ghost(Belief::PromisesForKeepers));
    g.world.get_component_mut::<Persona>(v1).unwrap().ghosts.push(ghost(Belief::PromisesForKeepers));
    // the room: v0's two came, v1's one stayed home, the organizer's
    // one of their two
    let came = vec![v0, v1, v3, v4, v5];
    let pledged = vec![
        (v0, 2u8, vec![v3, v4]),
        (v1, 1u8, vec![v6]),
        (g.player, 2u8, vec![v5]),
    ];
    let id = g.open_meeting_record(g.player, &came);
    let msg = g.resolve_meeting(&came, 0, id, (false, false, false, false, false), &pledged, false);
    let n0 = g.name_of(v0);
    let n1 = g.name_of(v1);
    assert!(msg.contains(&format!("{} promised 2, brought 2", n0)), "{}", msg);
    assert!(msg.contains(&format!("{} promised 1, brought 0", n1)), "{}", msg);
    assert!(msg.contains("you promised 2, brought 1"), "{}", msg);
    // the kept word hears PromiseKept, the blown one confirms the ghost
    let s0 = g.world.get_component::<Persona>(v0).unwrap().ghosts.iter()
        .find(|gh| gh.lie == Belief::PromisesForKeepers).unwrap().strength;
    let s1 = g.world.get_component::<Persona>(v1).unwrap().ghosts.iter()
        .find(|gh| gh.lie == Belief::PromisesForKeepers).unwrap().strength;
    assert!(s0 < 0.6, "a kept count weakens the ghost: {}", s0);
    assert!(s1 > 0.6, "a blown count confirms it: {}", s1);
    // and the room hears its own tally on the record
    let rec = g.talk_log.iter().find(|r| r.id == id).unwrap();
    assert!(rec.lines.iter().any(|l| l.text.contains(" promised ") && l.text.contains(" brought ")),
        "the tally rides the meeting's record");
}
