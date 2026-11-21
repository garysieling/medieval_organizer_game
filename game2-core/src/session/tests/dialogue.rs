//! Conversations: the rules cascade in play, the talk log, and the TALK and conversation panels.
use crate::session::*;
use crate::session::dialogue::*;
use crate::engine::ecs::Entity;
use super::fixtures::plan_ready_meeting;
use crate::game::conversation::{Act, Relation};
use crate::game::relationships::RelKind;
use crate::game::traits::Personality;
use crate::game::villager::{self, Hunger};
use crate::game::vocation::{Kid, Vocation};
use crate::game::traits::Trait;
use crate::game::villager::Pos;

/// Living grown-up villager indices
fn adults(g: &Game) -> Vec<usize> {
    (0..g.villagers.len()).filter(|&i| g.world.get_component::<Kid>(g.villagers[i]).is_none()
        && !villager::is_dead(&g.world, g.villagers[i])).collect()
}

#[test]
fn test_speaker_reads_traits_trade_and_circumstance_off_the_world() {
    let mut g = Game::new();
    let v = g.villagers[adults(&g)[0]];
    g.world.add_component(v, Personality { traits: vec![Trait::Hotheaded] });
    g.world.add_component(v, crate::game::vocation::craft(Vocation::Weaver, 0.5));
    g.world.add_component(v, Hunger { value: 5.0 });
    let s = speaker_of(&g.world, v, "Eliza", g.tick);
    assert_eq!(s.traits, vec![Trait::Hotheaded]);
    assert!(s.status.contains(&"weaver") && s.status.contains(&"hungry"), "{:?}", s.status);
    g.world.add_component(v, crate::game::vocation::craft(Vocation::Priest, 0.5));
    assert_eq!(speaker_of(&g.world, v, "Eliza", g.tick).ideology, "clergy");
    g.world.add_component(v, Kid);
    assert_eq!(speaker_of(&g.world, v, "Eliza", g.tick).status, vec!["child"]);
}

#[test]
fn test_neighbours_are_acquaintances_but_the_wanderer_is_a_stranger() {
    assert_eq!(relation_of(Some(RelKind::Family)), Relation::Kin);
    assert_eq!(relation_of(Some(RelKind::Rival)), Relation::Rival);
    assert_eq!(relation_of(Some(RelKind::Friend)), Relation::Friend);
    assert_eq!(relation_of(None), Relation::Acquaintance);
    assert_eq!(relation_of(Some(RelKind::Stranger)), Relation::Acquaintance);
    assert_eq!(relation_with_wanderer(None), Relation::Stranger);
}

#[test]
fn test_villagers_chat_goes_on_the_record_not_over_the_map() {
    let mut g = Game::new();
    let ad = adults(&g);
    let (a, b) = (g.villagers[ad[0]], g.villagers[ad[1]]);
    g.bubbles.clear();
    assert!(g.villagers_chat(a, b));
    assert!(!g.bubbles.iter().any(|x| x.0 == a || x.0 == b), "no bubbles for villager chatter");
    let r = g.talk_log.last().unwrap().clone();
    assert_eq!(r.kind, TalkKind::Chat);
    assert!(r.lines.len() >= 2 && r.lines.iter().any(|l| l.who == Some(b)), "both speak: {:?}", r.lines);
    // clicking either of them shows it
    g.seats[0].selected_target = ad[1];
    g.seats[0].picked = true;
    let p = g.talk_panel_at(0);
    assert!(p.title.contains(&g.name_of(b)), "{}", p.title);
    assert!(p.rows.iter().any(|row| row.text == r.lines[0].text));
}

#[test]
fn test_kids_keep_to_childrens_talk() {
    let mut g = Game::new();
    let ad = adults(&g);
    let (a, b, c) = (g.villagers[ad[0]], g.villagers[ad[1]], g.villagers[ad[2]]);
    g.world.add_component(a, Kid);
    g.world.add_component(b, Kid);
    assert!(!g.villagers_chat(a, c), "a child and a grown-up just nod");
    assert!(g.villagers_chat(a, b));
    let said: Vec<String> = g.talk_log.last().unwrap().lines.iter().map(|l| l.text.clone()).collect();
    let kids_lines: Vec<&str> = ["Oi, ", "Hast thou any errands", "Psst"].to_vec();
    assert!(kids_lines.iter().any(|k| said[0].contains(k)), "opens on the children's tree: {:?}", said);
}

/// Stand the player beside villager index `i` and Talk
fn talk_beside(g: &mut Game, i: usize) -> Entity {
    let v = g.villagers[i];
    let p = *g.world.get_component::<Pos>(v).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + 0.5, y: p.y };
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    g.act_at(0, 0);
    v
}

#[test]
fn test_talk_opens_a_conversation_on_the_talk_panel_with_replies() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let log_before = g.log.len();
    let v = talk_beside(&mut g, i);
    let rec = g.talk_log.last().unwrap().clone();
    assert_eq!(rec.kind, TalkKind::Talk);
    assert!(rec.lines.iter().any(|l| l.gist && l.who == Some(v)), "what's on their mind is on the record");
    assert!(!g.log[log_before..].iter().any(|l| rec.lines.iter().any(|r| r.gist && l.contains(&r.text))),
        "the words aren't in the event log");
    let t = g.seats[0].talk.as_ref().expect("a talk under way").clone();
    assert!(!t.options.is_empty() && t.options.len() <= 2);
    let panel = g.talk_panel_at(0);
    assert!(panel.in_talk && panel.options.len() == t.options.len() + t.asks_shown());
    // pick the first reply: it's said, as previewed
    g.talk_pick_at(0, Some(0));
    let rec = g.talk_log.iter().find(|r| r.id == rec.id).unwrap();
    assert!(rec.lines.iter().any(|l| l.who == Some(g.player) && l.text == t.options[0].1), "{:?}", rec.lines);
}

