use crate::session::*;
use crate::game::relationships::RelKind;
use crate::game::organizing::MeetingPlan;
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
    game.world.add_component(rel, Relationship { from: game.player, other: eliza, kind: RelKind::Friend, strength: 0.7, interactions: 6, ..Default::default() });
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
    game.world.add_component(rel, Relationship { from: game.player, other: eliza, kind: RelKind::Friend, strength: 0.7, interactions: 6, ..Default::default() });
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

/// A vote beat reads the count and the names (#1612): each member's
/// ballot lands on the meeting's record, and the call's lean - the log
/// line - carries the tally
#[test]
fn test_a_founding_vote_shows_its_tally() {
    let mut g = Game::new();
    org_ready(&mut g);
    let came = g.villagers[..3].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    let msg = g.resolve_meeting(&came, 0, id, (false, false, false, false, false));
    let d = g.seats[0].pending_founding.as_ref().expect("the call's on the table");
    assert!(d.vote.is_some(), "a room with a heard issue takes the vote");
    // each member's pick named in the transcript
    let rec = g.talk_log.iter().find(|r| r.id == id).unwrap();
    let ballots: Vec<&crate::session::dialogue::TalkLine> = rec.lines.iter()
        .filter(|l| l.text.starts_with("votes ") || l.text == "holds back").collect();
    assert_eq!(ballots.len(), 3, "every member's ballot is named: {:?}",
        rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
    assert!(ballots.iter().all(|l| l.who.map(|w| came.contains(&w)).unwrap_or(false)),
        "a ballot names its member");
    // and the call reads the count - `resolve_meeting`'s returned line
    // is what the log gets
    let winner = crate::game::desires::issue_name(d.vote.as_ref().unwrap().winner.clone());
    let tally = format!("3 for {}", winner);
    assert!(msg.contains(&tally), "the tally rides the call: {}", msg);
    // and the founding log line carries it too, once it's taken
    let note = g.maybe_found_campaign(0).expect("the vote still stands");
    assert!(note.contains(&tally), "the tally rides the log line: {}", note);
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
    use crate::game::desires::{Problem, Reveal, Want};
    let __pe = g.player; for __r in vec![
        Reveal::Want(Want::GoodFood), Reveal::Problem(Problem::Taxed), Reveal::Want(Want::Love)] { crate::game::desires::note(&mut g.world, __pe, v, __r); }
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
    use crate::game::organizing::{props_on_offer, PropKind, RelStage};
    use crate::game::desires::{Problem, Reveal, Want};
    use crate::game::relationships::{record_interaction, find_relationship,
        Interaction, InteractionKind, Relationship};
    let mut g = Game::new();
    let (pe, v) = (g.player, g.villagers[1]);
    // nothing heard from them: no asks on the table
    assert!(props_on_offer(&g.world, g.org, v).is_empty());
    // one piece heard but no bond yet - heard alone isn't a claim
    let __pe = g.player; for __r in vec![Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    assert_eq!(props_on_offer(&g.world, g.org, v), Vec::<PropKind>::new());
    // an acquaintance hears the light ask only: who to know next
    record_interaction(&mut g.world, pe, Interaction {
        other: v, kind: InteractionKind::Conversation, intensity: 0.3 });
    let asks = props_on_offer(&g.world, g.org, v);
    assert_eq!(asks, vec![PropKind::Introduce], "{:?}", asks);
    // known to you (three pieces): they'll hear about bringing friends,
    // but a meeting's still too big a claim (#1188)
    let __pe = g.player; for __r in vec![
        Reveal::Problem(Problem::Taxed), Reveal::Want(Want::GoodFood), Reveal::Want(Want::Love)] { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    let asks = props_on_offer(&g.world, g.org, v);
    assert!(asks.contains(&PropKind::Introduce) && asks.contains(&PropKind::InviteFriends), "{:?}", asks);
    assert!(!asks.contains(&PropKind::ComeAlong), "{:?}", asks);
    // a follower hears the meeting asks - the bond can carry them
    find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component_mut::<Relationship>(r)).unwrap().strength = 0.6;
    assert_eq!(crate::game::organizing::stage_of(&g.world, pe, v), RelStage::Follower);
    let asks = props_on_offer(&g.world, g.org, v);
    assert!(asks.contains(&PropKind::ComeAlong) && asks.contains(&PropKind::OpenWithWord)
        && asks.contains(&PropKind::Introduce), "{:?}", asks);
}

#[test]
fn test_one_job_apiece_spreads_the_meetings_work() {
    use crate::game::organizing::{props_on_offer, propose, PropKind};
    use crate::game::desires::{Problem, Reveal};
    let mut g = Game::new();
    let (pe, first, next) = (g.player, g.villagers[1], g.villagers[2]);
    for v in [first, next] {
        let __pe = g.player; for __r in vec![Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, v, __r); }
        // the meeting asks only open for a follower (#1188)
        crate::game::relationships::record_interaction(&mut g.world, pe,
            crate::game::relationships::Interaction { other: v,
                kind: crate::game::relationships::InteractionKind::Conversation, intensity: 0.3 });
        crate::game::relationships::find_relationship(&g.world, pe, v)
            .and_then(|r| g.world.get_component_mut::<crate::game::relationships::Relationship>(r))
            .unwrap().strength = 0.6;
    }
    // the first villager's promised the snacks: the rest of the roles
    // leave their card, though they'll still hear the standing asks
    let roles = [PropKind::BringSnacks, PropKind::TakeNotes, PropKind::OpenWithWord,
        PropKind::GiveSong, PropKind::CloseOut];
    let (oe, tick) = (g.org, g.tick);
    let _ = propose(&mut g.world, pe, oe, first, PropKind::BringSnacks, "Bram", 0.0, tick);
    let asks = props_on_offer(&g.world, g.org, first);
    assert!(roles.iter().all(|r| !asks.contains(r)), "{:?}", asks);
    assert!(asks.contains(&PropKind::ComeAlong) && asks.contains(&PropKind::Introduce));
    // and the next person still gets every role that's still unfilled
    let asks = props_on_offer(&g.world, g.org, next);
    for r in [PropKind::TakeNotes, PropKind::OpenWithWord, PropKind::GiveSong, PropKind::CloseOut] {
        assert!(asks.contains(&r), "{:?}", asks);
    }
}

#[test]
fn test_the_word_alone_convenes_a_meeting_at_the_square() {
    use crate::game::organizing::{MeetingPlan, PropKind, propose};
    use crate::game::desires::{Problem, Reveal};
    let mut g = Game::new();
    let pe = g.player;
    // no house lent - just a round of come-alongs the word carried.
    // Enough of them that the turnout roll can't empty the square
    let guests: Vec<_> = g.villagers[1..9].to_vec();
    for &v in &guests {
        let __pe = g.player; for __r in vec![Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    }
    let (oe, tick) = (g.org, g.tick);
    for v in guests {
        let _ = propose(&mut g.world, pe, oe, v, PropKind::ComeAlong, "Bram", 0.0, tick);
    }
    assert!(g.world.get_component::<MeetingPlan>(g.org).unwrap().ready());
    let msg = g.call_meeting();
    assert!(g.meeting.is_some(), "{}", msg);
    let m = g.meeting.as_ref().unwrap();
    // the cross is the standing ground when nobody lent a house
    assert_eq!(m.venue, g.village);
    assert_eq!(m.label, "the square");
}

#[test]
fn test_a_yes_fills_the_plan_and_the_gather_button_shows() {
    use crate::game::organizing::{MeetingPlan, PropKind, propose};
    let mut g = Game::new();
    let (pe, host, guest) = (g.player, g.villagers[1], g.villagers[2]);
    // the host lends their house, the guest says they'll come (roll 0 = yes)
    let home = crate::game::household::home_of(&g.world, host);
    let home = home.unwrap_or_else(|| { let e = g.world.create_entity(); g.world.add_component(e, Pos { x: 16.0, y: 16.0 }); e });
    g.world.add_component(g.org, MeetingPlan { venue: Some(crate::game::organizing::Venue::House(host, home)), ..Default::default() });
    let (oe, tick) = (g.org, g.tick); let _ = propose(&mut g.world, pe, oe, guest, PropKind::ComeAlong, "Bram", 0.0, tick);
    assert!(g.world.get_component::<MeetingPlan>(g.org).unwrap().ready());
    g.update_ctx_actions();
    // the Gather button's gone (#1596): no ctx button calls the meeting
    assert!(g.seats[0].ctx_actions.iter().all(|&a| ctx_button(a).label != "Gather"),
        "the Gather button's back");
    // the call itself still convenes the committed
    let msg = g.call_meeting();
    assert!(g.meeting.is_some(), "{}", msg);
}

#[test]
fn test_half_the_merely_invited_never_make_it() {
    use crate::game::organizing::{shows_up, Invite};
    let mut g = Game::new();
    let v = g.villagers[1];
    // a staked stranger who just got a flyer is a coin's toss - high
    // rolls keep them home
    crate::game::desires::add_grievance(&mut g.world, v,
        crate::game::desires::Problem::Taxed, g.tick + 100);
    assert!(!shows_up(&g.world, g.player, v, (16, 16), Invite::Word, 0.99, g.tick));
    assert!(shows_up(&g.world, g.player, v, (16, 16), Invite::Word, 0.01, g.tick));
}

#[test]
fn test_the_disinterested_drift_in_thinly_not_never() {
    use crate::game::organizing::{shows_up, Invite};
    use crate::game::desires::{self, add_grievance, Problem};
    use crate::game::relationships::{self, RelKind};
    let mut g = Game::new();
    // someone with nothing left to say: no wants, no problems hanging
    let pe = g.player; let v = g.villagers.iter().copied().find(|&v| {
        if let Some(l) = g.world.get_component_mut::<desires::InnerLife>(v) { l.wants.clear(); }
        desires::unrevealed(&g.world, pe, v, g.tick).is_empty()
    }).expect("someone content enough to have nothing to say");
    // the meeting comes to them - a zero-length walk keeps the roll honest
    let at = {
        let p = *g.world.get_component::<Pos>(v).unwrap();
        (p.x as i32, p.y as i32)
    };
    // curiosity still drifts them in, at a quarter's pull: a tenth of the
    // room answers the coin, three-quarters of its rolls don't
    assert!(!shows_up(&g.world, g.player, v, at, Invite::Word, 0.2, g.tick),
        "nothing to say: most rolls keep them home");
    assert!(shows_up(&g.world, g.player, v, at, Invite::Word, 0.05, g.tick),
        "but a drift-in still happens");
    // a grievance is a reason: the wronged roll the full coin
    add_grievance(&mut g.world, v, Problem::Taxed, g.tick + 100);
    assert!(shows_up(&g.world, g.player, v, at, Invite::Word, 0.4, g.tick),
        "a live grievance draws them out");
    assert!(!shows_up(&g.world, g.player, v, at, Invite::Word, 0.6, g.tick));
    // once it's spent they're back to thin rolls
    let g2 = g.tick + 200;
    assert!(!shows_up(&g.world, g.player, v, at, Invite::Word, 0.2, g2),
        "a spent grievance no longer draws");
    // but a friend holding the door is reason enough on its own -
    // no stake asked of a pulled guest (#1075)
    assert!(shows_up(&g.world, g.player, v, at, Invite::Pulled, 0.4, g2),
        "a pulled friend needs no stake");
    // and a true friend of the host needs no other reason
    let pe = g.player; let w = g.villagers.iter().copied().find(|&w| {
        if let Some(l) = g.world.get_component_mut::<desires::InnerLife>(w) { l.wants.clear(); }
        desires::unrevealed(&g.world, pe, w, g2).is_empty()
    }).expect("another contented one");
    let at = {
        let p = *g.world.get_component::<Pos>(w).unwrap();
        (p.x as i32, p.y as i32)
    };
    let rel = g.world.create_entity();
    g.world.add_component(rel, relationships::Relationship {
        from: g.player, other: w, kind: RelKind::Friend, strength: 0.8, interactions: 5, ..Default::default() });
    assert!(shows_up(&g.world, g.player, w, at, Invite::Word, 0.4, g2),
        "a real bond counts as its own stake");
}

/// The door's thinner than the asking: when the pool shrinks the word
/// goes out with a count, not a roll of who stayed in
#[test]
fn test_the_door_counts_who_stayed_home_without_naming_them() {
    let mut g = Game::new();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: 16.0, y: 16.0 };
    let host = g.villagers[1];
    let (bram, corvin) = (g.villagers[2], g.villagers[3]);
    for (v, x, y) in [(host, 17.0, 16.0), (bram, 16.0, 17.0)] {
        *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x, y };
    }
    // Corvin gave his yes, but a grave keeps its own hours - a promise
    // the door can't collect
    g.world.add_component(corvin, villager::Dead);
    let rel = g.world.create_entity();
    g.world.add_component(rel, Relationship { from: g.player, other: host,
        kind: RelKind::Friend, strength: 0.7, interactions: 6, ..Default::default() });
    plan_ready_meeting(&mut g, host, (17.0, 16.0), &[bram, corvin]);
    g.call_meeting();
    let last = g.log.last().unwrap();
    assert!(last.contains("of those asked stayed home"), "the count's told plainly: {last}");
    assert!(!last.contains("Corvin"), "and nobody's named for it: {last}");
}

