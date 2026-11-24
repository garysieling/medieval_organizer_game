//! Hooks (#1243): an old wound colors their talk, a deep bond lays it to
//! rest - yours or a friend's - and it's remembered.
use crate::session::*;
use crate::engine::ecs::Entity;
use crate::game::conversation::Criterion;
use crate::game::hooks::{self, Carried, Hooks, HOOKS};
use crate::game::mentorship::SEASON;
use crate::game::organizing::FOLLOWER_TIE;
use crate::game::relationships::{self, find_relationship, Interaction, InteractionKind, RelKind, Relationship};
use crate::game::villager::{self, Pos};
use crate::game::vocation::Kid;

fn adults(g: &Game) -> Vec<usize> {
    (0..g.villagers.len()).filter(|&i| g.world.get_component::<Kid>(g.villagers[i]).is_none()
        && !villager::is_dead(&g.world, g.villagers[i])).collect()
}

fn carry(g: &mut Game, v: Entity, id: usize) {
    g.world.add_component(v, Hooks { carried: vec![Carried { id, laid: None, told: false, ghost: None }] });
}

fn bond(g: &mut Game, v: Entity, s: f32) {
    let pe = g.player;
    relationships::record_interaction(&mut g.world, pe,
        Interaction { other: v, kind: InteractionKind::Conversation, intensity: 0.3 });
    find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component_mut::<Relationship>(r)).unwrap().strength = s;
}

fn beside(g: &mut Game, i: usize) {
    let at = *g.world.get_component::<Pos>(g.villagers[i]).unwrap();
    let pe = g.player;
    *g.world.get_component_mut::<Pos>(pe).unwrap() = Pos { x: at.x + 0.5, y: at.y };
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    g.update_ctx_actions_at(0);
}

fn talk_notes(g: &mut Game, i: usize) -> Vec<String> {
    beside(g, i);
    g.act_at(0, 0);
    g.seats[0].talk.as_ref().expect("talking").notes.clone()
}

#[test]
fn test_setup_draws_up_to_two_hooks_the_same_each_time() {
    let (a, b) = (Game::new(), Game::new());
    let draw = |g: &Game| g.villagers.iter().map(|&v| g.world.get_component::<Hooks>(v)
        .map(|h| h.carried.iter().map(|c| c.id).collect::<Vec<_>>()).unwrap_or_default()).collect::<Vec<_>>();
    let (da, db) = (draw(&a), draw(&b));
    assert_eq!(da, db);
    assert!(da.iter().all(|h| h.len() <= 2));
    assert!(da.iter().filter(|h| !h.is_empty()).count() >= 5, "hardly anyone carries a hook: {da:?}");
    let starters = hooks::starters();
    assert!(da.iter().flatten().all(|id| starters.contains(id)), "a sequel drawn at setup");
}

#[test]
fn test_an_open_hook_colors_their_small_talk() {
    for h in HOOKS {
        let want = format!("self:status:{}", h.status);
        assert!(crate::game::dialogue_trees::SMALL_CHAT.voice.iter()
            .any(|v| v.criteria.iter().any(|c| matches!(c, Criterion::Has(s) if *s == want))),
            "no small talk hints at {}", h.status);
    }
    let mut g = Game::new();
    let v = g.villagers[adults(&g)[0]];
    carry(&mut g, v, 2);
    let s = crate::session::dialogue::speaker_of(&g.world, v, "x", g.tick);
    assert!(s.status.contains(&"hook:miller_debt"), "{:?}", s.status);
}

#[test]
fn test_only_a_follower_within_earshot_is_asked_after_it() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    carry(&mut g, v, 2);
    bond(&mut g, v, 0.3);
    beside(&mut g, i);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::AskAfter), "asked of a mere acquaintance");
    bond(&mut g, v, FOLLOWER_TIE + 0.05);
    beside(&mut g, i);
    assert!(g.seats[0].ctx_actions.contains(&CtxAct::AskAfter), "a follower can't be asked");
    let pe = g.player;
    g.world.get_component_mut::<Pos>(pe).unwrap().x += 8.0;
    g.update_ctx_actions_at(0);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::AskAfter), "asked from across the street");
}

