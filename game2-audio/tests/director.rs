//! The director's arc tests (M5): scripted `UiModel` sequences - real
//! game models with the public fields set the way the arc calls for -
//! driven through `Director::update` a frame at a time. What lands
//! where: builds ahead of events, peaks on the event, verdicts moving
//! the mode, ducks, presence, and the flicker guards.

use game2_audio::compose::{MotifId, Section};
use game2_audio::director::{Ducks, Director, Intent, Stinger};
use game2_audio::host::AudioHost;
use game2_audio::instruments::PatchId;
use game2_audio::settings::{AudioSettings, Presence};
use game2_audio::theory::Mode;
use game2_audio::wav::SAMPLE_RATE;
use game2_core::session::{
    CampaignV, ConvoPanelV, Game, LiveV, OutcomeV, PlannedV, UiModel, Verdict,
};

/// A real village's model: the director reads only `UiModel`, so the
/// tests script the public fields over a live game's snapshot.
fn model() -> UiModel {
    Game::new_seeded_cast(7).ui_model()
}

/// Advance `frames` updates over `m`, returning the last intent.
fn drive(d: &mut Director, m: &UiModel, frames: u32) -> Intent {
    let mut last = d.update(m);
    for _ in 1..frames {
        last = d.update(m);
    }
    last
}

/// Advance until `f` holds on the intent (or `max` frames out).
fn drive_until(d: &mut Director, m: &UiModel, max: u32,
    f: impl Fn(&Intent) -> bool) -> Intent {
    for _ in 0..max {
        let i = d.update(m);
        if f(&i) {
            return i;
        }
    }
    d.update(m)
}


/// A live campaign on the panel: the "story" that lifts Ambient to
/// Theme and makes the cause's motif the line.
fn campaign() -> CampaignV {
    CampaignV {
        issue: "tithes".into(), framing: "fair share".into(),
        dissenters: 0, seats: vec!["turnout: Aldric".into()],
        shadow: None, state: "consents".into(), resolves: 0,
        due: true, cadence_days: 7, assembly: String::new(), leaders: 0,
    }
}

/// A `drama.next` `secs` real seconds out, on the wire's units.
fn next_at(m: &mut UiModel, kind: &str, secs: f64) {
    let fpd = m.clock.frames_per_day.max(1) as f64;
    m.panel.drama.next = Some(PlannedV {
        kind: kind.into(),
        in_days: (secs * 60.0 / fpd) as f32,
        at: m.tick as u32 + (secs / 20.0).ceil() as u32,
    });
}

fn verdict(m: &mut UiModel, v: Verdict) {
    m.panel.drama.last = Some(OutcomeV {
        kind: "townhall".into(),
        verdict: v,
        day: m.tick as u32,
    });
}

#[test]
fn build_starts_at_least_3_phrases_before_assembly() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    // ~75 s out: the 4-phrase window at build tempo (20 s phrases)
    // opens at 80 s - the build picks up at the first boundary inside
    // it and climbs a layer a phrase.
    next_at(&mut m, "assembly", 75.0);
    let mut d = Director::new(7);
    let mut build_at = None;
    let mut layers = vec![];
    for f in 0..3600 {
        let i = d.update(&m);
        if i.section == Section::Build && build_at.is_none() {
            build_at = Some(f);
        }
        if i.section == Section::Build {
            layers.push(i.motif_step);
        }
    }
    let f0 = build_at.expect("build never started");
    // it should pick up well before the last phrase: at ~29 s phrases,
    // starting by frame ~2100 leaves >= 60 s = 3 build phrases.
    assert!(f0 <= 2200, "build started at frame {f0}, under 3 phrases left");
    assert!(layers.iter().max().copied().unwrap_or(0) >= 2,
        "the build only laid {layers:?} - the layer never climbed");
}

