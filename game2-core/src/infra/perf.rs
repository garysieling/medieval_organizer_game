//! Perf telemetry: where frame time goes, section by section.
//!
//! This measures the engine, not the game. It is separate from the game KPIs
//! (`Game::telemetry`, the probe). It's off unless a bench turns it on
//! (`GAME2_BENCH=N`, see `session::render::bench`), and when off a section
//! costs one thread-local read.
//!
//! Sections are named by layer: `sim.*` for the daily ticks, `ecs.*` for
//! scheduler systems, `render.*` for the View's passes. A section may run
//! many times a frame (its times add up) or not at all (a day only ticks
//! every few hundred frames, so `sim.*` sections show up as rare spikes;
//! the p95/max columns are what catch them).
//!
//! ```ignore
//! let _s = perf::scope("sim.tavern");   // timed until _s drops
//! let mut m = perf::Marks::new();       // or consecutive passes:
//! m.mark("render.ground"); m.mark("render.sprites");
//! perf::frame_end();                    // once per frame, by the bench
//! ```

use std::cell::{Cell, RefCell};
use std::time::Instant;

#[derive(Default)]
struct Store {
    /// This frame's time per section (ms), in first-seen order
    frame: Vec<(&'static str, f64)>,
    /// Per section, one entry per finished frame it ran in
    hist: Vec<(&'static str, Vec<f64>)>,
    frames: u32,
}

thread_local! {
    // per thread: the bench's sim and render run on its thread, and tests
    // running in parallel can't switch each other's recording on or off
    static ON: Cell<bool> = const { Cell::new(false) };
    static STORE: RefCell<Store> = RefCell::new(Store::default());
}

/// Whether sections record right now
#[inline]
pub fn on() -> bool { ON.with(|o| o.get()) }

/// Start recording, clearing anything from an earlier run
pub fn start() {
    STORE.with(|s| *s.borrow_mut() = Store::default());
    ON.with(|o| o.set(true));
}

/// Stop recording and hand back the run's summary
pub fn stop() -> Summary {
    // a frame left open (recorded into, no frame_end yet) still counts
    if STORE.with(|s| !s.borrow().frame.is_empty()) { frame_end(); }
    ON.with(|o| o.set(false));
    STORE.with(|s| summarize(&s.borrow()))
}

/// Add `ms` to section `name` for this frame
pub fn record(name: &'static str, ms: f64) {
    if !on() { return; }
    STORE.with(|s| {
        let mut s = s.borrow_mut();
        match s.frame.iter_mut().find(|e| e.0 == name) {
            Some(e) => e.1 += ms,
            None => s.frame.push((name, ms)),
        }
    });
}

/// Close the frame: this frame's section times join the history
pub fn frame_end() {
    if !on() { return; }
    STORE.with(|s| {
        let mut s = s.borrow_mut();
        let frame = std::mem::take(&mut s.frame);
        for (name, ms) in frame {
            match s.hist.iter_mut().find(|e| e.0 == name) {
                Some(e) => e.1.push(ms),
                None => s.hist.push((name, vec![ms])),
            }
        }
        s.frames += 1;
    });
}

/// Time an expression as section `name`: `timed!("sim.tavern", self.tavern_tick(tick))`
#[macro_export]
macro_rules! timed {
    ($name:expr, $e:expr) => {{ let _s = $crate::infra::perf::scope($name); $e }};
}

/// Times its section until dropped
pub struct Scope(Option<(&'static str, Instant)>);

/// Time a section until the returned guard drops
#[inline]
pub fn scope(name: &'static str) -> Scope {
    Scope(on().then(|| (name, Instant::now())))
}

impl Drop for Scope {
    fn drop(&mut self) {
        if let Some((name, t)) = self.0.take() {
            record(name, t.elapsed().as_secs_f64() * 1e3);
        }
    }
}

/// Consecutive passes: each `mark` closes the section since the last one
pub struct Marks(Instant);

impl Default for Marks {
    fn default() -> Self { Marks::new() }
}

impl Marks {
    pub fn new() -> Marks { Marks(Instant::now()) }
    pub fn mark(&mut self, name: &'static str) {
        if !on() { return; }
        let now = Instant::now();
        record(name, (now - self.0).as_secs_f64() * 1e3);
        self.0 = now;
    }
}

/// One section's numbers over a run (ms)
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SectionStats {
    pub name: &'static str,
    /// Frames the section ran in
    pub frames: u32,
    /// Mean over every frame of the run (frames it didn't run in count 0),
    /// so the means add up to the frame
    pub mean_per_frame: f64,
    /// Percentiles over the frames it ran in
    pub p50: f64,
    pub p95: f64,
    pub max: f64,
    pub total: f64,
}

/// A run's sections, slowest total first
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Summary {
    pub frames: u32,
    pub sections: Vec<SectionStats>,
}

/// Percentile `p` (0-1) of sorted samples, nearest-rank
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() { return 0.0; }
    let rank = (p * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

/// Stats over a list of samples (ms)
pub fn stats(name: &'static str, samples: &[f64], frames: u32) -> SectionStats {
    let mut v = samples.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let total: f64 = v.iter().sum();
    SectionStats {
        name,
        frames: v.len() as u32,
        mean_per_frame: total / frames.max(1) as f64,
        p50: percentile(&v, 0.50),
        p95: percentile(&v, 0.95),
        max: v.last().copied().unwrap_or(0.0),
        total,
    }
}

/// How a section prints: ECS systems record under their type path
/// (`game2_core::game::villager::SeekFoodSystem`) and print as `ecs.SeekFoodSystem`
pub fn display_name(name: &str) -> String {
    match name.rsplit_once("::") {
        Some((_, last)) => format!("ecs.{}", last),
        None => name.to_string(),
    }
}

fn summarize(s: &Store) -> Summary {
    let mut sections: Vec<SectionStats> = s.hist.iter().map(|(n, v)| stats(n, v, s.frames)).collect();
    sections.sort_by(|a, b| b.total.total_cmp(&a.total));
    Summary { frames: s.frames, sections }
}

impl Summary {
    /// Sections whose name starts with `prefix` (e.g. "render.")
    pub fn layer<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = &'a SectionStats> + 'a {
        self.sections.iter().filter(move |s| s.name.starts_with(prefix))
    }

    /// A fixed-width table, `top` slowest sections (by total)
    pub fn table(&self, top: usize) -> String {
        let mut out = format!("{:<28} {:>6} {:>9} {:>8} {:>8} {:>8}\n",
            format!("section ({} frames)", self.frames), "frames", "ms/frame", "p50", "p95", "max");
        for s in self.sections.iter().take(top) {
            out += &format!("{:<28} {:>6} {:>9.3} {:>8.3} {:>8.3} {:>8.3}\n",
                display_name(s.name), s.frames, s.mean_per_frame, s.p50, s.p95, s.max);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_are_nearest_rank() {
        let v: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        assert_eq!(percentile(&v, 0.5), 50.0);
        assert_eq!(percentile(&v, 0.95), 95.0);
        assert_eq!(percentile(&v, 1.0), 100.0);
        assert_eq!(percentile(&[7.0], 0.95), 7.0);
        assert_eq!(percentile(&[], 0.5), 0.0);
    }

    #[test]
    fn sections_record_only_while_on_and_sum_within_a_frame() {
        // off: nothing records
        record("x", 5.0);
        start();
        record("a", 1.0);
        record("a", 2.0); // same frame: adds up
        frame_end();
        record("a", 4.0);
        record("b", 10.0); // b runs in one frame of two
        let s = stop();
        assert_eq!(s.frames, 2);
        let a = s.sections.iter().find(|x| x.name == "a").unwrap();
        assert_eq!((a.frames, a.max, a.total), (2, 4.0, 7.0));
        let b = s.sections.iter().find(|x| x.name == "b").unwrap();
        assert_eq!(b.frames, 1);
        assert_eq!(b.mean_per_frame, 5.0); // 10ms over 2 frames
        assert!(s.sections.iter().all(|x| x.name != "x"));
        assert_eq!(s.sections[0].name, "b"); // slowest first
        // off again: a scope costs nothing and records nothing
        { let _g = scope("c"); }
        start();
        let empty = stop();
        assert!(empty.sections.is_empty());
        assert_eq!(empty.frames, 0, "an empty open frame isn't a frame");
        assert_eq!(display_name("game2_core::game::villager::SeekFoodSystem"), "ecs.SeekFoodSystem");
        assert_eq!(display_name("sim.tavern"), "sim.tavern");
    }
}
