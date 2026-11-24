//! The AudioHost↔device bridge's proofs (#1494): a failing sink goes
//! silent instead of crashing, frames flow in order, a full queue drops
//! its oldest, M's flag zeroes the pump's output, and --no-audio never
//! opens a device.
use crate::*;
use game2_audio::host::{AudioFrame, AudioHost, AudioSink, CueKind, HostSettings, FRAME_QUEUE};
use game2_core::session::{Game, MAP_SEED};
use std::io;

/// A sink that records what it was asked to do: opens (or refuses),
/// and pumps rendered blocks into `collected` when driven.
struct MockSink {
    fail: bool,
    opened: usize,
    pump: Option<Box<dyn FnMut(&mut [f32]) + Send>>,
    pub collected: Vec<f32>,
}
impl MockSink {
    fn ok() -> Self { MockSink { fail: false, opened: 0, pump: None, collected: vec![] } }
    fn closed() -> Self { MockSink { fail: true, opened: 0, pump: None, collected: vec![] } }
    fn tick(&mut self, n: usize) {
        let mut buf = vec![0.0f32; n];
        self.pump.as_mut().unwrap()(&mut buf);
        self.collected.extend_from_slice(&buf);
    }
}
impl AudioSink for MockSink {
    fn sample_rate(&self) -> u32 { 44100 }
    fn open(&mut self, pump: Box<dyn FnMut(&mut [f32]) + Send>) -> Result<(), io::Error> {
        self.opened += 1;
        if self.fail { Err(io::Error::new(io::ErrorKind::Other, "no such device")) }
        else { self.pump = Some(pump); Ok(()) }
    }
}

fn frame_with(cue: &str, seed: u64) -> AudioFrame {
    let cue = match cue {
        "bell" => CueKind::Bell { pan: 0.0, gain: 0.8 },
        "rain" => CueKind::Rain { pan: 0.0, gain: 0.8 },
        _ => CueKind::Mood { pan: 0.0, gain: 0.8 },
    };
    AudioFrame { cues: vec![cue], emitters: vec![], seed, ..Default::default() }
}

#[test]
fn silent_host_when_no_device() {
    let mut sink = MockSink::closed();
    let mut host = AudioHost::start(&HostSettings::default(), &mut sink);
    assert!(host.is_silent(), "a failed open didn't fall back to silent");
    let mut buf = [0.0f32; 32];
    host.render_block(&mut buf);
    assert!(buf.iter().all(|&s| s == 0.0));
}

#[test]
fn frames_flow_to_sink_in_order() {
    let mut sink = MockSink::ok();
    let mut host = AudioHost::start(&HostSettings::default(), &mut sink);
    assert!(!host.is_silent());
    host.push_frame(&frame_with("bell", 1));
    host.push_frame(&frame_with("rain", 2));
    host.push_frame(&frame_with("other", 3));
    let q = host.live_queue().unwrap();
    let seeds: Vec<u64> = q.lock().unwrap().iter().map(|f| f.seed).collect();
    assert_eq!(seeds, [1, 2, 3], "frames queued out of order");
    drop(q);
    sink.tick(4096);
    assert!(sink.collected.iter().any(|&s| s != 0.0),
        "the pushed cues never reached the sink");
}

#[test]
fn full_channel_drops_oldest_not_blocks() {
    let mut sink = MockSink::ok();
    let mut host = AudioHost::start(&HostSettings::default(), &mut sink);
    for i in 0..FRAME_QUEUE + 3 {
        host.push_frame(&frame_with("rain", i as u64));
    }
    let q = host.live_queue().unwrap();
    let seeds: Vec<u64> = q.lock().unwrap().iter().map(|f| f.seed).collect();
    assert_eq!(seeds, [3, 4, 5, 6, 7, 8, 9, 10],
        "the queue didn't drop its oldest: {seeds:?}");
}

#[test]
fn mute_flag_zeroes_output() {
    let mut sink = MockSink::ok();
    let mut host = AudioHost::start(&HostSettings::default(), &mut sink);
    host.push_frame(&frame_with("bell", 1));
    host.set_muted(true);
    sink.tick(4096);
    assert!(sink.collected.iter().all(|&s| s == 0.0),
        "muted output wasn't silent");
    host.set_muted(false);
    sink.tick(4096);
    assert!(sink.collected.iter().any(|&s| s != 0.0), "unmute stayed silent");
}

#[test]
fn no_audio_flag_never_opens_device() {
    let mut sink = MockSink::ok();
    let host = audio::open_host(true, &HostSettings::default(), Some(&mut sink));
    assert!(host.is_silent());
    assert_eq!(sink.opened, 0, "--no-audio opened a device");
    // and the same call with audio wanted does open one
    let host = audio::open_host(false, &HostSettings::default(), Some(&mut sink));
    assert!(!host.is_silent());
    assert_eq!(sink.opened, 1);
}

#[test]
fn sfx_cues_play_for_the_real_game() {
    // a real UiModel's sfx_v becomes pump-able frames (the S3 voices)
    let mut g = Game::new_seeded(MAP_SEED);
    g.update();
    let m = g.ui_model();
    let mut sink = MockSink::ok();
    let mut host = AudioHost::start(&HostSettings::default(), &mut sink);
    host.push(&m);
    sink.tick(4096);
    assert_eq!(sink.collected.len(), 4096);
    // and a cue-bearing frame survives the queue into rendered audio
    let mut host2 = AudioHost::start(&HostSettings::default(), &mut sink);
    host2.push_frame(&frame_with("bell", 9));
    sink.tick(8192);
    let peak = sink.collected[4096..].iter().fold(0.0f32, |m, s| m.max(s.abs()));
    assert!(peak > 0.01, "a queued cue voice didn't reach the device");
}