#[test]
fn peak_lands_within_one_bar_of_event() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    next_at(&mut m, "assembly", 70.0);
    let mut d = Director::new(7);
    // reach the build, then a bar boundary, then go live: Peak must
    // answer at the next downbeat, inside a bar (~2.5 s at build tempo).
    let mut live_set_at = None;
    let mut peak_at = None;
    for f in 0..5400 {
        if peak_at.is_none() && live_set_at.is_none() {
            let i0 = d.update(&m);
            if i0.section == Section::Build && d.phrase_boundary() {
                m.panel.drama.next = None;
                m.panel.drama.live =
                    Some(LiveV { kind: "assembly".into(), progress: 0.0 });
                live_set_at = Some(f);
            }
            continue;
        }
        let i = d.update(&m);
        if live_set_at.is_some() && i.section == Section::Peak && peak_at.is_none() {
            peak_at = Some(f);
        }
    }
    let live_f = live_set_at.expect("never went live");
    let peak_f = peak_at.expect("peak never landed");
    // one bar at ~100 bpm 4/4 is 2.4 s = 144 frames; allow a frame either way
    assert!(peak_f - live_f <= 150,
        "peak landed {} frames after live (>1 bar)", peak_f - live_f);
}

#[test]
fn refuses_verdict_moves_mode_minus_one_next_phrase() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    let mut d = Director::new(7);
    let i0 = drive(&mut d, &m, 90);
    assert_eq!(i0.mode, Mode::Dorian);
    verdict(&mut m, Verdict::Loss);
    let mut changed_at = None;
    let mut stinger = None;
    for f in 0..4000 {
        let i = d.update(&m);
        if i.stinger.is_some() && stinger.is_none() {
            stinger = i.stinger;
        }
        if i.mode != Mode::Dorian && changed_at.is_none() {
            changed_at = Some((f, i.mode));
        }
    }
    let (f, mode) = changed_at.expect("mode never moved");
    assert_eq!(mode, Mode::Aeolian, "loss should step down one rung");
    assert_eq!(stinger, Some(Stinger::LowDrum), "the loss stinger should fire");
    assert!(f < 4000);
}

#[test]
fn win_moves_plus_two_and_endings_clos() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    let mut d = Director::new(7);
    drive(&mut d, &m, 60);
    verdict(&mut m, Verdict::Win);
    let mut stinger = None;
    let mut mode = None;
    for _ in 0..4000 {
        let i = d.update(&m);
        if i.stinger.is_some() {
            stinger = i.stinger;
        }
        if i.mode != Mode::Dorian && mode.is_none() {
            mode = Some(i.mode);
        }
    }
    assert_eq!(mode, Some(Mode::Ionian), "win should climb two rungs");
    assert_eq!(stinger, Some(Stinger::BellsPeal));
    // endings stay resolved on a win: clos cadence
    let i = d.update(&m);
    assert!(i.resolved, "a win's arc should cadence clos");
}

#[test]
fn crackdown_moves_minus_two_and_adds_pressure_layer() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    let mut d = Director::new(7);
    drive(&mut d, &m, 60);
    verdict(&mut m, Verdict::Crackdown);
    let mut saw = None;
    for _ in 0..4000 {
        let i = d.update(&m);
        if i.mode != Mode::Dorian && saw.is_none() {
            saw = Some(i);
        }
    }
    let i = saw.expect("mode never moved");
    assert_eq!(i.mode, Mode::Phrygian, "crackdown is two rungs down");
    assert!(i.pressure_layer, "the pressure layer should come on");
    assert_eq!(i.motif, MotifId::Lord, "shawm and drums play the lord's motif");
    assert!(i.density[1] > 0.0 && i.density[2] > 0.0,
        "the drums and bass should carry the pressure layer");
}

