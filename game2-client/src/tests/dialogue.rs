//! Conversations: the rules cascade in play, the talk log, and the TALK and conversation panels.
use game2_core::session::*;
use game2_core::engine::ecs::Entity;
use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::Sprite;
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
        rows: (0..6).map(|k| TalkRowV { name: Some(format!("Speaker{}", k)), text: long.clone(), gist: k == 2, you: k % 2 == 1, rule: false, who: None, mood: 0 }).collect(),
        options: vec![long.clone(), "Short one.".into()],
        in_talk: true,
    };
    let (rows, rects) = talk_layout(&p);
    for (y, text, _, name, _) in &rows {
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
    let (boxes, _) = convo_layout(&c, &mut ConvoPin::default(), 30);
    let reply = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Reply(Some(0)))))
        .expect("a reply button");
    assert_eq!(convo_click(&c, reply.x + 2, reply.y + 2, &mut ConvoPin::default(), 30), Some(ConvoClick::Reply(Some(0))));
    // and the villager's own line selects them
    let line = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Speaker(_))))
        .expect("a speaker's line");
    match convo_click(&c, line.x + 2, line.y + 2, &mut ConvoPin::default(), 30) {
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
    let (boxes, _) = convo_layout(&c, &mut ConvoPin::default(), 30);
    let rest = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Decide(3))))
        .expect("the let-it-rest button");
    assert_eq!(convo_click(&c, rest.x + 2, rest.y + 2, &mut ConvoPin::default(), 30), Some(ConvoClick::Decide(3)));
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
fn test_escape_routes_off_the_real_boxes() {
    let mut g = Game::new_seeded(MAP_SEED);
    // nothing up: a bare Escape is the menu's
    assert_eq!(crate::esc_route(&g.ui_model()), crate::EscRoute::Pause);
    // a Talk under way: its leave, never the menu
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    assert_eq!(crate::esc_route(&g.ui_model()), crate::EscRoute::TalkLeave);
    g.command(Command::TalkPick(None));
    // the founding call a meeting leaves: a box that isn't a Talk
    // holds the press instead of the menu opening over it
    org_ready(&mut g);
    let came = g.villagers[..3].to_vec();
    let id = g.open_meeting_record(g.player, &came);
    g.resolve_meeting(&came, 0, id, (false, false, false, false, false));
    assert_eq!(crate::esc_route(&g.ui_model()), crate::EscRoute::Swallow);
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
            TalkRowV { name: Some("Eliza".into()), text: "word ".repeat(30), gist: false, you: false, rule: false, who: Some(2), mood: 0 },
            TalkRowV { name: None, text: "The room leans to the taxes - your call.".into(), gist: true, you: false, rule: false, who: None, mood: 0 },
        ],
        options: vec!["Take it on - a burden shared".into(), "Let it rest".into()],
        decides: true, leave: false,
    };
    let mut buf = Buffer::new(game2_core::session::W, game2_core::session::H);
    draw_convo_panel(&mut buf, &p, (-1, -1), &mut ConvoPin::default(), 30, &|_| None);
    let (boxes, h) = convo_layout(&p, &mut ConvoPin::default(), 30);
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
    draw_convo_panel(&mut buf, &c, (-1, -1), &mut ConvoPin::default(), 30, &|_| None);
    let (boxes, _) = convo_layout(&c, &mut ConvoPin::default(), 30);
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
        gist: false, you: false, rule: false, who: Some(i), mood: 0,
    };
    // under capacity: an appended line lands below, nothing moves
    let mut p = ConvoPanelV {
        title: "talk".into(),
        rows: (0..10).map(row).collect(),
        options: vec![], decides: false, leave: false,
    };
    let mut pin = ConvoPin::default();
    let (boxes, _) = convo_layout(&p, &mut pin, 30);
    p.rows.push(row(10));
    let (boxes2, _) = convo_layout(&p, &mut pin, 30);
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
    let (boxes, _) = convo_layout(&p, &mut pin, 30);
    let head = boxes[0].rows[0].1.clone();
    let drop = boxes[0].h + 3;
    p.rows.push(row(60));
    let (boxes2, _) = convo_layout(&p, &mut pin, 30);
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
        gist: true, you: false, rule: false, who: None, mood: 0,
    });
    let (boxes3, _) = convo_layout(&p, &mut pin, 30);
    assert!(boxes3.iter().all(|b| b.rows.iter().all(|(_, t, _, _, _)| !t.contains("whisper"))),
        "a line inserted above the fold showed");
    assert_eq!(boxes3[0].rows[0].1, boxes2[0].rows[0].1,
        "the insert moved the fold off the line it was showing");
    for (a, b) in boxes3.iter().zip(boxes2.iter()) {
        assert_eq!(a.y, b.y, "a shown line moved for a line it can't see");
    }

    // choices take room: when they go, what's scrolled off stays off -
    // the window doesn't reopen behind them (the old newest-fit pass
    // resurrected lines and jumped every y)
    let mut p = ConvoPanelV {
        title: "talk".into(),
        rows: (0..60).map(row).collect(),
        options: vec!["a much longer option label that wraps to several rows inside the box".into(); 4],
        decides: false, leave: true,
    };
    let mut pin = ConvoPin::default();
    let (boxes, _) = convo_layout(&p, &mut pin, 30);
    let lines1 = boxes.iter().filter(|b| !b.choice).count();
    p.options.clear();
    p.leave = false;
    let (boxes2, _) = convo_layout(&p, &mut pin, 30);
    assert_eq!(boxes2.len(), lines1, "dropped lines came back");
    assert_eq!(boxes2[0].rows[0].1, boxes[0].rows[0].1,
        "the fold reopened behind the choices");

    // a different panel starts the pin over at the top
    let q = ConvoPanelV {
        title: "another call".into(),
        rows: (0..3).map(row).collect(),
        options: vec![], decides: false, leave: false,
    };
    let (boxes4, _) = convo_layout(&q, &mut pin, 30);
    // a named opener caps its lead ("PEAKER0: ") - the words stay whole
    assert_eq!(boxes4[0].rows[0].1, "line 0", "a new panel didn't open on its first line");
}

