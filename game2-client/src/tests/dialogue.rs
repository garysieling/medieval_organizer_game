//! Conversations: the rules cascade in play, the talk log, and the TALK and conversation panels.
use game2_core::session::*;
use game2_core::engine::ecs::Entity;
use game2_core::engine::render::Buffer;
use game2_core::game::villager;
use game2_core::game::vocation::Kid;
use game2_core::session::model::{ConvoPanelV, TalkPanelV, TalkRowV};
use crate::talk::*;
use game2_core::game::villager::Pos;

/// Living grown-up villager indices
fn adults(g: &Game) -> Vec<usize> {
    (0..g.villagers.len()).filter(|&i| g.world.get_component::<Kid>(g.villagers[i]).is_none()
        && !villager::is_dead(&g.world, g.villagers[i])).collect()
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
fn test_the_talk_fills_the_conversation_box_with_clickable_lines() {
    let mut g = Game::new_seeded(MAP_SEED);
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    let c = g.convo_panel_at(0).expect("the talk on the conversation box");
    assert!(c.leave && !c.decides && !c.options.is_empty(), "{:?}", c.options);
    // a reply button clicks through as that reply
    let (boxes, _) = convo_layout(&c, &mut ConvoPin::default());
    let reply = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Reply(Some(0)))))
        .expect("a reply button");
    assert_eq!(convo_click(&c, reply.x + 2, reply.y + 2, &mut ConvoPin::default()), Some(ConvoClick::Reply(Some(0))));
    // and the villager's own line selects them
    let line = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Speaker(_))))
        .expect("a speaker's line");
    match convo_click(&c, line.x + 2, line.y + 2, &mut ConvoPin::default()) {
        Some(ConvoClick::Speaker(v)) => assert_eq!(v, i),
        other => panic!("a speaker's line clicked: {:?}", other),
    }
}

/// An org ready for the vote: three living members, one issue heard
fn org_ready(g: &mut Game) {
    for &v in &g.villagers[..3] {
        game2_core::game::desires::add_grievance(&mut g.world, v, game2_core::game::desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<game2_core::sim::community::Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<game2_core::game::desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
}

#[test]
fn test_a_house_meeting_ends_in_the_organizers_call_not_a_done_deal() {
    let mut g = Game::new_seeded(MAP_SEED);
    org_ready(&mut g);
    let came = g.villagers[..3].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    let msg = g.resolve_meeting(&came, 0, id, (false, false, false, false, false));
    // the room leans, but nothing is founded until the organizer says so
    assert!(g.seats[0].pending_founding.is_some(), "{}", msg);
    assert!(game2_core::game::campaign::campaign_of(&g.world, g.org).is_none());
    let c = g.convo_panel_at(0).expect("the call on the box");
    assert!(c.decides && c.options.len() == 4, "{:?}", c.options);
    let issue = game2_core::game::desires::issue_name("the taxes".into());
    assert!(c.options.iter().take(3).all(|o| o.contains(&issue)), "{:?} vs {}", c.options, issue);
    // letting it rest answers the call and founds nothing
    let (boxes, _) = convo_layout(&c, &mut ConvoPin::default());
    let rest = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Decide(3))))
        .expect("the let-it-rest button");
    assert_eq!(convo_click(&c, rest.x + 2, rest.y + 2, &mut ConvoPin::default()), Some(ConvoClick::Decide(3)));
    g.decide_at(0, 3);
    assert!(g.seats[0].pending_founding.is_none());
    assert!(game2_core::game::campaign::campaign_of(&g.world, g.org).is_none());
}

#[test]
fn test_the_numbered_rows_take_the_digit_keys() {
    let mut g = Game::new_seeded(MAP_SEED);
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    let c = g.convo_panel_at(0).expect("the talk on the conversation box");
    assert!(c.leave && !c.options.is_empty());
    // a digit presses the reply drawn beside it
    match digit_cmd(Some(&c), 0) {
        Some(Command::TalkPick(Some(0))) => {}
        other => panic!("a digit mid-talk picked: {:?}", other),
    }
    // past the drawn rows a digit is swallowed - not a villager pick
    assert!(digit_cmd(Some(&c), c.options.len()).is_none());
    // and with no box open the digit picks a villager, as ever
    match digit_cmd(None, 0) {
        Some(Command::Select(0)) => {}
        other => panic!("a digit with no talk picked: {:?}", other),
    }
    // X and Escape take the offered leave - the pick the "x)" row clicks
    assert!(c.leave || g.talk_panel_at(0).in_talk, "the leave is offered somewhere");
    g.command(Command::TalkPick(None));
    assert!(g.seats[0].talk.is_none(), "the leave was taken");
}

#[test]
fn test_the_calls_rows_take_the_digit_keys_too() {
    let mut g = Game::new_seeded(MAP_SEED);
    org_ready(&mut g);
    let came = g.villagers[..3].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    g.resolve_meeting(&came, 0, id, (false, false, false, false, false));
    let c = g.convo_panel_at(0).expect("the call on the box");
    assert!(c.decides && c.options.len() == 4);
    // the founding call's framings answer digits as Decide, not TalkPick
    match digit_cmd(Some(&c), 3) {
        Some(Command::Decide(3)) => {}
        other => panic!("a digit on the call picked: {:?}", other),
    }
    assert!(digit_cmd(Some(&c), 8).is_none(), "a digit with no row is swallowed");
    // and the digit decides what the click does
    g.command(Command::Decide(3));
    assert!(g.seats[0].pending_founding.is_none());
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
    let mut buf = Buffer::new(game2_core::session::W, game2_core::session::H);
    draw_convo_panel(&mut buf, &p, (-1, -1), &mut ConvoPin::default());
    let (boxes, h) = convo_layout(&p, &mut ConvoPin::default());
    for b in &boxes {
        assert!(b.y >= CONVO_Y && b.y + b.h <= CONVO_Y + h, "a button sits inside the box");
        assert!(b.x >= CONVO_X && b.x + b.w <= CONVO_X + CONVO_W);
    }
}

