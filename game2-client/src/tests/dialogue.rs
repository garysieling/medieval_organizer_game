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
    let mut pin = ConvoPin::default();
    let (boxes, _) = convo_layout(&c, &mut pin);
    let reply = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Reply(Some(0)))))
        .expect("a reply button");
    assert_eq!(convo_click(&c, reply.x + 2, reply.y + 2, &mut pin), Some(ConvoClick::Reply(Some(0))));
    // and the villager's own line selects them
    let line = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Speaker(_))))
        .expect("a speaker's line");
    match convo_click(&c, line.x + 2, line.y + 2, &mut pin) {
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
    let mut pin = ConvoPin::default();
    let (boxes, _) = convo_layout(&c, &mut pin);
    let rest = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Decide(3))))
        .expect("the let-it-rest button");
    assert_eq!(convo_click(&c, rest.x + 2, rest.y + 2, &mut pin), Some(ConvoClick::Decide(3)));
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
    let mut pin = ConvoPin::default();
    draw_convo_panel(&mut buf, &p, (-1, -1), &mut pin);
    let (boxes, h) = convo_layout(&p, &mut pin);
    for b in &boxes {
        assert!(b.y >= CONVO_Y && b.y + b.h <= CONVO_Y + h, "a button sits inside the box");
        assert!(b.x >= CONVO_X && b.x + b.w <= CONVO_X + CONVO_W);
    }
}

#[test]
fn test_the_conversation_box_appends_and_scrolls_without_jumping() {
    // one wrapped row per line, so every button is the same 19px pitch
    let line = |k: usize| TalkRowV {
        name: Some(format!("Speaker{}", k % 3)), text: format!("line {} said", k),
        gist: false, you: false, rule: false, who: Some(k),
    };
    let mut p = ConvoPanelV {
        title: "TALK - SOMEONE".into(), rows: vec![],
        options: vec!["A reply".into()], decides: false, leave: true,
    };
    let mut pin = ConvoPin::default();
    // the line boxes in row order: (row index, y, h)
    let ys = |p: &ConvoPanelV, pin: &mut ConvoPin| -> Vec<(usize, i32, i32)> {
        convo_layout(p, pin).0.into_iter()
            .filter(|b| !b.choice)
            .map(|b| match b.click {
                Some(ConvoClick::Speaker(k)) => (k, b.y, b.h),
                _ => unreachable!("every test line names its speaker"),
            })
            .collect()
    };
    // grow the conversation a line at a time: while the box has room
    // every line keeps the y it was first drawn at
    let mut before: Vec<(usize, i32)> = vec![];
    for k in 0..31 {
        p.rows.push(line(k));
        let now = ys(&p, &mut pin);
        for &(i, y) in &before {
            assert_eq!(now.iter().find(|(j, _, _)| *j == i).map(|(_, y, _)| *y), Some(y),
                "line {} moved while the box was still growing", i);
        }
        before = now.iter().map(|&(i, y, _)| (i, y)).collect();
    }
    // past the cap the oldest lines drop off the top: the kept rows stay
    // a contiguous suffix ending at the newest, the first never comes
    // back, and the survivors scroll as one block - same gaps, shifted
    // together, never re-laid out
    let mut first = 0usize;
    for k in 31..48 {
        p.rows.push(line(k));
        let now = ys(&p, &mut pin);
        assert_eq!(now.last().map(|r| r.0), Some(k), "the newest line is always shown");
        for w in now.windows(2) {
            assert_eq!(w[1].0, w[0].0 + 1, "the kept lines stay contiguous");
            assert_eq!(w[1].1 - w[0].1, 19, "the block keeps its pitch");
        }
        let deltas: Vec<i32> = now.iter().filter_map(|&(i, y, _)|
            before.iter().find(|(j, _)| *j == i).map(|(_, y0)| y - y0)).collect();
        if !deltas.is_empty() {
            assert!(deltas.iter().all(|&d| d == deltas[0]),
                "survivors moved as one block, not re-laid out");
        }
        assert!(now[0].0 >= first, "a dropped line came back");
        first = now[0].0;
        before = now.iter().map(|&(i, y, _)| (i, y)).collect();
    }
    // shrinking the choices doesn't reshow what already fell
    p.options.clear();
    p.leave = false;
    let now = ys(&p, &mut pin);
    assert_eq!(now[0].0, first, "lines that fell stayed fallen");
    // a different conversation starts from the top again - with the
    // extra room the choices freed, more of the tail fits than did
    // under the stale anchor
    p.title = "HOUSE MEETING - ELSEWHERE".into();
    let now = ys(&p, &mut pin);
    assert!(now[0].0 < first, "a new conversation re-anchors at the top");
}

/// For eyes: a Talk's box and a long scrolled one, rendered as
/// --dump-frame would. `CONVO_FRAME=<path.bmp> cargo test -p
/// game2-client --lib convo_pin_frames -- --ignored`
#[test]
#[ignore]
fn convo_pin_frames() {
    let Ok(path) = std::env::var("CONVO_FRAME") else { return };
    let mut g = Game::new_seeded(MAP_SEED);
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    let view = crate::View::new(crate::Art::load_fs("../assets", MAP_SEED));
    view.render(&g.ui_model(), (-1, -1)).save_bmp(&path).unwrap();
    // and the box tall and scrolled, the way a long Talk leaves it
    let line = |k: usize| TalkRowV {
        name: Some(format!("Speaker{}", k % 3)), text: format!("line {} said", k),
        gist: false, you: false, rule: false, who: Some(k),
    };
    let mut p = ConvoPanelV {
        title: "TALK - SOMEONE".into(),
        rows: (0..48).map(line).collect(),
        options: vec!["A reply".into(), "Another".into()], decides: false, leave: true,
    };
    let mut pin = ConvoPin::default();
    let mut buf = Buffer::new(game2_core::session::W, game2_core::session::H);
    // lay it out the way frames would: grown, then past the cap
    for _ in 0..3 { convo_layout(&p, &mut pin); }
    p.rows.push(line(48));
    draw_convo_panel(&mut buf, &p, (-1, -1), &mut pin);
    let long = path.replace(".bmp", "-scrolled.bmp");
    buf.save_bmp(&long).unwrap();
    println!("saved {} and {}", path, long);
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
    let mut pin = ConvoPin::default();
    draw_convo_panel(&mut buf, &c, (-1, -1), &mut pin);
    let (boxes, _) = convo_layout(&c, &mut pin);
    assert!(boxes.len() >= c.options.len(), "both questions are buttons");
    g.command(Command::Decide(1));
    assert!(g.seats[0].frame_offer.is_none(), "the pick answers it");
}

#[test]
fn test_the_testimony_rows_decide_the_story() {
    let mut g = Game::new_seeded(MAP_SEED);
    g.seats[0].testimony_offer = Some((g.tick, vec!["the bailiff hanged Bram.".into(), "the mill burned.".into()]));
    let c = g.convo_panel_at(0).expect("which story to tell");
    assert_eq!(c.title, "THE CALL - WHAT WILL YOU TELL");
    let mut buf = Buffer::new(game2_core::session::W, game2_core::session::H);
    let mut pin = ConvoPin::default();
    draw_convo_panel(&mut buf, &c, (-1, -1), &mut pin);
    assert!(convo_layout(&c, &mut pin).0.len() >= 2, "both stories are buttons");
    match digit_cmd(Some(&c), 1) {
        Some(Command::Decide(1)) => {}
        other => panic!("a digit on the stories picked: {:?}", other),
    }
    g.command(Command::Decide(1));
    assert!(g.seats[0].testimony_offer.is_none(), "the pick answers it");
}
