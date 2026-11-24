use super::clock::*;
use super::{Meter, NoteEvent, PatchId, Quantize, Voice};

fn ev(at: u64, voice: Voice) -> NoteEvent {
    NoteEvent {
        at_sample: at,
        dur_samples: 100,
        voice,
        midi: 62.0,
        vel: 0.8,
        patch: PatchId(0),
    }
}

#[test]
fn beat_exact_at_48k_90bpm_is_32000_samples() {
    // 60/90 s per quarter at 48 kHz: a 4/4 beat is exactly 32000 samples.
    let mut c = Clock::new(48_000, 90.0, Meter::FOUR_FOUR);
    let ticks = c.advance(32_001);
    let beats: Vec<u64> = ticks
        .iter()
        .filter_map(|t| match t {
            Tick::Beat { at_sample, .. } => Some(*at_sample),
            _ => None,
        })
        .collect();
    assert_eq!(beats, vec![0, 32_000]);
}

#[test]
fn no_drift_over_10000_bars_at_93_bpm_44100() {
    let mut c = Clock::new(44_100, 93.0, Meter::FOUR_FOUR);
    // 10000 bars = 40000 beats; walk the whole span in advance chunks.
    let mut last_bar_at = 0u64;
    while c.bar() <= 10_000 {
        for t in c.advance(4096) {
            if let Tick::Bar { at_sample, .. } = t {
                last_bar_at = at_sample;
            }
        }
    }
    // Exact position of the last emitted bar boundary vs the ideal.
    let bar_len = 4.0 * 60.0 / 93.0 * 44_100.0;
    let ideal = bar_len * 10_000.0;
    assert!(
        (last_bar_at as f64 - ideal).abs() <= 1.0,
        "bar 10000 at {last_bar_at}, ideal {ideal}"
    );
}

#[test]
fn phrase_is_8_bars() {
    let mut c = Clock::new(48_000, 96.0, Meter::FOUR_FOUR);
    let mut bars = 0u64;
    let mut phrases = 0u64;
    // Run just past the second phrase boundary.
    while phrases < 2 {
        for t in c.advance(4096) {
            match t {
                Tick::Bar { .. } => bars += 1,
                Tick::Phrase { .. } => phrases += 1,
                _ => {}
            }
        }
    }
    // Phrase 0's downbeat is also bar 0; the next Phrase tick closes
    // the phrase after 8 more bars - 9 bar ticks bound it.
    assert_eq!(bars, 9);
}

#[test]
fn change_waits_for_bar_boundary() {
    let mut c = Clock::new(48_000, 90.0, Meter::FOUR_FOUR);
    c.advance(1); // now mid-bar, a bar boundary is not pending at pos
    c.request_tempo(120.0, Quantize::Bar);
    assert_eq!(c.tempo_bpm, 90.0);
    // Advance to the first bar boundary (4 beats of 32000 = 128000).
    c.advance(128_000);
    assert_eq!(c.tempo_bpm, 120.0);
}

#[test]
fn change_waits_for_phrase_boundary() {
    let mut c = Clock::new(48_000, 90.0, Meter::FOUR_FOUR);
    c.advance(1);
    c.request_tempo(100.0, Quantize::Phrase);
    c.advance(128_000); // one bar passed; phrase boundary still ahead
    assert_eq!(c.tempo_bpm, 90.0);
    // One full phrase = 8 bars of 4 beats = 1_024_000 samples at 90 bpm.
    c.advance(1_024_000 - 128_000);
    assert_eq!(c.tempo_bpm, 100.0);
}

#[test]
fn second_request_replaces_first() {
    let mut c = Clock::new(48_000, 90.0, Meter::FOUR_FOUR);
    c.advance(1);
    c.request_tempo(110.0, Quantize::Bar);
    c.request_tempo(80.0, Quantize::Bar);
    c.advance(128_000);
    assert_eq!(c.tempo_bpm, 80.0);
}

#[test]
fn meters_3_4_and_6_8_beat_counts() {
    let mut c = Clock::new(48_000, 90.0, Meter::THREE_FOUR);
    let mut beats = 0;
    let mut bars = 0;
    while bars < 2 {
        for t in c.advance(4096) {
            match t {
                Tick::Beat { .. } => beats += 1,
                Tick::Bar { .. } => bars += 1,
                _ => {}
            }
        }
    }
    assert_eq!(beats, 4); // beat 0 of bar 0, then 3 more beats in bar 0

    let mut c = Clock::new(48_000, 90.0, Meter::SIX_EIGHT);
    let mut beats = 0;
    let mut bars = 0;
    while bars < 2 {
        for t in c.advance(4096) {
            match t {
                Tick::Beat { .. } => beats += 1,
                Tick::Bar { .. } => bars += 1,
                _ => {}
            }
        }
    }
    assert_eq!(beats, 7); // beat 0, then 6 eighth-note beats in bar 0
}

#[test]
fn events_drain_in_order_and_exactly_once() {
    let mut c = Clock::new(48_000, 90.0, Meter::FOUR_FOUR);
    c.advance(1);
    c.schedule(ev(9_000, Voice::Melody));
    c.schedule(ev(9_000, Voice::Drone));
    c.schedule(ev(4_000, Voice::Pulse));
    c.schedule(ev(20_000, Voice::Choir));
    let due = c.drain_due(10_000);
    let order: Vec<(u64, Voice)> = due.iter().map(|e| (e.at_sample, e.voice)).collect();
    assert_eq!(
        order,
        vec![
            (4_000, Voice::Pulse),
            (9_000, Voice::Drone),
            (9_000, Voice::Melody),
        ]
    );
    assert!(c.drain_due(10_000).is_empty());
    let rest = c.drain_due(20_000);
    assert_eq!(rest.len(), 1);
}

#[test]
fn tempo_never_changes_mid_bar() {
    let mut c = Clock::new(48_000, 90.0, Meter::FOUR_FOUR);
    c.advance(1);
    c.request_tempo(200.0, Quantize::Bar);
    // Walk to just before the boundary: tempo must still read 90.
    c.advance(127_998); // pos = 127_999, one sample shy
    assert_eq!(c.tempo_bpm, 90.0);
    c.advance(1); // pos = 128_000, crosses the bar line exactly
    assert_eq!(c.tempo_bpm, 200.0);
}

#[test]
fn meter_changes_at_phrase_boundary_only() {
    let mut c = Clock::new(48_000, 90.0, Meter::FOUR_FOUR);
    c.advance(1);
    c.request_meter(Meter::THREE_FOUR);
    c.advance(128_000);
    assert_eq!(c.meter, Meter::FOUR_FOUR);
    c.advance(1_024_000 - 128_000);
    assert_eq!(c.meter, Meter::THREE_FOUR);
}

#[test]
fn late_events_drop_not_fire() {
    let mut c = Clock::new(48_000, 90.0, Meter::FOUR_FOUR);
    c.advance(10_000);
    c.schedule(ev(5_000, Voice::Melody));
    assert_eq!(c.dropped_late, 1);
    assert!(c.drain_due(10_000).is_empty());
}