#[test]
fn test_the_calls_question_rows_decide_the_probe() {
    let mut g = Game::new_seeded(MAP_SEED);
    g.seats[0].frame_offer = Some(g.tick);
    let c = g.convo_panel_at(0).expect("the question on the box");
    assert_eq!(c.title, "THE CALL - WHAT WILL YOU ASK");
    assert!(c.decides && c.options.len() == 2);
    match digit_cmd(Some(&c), 1) {
        Some(Command::Decide(1)) => {}
        other => panic!("a digit on the question picked: {:?}", other),
    }
    let mut buf = Buffer::new(game2_core::session::W, game2_core::session::H);
    draw_convo_panel(&mut buf, &c, (-1, -1), &mut ConvoPin::default());
    let (boxes, _) = convo_layout(&c, &mut ConvoPin::default());
    assert!(boxes.len() >= c.options.len(), "both questions are buttons");
    g.command(Command::Decide(1));
    assert!(g.seats[0].frame_offer.is_none(), "the pick answers it");
}

/// The box pins what it drew (#1317): a new line appends below, a drop
/// scrolls the survivors as one block, and a note slid in above the fold
/// never shows
#[test]
fn test_the_conversation_box_pins_its_lines() {
    let row = |i: usize| TalkRowV {
        name: Some(format!("Speaker{}", i)),
        text: format!("line {}", i),
        gist: false, you: false, rule: false, who: Some(i),
    };
    // under capacity: an appended line lands below, nothing moves
    let mut p = ConvoPanelV {
        title: "talk".into(),
        rows: (0..10).map(row).collect(),
        options: vec![], decides: false, leave: false,
    };
    let mut pin = ConvoPin::default();
    let (boxes, _) = convo_layout(&p, &mut pin);
    p.rows.push(row(10));
    let (boxes2, _) = convo_layout(&p, &mut pin);
    assert_eq!(boxes2.len(), boxes.len() + 1, "the new line appended");
    for (a, b) in boxes.iter().zip(boxes2.iter()) { assert_eq!(a.y, b.y, "a drawn line moved"); }

    // past capacity: the oldest kept line drops off the top and every
    // survivor sits exactly its own height higher - a rigid scroll
    let mut p = ConvoPanelV {
        title: "talk".into(),
        rows: (0..60).map(row).collect(),
        options: vec![], decides: false, leave: false,
    };
    let mut pin = ConvoPin::default();
    let (boxes, _) = convo_layout(&p, &mut pin);
    let head = boxes[0].rows[0].1.clone();
    let drop = boxes[0].h + 3;
    p.rows.push(row(60));
    let (boxes2, _) = convo_layout(&p, &mut pin);
    assert_eq!(boxes2.len(), boxes.len(), "a line still shows per drop");
    assert_ne!(boxes2[0].rows[0].1, head, "the oldest line didn't drop");
    assert_eq!(boxes2[0].rows[0].1, boxes[1].rows[0].1, "the drop wasn't the oldest");
    for (a, b) in boxes.iter().skip(1).zip(boxes2.iter()) {
        assert_eq!(a.y - b.y, drop, "a survivor didn't scroll with the block");
    }

    // a gist note slid in under the top stays invisible: the pin follows
    // the row it already showed, not the index it sat at - the visible
    // set comes out identical, nothing scrolls for it
    p.rows.insert(1, TalkRowV {
        name: None, text: "a whisper over the talk".into(),
        gist: true, you: false, rule: false, who: None,
    });
    let (boxes3, _) = convo_layout(&p, &mut pin);
    assert!(boxes3.iter().all(|b| b.rows.iter().all(|(_, t, _, _)| !t.contains("whisper"))),
        "a line inserted above the fold showed");
    assert_eq!(boxes3[0].rows[0].1, boxes2[0].rows[0].1,
        "the insert moved the fold off the line it was showing");
    for (a, b) in boxes3.iter().zip(boxes2.iter()) {
        assert_eq!(a.y, b.y, "a shown line moved for a line it can't see");
    }

    // a different panel starts the pin over at the top
    let q = ConvoPanelV {
        title: "another call".into(),
        rows: (0..3).map(row).collect(),
        options: vec![], decides: false, leave: false,
    };
    let (boxes4, _) = convo_layout(&q, &mut pin);
    assert_eq!(boxes4[0].rows[0].1, "line 0", "a new panel didn't open on its first line");
}

#[test]
fn test_the_testimony_rows_decide_the_story() {
    let mut g = Game::new_seeded(MAP_SEED);
    g.seats[0].testimony_offer = Some((g.tick, vec!["the bailiff hanged Bram.".into(), "the mill burned.".into()]));
    let c = g.convo_panel_at(0).expect("which story to tell");
    assert_eq!(c.title, "THE CALL - WHAT WILL YOU TELL");
    let mut buf = Buffer::new(game2_core::session::W, game2_core::session::H);
    draw_convo_panel(&mut buf, &c, (-1, -1), &mut ConvoPin::default());
    assert!(convo_layout(&c, &mut ConvoPin::default()).0.len() >= 2, "both stories are buttons");
    match digit_cmd(Some(&c), 1) {
        Some(Command::Decide(1)) => {}
        other => panic!("a digit on the stories picked: {:?}", other),
    }
    g.command(Command::Decide(1));
    assert!(g.seats[0].testimony_offer.is_none(), "the pick answers it");
}
