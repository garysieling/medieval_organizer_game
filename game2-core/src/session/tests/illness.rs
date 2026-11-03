use crate::session::*;

#[test]
fn test_flu_arrives_spreads_through_the_log_and_passes() {
    let mut game = Game::new();
    while game.tick < FLU_DAY + 5 { game.update(); }
    // the herb-wife may already have seen to him - the arrival itself
    // is the log line, not whether he's still down with it
    assert!(game.log.iter().any(|l| l.contains("Bram is taken with la grippe")), "the log should say who fell ill");
    while game.tick < 600 { game.update(); }
    // la grippe itself runs its course - whatever else the village
    // caught meanwhile is another story
    assert!(game.world.query::<Disease>().iter().all(|&d|
        game.world.get_component::<Disease>(d).map(|dd| dd.kind != DiseaseKind::Influenza).unwrap_or(true)),
        "la grippe should have run its course by day 600");
    assert!(game.log.iter().any(|l| l.contains("recovered from la grippe")), "the log should say who recovered");
}
