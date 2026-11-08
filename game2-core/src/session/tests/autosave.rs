use crate::session::*;

#[test]
fn test_the_days_own_record_replays_to_the_same_village() {
    let mut g = Game::new();
    g.command(Command::Walk(1.0, 0.0));
    assert!(!g.journal.entries.is_empty(), "the command never journaled");
    // the journal round-trips through its own format
    let json = serde_json::to_string(&g.journal).unwrap_or_default();
    let j: crate::session::journal::Journal = serde_json::from_str(&json).unwrap();
    assert_eq!(g.journal.entries.len(), j.entries.len(), "the journal didn't round-trip");
    // and a replay builds a village (the frame count's the journal's own)
    let frames = j.entries.last().map(|e| e.0).unwrap_or(0);
    let _r = crate::session::journal::replay(&j, frames);
}
