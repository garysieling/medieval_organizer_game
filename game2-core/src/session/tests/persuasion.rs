//! The one-on-one argument (#1234): a private case, turned over for a
//! few days, moves where they stand on an issue.
use crate::session::*;
use crate::engine::ecs::Entity;
use crate::game::desires::{Known, Problem, Reveal, Want};
use crate::game::organizing::{stage_of, PropKind, RelStage};
use crate::game::persuasion::{self, ArgueMove, Outcome, PendingPersuasion, Stance, THINK_DAYS};
use crate::game::reactions::Sympathized;
use crate::game::relationships::{self, find_relationship, Interaction, InteractionKind, Relationship};
use crate::game::traits::{Personality, Trait};
use crate::game::villager::{self, Pos};
use crate::game::vocation::Kid;

fn adult(g: &Game) -> usize {
    (0..g.villagers.len()).find(|&i| g.world.get_component::<Kid>(g.villagers[i]).is_none()
        && !villager::is_dead(&g.world, g.villagers[i])).unwrap()
}

/// Make villager `i` someone the organizer knows (three pieces heard),
/// with the given temper and bond strength
fn known(g: &mut Game, i: usize, traits: Vec<Trait>, strength: f32) -> Entity {
    let (pe, v) = (g.player, g.villagers[i]);
    g.world.add_component(v, Personality { traits });
    g.world.add_component(v, Known { revealed: vec![
        Reveal::Problem(Problem::Taxed), Reveal::Want(Want::GoodFood), Reveal::Want(Want::Love)] });
    relationships::record_interaction(&mut g.world, pe,
        Interaction { other: v, kind: InteractionKind::Conversation, intensity: 0.3 });
    set_strength(g, v, strength);
    v
}

fn set_strength(g: &mut Game, v: Entity, s: f32) {
    let pe = g.player;
    find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component_mut::<Relationship>(r)).unwrap().strength = s;
}

fn strength(g: &Game, v: Entity) -> f32 {
    find_relationship(&g.world, g.player, v)
        .and_then(|r| g.world.get_component::<Relationship>(r)).map(|r| r.strength).unwrap()
}

fn sympathizes(g: &Game, v: Entity, issue: &str) -> bool {
    g.world.get_component::<Sympathized>(v).map(|s| s.issues.iter().any(|(t, _)| t == issue)).unwrap_or(false)
}

fn talk_beside(g: &mut Game, i: usize) {
    let v = g.villagers[i];
    let p = *g.world.get_component::<Pos>(v).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + 0.5, y: p.y };
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    g.act_at(0, 0);
}

/// Pick a shown choice by its label
fn pick(g: &mut Game, label: &str) {
    let t = g.seats[0].talk.as_ref().expect("still talking");
    let labels = t.choice_labels();
    let i = labels.iter().position(|l| l.starts_with(label))
        .unwrap_or_else(|| panic!("{label:?} among {labels:?}"));
    g.talk_pick_at(0, Some(i));
}

fn record_texts(g: &Game) -> Vec<String> {
    let id = g.seats[0].talk.as_ref().unwrap().id;
    g.talk_log.iter().find(|r| r.id == id).unwrap().lines.iter().map(|l| l.text.clone()).collect()
}

#[test]
fn test_the_argument_waits_on_knowing_them() {
    let mut g = Game::new();
    let i = adult(&g);
    let v = known(&mut g, i, vec![], 0.3);
    assert_eq!(stage_of(&g.world, g.player, v), RelStage::Known);
    let offers = crate::game::organizing::ask_offers(&g.world, g.org, v, 0);
    assert_eq!(offers.first().map(|o| o.kind), Some(PropKind::Argue), "{offers:?}");
    // an acquaintance (one piece heard) gets no hearing for your case
    g.world.add_component(v, Known { revealed: vec![Reveal::Problem(Problem::Taxed)] });
    assert_eq!(stage_of(&g.world, g.player, v), RelStage::Acquaintance);
    assert!(!crate::game::organizing::ask_offers(&g.world, g.org, v, 0).iter().any(|o| o.kind == PropKind::Argue));
}

#[test]
fn test_any_live_issue_is_arguable_not_only_the_campaigns() {
    let mut g = Game::new();
    let i = adult(&g);
    let v = known(&mut g, i, vec![], 0.3);
    assert!(crate::game::campaign::campaign_of(&g.world, g.org).is_none());
    let issues = persuasion::arguable_issues(&g.world, g.org, g.player, v);
    assert!(issues.iter().any(|t| t == "the taxes"), "their own trouble is arguable: {issues:?}");
}

#[test]
fn test_a_won_case_takes_only_after_days_of_thought() {
    let mut g = Game::new();
    let i = adult(&g);
    let v = known(&mut g, i, vec![], 0.6);
    let issue = "the taxes".to_string();
    let t0 = g.tick;
    persuasion::push_argument(&mut g.world, v, &issue, Stance::For, Outcome::Won, 1.2, t0);
    g.marks_tick(t0 + 1);
    assert!(!sympathizes(&g, v, &issue), "not the same day: they turn it over first");
    assert!(g.world.get_component::<PendingPersuasion>(v).is_some());
    g.marks_tick(t0 + THINK_DAYS);
    assert!(sympathizes(&g, v, &issue), "days later it takes");
    assert!(g.world.get_component::<PendingPersuasion>(v).is_none());
}

