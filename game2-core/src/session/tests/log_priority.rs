use crate::session::*;

#[test]
fn test_old_gossip_does_not_push_events_out_of_the_log() {
    let mut log: Vec<String> = vec!["t1: The Merchant Guild formed!".into(), "t120: Bram is taken with la grippe".into()];
    for t in (160..=600).step_by(40) { log.push(format!("t{}: Aldric tells Bram - word spreads", t)); }
    let shown = visible_log(&log);
    assert!(shown.iter().any(|e| e.contains("la grippe")), "the grippe should still be in view: {:?}", shown);
    assert!(shown.iter().any(|e| e.contains("Guild formed")), "the guild should still be in view");
    assert_eq!(shown.iter().filter(|e| is_gossip(e)).count(), LOG_GOSSIP_SHOWN);
    // the important pin rides above the newest gossip - the flu's
    // still first; the word-of-mouth line's just under it
    assert!(shown[0].contains("la grippe"), "important pins top");
    assert_eq!(shown[1], &"t600: Aldric tells Bram - word spreads".to_string(), "newest of the rest next");
}

#[test]
fn test_weather_changes_do_not_push_events_out_of_the_log() {
    let mut log: Vec<String> = vec!["t120: Bram is taken with la grippe".into()];
    for t in (130..=600).step_by(5) {
        log.push(format!("t{}: drizzle begins", t));
        log.push(format!("t{}: the rain stops - a rainbow!", t + 2));
    }
    let shown = visible_log(&log);
    assert!(shown.iter().any(|e| e.contains("la grippe")), "the grippe should still be in view: {:?}", shown);
    assert_eq!(shown.iter().filter(|e| is_weather_note(e)).count(), LOG_WEATHER_SHOWN);
}

#[test]
fn test_flu_from_a_real_game_stays_in_the_log_by_day_600() {
    let mut game = Game::new();
    game.speed = 8;
    while game.tick < 600 { game.update(); }
    // however busy the village's been, the last sickness still pins above
    // everyday chatter - newer happenings (a birth, an exile) may outrank
    // it, but gossip and weather never do
    let shown = visible_log(&game.log);
    let sick = shown.iter().position(|e| e.contains("is taken with") || e.contains("recovered from"));
    let chatter = shown.iter().position(|e| is_gossip(e) || is_weather_note(e));
    assert!(sick.is_some(), "sickness should reach the log in a real game by day 600: {:?}",
        game.log.iter().rev().take(20).collect::<Vec<_>>());
    if let (Some(s), Some(c)) = (sick, chatter) {
        assert!(s < c, "the sickness should outrank routine chatter: {:?} vs {:?}", shown[s], shown[c]);
    }
}

#[test]
fn test_the_log_is_bounded_over_a_long_game() {
    let mut game = Game::new();
    // a running tap of lines, far past the cap
    for t in 1..=(LOG_MAX + 5000) { game.log.push(format!("t{}: a line", t)); }
    game.update();
    assert!(game.log.len() <= LOG_MAX,
        "the log keeps only its recent window: {}", game.log.len());
    assert_eq!(game.log.last().unwrap(), &format!("t{}: a line", LOG_MAX + 5000),
        "and what it keeps is the newest end");
    assert_eq!(game.log_dropped + game.log.len(), LOG_MAX + 5000 + 2,
        "dropped + kept accounts for every line written");
}

#[test]
fn test_the_view_marks_what_rolled_off_the_log() {
    let mut game = Game::new();
    for t in 1..=(LOG_MAX + 500) { game.log.push(format!("t{}: a line", t)); }
    game.update();
    let m = game.ui_model_at(0);
    assert!(m.log_dropped > 0, "the view doesn't hear the drop");
    assert_eq!(m.log_dropped, game.log_dropped, "the count's wrong");
}
