//! The organizer's own line (#1236): what each villager hears you keep
//! saying hardens into their read of you.
use crate::session::*;
use crate::engine::ecs::Entity;
use crate::game::conversation::Act;
use crate::game::desires::{Known, Problem, Reveal, Want};
use crate::game::rhetoric::{self, Line, RhetoricMemory, HARDEN};
use crate::game::relationships::{self, find_relationship, Interaction, InteractionKind, Relationship};
use crate::game::traits::{Personality, Trait};
use crate::game::villager::{self, Pos};
use crate::game::vocation::Kid;

fn adults(g: &Game) -> Vec<usize> {
    (0..g.villagers.len()).filter(|&i| g.world.get_component::<Kid>(g.villagers[i]).is_none()
        && !villager::is_dead(&g.world, g.villagers[i])).collect()
}

fn known(g: &mut Game, i: usize, traits: Vec<Trait>) -> Entity {
    let (pe, v) = (g.player, g.villagers[i]);
    g.world.add_component(v, Personality { traits });
    g.world.add_component(v, Known { revealed: vec![
        Reveal::Problem(Problem::Taxed), Reveal::Want(Want::GoodFood), Reveal::Want(Want::Love)] });
    relationships::record_interaction(&mut g.world, pe,
        Interaction { other: v, kind: InteractionKind::Conversation, intensity: 0.3 });
    find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component_mut::<Relationship>(r)).unwrap().strength = 0.3;
    v
}

fn talk_beside(g: &mut Game, i: usize) {
    let v = g.villagers[i];
    let p = *g.world.get_component::<Pos>(v).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + 0.5, y: p.y };
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    g.act_at(0, 0);
}

fn pick(g: &mut Game, label: &str) {
    let labels = g.seats[0].talk.as_ref().expect("still talking").choice_labels(&g.world);
    let i = labels.iter().position(|l| l.starts_with(label))
        .unwrap_or_else(|| panic!("{label:?} among {labels:?}"));
    g.talk_pick_at(0, Some(i));
}

fn weight(g: &Game, v: Entity, l: Line) -> f32 {
    g.world.get_component::<RhetoricMemory>(v)
        .and_then(|m| m.by_org.get(&g.org))
        .map(|r| r.weight[l as usize]).unwrap_or(0.0)
}

fn harden(g: &mut Game, v: Entity, l: Line) {
    let org = g.org;
    for _ in 0..HARDEN as usize { rhetoric::witness(&mut g.world, v, org, l, g.tick); }
}

#[test]
fn test_a_line_hardens_after_enough_witnessing() {
    let mut g = Game::new();
    let org = g.org;
    let v = g.villagers[adults(&g)[0]];
    rhetoric::witness(&mut g.world, v, org, Line::Firebrand, 5);
    rhetoric::witness(&mut g.world, v, org, Line::Firebrand, 6);
    assert!(rhetoric::hardened(&g.world, v, org).is_empty(), "two words aren't a line yet");
    rhetoric::witness(&mut g.world, v, org, Line::Firebrand, 7);
    assert_eq!(rhetoric::hardened(&g.world, v, org), vec![Line::Firebrand]);
    // and it's theirs alone: nobody else heard it
    let other = g.villagers[adults(&g)[1]];
    assert!(rhetoric::hardened(&g.world, other, org).is_empty());
}

#[test]
fn test_going_back_on_a_line_costs_it_and_is_named_once() {
    let mut g = Game::new();
    let org = g.org;
    let v = g.villagers[adults(&g)[0]];
    harden(&mut g, v, Line::Patient);
    let before = weight(&g, v, Line::Patient);
    rhetoric::witness(&mut g.world, v, org, Line::Firebrand, 200);
    assert!(weight(&g, v, Line::Patient) < before * 0.6, "the opposite pick halves the line");
    assert!(!rhetoric::hardened(&g.world, v, org).contains(&Line::Patient), "and it no longer holds");
    let q = rhetoric::take_turn(&mut g.world, v, org).expect("the turn is named");
    assert!(q.contains("preached patience in spring"), "{q}");
    assert!(rhetoric::take_turn(&mut g.world, v, org).is_none(), "named once");
}

#[test]
fn test_an_unrepeated_line_fades() {
    let mut g = Game::new();
    let org = g.org;
    let v = g.villagers[adults(&g)[0]];
    harden(&mut g, v, Line::Patient);
    for _ in 0..40 { rhetoric::fade(&mut g.world, v); }
    assert!(rhetoric::hardened(&g.world, v, org).is_empty(), "a turned page can turn back");
}

