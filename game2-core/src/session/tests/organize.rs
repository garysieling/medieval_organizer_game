use crate::session::*;
use crate::game::relationships::RelKind;
use super::fixtures::plan_ready_meeting;

#[test]
fn test_organize_holds_a_house_meeting_with_nearby_villagers() {
    let mut game = Game::new();
    let (eliza, bram, corvin) = (game.villagers[1], game.villagers[2], game.villagers[3]);
    // the player stands among Eliza and Bram; Corvin is far away
    game.world.get_component_mut::<Pos>(game.player).unwrap().x = 16.0;
    game.world.get_component_mut::<Pos>(game.player).unwrap().y = 16.0;
    for (v, x, y) in [(eliza, 17.0, 16.0), (bram, 15.0, 17.0), (corvin, 2.0, 2.0)] {
        let p = game.world.get_component_mut::<Pos>(v).unwrap();
        p.x = x; p.y = y;
    }
    // everyone else is off across the fields, well out of reach
    for (i, v) in game.villagers.clone().into_iter().enumerate() {
        if i != 1 && i != 2 { *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 3.0 + i as f32 * 0.5, y: 3.0 }; }
    }
    // Eliza already trusts the player; Bram is a stranger who's merely
    // said he'll come - the meeting is earned: Eliza lends the house
    let rel = game.world.create_entity();
    game.world.add_component(rel, Relationship { from: game.player, other: eliza, kind: RelKind::Friend, strength: 0.7, interactions: 6 });
    plan_ready_meeting(&mut game, eliza, (17.0, 16.0), &[bram]);

    assert!(game.meeting.is_none());
    game.call_meeting();
    finish_meeting(&mut game);

    let members = game.world.get_component::<Organization>(game.org).unwrap().members.clone();
    assert_eq!(members, vec![eliza], "the trusted neighbour joins; the stranger and the absent don't");
    let last = game.log.last().unwrap();
    assert!(last.contains("House meeting - 2 came") && last.contains("Eliza joined"), "{}", last);
    assert!(relationships::find_relationship(&game.world, game.player, bram).is_some(), "Bram got to know the player");
    // only the two on the plan attend - Corvin, with no promise and no
    // place on the plan, was never invited either
    assert!(!members.contains(&corvin), "Corvin wasn't there");
}

#[test]
fn test_a_meeting_takes_time_people_gather_then_talk_then_it_ends() {
    let mut game = Game::new();
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: 16.0, y: 16.0 };
    let (eliza, bram) = (game.villagers[1], game.villagers[2]);
    for (v, x, y) in [(eliza, 18.0, 16.0), (bram, 15.0, 19.0)] { *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x, y }; }
    for (i, v) in game.villagers.clone().into_iter().enumerate() {
        if i != 1 && i != 2 { *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 3.0 + i as f32, y: 3.0 }; }
    }
    let rel = game.world.create_entity();
    game.world.add_component(rel, Relationship { from: game.player, other: eliza, kind: RelKind::Friend, strength: 0.7, interactions: 6 });
    plan_ready_meeting(&mut game, eliza, (18.0, 16.0), &[bram]);
    game.call_meeting();
    assert!(game.log.last().unwrap().contains("gather at"), "{}", game.log.last().unwrap());
    assert!(game.world.get_component::<Organization>(game.org).unwrap().members.is_empty(), "nothing's decided yet");
    assert_eq!(game.call_meeting(), "A meeting is already under way.");
    // they walk over...
    let spots = game.meeting.as_ref().unwrap().spots.clone();
    let far = |game: &Game| [eliza, bram].iter().zip(&spots).map(|(v, s)| {
        let p = game.world.get_component::<Pos>(*v).unwrap(); (p.x - s.0 as f32).abs() + (p.y - s.1 as f32).abs()
    }).sum::<f32>();
    let start = far(&game);
    for _ in 0..FPS { game.update(); }
    assert!(far(&game) < start, "people walk to the meeting");
    // ...then talk it over in thought bubbles, and only then does it end
    let mut thoughts = vec![];
    let mut frames = 0;
    while game.meeting.is_some() {
        game.update();
        frames += 1;
        for b in &game.bubbles { if b.3 && !thoughts.contains(&(b.0, b.1.clone())) { thoughts.push((b.0, b.1.clone())); } }
        assert!(frames < 60 * FPS, "the meeting never ended");
    }
    assert!(thoughts.iter().any(|t| t.0 == eliza) && thoughts.iter().any(|t| t.0 == bram) && thoughts.iter().any(|t| t.0 == game.player),
        "everyone had their say: {:?}", thoughts);
    assert!(frames > 5 * LINE_FRAMES, "a meeting takes a while ({} frames)", frames);
    assert_eq!(game.world.get_component::<Organization>(game.org).unwrap().members, vec![eliza]);
    assert!(game.log.last().unwrap().contains("Eliza joined"), "{}", game.log.last().unwrap());
}

