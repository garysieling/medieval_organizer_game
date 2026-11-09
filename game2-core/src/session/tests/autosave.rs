use crate::session::*;

#[test]
fn test_the_days_own_record_replays_to_the_same_village() {
    let mut g = Game::new();
    g.command(Command::Walk(1.0, 0.0));
    assert!(!g.journal.entries.is_empty(), "the command never journaled");
    // the record tags inputs by frame, not day: a command mid-run lands
    // where the replay applies it (#822)
    for _ in 0..12 { g.update(); }
    g.command(Command::Walk(0.0, 1.0));
    g.command(Command::TogglePause);
    g.command(Command::TogglePause);
    for _ in 0..6 { g.update(); }
    // the journal round-trips through its own format
    let json = serde_json::to_string(&g.journal).unwrap_or_default();
    let j: crate::session::journal::Journal = serde_json::from_str(&json).unwrap();
    assert_eq!(g.journal.entries.len(), j.entries.len(), "the journal didn't round-trip");
    // and a replay builds the same world, not just a village (#822)
    let r = crate::session::journal::replay(&j, g.frame_n as u64);
    assert_eq!(r.state_digest(), g.state_digest(), "the replayed world drifted from the played one");
}

#[test]
fn test_the_record_carries_the_roster_pick() {
    let mut g = Game::new();
    g.pick_character_at(0, 2);
    for _ in 0..10 { g.update(); }
    let j: crate::session::journal::Journal = serde_json::from_str(
        &serde_json::to_string(&g.journal).unwrap()).unwrap();
    assert!(j.entries.iter().any(|(_, i)| matches!(i,
        crate::session::journal::Input::PickCharacter { seat: 0, idx: 2 })),
        "the pick never journaled");
    let r = crate::session::journal::replay(&j, g.frame_n as u64);
    assert_eq!(r.state_digest(), g.state_digest(), "the pick didn't survive the replay");
}
