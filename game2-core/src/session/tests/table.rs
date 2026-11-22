//! The shared table (#1241): a warm bond asks you in, and the meal's
//! three courses carry what the doorstep doesn't.
use crate::session::*;
use crate::engine::ecs::Entity;
use crate::game::desires::{self, Known, Problem, Reveal, Want};
use crate::game::household;
use crate::game::relationships::{self, find_relationship, Interaction, InteractionKind, Relationship};
use crate::game::villager::{self, Pos};
use crate::game::vocation::Kid;
use crate::session::dialogue::TalkKind;
use crate::session::subsystems::table::{COURSE_FRAMES, TABLE_GAP_DAYS};

/// Grown villagers with a roof of their own, one per household
fn householders(g: &Game) -> Vec<usize> {
    let mut homes = vec![];
    (0..g.villagers.len()).filter(|&i| {
        let v = g.villagers[i];
        if g.world.get_component::<Kid>(v).is_some() || villager::is_dead(&g.world, v) { return false; }
        match household::home_of(&g.world, v) {
            Some(h) if !homes.contains(&h) => { homes.push(h); true }
            _ => false,
        }
    }).collect()
}

fn known(g: &mut Game, i: usize, strength: f32) -> Entity {
    let (pe, v) = (g.player, g.villagers[i]);
    g.world.add_component(v, Known { revealed: vec![
        Reveal::Problem(Problem::Taxed), Reveal::Want(Want::GoodFood), Reveal::Want(Want::Love)] });
    relationships::record_interaction(&mut g.world, pe,
        Interaction { other: v, kind: InteractionKind::Conversation, intensity: 0.3 });
    find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component_mut::<Relationship>(r)).unwrap().strength = strength;
    v
}

fn strength(g: &Game, v: Entity) -> f32 {
    find_relationship(&g.world, g.player, v)
        .and_then(|r| g.world.get_component::<Relationship>(r)).map(|r| r.strength).unwrap()
}

/// Stand the player beside villager `i`, picked
fn beside(g: &mut Game, i: usize) {
    let v = g.villagers[i];
    let p = *g.world.get_component::<Pos>(v).unwrap();
    let pe = g.player;
    g.world.add_component(pe, Pos { x: p.x + 0.5, y: p.y });
    g.seats[0].picked = true;
    g.seats[0].selected_target = i;
    g.update_ctx_actions_at(0);
}

/// Days of invitations from `from`, the invitations each day's left
fn invites(g: &mut Game, from: u64, days: u64) -> Vec<Entity> {
    let mut hosts = vec![];
    for t in from..from + days {
        g.tick = t;
        g.table_tick(t);
        if let Some((h, _)) = g.seats[0].table_invite.take() { hosts.push(h); }
    }
    hosts
}

fn sit(g: &mut Game, i: usize) {
    let v = g.villagers[i];
    g.seats[0].table_invite = Some((v, g.tick + 2));
    beside(g, i);
    let k = g.seats[0].ctx_actions.iter().position(|a| *a == CtxAct::Dine).expect("no table button");
    g.ctx_act_at(0, k);
    assert!(g.seats[0].table.is_some(), "didn't sit down");
}

fn meal_lines(g: &Game) -> Vec<(String, bool)> {
    g.talk_log.iter().rev().find(|r| matches!(r.kind, TalkKind::Meal))
        .map(|r| r.lines.iter().map(|l| (l.text.clone(), l.gist)).collect()).unwrap_or_default()
}

#[test]
fn test_a_warm_bond_asks_you_in_and_a_cool_one_doesnt() {
    let mut g = Game::new();
    let hs = householders(&g);
    let warm = known(&mut g, hs[0], 0.6);
    let cool = known(&mut g, hs[1], 0.3);
    let hosts = invites(&mut g, 1, 120);
    assert!(hosts.contains(&warm), "the warm bond never asked");
    assert!(!hosts.contains(&cool), "a cool bond asked you in");
    assert!(hosts.iter().all(|&h| h == warm), "a stranger asked you in");
}

