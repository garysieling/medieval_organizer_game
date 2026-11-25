//! Sound-caption tests for #1508 (U4): the feed's on/off gate, the
//! radius, the nearest-three cap and the three-second life. Names are
//! as the ticket requires.
use crate::*;
use game2_audio::soundscape::{Emitter, EmitterKind, Listener, Tier};

/// A signal emitter at a tile spot, as `scan` would report it
fn sig(kind: EmitterKind, x: f32, y: f32) -> Emitter {
    Emitter { kind, tier: Tier::Signal, pos: Some((x, y)), weight: 0.5,
        gain: 0.5, pan: 0.0, lowpass_hz: 16000.0, reverb: 0.0,
        rate_hz: 0.3, min_gap_s: 1.0 }
}

fn ear() -> Listener { Listener::at(10.0, 10.0, 1.0) } // radius 10

#[test]
fn no_captions_when_off() {
    let mut c = Captions::default();
    c.update(&[sig(EmitterKind::Smithy, 11.0, 10.0)], &ear(), 0, false);
    assert!(c.rows().is_empty(), "captions off draws nothing");
    // and nothing is tracked while off: flipping on names what's
    // sounding now, not a stale memory
    c.update(&[sig(EmitterKind::Smithy, 11.0, 10.0)], &ear(), 400, true);
    assert_eq!(c.rows().len(), 1, "the smithy captions when it's enabled");
}

#[test]
fn caption_shows_for_signal_in_radius() {
    let mut c = Captions::default();
    c.update(&[sig(EmitterKind::Smithy, 11.0, 10.0)], &ear(), 0, true);
    assert_eq!(c.rows(), &[("Hammering".to_string(), 1.0)]);
    // a soundmark captions too - the church bell is the design's example
    let mut c = Captions::default();
    let bell = Emitter { tier: Tier::Soundmark, ..sig(EmitterKind::ChurchBell, 12.0, 10.0) };
    c.update(&[bell], &ear(), 0, true);
    assert_eq!(c.rows()[0].0, "Alarm bell - church");
}

#[test]
fn no_caption_outside_radius() {
    let mut c = Captions::default();
    // the smithy rings 40 tiles off - a 10-tile ear hears nothing
    c.update(&[sig(EmitterKind::Smithy, 50.0, 10.0)], &ear(), 0, true);
    assert!(c.rows().is_empty(), "a signal past the radius stays off");
}

#[test]
fn at_most_three_captions() {
    let mut c = Captions::default();
    let es = [
        sig(EmitterKind::Fold, 20.0, 10.0),       // d 10
        sig(EmitterKind::Tavern, 16.0, 10.0),     // d 6
        sig(EmitterKind::Cart, 12.0, 10.0),       // d 2
        sig(EmitterKind::Graveyard, 18.0, 10.0),  // d 8
        sig(EmitterKind::Gate, 14.0, 10.0),       // d 4
    ];
    c.update(&es, &ear(), 0, true);
    let texts: Vec<&str> = c.rows().iter().map(|r| r.0.as_str()).collect();
    assert_eq!(texts, vec!["Cart wheels", "The gate creaks", "A lute at the tavern"],
        "the nearest three only: {texts:?}");
}

#[test]
fn captions_expire_after_3s() {
    let mut c = Captions::default();
    let es = [sig(EmitterKind::Smithy, 11.0, 10.0)];
    c.update(&es, &ear(), 0, true);
    assert_eq!(c.rows().len(), 1, "the caption lands");
    c.update(&es, &ear(), 179, true);
    assert_eq!(c.rows().len(), 1, "just inside 3 s it still shows");
    c.update(&es, &ear(), 180, true);
    assert!(c.rows().is_empty(), "at 3 s it's gone: {:?}", c.rows());
    // and it stays gone while the signal stays in earshot - a smithy
    // doesn't re-announce itself every few seconds
    c.update(&es, &ear(), 400, true);
    assert!(c.rows().is_empty(), "no re-arm while still audible");
    // it leaves, it returns: that's a new caption
    c.update(&[], &ear(), 500, true);
    c.update(&es, &ear(), 600, true);
    assert_eq!(c.rows().len(), 1, "a signal that left and came back captions anew");
}

/// The live wiring: a real village's scan reaches the feed - near the
/// player something is almost always sounding (homesteads, the fold,
/// a smithy), and a far corner of the map hears nothing.
#[test]
fn live_scan_feeds_captions() {
    let mut g = Game::new_seeded(MAP_SEED);
    for _ in 0..60 { g.update(); }
    let m = g.ui_model();
    let ear = Listener::from_model(&m);
    let es = game2_audio::soundscape::scan(&m, &ear);
    let mut c = Captions::default();
    c.update(&es, &ear, 0, true);
    assert!(!c.rows().is_empty(), "nothing captionable in earshot: {es:?}");
    // somewhere the map goes quiet: find such a spot (the boats keep a
    // lot of river audible) and check the feed stays empty there
    let mut quiet = None;
    'seek: for x in (2..m.map.width as i32).step_by(7) {
        for y in (2..m.map.height as i32).step_by(7) {
            let l = Listener::at(x as f32, y as f32, 1.0);
            let es = game2_audio::soundscape::scan(&m, &l);
            if !es.iter().any(|e| e.tier != Tier::Bed && e.pos.is_some()) {
                quiet = Some(l); break 'seek;
            }
        }
    }
    let l = quiet.expect("the map should have a quiet spot somewhere");
    let mut c2 = Captions::default();
    let es = game2_audio::soundscape::scan(&m, &l);
    c2.update(&es, &l, 0, true);
    assert!(c2.rows().is_empty(), "the quiet spot shouldn't hear: {es:?}");
}

/// Same-kind signals collapse to one chip: three smithies in earshot
/// caption "Hammering" once, and the slot goes to a different sound.
#[test]
fn same_kind_captions_once() {
    let mut c = Captions::default();
    let es = [
        sig(EmitterKind::Smithy, 11.0, 10.0),
        sig(EmitterKind::Smithy, 12.0, 10.0),
        sig(EmitterKind::Cart, 13.0, 10.0),
    ];
    c.update(&es, &ear(), 0, true);
    let texts: Vec<&str> = c.rows().iter().map(|r| r.0.as_str()).collect();
    assert_eq!(texts, vec!["Hammering", "Cart wheels"], "{texts:?}");
}