/// The Gather press asks a day instead of firing: the word carries the
/// call, the board posts it, and the door only rolls when the day comes
#[test]
fn test_a_called_meeting_waits_for_the_day_its_word_names() {
    let mut g = Game::new();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: 16.0, y: 16.0 };
    let host = g.villagers[1];
    let bram = g.villagers[2];
    for (v, x, y) in [(host, 17.0, 16.0), (bram, 16.0, 17.0)] {
        *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x, y };
    }
    let rel = g.world.create_entity();
    g.world.add_component(rel, Relationship { from: g.player, other: host,
        kind: RelKind::Friend, strength: 0.7, interactions: 6, ..Default::default() });
    plan_ready_meeting(&mut g, host, (17.0, 16.0), &[bram]);
    // a scheduling answer - the Meet again one, since the Gather
    // button's gone (#1596) - asks the day on the box; nothing
    // gathers until it's picked
    g.seats[0].meet_offer = Some(g.tick);
    assert!(g.meeting.is_none(), "the ask waits for a day, it doesn't fire");
    let panel = g.convo_panel_at(0).expect("the day question's on the box");
    assert!(panel.decides && panel.options.len() == 3, "{:?}", panel.options);
    // two days out: the plan keeps its promises and the board carries it
    g.decide_at(0, 0);
    let plan = g.world.get_component::<MeetingPlan>(g.org).unwrap();
    assert_eq!(plan.for_day, Some(g.tick + 2), "the call's dated");
    assert!(plan.coming.contains(&bram), "the plan stays open on the org");
    assert!(g.notices.iter().any(|(k, _, u)| k == "meet" && *u == g.tick + 2),
        "the board carries the call till the day: {:?}", g.notices);
    // the day arrives: the door rolls then, not before
    g.livelihood_tick(g.tick + 1);
    assert!(g.meeting.is_none(), "a day early is a day early");
    g.livelihood_tick(g.tick + 2);
    assert!(g.meeting.is_some(), "the day fires the call: {:?}", g.log.last());
    assert!(g.log.iter().any(|l| l.contains("gather at")), "the word goes out");
}

/// A lent house that can't hold the day cancels the call outright
#[test]
fn test_a_lost_house_calls_the_meeting_off() {
    let mut g = Game::new();
    let host = g.villagers[1];
    let bram = g.villagers[2];
    plan_ready_meeting(&mut g, host, (17.0, 16.0), &[bram]);
    g.seats[0].meet_offer = Some(g.tick);
    g.decide_at(0, 2); // a week out
    let for_day = g.world.get_component::<MeetingPlan>(g.org).unwrap().for_day.unwrap();
    // the house's keeper dies before the day
    g.world.add_component(host, villager::Dead);
    g.livelihood_tick(for_day);
    assert!(g.meeting.is_none(), "no meeting without the house");
    assert!(g.world.get_component::<MeetingPlan>(g.org).is_none(),
        "the plan's dropped with the call");
    assert!(g.log.iter().any(|l| l.contains("the meeting's off")), "{}", g.log.last().unwrap());
}

/// A date question nobody answers lapses like any unanswered call
#[test]
fn test_an_unanswered_date_question_lapses() {
    let mut g = Game::new();
    let host = g.villagers[1];
    let bram = g.villagers[2];
    plan_ready_meeting(&mut g, host, (17.0, 16.0), &[bram]);
    g.seats[0].meet_offer = Some(g.tick);
    g.livelihood_tick(g.tick + FOUNDING_OFFER_EXPIRY_DAYS);
    assert!(g.seats[0].meet_offer.is_none(), "the question lapses");
    assert!(g.world.get_component::<MeetingPlan>(g.org).unwrap().for_day.is_none(),
        "and nothing's dated");
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
    // meet again reseeds the plan: they'd come back - and the answer
    // reopens the day question itself now that no button's left to (#1596)
    g.decide_at(0, 0);
    let plan = g.world.get_component::<crate::game::organizing::MeetingPlan>(g.org).unwrap();
    assert!(!plan.coming.is_empty(), "the willing are on the next plan");
    assert!(g.seats[0].meet_offer.is_some(), "the box asks when they'll gather");
    let panel = g.convo_panel_at(0).expect("the day question follows the answer");
    assert!(panel.decides && panel.options.len() == 3, "{:?}", panel.options);
    g.decide_at(0, 0); // two days out
    assert_eq!(g.world.get_component::<crate::game::organizing::MeetingPlan>(g.org)
        .unwrap().for_day, Some(g.tick + 2), "the follow-up's dated");
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
    let t = g.seats[0].talk.as_ref().expect("the talk's open");
    let expected: Vec<String> = [
        "{other}! Good day - I was hoping I'd see {2:obj} about.",
        "There {2:subj} {2:be}, {other}. Word is {2:subj} {2:be} still talking to folk.",
        "A fair morning to {2:obj}, {other} - and to the work we share.",
        "I saved {2:obj} a thought or two since last we spoke, {other}."]
        .iter().map(|s| super::fixtures::said(t, false, *s)).collect();
    assert!(expected.contains(first), "a follower greets the bond: {}", first);
}

#[test]
fn test_an_introduction_names_someone_to_look_at() {
    use crate::game::desires::{Problem, Reveal, Want};
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
    let __pe = g.player; for __r in vec![
        Reveal::Want(Want::GoodFood), Reveal::Want(Want::Love), Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, asker, __r); }
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
        let intro = t.asks.iter().take(shown).position(|a| a.kind == PropKind::Introduce);
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
    use crate::game::desires::{Problem, Reveal, Want};
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
    let __pe = g.player; for __r in vec![
        Reveal::Want(Want::GoodFood), Reveal::Want(Want::Love), Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, asker, __r); }
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
        // a plan where they now stand, so a later sim doesn't walk them back
        g.plan_visit(p, g.tick);
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

#[test]
fn test_asking_someone_to_bring_friends_is_a_yes_or_a_no() {
    use crate::game::organizing::{propose, MeetingPlan, PropKind};
    let mut g = Game::new();
    let (pe, bram, corvin) = (g.player, g.villagers[2], g.villagers[3]);
    let (oe, tick) = (g.org, g.tick);
    // the ask lands: they go on the spreaders
    let (yes, _) = propose(&mut g.world, pe, oe, bram, PropKind::InviteFriends, "Bram", 0.0, tick);
    assert!(yes.contains("friends"), "{}", yes);
    assert!(g.world.get_component::<MeetingPlan>(oe).unwrap().spreaders.contains(&bram));
    // and a refusal doesn't
    let (no, _) = propose(&mut g.world, pe, oe, corvin, PropKind::InviteFriends, "Corvin", 0.99, tick);
    assert!(no.contains("won't hear it"), "{}", no);
    assert!(!g.world.get_component::<MeetingPlan>(oe).unwrap().spreaders.contains(&corvin));
}

