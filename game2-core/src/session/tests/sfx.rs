use crate::session::*;
use crate::session::routine::HOLY_EVERY;

#[test]
fn test_the_soundscape_raises_its_cues() {
    let mut g = Game::new();
    g.sfx_tick(45); // a feast day
    assert!(g.sfx.iter().any(|c| c.kind == "festival"), "the festival hums");
    // rain's a cue while it falls
    g.precip = Some(crate::sim::precipitation::PrecipKind::Rain);
    g.sfx.clear();
    g.sfx_tick(20);
    assert!(g.sfx.iter().any(|c| c.kind == "rain"), "the bed keeps the weather");
}

#[test]
fn test_the_holy_day_is_seen_and_heard() {
    let mut g = Game::new();
    // run through the first holy day's dawn frame
    while g.tick < HOLY_EVERY { g.update(); }
    // the bell rings over the square that morning
    assert!(g.sfx.iter().any(|c| c.kind == "bell"),
        "the bell should ring on the holy day: {:?}", g.sfx.iter().map(|c| c.kind).collect::<Vec<_>>());
    // and somebody says the word over their head
    let said = g.bubbles.iter().any(|b| {
        ["gather", "faith", "Holy", "Pray"].iter().any(|k| b.1.contains(k))
    });
    assert!(said, "somebody should speak at the gathering: {:?}",
        g.bubbles.iter().map(|b| &b.1).collect::<Vec<_>>());
}

#[test]
fn test_cues_reach_the_scene() {
    let mut g = Game::new();
    g.emit_sfx("wonder", 20.0, 5.0, 0.7);
    let m = g.ui_model();
    assert!(m.sfx.iter().any(|k| k == "wonder"), "the cue's on the scene for the audio layer");
    // and a frame later it's gone - cues live exactly one read
    g.update();
    assert!(g.sfx.is_empty(), "the queue resets");
}