#[test]
fn test_the_place_waits_for_the_host_within_earshot() {
    let mut g = Game::new();
    let hs = householders(&g);
    let v = known(&mut g, hs[0], 0.6);
    known(&mut g, hs[1], 0.6);
    g.seats[0].table_invite = Some((v, g.tick + 2));
    beside(&mut g, hs[1]);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::Dine), "offered at the wrong host's side");
    beside(&mut g, hs[0]);
    assert!(g.seats[0].ctx_actions.contains(&CtxAct::Dine), "the host's place isn't offered");
    let pe = g.player;
    g.world.get_component_mut::<Pos>(pe).unwrap().x += 8.0;
    g.update_ctx_actions_at(0);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::Dine), "offered from across the street");
}

#[test]
fn test_three_courses_carry_the_house_and_warm_it() {
    let mut g = Game::new();
    let hs = householders(&g);
    let v = known(&mut g, hs[0], 0.6);
    let until = g.tick + 300;
    desires::add_grievance(&mut g.world, v, Problem::Broke, until);
    let before = strength(&g, v);
    sit(&mut g, hs[0]);
    for _ in 0..COURSE_FRAMES * 4 { g.advance_tables(); }
    assert!(g.seats[0].table.is_none(), "the meal never ended");
    let lines = meal_lines(&g);
    let at = |s: &str| lines.iter().position(|(t, _)| t.contains(s)).unwrap_or_else(|| panic!("no '{s}' in {lines:?}"));
    assert!(at("bread") < at("pottage") && at("pottage") < at("last cup") && at("last cup") < at("cleared"), "{lines:?}");
    assert!(lines.iter().any(|(t, gist)| *gist && t.contains("is nearly broke")), "the house's trouble wasn't told: {lines:?}");
    assert!(strength(&g, v) > before, "the table warmed nothing");
    let heard = g.world.get_component::<Known>(v).unwrap();
    assert!(heard.revealed.contains(&Reveal::Problem(Problem::Broke)), "the organizer didn't hear it");
}

#[test]
fn test_rising_early_leaves_the_rest_unsaid() {
    let mut g = Game::new();
    let hs = householders(&g);
    let v = known(&mut g, hs[0], 0.6);
    let before = strength(&g, v);
    sit(&mut g, hs[0]);
    for _ in 0..2 { g.advance_tables(); }
    g.command(Command::Walk(1.0, 0.0));
    for _ in 0..COURSE_FRAMES * 4 { g.advance_tables(); }
    let lines = meal_lines(&g);
    assert!(lines.iter().any(|(t, _)| t.contains("bread")), "{lines:?}");
    assert!(!lines.iter().any(|(t, _)| t.contains("last cup")), "the meal went on without you: {lines:?}");
    assert!(lines.iter().any(|(t, _)| t.contains("goes unsaid")), "{lines:?}");
    assert!(strength(&g, v) <= before, "warmth earned from a half-eaten meal");
}

#[test]
fn test_a_household_waits_out_its_gap_but_another_may_ask() {
    let mut g = Game::new();
    let hs = householders(&g);
    let v = known(&mut g, hs[0], 0.6);
    sit(&mut g, hs[0]);
    for _ in 0..COURSE_FRAMES * 4 { g.advance_tables(); }
    let w = known(&mut g, hs[1], 0.6);
    let t0 = g.tick;
    let hosts = invites(&mut g, t0 + 1, TABLE_GAP_DAYS - 1);
    assert!(!hosts.contains(&v), "the same table asked again inside the gap");
    let later = invites(&mut g, t0 + 1 + TABLE_GAP_DAYS, 120);
    assert!(later.contains(&v), "the gap never ran out");
    assert!(hosts.contains(&w) || later.contains(&w), "another household never asked");
}