#[test]
fn test_exacting_hardens_across_the_dilemma_s_real_spacing() {
    // Exacting's only production evidence is the burdened man's dilemma
    // (`EventId::Dilemma`, gap 300 + a 150-day jitter): a villager who
    // sees the hard branch chosen roundly, visit after visit 300-450
    // days apart, should eventually read the organizer as exacting -
    // the old uniform daily fade decayed that evidence to nothing
    // between visits and it could never harden.
    let mut g = Game::new();
    let org = g.org;
    let v = g.villagers[adults(&g)[0]];
    for visit in 0..6 {
        rhetoric::witness(&mut g.world, v, org, Line::Exacting, visit * 375);
        if rhetoric::hardened(&g.world, v, org).contains(&Line::Exacting) { break; }
        for _ in 0..375 { rhetoric::fade(&mut g.world, v); }
    }
    assert!(rhetoric::hardened(&g.world, v, org).contains(&Line::Exacting),
        "repeated hard choices, spaced like the dilemma actually recurs, should harden");
}

#[test]
fn test_the_talk_opens_on_their_read_and_names_the_turn() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = known(&mut g, i, vec![]);
    harden(&mut g, v, Line::Firebrand);
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().expect("talking").clone();
    assert!(t.player.status.contains(&"firebrand"), "the read rides on your speaker: {:?}", t.player.status);
    assert!(t.notes.iter().any(|n| n.contains("the firebrand")), "{:?}", t.notes);
    g.close_player_talk(0, None);
    // now they hear you counsel patience - and next time they say so
    let org = g.org;
    for k in 0..2 { rhetoric::witness(&mut g.world, v, org, Line::Patient, g.tick + k); }
    g.tick += 1;
    talk_beside(&mut g, i);
    let t = g.seats[0].talk.as_ref().expect("talking").clone();
    assert!(t.notes.iter().any(|n| n.contains("names the turn") && n.contains("called for fire")), "{:?}", t.notes);
    assert!(!t.player.status.contains(&"firebrand"));
}

#[test]
fn test_their_read_moves_the_asks_a_rung() {
    let mut g = Game::new();
    let a = adults(&g);
    let hot = known(&mut g, a[0], vec![Trait::Hotheaded]);
    let kind = known(&mut g, a[1], vec![Trait::Kind]);
    harden(&mut g, hot, Line::Firebrand);
    harden(&mut g, kind, Line::Firebrand);
    let org = g.org;
    assert_eq!(rhetoric::ask_shift(&g.world, hot, org), 1, "hotheads find the firebrand");
    assert_eq!(rhetoric::ask_shift(&g.world, kind, org), -1, "the gentle shy off");
    talk_beside(&mut g, a[0]);
    let asks = g.seats[0].talk.as_ref().unwrap().asks.clone();
    assert_eq!(asks, crate::game::organizing::ask_offers(&g.world, g.org, hot, 1), "the open talk takes the shift");
    let mut p = Game::new();
    let pa = adults(&p);
    let hot = known(&mut p, pa[0], vec![Trait::Hotheaded]);
    harden(&mut p, hot, Line::Patient);
    let porg = p.org;
    assert_eq!(rhetoric::ask_shift(&p.world, hot, porg), -1, "the hothead shrugs at the patient one");
}

#[test]
fn test_how_you_argue_is_a_line_they_hear() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = known(&mut g, i, vec![]);
    talk_beside(&mut g, i);
    pick(&mut g, "Argue a point");
    pick(&mut g, "Argue the tax");
    pick(&mut g, "Argue for");
    pick(&mut g, "Press");
    assert_eq!(weight(&g, v, Line::Firebrand), 1.0);
    let other = g.villagers[adults(&g)[1]];
    assert_eq!(weight(&g, other, Line::Firebrand), 0.0, "only the one you argued with heard it");
}

