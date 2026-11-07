//! Village mysteries: a question doing the rounds, the gossip learned a
//! fact at a time in talk, the solve carried back to whoever asked -
//! and they remember it
use crate::session::*;
use crate::session::mystery::SeatMystery;
use crate::game::mysteries::{self, FACTS, MYSTERIES};
use crate::game::relationships::{find_relationship, Relationship, RelKind};

/// A grown villager of the village - old enough to be asking after things
fn askers(g: &Game) -> Vec<Entity> {
    g.villagers.iter().copied().filter(|&v| {
        g.world.get_component::<crate::game::vocation::Kid>(v).is_none()
            && !crate::game::villager::is_dead(&g.world, v)
            && g.world.get_component::<crate::session::country::Outlander>(v).is_none()
    }).collect()
}

#[test]
fn test_a_mystery_starts_doing_the_rounds() {
    let mut g = Game::new();
    let mut rng = Rng(11);
    // the daily chance is small - run enough dawns that one lands
    for _ in 0..300 {
        g.mystery_dawn(g.tick, &mut rng);
        if g.seats[0].mystery.is_some() { break; }
    }
    let m = g.seats[0].mystery.as_ref().expect("a mystery should be doing the rounds");
    assert!(m.def < MYSTERIES.len(), "the mystery's one of the written ones");
    assert!(g.villagers.contains(&m.asker), "and somebody's asking after it");
    assert_eq!(m.heard(), 0, "nothing heard yet");
    assert!(g.log.iter().any(|l| l.contains("asking after")),
        "the log says who's asking: {:?}", g.log.iter().rev().take(3).collect::<Vec<_>>());
}

#[test]
fn test_talk_now_and_then_passes_on_a_fact() {
    let mut g = Game::new();
    let folk = askers(&g);
    assert!(folk.len() >= 2, "enough folk to talk to");
    g.seats[0].mystery = Some(SeatMystery { def: 0, known: 0, asker: folk[0] });
    // talk to folk until the gossip's given up all five facts
    let mut heard = 0;
    'talks: for _ in 0..2000 {
        for &v in &folk {
            let who = g.villagers.iter().position(|&x| x == v).unwrap();
            g.talk_to_at(0, who);
            let m = g.seats[0].mystery.as_ref().unwrap();
            if m.heard() > heard {
                heard = m.heard();
                if m.solved() { break 'talks; }
            }
        }
    }
    let m = g.seats[0].mystery.as_ref().unwrap();
    assert!(m.solved(), "five facts and it's whole - got {}", m.heard());
    assert!(g.toasts.iter().any(|(t, ..)| t.contains("Mystery solved")),
        "the solve is a notification: {:?}", g.toasts);
    // and the panel says how much of it's in
    let line = g.mystery_panel_line(0).expect("the panel names the mystery");
    assert!(line.contains(&mysteries::MYSTERIES[m.def].title) && line.contains("5/5"),
        "the line reads the question and the count: {}", line);
}

#[test]
fn test_telling_the_asker_makes_them_a_friend() {
    let mut g = Game::new();
    let folk = askers(&g);
    let asker = folk[0];
    g.seats[0].mystery = Some(SeatMystery {
        def: 1, known: (1 << FACTS) - 1, asker });
    // picked and beside the asker, the card offers to tell them
    g.seats[0].picked = true;
    g.seats[0].selected_target = g.villagers.iter().position(|&v| v == asker).unwrap();
    let ap = *g.world.get_component::<Pos>(asker).unwrap();
    if let Some(p) = g.world.get_component_mut::<Pos>(g.player) { p.x = ap.x; p.y = ap.y; }
    g.update_ctx_actions_at(0);
    assert!(g.seats[0].ctx_actions.contains(&CtxAct::ShareFindings),
        "the delivery's on the card: {:?}", g.seats[0].ctx_actions);
    let i = g.seats[0].ctx_actions.iter().position(|&a| a == CtxAct::ShareFindings).unwrap();
    g.ctx_act_at(0, i);
    assert!(g.seats[0].mystery.is_none(), "told, the mystery's done");
    let kind = find_relationship(&g.world, g.player, asker)
        .and_then(|r| g.world.get_component::<Relationship>(r)).map(|r| r.kind);
    assert_eq!(kind, Some(RelKind::Friend), "they thank you and you're friends");
    assert!(g.log.iter().any(|l| l.contains("pieced together")),
        "the log says what was told: {:?}", g.log.iter().rev().take(3).collect::<Vec<_>>());
    // and another mystery can start once the wait's up
    assert!(g.seats[0].mystery_wait > g.tick, "a spell before the next one");
}