/// New lines fade up over CONVO_FADE frames while their elders hold
/// full colour (#1318); the box's shape never reflows for it
#[test]
fn test_new_lines_fade_in_as_theyre_said() {
    let row = |i: usize| TalkRowV {
        name: Some(format!("Speaker{}", i)),
        text: format!("line {}", i),
        gist: false, you: false, rule: false, who: Some(i), mood: 0,
    };
    let mut p = ConvoPanelV {
        title: "talk".into(),
        rows: (0..3).map(row).collect(),
        options: vec!["a reply".into()], decides: false, leave: false,
    };
    let mut pin = ConvoPin::default();
    let (fresh, h1) = convo_layout(&p, &mut pin, 100);
    let (aged, h2) = convo_layout(&p, &mut pin, 100 + CONVO_FADE);
    assert_eq!(h1, h2, "the fade reflowed the box");
    // drawn: the same panel at first sight vs faded in differs in pixels,
    // and nowhere outside the said lines' rows
    let draw = |pin: &mut ConvoPin, frame: u32| {
        let mut buf = Buffer::new(game2_core::session::W, game2_core::session::H);
        draw_convo_panel(&mut buf, &p, (-1, -1), pin, frame, &|_| None);
        buf
    };
    let mut pin2 = ConvoPin::default();
    let b0 = draw(&mut pin2, 200);
    let b1 = draw(&mut pin2, 200 + CONVO_FADE);
    let rows_hi = fresh.iter().rev().find(|b| !b.choice).map(|b| b.y + b.h).unwrap_or(0);
    let (mut diff_in, mut diff_out) = (0, 0);
    for y in 0..game2_core::session::H as i32 {
        for x in CONVO_X..CONVO_X + CONVO_W {
            let i = (y as usize * b0.width + x as usize) * 4;
            if b0.pixels[i..i + 3] != b1.pixels[i..i + 3] {
                if y < rows_hi { diff_in += 1; } else { diff_out += 1; }
            }
        }
    }
    assert!(diff_in > 0, "the said lines didn't fade in the drawn frame");
    assert_eq!(diff_out, 0, "the choices or the frame dimmed too");
    // a said line dims at first sight and fills to the body colour
    let d = fresh[0].rows[0].2;
    let f = aged[0].rows[0].2;
    assert!(d.0 < f.0 && d.1 < f.1 && d.2 < f.2, "a line didn't fade up: {:?} -> {:?}", d, f);
    assert_eq!(f, (215, 215, 200), "the full colour isn't the body colour");
    // the speaker's name fades with its line
    assert!(fresh[0].rows[0].3.as_ref().unwrap().1.0 < aged[0].rows[0].3.as_ref().unwrap().1.0,
        "the name didn't fade with its line");
    // the reply button never dims
    let choice = fresh.iter().find(|b| b.choice).expect("a reply button");
    assert_eq!(choice.rows[0].2, (140, 230, 140), "a choice faded like a said line");
    // a line landing later fades alone - its elders stay full
    p.rows.push(row(3));
    let (boxes, _) = convo_layout(&p, &mut pin, 100 + CONVO_FADE + 1);
    let new = boxes.iter().rev().find(|b| !b.choice).unwrap();
    assert_eq!(new.rows[0].2, d, "the new line didn't start dim");
    assert_eq!(boxes[0].rows[0].2, f, "an elder line dimmed with it");
}

