//! Perf probe: run the real game headless and print per-day wall time
//! plus growth counters (log lines, villager/entity/relationship counts)
//! as CSV on stdout.
//!
//! `cargo run --release --bin perf_probe [days] [seed]`
//!
//! Pair it with an external RSS sampler to watch process memory while a
//! long game runs - the CSV's elapsed_ms column is the join key.

use game2_core::session::{driver, Game};
use game2_core::game::relationships::Relationship;
use game2_core::infra::perf;
use std::time::Instant;

fn main() {
    game2_core::infra::crash::install();
    let args: Vec<String> = std::env::args().collect();
    let days: u64 = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(200);
    let seed: u64 = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(42);
    // PERF_SECTIONS=1 records the timed! sections and prints the table at the end
    let sections = std::env::var("PERF_SECTIONS").is_ok();
    if sections { perf::start(); }

    let mut game = Game::new_seeded(seed);
    game.speed = std::env::var("SPEED").ok().and_then(|v| v.parse().ok()).unwrap_or(4); // turbo, same as balance_test

    let start = Instant::now();
    let mut day_start = Instant::now();
    let mut last_tick = game.tick;
    let budget = (days + 50) * game.ticks_per_frame() as u64 * 2;
    println!("day,elapsed_ms,day_ms,log_lines,villagers,entities,relationships");
    for _ in 0..budget {
        if game.tick >= days { break; }
        game.update();
        perf::frame_end();
        if game.frame_n % driver::DRIVE_EVERY as u32 == 0 { driver::drive_player(&mut game, 0); }
        if game.tick != last_tick {
            last_tick = game.tick;
            println!("{},{:.0},{:.1},{},{},{},{}",
                game.tick,
                start.elapsed().as_secs_f64() * 1e3,
                day_start.elapsed().as_secs_f64() * 1e3,
                game.log.len(),
                game.villagers.len(),
                game.world.entity_count(),
                game.world.query::<Relationship>().len());
            day_start = Instant::now();
        }
    }
    if sections {
        eprintln!("\n{}", perf::stop().table(30));
    }
}
