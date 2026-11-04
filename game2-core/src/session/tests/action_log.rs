use crate::session::*;
use crate::client::*;

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

    game.act(1); // not enough influence yet
    let recruit = game.log.last().unwrap().clone();
    assert!(recruit.starts_with("t3: Can't recruit") && !recruit.contains("influence)"), "{}", recruit);

    game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().resources = 20.0;
    game.act(2);
    let invest = game.log.last().unwrap().clone();
    assert!(invest.starts_with("t3: Invested 10 gold") && !invest.contains("influence"), "{}", invest);

}

#[test]
fn test_influence_lines_are_not_coloured_as_illness() {
    // "influence" contains "flu"; it used to turn every influence line weather-cyan
    let illness = (120, 200, 255);
    assert_ne!(log_colour("t3: Invested 10 gold"), illness);
    assert_ne!(log_colour("t3: Can't recruit: need influence 0.40."), illness);
    assert_ne!(log_colour("t9: the grain store is low"), illness);
    assert_eq!(log_colour("t120: Bram is taken with la grippe"), illness);
    assert_eq!(log_colour("t200: it's raining"), illness);
}
