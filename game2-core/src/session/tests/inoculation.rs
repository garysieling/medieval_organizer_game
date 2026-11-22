//! Inoculation (#1246): name the counter-attack before it lands, and it
//! lands on a prepared member - softened, once, for that kind only.
use crate::session::*;
use crate::engine::ecs::Entity;
use crate::game::inoculation::{self as inoc, Counter, COUNTERS};
use crate::game::villager::{self, Pos};
use crate::game::vocation::Kid;

fn adult(g: &Game) -> usize {
    (0..g.villagers.len()).find(|&i| g.world.get_component::<Kid>(g.villagers[i]).is_none()
        && !villager::is_dead(&g.world, g.villagers[i])).unwrap()
}

fn sign_on(g: &mut Game, v: Entity) {
    let org = g.org;
    let o = g.world.get_component_mut::<crate::sim::community::Organization>(org).unwrap();
    if !o.members.contains(&v) { o.members.push(v); }
}

fn talk_beside(g: &mut Game, i: usize) {
    let v = g.villagers[i];
    let p = *g.world.get_component::<Pos>(v).unwrap();
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: p.x + 0.5, y: p.y };
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    g.act_at(0, 0);
}

fn labels(g: &Game) -> Vec<String> {
    g.seats[0].talk.as_ref().map(|t| t.choice_labels()).unwrap_or_default()
}

fn pick(g: &mut Game, label: &str) {
    let ls = labels(g);
    let i = ls.iter().position(|l| l.starts_with(label))
        .unwrap_or_else(|| panic!("{label:?} among {ls:?}"));
    g.talk_pick_at(0, Some(i));
}

fn record_texts(g: &Game) -> Vec<String> {
    let id = g.seats[0].talk.as_ref().unwrap().id;
    g.talk_log.iter().find(|r| r.id == id).map(|r| r.lines.iter().map(|l| l.text.clone()).collect())
        .unwrap_or_default()
}

/// The lord is watching hard enough to run his counters
fn watched(g: &mut Game) {
    let lords: Vec<Entity> = g.sites.iter().filter_map(|s| s.lord).collect();
    assert!(!lords.is_empty(), "a castle lord to watch");
    for l in lords {
        g.world.add_component(l, crate::session::subsystems::society::Watcher { paranoia: 0.9, legitimacy: 0.5 });
    }
}

#[test]
fn test_the_counter_is_named_only_once_they_have_signed_on() {
    let mut g = Game::new();
    let i = adult(&g);
    let v = g.villagers[i];
    let org = g.org;
    assert!(!crate::game::organizing::ask_offers(&g.world, org, v, 0).iter()
        .any(|o| o.kind == crate::game::organizing::PropKind::Inoculate), "not before they've committed");
    sign_on(&mut g, v);
    assert!(crate::game::organizing::ask_offers(&g.world, org, v, 0).iter()
        .any(|o| o.kind == crate::game::organizing::PropKind::Inoculate), "offered once they're in");
}

#[test]
fn test_naming_the_threat_rehearses_it_and_readies_them_for_that_alone() {
    let mut g = Game::new();
    let i = adult(&g);
    let v = g.villagers[i];
    sign_on(&mut g, v);
    talk_beside(&mut g, i);
    pick(&mut g, "Ask how they think the lord will answer");
    let shown = labels(&g);
    for c in COUNTERS { assert!(shown.iter().any(|l| l == inoc::label(c)), "{shown:?}"); }
    assert!(shown.iter().any(|l| l == "Let it drop"), "{shown:?}");
    pick(&mut g, "Name the steward's threat");
    let lines = record_texts(&g);
    assert!(lines.iter().any(|l| l == inoc::fear(Counter::Threat)), "they voice the fear: {lines:?}");
    assert!(lines.iter().any(|l| l == inoc::answer(Counter::Threat)), "you answer it: {lines:?}");
    assert!(inoc::shielded(&g.world, v, Counter::Threat));
    assert!(!inoc::shielded(&g.world, v, Counter::Plea) && !inoc::shielded(&g.world, v, Counter::Denounce),
        "only the named counter");
    assert!(!g.seats[0].talk.as_ref().unwrap().inoculating, "the rehearsal's done");
}