#[test]
fn test_the_choice_list_stops_at_three() {
    use crate::game::desires::{Known, Problem, Reveal};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    // they've told you something of themselves, and the bond's a
    // follower's - the asks open up
    g.world.add_component(v, Known { revealed: vec![Reveal::Problem(Problem::Taxed)] });
    let pe = g.player;
    crate::game::relationships::record_interaction(&mut g.world, pe,
        crate::game::relationships::Interaction { other: v,
            kind: crate::game::relationships::InteractionKind::Conversation, intensity: 0.3 });
    crate::game::relationships::find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component_mut::<crate::game::relationships::Relationship>(r))
        .unwrap().strength = 0.6;
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap().clone();
    assert!(t.asks.len() > 1, "asks on offer: {:?}", t.asks);
    // replies plus asks, but never more than three on offer at once
    let convo = g.convo_panel_at(0).unwrap();
    assert_eq!(convo.options.len(), t.options.len() + t.asks_shown());
    assert!(convo.options.len() <= TALK_MAX_CHOICES, "{:?}", convo.options);
    let panel = g.talk_panel_at(0);
    assert_eq!(panel.options.len(), convo.options.len());
    // a pick past the shown three does nothing; the first ask still lands
    g.talk_pick_at(0, Some(t.options.len() + t.asks_shown()));
    assert!(g.seats[0].talk.is_some(), "an unseen ask can't be picked");
    let ask = t.asks[0];
    g.talk_pick_at(0, Some(t.options.len()));
    assert!(!g.seats[0].talk.as_ref().unwrap().asks.contains(&ask), "the ask was spent");
}

#[test]
fn test_leaving_or_walking_off_ends_the_talk() {
    let mut g = Game::new();
    let ad = adults(&g);
    talk_beside(&mut g, ad[0]);
    assert!(g.seats[0].talk.is_some());
    g.talk_pick_at(0, None);
    assert!(g.seats[0].talk.is_none());
    assert_eq!(g.talk_log.last().unwrap().lines.last().unwrap().text, "You take your leave.");
    let v = talk_beside(&mut g, ad[1]);
    assert!(g.seats[0].talk.is_some());
    let p = *g.world.get_component::<Pos>(v).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + PLAYER_TALK_RANGE + 2.0, y: p.y };
    g.advance_player_talks();
    assert!(g.seats[0].talk.is_none(), "walking away ends it");
}

#[test]
fn test_a_whole_talk_runs_to_its_end_by_picking_replies() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    let mut picks = 0;
    while g.seats[0].talk.is_some() && picks < 30 { g.talk_pick_at(0, Some(0)); picks += 1; }
    assert!(g.seats[0].talk.is_none(), "it ends: farewell, walk-off or out of things to say");
    assert!(picks >= 1);
}

#[test]
fn test_a_spent_talk_closes_itself_on_a_mutual_parting() {
    // #1179: when the material's spent the talk ends itself - a mutual
    // parting, not an empty option list to hover in
    let mut g = Game::new();
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    let id = g.seats[0].talk.as_ref().unwrap().id;
    let mut picks = 0;
    // drive it dry: farewells never sit beside live replies now, so pick 0
    // keeps talking until the talk itself decides it's done
    while g.seats[0].talk.is_some() && picks < 30 {
        if let Some(t) = &g.seats[0].talk {
            let farewells = t.options.iter().filter(|(a, _)| *a == Act::Farewell).count();
            assert!(farewells == 0 || farewells == t.options.len(),
                "farewells shared a list with live replies: {:?}", t.options);
        }
        g.talk_pick_at(0, Some(0));
        picks += 1;
    }
    assert!(g.seats[0].talk.is_none(), "a spent talk doesn't hover");
    let rec = g.talk_log.iter().find(|r| r.id == id).unwrap();
    assert!(rec.lines.iter().any(|l| l.text.contains("part") || l.text.contains("leave")
        || l.text.contains("walks off") || l.text.contains("in peace")),
        "the parting's on the record: {:?}", rec.lines.last().map(|l| l.text.clone()));
}

#[test]
fn test_a_house_meeting_is_kept_on_the_talk_panel() {
    let mut g = Game::new();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: 16.0, y: 16.0 };
    let (eliza, bram) = (g.villagers[1], g.villagers[2]);
    for (v, x, y) in [(eliza, 18.0, 16.0), (bram, 15.0, 19.0)] { *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x, y }; }
    for (i, v) in g.villagers.clone().into_iter().enumerate() {
        if i != 1 && i != 2 { *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 3.0 + i as f32, y: 3.0 }; }
    }
    plan_ready_meeting(&mut g, eliza, (18.0, 16.0), &[bram]);
    g.call_meeting();
    for _ in 0..crate::session::FPS * 60 { if g.meeting.is_none() { break; } g.update(); }
    assert!(g.meeting.is_none(), "the meeting ran its course");
    let r = g.talk_log.iter().rev().find(|r| r.kind == TalkKind::Meeting).expect("a meeting on the record");
    assert!(r.with.contains(&g.player));
    assert!(r.lines.iter().any(|l| l.who == Some(g.player)) && r.lines.iter().any(|l| l.who == Some(eliza) || l.who == Some(bram)),
        "{:?}", r.lines);
    // the room's talk, not just the stock remarks
    assert!(r.lines.len() >= 5);
}

#[test]
fn test_each_seat_sees_its_own_lines_as_you_and_hosts_its_own_meeting() {
    let mut g = Game::new();
    let s1 = g.add_seat();
    let (p0, p1) = (g.seats[0].player, g.seats[s1].player);
    let v = g.villagers[adults(&g)[0]];
    // seat 1 hosts: the prompts are theirs, not seat 0's
    let lines = g.meeting_lines(&[v], s1, (false, false, false, false, false));
    assert!(lines.iter().any(|l| l.0 == p1) && !lines.iter().any(|l| l.0 == p0), "seat 1 asks the questions");
    let n1 = g.speaker_name(p1);
    g.record_meeting(vec![p1, p0, v], vec![(Some(p1), n1.clone(), "Thanks for coming.".into())]);
    let row = |g: &Game, seat: usize| g.talk_panel_at(seat).rows.into_iter()
        .find(|r| r.text == "Thanks for coming.").and_then(|r| r.name).unwrap();
    assert_eq!(row(&g, s1), "You", "the host's own panel");
    assert_eq!(row(&g, 0), n1, "the other seat sees who said it");
    assert_ne!(n1, "You");
}

