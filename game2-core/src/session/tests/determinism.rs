//! One seed, one game: two runs of the same seed must agree tick for tick.
//! Replays, lockstep multiplayer and golden-frame tests all lean on it.
use crate::session::*;

/// Everything a run can differ in, as text: the omniscient probe, the
/// secret gauges, where everyone stands and the log
fn state_of(g: &Game) -> String {
    let pos: Vec<(f32, f32)> = g.villagers.iter()
        .map(|&v| g.world.get_component::<villager::Pos>(v).map(|p| (p.x, p.y)).unwrap_or((-1.0, -1.0)))
        .collect();
    format!("{}\n{:?}\n{:?}\n{:?}", serde_json::to_string(&g.probe()).unwrap(), g.secret_kpis(), pos, g.log)
}

/// First tick at which two runs of `seed` disagree, and the first differing line
fn first_divergence(seed: u64, ticks: u64) -> Option<(u64, String, String)> {
    // turbo+: the check is once a day either way, and 150 days of frames
    // is the suite's slowest test by far at walking pace
    let (mut a, mut b) = (Game::new_seeded(seed), Game::new_seeded(seed));
    a.speed = 16;
    b.speed = 16;
    while a.tick < ticks {
        let day = a.tick;
        a.update();
        b.update();
        assert_eq!(a.tick, b.tick, "the clocks run together");
        // compare once a day: the state's big, and a day is when most of it moves
        if a.tick == day { continue; }
        let (sa, sb) = (state_of(&a), state_of(&b));
        if sa != sb {
            let (la, lb) = sa.lines().zip(sb.lines()).find(|(x, y)| x != y).unwrap_or(("?", "?"));
            let at = la.char_indices().zip(lb.chars()).find(|((_, x), y)| x != y).map(|((i, _), _)| i).unwrap_or(0);
            let cut = |s: &str| s.chars().skip(at.saturating_sub(120)).take(260).collect::<String>();
            return Some((a.tick, cut(la), cut(lb)));
        }
    }
    None
}

#[test]
fn the_same_seed_plays_the_same_game() {
    if let Some((t, a, b)) = first_divergence(42, std::env::var("GAME2_DET_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(150)) {
        panic!("two runs of seed 42 diverge at tick {t}:\n  a: {a}\n  b: {b}");
    }
}