#[test]
fn test_the_promised_parts_of_a_meeting_actually_happen() {
    use crate::game::organizing::MeetingPlan;
    let mut game = Game::new();
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: 16.0, y: 16.0 };
    let (host, snacker, noter, opener, singer, closer) =
        (game.villagers[1], game.villagers[2], game.villagers[3], game.villagers[4], game.villagers[5], game.villagers[6]);
    for (i, v) in game.villagers.clone().into_iter().enumerate() {
        *game.world.get_component_mut::<Pos>(v).unwrap() =
            if i == 1 { Pos { x: 17.0, y: 16.0 } } else { Pos { x: 16.0 + (i % 5) as f32, y: 14.0 + (i / 5) as f32 * 0.5 } };
    }
    plan_ready_meeting(&mut game, host, (17.0, 16.0), &[]);
    {
        let plan = game.world.get_component_mut::<MeetingPlan>(game.org).unwrap();
        plan.snacks = Some(snacker); plan.notes = Some(noter); plan.opener = Some(opener);
        plan.song = Some(singer); plan.closer = Some(closer);
    }
    game.call_meeting();
    while game.meeting.as_ref().map(|m| m.lines.is_empty()).unwrap_or(false) { game.update(); }
    let m = game.meeting.as_ref().expect("the meeting's under way");
    // every part happens, in its place: the opener's welcome first,
    // the bread and the song along the way, the notes and the
    // send-off last - and the song fills the music-want it promised
    assert_eq!(m.lines[0].0, opener, "the opener speaks first: {:?}", m.lines.first());
    assert!(m.lines[0].1.contains("you came"), "a welcome, not the host's question: {}", m.lines[0].1);
    assert!(m.lines.iter().any(|&(w, ref t)| w == snacker && t.contains("Bread's on the table")), "the bread goes round");
    assert!(m.lines.iter().any(|&(w, ref t)| w == singer && t.contains("road to the fair")), "the song is sung");
    assert!(m.lines.iter().any(|&(w, ref t)| w == noter && t.contains("keep the word")), "the notes keep it");
    // the meeting's rite gives the very last word - the promised
    // send-off lands just ahead of it (#1185)
    assert_eq!(m.lines[m.lines.len() - 2].0, closer, "the closer sends them off, ahead of the rite's own");
    assert!(crate::game::desires::is_satisfied(&game.world, singer, crate::game::desires::Want::Music, game.tick),
        "the song warmed the room's music-want");
    // a promised part whose keeper never made it stays un-kept
    let mut game2 = Game::new();
    *game2.world.get_component_mut::<Pos>(game2.player).unwrap() = Pos { x: 16.0, y: 16.0 };
    let (host2, song2) = (game2.villagers[1], game2.villagers[9]);
    for (i, v) in game2.villagers.clone().into_iter().enumerate() {
        *game2.world.get_component_mut::<Pos>(v).unwrap() =
            if i == 1 { Pos { x: 17.0, y: 16.0 } } else { Pos { x: 16.0 + (i % 5) as f32, y: 14.0 + (i / 5) as f32 * 0.5 } };
    }
    plan_ready_meeting(&mut game2, host2, (17.0, 16.0), &[]);
    // the singer promised but sits in the cells: no song is sung
    game2.world.get_component_mut::<MeetingPlan>(game2.org).unwrap().song = Some(song2);
    game2.world.add_component(song2, crate::game::dungeon::Imprisoned { since: game2.tick, charge: "the lord's suspicion".into() });
    game2.call_meeting();
    while game2.meeting.as_ref().map(|m| m.lines.is_empty()).unwrap_or(false) { game2.update(); }
    let m2 = game2.meeting.as_ref().expect("the meeting's under way");
    assert!(!m2.roles.3, "a song whose singer's held sings nothing");
    assert!(m2.lines.iter().all(|(_, t)| !t.contains("road to the fair")), "no song aired");
}

#[test]
fn test_the_committed_bring_a_friend_along() {
    use crate::game::organizing::MeetingPlan;
    use crate::game::relationships::{Relationship, RelKind};
    let mut game = Game::new();
    let (host, comer, friend) = (game.villagers[1], game.villagers[2], game.villagers[4]);
    // the venue's at the host's; the friend waits close by
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: 16.0, y: 16.0 };
    for (v, x, y) in [(host, 17.0, 16.0), (friend, 18.0, 17.0)] {
        *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x, y };
    }
    for (i, v) in game.villagers.clone().into_iter().enumerate() {
        if i != 1 && i != 4 { *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 3.0 + i as f32 * 0.5, y: 3.0 }; }
    }
    // the comer's strongest bond is the friend, and the friend knows
    // the player well enough to make the walk
    let r = game.world.create_entity();
    game.world.add_component(r, Relationship { from: comer, other: friend, kind: RelKind::Friend, strength: 9.0, interactions: 3, ..Default::default() });
    let rf = game.world.create_entity();
    game.world.add_component(rf, Relationship { from: game.player, other: friend, kind: RelKind::Friend, strength: 3.0, interactions: 5, ..Default::default() });
    plan_ready_meeting(&mut game, host, (17.0, 16.0), &[]);
    // the comer gave a bare yes - no spreading promised - and still a
    // friend comes in tow
    game.world.get_component_mut::<MeetingPlan>(game.org).unwrap().coming = vec![comer];
    game.call_meeting();
    let invited = game.meeting.as_ref().expect("a meeting was called").invited.clone();
    assert!(invited.contains(&comer), "the comer came: {:?}", invited);
    assert!(invited.contains(&friend), "a yes holds the door for a friend: {:?}", invited);
}

#[test]
fn test_spreaders_bring_their_friends_and_nobody_twice() {
    use crate::game::organizing::MeetingPlan;
    let mut game = Game::new();
    let (host, spreader_a, spreader_b, friend) = (game.villagers[1], game.villagers[2], game.villagers[3], game.villagers[4]);
    // the venue's at Eliza's; the friend waits close by, everyone else is far
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: 16.0, y: 16.0 };
    for (v, x, y) in [(host, 17.0, 16.0), (friend, 18.0, 17.0)] {
        *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x, y };
    }
    for (i, v) in game.villagers.clone().into_iter().enumerate() {
        if i != 1 && i != 4 { *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 3.0 + i as f32 * 0.5, y: 3.0 }; }
    }
    // both spreaders count the same soul their strongest bond - the
    // overlap must still mean one invite
    for s in [spreader_a, spreader_b] {
        let r = game.world.create_entity();
        game.world.add_component(r, Relationship { from: s, other: friend, kind: RelKind::Friend, strength: 9.0, interactions: 3, ..Default::default() });
    }
    // and the friend knows the player well enough to actually come
    let rf = game.world.create_entity();
    game.world.add_component(rf, Relationship { from: game.player, other: friend, kind: RelKind::Friend, strength: 3.0, interactions: 5, ..Default::default() });
    plan_ready_meeting(&mut game, host, (17.0, 16.0), &[]);
    game.world.get_component_mut::<MeetingPlan>(game.org).unwrap().spreaders = vec![spreader_a, spreader_b];
    game.call_meeting();
    let invited = game.meeting.as_ref().expect("a meeting was called").invited.clone();
    assert!(invited.contains(&friend), "the friend came: {:?}", invited);
    assert_eq!(invited.iter().filter(|&&e| e == friend).count(), 1,
        "one invite however many spreaders name them: {:?}", invited);
}

/// Coin reaches where the bond can't (#1225): a stranger - nothing
/// heard, no bond - is offered only priced asks, and taking one pays
/// the purse, lands the promise, and marks it bought on the plan
#[test]
fn test_coin_buys_an_ask_the_bond_cant_reach() {
    use crate::game::organizing::{ask_label, ask_offers, propose_keyed, AskKey, MeetingPlan, PropKind};
    use crate::game::livelihood::Purse;
    use crate::game::relationships::find_relationship;
    let mut g = Game::new();
    let (pe, v) = (g.player, g.villagers[1]);
    g.world.get_component_mut::<Purse>(pe).unwrap().gold = 100.0;
    // they'll take coin - nothing in them refuses the asking
    g.world.add_component(v, crate::game::traits::Personality { traits: vec![] });
    // nothing heard, nothing between you: coin's the only key that turns
    let offers = ask_offers(&g.world, g.org, v, 0, g.tick);
    assert!(offers.iter().all(|o| o.key == AskKey::Coin), "{:?}", offers);
    let o = offers.iter().find(|o| o.kind == PropKind::ComeAlong).expect("come is priced");
    assert!(ask_label(&g.world, g.player, v, o).contains("for "), "{}", ask_label(&g.world, g.player, v, o));
    let (oe, tick) = (g.org, g.tick);
    let (msg, _) = propose_keyed(&mut g.world, pe, oe, v, o.kind, "Bram", 0.0, tick, o.key, o.price);
    assert!(msg.contains("coin"), "{}", msg);
    let plan = g.world.get_component::<MeetingPlan>(g.org).unwrap();
    assert!(plan.coming.contains(&v) && plan.bought.contains(&v), "{:?}", plan.coming);
    // the price came out of the purse, and a bought yes buys no warmth
    let gold = g.world.get_component::<Purse>(pe).unwrap().gold;
    assert!((gold - (100.0 - o.price)).abs() < 0.001, "gold {}", gold);
    assert!(find_relationship(&g.world, pe, v).is_none(), "a bought yes is no warmth");
}

/// The folk who won't be bought don't forgive the asking (#1225): a
/// Proud or Stingy villager refuses the coin outright, keeps their
/// purse's distance, and the offer itself costs a chill
#[test]
fn test_the_proud_wont_be_bought() {
    use crate::game::organizing::{propose_keyed, AskKey, MeetingPlan, PropKind};
    use crate::game::livelihood::Purse;
    use crate::game::relationships::{find_relationship, Relationship};
    use crate::game::traits::{Personality, Trait};
    let mut g = Game::new();
    let (pe, v) = (g.player, g.villagers[1]);
    g.world.get_component_mut::<Purse>(pe).unwrap().gold = 100.0;
    g.world.add_component(v, Personality { traits: vec![Trait::Proud] });
    let (oe, tick) = (g.org, g.tick);
    let (msg, _) = propose_keyed(&mut g.world, pe, oe, v, PropKind::ComeAlong,
        "Bram", 0.0, tick, AskKey::Coin, 8.0);
    assert!(msg.contains("won't be bought"), "{}", msg);
    let r = find_relationship(&g.world, pe, v)
        .and_then(|e| g.world.get_component::<Relationship>(e)).unwrap();
    assert!(r.strength < 0.0, "the asking itself smarts: {}", r.strength);
    assert_eq!(g.world.get_component::<Purse>(pe).unwrap().gold, 100.0);
    assert!(g.world.get_component::<MeetingPlan>(g.org).map(|p| p.coming.is_empty()).unwrap_or(true));
}

