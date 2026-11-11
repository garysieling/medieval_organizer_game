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

#[test]
fn test_the_stones_honored_shift_with_the_roll() {
    let mut g = Game::new();
    g.world.get_component_mut::<Organization>(g.org).unwrap().resources = 100.0;
    g.seats[0].ctx_actions = vec![CtxAct::School];
    g.ctx_act_at(0, 0);
    let (teacher, season_end) = g.teacher.unwrap();
    let ti = g.villagers.iter().position(|&v| v == teacher).unwrap();
    // a later arrival sits after her - the stones that honor him must
    // follow him home, and hers mustn't hang on him
    let home = g.building_entities.iter().copied().find(|&b|
        g.world.get_component::<household::Building>(b)
            .map(|bb| bb.kind == household::BuildingKind::Home && bb.occupants.len() < 4)
            .unwrap_or(false)).expect("no home with a spare bed");
    g.spawn_kid(home, g.tick);
    g.cairns.push(Cairn { pos: (5, 5), honoree: ti + 1, deed: "of the bench".into(), stones: 1 });
    g.cairns.push(Cairn { pos: (6, 6), honoree: ti, deed: "of the desk".into(), stones: 1 });
    g.monuments.push(Monument { pos: (7, 7), honoree: ti + 1, title: "a deed".into(),
        build_until: 0, stood_on: Some(1), kind: MonumentKind::Column });
    g.monuments.push(Monument { pos: (8, 8), honoree: usize::MAX, title: "the ring".into(),
        build_until: 0, stood_on: Some(1), kind: MonumentKind::Circle });
    g.tick = season_end;
    g.school_tick(season_end);
    assert_eq!(g.cairns[0].honoree, ti, "his pile didn't follow him home");
    assert_eq!(g.cairns[1].honoree, usize::MAX, "her pile hangs on the wrong name");
    assert_eq!(g.monuments[0].honoree, ti, "his column didn't follow him home");
    assert_eq!(g.monuments[1].honoree, usize::MAX, "the ring's sentinel moved");
    // and the rolls still read: no panic past the end of the roll
    g.cairn_tick(g.tick + 1);
    g.monument_tick(g.tick + 1);
}

#[test]
fn test_the_community_teacher_grows_the_agency() {
    let mut g = Game::new();
    g.folk_school = true;
    g.community_teacher = true;
    let m = g.villagers[0];
    g.world.add_component(m, crate::game::desires::Domination {
        internalized_domination: 0.5, critical_agency: 0.2, experience: 0.0 });
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !o.members.contains(&m) { o.members.push(m); }
    }
    g.movement_school_tick(35);
    let d = g.world.get_component::<crate::game::desires::Domination>(m);
    let agency = d.map(|d| d.critical_agency).unwrap_or(0.2);
    assert!(agency > 0.2, "the community teacher never grew the agency: {}", agency);
}

#[test]
fn test_the_imported_expert_grows_the_dependency() {
    let mut g = Game::new();
    g.folk_school = true;
    g.community_teacher = false; // the imported expert
    let m = g.villagers[0];
    g.world.add_component(m, crate::game::desires::Domination {
        internalized_domination: 0.5, critical_agency: 0.2, experience: 0.0 });
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        if !o.members.contains(&m) { o.members.push(m); }
    }
    g.movement_school_tick(35);
    let dom = g.world.get_component::<crate::game::desires::Domination>(m)
        .map(|d| d.internalized_domination).unwrap_or(0.5);
    assert!(dom > 0.5, "the imported expert never grew the dependency: {}", dom);
}

#[test]
fn test_the_ask_waits_on_place_and_purse() {
    // #1071: the school ask used to show anywhere, always. It lives
    // where the thing's done - the market plot it'd rise on - and only
    // while the treasury could pay the press
    let mut g = Game::new();
    let mp = g.world.get_component::<Pos>(g.market_food).copied().unwrap();
    // funded but forty tiles off: no ask follows you across the map
    g.world.get_component_mut::<Organization>(g.org).unwrap().resources = 100.0;
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: mp.x + 40.0, y: mp.y + 40.0 };
    g.update_ctx_actions();
    assert!(!g.ctx_actions.contains(&CtxAct::School), "the ask followed the player far from the site");
    // beside the market cross but the chest's light: still no ask
    *g.world.get_component_mut::<Pos>(g.player).unwrap() = Pos { x: mp.x + 1.0, y: mp.y };
    g.world.get_component_mut::<Organization>(g.org).unwrap().resources = 5.0;
    g.update_ctx_actions();
    assert!(!g.ctx_actions.contains(&CtxAct::School), "an empty chest still offered the ask");
    // near and funded: the ask's on the card
    g.world.get_component_mut::<Organization>(g.org).unwrap().resources = 100.0;
    g.update_ctx_actions();
    assert!(g.ctx_actions.contains(&CtxAct::School), "beside the site with a full chest, no ask");
}

#[test]
fn test_the_hover_says_what_the_press_does() {
    // #1071: "The school" reads as a mystery - the hint line tells the
    // press ahead of it, per state, and doesn't bar the button
    let mut g = Game::new();
    assert_eq!(g.ctx_hint_at(0, CtxAct::School).as_deref(),
        Some("the treasury raises the schoolhouse by the market - 30 gold"));
    g.world.get_component_mut::<Organization>(g.org).unwrap().resources = 100.0;
    g.seats[0].ctx_actions = vec![CtxAct::School];
    g.ctx_act_at(0, 0);
    assert!(g.school.is_some(), "no schoolhouse raised");
    // the house stands but the desk's empty: the ask turns to keeping it
    g.teacher = None;
    assert_eq!(g.ctx_hint_at(0, CtxAct::School).as_deref(),
        Some("send for a teacher for the season - 10 gold"));
    // the model carries it parallel to the ctx list, index for index
    let m = g.ui_model();
    assert_eq!(m.ctx_hint.len(), m.ctx.len(), "hints fell out of step with the ctx list");
    let i = m.ctx.iter().position(|a| *a == CtxAct::School);
    if let Some(i) = i {
        assert!(m.ctx_hint[i].is_some(), "the School button has no hint");
        assert!(m.ctx_blocked[i].is_none(), "a hint is not a block");
    }
}

#[test]
fn test_the_button_names_the_deed_not_the_place() {
    let mut g = Game::new();
    // no school yet: the ask is to raise it
    assert_eq!(g.ctx_label_at(0, CtxAct::School), "Found the school");
    // the room stands but the desk's empty: the ask is to fill it
    g.school = Some(g.world.create_entity());
    g.teacher = None;
    assert_eq!(g.ctx_label_at(0, CtxAct::School), "Send for a teacher");
}
