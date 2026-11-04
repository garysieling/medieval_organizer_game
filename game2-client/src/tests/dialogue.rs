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
    let msg = g.resolve_meeting(&came, 0, id);
    // the room leans, but nothing is founded until the organizer says so
    assert!(g.seats[0].pending_founding.is_some(), "{}", msg);
    assert!(game2_core::game::campaign::campaign_of(&g.world, g.org).is_none());
    let c = g.convo_panel_at(0).expect("the call on the box");
    assert!(c.decides && c.options.len() == 4, "{:?}", c.options);
    let issue = game2_core::game::desires::issue_name("the taxes".into());
    assert!(c.options.iter().take(3).all(|o| o.contains(&issue)), "{:?} vs {}", c.options, issue);
    // letting it rest answers the call and founds nothing
    let (boxes, _) = convo_layout(&c);
    let rest = boxes.iter().find(|b| matches!(b.click, Some(ConvoClick::Decide(3))))
        .expect("the let-it-rest button");
    assert_eq!(convo_click(&c, rest.x + 2, rest.y + 2), Some(ConvoClick::Decide(3)));
    g.decide_at(0, 3);
    assert!(g.seats[0].pending_founding.is_none());
    assert!(game2_core::game::campaign::campaign_of(&g.world, g.org).is_none());
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
    draw_convo_panel(&mut buf, &p, (-1, -1));
    let (boxes, h) = convo_layout(&p);
    for b in &boxes {
        assert!(b.y >= CONVO_Y && b.y + b.h <= CONVO_Y + h, "a button sits inside the box");
        assert!(b.x >= CONVO_X && b.x + b.w <= CONVO_X + CONVO_W);
    }
}