/// Three villagers aggrieved over the taxes, in the player's org, and the
/// organizer holding the issue: a house meeting will end in the founding call
fn org_ready(g: &mut Game) {
    for &v in &g.villagers[..3] {
        crate::game::desires::add_grievance(&mut g.world, v, crate::game::desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<crate::game::desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
}

#[test]
fn test_an_unanswered_founding_call_lapses_instead_of_halving_pace_forever() {
    let mut g = Game::new();
    org_ready(&mut g);
    let came = g.villagers[..3].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    g.resolve_meeting(&came, 0, id, (false, false, false, false, false));
    assert!(g.seats[0].pending_founding.is_some());
    // held open: the calendar runs at the house-meeting's half pace
    let base = if g.speed == 1 { 21 * FPS } else { 15 * FPS / 4 };
    assert_eq!(g.ticks_per_frame(), base * 2);
    // short of a week unanswered: still held open, still slow
    g.livelihood_tick(g.tick + FOUNDING_OFFER_EXPIRY_DAYS - 1);
    assert!(g.seats[0].pending_founding.is_some());
    assert_eq!(g.ticks_per_frame(), base * 2);
    // a week unanswered (no client ever sent Command::Decide): the
    // call lapses on its own, same as letting it rest, and the
    // calendar returns to its ordinary pace
    g.livelihood_tick(g.tick + FOUNDING_OFFER_EXPIRY_DAYS);
    assert!(g.seats[0].pending_founding.is_none());
    assert_eq!(g.ticks_per_frame(), base);
    assert!(crate::game::campaign::campaign_of(&g.world, g.org).is_none());
}

/// A house meeting is a real conversation: everyone who spoke counts in
/// the sidebar's bond ledger even if the host never used Talk on them
#[test]
fn test_a_house_meeting_counts_toward_the_bond_ledger() {
    let mut g = Game::new();
    org_ready(&mut g);
    let came = g.villagers[..3].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    g.resolve_meeting(&came, 0, id, (false, false, false, false, false));
    let m = g.ui_model();
    assert_eq!(m.panel.social.met, 3, "everyone who spoke at the meeting counts");
}

#[test]
fn test_the_ladder_climbs_from_stranger_to_co_leader() {
    use crate::game::organizing::{stage_of, RelStage};
    let mut g = Game::new();
    let (pe, v) = (g.player, g.villagers[1]);
    assert_eq!(stage_of(&g.world, pe, v), RelStage::Stranger);
    // a word or two makes an acquaintance
    relationships::record_interaction(&mut g.world, pe, relationships::Interaction {
        other: v, kind: relationships::InteractionKind::Conversation, intensity: 0.5 });
    assert_eq!(stage_of(&g.world, pe, v), RelStage::Acquaintance);
    // three pieces of their self-interest told, and they're known to you
    use crate::game::desires::{Known, Problem, Reveal, Want};
    g.world.add_component(v, Known { revealed: vec![
        Reveal::Want(Want::GoodFood), Reveal::Problem(Problem::Taxed), Reveal::Want(Want::Love)] });
    assert_eq!(stage_of(&g.world, pe, v), RelStage::Known);
    // a real bond makes a follower
    crate::game::relationships::find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component_mut::<Relationship>(r)).unwrap().strength = 0.6;
    assert_eq!(stage_of(&g.world, pe, v), RelStage::Follower);
    // and a follower who's taken on the work co-leads
    g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(v);
    assert_eq!(stage_of(&g.world, pe, v), RelStage::CoLeader);
}

#[test]
fn test_you_cant_put_a_claim_on_a_stranger() {
    use crate::game::organizing::{props_on_offer, PropKind};
    let mut g = Game::new();
    let v = g.villagers[1];
    // nothing heard from them: no asks on the table
    assert!(props_on_offer(&g.world, g.org, v).is_empty());
    // one piece heard: the asks open up - come along and the empty roles
    use crate::game::desires::{Known, Problem, Reveal};
    g.world.add_component(v, Known { revealed: vec![Reveal::Problem(Problem::Taxed)] });
    let asks = props_on_offer(&g.world, g.org, v);
    assert!(asks.contains(&PropKind::ComeAlong) && asks.contains(&PropKind::Introduce), "{:?}", asks);
}

#[test]
fn test_a_yes_fills_the_plan_and_the_gather_button_shows() {
    use crate::game::organizing::{MeetingPlan, PropKind, propose};
    let mut g = Game::new();
    let (pe, host, guest) = (g.player, g.villagers[1], g.villagers[2]);
    // the host lends their house, the guest says they'll come (roll 0 = yes)
    let home = crate::game::household::home_of(&g.world, host);
    let home = home.unwrap_or_else(|| { let e = g.world.create_entity(); g.world.add_component(e, Pos { x: 16.0, y: 16.0 }); e });
    g.world.add_component(g.org, MeetingPlan { venue: Some((host, home)), ..Default::default() });
    let (oe, tick) = (g.org, g.tick); let _ = propose(&mut g.world, pe, oe, guest, PropKind::ComeAlong, "Bram", 0.0, tick);
    assert!(g.world.get_component::<MeetingPlan>(g.org).unwrap().ready());
    g.update_ctx_actions();
    assert!(g.seats[0].ctx_actions.contains(&crate::session::CtxAct::Gather));
    // Gather calls it: the committed walk over
    let msg = g.call_meeting();
    assert!(g.meeting.is_some(), "{}", msg);
}

#[test]
fn test_half_the_merely_invited_never_make_it() {
    use crate::game::organizing::shows_up;
    let g = Game::new();
    let v = g.villagers[1];
    // a promised guest with a bond almost always comes; a stranger who
    // just got a flyer is a coin's toss - high rolls keep them home
    assert!(!shows_up(&g.world, g.player, v, (16, 16), false, 0.99));
    assert!(shows_up(&g.world, g.player, v, (16, 16), false, 0.01));
}

#[test]
fn test_a_meeting_with_a_cause_ends_in_next_steps() {
    let mut g = Game::new();
    org_ready(&mut g);
    // found the cause first: the meeting's call is steps, not a vote
    let (oe, tick) = (g.org, g.tick);
    let c = crate::game::campaign::found_campaign(&mut g.world, oe,
        &crate::game::campaign::IssueVote { winner: "the taxes".into(), ballots: vec![] },
        crate::game::campaign::Framing::SharedGrievance, crate::game::campaign::DecisionRule::Majority, tick);
    let _ = c;
    let came = g.villagers[..3].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    g.resolve_meeting(&came, 0, id, (false, false, false, false, false));
    let call = g.convo_panel_at(0).expect("the call on the box");
    assert!(call.options.iter().any(|o| o.contains("Meet again")), "{:?}", call.options);
    assert!(call.options.iter().any(|o| o.contains("street")), "{:?}", call.options);
    // meet again reseeds the plan: they'd come back
    g.decide_at(0, 0);
    let plan = g.world.get_component::<crate::game::organizing::MeetingPlan>(g.org).unwrap();
    assert!(!plan.coming.is_empty(), "the willing are on the next plan");
}

#[test]
fn test_a_follower_greets_you_differently() {
    let mut g = Game::new();
    let (pe, v) = (g.player, g.villagers[1]);
    // a follower: a real bond already earned
    relationships::record_interaction(&mut g.world, pe, relationships::Interaction {
        other: v, kind: relationships::InteractionKind::Conversation, intensity: 0.5 });
    crate::game::relationships::find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component_mut::<Relationship>(r)).unwrap().strength = 0.7;
    use crate::game::organizing::{stage_of, RelStage};
    assert_eq!(stage_of(&g.world, pe, v), RelStage::Follower);
    g.talk_to(1);
    let r = g.talk_log.iter().rev().find(|r| r.kind == crate::session::dialogue::TalkKind::Talk).expect("the talk record");
    let first = &r.lines[0].text;
    let expected: Vec<String> = [
        "{other}! Good day - I was hoping I'd see thee about.",
        "There thou art, {other}. Word is thou'rt still talking to folk.",
        "A fair morning to thee, {other} - and to the work we share.",
        "I saved thee a thought or two since last we spoke, {other}."]
        .iter().map(|s| s.replace("{other}", "wanderer")).collect();
    assert!(expected.contains(first), "a follower greets the bond: {}", first);
}

