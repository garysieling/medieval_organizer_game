//! Conversations: the rules cascade in play, the talk log, and the TALK and conversation panels.
use crate::session::*;
use crate::session::dialogue::*;
use crate::engine::ecs::Entity;
use crate::engine::render::Buffer;
use crate::game::conversation::Relation;
use crate::game::relationships::RelKind;
use crate::game::traits::Personality;
use crate::game::villager::{self, Hunger};
use crate::game::vocation::{Kid, Vocation};
use crate::session::model::{ConvoPanelV, TalkPanelV, TalkRowV};
use crate::client::talk::*;
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
    assert!(panel.in_talk && panel.options.len() == t.options.len());
    // pick the first reply: it's said, as previewed
    g.talk_pick_at(0, Some(0));
    let rec = g.talk_log.iter().find(|r| r.id == rec.id).unwrap();
    assert!(rec.lines.iter().any(|l| l.who == Some(g.player) && l.text == t.options[0].1), "{:?}", rec.lines);
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
fn test_long_lines_wrap_inside_the_talk_box_and_replies_click() {
    let long = "word ".repeat(40);
    let p = TalkPanelV {
        title: "TALK".into(),
        rows: (0..6).map(|k| TalkRowV { name: Some(format!("Speaker{}", k)), text: long.clone(), gist: k == 2, you: k % 2 == 1, rule: false, who: None }).collect(),
        options: vec![long.clone(), "Short one.".into()],
        in_talk: true,
    };
    let (rows, rects) = talk_layout(&p);
    for (y, text, _, name) in &rows {
        let n = name.as_ref().map(|n| n.0.len()).unwrap_or(0);
        assert!(n + text.len() <= TALK_CHARS, "row overflows: {:?}", text);
        assert!(*y >= TALK_Y + 14 && *y + 7 <= TALK_Y + TALK_H, "row at {} outside the box", y);
    }
    assert!(rows[0].3.is_some(), "the view opens on a speaker's line, not the tail of one");
    assert_eq!(rects.len(), 3, "two replies and Leave");
    let (x, y, w, h, _) = rects[1];
    assert_eq!(talk_click(&p, x + w / 2, y + h / 2), Some(Some(1)));
    let (x, y, _, _, _) = rects[2];
    assert_eq!(talk_click(&p, x + 1, y + 1), Some(None));
    assert_eq!(talk_click(&p, TALK_X - 5, y), None);
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

#[test]
fn test_the_talk_fills_the_conversation_box_with_clickable_lines() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    let c = g.convo_panel_at(0).expect("the talk on the conversation box");
    assert!(c.leave && !c.decides && !c.options.is_empty(), "{:?}", c.options);
    // a reply button clicks through as that reply
    let (boxes, _) = convo_layout(&c);
    let reply = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Reply(Some(0)))))
        .expect("a reply button");
    assert_eq!(convo_click(&c, reply.x + 2, reply.y + 2), Some(ConvoClick::Reply(Some(0))));
    // and the villager's own line selects them
    let line = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Speaker(_))))
        .expect("a speaker's line");
    match convo_click(&c, line.x + 2, line.y + 2) {
        Some(ConvoClick::Speaker(v)) => assert_eq!(v, i),
        other => panic!("a speaker's line clicked: {:?}", other),
    }
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
fn test_a_house_meeting_ends_in_the_organizers_call_not_a_done_deal() {
    let mut g = Game::new();
    org_ready(&mut g);
    let came = g.villagers[..3].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    let msg = g.resolve_meeting(&came, 0, id, (false, false, false, false, false));
    // the room leans, but nothing is founded until the organizer says so
    assert!(g.seats[0].pending_founding.is_some(), "{}", msg);
    assert!(crate::game::campaign::campaign_of(&g.world, g.org).is_none());
    let c = g.convo_panel_at(0).expect("the call on the box");
    assert!(c.decides && c.options.len() == 4, "{:?}", c.options);
    let issue = crate::game::desires::issue_name("the taxes".into());
    assert!(c.options.iter().take(3).all(|o| o.contains(&issue)), "{:?} vs {}", c.options, issue);
    // letting it rest answers the call and founds nothing
    let (boxes, _) = convo_layout(&c);
    let rest = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Decide(3))))
        .expect("the let-it-rest button");
    assert_eq!(convo_click(&c, rest.x + 2, rest.y + 2), Some(ConvoClick::Decide(3)));
    g.decide_at(0, 3);
    assert!(g.seats[0].pending_founding.is_none());
    assert!(crate::game::campaign::campaign_of(&g.world, g.org).is_none());
}

#[test]
fn test_the_conversation_box_draws_its_buttons() {
    let p = ConvoPanelV {
        title: "THE HARVESTERS - YOUR CALL".into(),
        rows: vec![
            TalkRowV { name: Some("Eliza".into()), text: "word ".repeat(30), gist: false, you: false, rule: false, who: Some(2) },
            TalkRowV { name: None, text: "The room leans to the taxes - your call.".into(), gist: true, you: false, rule: false, who: None },
        ],
        options: vec!["Take it on - a burden shared".into(), "Let it rest".into()],
        decides: true, leave: false,
    };
    let mut buf = Buffer::new(crate::session::W, crate::session::H);
    draw_convo_panel(&mut buf, &p, (-1, -1));
    let (boxes, h) = convo_layout(&p);
    for b in &boxes {
        assert!(b.y >= CONVO_Y && b.y + b.h <= CONVO_Y + h, "a button sits inside the box");
        assert!(b.x >= CONVO_X && b.x + b.w <= CONVO_X + CONVO_W);
    }
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
