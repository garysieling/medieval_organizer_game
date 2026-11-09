use crate::session::*;

#[test]
fn test_the_setup_fires_the_arc() {
    let mut g = Game::new();
    g.storyteller_tick(45);
    assert!(g.storyteller.is_some(), "the teller never spoke");
    assert_eq!(g.arc_phase, 1, "the setup never landed: {}", g.arc_phase);
    assert!(g.log.iter().any(|l| l.contains("the teller's own setup")),
        "the setup never logged");
}

#[test]
fn test_the_arc_escalates_through_the_phases() {
    let mut g = Game::new();
    g.storyteller = Some(g.tick);
    g.arc_phase = 1;
    g.storyteller_tick(45);
    assert_eq!(g.arc_phase, 2, "the escalation never came: {}", g.arc_phase);
    g.storyteller_tick(75);
    assert_eq!(g.arc_phase, 3, "the climax never fired: {}", g.arc_phase);
}

#[test]
fn test_the_resolution_closes_and_the_teller_rests() {
    let mut g = Game::new();
    g.storyteller = Some(g.tick);
    g.arc_phase = 3;
    g.storyteller_tick(45);
    assert_eq!(g.arc_phase, 0, "the arc never closed: {}", g.arc_phase);
    assert!(g.storyteller.is_none(), "the teller never rested");
}
