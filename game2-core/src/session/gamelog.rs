//! The game's event log: the rolling window of "t<tick>: ..." lines, plus
//! for each line the click-through meta a player gets on the LOG panel -
//! where it happened and/or who it was about (#645). The meta vecs stay
//! index-aligned with `lines`; every write goes through the `push_*`
//! methods so they can't drift. Reads deref to the plain lines, so the
//! ~280 `log.iter()` / `contains` / index sites and the string-matching
//! filters keep working untouched.

use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// One log line's click-through: a map position (events about a place)
/// and/or a villager index (events about someone, same convention as
/// `TalkRowV.who` / `Command::Select`). Snapshotted at push time -
/// villagers keep walking afterwards.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogMeta {
    pub at: Option<(f32, f32)>,
    pub who: Option<usize>,
}

/// Log lines with their click-through meta, index-aligned
#[derive(Clone, Debug, Default)]
pub struct GameLog {
    lines: Vec<String>,
    at: Vec<Option<(f32, f32)>>,
    who: Vec<Option<usize>>,
}

impl GameLog {
    /// A plain line: text only, nothing to click through to
    pub fn push(&mut self, line: String) {
        self.push_meta(line, None, None);
    }

    /// A line about a place: the camera jumps there on click
    pub fn push_at(&mut self, line: String, at: (f32, f32)) {
        self.push_meta(line, Some(at), None);
    }

    /// A line about a villager: click selects them (and jumps if `at`)
    pub fn push_who(&mut self, line: String, who: usize) {
        self.push_meta(line, None, Some(who));
    }

    /// A line with both: villager `who` standing at `at` when it happened
    pub fn push_at_who(&mut self, line: String, at: Option<(f32, f32)>, who: Option<usize>) {
        self.push_meta(line, at, who);
    }

    fn push_meta(&mut self, line: String, at: Option<(f32, f32)>, who: Option<usize>) {
        self.lines.push(line);
        self.at.push(at);
        self.who.push(who);
    }

    /// The click-through meta for line `i`, parallel to `lines`
    pub fn meta(&self, i: usize) -> LogMeta {
        LogMeta { at: self.at[i], who: self.who[i] }
    }

    /// Drop `n` lines off the front, meta in lockstep (the LOG_MAX roll)
    pub fn drop_front(&mut self, n: usize) {
        self.lines.drain(..n);
        self.at.drain(..n);
        self.who.drain(..n);
    }

    /// Empty the log, meta in lockstep (tests that reset a game)
    pub fn clear(&mut self) {
        self.lines.clear();
        self.at.clear();
        self.who.clear();
    }
}

/// Read-only view as the plain lines: `iter`, `last`, `len`, `contains`,
/// indexing and `&log[..]`-as-slice sites all keep working. No `DerefMut`:
/// writes only go through the `push_*`/`drop_front` methods that keep the
/// meta vecs aligned.
impl Deref for GameLog {
    type Target = Vec<String>;
    fn deref(&self) -> &Vec<String> { &self.lines }
}

/// Replay determinism (journal.rs): the meta derives from positions the
/// hash already covers, so only the lines themselves hash
impl Hash for GameLog {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lines.hash(state);
    }
}