/// Nerve presses past the rung you haven't reached (#1225): a piece of
/// self-interest heard is thread enough to pull on - a pressed yes
/// lands and warms deeper than a plain one; a pressed no costs the
/// decline's chill
#[test]
fn test_nerve_presses_past_the_stage_gate() {
    use crate::game::organizing::{ask_offers, propose_keyed, AskKey, MeetingPlan, PropKind};
    use crate::game::desires::{Problem, Reveal};
    use crate::game::relationships::{find_relationship, record_interaction,
        Interaction, InteractionKind, Relationship};
    let mut g = Game::new();
    let (pe, v, v2) = (g.player, g.villagers[1], g.villagers[2]);
    for p in [v, v2] {
        let __pe = g.player; for __r in vec![Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, p, __r); }
        record_interaction(&mut g.world, pe, Interaction {
            other: p, kind: InteractionKind::Conversation, intensity: 0.3 });
    }
    // an acquaintance: the meeting asks are above their rung, so the
    // press - not the plain ask - is what's on the table for them
    let offers = ask_offers(&g.world, g.org, v, 0, g.tick);
    let pressed = offers.iter().find(|o| o.kind == PropKind::ComeAlong && o.key == AskKey::Nerve);
    assert!(pressed.is_some(), "{:?}", offers);
    assert!(!offers.iter().any(|o| o.kind == PropKind::ComeAlong && o.key == AskKey::Earned));
    // the press lands: the promise is theirs, and the warmth runs past
    // a plain yes's - the gamble paid on both ends
    let before = find_relationship(&g.world, pe, v)
        .and_then(|e| g.world.get_component::<Relationship>(e)).unwrap().strength;
    let (oe, tick) = (g.org, g.tick);
    let (msg, _) = propose_keyed(&mut g.world, pe, oe, v, PropKind::ComeAlong,
        "Bram", 0.0, tick, AskKey::Nerve, 0.0);
    assert!(msg.contains("mean it"), "{}", msg);
    let after = find_relationship(&g.world, pe, v)
        .and_then(|e| g.world.get_component::<Relationship>(e)).unwrap().strength;
    assert!(after - before > 0.1, "a pressed yes warms past a plain one: {} -> {}", before, after);
    assert!(g.world.get_component::<MeetingPlan>(g.org).unwrap().coming.contains(&v));
    // and when the press fails it costs what a no costs
    let before = find_relationship(&g.world, pe, v2)
        .and_then(|e| g.world.get_component::<Relationship>(e)).unwrap().strength;
    let (msg, _) = propose_keyed(&mut g.world, pe, oe, v2, PropKind::ComeAlong,
        "Corvin", 1.0, tick, AskKey::Nerve, 0.0);
    assert!(msg.contains("won't hear it"), "{}", msg);
    let after = find_relationship(&g.world, pe, v2)
        .and_then(|e| g.world.get_component::<Relationship>(e)).unwrap().strength;
    assert!(after < before, "a pressed no chills: {} -> {}", before, after);
    assert!(!g.world.get_component::<MeetingPlan>(g.org).unwrap().coming.contains(&v2));
}

/// A thin purse prices the coin out (#1225): nothing in the purse, no
/// priced asks on the table - the nerve's press is the only key left
#[test]
fn test_a_short_purse_offers_no_coin() {
    use crate::game::organizing::{ask_offers, AskKey};
    use crate::game::livelihood::Purse;
    use crate::game::desires::{Problem, Reveal};
    let mut g = Game::new();
    let (pe, v, heard_v) = (g.player, g.villagers[1], g.villagers[2]);
    g.world.get_component_mut::<Purse>(pe).unwrap().gold = 0.0;
    // the stranger has nothing for you: coin's the only key that turns
    // for them, and yours is empty
    assert!(ask_offers(&g.world, g.org, v, 0, g.tick).is_empty());
    // someone you've heard still takes the press, free of coin
    let __pe = g.player; for __r in vec![Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, heard_v, __r); }
    let offers = ask_offers(&g.world, g.org, heard_v, 0, g.tick);
    assert!(offers.iter().all(|o| o.key == AskKey::Nerve), "{:?}", offers);
}

/// A refused ask cools off for the season (#1347): the "won't hear it"
/// takes that ask off the table till its days are out - then it can be
/// put again, and land differently
#[test]
fn test_a_refused_ask_cools_off_for_the_season() {
    use crate::game::organizing::{ask_offers, propose_keyed, AskKey, PropKind};
    use crate::game::desires::{Problem, Reveal};
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
    let mut g = Game::new();
    let (pe, v) = (g.player, g.villagers[1]);
    let __pe = g.player; for __r in vec![Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    record_interaction(&mut g.world, pe, Interaction {
        other: v, kind: InteractionKind::Conversation, intensity: 0.3 });
    let (oe, tick) = (g.org, g.tick);
    assert!(ask_offers(&g.world, oe, v, 0, tick).iter()
        .any(|o| o.kind == PropKind::ComeAlong && o.key == AskKey::Nerve),
        "the press is on the table");
    let (msg, _) = propose_keyed(&mut g.world, pe, oe, v, PropKind::ComeAlong,
        "Bram", 1.0, tick, AskKey::Nerve, 0.0);
    assert!(msg.contains("won't hear it"), "{}", msg);
    assert!(!ask_offers(&g.world, oe, v, 0, tick).iter()
        .any(|o| o.kind == PropKind::ComeAlong && o.key == AskKey::Nerve),
        "the refused ask cools off");
    // the season out, it's back - hearts change, the second ask lands
    // on a new roll
    assert!(ask_offers(&g.world, oe, v, 0, tick + crate::game::organizing::ASK_COOLDOWN_DAYS).iter()
        .any(|o| o.kind == PropKind::ComeAlong && o.key == AskKey::Nerve),
        "the ask re-qualifies when its days are out");
}

/// A purchased yes reads cheap at the reckoning (#1225): bought
/// promise-keepers still fill their slots and show, but they pull no
/// friends in their name and a bought role counts nothing toward the
/// meeting's kept-promise warmth
#[test]
fn test_bought_promises_read_cheap_at_the_meeting() {
    use crate::game::organizing::{propose_keyed, AskKey, PropKind};
    use crate::game::livelihood::Purse;
    use crate::game::relationships::Relationship;
    let mut g = Game::new();
    let (pe, host, spreader, snack) = (g.player, g.villagers[1],
        g.villagers[2], g.villagers[4]);
    // the friend is nobody else's bond: only the spreader's word could
    // ever bring them, so their absence is the bought mark's doing
    let friend = g.world.create_entity();
    g.villagers.push(friend);
    g.world.get_component_mut::<Purse>(pe).unwrap().gold = 100.0;
    plan_ready_meeting(&mut g, host, (17.0, 16.0), &[host]);
    // a bought snacks-keeper and a bought spreader - both will take it
    for p in [snack, spreader] {
        g.world.add_component(p, crate::game::traits::Personality { traits: vec![] });
    }
    let (oe, tick) = (g.org, g.tick);
    let _ = propose_keyed(&mut g.world, pe, oe, snack, PropKind::BringSnacks, "Agnes", 0.0, tick, AskKey::Coin, 6.0);
    let _ = propose_keyed(&mut g.world, pe, oe, spreader, PropKind::InviteFriends, "Bram", 0.0, tick, AskKey::Coin, 12.0);
    // the bought spreader's dearest friend would surely be pulled, if
    // the word were earned
    let r = g.world.create_entity();
    g.world.add_component(r, Relationship { from: spreader, other: friend,
        kind: RelKind::Friend, strength: 9.0, interactions: 3, ..Default::default() });
    let rf = g.world.create_entity();
    g.world.add_component(rf, Relationship { from: pe, other: friend,
        kind: RelKind::Friend, strength: 3.0, interactions: 5, ..Default::default() });
    g.call_meeting();
    let m = g.meeting.as_ref().expect("a meeting was called");
    // bought folk still fill their slots and walk to the door...
    assert!(m.bought.contains(&snack) && m.bought.contains(&spreader), "{:?}", m.bought);
    // ...but the purchased word pulls nobody in
    assert!(!m.invited.contains(&friend), "a bought spreader pulls no friends: {:?}", m.invited);
    // and the bought role counts nothing toward the kept-promise warmth
    assert!(!m.credit_roles().0, "a bought table reads cheap");
}

fn tie(g: &mut Game, a: Entity, b: Entity, strength: f32) {
    relationships::record_interaction(&mut g.world, a, relationships::Interaction {
        other: b, kind: relationships::InteractionKind::Conversation, intensity: 0.5 });
    relationships::find_relationship(&g.world, a, b)
        .and_then(|r| g.world.get_component_mut::<relationships::Relationship>(r)).unwrap().strength = strength;
}

#[test]
fn test_the_day_pick_asks_what_the_call_will_ask() {
    let mut g = Game::new();
    let (host, bram) = (g.villagers[1], g.villagers[2]);
    plan_ready_meeting(&mut g, host, (17.0, 16.0), &[bram]);
    g.seats[0].meet_offer = Some(g.tick);
    g.decide_at(0, 1);
    assert!(g.seats[0].frame_offer.is_some(), "the day picked, the question's asked");
    let p = g.convo_panel_at(0).expect("the question on the box");
    assert_eq!(p.title, "THE CALL - WHAT WILL YOU ASK");
    assert!(p.decides && p.options.len() == 2);
    g.decide_at(0, 1);
    assert!(g.seats[0].frame_offer.is_none());
    assert!(g.world.get_component::<MeetingPlan>(g.org).unwrap().probe, "the probe is the plan's question");
    assert!(g.ui_model().panel.meeting.unwrap().roles.iter().any(|r| r == "question: what they want"));
}

#[test]
fn test_putting_the_matter_or_letting_it_lapse_keeps_the_matter() {
    let mut g = Game::new();
    let (host, bram) = (g.villagers[1], g.villagers[2]);
    plan_ready_meeting(&mut g, host, (17.0, 16.0), &[bram]);
    g.seats[0].frame_offer = Some(g.tick);
    g.decide_at(0, 0);
    assert!(!g.world.get_component::<MeetingPlan>(g.org).unwrap().probe);
    g.seats[0].frame_offer = Some(g.tick);
    g.livelihood_tick(g.tick + FOUNDING_OFFER_EXPIRY_DAYS);
    assert!(g.seats[0].frame_offer.is_none(), "the question lapses");
    assert!(!g.world.get_component::<MeetingPlan>(g.org).unwrap().probe, "and the matter stands");
}

#[test]
fn test_a_probe_meeting_asks_what_they_want_and_who_for() {
    use crate::game::desires::{InnerLife, Reveal, Want};
    let mut g = Game::new();
    let pe = g.player;
    let (deep, loved, guarded) = (g.villagers[1], g.villagers[2], g.villagers[3]);
    tie(&mut g, pe, deep, 0.8);
    tie(&mut g, deep, loved, 0.95);
    g.world.get_component_mut::<InnerLife>(guarded).unwrap().wants = vec![Want::Music];
    let came = vec![deep, guarded];
    let lines = g.meeting_lines(&came, 0, (false, false, false, false, false), true, None);
    assert!(lines.iter().any(|(e, l)| *e == pe
            && *l == g.pair_said(pe, g.player, crate::game::dialogue_trees::MEET_PROBE.text)),
        "the host asks the probe: {:?}", lines);
    let loved_name = g.name_of(loved);
    assert!(lines.iter().any(|(e, l)| *e == deep && l.contains(&loved_name)),
        "the follower names who they do it for: {:?}", lines);
    assert!(g.log.iter().any(|l| l.contains(&format!("names {}", loved_name))), "the room hears the bond");
    assert!(crate::game::desires::known(&g.world, g.player, guarded).contains(&Reveal::Want(Want::Music)),
        "the guarded one gives up a want");
}

#[test]
fn test_the_lords_ear_takes_the_name_home() {
    use crate::game::desires::{InnerLife, Problem};
    let mut g = Game::new();
    let pe = g.player;
    let (deep, loved, spy) = (g.villagers[1], g.villagers[2], g.villagers[4]);
    tie(&mut g, pe, deep, 0.8);
    tie(&mut g, deep, loved, 0.95);
    g.plant = Some(spy);
    g.meeting_lines(&[deep, spy], 0, (false, false, false, false, false), true, None);
    assert!(g.world.get_component::<InnerLife>(loved).unwrap().grievances.iter()
        .any(|(p, _)| *p == Problem::WritBorne), "a writ finds the one named");
}

#[test]
fn test_the_probe_plants_a_want_that_ripens() {
    use crate::game::desires::{InnerLife, Planted, PLANT_RIPEN_DAYS};
    let mut g = Game::new();
    let pe = g.player;
    let v = g.villagers[3];
    g.world.get_component_mut::<InnerLife>(v).unwrap().wants.clear();
    // known to you: a deed done
    tie(&mut g, pe, v, 0.3);
    let bond = relationships::find_relationship(&g.world, pe, v).unwrap();
    g.world.add_component(bond, crate::game::organizing::Deeds { n: 1 });
    g.meeting_lines(&[v], 0, (false, false, false, false, false), true, None);
    let (w, _, _) = g.world.get_component::<Planted>(v).expect("the question stays with them").wants[0];
    assert!(!g.world.get_component::<InnerLife>(v).unwrap().wants.contains(&w), "only a thought yet");
    let before = g.world.get_component::<relationships::Relationship>(bond).unwrap().strength;
    let t = g.tick + PLANT_RIPEN_DAYS;
    g.livelihood_tick(t);
    assert!(g.world.get_component::<InnerLife>(v).unwrap().wants.contains(&w), "it ripens into a want");
    assert!(g.world.get_component::<relationships::Relationship>(bond).unwrap().strength > before,
        "and the warm hearer comes a step nearer");
}

#[test]
fn test_the_matter_plants_nothing() {
    use crate::game::desires::Planted;
    let mut g = Game::new();
    let v = g.villagers[3];
    g.meeting_lines(&[v], 0, (false, false, false, false, false), false, None);
    assert!(g.world.get_component::<Planted>(v).is_none());
}

#[test]
fn test_a_planted_want_dies_with_its_hearer() {
    use crate::game::desires::{InnerLife, Planted, PLANT_RIPEN_DAYS};
    let mut g = Game::new();
    let v = g.villagers[3];
    g.world.get_component_mut::<InnerLife>(v).unwrap().wants.clear();
    g.meeting_lines(&[v], 0, (false, false, false, false, false), true, None);
    assert!(g.world.get_component::<Planted>(v).is_some());
    g.world.add_component(v, crate::game::villager::Dead);
    let t = g.tick + PLANT_RIPEN_DAYS;
    g.livelihood_tick(t);
    assert!(g.world.get_component::<InnerLife>(v).unwrap().wants.is_empty(), "the dead take no new wants");
}

fn hard_news(g: &mut Game) -> String {
    let line = "the bailiff returns with knights - Bram is hanged for the defiance.";
    g.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Major), format!("t{}: {}", g.tick, line));
    line.to_string()
}