#[test]
fn test_an_introduction_names_someone_to_look_at() {
    use crate::game::desires::{Known, Problem, Reveal, Want};
    use crate::game::organizing::PropKind;
    use crate::game::vocation::{craft, Vocation};
    let mut g = Game::new();
    let (pe, asker, friend) = (g.player, g.villagers[1], g.villagers[2]);
    let friend_i = g.villagers.iter().position(|&v| v == friend).unwrap();
    // the asker is a follower who knows you, and has someone to name: a
    // bond of theirs in the same trade
    relationships::record_interaction(&mut g.world, pe, relationships::Interaction {
        other: asker, kind: relationships::InteractionKind::Conversation, intensity: 0.5 });
    crate::game::relationships::find_relationship(&g.world, pe, asker)
        .and_then(|r| g.world.get_component_mut::<Relationship>(r)).unwrap().strength = 0.7;
    g.world.add_component(asker, Known { revealed: vec![
        Reveal::Want(Want::GoodFood), Reveal::Want(Want::Love), Reveal::Problem(Problem::Taxed)] });
    g.world.add_component(asker, craft(Vocation::Weaver, 0.5));
    g.world.add_component(friend, craft(Vocation::Weaver, 0.5));
    relationships::record_interaction(&mut g.world, asker, relationships::Interaction {
        other: friend, kind: relationships::InteractionKind::Conversation, intensity: 0.8 });
    // the talk can be started with a building still picked - the named
    // friend takes the selection over, building and all
    g.seats[0].selected_building = Some(0);
    // ask until the proposition lands - the roll is the rng's, so a few
    // tries may be needed; each refusal just asks again
    let mut landed = false;
    for _ in 0..80 {
        if g.seats[0].talk.is_none() {
            let p = *g.world.get_component::<Pos>(asker).unwrap();
            *g.world.get_component_mut::<Pos>(pe).unwrap() = Pos { x: p.x + 0.5, y: p.y };
            g.seats[0].selected_target = 1;
            g.seats[0].picked = true;
            g.act_at(0, 0);
        }
        let Some(t) = g.seats[0].talk.as_ref() else { break };
        let (n, shown) = (t.options.len(), t.asks_shown());
        let intro = t.asks.iter().take(shown).position(|a| *a == PropKind::Introduce);
        match intro {
            Some(k) => {
                g.talk_pick_at(0, Some(n + k));
                if g.seats[0].focus.is_some() { landed = true; break; }
                g.talk_pick_at(0, None);
            }
            None => {
                // the ask isn't on show this turn: spend a shown ask or a
                // reply to move the conversation on
                g.talk_pick_at(0, Some(if shown > 0 { n } else { 0 }));
            }
        }
    }
    assert!(landed, "an introduction lands and the camera's asked over");
    assert!(g.seats[0].picked, "the named one's picked");
    assert_eq!(g.seats[0].selected_building, None, "the building's selection is let go");
    assert_eq!(g.seats[0].selected_target, friend_i, "it's them, not the asker");
    let (fx, fy, ff) = g.seats[0].focus.unwrap();
    let fp = *g.world.get_component::<Pos>(friend).unwrap();
    assert_eq!((fx, fy), (fp.x, fp.y), "the camera's asked to where they stand");
    assert_eq!(ff, g.frame_n, "asked this frame");
    // and the model carries the ask for the client to answer
    assert_eq!(g.ui_model_at(0).focus, Some((fx, fy, ff)));
}

