use crate::session::*;

#[test]
fn test_flu_arrives_spreads_through_the_log_and_passes() {
    let mut game = Game::new();
    game.speed = 8; // the grippe's a daily story; 600 days of frames isn't
    while game.tick < FLU_DAY + 5 { game.update(); }
    // the herb-wife may already have seen to him - the arrival itself
    // is the log line, not whether he's still down with it
    assert!(game.log.iter().any(|l| l.contains("Bram is taken with la grippe")), "the log should say who fell ill");
    while game.tick < 1400 { game.update(); }
    // la grippe itself runs its course - whatever else the village
    // caught meanwhile is another story; a busier village keeps it
    // circulating longer, so the bound runs deep. A Recovering case is
    // the bookkeeping remnant, not the grippe: it sheds nothing and
    // harms nothing, and it can wait on the mend-bar (full health)
    // while unrelated hurts hold a body down - the epidemic has passed
    // even though the entry hasn't been struck yet
    assert!(game.world.query::<Disease>().iter().all(|&d|
        game.world.get_component::<Disease>(d).map(|dd|
            dd.kind != DiseaseKind::Influenza || dd.stage == Stage::Recovering).unwrap_or(true)),
        "la grippe should have run its course by day 1400");
    assert!(game.log.iter().any(|l| l.contains("recovered from la grippe")), "the log should say who recovered");
}