#[test]
fn test_testimony_is_offered_from_real_history() {
    let mut g = Game::new();
    let (host, bram) = (g.villagers[1], g.villagers[2]);
    plan_ready_meeting(&mut g, host, (17.0, 16.0), &[bram]);
    let ev = hard_news(&mut g);
    g.seats[0].frame_offer = Some(g.tick);
    let p = g.convo_panel_at(0).unwrap();
    assert_eq!(p.options.len(), 3, "testify is on offer: {:?}", p.options);
    g.decide_at(0, 2);
    let p = g.convo_panel_at(0).expect("which story to tell");
    assert_eq!(p.title, "THE CALL - WHAT WILL YOU TELL");
    assert_eq!(p.options[0], ev, "the organizer's own history, as it happened");
    g.decide_at(0, 0);
    assert!(g.seats[0].testimony_offer.is_none());
    assert_eq!(g.world.get_component::<MeetingPlan>(g.org).unwrap().testimony.as_deref(), Some(ev.as_str()));
    assert!(g.ui_model().panel.meeting.unwrap().roles.iter().any(|r| r == "opening: your testimony"));
}

#[test]
fn test_testimony_waits_out_its_cooldown() {
    let mut g = Game::new();
    hard_news(&mut g);
    g.seats[0].last_testimony = Some(g.tick);
    g.seats[0].frame_offer = Some(g.tick);
    assert_eq!(g.convo_panel_at(0).unwrap().options.len(), 2, "testimony repeated is a speech");
    g.seats[0].last_testimony = Some(0);
    g.tick += crate::game::organizing::TESTIMONY_COOLDOWN_DAYS;
    assert_eq!(g.convo_panel_at(0).unwrap().options.len(), 3);
}

fn episodes(g: &Game, v: Entity) -> usize {
    g.world.get_component::<crate::sim::psychology::AttachmentHistory>(v).map(|h| h.episodes.len()).unwrap_or(0)
}

#[test]
fn test_testimony_tells_self_us_now_and_lands_on_a_warm_room() {
    let mut g = Game::new();
    let pe = g.player;
    let (a, b) = (g.villagers[1], g.villagers[2]);
    let ev = hard_news(&mut g);
    tie(&mut g, a, pe, 0.6);
    tie(&mut g, b, pe, 0.6);
    let before = (episodes(&g, a), episodes(&g, b));
    let lines = g.meeting_lines(&[a, b], 0, (false, false, false, false, false), false, Some(&ev));
    let host: Vec<&String> = lines.iter().filter(|(e, _)| *e == pe).map(|(_, l)| l).collect();
    assert!(host.iter().any(|l| l.contains(&ev)), "Self tells the event: {:?}", host);
    assert!(host.iter().any(|l| l.starts_with("And it's not mine alone")), "Us");
    assert!(host.iter().any(|l| l.starts_with("So tonight's the choosing")), "Now");
    assert_eq!((episodes(&g, a), episodes(&g, b)), (before.0 + 2, before.1 + 2), "a warm room carries it home");
    assert_eq!(g.seats[0].last_testimony, Some(g.tick));
}

#[test]
fn test_testimony_to_a_cold_room_lands_weaker_but_never_hurts() {
    let mut g = Game::new();
    let pe = g.player;
    let a = g.villagers[3];
    let ev = hard_news(&mut g);
    tie(&mut g, a, pe, 0.0);
    let before = episodes(&g, a);
    g.meeting_lines(&[a], 0, (false, false, false, false, false), false, Some(&ev));
    assert_eq!(episodes(&g, a), before + 1);
    assert!(g.log.last().unwrap().contains("cold room"));
}

/// Testimony opens the room (#1333): the same seeded meeting hears more
/// secrets once a story's told - the rolls are the same, the chance isn't
#[test]
fn test_a_told_testimony_lets_more_of_the_room_speak() {
    let secrets = |testify: bool| -> usize {
        let mut g = Game::new();
        let ev = hard_news(&mut g);
        let came: Vec<Entity> = g.villagers[1..25].to_vec();
        g.meeting_lines(&came, 0, (false, false, false, false, false), false,
            if testify { Some(&ev) } else { None });
        g.voiced.len()
    };
    let (with, without) = (secrets(true), secrets(false));
    assert!(with > without, "testimony should open the room: {with} vs {without} shared");
}

/// And the hearing marks them: a villager who carried the story home
/// answers the probe as deep as what presses them - a problem, not just
/// a want - and their next talk opens a rung warmer (#1333)
#[test]
fn test_a_hearer_answers_deeper_and_opens_warm() {
    use crate::game::desires::{add_grievance, InnerLife, Problem, Reveal};
    use crate::sim::psychology::heard_testimony;
    use crate::session::dialogue::TalkMood;
    let mut g = Game::new();
    let (host, i) = (g.player, 4);
    let v = g.villagers[i];
    // an acquaintance only: the follower tier isn't theirs, and their
    // one unrevealed thing is a grievance, not a want
    tie(&mut g, v, host, 0.0);
    g.world.get_component_mut::<InnerLife>(v).unwrap().wants.clear();
    add_grievance(&mut g.world, v, Problem::Taxed, g.tick + 30);
    assert!(!heard_testimony(&g.world, v, host), "no telling yet");
    let ev = hard_news(&mut g);
    g.meeting_lines(&[v], 0, (false, false, false, false, false), true, Some(&ev));
    assert!(heard_testimony(&g.world, v, host), "the telling rides their journal");
    assert!(crate::game::desires::known(&g.world, host, v)
        .iter().any(|r| matches!(r, Reveal::Problem(_))),
        "a hearer's probe answer gives up what presses");
    // and the next Talk with them opens a rung warmer
    let p = *g.world.get_component::<Pos>(v).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + 0.5, y: p.y };
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    g.act_at(0, 0);
    assert_eq!(g.seats[0].talk.as_ref().map(|t| t.mood), Some(TalkMood::Warmed),
        "they heard the story - the talk opens warm");
}

/// The ask in their own words (#1247): when an ask suits a trouble
/// they've let out, its label quotes it back - and the same ask stays
/// plain on someone who's told you nothing that fits
#[test]
fn test_the_ask_quotes_their_trouble_back() {
    use crate::game::desires::{Problem, Reveal, Want};
    use crate::game::organizing::{ask_label, AskKey, AskOffer, PropKind};
    let mut g = Game::new();
    let (v_taxed, v_plain) = (g.villagers[1], g.villagers[2]);
    let __pe = g.player; for __r in vec![Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, v_taxed, __r); }
    let come = AskOffer { kind: PropKind::ComeAlong, key: AskKey::Earned, price: 0.0 };
    assert!(ask_label(&g.world, g.player, v_taxed, &come).contains("the tolls"),
        "{}", ask_label(&g.world, g.player, v_taxed, &come));
    assert_eq!(ask_label(&g.world, g.player, v_plain, &come), "Ask them to come to a meeting");
    // a non-suiting ask stays plain on the same villager
    let song = AskOffer { kind: PropKind::GiveSong, key: AskKey::Earned, price: 0.0 };
    assert_eq!(ask_label(&g.world, g.player, v_taxed, &song), "Ask them for a song");
    // and the quote answers the ask that suits it, not just any problem
    { let __pe = g.player; crate::game::desires::note(&mut g.world, __pe, v_taxed, Reveal::Want(Want::Love)); }
    let friends = AskOffer { kind: PropKind::InviteFriends, key: AskKey::Earned, price: 0.0 };
    assert!(ask_label(&g.world, g.player, v_taxed, &friends).contains("company"),
        "{}", ask_label(&g.world, g.player, v_taxed, &friends));
}