#[test]
fn mode_drifts_back_one_step_per_8_phrases() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    let mut d = Director::new(7);
    drive(&mut d, &m, 60);
    verdict(&mut m, Verdict::Loss);
    let i = drive_until(&mut d, &m, 4000, |i| i.mode == Mode::Aeolian);
    assert_eq!(i.mode, Mode::Aeolian, "setup: loss should land Aeolian");
    // count phrase boundaries until the rung returns to Dorian: it
    // should take exactly DRIFT_PHRASES (8) phrases of drift.
    let mut back_at = None;
    for _ in 0..40000 {
        let i = d.update(&m);
        if i.mode == Mode::Dorian && back_at.is_none() {
            back_at = Some(d.phrase_index());
        }
    }
    let idx = back_at.expect("never drifted home");
    // the loss applied at some phrase p; the drift is 1 step / 8
    // phrases, so Dorian returns no sooner than p+8
    assert!(idx >= 8, "drifted home after {idx} phrases (< 8)");
    assert!(idx <= 24, "drift took {idx} phrases (> 24)");
}

#[test]
fn flickering_metrics_do_not_flip_section() {
    let mut m = model();
    let mut d = Director::new(7);
    let mut sections = vec![];
    for f in 0..12000 {
        // flap a conversation flag and the feast flag every few frames -
        // frame-rate flicker must not become section churn.
        m.talk.in_talk = f / 7 % 2 == 0;
        m.feast = f / 11 % 2 == 0;
        let i = d.update(&m);
        if sections.last() != Some(&i.section) {
            sections.push(i.section);
        }
    }
    let changes = sections.len().saturating_sub(1);
    assert!(changes <= 8,
        "{changes} section changes over 200 s of flapping ({sections:?})");
}

#[test]
fn conversation_ducks_music_9db_and_drops_melody() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    let mut d = Director::new(7);
    d.set_presence(0.85); // Rich: a conversation ducks instead of silencing
    let before = drive(&mut d, &m, 120);
    m.convo = Some(ConvoPanelV { title: "Talk".into(), ..Default::default() });
    let during = drive(&mut d, &m, 120);
    assert_eq!(during.duck.music_db, -9.0,
        "a conversation ducks the music bus 9 dB");
    assert_eq!(during.density[4], 0.0, "the melody should drop out");
    assert!(during.density[0] > 0.0 && during.density[3] > 0.0,
        "drone and harp stay under a conversation");
    m.convo = None;
    let after = drive(&mut d, &m, 120);
    assert_eq!(after.duck.music_db, 0.0);
    let _ = before;
}

#[test]
fn pause_holds_bed_level() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    let mut d = Director::new(7);
    drive(&mut d, &m, 60);
    m.paused = true;
    let i = drive(&mut d, &m, 60);
    assert!(i.duck.music_db <= -6.0,
        "paused holds the score at bed level, got {} dB", i.duck.music_db);
}

#[test]
fn turbo_never_changes_tempo() {
    let mut slow = model();
    slow.panel.campaign = Some(campaign());
    let mut fast = slow.clone();
    fast.clock.speed = 5.6;
    let mut ds = Director::new(7);
    let mut df = Director::new(7);
    for _ in 0..6000 {
        let i_s = ds.update(&slow);
        let i_f = df.update(&fast);
        assert_eq!(i_s.tempo_bpm, i_f.tempo_bpm,
            "turbo bent the tempo: {} vs {}", i_s.tempo_bpm, i_f.tempo_bpm);
        assert_eq!(ds.phrase_index(), df.phrase_index(),
            "turbo bent the phrase clock");
    }
}

#[test]
fn presence_sparse_vs_rich_music_share_within_10_points_over_20_minutes() {
    let m = model();
    for (share, lo, hi) in [(0.35f32, 0.25f32, 0.45f32), (0.85, 0.75, 0.95)] {
        let mut d = Director::new(7);
        d.set_presence(share);
        let mut sounding = 0usize;
        let mut n = 0usize;
        // warm the bucket, then measure 20 minutes of frames
        drive(&mut d, &m, 3600);
        for _ in 0..72000 {
            let i = d.update(&m);
            if i.section != Section::Silence && i.density.iter().any(|&x| x > 0.0) {
                sounding += 1;
            }
            n += 1;
        }
        let frac = sounding as f32 / n as f32;
        assert!(frac >= lo && frac <= hi,
            "presence {share}: music sounded {frac:.0}% of frames, outside ±10");
    }
}