#[test]
fn test_villagers_talk_in_a_running_game() {
    let mut g = Game::new();
    for _ in 0..crate::session::FPS * 60 * 2 { g.update(); }
    let chats = g.talk_log.iter().filter(|r| r.kind == TalkKind::Chat).count();
    assert!(chats > 0, "villagers who meet should talk");
}

/// An org ready for the vote: three living members, one issue heard
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
fn test_deciding_founds_the_campaign_on_the_picked_framing() {
    let mut g = Game::new();
    org_ready(&mut g);
    let came = g.villagers[..3].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    g.resolve_meeting(&came, 0, id, (false, false, false, false, false));
    g.decide_at(0, 1); // take it on, naming the foe
    let c = crate::game::campaign::campaign_of(&g.world, g.org).expect("the campaign founded");
    let cp = g.world.get_component::<crate::game::campaign::Campaign>(c).unwrap();
    assert_eq!(cp.issue, "the taxes");
    assert!(matches!(cp.framing, crate::game::campaign::Framing::CommonFoe));
    assert!(g.seats[0].pending_founding.is_none());
    // and the outcome lands on the meeting's record
    let rec = g.talk_log.iter().find(|r| r.id == id).unwrap();
    assert!(rec.lines.iter().any(|l| l.text.contains("they'll fight for the taxes")), "{:?}", rec.lines);
}

#[test]
fn test_a_talk_opens_with_who_they_are() {
    let mut g = Game::new();
    let v = g.villagers[7];
    g.open_player_talk(0, v, crate::game::conversation::Relation::Acquaintance, "the day's work");
    let t = g.seats[0].talk.as_ref().expect("a talk under way");
    // the profile line heads the talk - who they are before the gist
    assert!(!t.profile.is_empty(), "the talk opened bare");
}

#[test]
fn test_pressing_talk_again_keeps_the_conversation_going() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    let before = g.seats[0].talk.as_ref().expect("a talk under way").id;
    let records = g.talk_log.len();
    // pressing T again while talking must not reset the conversation
    g.act_at(0, 0);
    let t = g.seats[0].talk.as_ref().expect("the talk must stay open");
    assert_eq!(t.id, before, "a second Talk press restarted the talk");
    assert_eq!(g.talk_log.len(), records, "the re-press opened a fresh talk");
    // and talking to someone new still opens fresh
    let j = adults(&g).into_iter().find(|&k| k != i).unwrap();
    talk_beside(&mut g, j);
    let after = g.seats[0].talk.as_ref().expect("the new talk opened").id;
    assert_ne!(after, before, "a different villager gets their own talk");
}

#[test]
fn test_a_fresh_talk_always_offers_something_to_say() {
    // a normal opening offers replies, or at least the leave - never a
    // bare panel that hangs up on its own (#1058)
    let mut g = Game::new();
    for &i in adults(&g).iter().take(10) {
        g.seats[0].talk = None;
        talk_beside(&mut g, i);
        let t = g.seats[0].talk.as_ref().expect("the talk must stay open");
        assert!(!t.options.is_empty(), "villager {i} opened with nothing to say");
    }
}

#[test]
fn test_an_old_hello_greys_the_name_but_a_stranger_stays_bright() {
    // a face you let drift greys out; one you've never met doesn't (#1069)
    let mut g = Game::new();
    let (met, stranger) = (g.villagers[0], g.villagers[1]);
    let pe = g.player;
    crate::game::player::act_talk(&mut g.world, pe, met, 0);
    g.tick = 30; // half gone grey
    let stale_of = |g: &Game, v| g.ui_model().people.iter()
        .find(|p| p.e == v).map(|p| p.stale).unwrap_or(-1.0);
    let (sm, ss) = (stale_of(&g, met), stale_of(&g, stranger));
    assert!((sm - 0.5).abs() < 0.01, "30 days of silence reads half-faded, got {}", sm);
    assert_eq!(ss, 0.0, "a stranger has nothing to fade");
    g.tick = 200; // past the fade's end
    assert_eq!(stale_of(&g, met), 1.0, "long silence greys all the way out");
    // a meeting counts as speaking: the clock starts over
    let (pe, tick) = (g.player, g.tick);
    crate::game::player::note_conversation(&mut g.world, pe, met, tick);
    assert_eq!(stale_of(&g, met), 0.0, "seeing them at the meeting freshens the name");
}

#[test]
fn test_a_talk_holds_them_where_they_stand() {
    // folk in a Talk with you don't walk off mid-line: a walk under way
    // stops, no new one dispatches, and the pull home leaves them be -
    // while a villager you aren't talking to still drifts to the table
    let mut g = Game::new();
    let (i, j) = (adults(&g)[0], adults(&g)[1]);
    let (v, w) = (g.villagers[i], g.villagers[j]);
    // a walk under way when you hail them: the hail stops their feet
    let p = *g.world.get_component::<Pos>(v).unwrap();
    g.travels.push(Travel { who: v, path: vec![(p.x.round() as i32 + 4, p.y.round() as i32)], goal: Goal::Wander });
    talk_beside(&mut g, i);
    assert!(g.seats[0].talk.is_some(), "a talk under way");
    assert!(!g.travels.iter().any(|t| t.who == v), "the hail stops their feet");
    // and no new one starts while they stand with you
    assert!(!g.send(v, Goal::Wander), "no fresh errand mid-talk");
    assert!(!g.travels.iter().any(|t| t.who == v));
    // hungry and away from the table: the seek and the pull home would
    // carry anyone else off, but not one you're talking to
    for e in [v, w] {
        g.world.get_component_mut::<villager::Hunger>(e).unwrap().value = 5.0;
    }
    let (vp, wp0) = (*g.world.get_component::<Pos>(v).unwrap(),
        *g.world.get_component::<Pos>(w).unwrap());
    for _ in 0..40 { g.update(); }
    let (vp1, wp1) = (*g.world.get_component::<Pos>(v).unwrap(),
        *g.world.get_component::<Pos>(w).unwrap());
    assert_eq!((vp1.x, vp1.y), (vp.x, vp.y),
        "the one you're talking to holds their ground");
    assert!((wp1.x - wp0.x).abs() + (wp1.y - wp0.y).abs() > 0.1,
        "the one you aren't talking to still heads for the table: {:?} -> {:?}", (wp0.x, wp0.y), (wp1.x, wp1.y));
    // once the talk's done their feet are their own again: you walk on,
    // it closes, and the next errand can send them
    let pp = g.world.get_component::<Pos>(g.player).unwrap();
    let far = Pos { x: pp.x + PLAYER_TALK_RANGE as f32 + 4.0, y: pp.y };
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = far;
    g.advance_player_talks();
    assert!(g.seats[0].talk.is_none(), "walking away still ends it");
    assert!(g.send(v, Goal::Wander), "free to be sent once it's done");
}

