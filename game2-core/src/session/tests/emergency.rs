use crate::session::*;

#[test]
fn test_the_emergency_bans_the_square() {
    let mut g = Game::new();
    g.emergency = Some((g.tick, g.tick + 20));
    assert!(g.emergency.is_some(), "the ban never stood");
    // assemblies won't seat while it stands
    let before = g.protests.len();
    g.call_protest((10.0, 10.0), vec![g.villagers[0]], 1, g.tick + 10, g.tick);
    assert_eq!(g.protests.len(), before, "the ban let a gathering seat");
}

#[test]
fn test_four_witnesses_break_the_tale() {
    let mut g = Game::new();
    g.emergency = Some((g.tick, g.tick + 20));
    for _ in 0..4 { g.decide_at(0, 0); }
    assert!(g.emergency.is_none(), "four witnesses never broke it");
    assert!(g.log.iter().any(|l| l.contains("staged brawl")),
        "the scandal never landed");
}

#[test]
fn test_the_window_that_runs_out_normalizes() {
    let mut g = Game::new();
    g.emergency = Some((g.tick, g.tick + 5));
    g.emergency_tick(g.tick + 10);
    assert!(g.emergency.is_none(), "the window never ran out");
    assert!(g.log.iter().any(|l| l.contains("just how things are")),
        "the normalization never landed");
}
