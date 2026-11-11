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
    // harms nothing. Since #957 the remnant is also bounded: the
    // mend-clock strikes every entry inside three weeks, whatever else
    // holds the body down
    assert!(game.world.query::<Disease>().iter().all(|&d|
        game.world.get_component::<Disease>(d).map(|dd|
            dd.kind != DiseaseKind::Influenza || (dd.stage == Stage::Recovering && dd.recovering < crate::sim::illness::RECOVERY_DAYS)).unwrap_or(true)),
        "la grippe should have run its course by day 1400, or be inside its mending clock");
    assert!(game.log.iter().any(|l| l.contains("recovered from la grippe")), "the log should say who recovered");
}

#[test]
fn test_no_disease_lingers_past_its_mending() {
    // #957: a case pinned under the recovered mark by unrelated hurts
    // once sat Recovering for years (Bram carried la grippe 200->2000+).
    // The mend-clock strikes every entry: sweep the whole village's
    // illnesses - weather, want and all - and no case may sit in
    // Recovering past its days
    let mut game = Game::new();
    game.speed = 8;
    while game.tick < 1600 {
        game.update();
        if game.tick % 100 != 0 { continue; }
        for &d in game.world.query::<Disease>().iter() {
            let dd = game.world.get_component::<Disease>(d).unwrap();
            assert!(!(dd.stage == Stage::Recovering && dd.recovering >= crate::sim::illness::RECOVERY_DAYS),
                "t{}: {} has been on the mend {} days - the clock should have struck it",
                game.tick, dd.name, dd.recovering);
        }
    }
}
