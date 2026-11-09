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
