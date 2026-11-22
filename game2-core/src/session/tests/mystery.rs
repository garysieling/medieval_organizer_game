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
    // and the panel says how much of it's in, with the asker as data
    // so their name can be a button (#646)
    let mz = g.mystery_panel_line(0).expect("the panel names the mystery");
    assert_eq!(mz.title, mysteries::MYSTERIES[m.def].title);
    assert_eq!((mz.heard, mz.total), (FACTS, FACTS), "five of five whispers");
    assert_eq!(mz.asker, Some(g.villagers.iter().position(|&v| v == m.asker).unwrap()));
    assert_eq!(mz.asker_name, g.name_of(m.asker));
    assert!(mz.asker_at.is_some(), "the asker's spot rides along");
    // the solved line names the asker and carries their click-through
    // meta - entity and spot snapshotted when it was pushed (#646)
    let i = g.log.iter().position(|l| l.contains("that's all of it")).expect("the solve is logged");
    let line = &g.log[i];
    assert!(line.contains(&g.name_of(m.asker)), "the asker is named: {}", line);
    let meta = g.log.meta(i);
    let want = g.villagers.iter().position(|&v| v == m.asker).unwrap();
    assert_eq!(meta.who, Some(want), "clicking selects the asker");
    let ap = *g.world.get_component::<Pos>(m.asker).unwrap();
    assert_eq!(meta.at, Some((ap.x, ap.y)), "and jumps to where they stood");
}

#[test]
fn test_a_heard_fact_is_marked_in_the_moment() {
    let mut g = Game::new();
    let folk = askers(&g);
    g.seats[0].mystery = Some(SeatMystery { def: 0, known: 0, asker: folk[0] });
    // talk to folk until gossip gives up a fact - the moment's marked
    // on the talk itself (#1184)
    let mut marked = false;
    'talks: for _ in 0..2000 {
        for &v in &folk {
            let who = g.villagers.iter().position(|&x| x == v).unwrap();
            g.talk_to_at(0, who);
            let t = g.seats[0].talk.as_ref().expect("a talk's open");
            if let Some(n) = t.notes.iter().find(|n| n.contains("Worth remembering")) {
                assert!(n.contains("of 5 heard"), "the tally rides along: {}", n);
                marked = true;
                break 'talks;
            }
        }
    }
    assert!(marked, "a heard fact says so on the talk");
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


/// Stand beside villager `v` and Talk to them: the conversation opens on
/// the TALK panel
fn talk_to(g: &mut Game, v: Entity) {
    let i = g.villagers.iter().position(|&x| x == v).unwrap();
    let p = *g.world.get_component::<Pos>(v).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + 0.5, y: p.y };
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    g.act_at(0, 0);
}

fn solved(g: &mut Game, def: usize, asker: Entity) {
    g.seats[0].mystery = Some(SeatMystery { def, known: (1 << FACTS) - 1, asker });
}

/// The telling's pick on the open talk, if it's on offer
fn tell_pick(g: &Game) -> Option<usize> {
    g.seats[0].talk.as_ref()?.choice_labels().iter()
        .position(|l| l.starts_with("Tell them what you pieced together"))
}

#[test]
fn test_a_solved_mystery_is_told_in_the_talk() {
    let mut g = Game::new();
    let asker = askers(&g)[0];
    solved(&mut g, 1, asker);
    talk_to(&mut g, asker);
    let labels = g.seats[0].talk.as_ref().expect("the talk opens").choice_labels();
    let p = tell_pick(&g).unwrap_or_else(|| panic!("the asker's talk offers the telling: {:?}", labels));
    assert_eq!(p, labels.len() - 1, "it rides last, so the other picks keep their numbers");
    assert!(labels[p].contains(MYSTERIES[1].title), "{}", labels[p]);
    g.talk_pick_at(0, Some(p));
    assert!(g.seats[0].mystery.is_none(), "told, the mystery's done");
    let t = g.seats[0].talk.as_ref().expect("the talk goes on after").clone();
    assert!(tell_pick(&g).is_none(), "told once: {:?}", t.choice_labels());
    let rec = g.talk_log.iter().find(|r| r.id == t.id).unwrap();
    let said: Vec<&str> = rec.lines.iter().map(|l| l.text.as_str()).collect();
    assert!(rec.lines.iter().any(|l| l.who == Some(asker) && l.text == MYSTERIES[1].wonders),
        "they take the truth in, on the record: {:?}", said);
    assert!(rec.lines.iter().any(|l| l.who == Some(asker) && l.text == MYSTERIES[1].thanks),
        "and thank you for it: {:?}", said);
    let kind = find_relationship(&g.world, g.player, asker)
        .and_then(|r| g.world.get_component::<Relationship>(r)).map(|r| r.kind);
    assert_eq!(kind, Some(RelKind::Friend), "and you're friends");
    assert!(g.toasts.iter().any(|(t, ..)| t.starts_with("Mystery told")), "{:?}", g.toasts);
}