#[test]
fn test_a_talk_pick_reaches_only_the_partner() {
    // push back on whoever offers the chance first; the day's first
    // bond-moving pick is the one they remember
    let mut g = Game::new();
    let a = adults(&g);
    let mut seen = None;
    'outer: for &i in &a {
        let v = known(&mut g, i, vec![]);
        talk_beside(&mut g, i);
        for _ in 0..6 {
            let Some(t) = g.seats[0].talk.as_ref() else { break };
            if t.argue.is_some() { break; }
            if let Some(k) = t.options.iter().position(|(act, _)| *act == Act::PushBack) {
                let probing = t.opt_tree == Some(crate::game::conversation::PROBE_TREE);
                g.talk_pick_at(0, Some(k));
                if !probing { seen = Some(v); break 'outer; }
                continue;
            }
            if t.options.is_empty() { break; }
            let k = t.options.iter().position(|(act, _)| !matches!(act, Act::Agree | Act::Answer | Act::Greet | Act::Farewell)).unwrap_or(usize::MAX);
            if k == usize::MAX { break; }
            g.talk_pick_at(0, Some(k));
        }
        g.close_player_talk(0, None);
    }
    let v = seen.expect("someone offered a push-back");
    assert_eq!(weight(&g, v, Line::Firebrand), 1.0);
    assert!(a.iter().map(|&i| g.villagers[i]).filter(|&o| o != v)
        .all(|o| weight(&g, o, Line::Firebrand) == 0.0), "nobody else was there");
}

#[test]
fn test_a_panel_pick_reaches_only_members_at_hand() {
    let mut g = Game::new();
    let a = adults(&g);
    let (near, far, victim) = (g.villagers[a[0]], g.villagers[a[1]], g.villagers[a[2]]);
    let org = g.org;
    g.world.get_component_mut::<crate::sim::community::Organization>(org).unwrap().members.extend([near, far]);
    let at = *g.world.get_component::<Pos>(g.player).unwrap();
    *g.world.get_component_mut::<Pos>(near).unwrap() = Pos { x: at.x + 2.0, y: at.y };
    *g.world.get_component_mut::<Pos>(far).unwrap() = Pos { x: at.x + 40.0, y: at.y };
    g.dilemma = Some(crate::session::events::MoralKnot {
        text: "the grain or the children?".into(),
        branches: vec![
            crate::session::events::DilemmaBranch { text: "the grain".into(), debt: 0.6, victim },
            crate::session::events::DilemmaBranch { text: "the children".into(), debt: 0.3, victim },
        ] });
    g.decide_at(0, 0);
    assert_eq!(weight(&g, near, Line::Exacting), 1.0, "the wounding branch is the hard line");
    assert_eq!(weight(&g, far, Line::Exacting), 0.0, "out of earshot, unwitnessed");
    g.dilemma = Some(crate::session::events::MoralKnot {
        text: "again?".into(),
        branches: vec![
            crate::session::events::DilemmaBranch { text: "the grain".into(), debt: 0.6, victim },
            crate::session::events::DilemmaBranch { text: "the children".into(), debt: 0.3, victim },
        ] });
    g.decide_at(0, 1);
    assert_eq!(weight(&g, near, Line::Merciful), 1.0, "the softest branch is mercy");
}

#[test]
fn test_two_organizers_keep_separate_reads() {
    // Alice (seat 0) and Bob (a second seat) address the same villager.
    // One organizer's rhetoric is theirs alone (#1236): Bob shouldn't
    // inherit Alice's hardened firebrand reputation, her ask bonus, or
    // her pending turn, and Alice's read shouldn't move when Bob talks
    // patience to the same villager.
    let mut g = Game::new();
    let v = g.villagers[adults(&g)[0]];
    let org_a = g.org;
    let seat_b = g.add_seat();
    let org_b = g.seats[seat_b].org;
    assert_ne!(org_a, org_b);

    for _ in 0..HARDEN as usize { rhetoric::witness(&mut g.world, v, org_a, Line::Firebrand, g.tick); }
    assert!(rhetoric::hardened(&g.world, v, org_a).contains(&Line::Firebrand), "Alice's line hardened");
    assert!(rhetoric::hardened(&g.world, v, org_b).is_empty(), "Bob inherits none of Alice's reputation");
    assert_eq!(rhetoric::ask_shift(&g.world, v, org_b), 0, "Bob gets none of Alice's ask bonus");

    // Bob counsels patience on his own first talk with this villager
    rhetoric::witness(&mut g.world, v, org_b, Line::Patient, g.tick);
    assert!(rhetoric::hardened(&g.world, v, org_a).contains(&Line::Firebrand),
        "Alice's reputation is untouched by Bob's talk");
    assert!(rhetoric::take_turn(&mut g.world, v, org_a).is_none(),
        "Alice never went back on her own line - nothing of hers to name");
    assert!(rhetoric::take_turn(&mut g.world, v, org_b).is_none(),
        "Bob hasn't hardened a line of his own to turn on");
}