#[test]
fn test_the_question_passes_on_when_the_asker_is_gone() {
    let mut g = Game::new();
    let folk = askers(&g);
    let asker = folk[0];
    g.seats[0].mystery = Some(SeatMystery {
        def: 0, known: (1 << FACTS) - 1, asker });
    // the asker dies - telling it to them is refused
    g.world.add_component(asker, crate::game::villager::Dead);
    g.seats[0].picked = true;
    g.seats[0].selected_target = g.villagers.iter().position(|&v| v == asker).unwrap();
    let msg = g.mystery_deliver_at(0);
    assert!(msg.contains("gone from the village"), "{}", msg);
    assert!(g.seats[0].mystery.is_some(), "the mystery's still under way");
    // and at dawn the question finds new ears - what was heard still counts
    let mut rng = Rng(5);
    g.mystery_dawn(g.tick, &mut rng);
    let m = g.seats[0].mystery.as_ref().expect("the mystery lives on");
    assert_ne!(m.asker, asker, "somebody else is asking now");
    assert!(g.villagers.contains(&m.asker) && !crate::game::villager::is_dead(&g.world, m.asker),
        "and they're somebody who can still hear the answer");
    assert_eq!(m.heard(), FACTS, "what was heard still counts");
}

#[test]
fn test_the_same_question_doesnt_come_round_twice() {
    let mut g = Game::new();
    let mut rng = Rng(9);
    // the last-told mystery sits out of the next draw: no two deals in
    // a row pick the same one
    let mut last_dealt: Option<usize> = None;
    let mut deals = 0;
    for _ in 0..400 {
        g.mystery_dawn(g.tick, &mut rng);
        if let Some(m) = &g.seats[0].mystery {
            if let Some(p) = last_dealt {
                assert_ne!(m.def, p, "the same question doesn't come round twice running");
            }
            last_dealt = Some(m.def);
            deals += 1;
            // clear it so the next dawn can deal another
            g.seats[0].mystery = None;
            g.seats[0].mystery_wait = 0;
        }
    }
    assert!(deals > 5, "enough dawns dealt a few: {}", deals);
}

#[test]
fn test_it_wont_be_told_to_the_wrong_person() {
    let mut g = Game::new();
    let folk = askers(&g);
    let (asker, other) = (folk[0], folk[1]);
    g.seats[0].mystery = Some(SeatMystery {
        def: 2, known: (1 << FACTS) - 1, asker });
    // somebody else's card: no delivery on offer
    g.seats[0].picked = true;
    g.seats[0].selected_target = g.villagers.iter().position(|&v| v == other).unwrap();
    let op = *g.world.get_component::<Pos>(other).unwrap();
    if let Some(p) = g.world.get_component_mut::<Pos>(g.player) { p.x = op.x; p.y = op.y; }
    g.update_ctx_actions_at(0);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::ShareFindings),
        "only the asker gets it: {:?}", g.seats[0].ctx_actions);
    // and calling it anyway is refused
    let msg = g.mystery_deliver_at(0);
    assert!(msg.contains("weren't the one asking"), "{}", msg);
    assert!(g.seats[0].mystery.is_some(), "the mystery's still under way");
}
