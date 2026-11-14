use crate::session::*;

#[test]
fn test_teach_passes_the_craft_on() {
    let mut g = Game::new();
    let trainer = g.villagers[0];
    let student = g.villagers[1];
    // the trainer's apprentice-deep in the craft
    crate::game::org_skills::practice(&mut g.world, trainer, crate::game::org_skills::OrgSkill::Turnout, 0.5);
    let before_t = g.world.get_component::<crate::game::org_skills::Organizing>(trainer)
        .map(|o| o.get(crate::game::org_skills::OrgSkill::Training)).unwrap_or(0.0);
    let ok = crate::game::org_skills::teach(&mut g.world, None, trainer, student,
        crate::game::org_skills::OrgSkill::Turnout, g.tick);
    assert!(ok, "the lesson never ran");
    let after = g.world.get_component::<crate::game::org_skills::Organizing>(student)
        .map(|o| o.get(crate::game::org_skills::OrgSkill::Turnout)).unwrap_or(0.0);
    assert!(after > 0.0, "the student never learned");
    let after_t = g.world.get_component::<crate::game::org_skills::Organizing>(trainer)
        .map(|o| o.get(crate::game::org_skills::OrgSkill::Training)).unwrap_or(0.0);
    assert!(after_t > before_t, "the trainer never grew");
}

/// The muscle only gets so big: a full season of transformational work
/// fills the gauge and stops at the cap - the rest of a big cause's
/// size shows in the lord's answer, not the number
#[test]
fn test_the_muscle_never_outgrows_its_size() {
    let mut g = Game::new();
    for _ in 0..30 { g.record_power(crate::sim::organized_power::ActionQuality::Transformational); }
    let cap = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
        .map(|p| p.capacity).unwrap_or(0.0);
    assert!(cap <= crate::sim::organized_power::CAPACITY_CAP,
        "the cause outgrew its own muscle: {}", cap);
}

#[test]
fn test_a_hidden_school_trains_slower() {
    let mut g = Game::new();
    let trainer = g.villagers[0];
    let student = g.villagers[1];
    crate::game::org_skills::practice(&mut g.world, trainer, crate::game::org_skills::OrgSkill::Turnout, 0.5);
    // open: a full lesson
    crate::game::org_skills::teach(&mut g.world, None, trainer, student,
        crate::game::org_skills::OrgSkill::Turnout, g.tick);
    let open_gain = g.world.get_component::<crate::game::org_skills::Organizing>(student)
        .map(|o| o.get(crate::game::org_skills::OrgSkill::Turnout)).unwrap_or(0.0);
    // hidden: the same lesson runs slower
    let student2 = g.villagers[2];
    g.world.add_component(trainer, crate::game::org_skills::HiddenVenue);
    crate::game::org_skills::teach(&mut g.world, None, trainer, student2,
        crate::game::org_skills::OrgSkill::Turnout, g.tick);
    let hidden_gain = g.world.get_component::<crate::game::org_skills::Organizing>(student2)
        .map(|o| o.get(crate::game::org_skills::OrgSkill::Turnout)).unwrap_or(0.0);
    assert!(hidden_gain < open_gain, "the hidden school never ran slower: {} vs {}", hidden_gain, open_gain);
}