#[test]
fn test_the_person_you_asked_comes_back_around() {
    use crate::game::desires::{Known, Problem, Reveal, Want};
    use crate::game::organizing::{propose, PropKind, RoundBack};
    use crate::game::vocation::{craft, Vocation};
    let mut g = Game::new();
    let (pe, asker, friend) = (g.player, g.villagers[1], g.villagers[2]);
    // the asker is a follower who knows you, with a friend in the same
    // trade to name - the introduction lands (roll 0 is a yes)
    relationships::record_interaction(&mut g.world, pe, relationships::Interaction {
        other: asker, kind: relationships::InteractionKind::Conversation, intensity: 0.5 });
    crate::game::relationships::find_relationship(&g.world, pe, asker)
        .and_then(|r| g.world.get_component_mut::<Relationship>(r)).unwrap().strength = 0.7;
    g.world.add_component(asker, Known { revealed: vec![
        Reveal::Want(Want::GoodFood), Reveal::Want(Want::Love), Reveal::Problem(Problem::Taxed)] });
    g.world.add_component(asker, craft(Vocation::Weaver, 0.5));
    g.world.add_component(friend, craft(Vocation::Weaver, 0.5));
    relationships::record_interaction(&mut g.world, asker, relationships::Interaction {
        other: friend, kind: relationships::InteractionKind::Conversation, intensity: 0.8 });
    let (oe, tick) = (g.org, g.tick);
    let (msg, named) = propose(&mut g.world, pe, oe, asker, PropKind::Introduce, "Eliza", 0.0, tick);
    assert!(named.is_some(), "{}", msg);
    // they said they'd come find you - tomorrow, not this minute
    let rb = *g.world.get_component::<RoundBack>(asker).expect("the return call is noted");
    assert_eq!(rb.to, pe);
    assert_eq!(rb.misses, 0);
    assert!(!g.round_back(asker, tick), "the call isn't due the same day");
    // the next day's round: the day loop sends them over - and the note
    // stands until the greeting lands, so a lost walk is simply retried
    g.livelihood_tick(tick + 1);
    assert!(g.travels.iter().any(|t| t.who == asker && matches!(t.goal, Goal::TalkTo(e) if e == pe)),
        "they set out to find you");
    assert!(g.world.get_component::<RoundBack>(asker).is_some(), "the note stands for the walk");
    // an empty doorstep doesn't count: you've moved on, so they note the
    // miss and try again another day
    let ap = *g.world.get_component::<Pos>(asker).unwrap();
    *g.world.get_component_mut::<Pos>(pe).unwrap() = Pos { x: ap.x + 40.0, y: ap.y };
    g.arrive(asker, Goal::TalkTo(pe));
    assert!(!g.bubbles.iter().any(|b| b.0 == asker && b.1.contains("Good day")),
        "no hello for an empty doorstep");
    let rb = g.world.get_component::<RoundBack>(asker).expect("the miss is noted, not forgotten");
    assert_eq!(rb.misses, 1);
    // back beside them, the next day's round brings the hello
    g.travels.retain(|t| t.who != asker);
    *g.world.get_component_mut::<Pos>(pe).unwrap() = Pos { x: ap.x + 0.5, y: ap.y };
    g.livelihood_tick(tick + 2);
    assert!(g.travels.iter().any(|t| t.who == asker && matches!(t.goal, Goal::TalkTo(e) if e == pe)),
        "they come back another day");
    let before = relationships::find_relationship(&g.world, asker, pe)
        .and_then(|r| g.world.get_component::<Relationship>(r)).map(|r| r.strength).unwrap_or(0.0);
    g.arrive(asker, Goal::TalkTo(pe));
    assert!(g.bubbles.iter().any(|b| b.0 == asker && b.1.contains("Good day to you.")),
        "they greet you on arrival");
    let after = relationships::find_relationship(&g.world, asker, pe)
        .and_then(|r| g.world.get_component::<Relationship>(r)).map(|r| r.strength).unwrap_or(0.0);
    assert!(after > before, "the visit warms the bond");
    assert!(g.world.get_component::<RoundBack>(asker).is_none(), "the note's spent on the hello");
}

