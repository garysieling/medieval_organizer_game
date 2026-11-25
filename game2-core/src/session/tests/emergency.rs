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

/// A decision box never lists a lone option (#1627): the ban's window
/// always carries the out - let the tale stand today and the box waits
/// out its own day while the ban still stands
#[test]
fn test_the_ban_can_be_let_stand_for_the_day() {
    let mut g = Game::new();
    g.emergency = Some((g.tick, g.tick + 20));
    let panel = g.convo_panel_at(0).expect("the ban's window never offered");
    assert_eq!(panel.title, "THE LORD'S EMERGENCY");
    assert!(panel.options.len() >= 2, "a pick list of one isn't a choice");
    g.decide_at(0, panel.options.len() - 1);
    assert!(g.emergency.is_some(), "letting it stand broke the ban itself");
    assert_eq!(g.emergency_declined, Some(g.tick), "the wave-off never marked the day");
    assert_ne!(g.convo_panel_at(0).map(|p| p.title).as_deref(), Some("THE LORD'S EMERGENCY"),
        "the box re-offered on a declined day");
    // tomorrow the window presses again while the ban still stands
    g.tick += 1;
    assert_eq!(g.convo_panel_at(0).map(|p| p.title).as_deref(), Some("THE LORD'S EMERGENCY"),
        "the window never came round a new day");
}
