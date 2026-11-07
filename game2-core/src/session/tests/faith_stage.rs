use crate::session::*;
use crate::sim::religion::{self, FaithStage};

#[test]
fn test_children_stay_in_the_early_stages() {
    let g = Game::new();
    let mut seen = 0;
    for &v in &g.villagers {
        // the parish's children carry a stage; the country's have no
        // congregation to be staged in
        if g.world.get_component::<Kid>(v).is_none() || g.site_of(v).is_some() { continue; }
        seen += 1;
        let s = *g.world.get_component::<FaithStage>(v).expect("a child without a stage");
        assert!(s <= FaithStage::MythicLiteral, "a child reached {:?}", s);
    }
    assert!(seen > 0, "no children to check");
}

#[test]
fn test_the_stage_draw_weights_by_age() {
    // children never past the literal, whatever the roll
    for r in 0..1000u64 {
        assert!(FaithStage::for_years(8.0, r) <= FaithStage::MythicLiteral);
    }
    // the old can reach the universalizing - rare, but there
    assert!((0..1000u64).any(|r| FaithStage::for_years(75.0, r) == FaithStage::Universalizing),
        "no elder roll reaches the deepest stage");
    assert!((0..1000u64).any(|r| FaithStage::for_years(75.0, r) >= FaithStage::Conjunctive),
        "no elder roll reaches the conjunctive");
    // working adults hold the middle - reflective is rare for them
    let deep = (0..1000u64)
        .filter(|&r| FaithStage::for_years(35.0, r) >= FaithStage::IndividuativeReflective).count();
    assert!(deep > 0 && deep < 200, "adults reach reflective too often or never: {}", deep);
}

#[test]
fn test_a_devout_soul_deepens_one_stage_at_a_time() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<FaithStage>(v).is_some()
            && *g.world.get_component::<FaithStage>(v).unwrap() < FaithStage::Universalizing)
        .expect("a soul not yet universal");
    g.world.get_component_mut::<religion::Faith>(v).unwrap().devotion = 0.9;
    let before = *g.world.get_component::<FaithStage>(v).unwrap();
    // roll 0 always fires the rare step; the lapsed don't move at all
    religion::stage_up(&mut g.world, v, 0);
    assert_eq!(*g.world.get_component::<FaithStage>(v).unwrap(), before.step().unwrap(),
        "the office didn't move them a stage");
    // and a soul whose faith has lapsed stays put
    let w = g.villagers.iter().copied().find(|&v|
        *g.world.get_component::<FaithStage>(v).unwrap() < FaithStage::Universalizing).unwrap();
    g.world.get_component_mut::<religion::Faith>(w).unwrap().devotion = 0.1;
    let wb = *g.world.get_component::<FaithStage>(w).unwrap();
    religion::stage_up(&mut g.world, w, 0);
    assert_eq!(*g.world.get_component::<FaithStage>(w).unwrap(), wb, "a lapsed soul deepened");
}

#[test]
fn test_the_cult_locked_stop_at_the_literal() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.world.add_component(v, religion::Cultish);
    g.world.add_component(v, FaithStage::IntuitiveProjective);
    g.world.get_component_mut::<religion::Faith>(v).unwrap().devotion = 0.9;
    religion::stage_up(&mut g.world, v, 0);
    assert_eq!(*g.world.get_component::<FaithStage>(v).unwrap(), FaithStage::MythicLiteral,
        "the literal is the cult's ceiling - they reach it");
    religion::stage_up(&mut g.world, v, 0);
    assert_eq!(*g.world.get_component::<FaithStage>(v).unwrap(), FaithStage::MythicLiteral,
        "a cult-locked soul reached past the literal");
    // and the cap itself says so
    assert_eq!(religion::stage_cap(&g.world, v), FaithStage::MythicLiteral);
}

#[test]
fn test_a_soul_far_along_speaks_as_one() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.world.add_component(v, FaithStage::Universalizing);
    let s = dialogue::speaker_of(&g.world, v, "test", 0);
    assert!(s.status.contains(&"sage"), "the sage speaks plainly: {:?}", s.status);
    // and an ordinary soul carries no such name
    let w = g.villagers.iter().copied().find(|&v|
        matches!(g.world.get_component::<FaithStage>(v),
            Some(FaithStage::MythicLiteral | FaithStage::SyntheticConventional)))
        .expect("a middling soul");
    let s2 = dialogue::speaker_of(&g.world, w, "test", 0);
    assert!(!s2.status.iter().any(|t| matches!(*t, "seeker" | "mystic" | "sage")),
        "a middling soul took a deep name: {:?}", s2.status);
}
