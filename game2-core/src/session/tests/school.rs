use crate::session::*;

#[test]
fn test_the_school_opens_and_the_desk_counts_the_days() {
    let mut g = Game::new();
    // the treasury raises the room and sends for a teacher
    g.world.get_component_mut::<Organization>(g.org).unwrap().resources = 100.0;
    g.seats[0].ctx_actions = vec![CtxAct::School];
    g.ctx_act_at(0, 0);
    assert!(g.school.is_some(), "no schoolhouse raised");
    assert!(g.teacher.is_some(), "no teacher sent for");
    // a kid's day at the desk counts
    let kid = g.villagers.iter().copied()
        .find(|&v| g.world.get_component::<Kid>(v).is_some()).expect("no kid");
    let sp = g.world.get_component::<Pos>(g.school.unwrap()).copied().unwrap();
    g.world.add_component(kid, routine::Routine {
        mode: routine::Mode::Playing, spot: routine::Spot::Play,
        anchor: (sp.x, sp.y), face: None, drift: 0.0, leads: false, day: g.tick, reached: false });
    g.school_tick(g.tick);
    assert_eq!(g.world.get_component::<villager::Scholared>(kid).map(|s| s.days), Some(1),
        "the day at the desk didn't count");
    // and the season runs out: the desk stands empty till another's sent for
    let season_end = g.teacher.unwrap().1;
    g.tick = season_end;
    g.school_tick(season_end);
    assert!(g.teacher.is_none(), "the season never ended");
    assert!(g.log.iter().any(|l| l.contains("schoolmistress moves on")), "{:?}", g.log.last());
}

#[test]
fn test_the_hire_grows_the_rolls_and_her_card_opens() {
    let mut g = Game::new();
    g.world.get_component_mut::<Organization>(g.org).unwrap().resources = 100.0;
    g.seats[0].ctx_actions = vec![CtxAct::School];
    g.ctx_act_at(0, 0);
    assert!(g.teacher.is_some(), "no teacher sent for");
    // the hire is a name on the roster: the parallel rolls keep step or
    // the next lookup panics (#940)
    assert_eq!(g.was_sick.len(), g.villagers.len(), "was_sick fell behind the roster");
    assert_eq!(g.died_on.len(), g.villagers.len(), "died_on fell behind the roster");
    // her card opens - she's the last name on the roll
    g.seats[0].selected_target = g.villagers.len() - 1;
    let ins = g.villager_inspector_at(0);
    assert!(!ins.lines.is_empty(), "no card for the schoolmistress");
}

#[test]
fn test_the_desk_emptying_shifts_the_rolls_home() {
    let mut g = Game::new();
    g.world.get_component_mut::<Organization>(g.org).unwrap().resources = 100.0;
    g.seats[0].ctx_actions = vec![CtxAct::School];
    g.ctx_act_at(0, 0);
    let (teacher, season_end) = g.teacher.unwrap();
    let ti = g.villagers.iter().position(|&v| v == teacher).unwrap();
    // a later arrival sits after her on the roll, so her leaving shifts him
    let home = g.building_entities.iter().copied().find(|&b|
        g.world.get_component::<household::Building>(b)
            .map(|bb| bb.kind == household::BuildingKind::Home && bb.occupants.len() < 4)
            .unwrap_or(false)).expect("no home with a spare bed");
    g.spawn_kid(home, g.tick);
    let after = g.villagers[ti + 1];
    assert_ne!(after, teacher, "the new arrival landed before her");
    // the pick watched the one after her - it should follow him home
    g.seats[0].selected_target = ti + 1;
    g.died_on[ti] = Some(3); // a mark on her line, not his
    g.tick = season_end;
    g.school_tick(season_end);
    assert!(g.teacher.is_none(), "the season never ended");
    assert_eq!(g.was_sick.len(), g.villagers.len(), "was_sick didn't shift home");
    assert_eq!(g.died_on.len(), g.villagers.len(), "died_on didn't shift home");
    assert_eq!(g.villagers[ti], after, "the one after her didn't shift home");
    assert_eq!(g.died_on[ti], None, "her record hung on the wrong villager");
    assert_eq!(g.seats[0].selected_target, ti, "the pick didn't follow the shift");
}
