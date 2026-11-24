//! The design's Mix bar over the whole scenario library (I1, #1509):
//! a 60 s render of every named scenario sits inside -20 +/-3 LUFS
//! integrated, true peak at or under -1 dBTP, zero clicks, ambience
//! never fully silent, music present within the presence budget, and
//! the render far faster than real time. Run it `--release` - the
//! speed bar is an unloaded-machine one.
mod common;

use game2_audio::analysis;
use game2_audio::wav::SAMPLE_RATE;

#[test]
fn mix_all_ten_scenarios() {
    let scenarios = game2_audio::scenarios::registry();
    assert_eq!(scenarios.len(), 10,
        "the library should hold the 10 named scenarios");
    for sc in scenarios {
        let name = sc.name;
        let r = common::render_scenario(name, 3600);
        let loud = analysis::loudness(&r.pcm, SAMPLE_RATE);
        let clicks = analysis::clicks(&r.pcm, SAMPLE_RATE);
        let gap = analysis::longest_silence(&r.pcm, SAMPLE_RATE, -60.0);
        eprintln!("{name}: {:.1} LUFS, tp {:.2} dBTP, {} clicks, \
            silence {:.1}s, {} voices, {} notes, render {:.2}s",
            loud.integrated_lufs, loud.true_peak_dbtp, clicks.len(),
            gap, r.voices_peak, r.notes.len(), r.render_secs);
        assert!(loud.integrated_lufs > -23.0 && loud.integrated_lufs < -17.0,
            "{name}: integrated {:.1} LUFS outside -20+/-3", loud.integrated_lufs);
        assert!(loud.true_peak_dbtp <= -1.0,
            "{name}: true peak {:.2} dBTP over -1", loud.true_peak_dbtp);
        assert!(clicks.is_empty(), "{name}: clicks at {clicks:?}");
        assert!(gap < 3.0, "{name}: ambience silent for {gap:.1}s");
        assert!(r.voices_peak <= 48, "{name}: {} voices over the cap", r.voices_peak);
        // the presence budget lets the score rest, not vanish: the
        // first phrase always sounds (the debt bucket starts empty)
        // and a rest never outlasts three ambient phrases (~87 s)
        assert!(r.notes.len() >= 10,
            "{name}: the score wrote only {} notes", r.notes.len());
        let first = r.notes.first().unwrap().0 as f64 / SAMPLE_RATE as f64;
        assert!(first < 35.0,
            "{name}: first score note at {first:.1}s - a phrase of silence up front");
        let mut worst = 0.0f64;
        for w in r.notes.windows(2) {
            worst = worst.max((w[1].0 - w[0].0) as f64 / SAMPLE_RATE as f64);
        }
        assert!(worst <= 95.0,
            "{name}: a {worst:.0}s music gap outlasts the budget's 3-phrase rest");
        if cfg!(not(debug_assertions)) {
            // the design's 3 s bar is an unloaded machine; under the
            // parallel suite render threads contend, so this stays a
            // regression tripwire rather than the bar itself
            assert!(r.render_secs < 8.0,
                "{name}: 60s took {:.1}s to render (unloaded budget 3s)",
                r.render_secs);
        }
    }
}