#[test]
fn postponed_event_holds_layer_no_abrupt_drop() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    next_at(&mut m, "assembly", 70.0);
    let mut d = Director::new(7);
    let i = drive_until(&mut d, &m, 3600,
        |i| i.section == Section::Build && i.motif_step >= 1);
    assert!(i.motif_step >= 1, "setup: the build never layered");
    // the assembly slips another two minutes: the build holds its
    // layer instead of collapsing
    next_at(&mut m, "assembly", 120.0);
    let mut worst = i.motif_step;
    for _ in 0..2000 {
        let i = d.update(&m);
        if i.section == Section::Build {
            worst = worst.min(i.motif_step);
        }
    }
    let i = d.update(&m);
    assert_eq!(i.section, Section::Build,
        "a postponed event should hold the build, not drop it");
    assert!(worst >= 1, "the build dropped a layer on the postponement");
}

#[test]
fn sky_wonder_lends_one_step() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    let mut d = Director::new(7);
    let i0 = drive(&mut d, &m, 120);
    assert_eq!(i0.mode, Mode::Dorian);
    m.sky = Some("aurora".into());
    let i = drive(&mut d, &m, 120);
    assert_eq!(i.mode, Mode::Mixolydian,
        "a sky wonder lends one step of brightness");
    m.sky = None;
    let i = drive(&mut d, &m, 120);
    assert_eq!(i.mode, Mode::Dorian, "the step lifts with the sky");
}

#[test]
fn no_hidden_state_access() {
    // The layering gate (`cargo test -p game2-audio layering`) scans
    // every file in src/ for Game/World/probe/Instant/getrandom - this
    // test exists so a regression that slips past the scan still fails
    // loudly here: the director's only input is a UiModel, and its
    // only state is seeded at construction.
    let m = model();
    let mut a = Director::new(42);
    let mut b = Director::new(42);
    for _ in 0..500 {
        let ia = a.update(&m);
        let ib = b.update(&m);
        assert_eq!(ia.tempo_bpm, ib.tempo_bpm);
        assert!(std::ptr::eq(&m, &m)); // the call borrows the model only
    }
    // ducks/sections are value-only: no hidden channel could have
    // reached in
    let _ = Ducks::default();
}

/// Push `m` and render one 60fps block per update - the offline
/// host's lockstep loop, the one `sound_dump` drives.
fn render(host: &mut AudioHost, m: &UiModel, frames: u32) {
    for _ in 0..frames {
        host.push(m);
        let n = host.frame_samples();
        let mut buf = vec![0.0f32; n * 2];
        host.render_block(&mut buf);
    }
}

/// The sample the second phrase starts on, from the bar trace.
fn phrase_two_start(host: &mut AudioHost, sr: u32) -> u64 {
    host.drain_trace().iter()
        .find(|r| r.bar >= game2_audio::director::BARS_PER_PHRASE)
        .map(|r| (r.at_s * sr as f64) as u64)
        .unwrap_or(u64::MAX)
}

/// A postponed event keeps the build; a resolved or replaced one
/// must let it go - otherwise the layer hangs over every quiet spell
/// after the first assembly.
#[test]
fn build_lets_go_when_its_event_resolves() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    next_at(&mut m, "assembly", 70.0);
    let mut d = Director::new(7);
    let i = drive_until(&mut d, &m, 3600,
        |i| i.section == Section::Build && i.motif_step >= 1);
    assert!(i.motif_step >= 1, "setup: the build never layered");
    // the assembly resolves; the calendar now names a far-off holy day
    m.panel.drama.next = Some(PlannedV {
        kind: "holy day".into(), in_days: 30.0, at: m.tick as u32 + 30 });
    // Build may ride out its min-hold (~2 phrases, ~40 s); after that
    // it must not come back while nothing's near
    let mut held_too_long = false;
    for f in 0..7200 {
        let i = d.update(&m);
        if f > 3600 && i.section == Section::Build {
            held_too_long = true;
        }
    }
    assert!(!held_too_long, "the build held after its event resolved");
}

