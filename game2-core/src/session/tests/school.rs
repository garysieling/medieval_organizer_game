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