/// A named speaker's line carries their head in the slot beside the
/// name lead and wraps a touch narrower for it; a line nobody owns
/// keeps the plain lead (#1319)
#[test]
fn test_a_speaker_line_carries_their_portrait() {
    // an opaque head stands in for the art's stills
    let mut px = vec![0u8; 10 * 10 * 4];
    for i in 0..10 * 10 { px[i * 4] = 220; px[i * 4 + 3] = 255; }
    let head = Sprite { width: 10, height: 10, pixels: px };
    let p = ConvoPanelV {
        title: "talk".into(),
        rows: vec![
            TalkRowV { name: None, text: "the town talks the matter over".into(),
                gist: false, you: false, rule: false, who: None, mood: 0 },
            TalkRowV { name: Some("Eliza".into()),
                text: "the miller took an extra scoop of flour, they saw it plain".into(),
                gist: false, you: false, rule: false, who: Some(3), mood: 0 },
            TalkRowV { name: None, text: "a line nobody owns keeps its plain lead".into(),
                gist: false, you: false, rule: false, who: None, mood: 0 },
        ],
        options: vec!["say so".into()], decides: false, leave: false,
    };
    let mut with = Buffer::new(game2_core::session::W, game2_core::session::H);
    draw_convo_panel(&mut with, &p, (-1, -1), &mut ConvoPin::default(), 30, &|_| Some(&head));
    let mut without = Buffer::new(game2_core::session::W, game2_core::session::H);
    draw_convo_panel(&mut without, &p, (-1, -1), &mut ConvoPin::default(), 30, &|_| None);
    let (boxes, _) = convo_layout(&p, &mut ConvoPin::default(), 30);
    let said = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Speaker(_)))).expect("a speaker's line");
    let px_at = |b: &Buffer, x: i32, y: i32| { let i = (y as usize * b.width + x as usize) * 4; b.pixels[i..i + 3].to_vec() };
    // the head lands in the slot beside the lead
    let mut head_px = 0;
    for y in said.y + 2..said.y + 2 + CONVO_PORT {
        for x in said.x + 3..said.x + 3 + CONVO_PORT - 4 {
            if px_at(&with, x, y) != px_at(&without, x, y) { head_px += 1; }
        }
    }
    assert!(head_px > 10, "no head drew in the speaker slot");
    // and the words hang off it: the gap between icon and lead stays
    // box-dark in the icon frame, words in the plain one
    let bg = px_at(&with, said.x + 30, said.y + 2);
    for x in said.x + CONVO_PORT..said.x + CONVO_PORT + 4 {
        assert_eq!(px_at(&with, x, said.y + 5), bg, "words started inside the slot");
    }
    // the speaker's row wraps a touch narrower for the slot: the lead's
    // rest can't outrun the shrunken width
    assert!(said.rows[0].1.len() <= CONVO_CHARS - CONVO_PORT_CHARS - "Eliza: ".len(),
        "the speaker's lead didn't narrow for the head: {:?}", said.rows[0].1);
    // a line nobody owns keeps the plain lead: identical pixels
    let plain = boxes.iter().find(|b| b.click.is_none() && !b.choice).expect("an unowned line");
    for y in plain.y..plain.y + plain.h {
        for x in plain.x..plain.x + plain.w {
            assert_eq!(px_at(&with, x, y), px_at(&without, x, y), "an unowned line shifted for the icon");
        }
    }
}

