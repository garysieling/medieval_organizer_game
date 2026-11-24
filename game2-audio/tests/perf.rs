//! I4's render budget: the busiest scene the scenario library can
//! stage - the crackdown march over a market-day crowd - must keep the
//! engine inside its CPU budget per rendered block, or a weak device
//! would crackle where a fast one doesn't.

mod common;

use game2_audio::host::AudioHost;
use game2_audio::scenarios;
use game2_audio::soundscape::Listener;
use game2_audio::wav::SAMPLE_RATE;
use game2_core::session::stage::{self, Stage};

/// A 60 fps block at the offline rate is ~16.7 ms of audio; the design
/// budget is per 10 ms, so a frame's cost scales by its block length.
fn per_10ms(ms: f64, samples: usize) -> f64 {
    ms * (SAMPLE_RATE as f64 / 100.0) / samples as f64
}

#[test]
fn perf_budget_block_render() {
    if cfg!(debug_assertions) {
        eprintln!("perf_budget_block_render is a release-only test - rerun with --release");
        return;
    }
    // march_crackdown's script over a market-day village: the march
    // bed, the crowd bed and a Peak-mode score all sounding at once.
    let mut g = (scenarios::by_name("market_square").unwrap().build)(42);
    stage::stage(&mut g, Stage::StrongOrg);
    let mut host = AudioHost::offline(SAMPLE_RATE);
    host.set_bank(common::test_bank());
    let mut block: Vec<f32> = vec![];
    let mut per: Vec<f64> = vec![];
    for f in 0..3600u64 { // 60 s at 60 fps
        if f == 60 { let _ = g.force_road_march(0, 0); }
        g.update();
        let m = g.ui_model();
        let l = m.market
            .map(|(x, y)| Listener::at(x, y, 1.0))
            .unwrap_or_else(|| Listener::from_model(&m));
        host.push_with(&m, &l);
        let n = host.frame_samples();
        block.clear();
        block.resize(n * 2, 0.0);
        let t = std::time::Instant::now();
        host.render_block(&mut block);
        per.push(per_10ms(t.elapsed().as_secs_f64() * 1e3, n));
    }
    per.sort_by(|a, b| a.total_cmp(b));
    let p95 = per[(per.len() as f64 * 0.95) as usize - 1];
    let (max, mean) = (*per.last().unwrap(), per.iter().sum::<f64>() / per.len() as f64);
    // shared CI runners can't hold the desktop budget - it relaxes 3x
    let budget = if std::env::var_os("CI").is_some() { 3.0 } else { 1.0 };
    eprintln!("perf: {mean:.3} ms mean, {p95:.3} p95, {max:.3} max per 10 ms block (budget {budget})");
    assert!(p95 <= budget, "audio render p95 {p95:.3} ms per 10 ms block, over the {budget} ms budget");
}