/// The scouting read (#1249): an acquaintance will tell you who the
/// room follows - the ask yields their picture in fuzzy words, and the
/// ask's spent for the talk once put
#[test]
fn test_the_scout_ask_names_who_the_room_follows() {
    use crate::game::organizing::{ask_offers, pull_of, PropKind, RelStage, stage_of};
    let mut g = Game::new();
    let (pe, talker, bellwether) = (g.player, g.villagers[1], g.villagers[2]);
    // a stranger reads no room for you; an acquaintance does
    assert!(!ask_offers(&g.world, g.org, talker, 0, g.tick).iter().any(|o| o.kind == PropKind::Scout));
    relationships::record_interaction(&mut g.world, pe, relationships::Interaction {
        other: talker, kind: relationships::InteractionKind::Conversation, intensity: 0.5 });
    assert_eq!(stage_of(&g.world, pe, talker), RelStage::Acquaintance);
    assert!(ask_offers(&g.world, g.org, talker, 0, g.tick).iter().any(|o| o.kind == PropKind::Scout));
    // the bellwether holds strong family bonds - the room leans on them
    for kin in g.villagers.iter().copied().skip(3).take(6) {
        let r = g.world.create_entity();
        g.world.add_component(r, Relationship { from: bellwether, other: kin,
            kind: RelKind::Family, strength: 0.9, interactions: 30, ..Default::default() });
    }
    assert!(pull_of(&g.world, bellwether) > pull_of(&g.world, talker).max(1.0),
        "bonds carry the voice: {} vs {}", pull_of(&g.world, bellwether), pull_of(&g.world, talker));
    // open the talk, take the scouting ask: the read names the bellwether
    let i = g.villagers.iter().position(|&x| x == talker).unwrap();
    let p = *g.world.get_component::<Pos>(talker).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + 0.5, y: p.y };
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    g.act_at(0, 0);
    let labels = g.seats[0].talk.as_ref().unwrap().choice_labels(&g.world);
    let pick = labels.iter().position(|l| l.starts_with("Ask who folk turn to"))
        .unwrap_or_else(|| panic!("scout ask on offer: {labels:?}"));
    g.talk_pick_at(0, Some(pick));
    let id = g.seats[0].talk.as_ref().unwrap().id;
    let texts: Vec<String> = g.talk_log.iter().find(|r| r.id == id).unwrap()
        .lines.iter().map(|l| l.text.clone()).collect();
    assert!(texts.iter().any(|t| t.contains("Who do folk turn to")), "{texts:?}");
    assert!(texts.iter().any(|t| t.contains(&g.name_of(bellwether))),
        "names the bellwether: {texts:?}");
    let labels = g.seats[0].talk.as_ref().unwrap().choice_labels(&g.world);
    assert!(!labels.iter().any(|l| l.starts_with("Ask who folk turn to")),
        "spent for the talk: {labels:?}");
}

/// A plan on the organizer with a set day and promises: the inside
/// news the cause's members carry (#1287)
fn plan_news(g: &mut Game) {
    let (host, home) = (g.villagers[0], g.world.create_entity());
    g.world.add_component(g.player, MeetingPlan {
        venue: Some(crate::game::organizing::Venue::House(host, home)), notes: Some(g.villagers[1]),
        for_day: Some(g.tick + 4), ..Default::default() });
}

#[test]
fn test_inside_news_passes_member_to_member() {
    let mut g = Game::new();
    org_ready(&mut g);
    plan_news(&mut g);
    let (a, b) = (g.villagers[0], g.villagers[1]);
    let mut hit = false;
    for seed in 0..20u64 {
        let mut lines = vec![];
        g.rumor_pass(a, b, &mut lines, seed);
        if !lines.is_empty() {
            hit = true;
            assert!(lines.iter().any(|l| l.text.contains("there's a meet at")
                || l.text.contains("promised")), "inside news, told: {:?}", lines);
            break;
        }
    }
    assert!(hit, "a member hands the cause's news to a member");
}

#[test]
fn test_a_friendly_ear_carries_the_news() {
    let mut g = Game::new();
    org_ready(&mut g);
    plan_news(&mut g);
    let (a, b) = (g.villagers[0], g.villagers[9]);
    crate::game::relationships::record_interaction(&mut g.world, a,
        crate::game::relationships::Interaction { other: b,
            kind: crate::game::relationships::InteractionKind::Conversation, intensity: 0.8 });
    let rel = crate::game::relationships::find_relationship(&g.world, a, b).unwrap();
    g.world.get_component_mut::<crate::game::relationships::Relationship>(rel).unwrap().kind = RelKind::Friend;
    let mut hit = false;
    for seed in 0..40u64 {
        let mut lines = vec![];
        g.rumor_pass(a, b, &mut lines, seed);
        if !lines.is_empty() { hit = true; break; }
    }
    assert!(hit, "a friendly ear leans in");
    let r = g.world.get_component::<crate::game::rumor::Rumor>(b)
        .expect("the news they heard is carried");
    assert!(!r.facts.is_empty() && !r.facts[0].wrong_ear,
        "a friendly ear's news isn't marked: {:?}", r.facts);
}

#[test]
fn test_a_wrong_ear_gets_the_news_marked() {
    let mut g = Game::new();
    org_ready(&mut g);
    plan_news(&mut g);
    let (a, b) = (g.villagers[0], g.villagers[9]);
    // no warmth for the teller - and a quarrel with the cause's head:
    // the ears that shouldn't have it
    let pe = g.player;
    crate::game::relationships::record_interaction(&mut g.world, b,
        crate::game::relationships::Interaction { other: pe,
            kind: crate::game::relationships::InteractionKind::Conflict, intensity: 0.8 });
    let rel = crate::game::relationships::find_relationship(&g.world, b, pe).unwrap();
    g.world.get_component_mut::<crate::game::relationships::Relationship>(rel).unwrap().kind = RelKind::Rival;
    let mut hit = false;
    for seed in 0..40u64 {
        let mut lines = vec![];
        g.rumor_pass(a, b, &mut lines, seed);
        if !lines.is_empty() { hit = true; break; }
    }
    assert!(hit, "a wrong ear overhears anyway - the leak's a cost");
    let r = g.world.get_component::<crate::game::rumor::Rumor>(b)
        .expect("even the wrong ear carries what it heard");
    assert!(r.facts.iter().any(|f| f.wrong_ear),
        "the mark the counter-intel seam reads: {:?}", r.facts);
}

#[test]
fn test_a_stranger_hears_no_inside_news() {
    let mut g = Game::new();
    org_ready(&mut g);
    plan_news(&mut g);
    let (a, b) = (g.villagers[0], g.villagers[9]);
    // no bond at all: a stranger doesn't stop for the cause's news
    for seed in 0..40u64 {
        let mut lines = vec![];
        g.rumor_pass(a, b, &mut lines, seed);
        assert!(lines.is_empty(), "a stranger hears nothing: {:?}", lines);
    }
    assert!(g.world.get_component::<crate::game::rumor::Rumor>(b).is_none());
}

/// A ghost-carrier hangs back when only the bare word calls (E3-4,
/// #1425): the levy's wound halves their showing - and when they say
/// no, they say it in the wound's own words
#[test]
fn test_a_ghost_hangs_back_from_the_bare_word() {
    use crate::game::organizing::{no_reason, shows_up, Invite};
    use crate::game::persona::*;
    let mut g = Game::new();
    // a contented guest whose roll floats near the drift-in edge
    let pe = g.player; let v = g.villagers.iter().copied().find(|&v| {
        if let Some(l) = g.world.get_component_mut::<crate::game::desires::InnerLife>(v) { l.wants.clear(); }
        desires::unrevealed(&g.world, pe, v, g.tick).is_empty()
    }).expect("someone content enough to have nothing to say");
    let at = {
        let p = *g.world.get_component::<Pos>(v).unwrap();
        (p.x as i32, p.y as i32)
    };
    // the drift-in edge: this roll passes while unhurt
    assert!(shows_up(&g.world, g.player, v, at, Invite::Word, 0.05, g.tick),
        "the bare roll should drift them in");
    let mut p = Persona {
        seed: 1, sex: villager::Sex::Male, archetype: Archetype::OldGuard,
        temperament: [0.5; 5], trust_propensity: 0.5, values: [0.5; 10],
        story: vec![], ghosts: vec![], signature: Signature::Habit("x"),
        hidden: Landmark::Skill("y"), contradiction: None,
        idiolect: crate::game::idiolect::Idiolect {
            phrase: "x", habit: crate::game::idiolect::Habit::Echo },
    };
    p.ghosts.push(Ghost { event: EventId(0), kind: EventKind::Levy, hook: None,
        lie: Belief::NeverCrossReeve, strength: 0.8, fear: Fear::BailiffKnowsMyFace,
        need: Need::ToBeSafe, turned: false });
    g.world.add_component(v, p);
    // the same roll now keeps them home (0.55 levy x 0.45 reeve lie)
    assert!(!shows_up(&g.world, g.player, v, at, Invite::Word, 0.10, g.tick),
        "the wound hangs them back");
    assert!(shows_up(&g.world, g.player, v, at, Invite::Word, 0.01, g.tick),
        "but a low roll still finds them");
    // and a no lands in the wound's own words, not a trait's stock excuse
    assert_eq!(no_reason(&g.world, g.player, v, crate::game::organizing::PropKind::ComeAlong, g.tick),
        Some("You'll not see me stand under that gate. Ask another."));
    // a warm ghost cuts the other way: the feast-keeper comes readier
    let pe = g.player; let w = g.villagers.iter().copied().find(|&w| {
        if let Some(l) = g.world.get_component_mut::<crate::game::desires::InnerLife>(w) { l.wants.clear(); }
        desires::unrevealed(&g.world, pe, w, g.tick).is_empty()
    }).expect("another contented one");
    let at2 = {
        let p = *g.world.get_component::<Pos>(w).unwrap();
        (p.x as i32, p.y as i32)
    };
    assert!(!shows_up(&g.world, g.player, w, at2, Invite::Word, 0.14, g.tick),
        "the bare drift-in stops short here");
    let mut p2 = Persona {
        seed: 2, sex: villager::Sex::Female, archetype: Archetype::OldGuard,
        temperament: [0.5; 5], trust_propensity: 0.5, values: [0.5; 10],
        story: vec![], ghosts: vec![], signature: Signature::Habit("x"),
        hidden: Landmark::Skill("y"), contradiction: None,
        idiolect: crate::game::idiolect::Idiolect {
            phrase: "x", habit: crate::game::idiolect::Habit::Echo },
    };
    p2.ghosts.push(Ghost { event: EventId(0), kind: EventKind::HarvestFeast, hook: None,
        lie: Belief::StrongTogether, strength: 0.8, fear: Fear::AloneInIt,
        need: Need::ToBelong, turned: false });
    g.world.add_component(w, p2);
    assert!(shows_up(&g.world, g.player, w, at2, Invite::Word, 0.05, g.tick),
        "the feast-keeper's year draws them out (1.15 x 1.15)");
}