/// The cold brush-offs a Slighted memory serves (#1183)
fn cold_lines() -> [&'static str; 4] {
    ["You're back. What is it now?", "Hmph. Say your piece and go.",
     "We left it badly last time. Say what you came for.", "You've a nerve, coming back."]
}

#[test]
fn test_a_talked_back_answer_is_remembered_next_time() {
    use crate::game::desires::{Known, Problem, Reveal};
    let mut g = Game::new();
    let ad = adults(&g);
    // someone whose first options include a PushBack
    let mut slighted = None;
    for &i in ad.iter() {
        let v = talk_beside(&mut g, i);
        let has = g.seats[0].talk.as_ref().unwrap().options.iter()
            .position(|(a, _)| *a == Act::PushBack);
        if let Some(p) = has {
            g.talk_pick_at(0, Some(p));
            g.talk_pick_at(0, None);
            slighted = Some(v);
            break;
        }
        g.close_player_talk(0, None);
    }
    let v = slighted.expect("someone takes a PushBack on this map");
    assert_eq!(g.talk_mood_of(v), Some(TalkMood::Slighted));
    // they've told you a secret before: asks would be on offer...
    g.world.add_component(v, Known { revealed: vec![Reveal::Problem(Problem::Taxed)] });
    let i = g.villagers.iter().position(|&x| x == v).unwrap();
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap();
    // ...but what's on their mind now is you, and no proposition lands
    assert!(cold_lines().contains(&t.gist.as_str()), "the gist is the cold shoulder: {:?}", t.gist);
    assert!(t.asks.is_empty(), "a cold ending closes the asks: {:?}", t.asks);
}

#[test]
fn test_a_kind_answer_warms_the_hello_after() {
    let mut g = Game::new();
    let ad = adults(&g);
    let mut warmed = None;
    for &i in ad.iter() {
        let v = talk_beside(&mut g, i);
        // talk along until a kind reply is on offer (the first beat's
        // usually a greeting), taking it when it comes
        for _ in 0..10 {
            let Some(t) = &g.seats[0].talk else { break };
            match t.options.iter().position(|(a, _)| matches!(a, Act::Agree | Act::Answer)) {
                Some(p) => { g.talk_pick_at(0, Some(p)); break; }
                None => g.talk_pick_at(0, Some(0)),
            }
        }
        if g.seats[0].talk.is_some() {
            if g.seats[0].talk.as_ref().unwrap().mood == TalkMood::Warmed {
                g.talk_pick_at(0, None);
                warmed = Some(v);
                break;
            }
        } else if g.talk_mood_of(v) == Some(TalkMood::Warmed) { warmed = Some(v); break; }
        g.close_player_talk(0, None);
    }
    let v = warmed.expect("someone takes a kind answer on this map");
    assert_eq!(g.talk_mood_of(v), Some(TalkMood::Warmed));
    // the next hello opens on their real mind, not a cold shoulder
    let i = g.villagers.iter().position(|&x| x == v).unwrap();
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(!cold_lines().contains(&t.gist.as_str()), "still cold after a warm parting: {:?}", t.gist);
}

#[test]
fn test_leaving_before_they_speak_leaves_it_cool() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = talk_beside(&mut g, i);
    g.talk_pick_at(0, None);
    assert_eq!(g.talk_mood_of(v), Some(TalkMood::Cool));
}

/// Open a talk on a stingy villager who's already shared something (the
/// asks are on the table), answer the greeting, and take the landing or
/// the wrong note of their audition - returns the talk's record id
fn audition(g: &mut Game, i: usize, pass: bool) -> (Entity, u64) {
    use crate::game::desires::{Problem, Reveal};
    audition_bonded(g, i, pass, vec![Reveal::Problem(Problem::Taxed)], false)
}

/// `audition` with the bond and what you've heard set up ahead of time -
/// `bond` puts one past conversation on the roll so a stage exists to
/// climb (#1188)
fn audition_bonded(g: &mut Game, i: usize, pass: bool, revealed: Vec<crate::game::desires::Reveal>, bond: bool) -> (Entity, u64) {
    use crate::game::desires::Known;
    let v = g.villagers[i];
    g.world.add_component(v, Personality { traits: vec![Trait::Stingy] });
    g.world.add_component(v, Known { revealed });
    if bond {
        let pe = g.player;
        crate::game::relationships::record_interaction(&mut g.world, pe,
            crate::game::relationships::Interaction { other: v,
                kind: crate::game::relationships::InteractionKind::Conversation, intensity: 0.3 });
    }
    talk_beside(g, i);
    let t = g.seats[0].talk.as_ref().expect("a talk under way").clone();
    if bond {
        assert!(!t.asks.is_empty(), "they'd bring a proposition to this talk");
    }
    g.talk_pick_at(0, Some(0)); // the greeting's answer
    let t = g.seats[0].talk.as_ref().expect("still talking").clone();
    assert_eq!(t.opt_tree, Some(crate::game::conversation::PROBE_TREE),
        "after the hello they put their test to you: {:?}", t.opt_tree);
    assert_eq!(t.options.len(), 2, "the audition offers the landing and the wrong note");
    // pick the fork whose words are the stingy test's pass (or miss)
    let probe = crate::game::conversation::probe_for(&t.villager).unwrap();
    let wanted = if pass { probe.pass.1 } else { probe.fail.1 };
    let pick = t.options.iter().position(|(_, l)| wanted.contains(&l.as_str()))
        .expect("their fork among the two");
    g.talk_pick_at(0, Some(pick));
    (v, t.id)
}

