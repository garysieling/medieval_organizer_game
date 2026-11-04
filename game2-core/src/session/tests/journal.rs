//! A seed and its journal rebuild the world exactly (session::journal)
use crate::session::*;
use crate::session::journal::{replay, Input, Journal};

/// Play a world the way the server's loop does: inputs at a frame, then an
/// update, and note every input in the journal
fn play(frames: u64) -> (Game, Journal) {
    let mut g = Game::new_seeded(MAP_SEED);
    let mut j = Journal::new(MAP_SEED);
    let take = |g: &mut Game, j: &mut Journal, f: u64, input: Input| { g.apply_input(&input); j.record(f, input); };
    take(&mut g, &mut j, 0, Input::AddSeat);
    let villager = g.villagers[g.free_villager().unwrap()];
    take(&mut g, &mut j, 0, Input::AddAgentSeat { villager });
    let (wanderer, agent) = (g.seats.len() - 2, g.seats.len() - 1);
    for f in 1..=frames {
        g.update();
        if f % 7 == 0 { take(&mut g, &mut j, f, Input::Command { seat: wanderer, cmd: Command::Walk(1.0, 0.5) }); }
        if f % 97 == 0 { take(&mut g, &mut j, f, Input::AgentCmd { seat: agent, cmd: AgentCmd::GoTo { x: 20, y: 20 } }); }
        if f % 40 == 0 { take(&mut g, &mut j, f, Input::DriveIdle { seat: wanderer }); }
        if f == 300 { take(&mut g, &mut j, f, Input::AddSeat); }
    }
    (g, j)
}

#[test]
fn the_journal_rebuilds_the_world() {
    let frames = 1200;
    let (live, j) = play(frames);
    assert!(j.entries.len() > 100, "a busy journal");
    let again = replay(&j, frames);
    assert_eq!(again.state_digest(), live.state_digest(), "the replay is the world that was played");
    assert_eq!(again.seats.len(), live.seats.len());
    // and through JSON, as the save file holds it
    let text = serde_json::to_string(&j).unwrap();
    let back: Journal = serde_json::from_str(&text).unwrap();
    assert_eq!(replay(&back, frames).state_digest(), live.state_digest(), "the saved journal replays the same");
}

#[test]
fn the_digest_tells_worlds_apart() {
    let (a, _) = play(200);
    let (mut b, _) = play(200);
    assert_eq!(a.state_digest(), b.state_digest());
    b.update();
    assert_ne!(a.state_digest(), b.state_digest(), "a frame later is another world");
}