#[test]
fn test_the_testimony_rows_decide_the_story() {
    let mut g = Game::new_seeded(MAP_SEED);
    g.seats[0].testimony_offer = Some((g.tick, vec!["the bailiff hanged Bram.".into(), "the mill burned.".into()]));
    let c = g.convo_panel_at(0).expect("which story to tell");
    assert_eq!(c.title, "THE CALL - WHAT WILL YOU TELL");
    let mut buf = Buffer::new(game2_core::session::W, game2_core::session::H);
    draw_convo_panel(&mut buf, &c, (-1, -1), &mut ConvoPin::default(), 30, &|_| None);
    assert!(convo_layout(&c, &mut ConvoPin::default(), 30).0.len() >= 2, "both stories are buttons");
    match digit_cmd(Some(&c), 1) {
        Some(Command::Decide(1)) => {}
        other => panic!("a digit on the stories picked: {:?}", other),
    }
    g.command(Command::Decide(1));
    assert!(g.seats[0].testimony_offer.is_none(), "the pick answers it");
}

/// The box draws the speaker's own head still beside their lead - the
/// still their figure wears on the map, resolved the same sexed-token
/// way (#1319)
#[test]
fn test_the_conversation_box_draws_the_speakers_portrait() {
    let mut g = Game::new_seeded(MAP_SEED);
    let i = adults(&g)[0];
    talk_beside(&mut g, i);
    let m = g.ui_model();
    let view = crate::View::new(crate::Art::load_fs("../assets", 0));
    let buf = view.render(&m, (-1, -1));
    let c = m.convo.as_ref().expect("the talk on the conversation box");
    let (boxes, _) = convo_layout(c, &mut ConvoPin::default(), m.frame);
    let said = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Speaker(v)) if v == i))
        .expect("the speaker's line");
    // the slot holds head pixels, not the box's dark ground
    let mut lit = 0;
    for y in said.y + 2..said.y + 2 + CONVO_PORT - 4 {
        for x in said.x + 3..said.x + 3 + CONVO_PORT - 4 {
            let p = (y as usize * buf.width + x as usize) * 4;
            if buf.pixels[p..p + 3] != [31, 31, 47] { lit += 1; }
        }
    }
    assert!(lit > 10, "no portrait drew in the speaker's slot");
    // and every look's head carries pixels - pack cells stand their
    // figure low in the frame, so a top-of-sprite crop there reads
    // blank (#1319, review find)
    let art = crate::Art::load_fs("../assets", 0);
    for (k, h) in art.villager_heads.iter().enumerate() {
        assert!(h.pixels.chunks(4).any(|p| p[3] > 128), "villager head {} is blank", k);
    }
}