/// Make `v` a follower of the player's org who's let one trouble out -
/// the earned asks are on the table
fn follower_with_a_grievance(g: &mut Game, v: Entity) {
    use crate::game::desires::{Problem, Reveal};
    let pe = g.player;
    relationships::record_interaction(&mut g.world, pe, relationships::Interaction {
        other: v, kind: relationships::InteractionKind::Conversation, intensity: 0.5 });
    crate::game::relationships::find_relationship(&g.world, g.player, v)
        .and_then(|r| g.world.get_component_mut::<Relationship>(r)).unwrap().strength = 0.7;
    let __pe = g.player; for __r in vec![Reveal::Problem(Problem::Taxed)] { crate::game::desires::note(&mut g.world, __pe, v, __r); }
}

/// Talk to villager `vi` and keep offering the come-along until a yes
/// lands the invitation's follow-up - a refusal's just asked again on
/// the next walk over
fn invite_until_asking(g: &mut Game, vi: usize) {
    use crate::game::organizing::PropKind;
    let who = g.villagers[vi];
    let p = *g.world.get_component::<Pos>(who).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + 0.5, y: p.y };
    for _ in 0..80 {
        if g.seats[0].talk.is_none() {
            g.seats[0].selected_target = vi;
            g.seats[0].picked = true;
            g.act_at(0, 0);
        }
        let Some(t) = g.seats[0].talk.as_ref() else { return };
        if t.inviting.is_some() { return; }
        let (n, shown) = (t.options.len(), t.asks_shown());
        match t.asks.iter().take(shown).position(|a| a.kind == PropKind::ComeAlong) {
            Some(k) => g.talk_pick_at(0, Some(n + k)),
            None => g.talk_pick_at(0, Some(if shown > 0 { n } else { 0 })),
        }
    }
}

/// The first "come to a meeting" that lands asks the call's own
/// questions back - when, then where - and the picks put the word
/// out themselves (#1337)
#[test]
fn test_an_invite_asks_when_and_where() {
    use crate::game::organizing::Venue;
    use crate::session::dialogue::InviteStep;
    let mut g = Game::new();
    let v = g.villagers[1];
    follower_with_a_grievance(&mut g, v);
    invite_until_asking(&mut g, 1);
    let t = g.seats[0].talk.as_ref().expect("the talk's still open");
    assert!(matches!(t.inviting, Some(InviteStep::When)),
        "the yes asks the when back: {:?}", t.inviting);
    // the picks are the call's own days, not the talk's replies
    let labels = t.choice_labels(&g.world);
    assert_eq!(labels.len(), 3, "{labels:?}");
    assert!(labels[2].contains("week"), "{labels:?}");
    g.talk_pick_at(0, Some(0)); // two days out
    let t = g.seats[0].talk.as_ref().unwrap();
    let (days, _) = match &t.inviting {
        Some(InviteStep::Where { days, venues }) => (*days, venues),
        s => panic!("the where follows the when: {s:?}"),
    };
    assert_eq!(days, 2);
    // the where's the public spots and "their call", last
    let labels = t.choice_labels(&g.world);
    assert_eq!(labels[0], "the square", "{labels:?}");
    assert!(labels.iter().any(|l| l == "the tavern"), "{labels:?}");
    assert!(labels.last().unwrap().contains("Their call"), "{labels:?}");
    g.talk_pick_at(0, Some(0)); // the square
    let plan = g.world.get_component::<MeetingPlan>(g.org).expect("the plan");
    assert_eq!(plan.for_day, Some(g.tick + 2), "the word's out two days on");
    assert_eq!(plan.venue, Some(Venue::Square));
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(t.inviting.is_none(), "the follow-up's done");
    // and the word's out: the day fires the call on its own, no panel
    g.livelihood_tick(g.tick + 2);
    assert!(g.meeting.is_some(), "the day fires the call: {:?}", g.log.last());
    assert!(g.log.iter().any(|l| l.contains("the word's out")), "the word went out");
}

/// A lent house stays learned past the call it was lent for (#1337):
/// the where-pick can name it again the next time a meet's uncalled
#[test]
fn test_a_lent_house_stays_learned() {
    use crate::game::organizing::{propose, KnownVenues, PropKind, Venue};
    let mut g = Game::new();
    let (pe, host, guest) = (g.player, g.villagers[1], g.villagers[2]);
    let home = crate::game::household::home_of(&g.world, host)
        .expect("the host has a house to lend");
    let (oe, tick) = (g.org, g.tick);
    let (msg, _) = propose(&mut g.world, pe, oe, host, PropKind::LendVenue, "Eliza", 0.0, tick);
    assert!(msg.contains("house"), "{msg}");
    assert!(g.world.get_component::<KnownVenues>(oe)
        .map(|k| k.homes.contains(&(host, home))).unwrap_or(false), "the door's learned");
    // the call spends the plan: the learning's what stays
    g.world.add_component(oe, MeetingPlan::default());
    follower_with_a_grievance(&mut g, guest);
    invite_until_asking(&mut g, 2);
    g.talk_pick_at(0, Some(0)); // two days out
    let labels = g.seats[0].talk.as_ref().unwrap().choice_labels(&g.world);
    let door = format!("{}'s house", g.cast_name(1));
    assert!(labels.iter().any(|l| *l == door),
        "the learned door's a pick again: {labels:?}");
    let pick = labels.iter().position(|l| *l == door).unwrap();
    g.talk_pick_at(0, Some(pick));
    let plan = g.world.get_component::<MeetingPlan>(g.org).unwrap();
    assert_eq!(plan.venue, Some(Venue::House(host, home)));
}

/// "Their call" names their own door when they have one (#1337): the
/// invitee lends the house with the answer, and the org learns it
#[test]
fn test_their_call_names_their_house() {
    use crate::game::organizing::{KnownVenues, Venue};
    let mut g = Game::new();
    let v = g.villagers[1];
    let home = crate::game::household::home_of(&g.world, v)
        .expect("they've a house to offer");
    follower_with_a_grievance(&mut g, v);
    invite_until_asking(&mut g, 1);
    g.talk_pick_at(0, Some(0)); // two days out
    let labels = g.seats[0].talk.as_ref().unwrap().choice_labels(&g.world);
    let last = labels.len() - 1;
    assert!(labels[last].contains("Their call"), "{labels:?}");
    g.talk_pick_at(0, Some(last));
    let plan = g.world.get_component::<MeetingPlan>(g.org).unwrap();
    assert_eq!(plan.venue, Some(Venue::House(v, home)), "their door's the venue");
    assert!(g.world.get_component::<KnownVenues>(g.org)
        .map(|k| k.homes.contains(&(v, home))).unwrap_or(false), "and it's learned");
}


/// The E8-1 done-when (#1442): a villager's realized yes-rate across
/// repeated asks lands on the act model's call - and the call itself
/// climbs as the ledger fills on them. Knowing the trouble that suits
/// the ask sits it cheaper; the agitation work pushes it further. At
/// both ends of knowing them the rolls and the read agree.
#[test]
fn test_the_models_calls_track_the_ledger() {
    use crate::game::organizing::{yes_chance, PropKind};
    let mut g = Game::new();
    let pe = g.player;
    let v = g.villagers[2];
    // a middling bond: enough that the ask isn't a stranger's, not so
    // much the read's already yes
    relationships::record_interaction(&mut g.world, pe, relationships::Interaction {
        other: v, kind: relationships::InteractionKind::Conversation, intensity: 0.5 });
    relationships::find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component_mut::<Relationship>(r)).unwrap().strength = 0.5;
    // nothing of theirs on the ledger yet
    let p0 = yes_chance(&g.world, pe, v, PropKind::ComeAlong, g.tick);
    // hear the trouble the come-along speaks to - the ledger fills
    desires::note(&mut g.world, pe, v, desires::Reveal::Problem(desires::Problem::Taxed));
    let p1 = yes_chance(&g.world, pe, v, PropKind::ComeAlong, g.tick);
    assert!(p1 > p0, "the suited ask sits lighter once it's heard: {p0:.2} -> {p1:.2}");
    // and the campaign's own pushes move it further still
    for _ in 0..3 { crate::game::act_model::argued(&mut g.world, v, true); }
    let p2 = yes_chance(&g.world, pe, v, PropKind::ComeAlong, g.tick);
    assert!(p2 > p1, "agitation moves the read: {p1:.2} -> {p2:.2}");
    // the realized yes-rate across rolls lands on each call
    for (i, p) in [p0, p1, p2].iter().enumerate() {
        let mut rng = Rng(0xE81 + i as u64);
        let yes = (0..400).filter(|_| p > &((rng.next() % 1000) as f32 / 1000.0)).count() as f32 / 400.0;
        assert!((yes - p).abs() < 0.1, "realized {yes:.2} vs model {p:.2}");
    }
}

