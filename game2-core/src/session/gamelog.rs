//! The game's event log: the rolling window of "t<tick>: ..." lines, plus
//! for each line the click-through meta a player gets on the LOG panel -
//! where it happened and/or who it was about (#645) - and the line's story
//! tag (#1132): which braid it belongs to, what it did to the player's
//! day, and how heavily it lands. The meta vecs stay index-aligned with
//! `lines`; every write goes through the `push_*` methods so they can't
//! drift. Reads deref to the plain lines, so the ~280 `log.iter()` /
//! `contains` / index sites and the string-matching filters keep working
//! untouched.
//!
//! Every `t<tick>:` line is expected to carry an `EvTag` - the balance
//! tester scores runs off the tags, not the strings, so an untagged
//! timestamped line is invisible to it (and `tests/log_tags.rs` fails).
//! Plain `push` is for lines without a tick stamp: disclaimers, hints.

use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// Which story a line belongs to - the braids a narrative score counts.
/// Written at log time by the village itself; the balance tester reads
/// the tag, not the phrasing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Thread {
    Weather,
    Illness,
    Fire,
    Law,
    Beast,
    Visitor,
    Org,
    Economy,
    Lifecycle,
    Craft,
    Player,
    Other,
}

/// What the line does to a player's day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Valence {
    Positive,
    Negative,
    Neutral,
}

/// How heavily the line lands: patter, a beat, a headline. Feeds
/// severity-weighted scoring (a rainbow is not a writ) and, later, the
/// LOG panel's own pinning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sev {
    Minor,
    Notable,
    Major,
}

/// A log line's story tag: its braid, its charge, its weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EvTag {
    pub thread: Thread,
    pub valence: Valence,
    pub sev: Sev,
}

impl EvTag {
    /// `EvTag::new(Thread::Law, Valence::Negative, Sev::Major)`
    pub const fn new(thread: Thread, valence: Valence, sev: Sev) -> Self {
        EvTag {
            thread,
            valence,
            sev,
        }
    }
    /// A line that moves the needle - the scorer's beat rule: charged,
    /// and real news (not weather patter, not command feedback, not
    /// pacing context).
    pub fn is_beat(&self) -> bool {
        self.valence != Valence::Neutral
            && !matches!(
                self.thread,
                Thread::Weather | Thread::Player | Thread::Other
            )
    }

    /// Tag a story-system kind so cascade beats, mysteries and recorded
    /// events land in the scorer's vocabulary without a second classifier.
    pub fn for_kind(k: crate::sim::story::EventKind) -> EvTag {
        use crate::sim::story::EventKind as K;
        match k {
            K::Birth => EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Notable),
            K::Death => EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Major),
            K::Illness => EvTag::new(Thread::Illness, Valence::Negative, Sev::Notable),
            K::Recovery => EvTag::new(Thread::Illness, Valence::Positive, Sev::Notable),
            K::Peace | K::Bonding | K::Discovery | K::Achievement => {
                EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Minor)
            }
            K::Betrayal | K::Conflict => {
                EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Minor)
            }
            K::Recruitment | K::Alliance => EvTag::new(Thread::Org, Valence::Positive, Sev::Minor),
            K::Rejection => EvTag::new(Thread::Org, Valence::Negative, Sev::Minor),
            K::Hardship => EvTag::new(Thread::Economy, Valence::Negative, Sev::Notable),
            K::Unrest => EvTag::new(Thread::Law, Valence::Negative, Sev::Notable),
        }
    }
}

/// One log line's click-through: a map position (events about a place)
/// and/or a villager index (events about someone, same convention as
/// `TalkRowV.who` / `Command::Select`). Snapshotted at push time -
/// villagers keep walking afterwards.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogMeta {
    pub at: Option<(f32, f32)>,
    pub who: Option<usize>,
}

/// Log lines with their click-through and story-tag meta, index-aligned
#[derive(Clone, Debug, Default)]
pub struct GameLog {
    lines: Vec<String>,
    at: Vec<Option<(f32, f32)>>,
    who: Vec<Option<usize>>,
    ev: Vec<Option<EvTag>>,
}

impl GameLog {
    /// A plain line: text only, nothing to click through to, no story
    /// tag. For lines without a `t<tick>:` stamp - timestamped lines go
    /// through `push_ev*` so the scorer can read them.
    pub fn push(&mut self, line: String) {
        self.push_full(line, None, None, None);
    }

    /// A line about a place: the camera jumps there on click
    pub fn push_at(&mut self, line: String, at: (f32, f32)) {
        self.push_full(line, Some(at), None, None);
    }

    /// A line about a villager: click selects them (and jumps if `at`)
    pub fn push_who(&mut self, line: String, who: usize) {
        self.push_full(line, None, Some(who), None);
    }

    /// A line with both: villager `who` standing at `at` when it happened
    pub fn push_at_who(&mut self, line: String, at: Option<(f32, f32)>, who: Option<usize>) {
        self.push_full(line, at, who, None);
    }

    /// A tagged line: village news the scorer can read
    pub fn push_ev(&mut self, ev: EvTag, line: String) {
        self.push_full(line, None, None, Some(ev));
    }

    /// A tagged line about a place
    pub fn push_ev_at(&mut self, ev: EvTag, line: String, at: (f32, f32)) {
        self.push_full(line, Some(at), None, Some(ev));
    }

    /// A tagged line about a villager
    pub fn push_ev_who(&mut self, ev: EvTag, line: String, who: usize) {
        self.push_full(line, None, Some(who), Some(ev));
    }

    /// A tagged line about a villager at a place
    pub fn push_ev_at_who(
        &mut self,
        ev: EvTag,
        line: String,
        at: Option<(f32, f32)>,
        who: Option<usize>,
    ) {
        self.push_full(line, at, who, Some(ev));
    }

    fn push_full(
        &mut self,
        line: String,
        at: Option<(f32, f32)>,
        who: Option<usize>,
        ev: Option<EvTag>,
    ) {
        self.lines.push(line);
        self.at.push(at);
        self.who.push(who);
        self.ev.push(ev);
    }

    /// The click-through meta for line `i`, parallel to `lines`
    pub fn meta(&self, i: usize) -> LogMeta {
        LogMeta { at: self.at[i], who: self.who[i] }
    }

    /// The story tag for line `i`, parallel to `lines` - `None` for
    /// plain pushes (which should all be non-timestamped lines)
    pub fn ev(&self, i: usize) -> Option<EvTag> {
        self.ev[i]
    }

    /// Drop `n` lines off the front, meta in lockstep (the LOG_MAX roll)
    pub fn drop_front(&mut self, n: usize) {
        self.lines.drain(..n);
        self.at.drain(..n);
        self.who.drain(..n);
        self.ev.drain(..n);
    }

    /// Empty the log, meta in lockstep (tests that reset a game)
    pub fn clear(&mut self) {
        self.lines.clear();
        self.at.clear();
        self.who.clear();
        self.ev.clear();
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