#[test]
fn test_letting_it_drop_readies_nothing() {
    let mut g = Game::new();
    let i = adult(&g);
    let v = g.villagers[i];
    sign_on(&mut g, v);
    talk_beside(&mut g, i);
    pick(&mut g, "Ask how they think the lord will answer");
    pick(&mut g, "Let it drop");
    assert!(COUNTERS.iter().all(|&c| !inoc::shielded(&g.world, v, c)));
}

#[test]
fn test_shields_stack_and_each_is_spent_once() {
    let mut g = Game::new();
    let v = g.villagers[adult(&g)];
    assert!(inoc::shield(&mut g.world, v, Counter::Threat));
    assert!(!inoc::shield(&mut g.world, v, Counter::Threat), "one shield per kind");
    assert!(inoc::shield(&mut g.world, v, Counter::Plea));
    assert!(inoc::take(&mut g.world, v, Counter::Threat));
    assert!(!inoc::take(&mut g.world, v, Counter::Threat), "spent");
    assert!(inoc::shielded(&g.world, v, Counter::Plea), "the other stands");
    assert!(!inoc::take(&mut g.world, v, Counter::Denounce), "never readied");
}

#[test]
fn test_the_de_oathing_lands_at_half_on_a_readied_member_once() {
    let mut g = Game::new();
    watched(&mut g);
    let (a, b) = { let ad: Vec<usize> = (0..g.villagers.len()).filter(|&i|
        g.world.get_component::<Kid>(g.villagers[i]).is_none()).take(2).collect(); (g.villagers[ad[0]], g.villagers[ad[1]]) };
    for &m in &[a, b] {
        sign_on(&mut g, m);
        g.world.add_component(m, crate::sim::psychology::Bond { target: g.player, security: 0.5 });
    }
    inoc::shield(&mut g.world, a, Counter::Threat);
    g.counter_ritual_tick(27);
    let sec = |g: &Game, m| g.world.get_component::<crate::sim::psychology::Bond>(m).unwrap().security;
    let (da, db) = (0.5 - sec(&g, a), 0.5 - sec(&g, b));
    assert!(db > 0.0 && (da - db / 2.0).abs() < 1e-5, "half for the readied: {da} vs {db}");
    assert!(!inoc::shielded(&g.world, a, Counter::Threat), "spent");
    g.counter_ritual_tick(67);
    assert!(((0.5 - da) - sec(&g, a) - db).abs() < 1e-5, "the next lands in full");
}

#[test]
fn test_the_amnesty_harvest_passes_over_a_member_readied_for_the_plea() {
    let mut g = Game::new();
    watched(&mut g);
    let adults: Vec<Entity> = (0..g.villagers.len()).map(|i| g.villagers[i])
        .filter(|&v| g.world.get_component::<Kid>(v).is_none()).take(10).collect();
    assert_eq!(adults.len(), 10);
    for &m in &adults {
        sign_on(&mut g, m);
        let mut o = crate::game::org_skills::Organizing::default();
        o.bump(crate::game::org_skills::OrgSkill::Turnout, 0.2);
        g.world.add_component(m, o);
    }
    let org = g.org;
    let members = |g: &Game| g.world.get_component::<crate::sim::community::Organization>(org).unwrap().members.clone();
    let last = *members(&g).last().unwrap();
    let before = members(&g).len();
    inoc::shield(&mut g.world, last, Counter::Plea);
    g.inquisition = Some(1);
    g.inquisition_tick(32);
    let after = members(&g);
    assert!(after.contains(&last), "the readied one stays");
    assert_eq!(after.len(), before - (before / 10 - 1), "the rest of the harvest still takes");
    assert!(!inoc::shielded(&g.world, last, Counter::Plea), "spent");
}