/// E8-2 (#1444) done-when: a campaign outcome the lie listens to moves
/// the ghost's strength; under the floor the ghost turns once, the core
/// memory files toward the player, and the villager says so out loud
#[test]
fn a_lie_weakens_after_a_campaign_win_and_the_villager_says_so() {
    use crate::game::beliefs::{tested_by, Tested, TURN_FLOOR};
    use crate::game::persona::*;
    let mut g = Game::new();
    let v = g.villagers[0];
    let mut p = Persona {
        seed: 3, sex: villager::Sex::Male, archetype: Archetype::OldGuard,
        temperament: [0.5; 5], trust_propensity: 0.5, values: [0.5; 10],
        story: vec![], ghosts: vec![], signature: Signature::Habit("x"),
        hidden: Landmark::Skill("y"), contradiction: None,
        idiolect: crate::game::idiolect::Idiolect {
            phrase: "x", habit: crate::game::idiolect::Habit::Echo },
    };
    p.ghosts.push(Ghost { event: EventId(0), kind: EventKind::Levy, hook: None,
        lie: Belief::NeverCrossReeve, strength: 0.5, fear: Fear::BailiffKnowsMyFace,
        need: Need::ToBeSafe, turned: false });
    g.world.add_component(v, p);
    // the first writ sent home empty weakens the rule; it still holds
    let (pe, tick) = (g.player, g.tick);
    assert_eq!(tested_by(&mut g.world, v, Tested::WritDefied, Some(pe), tick), None);
    let s = g.world.get_component::<Persona>(v).unwrap().ghosts[0].strength;
    assert!((s - 0.34).abs() < 0.001, "s={s}");
    // the second breaks it: the ghost turns and the words reach the log
    g.test_lies(&[v], Tested::WritDefied, Some(pe), tick);
    let ghost = &g.world.get_component::<Persona>(v).unwrap().ghosts[0];
    assert!(ghost.turned && ghost.strength < TURN_FLOOR);
    assert!(g.log.iter().any(|l| l.contains("the sky held")),
        "the villager says so: {:?}", g.log.last());
    // and the turn sits in their memory of the organizer
    let mem = g.world.get_component::<crate::game::episodes::PairMemory>(v).unwrap();
    assert!(mem.eps.iter().any(|e| e.kind == crate::game::episodes::MemKind::Turned
        && e.concepts.contains(&"never_cross_reeve")));
}

/// The kept promise credits on arrival against the snapshot the call
/// took - the live plan was already spent by then (#1444)
#[test]
fn a_keeper_who_walks_in_tests_their_own_keeping() {
    use crate::game::persona::*;
    let mut g = Game::new();
    let (host, keeper) = (g.villagers[1], g.villagers[2]);
    let mut p = Persona {
        seed: 4, sex: villager::Sex::Female, archetype: Archetype::OldGuard,
        temperament: [0.5; 5], trust_propensity: 0.5, values: [0.5; 10],
        story: vec![], ghosts: vec![], signature: Signature::Habit("x"),
        hidden: Landmark::Skill("y"), contradiction: None,
        idiolect: crate::game::idiolect::Idiolect {
            phrase: "x", habit: crate::game::idiolect::Habit::Echo },
    };
    p.ghosts.push(Ghost { event: EventId(0), kind: EventKind::Levy, hook: None,
        lie: Belief::PromisesForKeepers, strength: 0.3, fear: Fear::BailiffKnowsMyFace,
        need: Need::ToBeSafe, turned: false });
    g.world.add_component(keeper, p);
    *g.world.get_component_mut::<Pos>(keeper).unwrap() = Pos { x: 17.0, y: 16.0 };
    plan_ready_meeting(&mut g, host, (17.0, 16.0), &[keeper]);
    let msg = g.call_meeting_at(0);
    assert!(msg.contains("gather"), "{msg}");
    // the call snapshotted the promise-holders; the live plan's spent
    assert!(g.meeting.as_ref().unwrap().promised.contains(&keeper));
    assert!(g.world.get_component::<MeetingPlan>(g.org)
        .map(|p| !p.ready()).unwrap_or(true), "the call consumed the plan");
    finish_meeting(&mut g);
    // arrival tested the lie: under the floor it turns, and the
    // efficacy credit the act model reads landed on the showing
    let g0 = g.world.get_component::<Persona>(keeper).unwrap().ghosts[0];
    assert!(g0.turned && g0.strength < crate::game::beliefs::TURN_FLOOR,
        "the kept promise broke the rule: {g0:?}");
    assert!(g.world.get_component::<crate::game::act_model::Pressure>(keeper)
        .map(|p| p.efficacy > 0.0).unwrap_or(false),
        "the keeping itself credits the keeper");
}

/// E8-4 (#1454) done-when: cause-work that disproves a member's own
/// lie strains their belief; past the floor the softer side yields -
/// they drift out of the cause and the log says why
#[test]
fn work_that_disproves_your_own_lesson_pulls_you_out_of_the_cause() {
    use crate::game::persona::*;
    use crate::game::tension::{self, Axis, FLIP_AT};
    use crate::game::beliefs::Tested;
    let mut g = Game::new();
    let v = g.villagers[0];
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org)
        .unwrap().members = vec![v];
    let mut p = Persona {
        seed: 5, sex: villager::Sex::Male, archetype: Archetype::Firebrand,
        temperament: [0.5; 5], trust_propensity: 0.5, values: [0.5; 10],
        story: vec![], ghosts: vec![], signature: Signature::Habit("x"),
        hidden: Landmark::Skill("y"), contradiction: None,
        idiolect: crate::game::idiolect::Idiolect {
            phrase: "x", habit: crate::game::idiolect::Habit::Echo },
    };
    p.ghosts.push(Ghost { event: EventId(0), kind: EventKind::Levy, hook: None,
        lie: Belief::NeverCrossReeve, strength: 0.9, fear: Fear::BailiffKnowsMyFace,
        need: Need::ToBeSafe, turned: false });
    g.world.add_component(v, p);
    // a dissonant streak already wound near its floor: one more writ
    // sent home empty tips it
    tension::strained(&mut g.world, v, Axis::Belief, FLIP_AT - 0.04);
    let tick = g.tick;
    g.test_lies(&[v], Tested::WritDefied, None, tick);
    // the writ's failure reached their own lie through their own work -
    // the strain tipped, and the softer side yielded
    let o = g.world.get_component::<crate::sim::community::Organization>(g.org).unwrap();
    assert!(!o.members.contains(&v), "they drift out of the cause");
    assert_eq!(tension::strain_of(&g.world, v, Axis::Belief), 0.0,
        "the spent pull rests");
    assert!(g.log.iter().any(|l| l.contains("drifts out of the cause")
        && l.contains("the old lesson held")), "{:?}", g.log.last());
}

/// E8-4 (#1454): the edge-watcher's strain resolves the other way -
/// past the floor they come in off the edge and stand with the cause
#[test]
fn a_watcher_whose_issue_the_cause_lifts_comes_in() {
    use crate::game::tension::{self, Axis, FLIP_AT};
    let mut g = Game::new();
    let w = g.villagers[0];
    tension::strained(&mut g.world, w, Axis::Issue, FLIP_AT);
    let tick = g.tick;
    g.test_lies(&[w], crate::game::beliefs::Tested::WritDefied, None, tick);
    let o = g.world.get_component::<crate::sim::community::Organization>(g.org).unwrap();
    assert!(o.members.contains(&w), "the lifted issue drew them in");
    assert!(g.log.iter().any(|l| l.contains("stands with the cause")),
        "{:?}", g.log.last());
}

/// E8-4 (#1454): the org's ask over a bare house strains interest at
/// the meeting itself - past the floor, arrival is when they drift out
#[test]
fn the_orgs_ask_over_a_bare_house_calls_them_home() {
    use crate::game::tension::{self, Axis, FLIP_AT};
    use crate::game::desires::{self, Problem};
    let mut g = Game::new();
    let (host, member) = (g.villagers[1], g.villagers[2]);
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org)
        .unwrap().members = vec![host, member];
    // the bench's ask fights a bare shop
    g.world.get_component_mut::<crate::game::vocation::Craft>(member)
        .unwrap().workplace = None;
    assert!(desires::problems_of(&g.world, member, g.tick).iter()
        .any(|p| matches!(p, Problem::NoWorkplace | Problem::NoHome | Problem::Broke)),
        "the house must be bare for the strain to bite");
    tension::strained(&mut g.world, member, Axis::Interest, FLIP_AT - 0.02);
    // warm to the player: without the walked-out exclusion the resolve's
    // join pass would seat them right back
    let rel = g.world.create_entity();
    g.world.add_component(rel, Relationship { from: g.player, other: member,
        kind: RelKind::Friend, strength: 0.7, interactions: 6, ..Default::default() });
    *g.world.get_component_mut::<Pos>(member).unwrap() = Pos { x: 17.0, y: 16.0 };
    plan_ready_meeting(&mut g, host, (17.0, 16.0), &[member]);
    let msg = g.call_meeting_at(0);
    assert!(msg.contains("gather"), "{msg}");
    finish_meeting(&mut g);
    let o = g.world.get_component::<crate::sim::community::Organization>(g.org).unwrap();
    assert!(!o.members.contains(&member),
        "arriving tipped the strain - they drift out and stay out");
    assert!(g.log.iter().any(|l| l.contains("the belly called them home")),
        "{:?}", g.log.last());
}

/// E8-4 (#1454): a delegated seat doesn't outlive the membership it
/// stood on - the flip takes the role with it
#[test]
fn a_flip_takes_their_delegated_seat_with_it() {
    use crate::game::tension::{self, Axis, FLIP_AT};
    let mut g = Game::new();
    let (v, pe, tick) = (g.villagers[0], g.player, g.tick);
    for &w in &g.villagers[..3] {
        crate::game::desires::add_grievance(&mut g.world, w,
            crate::game::desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org)
        .unwrap().members = g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    assert!(crate::game::campaign::delegate_role(&mut g.world, c,
        crate::game::campaign::Role::NoteTaker, v));
    tension::strained(&mut g.world, v, Axis::Interest, FLIP_AT);
    g.test_lies(&[v], crate::game::beliefs::Tested::WritDefied, Some(pe), tick);
    let o = g.world.get_component::<crate::sim::community::Organization>(g.org).unwrap();
    assert!(!o.members.contains(&v), "they drift out");
    assert!(g.world.get_component::<crate::game::campaign::Campaign>(c).unwrap()
        .seats.iter().all(|s| s.holder != v), "the seat goes with the leaving");
}

/// E8-4 (#1454): tension answers to the acting seat's cause - a second
/// seat's event strains its own org, never the first's
#[test]
fn the_strain_lands_on_the_acting_seats_cause() {
    use crate::game::tension::{self, Axis, FLIP_AT};
    let mut g = Game::new();
    let seat1 = g.add_seat();
    let (o1, p1) = (g.seats[seat1].org, g.seats[seat1].player);
    let v = g.villagers[0];
    // on both rolls: only the acting seat's loses them
    g.world.get_component_mut::<crate::sim::community::Organization>(o1)
        .unwrap().members.push(v);
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org)
        .unwrap().members.push(v);
    tension::strained(&mut g.world, v, Axis::Interest, FLIP_AT);
    let tick = g.tick;
    g.test_lies(&[v], crate::game::beliefs::Tested::WritDefied, Some(p1), tick);
    assert!(!g.world.get_component::<Organization>(o1).unwrap().members.contains(&v),
        "the acting seat's cause loses them");
    assert!(g.world.get_component::<crate::sim::community::Organization>(g.org)
        .unwrap().members.contains(&v), "the first seat's roll is untouched");
}
