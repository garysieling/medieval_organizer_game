//! The servant's pass: the paper that proves standing - marked books,
//! masterless folk, the watch's reach, and the org's answers (#1261)
use crate::session::*;
use crate::session::subsystems::society::Watcher;

#[test]
fn test_a_hired_hand_gets_their_book_marked() {
    let mut g = Game::new();
    let (v, m) = (g.villagers[0], g.villagers[1]);
    g.world.get_component_mut::<Craft>(v).map(|c| c.tasked_by = Some(m));
    g.pass_tick(12 + 20);
    let p = g.world.get_component::<crate::game::pass::Pass>(v)
        .expect("a hired hand carries the master's mark");
    assert_eq!(p.marked_by, Some(m));
    assert!(!p.forged, "the master's mark isn't a forgery");
}

#[test]
fn test_the_wanderer_carries_a_paper() {
    let mut g = Game::new();
    g.pass_tick(12 + 20);
    assert!(g.world.get_component::<crate::game::pass::Pass>(g.player).is_some(),
        "the wanderer has no letter to show");
}

#[test]
fn test_the_masterless_take_no_lawful_work() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.world.add_component(v, crate::game::pass::Masterless);
    let (made, _) = crate::game::livelihood::work_day(&mut g.world, v, 5);
    assert_eq!(made, 0.0, "a masterless hand worked the day");
}

#[test]
fn test_a_held_member_organizes_at_half() {
    let mut g = Game::new();
    let (v, m) = (g.villagers[0], g.villagers[1]);
    g.world.get_component_mut::<Craft>(v).map(|c| c.tasked_by = Some(m));
    g.world.add_component(v, crate::game::pass::Pass { marked_by: Some(m), since: 0, forged: false });
    g.world.add_component(v, crate::game::org_skills::Organizing {
        turnout: 0.4, agitation: 0.2, ..Default::default() });
    assert!((g.organizing_of(v) - 0.3).abs() < 0.01,
        "held by the book: half weight, got {}", g.organizing_of(v));
}

#[test]
fn test_the_watch_sweeps_the_masterless_first() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.world.add_component(v, crate::game::pass::Masterless);
    let lord = g.sites.iter().find_map(|s| s.lord).expect("a castle's lord");
    if g.world.get_component::<Watcher>(lord).is_none() {
        g.world.add_component(lord, Watcher { paranoia: 0.0, legitimacy: 0.0 });
    }
    g.world.get_component_mut::<Watcher>(lord).map(|w| w.paranoia = 1.0);
    for t in (12..=12 + 20 * 60).step_by(20) { g.pass_tick(t); }
    assert!(g.sanctioned.iter().any(|(e, ..)| *e == v)
        || g.log.iter().any(|l| l.contains("masterless")),
        "the watch never reached for them");
}

#[test]
fn test_marking_the_book_frees_the_member() {
    let mut g = Game::new();
    let (v, m) = (g.villagers[0], g.villagers[1]);
    g.world.get_component_mut::<Craft>(v).map(|c| c.tasked_by = Some(m));
    g.world.add_component(v, crate::game::pass::Pass { marked_by: Some(m), since: 0, forged: false });
    g.world.get_component_mut::<crate::sim::community::Organization>(g.seats[0].org)
        .map(|o| o.members.push(v));
    g.seats[0].selected_target = 0;
    g.update_ctx_actions_at(0);
    let i = g.seats[0].ctx_actions.iter().position(|&a| a == CtxAct::MarkBook)
        .expect("the ask's not on the card");
    g.ctx_act_at(0, i);
    assert!(g.world.get_component::<crate::game::pass::Masterless>(v).is_some(),
        "free of the book - and paperless");
    assert!(g.world.get_component::<Craft>(v).map(|c| c.tasked_by.is_none()).unwrap_or(false),
        "the master still holds them");
}

#[test]
fn test_reporting_masterless_is_the_orgs_own_sanction() {
    let mut g = Game::new();
    let (pe, v) = (g.player, g.villagers[0]);
    g.world.add_component(v, crate::game::pass::Pass { marked_by: None, since: 0, forged: false });
    g.world.get_component_mut::<crate::sim::community::Organization>(g.seats[0].org)
        .map(|o| o.members.push(pe));
    g.seats[0].selected_target = 0;
    g.update_ctx_actions_at(0);
    let i = g.seats[0].ctx_actions.iter().position(|&a| a == CtxAct::ReportMasterless)
        .expect("the report's not offered");
    g.ctx_act_at(0, i);
    assert!(g.world.get_component::<crate::game::pass::Masterless>(v).is_some());
    assert!(g.world.get_component::<crate::game::pass::Pass>(v).is_none(),
        "the report left the paper standing");
}
