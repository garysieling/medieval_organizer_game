//! Balance test mode: run the real game headless and mark it.
//!
//! `cargo run --release --bin balance_test [days] [seed]`
//!
//! The Wanderer's seat is left to the idle driver (the same fallback the
//! server uses when a human goes quiet), so the run measures what a
//! passive player would feel: the world doing its work around them.
//! Each day's gauges and log lines feed the BalanceTester's KPI table
//! (see wiki/literature-reviews/ for where the bands come from).
//!
//! Exits 1 when any KPI red-flags, so it can gate a schedule like the
//! playtest gate does.

use game2_core::session::{driver, Game};
use game2_core::agents::balance_tester::{BalanceTester, DaySample, RunRecord};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let days: u64 = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(200);
    let seed: u64 = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(42);

    let mut game = Game::new_seeded(seed);
    game.speed = 4; // turbo: days tick ~4x faster headless, same sim

    let mut record = RunRecord { seed, days, samples: vec![] };
    let mut log_read = 0usize; // cursor into game.log - lines we've already taken
    let mut last_tick = game.tick;
    // a generous frame budget: ~ticks_per_frame per day plus slack for the
    // day-boundary work the game does around each tick
    let budget = (days + 50) * game.ticks_per_frame() as u64 * 2;
    for _ in 0..budget {
        if game.tick >= days { break; }
        game.update();
        if game.frame_n % driver::DRIVE_EVERY as u32 == 0 { driver::drive_player(&mut game, 0); }
        if game.tick != last_tick {
            last_tick = game.tick;
            let events: Vec<String> = game.log[log_read.min(game.log.len())..]
                .iter().map(|l| l.splitn(2, ": ").nth(1).unwrap_or(l).to_string()).collect();
            log_read = game.log.len();
            record.samples.push(DaySample { tick: game.tick, gauges: game.telemetry().iter()
                .map(|(k, v)| (k.to_string(), *v)).collect(), events });
        }
    }
    record.days = record.samples.len() as u64;

    let tester = BalanceTester::new();
    let report = tester.score(&record);
    print!("{}", tester.render(&report));

    let last_log = game.log.len().saturating_sub(15);
    println!("-- last log lines --");
    for l in &game.log[last_log..] { println!("  {}", l); }

    if report.flags.is_empty() { std::process::exit(0); } else { std::process::exit(1); }
}