#[test]
fn test_a_faint_case_builds_on_the_next() {
    let mut g = Game::new();
    let i = adult(&g);
    let v = known(&mut g, i, vec![], 0.3);
    let issue = "the taxes".to_string();
    let t0 = g.tick;
    persuasion::push_argument(&mut g.world, v, &issue, Stance::For, Outcome::Won, 0.6, t0);
    persuasion::push_argument(&mut g.world, v, &issue, Stance::For, Outcome::Won, 0.6, t0 + 1);
    persuasion::mature(&mut g.world, v, t0 + THINK_DAYS);
    assert!(!sympathizes(&g, v, &issue), "the latest word restarts the thinking");
    persuasion::mature(&mut g.world, v, t0 + 1 + THINK_DAYS);
    assert!(sympathizes(&g, v, &issue), "two fractional cases add up");
}

#[test]
fn test_arguing_against_lifts_a_sympathy() {
    let mut g = Game::new();
    let i = adult(&g);
    let v = known(&mut g, i, vec![], 0.6);
    let issue = "the taxes".to_string();
    let t0 = g.tick;
    g.world.add_component(v, Sympathized { issues: vec![(issue.clone(), t0)] });
    persuasion::push_argument(&mut g.world, v, &issue, Stance::Against, Outcome::Won, 1.5, t0);
    persuasion::mature(&mut g.world, v, t0 + THINK_DAYS);
    assert!(!sympathizes(&g, v, &issue), "argued round the other way");
}

#[test]
fn test_deeper_bonds_move_further_and_the_proud_barely_shift() {
    let mut g = Game::new();
    let i = adult(&g);
    let v = known(&mut g, i, vec![], 0.3);
    let issue = "the taxes".to_string();
    let m = |g: &Game| persuasion::magnitude(&g.world, g.player, v, &issue, ArgueMove::HearOut, g.tick);
    let at_known = m(&g);
    set_strength(&mut g, v, 0.9);
    assert!(stage_of(&g.world, g.player, v) >= RelStage::Follower);
    let at_follower = m(&g);
    assert!(at_follower > at_known, "{at_follower} > {at_known}");
    g.world.add_component(v, Personality { traits: vec![Trait::Proud] });
    let proud = persuasion::magnitude(&g.world, g.player, v, &issue, ArgueMove::Appeal, g.tick);
    assert!(proud < at_follower * 0.6, "{proud} vs {at_follower}");
}

#[test]
fn test_pressing_a_hothead_and_missing_bristles() {
    let mut g = Game::new();
    let i = adult(&g);
    let v = known(&mut g, i, vec![Trait::Hotheaded], 0.3);
    assert_eq!(persuasion::resolve(&g.world, g.player, v, ArgueMove::Press, 0.99), Outcome::Pressed);
    assert_ne!(persuasion::resolve(&g.world, g.player, v, ArgueMove::HearOut, 0.99), Outcome::Pressed);
}

#[test]
fn test_an_argument_plays_out_in_the_talk() {
    let mut g = Game::new();
    let i = adult(&g);
    let v = known(&mut g, i, vec![Trait::Stingy], 0.3);
    talk_beside(&mut g, i);
    pick(&mut g, "Argue a point");
    let t = g.seats[0].talk.as_ref().unwrap().clone();
    assert!(t.argue.is_some(), "the argument's under way");
    let shown = g.convo_panel_at(0).unwrap().options;
    assert!(shown.iter().any(|l| l.starts_with("Argue ")) && shown.iter().any(|l| l == "Let it drop"), "{shown:?}");
    pick(&mut g, "Argue the tax");
    pick(&mut g, "Argue for");
    let lines = record_texts(&g);
    assert!(lines.iter().any(|l| l.contains("make the case for")), "{lines:?}");
    assert!(lines.iter().any(|l| l.contains("who pays")), "the stingy answer costs it: {lines:?}");
    pick(&mut g, "Hear them out");
    let t = g.seats[0].talk.as_ref().unwrap().clone();
    assert!(t.argue.is_none(), "resolved");
    assert!(t.choice_labels().iter().all(|l| !l.starts_with("Argue a point")), "spent for this talk");
    let lines = record_texts(&g);
    assert!(lines.iter().any(|l| l.contains("think on it") || l.contains("Neither of you") || l.contains("other way")),
        "the outcome is narrated: {lines:?}");
    let _ = v;
}

#[test]
fn test_letting_it_drop_leaves_them_unmoved() {
    let mut g = Game::new();
    let i = adult(&g);
    let v = known(&mut g, i, vec![], 0.3);
    talk_beside(&mut g, i);
    pick(&mut g, "Argue a point");
    pick(&mut g, "Let it drop");
    assert!(g.seats[0].talk.as_ref().unwrap().argue.is_none());
    assert!(g.world.get_component::<PendingPersuasion>(v).is_none());
}

/// Run a full argument for the taxes and return the bond's change
fn argue_for_taxes(held: bool) -> f32 {
    let mut g = Game::new();
    let i = adult(&g);
    let v = known(&mut g, i, vec![], 0.3);
    if held {
        g.world.get_component_mut::<crate::game::desires::Organizer>(g.player).unwrap()
            .issues.push("the taxes".into());
    }
    talk_beside(&mut g, i);
    let before = strength(&g, v);
    pick(&mut g, "Argue a point");
    pick(&mut g, "Argue the tax");
    pick(&mut g, "Argue for");
    pick(&mut g, "Hear them out");
    strength(&g, v) - before
}

#[test]
fn test_arguing_what_your_cause_doesnt_hold_quietly_costs() {
    let sincere = argue_for_taxes(true);
    let laundered = argue_for_taxes(false);
    assert!(laundered < sincere, "laundered {laundered} vs sincere {sincere}");
}
