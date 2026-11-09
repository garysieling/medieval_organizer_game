use crate::session::*;

#[test]
fn test_the_stages_accrue_under_a_master() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let master = g.villagers[1];
    if let Some(c) = g.world.get_component_mut::<crate::game::vocation::Craft>(m) {
        c.master = Some(master);
        c.stage = 1;
    }
    g.craft_stage_tick(38);
    let stage = g.world.get_component::<crate::game::vocation::Craft>(m)
        .map(|c| c.stage).unwrap_or(1);
    assert!(stage > 1, "the stages never accrued: {}", stage);
}

#[test]
fn test_the_masterwork_lifts_the_skill() {
    let mut g = Game::new();
    let m = g.villagers[0];
    let master = g.villagers[1];
    if let Some(c) = g.world.get_component_mut::<crate::game::vocation::Craft>(m) {
        c.master = Some(master);
        c.stage = 3;
        c.skill = 0.3;
    }
    g.craft_stage_tick(38);
    let skill = g.world.get_component::<crate::game::vocation::Craft>(m)
        .map(|c| c.skill).unwrap_or(0.3);
    assert!(skill >= 0.6, "the masterwork never lifted: {}", skill);
    assert!(g.log.iter().any(|l| l.contains("the masterwork's own finish")),
        "the finish never logged");
}