/// The mood mark: a badge pip on the head slot's bottom corner - gold
/// warm, steel-blue cold, red angry; a neutral read draws none (#1320)
#[test]
fn test_the_speakers_mood_mark_badges_their_icon() {
    // an opaque head stands in for the art's stills
    let mut px = vec![0u8; 10 * 10 * 4];
    for i in 0..10 * 10 { px[i * 4] = 220; px[i * 4 + 3] = 255; }
    let head = Sprite { width: 10, height: 10, pixels: px };
    let panel = |mood: u8| ConvoPanelV {
        title: "talk".into(),
        rows: vec![TalkRowV { name: Some("Eliza".into()),
            text: "the miller took an extra scoop of flour".into(),
            gist: false, you: false, rule: false, who: Some(3), mood }],
        options: vec![], decides: false, leave: false,
    };
    let draw = |p: &ConvoPanelV| {
        let mut buf = Buffer::new(game2_core::session::W, game2_core::session::H);
        draw_convo_panel(&mut buf, p, (-1, -1), &mut ConvoPin::default(), 30, &|_| Some(&head));
        buf
    };
    // the mark rides the box off the row
    let p = panel(1);
    let (boxes, _) = convo_layout(&p, &mut ConvoPin::default(), 30);
    assert_eq!(boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Speaker(_)))).unwrap().mood, 1);
    let at = |b: &Buffer, x: i32, y: i32| {
        let i = (y as usize * b.width + x as usize) * 4; [b.pixels[i], b.pixels[i + 1], b.pixels[i + 2]]
    };
    let said = boxes.iter().find(|b| b.click.is_some()).unwrap();
    let (bx, by) = (said.x + 9, said.y + 8);
    assert_eq!(at(&draw(&panel(1)), bx, by), [236, 178, 88], "warm isn't gold");
    assert_eq!(at(&draw(&panel(2)), bx, by), [142, 176, 220], "cold isn't steel blue");
    assert_eq!(at(&draw(&panel(3)), bx, by), [222, 82, 62], "angry isn't red");
    // a neutral read draws nothing: the head's own pixels show through
    let plain = draw(&panel(0));
    assert_eq!(at(&plain, bx, by), [220, 0, 0], "a badge drew on a neutral read");
    // and the pip's dark frame hangs it off the head
    assert_eq!(at(&draw(&panel(1)), said.x + 7, said.y + 6), [15, 15, 22]);
}

/// The opener's first letter draws as a decorated drop cap beside its
/// first rows; a day rule isn't the opener, your own words take no cap,
/// and the box stands taller for the cap's span (#1321)
#[test]
fn test_the_opener_carries_a_drop_cap() {
    let p = ConvoPanelV {
        title: "talk".into(),
        rows: vec![
            TalkRowV { name: None, text: "day 12".into(), gist: false, you: false, rule: true, who: None, mood: 0 },
            TalkRowV { name: Some("Rowena".into()),
                text: "Peace be upon thy journey, traveler. Who art thou?".into(),
                gist: false, you: false, rule: false, who: Some(4), mood: 0 },
            TalkRowV { name: Some("You".into()), text: "Well met.".into(),
                gist: false, you: true, rule: false, who: None, mood: 0 },
        ],
        options: vec![], decides: false, leave: false,
    };
    let (boxes, _) = convo_layout(&p, &mut ConvoPin::default(), 30);
    let opener = boxes.iter().find(|b| b.cap.is_some()).expect("the opener caps");
    assert_eq!(opener.cap, Some('R'), "the day rule's no opener - the first spoken line is");
    // a named opener caps its lead word: "OWENA:" beside the cap, the
    // greeting kept whole after it
    assert_eq!(opener.rows[0].3.as_ref().unwrap().0, "OWENA: ");
    assert_eq!(opener.rows[0].1.trim_start().chars().take(5).collect::<String>(), "Peace");
    // the box stands taller than one row for the cap's span
    assert!(opener.h >= CONVO_CAP + 8, "the cap's span didn't raise the box: {}", opener.h);
    // and it wraps narrower beside it
    assert!(opener.rows[0].1.len() <= CONVO_CHARS - CONVO_PORT_CHARS - CAP_CHARS,
        "the opener didn't narrow for the cap: {:?}", opener.rows[0].1);
    // an unowned opener caps its first word instead
    let q = ConvoPanelV { title: "talk".into(),
        rows: vec![TalkRowV { name: None, text: "folk gather on the square".into(),
            gist: false, you: false, rule: false, who: None, mood: 0 }],
        options: vec![], decides: false, leave: false };
    let (boxes, _) = convo_layout(&q, &mut ConvoPin::default(), 30);
    assert_eq!(boxes[0].cap, Some('f'));
    assert_eq!(boxes[0].rows[0].1, "olk gather on the square");
    // your own words take no cap even as the first line
    let q = ConvoPanelV { title: "talk".into(),
        rows: vec![TalkRowV { name: Some("You".into()), text: "Aye, mine own words.".into(),
            gist: false, you: true, rule: false, who: None, mood: 0 }],
        options: vec![], decides: false, leave: false };
    assert!(convo_layout(&q, &mut ConvoPin::default(), 30).0.iter().all(|b| b.cap.is_none()),
        "your words capped");
    // draw-level: the decorated box's gold edge lands where the cap is
    let mut buf = Buffer::new(game2_core::session::W, game2_core::session::H);
    draw_convo_panel(&mut buf, &p, (-1, -1), &mut ConvoPin::default(), 30, &|_| None);
    let px_at = |x: i32, y: i32| { let i = (y as usize * buf.width + x as usize) * 4;
        [buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]] };
    let wx = CONVO_X + 10 + CONVO_PORT;   // the opener's a speaker: words after the head slot
    assert_eq!(px_at(wx + 10, opener.y + 1), [172, 136, 68], "no gold edge on the cap");
}