#[test]
fn test_asking_lays_it_to_rest_surfaces_the_sequel_and_is_remembered_once() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    carry(&mut g, v, 0);
    bond(&mut g, v, FOLLOWER_TIE + 0.05);
    beside(&mut g, i);
    let k = g.seats[0].ctx_actions.iter().position(|a| *a == CtxAct::AskAfter).unwrap();
    g.ctx_act_at(0, k);
    let h = g.world.get_component::<Hooks>(v).unwrap().clone();
    assert_eq!(h.carried[0].laid.map(|(by, _)| by), Some(g.player));
    assert_eq!(hooks::open(&g.world, v), Some(1), "the next layer didn't surface");
    let notes = talk_notes(&mut g, i);
    assert!(notes.iter().any(|n| n.contains("You're the one who finally asked after the brother who went to soldier")), "{notes:?}");
    assert!(g.world.get_component::<Hooks>(v).unwrap().carried[0].told);
    assert!(hooks::untold(&g.world, v).is_none(), "it'd be said again");
}

#[test]
fn test_a_paid_debt_lifts_their_standing() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    carry(&mut g, v, 2);
    bond(&mut g, v, FOLLOWER_TIE + 0.05);
    let before = g.world.get_component::<crate::sim::community::Citizen>(v).unwrap().influence;
    beside(&mut g, i);
    g.ask_after_at(0);
    let after = g.world.get_component::<crate::sim::community::Citizen>(v).unwrap().influence;
    assert!(after > before + 0.05, "{before} -> {after}");
    assert!(hooks::open(&g.world, v).is_none(), "the debt had no sequel");
}

#[test]
fn test_a_close_friend_lays_it_to_rest_and_it_is_told() {
    let mut g = Game::new();
    let ads = adults(&g);
    let (i, v, f) = (ads[0], g.villagers[ads[0]], g.villagers[ads[1]]);
    carry(&mut g, v, 2);
    relationships::record_interaction(&mut g.world, v,
        Interaction { other: f, kind: InteractionKind::Conversation, intensity: 0.3 });
    {
        let r = find_relationship(&g.world, v, f).unwrap();
        let r = g.world.get_component_mut::<Relationship>(r).unwrap();
        r.kind = RelKind::Friend;
        r.strength = 0.8;
    }
    for s in 1..60 {
        g.tick = s * SEASON;
        let t = g.tick;
        g.hooks_tick(t);
        if hooks::open(&g.world, v).is_none() { break; }
    }
    let h = g.world.get_component::<Hooks>(v).unwrap().clone();
    assert_eq!(h.carried[0].laid.map(|(by, _)| by), Some(f), "the friend never laid it to rest");
    assert!(g.log.iter().any(|l| l.contains("old trouble to rest")), "it happened silently");
    let named = format!("{} finally got me to speak of the debt owed the miller", g.name_of(f));
    let notes = talk_notes(&mut g, i);
    assert!(notes.iter().any(|n| n.contains(&named)), "{notes:?}");
}

#[test]
fn test_a_cool_acquaintance_lays_nothing_to_rest() {
    let mut g = Game::new();
    let ads = adults(&g);
    let (v, f) = (g.villagers[ads[0]], g.villagers[ads[1]]);
    carry(&mut g, v, 2);
    for &o in &g.villagers.clone() {
        if let Some(r) = find_relationship(&g.world, v, o) {
            g.world.get_component_mut::<Relationship>(r).unwrap().strength = 0.2;
        }
    }
    relationships::record_interaction(&mut g.world, v,
        Interaction { other: f, kind: InteractionKind::Conversation, intensity: 0.3 });
    for s in 1..60 { g.tick = s * SEASON; let t = g.tick; g.hooks_tick(t); }
    assert_eq!(hooks::open(&g.world, v), Some(2));
}

#[test]
fn test_the_dead_are_not_asked_after_anything() {
    let mut g = Game::new();
    let i = adults(&g)[0];
    let v = g.villagers[i];
    carry(&mut g, v, 2);
    bond(&mut g, v, FOLLOWER_TIE + 0.05);
    g.world.add_component(v, villager::Dead);
    beside(&mut g, i);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::AskAfter), "a corpse offered to speak");
    g.ask_after_at(0);
    assert_eq!(hooks::open(&g.world, v), Some(2), "the dead laid a hook to rest");
}