#[test]
fn test_a_good_answer_to_the_audition_opens_them_up() {
    use crate::game::conversation::Vibe;
    let mut g = Game::new();
    let i = adults(&g)[0];
    let (_, id) = audition(&mut g, i, true);
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    assert_eq!(t.vibe, Some(Vibe::Safe));
    assert!(!t.asks.is_empty(), "safe: the propositions stay on offer");
    let rec = g.talk_log.iter().find(|r| r.id == id).unwrap();
    assert!(rec.lines.iter().any(|l| l.text.contains("warms to you")),
        "the read is narrated: {:?}", rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

#[test]
fn test_a_wrong_note_keeps_the_talk_polite() {
    use crate::game::conversation::Vibe;
    let mut g = Game::new();
    let i = adults(&g)[0];
    let (_, id) = audition(&mut g, i, false);
    let t = g.seats[0].talk.as_ref().expect("the talk goes on, politely").clone();
    assert_eq!(t.vibe, Some(Vibe::Mismatch));
    // coin and nerve still stand to be tried (#1225) - it's the bond's
    // own asks a bad read keeps out of reach
    assert!(t.asks.iter().all(|o| o.key != crate::game::organizing::AskKey::Earned),
        "a mismatch brings no earned proposition: {:?}", t.asks);
    let rec = g.talk_log.iter().find(|r| r.id == id).unwrap();
    assert!(rec.lines.iter().any(|l| l.text.contains("keeps it to the weather")),
        "the shallow read is narrated: {:?}", rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

#[test]
fn test_a_safe_read_opens_the_next_rung_early() {
    use crate::game::conversation::Vibe;
    use crate::game::desires::{Problem, Reveal};
    use crate::game::organizing::{AskKey, PropKind};
    let mut g = Game::new();
    let i = adults(&g)[0];
    // an acquaintance with one piece heard offers only the light ask at
    // the hello - but a good answer to their test opens the next rung
    // right there in the talk (#1188)
    audition_bonded(&mut g, i, true, vec![Reveal::Problem(Problem::Taxed)], true);
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    assert_eq!(t.vibe, Some(Vibe::Safe));
    assert!(t.asks.iter().any(|o| o.kind == PropKind::Introduce && o.key == AskKey::Earned)
        && t.asks.iter().any(|o| o.kind == PropKind::InviteFriends && o.key == AskKey::Earned),
        "safe: the known-tier asks open early: {:?}", t.asks);
    assert!(!t.asks.iter().any(|o| o.kind == PropKind::ComeAlong && o.key == AskKey::Earned),
        "meetings still wait on the bond: {:?}", t.asks);
}

#[test]
fn test_a_mismatch_keeps_the_asks_shallow_not_barred() {
    use crate::game::conversation::Vibe;
    use crate::game::desires::{Problem, Reveal, Want};
    use crate::game::organizing::{AskKey, PropKind, RelStage};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    // a villager you've known through three pieces would offer the
    // deeper asks - a wrong note in their audition drops them a rung,
    // leaving only the light one (#1188)
    audition_bonded(&mut g, i, false, vec![
        Reveal::Problem(Problem::Taxed), Reveal::Want(Want::GoodFood), Reveal::Want(Want::Love)], true);
    assert_eq!(crate::game::organizing::stage_of(&g.world, g.player, v), RelStage::Known);
    let t = g.seats[0].talk.as_ref().expect("the talk goes on, politely").clone();
    assert_eq!(t.vibe, Some(Vibe::Mismatch));
    let earned: Vec<PropKind> = t.asks.iter()
        .filter(|o| o.key == AskKey::Earned).map(|o| o.kind).collect();
    assert_eq!(earned, vec![PropKind::Introduce],
        "a mismatch stays slow - the light ask still on the table: {:?}", t.asks);
}

/// Pick through a Talk until the ask-slot shows, take it, and hand the
/// topics it opened - `t.options` at that point is the question menu
fn open_ask_menu(g: &mut Game) -> Vec<(Act, String)> {
    for _ in 0..30 {
        let t = g.seats[0].talk.as_ref().expect("still talking").clone();
        if let Some(i) = t.options.iter().position(|(_, l)| l == "Put a question to them") {
            g.talk_pick_at(0, Some(i));
            return g.seats[0].talk.as_ref().expect("the menu opens").options.clone();
        }
        match t.options.iter().position(|(a, _)| *a != Act::Farewell) {
            Some(i) => g.talk_pick_at(0, Some(i)),
            None => break,
        }
    }
    panic!("the ask slot never showed");
}

#[test]
fn test_a_talk_lets_you_put_your_own_questions() {
    // the ask-slot rides the pick list (#1189): take it and the topics
    // they can speak to open as a menu - your question, their answer,
    // then the talk picks back up where it stood
    let mut g = Game::new();
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    // the standing asks never leave the table, and the out is a remark
    assert!(menu.iter().any(|(_, l)| l == "Ask about themselves"), "{:?}", menu);
    assert_eq!(menu.last().unwrap().1, "Say nothing of it");
    let p = menu.iter().position(|(_, l)| l == "Ask about themselves").unwrap();
    let pe = g.player;
    g.talk_pick_at(0, Some(p));
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    let asks = crate::game::dialogue_trees::ASK_TOPICS.iter()
        .find(|tp| tp.label == "themselves").unwrap().ask;
    assert!(rec.lines.iter().any(|l| l.who == Some(pe) && asks.contains(&l.text.as_str())),
        "your question's on the record: {:?}", rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
    assert!(!t.options.is_empty(), "asking isn't a farewell - options stay live");
}

#[test]
fn test_asking_what_weighs_on_them_draws_it_out() {
    // a kept secret opens the deeper ask (#1189): putting the question
    // is the way a trouble comes out - the same reveal a talk's opening
    // might have rolled, landed because you asked
    use crate::game::desires::{InnerLife, Known, Problem};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    g.world.add_component(v, InnerLife {
        wants: vec![], grievances: vec![(Problem::Taxed, 1000)], satisfied: vec![] });
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    let p = menu.iter().position(|(_, l)| l == "Ask about what weighs on them")
        .expect("a kept secret opens the deep ask");
    g.talk_pick_at(0, Some(p));
    let known = g.world.get_component::<Known>(v).expect("the asking drew something out");
    assert!(!known.revealed.is_empty(), "nothing drawn: {:?}", known.revealed);
    // and the telling of it lands on the talk's record
    let t = g.seats[0].talk.as_ref().expect("the talk goes on").clone();
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    assert!(rec.lines.iter().any(|l| l.who == Some(v)),
        "they told it on the record: {:?}", rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

#[test]
fn test_the_trouble_they_told_is_an_ask() {
    // what's already shared sits on the menu as its own question
    // (#1189) - asking after it is following up, not prying anew
    use crate::game::desires::{Known, Problem, Reveal};
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    g.world.add_component(v, Known { revealed: vec![Reveal::Problem(Problem::Taxed)] });
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    assert!(menu.iter().any(|(_, l)| l == "Ask about the trouble they told"),
        "a shared trouble opens the follow-up: {:?}", menu);
}

#[test]
fn test_a_live_mystery_is_its_own_ask() {
    // folk are asking after something (#1189): putting the question
    // pulls a piece of it the way the gossip line would
    use crate::session::mystery::SeatMystery;
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    g.seats[0].mystery = Some(SeatMystery { def: 0, known: 0, asker: v });
    talk_beside(&mut g, i);
    let menu = open_ask_menu(&mut g);
    let p = menu.iter().position(|(_, l)| l == "Ask about the question folk whisper of")
        .expect("a live mystery opens the ask");
    let before = g.seats[0].mystery.as_ref().unwrap().heard();
    g.talk_pick_at(0, Some(p));
    let m = g.seats[0].mystery.as_ref().unwrap();
    assert_eq!(m.heard(), before + 1, "the asking drew a fact");
    let t = g.seats[0].talk.as_ref().expect("still talking").clone();
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    let facts: Vec<&str> = crate::game::mysteries::MYSTERIES[0].facts.to_vec();
    assert!(rec.lines.iter().any(|l| l.who == Some(v) && facts.contains(&l.text.as_str())),
        "they passed the piece on: {:?}", rec.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

/// A villager pressed twice in a day runs dry (#1226): the second talk
/// opens flat - no fresh mind to speak, no propositions, one word at a
/// time - where the first ran full
#[test]
fn test_a_second_pressing_runs_flat() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = talk_beside(&mut g, i);
    assert_eq!(g.seats[0].talk.as_ref().unwrap().tired, 1, "the day's first pressing runs full");
    g.close_player_talk(0, None);
    assert_eq!(talk_beside(&mut g, i), v, "same villager, pressed again");
    let t = g.seats[0].talk.as_ref().expect("the talk goes on, flatter").clone();
    assert_eq!(t.tired, 2, "the day's tally counted the second pressing");
    // flat: no propositions, no fresh mind to speak, and they say so
    assert!(t.asks.is_empty(), "a tired villager keeps their asks: {:?}", t.asks);
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    assert!(!rec.lines.iter().any(|l| l.gist), "the gist's already been said today");
    assert!(rec.lines.iter().any(|l| l.text.contains("Still here")),
        "they acknowledge the pressing: {:?}", rec.lines);
    // and what comes back runs a word at a time
    let mut saw_short = false;
    for _ in 0..4 {
        let Some(t) = g.seats[0].talk.as_ref() else { break };
        assert!(t.options.len() <= 1, "the pressed list runs short: {:?}", t.options);
        saw_short |= t.options.len() == 1;
        g.talk_pick_at(0, Some(0));
    }
    assert!(saw_short, "a flat talk still answers");
}

/// The third pressing of a day finds them nearly talked out (#1226): a
/// word or two and the talk itself calls it done
#[test]
fn test_a_third_pressing_runs_dry() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    for _ in 0..2 { talk_beside(&mut g, i); g.close_player_talk(0, None); }
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().expect("a third talk still opens").clone();
    assert_eq!(t.tired, 3);
    let id = t.id;
    let mut picks = 0;
    while g.seats[0].talk.is_some() && picks < 6 {
        g.talk_pick_at(0, Some(0));
        picks += 1;
    }
    assert!(g.seats[0].talk.is_none(), "the third pressing doesn't linger");
    assert!(picks <= 3, "it ran a word or two, not a conversation: {} picks", picks);
    let rec = g.talk_log.iter().find(|r| r.id == id).unwrap();
    assert!(rec.lines.iter().any(|l| l.text.contains("run out of things to say")),
        "the thin parting's on the record: {:?}", rec.lines.last().map(|l| l.text.clone()));
}

/// The tally lifts at dawn (#1226): yesterday's pressings don't follow
/// a villager into the next day's talk
#[test]
fn test_a_new_day_lifts_the_tiredness() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    g.close_player_talk(0, None);
    g.tick += 1;
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().expect("a fresh day's talk").clone();
    assert_eq!(t.tired, 1, "dawn lifts the tally");
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    assert!(rec.lines.iter().any(|l| l.gist), "a fresh day's talk opens with the gist again");
}

// --- signature scenes (#1227): the village's small set of stories ---

/// Two grown villagers with no scene waiting for them
fn plain_pair(g: &Game) -> (Entity, Entity) {
    let ad = adults(g);
    for &i in &ad {
        for &j in &ad {
            if i == j { continue; }
            let (a, b) = (g.villagers[i], g.villagers[j]);
            if crate::game::scenes::scene_for(&g.world, a, b, g.tick).is_none() { return (a, b); }
        }
    }
    panic!("expected a pair with no signature scene");
}

#[test]
fn test_a_feuding_pair_plays_their_scene() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    crate::game::feuds::spawn_feud(&mut g.world, a, b, "a leaning hedge");
    crate::game::relationships::add_relationship(&mut g.world,
        crate::game::relationships::Relationship {
            from: a, other: b, kind: RelKind::Rival, strength: -0.5, interactions: 3 });
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    assert_eq!(r.kind, TalkKind::Chat);
    assert_eq!(r.lines[0].text, "Thy fence still leans on my strip.");
    // the offender speaks first, however the street order fell
    assert_eq!(r.lines[0].who, Some(a));
    assert_eq!(r.lines[1].who, Some(b));
    // and the TALK panel shows it like any village chat
    let i = g.villagers.iter().position(|&v| v == b).unwrap();
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    assert!(g.talk_panel_at(0).rows.iter().any(|row| row.text == r.lines[0].text));
}

#[test]
fn test_the_scene_take_moves_with_the_bond() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    crate::game::feuds::spawn_feud(&mut g.world, a, b, "a leaning hedge");
    let rel = crate::game::relationships::add_relationship(&mut g.world,
        crate::game::relationships::Relationship {
            from: a, other: b, kind: RelKind::Rival, strength: 0.0, interactions: 1 });
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    assert_eq!(r.lines[0].text, "Thy boy ran my fence line yesterday.",
        "a cooling feud plays the middle take: {:?}", r.lines);
    // bonds warm, the feud thaws - the next telling is a kinder one
    g.world.get_component_mut::<crate::game::relationships::Relationship>(rel).unwrap().strength = 0.5;
    g.tick += 14;
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    assert_eq!(r.lines[0].text, "Cold coming in early this year.",
        "a thawed feud plays the warm take: {:?}", r.lines);
}

#[test]
fn test_the_widow_and_the_priest_have_theirs() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    g.world.add_component(a, crate::game::desires::InnerLife {
        wants: vec![], satisfied: vec![],
        grievances: vec![(crate::game::desires::Problem::Bereaved, g.tick + 10)] });
    g.world.add_component(b, crate::game::vocation::craft(Vocation::Priest, 0.9));
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    assert_eq!(r.lines[0].text, "I saw thee at the grave again, child.");
    assert_eq!(r.lines[0].who, Some(b), "the priest opens: {:?}", r.lines);
    assert_eq!(r.lines[1].who, Some(a));
}

#[test]
fn test_the_pedlar_and_the_watch_have_theirs() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    g.world.add_component(a, crate::game::vocation::TownRole::Trader);
    g.world.add_component(b, crate::game::vocation::craft(Vocation::Knight, 0.5));
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    assert_eq!(r.lines[0].text, "Honest goods, watch - same as the last three times.");
    assert_eq!(r.lines[0].who, Some(a), "the pedlar opens: {:?}", r.lines);
    // and it holds whichever way the street lists them
    let mut g2 = Game::new();
    let (a2, b2) = (a, b);
    g2.world.add_component(a2, crate::game::vocation::TownRole::Trader);
    g2.world.add_component(b2, crate::game::vocation::craft(Vocation::Knight, 0.5));
    assert!(g2.villagers_chat(b2, a2));
    let r2 = g2.talk_log.last().unwrap().clone();
    assert_eq!(r2.lines[0].who, Some(a2));
    assert_eq!(r2.lines[0].text, r.lines[0].text);
}

#[test]
fn test_a_scene_cools_off_between_tellings() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    crate::game::feuds::spawn_feud(&mut g.world, a, b, "a leaning hedge");
    crate::game::relationships::add_relationship(&mut g.world,
        crate::game::relationships::Relationship {
            from: a, other: b, kind: RelKind::Rival, strength: -0.5, interactions: 3 });
    assert!(g.villagers_chat(a, b));
    assert_eq!(g.talk_log.last().unwrap().lines[0].text,
        "Thy fence still leans on my strip.");
    // pressed again the same day, they fall back to small talk
    assert!(g.villagers_chat(a, b));
    assert_ne!(g.talk_log.last().unwrap().lines[0].text,
        "Thy fence still leans on my strip.");
    // a fortnight on, the scene's theirs again
    g.tick += 14;
    assert!(g.villagers_chat(a, b));
    assert_eq!(g.talk_log.last().unwrap().lines[0].text,
        "Thy fence still leans on my strip.");
}

#[test]
fn test_a_pair_with_no_story_small_talks() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    let scripted: Vec<&str> = crate::game::scenes::SCENES.iter()
        .flat_map(|s| s.variants).flat_map(|v| v.lines.iter().map(|l| l.1)).collect();
    assert!(!r.lines.iter().any(|l| scripted.contains(&l.text.as_str())),
        "no scene lines for an unmatched pair: {:?}", r.lines);
}