#[test]
fn test_only_the_asker_is_offered_the_telling() {
    let mut g = Game::new();
    let folk = askers(&g);
    solved(&mut g, 0, folk[0]);
    talk_to(&mut g, folk[1]);
    assert!(tell_pick(&g).is_none(), "somebody else hasn't been asking");
    // and half a mystery isn't a telling
    g.close_player_talk(0, None);
    g.seats[0].mystery = Some(SeatMystery { def: 0, known: 0b111, asker: folk[0] });
    talk_to(&mut g, folk[0]);
    assert!(tell_pick(&g).is_none(), "not all heard yet");
}

#[test]
fn test_the_tithe_cart_pays_a_finders_fee() {
    let mut g = Game::new();
    let asker = askers(&g)[0];
    let def = MYSTERIES.iter().position(|m| matches!(m.payoff, mysteries::Payoff::Gift(_))).unwrap();
    let gold = |g: &Game| g.world.get_component::<crate::game::livelihood::Purse>(g.player).map(|p| p.gold).unwrap_or(0.0);
    let before = gold(&g);
    solved(&mut g, def, asker);
    talk_to(&mut g, asker);
    g.talk_pick_at(0, tell_pick(&g));
    assert!(gold(&g) > before, "coin for the trouble: {} -> {}", before, gold(&g));
    let t = g.seats[0].talk.as_ref().unwrap();
    assert!(t.notes.iter().any(|n| n.contains("presses")), "the talk says so: {:?}", t.notes);
}

#[test]
fn test_the_chapel_bell_lays_a_place_at_their_table() {
    let mut g = Game::new();
    let def = MYSTERIES.iter().position(|m| m.payoff == mysteries::Payoff::Table).unwrap();
    let asker = askers(&g).into_iter()
        .find(|&v| crate::game::household::home_of(&g.world, v).is_some()).expect("someone with a roof");
    solved(&mut g, def, asker);
    talk_to(&mut g, asker);
    g.talk_pick_at(0, tell_pick(&g));
    assert!(g.table_offered(0, asker), "they ask you in: {:?}", g.seats[0].table_invite);
}

#[test]
fn test_the_mill_wheel_leads_on_to_a_job() {
    let mut g = Game::new();
    let def = MYSTERIES.iter().position(|m| m.payoff == mysteries::Payoff::Lead).unwrap();
    let asker = askers(&g)[0];
    g.seats[0].quest = None;
    solved(&mut g, def, asker);
    talk_to(&mut g, asker);
    g.talk_pick_at(0, tell_pick(&g));
    let q = g.seats[0].quest.as_ref().expect("the next thing to run down is yours");
    assert_eq!(q.giver, asker);
    assert_eq!(q.kind, crate::session::quests::QuestKind::Note);
    // with a job already on, the lead waits on the board for you instead
    let mut g = Game::new();
    let asker = askers(&g)[0];
    if g.seats[0].quest.is_none() {
        g.seats[0].quest = Some(crate::session::quests::PlayerQuest {
            giver: askers(&g)[1], kind: crate::session::quests::QuestKind::Egg,
            target: crate::session::quests::QuestTarget::Villager(askers(&g)[1]),
            stage: crate::session::quests::QuestStage::ToFetch, pickup: Some((1.0, 1.0)), expires: 100 });
    }
    solved(&mut g, def, asker);
    talk_to(&mut g, asker);
    g.talk_pick_at(0, tell_pick(&g));
    assert!(g.quest_offer_at(asker).is_some(), "the lead's going begging till you're free");
}

#[test]
fn test_word_of_the_doorstep_gifts_goes_round() {
    let mut g = Game::new();
    let def = MYSTERIES.iter().position(|m| m.payoff == mysteries::Payoff::Word).unwrap();
    let folk = askers(&g);
    let (asker, other) = (folk[0], folk[1]);
    let before = crate::game::player::rapport(&g.world, g.player, other);
    solved(&mut g, def, asker);
    talk_to(&mut g, asker);
    g.talk_pick_at(0, tell_pick(&g));
    let after = crate::game::player::rapport(&g.world, g.player, other);
    assert!(after > before, "folk who never asked think the better of you: {} -> {}", before, after);
}