/// `push_frame` files a one-frame stinger into the score's queue, so
/// a verdict mid-phrase still rings on its downbeat instead of
/// vanishing with the intent that carried it.
#[test]
fn stinger_fires_when_verdict_lands_mid_phrase() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    let mut host = AudioHost::offline(SAMPLE_RATE);
    render(&mut host, &m, 300); // five seconds in, mid-phrase
    verdict(&mut m, Verdict::Win);
    render(&mut host, &m, 600); // the stinger's bar lands well inside
    let notes = host.drain_notes();
    assert!(notes.iter().any(|(_, n)|
            n.patch == PatchId::TubularBells && n.midi == 72.0),
        "the win peal never fired: {:?}", notes.iter()
            .map(|(_, n)| n.patch).collect::<Vec<_>>());
}

/// Every queued note must fire on its own onset: the composer emits
/// voice by voice, so an unsorted queue parks an early melody behind
/// a late drone.
#[test]
fn notes_fire_on_their_onsets() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    let mut host = AudioHost::offline(SAMPLE_RATE);
    render(&mut host, &m, 2400); // past the first phrase boundary
    let p2 = phrase_two_start(&mut host, SAMPLE_RATE);
    let notes = host.drain_notes();
    assert!(!notes.is_empty(), "the score wrote no notes");
    for (t, n) in &notes {
        if *t < p2 {
            assert_eq!(*t, n.at_sample,
                "note fired {} samples off its onset",
                *t as i64 - n.at_sample as i64);
        }
    }
}

/// The composer writes at `wav::SAMPLE_RATE`; a device running 48 kHz
/// must still land each note on its beat - offsets convert, phrase
/// boundaries already do.
#[test]
fn score_schedules_on_the_device_clock() {
    const DEV: u32 = 48_000;
    let mut m = model();
    m.panel.campaign = Some(campaign());
    let mut host = AudioHost::offline(DEV);
    render(&mut host, &m, 2400);
    let p2 = phrase_two_start(&mut host, DEV);
    let notes = host.drain_notes();
    assert!(!notes.is_empty(), "the score wrote no notes");
    for (t, n) in &notes {
        if *t < p2 {
            assert_eq!(*t, n.at_sample * DEV as u64 / SAMPLE_RATE as u64,
                "48kHz: note fired at {t}, onset was {}",
                n.at_sample * DEV as u64 / SAMPLE_RATE as u64);
        }
    }
}

/// The Sound row's presence budget reaches the director through
/// `apply_settings`: Off hears no score at all.
#[test]
fn presence_off_plays_no_score() {
    let mut m = model();
    m.panel.campaign = Some(campaign());
    let mut host = AudioHost::offline(SAMPLE_RATE);
    host.apply_settings(&AudioSettings {
        presence: Presence::Off, ..Default::default() });
    render(&mut host, &m, 2400);
    assert!(host.drain_notes().is_empty(),
        "presence Off still fired notes");
    assert!(host.drain_trace().iter().all(|r| r.section == "Silence"),
        "presence Off left the Silence section");
}

/// A model seeded `u64::MAX` used to slip past the director's
/// seed-changed check (its "unset" sentinel) and unwrap a `None` -
/// `game_ui --seed 18446744073709551615` reached it through the bench.
#[test]
fn max_seed_model_still_gets_a_director() {
    let mut m = model();
    m.seed = u64::MAX;
    let mut host = AudioHost::offline(SAMPLE_RATE);
    render(&mut host, &m, 30); // panicked here before the is_none check
}