// --- the bond register (#1228): how well they know you swaps how they talk ---

/// Walk the organizer's bond with a villager to a strength (and mark it
/// spoken-to, so the stage ladder can climb past stranger)
fn bond_to(g: &mut Game, v: Entity, strength: f32) {
    let pe = g.player;
    crate::game::relationships::record_interaction(&mut g.world, pe,
        crate::game::relationships::Interaction { other: v,
            kind: crate::game::relationships::InteractionKind::Conversation, intensity: 0.3 });
    crate::game::relationships::find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component_mut::<crate::game::relationships::Relationship>(r))
        .unwrap().strength = strength;
}

#[test]
fn test_an_acquaintance_opens_like_a_neighbor() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    bond_to(&mut g, v, 0.2);
    assert_eq!(crate::game::organizing::stage_of(&g.world, g.player, v),
        crate::game::organizing::RelStage::Acquaintance);
    talk_beside(&mut g, i);
    let rec = g.talk_log.last().unwrap().clone();
    let first = rec.lines.iter().find(|l| l.who == Some(v) && !l.gist)
        .map(|l| l.text.clone()).unwrap_or_default();
    let calls = ["Morrow, ", "good to see a familiar face", "Well met, ", "Neighbor!"];
    assert!(calls.iter().any(|c| first.contains(c)),
        "the neighbor register greets, not the sky's smalltalk: {}", first);
}