/// The tiers read at their own weights (#1321): the title doubled, a
/// day rule tracked capitals, a name lead capital - body rows stay 1x
#[test]
fn test_the_convo_box_tiers_its_text() {
    let p = ConvoPanelV {
        title: "TALK - ROWENA".into(),
        rows: vec![
            TalkRowV { name: None, text: "day 12".into(), gist: false, you: false, rule: true, who: None, mood: 0 },
            TalkRowV { name: Some("Eliza".into()), text: "the miller took an extra scoop".into(),
                gist: false, you: false, rule: false, who: Some(3), mood: 0 },
        ],
        options: vec![], decides: false, leave: false,
    };
    // the name lead goes up capital; the rule row marks itself rubric
    let (boxes, _) = convo_layout(&p, &mut ConvoPin::default(), 30);
    let rule = boxes.iter().find(|b| b.rows.iter().any(|r| r.4)).expect("no rubric row");
    assert_eq!(rule.rows[0].1, "day 12");
    let said = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Speaker(_)))).unwrap();
    // the opener caps her E: the lead shows its capital rest
    assert_eq!(said.rows[0].3.as_ref().unwrap().0, "LIZA: ", "the lead isn't capital");
    // a doubled title inks the second glyph row - plain 1x can't reach it
    let mut buf = Buffer::new(game2_core::session::W, game2_core::session::H);
    draw_convo_panel(&mut buf, &p, (-1, -1), &mut ConvoPin::default(), 30, &|_| None);
    let mut ink = 0;
    for y in CONVO_Y + 10..CONVO_Y + 16 {
        for x in CONVO_X + 6..CONVO_X + 100 {
            let i = (y as usize * buf.width + x as usize) * 4;
            if buf.pixels[i] == 150 { ink += 1; }
        }
    }
    assert!(ink > 5, "the title didn't draw doubled");
    // the rubric row draws tracked capitals: wider than its plain length
    let (rows, _) = talk_layout(&TalkPanelV { title: "t".into(), rows: vec![
        TalkRowV { name: None, text: "day 12".into(), gist: false, you: false, rule: true, who: None, mood: 0 }],
        options: vec![], in_talk: false });
    let mut buf = Buffer::new(game2_core::session::W, game2_core::session::H);
    let y0 = rows[0].0;
    draw_talk_panel(&mut buf, &TalkPanelV { title: "t".into(), rows: vec![
        TalkRowV { name: None, text: "day 12".into(), gist: false, you: false, rule: true, who: None, mood: 0 }],
        options: vec![], in_talk: false }, (-1, -1));
    let mut right = 0;
    for x in TALK_X + 6..TALK_X + 80 {
        for yy in y0..y0 + ROW_H {
            let i = (yy as usize * buf.width + x as usize) * 4;
            if buf.pixels[i] == 110 { right = x; }
        }
    }
    // plain "day 12" ends ~31px in; the tracked capitals reach past 38
    assert!(right - TALK_X - 6 >= 38, "the rubric didn't draw tracked: {}", right);
}