#[test]
fn test_an_intro_with_no_friend_sends_you_to_the_village_ear() {
    use crate::game::organizing::{propose, PropKind};
    use crate::game::vocation::{Craft, TownRole, Vocation};
    let mut g = Game::new();
    let (pe, asker) = (g.player, g.villagers[26]);
    // a kid with no trade and nothing heard: no friend can be named -
    // but folk still know where the village's ear is
    let (oe, tick) = (g.org, g.tick);
    let (msg, named) = propose(&mut g.world, pe, oe, asker, PropKind::Introduce, "Wick", 0.0, tick);
    let p = named.expect("someone to send you to");
    let voc = g.world.get_component::<Craft>(p).map(|c| c.vocation);
    let role = g.world.get_component::<TownRole>(p).copied();
    assert!(voc == Some(Vocation::Priest)
        || matches!(role, Some(TownRole::Matchmaker) | Some(TownRole::TaxCollector)),
        "the named one's an open ear: {:?} {:?}", voc, role);
    assert!(msg.contains("tell the"), "{}", msg);
    // and the named one starts a little warm to you
    assert!(relationships::find_relationship(&g.world, pe, p).is_some());
}

#[test]
fn test_a_remote_chaplain_isnt_the_village_ear() {
    use crate::game::organizing::{propose, PropKind};
    use crate::game::vocation::{Craft, TownRole, Vocation};
    let mut g = Game::new();
    let (pe, asker) = (g.player, g.villagers[26]);
    // every priest in the world is out in the country - none hears the
    // parish, so the ear must be a village role, not a distant chaplain
    let church = g.world.query::<Building>().iter().copied()
        .find(|&b| g.world.get_component::<Building>(b)
            .map(|bb| bb.kind == BuildingKind::Church).unwrap_or(false))
        .and_then(|b| g.world.get_component::<Pos>(b).copied()).expect("the church stands");
    let priests: Vec<Entity> = g.world.query::<Craft>().iter().copied()
        .filter(|&e| g.world.get_component::<Craft>(e)
            .map(|c| c.vocation == Vocation::Priest).unwrap_or(false)).collect();
    assert!(!priests.is_empty(), "the country keeps chaplains to exclude");
    for p in priests {
        *g.world.get_component_mut::<Pos>(p).unwrap() = Pos { x: church.x + 200.0, y: church.y };
    }
    let (oe, tick) = (g.org, g.tick);
    let (msg, named) = propose(&mut g.world, pe, oe, asker, PropKind::Introduce, "Wick", 0.0, tick);
    let p = named.expect("someone to send you to");
    let voc = g.world.get_component::<Craft>(p).map(|c| c.vocation);
    let role = g.world.get_component::<TownRole>(p).copied();
    assert!(voc != Some(Vocation::Priest), "a remote chaplain isn't the parish ear: {}", msg);
    assert!(matches!(role, Some(TownRole::Matchmaker) | Some(TownRole::TaxCollector)),
        "a village role answers instead: {:?} {:?}", voc, role);
}