#[test]
fn test_two_members_carry_the_cause_in_the_street() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    // no shared cause yet: they small-talk like any neighbors
    assert!(g.villagers_chat(a, b));
    let before = g.talk_log.last().unwrap().clone();
    assert!(!before.lines.iter().any(|l| l.text.contains("the cause")
        || l.text.contains("the meet's")), "open-air talk: {:?}", before.lines);
    // sworn to the same org, the next hello drops to a quiet word
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org)
        .unwrap().members = vec![a, b];
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    let calls = ["the meet's", "the cause keeps", "folk are talking", "Between us and the wall"];
    assert!(calls.iter().any(|c| r.lines[0].text.contains(c)),
        "the fellows register greets: {}", r.lines[0].text);
}

#[test]
fn test_one_member_still_small_talks() {
    let mut g = Game::new();
    let (a, b) = plain_pair(&g);
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org)
        .unwrap().members = vec![a];
    assert!(g.villagers_chat(a, b));
    let r = g.talk_log.last().unwrap().clone();
    let calls = ["the meet's", "the cause keeps", "Between us and the wall"];
    assert!(!calls.iter().any(|c| r.lines[0].text.contains(c)),
        "a lone member gets open-air talk: {}", r.lines[0].text);
}

#[test]
fn test_a_follower_opens_with_the_causes_weather() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    bond_to(&mut g, v, 0.6);
    g.seats[0].mystery = None;
    let pe = g.player;
    g.world.add_component(pe, crate::game::organizing::MeetingPlan {
        for_day: Some(g.tick + 3), ..Default::default() });
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().unwrap().clone();
    assert!(t.gist.contains("e gather in 3 days"),
        "the meet's on the gist, not the sky: {}", t.gist);
}

