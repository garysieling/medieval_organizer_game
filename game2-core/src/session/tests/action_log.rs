use crate::session::*;

#[test]
fn test_action_lines_are_dated_and_say_who_not_the_score() {
    let mut game = Game::new();
    while game.tick < 3 { game.update(); }
    // stand next to Eliza and select her
    let (ex, ey) = { let p = game.world.get_component::<Pos>(game.villagers[1]).unwrap(); (p.x, p.y) };
    { let p = game.world.get_component_mut::<Pos>(game.player).unwrap(); p.x = ex + 0.5; p.y = ey; }
    game.selected_target = 1;

    let logged = game.log.len();
    game.act(0);
    // a Talk's words go on the TALK panel, dated, not into the event log
    assert_eq!(game.log.len(), logged, "talk isn't an event: {:?}", game.log.last());
    let talk = game.talk_log.last().unwrap();
    assert!(talk.tick == 3 && talk.lines.iter().any(|l| l.gist && l.name == "Eliza"), "{:?}", talk.lines);

    // a stale action index (a client on an older build) is ignored outright
    game.act(1);
    assert_eq!(game.log.len(), logged, "no button 1 anymore: {:?}", game.log.last());

}