#[test]
fn test_the_causes_weather_is_a_sworn_privilege() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    // no bond - no inside weather
    assert!(g.cause_gist(0, v).is_none());
    // acquaintance is still the street's weather: the stage gates it
    bond_to(&mut g, v, 0.2);
    assert!(g.cause_gist(0, v).is_none());
    // sworn, with a live plan - the meet's the weather
    bond_to(&mut g, v, 0.6);
    let pe = g.player;
    g.world.add_component(pe, crate::game::organizing::MeetingPlan {
        for_day: Some(g.tick + 1), ..Default::default() });
    let (_, said) = g.cause_gist(0, v).unwrap();
    assert!(said.contains("tomorrow"), "{}", said);
}

// --- the first greeting carries the warmth (#1232): one ledger beat a day ---

/// How many bond-beats the pair's ledger has taken: every recorded
/// interaction is one
fn ledger(g: &Game, a: Entity, b: Entity) -> u32 {
    crate::game::relationships::find_relationship(&g.world, a, b)
        .and_then(|r| g.world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.interactions).unwrap_or(0)
}

/// Pick the first bond-moving option on offer (a kind answer, a slight,
/// a hello or a goodbye); false if none are up right now
fn pick_bond_moving(g: &mut Game) -> bool {
    let idx = g.seats[0].talk.as_ref().and_then(|t| t.options.iter()
        .position(|(a, _)| matches!(a, Act::PushBack | Act::Agree | Act::Answer
            | Act::Greet | Act::Farewell)));
    match idx { Some(i) => { g.talk_pick_at(0, Some(i)); true } None => false }
}

#[test]
fn test_a_re_talk_the_same_day_leaves_the_bond_alone() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let pe = g.player;
    let v = talk_beside(&mut g, i);
    let mut p = 0;
    while g.seats[0].talk.is_some() && p < 30 {
        if !pick_bond_moving(&mut g) { g.talk_pick_at(0, Some(0)); }
        p += 1;
    }
    assert!(crate::game::player::pick_spent_today(&g.world, v, pe, g.tick),
        "the first talk spent the day's beat");
    // let the sit-down's armed depth settle - it's the first talk's own
    // beat, and the baseline shouldn't count it as the re-talk's
    crate::engine::ecs::System::run(&mut crate::sim::community::OneOnOneSystem, &mut g.world);
    let beat = ledger(&g, pe, v);
    assert!(beat > 0, "the first talk moved the bond");
    let rap = crate::game::player::rapport(&g.world, pe, v);
    // a second sit-down the same day: the lines play, the ledger doesn't
    talk_beside(&mut g, i);
    let mut p = 0;
    while g.seats[0].talk.is_some() && p < 30 {
        if !pick_bond_moving(&mut g) { g.talk_pick_at(0, Some(0)); }
        p += 1;
    }
    crate::engine::ecs::System::run(&mut crate::sim::community::OneOnOneSystem, &mut g.world);
    assert_eq!(ledger(&g, pe, v), beat,
        "a re-talk's picks and its sit-down's depth spend nothing more");
    // rapport isn't the bond ledger - the catch-up still counts
    assert!(crate::game::player::rapport(&g.world, pe, v) > rap,
        "rapport grows on the repeat all the same");
}

#[test]
fn test_the_first_pick_spends_the_days_beat_either_valence() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let pe = g.player;
    let v = talk_beside(&mut g, i);
    let before = ledger(&g, pe, v);
    // the first bond-moving pick of the day - whatever it is - lands
    // and stamps; inside the same talk the rest read as flavour
    let mut stamped = false;
    for _ in 0..30 {
        if g.seats[0].talk.is_none() { break; }
        if pick_bond_moving(&mut g) { stamped = true; break; }
        g.talk_pick_at(0, Some(0));
    }
    assert!(stamped, "the talk offered a bond-moving pick");
    assert!(crate::game::player::pick_spent_today(&g.world, v, pe, g.tick));
    assert!(ledger(&g, pe, v) <= before + 1, "one pick, one beat at most");
    // and a second bond-moving pick inside the same talk adds nothing
    let now = ledger(&g, pe, v);
    for _ in 0..30 {
        if g.seats[0].talk.is_none() { break; }
        if pick_bond_moving(&mut g) { break; }
        g.talk_pick_at(0, Some(0));
    }
    assert_eq!(ledger(&g, pe, v), now,
        "a second pick in the same talk records nothing more");
}

#[test]
fn test_a_new_day_earns_a_new_beat() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let pe = g.player;
    let v = talk_beside(&mut g, i);
    let mut p = 0;
    while g.seats[0].talk.is_some() && p < 30 { g.talk_pick_at(0, Some(0)); p += 1; }
    let beat = ledger(&g, pe, v);
    g.tick += 1;
    talk_beside(&mut g, i);
    // the morning's beat is fresh - the day's first bond-moving pick lands
    let mut took = false;
    for _ in 0..30 {
        if g.seats[0].talk.is_none() { break; }
        if pick_bond_moving(&mut g) { took = true; break; }
        g.talk_pick_at(0, Some(0));
    }
    assert!(took);
    assert!(ledger(&g, pe, v) > beat, "a new day earns a new beat");
}
