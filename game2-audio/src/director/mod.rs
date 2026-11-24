//! The director (#1503, M5): the brain that turns public game state -
//! the `UiModel` and nothing else - into a musical `Intent` each frame.
//! The composer writes phrases from the intent; the engine plays them.
//!
//! Timing runs on real seconds: one `update` is one game frame (60 fps
//! in lockstep), so sim speed never moves the music's tempo - turbo
//! only shrinks how many real seconds a planned event sits away, which
//! is what the anticipatory build wants. Section and mode changes land
//! only at phrase boundaries, and a section holds at least two phrases.

use crate::compose::{Meter, MotifId, Section, tempo_band};
use crate::theory::Mode;
use game2_core::session::{guard_pose, CampaignPhaseV, PKind, UiModel, Verdict};

/// Frames per second the director counts time in (the lockstep rate
/// `AudioHost::push` runs at).
pub const FPS: f64 = 60.0;
/// Bars per phrase, matching the composer's estampie.
pub const BARS_PER_PHRASE: u64 = 8;
/// A section holds this many phrases before a non-urgent change lands.
pub const MIN_SECTION_PHRASES: u32 = 2;
/// The build starts at most this many phrases before an event so the
/// arrival lands on a phrase downbeat.
pub const BUILD_PHRASES: u32 = 4;
/// The mode drifts one ladder step toward baseline every this many
/// phrases.
pub const DRIFT_PHRASES: u32 = 8;
/// The tonic never moves: D, midi 62.
pub const TONIC: u8 = 62;

/// The seven mood channels, 0..1, eased toward their targets each frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mood {
    pub tension: f32,
    pub valence: f32,
    pub energy: f32,
    pub sacred: f32,
    pub grief: f32,
    pub wonder: f32,
    pub festivity: f32,
}

impl Mood {
    /// Pull each channel a fixed fraction toward `t` - about 1.7 s to
    /// cover most of a gap at 60 fps, so a verdict reads as a swell,
    /// not a snap.
    fn ease(&mut self, t: &Mood) {
        const K: f32 = 0.03;
        self.tension += (t.tension - self.tension) * K;
        self.valence += (t.valence - self.valence) * K;
        self.energy += (t.energy - self.energy) * K;
        self.sacred += (t.sacred - self.sacred) * K;
        self.grief += (t.grief - self.grief) * K;
        self.wonder += (t.wonder - self.wonder) * K;
        self.festivity += (t.festivity - self.festivity) * K;
    }
}

/// A one-shot punctuation the verdict's mode change lands on, fired on
/// the next downbeat after a phrase boundary (`Stinger` in the design's
/// outcome table).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stinger {
    /// Win: bells peal; the cause motif stated in full, clos.
    BellsPeal,
    /// Partial: the cause motif's first three notes, clos.
    CauseClose,
    /// Stall: an unresolved half-cadence.
    HalfCadence,
    /// Loss: a low drum; the cause motif inverted.
    LowDrum,
    /// Crackdown: an alarm bell rung off schedule, then shawm and drums
    /// on the lord's motif.
    Alarm,
}

/// Bus ducking in dB, applied on top of the settings mix each frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ducks {
    pub music_db: f32,
    pub ambience_db: f32,
}

/// The musical request for this frame: what the composer writes the
/// next phrase from when the boundary comes.
#[derive(Clone, Debug)]
pub struct Intent {
    pub mood: Mood,
    pub section: Section,
    pub mode: Mode,
    pub tonic: u8,
    pub tempo_bpm: f64,
    /// Per-voice density 0..1, in `Voice` order.
    pub density: [f32; 8],
    pub motif: MotifId,
    /// Queued punctuation, fired once on the downbeat it lands on.
    pub stinger: Option<Stinger>,
    pub duck: Ducks,
    /// The phrase's time signature (festive triples, the march's 2/4).
    pub meter: Meter,
    /// false leaves endings ouvert - an unresolved arc or a build.
    pub resolved: bool,
    /// The crackdown layer: the drums and bass stay up under the mode.
    pub pressure_layer: bool,
    /// Steps the motif lifts through the build (the design's sequence).
    pub motif_step: i32,
    /// The build's last phrase before the event rests its final bar.
    pub rest_bar: bool,
    /// The phrase index this intent composes next (trace alignment).
    pub phrase_idx: u64,
}

impl Default for Intent {
    /// The quiet baseline: ambient D Dorian, drone only. Scene replays
    /// and tests that build frames without a model start here.
    fn default() -> Self {
        Intent {
            mood: Mood::default(),
            section: Section::Ambient,
            mode: Mode::Dorian,
            tonic: TONIC,
            tempo_bpm: 66.0,
            density: density_for(Section::Ambient, 0, true),
            motif: MotifId::Village,
            stinger: None,
            duck: Ducks::default(),
            meter: Meter::FOUR_FOUR,
            resolved: true,
            pressure_layer: false,
            motif_step: 0,
            rest_bar: false,
            phrase_idx: 0,
        }
    }
}

/// The mode ladder's rungs, brightest first - a verdict's `steps`
/// moves along this order (Win +2 goes from Dorian to Ionian).
const LADDER: [Mode; 6] = [
    Mode::Lydian, Mode::Ionian, Mode::Mixolydian, Mode::Dorian,
    Mode::Aeolian, Mode::Phrygian,
];
const BASELINE_RUNG: i32 = 3; // Dorian: the ordinary village's rest

fn rung(m: Mode) -> i32 {
    LADDER.iter().position(|&x| x == m).unwrap_or(BASELINE_RUNG as usize) as i32
}

/// A verdict's mode steps, per the design's outcome table.
fn verdict_step(v: Verdict) -> i32 {
    match v {
        Verdict::Win => 2,
        Verdict::Partial => 1,
        Verdict::Stall => 0,
        Verdict::Loss => -1,
        Verdict::Crackdown => -2,
    }
}

fn verdict_stinger(v: Verdict) -> Stinger {
    match v {
        Verdict::Win => Stinger::BellsPeal,
        Verdict::Partial => Stinger::CauseClose,
        Verdict::Stall => Stinger::HalfCadence,
        Verdict::Loss => Stinger::LowDrum,
        Verdict::Crackdown => Stinger::Alarm,
    }
}

/// Per-section voice templates, indexed by `Voice` order (Drone, Pulse,
/// Ground, Harmony, Melody, Counter, Color, Choir).
fn density_for(s: Section, layer: u32, phrase_parity: bool) -> [f32; 8] {
    match s {
        Section::Silence => [0.0; 8],
        // Drone and sparse harp; the melody enters every other phrase.
        Section::Ambient => {
            [0.55, 0.0, 0.2, 0.3, if phrase_parity { 0.3 } else { 0.0 }, 0.0, 0.1, 0.0]
        }
        Section::Theme => [0.6, 0.4, 0.5, 0.5, 0.65, 0.3, 0.2, 0.0],
        // One layer per build phrase: harmony and melody first, then
        // pulse, ground, counter and color join as it climbs.
        Section::Build => [
            0.7,
            if layer >= 1 { 0.5 } else { 0.0 },
            if layer >= 2 { 0.5 } else { 0.0 },
            0.5,
            0.5 + 0.1 * layer as f32,
            if layer >= 3 { 0.4 } else { 0.0 },
            if layer >= 4 { 0.4 } else { 0.1 },
            0.0,
        ],
        Section::Peak => [0.8, 0.9, 0.8, 0.7, 0.9, 0.6, 0.5, 0.0],
        // Right after the verdict: stripped back like a breakdown.
        Section::Release => [0.5, 0.1, 0.2, 0.3, 0.4, 0.1, 0.2, 0.0],
        Section::Festive => [0.6, 0.7, 0.6, 0.6, 0.85, 0.4, 0.5, 0.0],
        // Organum and a chant pad over the drone.
        Section::Sacred => [0.6, 0.0, 0.2, 0.7, 0.3, 0.4, 0.3, 0.7],
        // Slow solo vielle and harp under a thin drone.
        Section::Lament => [0.3, 0.0, 0.0, 0.35, 0.6, 0.0, 0.1, 0.0],
    }
}

/// The anticipatory build's state: how many layers it has laid down
/// toward `next`, and whether the last phrase leaves its final bar
/// empty before the arrival.
#[derive(Clone, Copy, Debug, Default)]
struct Build {
    layer: u32,
    /// The event left the window or slipped: hold here, never drop.
    holding: bool,
}

/// The score's brain. `new` takes the world's seed so the trace lines
/// up with the sim's determinism; `update` runs once a frame on the
/// game thread, reads only the public `UiModel`, and hands back the
/// `Intent` the next phrase is written from.
pub struct Director {
    #[allow(dead_code)]
    seed: u64,
    mood: Mood,
    /// Real seconds since the score started.
    t_s: f64,
    /// Seconds into the current phrase.
    phrase_pos: f64,
    /// Seconds into the current bar (stingers and urgent sections land
    /// on downbeats, not just phrase boundaries).
    bar_pos: f64,
    phrase_idx: u64,
    section: Section,
    section_held: u32,
    /// The candidate last boundary's section-switch votes: a non-urgent
    /// change must hold for a full phrase before it lands (the
    /// flicker guard on top of `MIN_SECTION_PHRASES`).
    stable: Option<Section>,
    /// The mode ladder rung the verdicts pushed us to.
    rung: i32,
    drift_count: u32,
    /// A stall's ouvert latch: endings stay open till the next verdict.
    unresolved: bool,
    /// The last outcome we've staged (kind+day): a new one queues the
    /// release, the mode step and its stinger.
    last_outcome: Option<(String, u32)>,
    pending_release: bool,
    pending_step: i32,
    pending_crackdown: bool,
    /// Fires on the next phrase boundary (a downbeat).
    pending_stinger: Option<Stinger>,
    build: Build,
    /// Grief: folk the seat knew who now draw as graves, and how long
    /// the lament still has to run.
    grief_until: f64,
    known_dead: usize,
    /// Presence: the share of time music is allowed, counted in
    /// phrases - a sounding phrase spends `1 - share` of debt, a silent
    /// one earns `share`. Starting in credit gives a new village a run
    /// of music before the budget ever bites.
    presence: f32,
    debt: f32,
    /// Phrases of budget silence still owed: music sections end on a
    /// clos cadence and leave 1-3 phrases of ambience.
    silence_left: u32,
}

impl Director {
    pub fn new(seed: u64) -> Self {
        Director {
            seed,
            mood: Mood::default(),
            t_s: 0.0,
            phrase_pos: 0.0,
            bar_pos: 0.0,
            phrase_idx: 0,
            section: Section::Ambient,
            section_held: 0,
            stable: None,
            rung: BASELINE_RUNG,
            drift_count: 0,
            unresolved: false,
            last_outcome: None,
            pending_release: false,
            pending_step: 0,
            pending_crackdown: false,
            pending_stinger: None,
            build: Build::default(),
            grief_until: 0.0,
            known_dead: 0,
            presence: 0.6, // the settings' "Normal"
            debt: -4.0, // four free phrases before the budget bites
            silence_left: 0,
        }
    }

    /// The presence budget (0 / 0.35 / 0.6 / 0.85): the settings row's
    /// share of time with music.
    pub fn set_presence(&mut self, share: f32) {
        self.presence = share.clamp(0.0, 1.0);
    }

    /// True on the update that crossed a phrase boundary - the moment a
    /// new `Intent` becomes the phrase being written.
    pub fn phrase_boundary(&self) -> bool {
        self.phrase_pos == 0.0 && self.t_s > 0.0
    }

    /// The running phrase count (each boundary increments it).
    pub fn phrase_index(&self) -> u64 {
        self.phrase_idx
    }

    /// Seconds a phrase of `meter` at `tempo` spans: 8 bars of `num`
    /// beats where a beat is a `den`-note (`4/den` quarter notes).
    fn phrase_seconds(meter: Meter, tempo: f64) -> f64 {
        BARS_PER_PHRASE as f64 * meter.num as f64 * (4.0 / meter.den as f64)
            * 60.0 / tempo.max(30.0)
    }

    /// Seconds until `drama.next`'s event, per the design:
    /// `in_days * frames_per_day / 60`. A zero `frames_per_day` (older
    /// wire) or a missing event reads as "no build".
    fn seconds_to_event(m: &UiModel) -> Option<f64> {
        let p = m.panel.drama.next.as_ref()?;
        if m.clock.frames_per_day == 0 {
            return None;
        }
        Some(p.in_days.max(0.0) as f64 * m.clock.frames_per_day as f64 / FPS)
    }

    /// The verdict table's tension contribution, by campaign phase.
    fn phase_tension(m: &UiModel) -> f32 {
        match m.panel.drama.phase {
            CampaignPhaseV::Listening => 0.1,
            CampaignPhaseV::IssueCut => 0.25,
            CampaignPhaseV::Research => 0.4,
            CampaignPhaseV::Action | CampaignPhaseV::Evaluate => 0.6,
        }
    }

    /// The mood targets the model asks for this frame.
    fn mood_target(&self, m: &UiModel, grieving: bool) -> Mood {
        let d = &m.panel.drama;
        let mut t = Mood::default();
        // Tension: phase, pressure flags, ready guards, fires, and the
        // climb as the next event nears; a live march pins it high.
        t.tension = Self::phase_tension(m)
            + 0.15 * (d.pressure.guards_posted as u32
                + d.pressure.assembly_ban as u32
                + (d.pressure.members_held > 0) as u32
                + d.pressure.leader_taken as u32) as f32;
        if m.people.iter()
            .any(|p| matches!(p.kind, PKind::Guard(_)) && p.pose == guard_pose::READY)
        {
            t.tension += 0.15;
        }
        if !m.burning.is_empty() {
            t.tension += 0.15;
        }
        if let Some(secs) = Self::seconds_to_event(m) {
            let window = BUILD_PHRASES as f64 * 20.0;
            t.tension += 0.2 * (1.0 - (secs / window).min(1.0)) as f32;
        }
        if d.live.as_ref().is_some_and(|l| l.kind == "march") {
            t.tension = 0.9;
        }
        // Valence: the fortune baseline (food, the sick and hungry)
        // plus the last verdict's echo.
        let folk = m.people.len().max(1) as f32;
        let well = m.people.iter().filter(|p| !p.hungry && !p.sick && !p.dead).count() as f32;
        t.valence = (0.3 + 0.5 * well / folk).clamp(0.0, 1.0);
        if let Some(o) = &d.last {
            t.valence += match o.verdict {
                Verdict::Win => 0.25,
                Verdict::Partial => 0.12,
                Verdict::Stall => 0.0,
                Verdict::Loss => -0.2,
                Verdict::Crackdown => -0.3,
            };
        }
        t.energy = (t.tension * 0.6 + if m.feast { 0.4 } else { 0.15 }).clamp(0.0, 1.0);
        t.sacred = if m.calendar.holy_day { 0.8 } else { 0.05 };
        t.grief = if grieving { 0.85 } else { 0.0 };
        t.wonder = if m.sky.is_some() || m.rainbow_from.is_some() { 0.8 } else { 0.0 };
        t.festivity = if m.feast || m.calendar.feast.is_some() { 0.85 } else { 0.0 };
        Mood {
            tension: t.tension.clamp(0.0, 1.0),
            valence: t.valence.clamp(0.0, 1.0),
            energy: t.energy.clamp(0.0, 1.0),
            sacred: t.sacred.clamp(0.0, 1.0),
            grief: t.grief.clamp(0.0, 1.0),
            wonder: t.wonder.clamp(0.0, 1.0),
            festivity: t.festivity.clamp(0.0, 1.0),
        }
    }

    /// Someone the seat has warmth for lies under a grave marker now.
    /// The lament runs for a while, then lets the village move on.
    fn grieving(&mut self, m: &UiModel) -> bool {
        let dead_known = m.people.iter()
            .filter(|p| p.dead && m.bonds.iter().any(|(e, h)| *e == p.e && *h > 0.0))
            .count();
        if dead_known > self.known_dead {
            self.grief_until = self.t_s + 120.0;
        }
        self.known_dead = self.known_dead.max(dead_known);
        dead_known > 0 && self.t_s < self.grief_until
    }

    /// Whether a live event reads as the score's Peak.
    fn live_peak(m: &UiModel) -> bool {
        m.panel.drama.live.as_ref().is_some_and(|l| {
            matches!(l.kind.as_str(),
                "march" | "assembly" | "townhall" | "campaign meeting"
                | "house meeting" | "strike vote" | "feast" | "funeral" | "parade")
        })
    }

    /// The section the model wants this frame, before the hold rules.
    fn candidate(&self, m: &UiModel, grieving: bool, build_on: bool) -> Section {
        let convo = m.convo.is_some() || m.talk.in_talk;
        if self.presence <= 0.0
            || (convo && self.presence < 0.6)
            || self.debt > 0.0
            || self.silence_left > 0
        {
            return Section::Silence;
        }
        if grieving {
            return Section::Lament;
        }
        if Self::live_peak(m) {
            return Section::Peak;
        }
        if self.pending_release {
            return Section::Release;
        }
        if build_on {
            return Section::Build;
        }
        if m.feast || m.calendar.feast.is_some() {
            return Section::Festive;
        }
        if m.calendar.holy_day {
            return Section::Sacred;
        }
        let has_story = m.panel.campaign.is_some() || !m.panel.members.is_empty();
        if !has_story && self.mood.tension < 0.3 {
            Section::Ambient
        } else {
            Section::Theme
        }
    }

    /// Urgent sections jump the min-hold queue at the phrase boundary:
    /// a live event, a verdict's release, a lament, or running out of
    /// budget.
    fn urgent(s: Section) -> bool {
        matches!(s, Section::Peak | Section::Release | Section::Lament | Section::Silence)
    }

    /// The subset that can't wait for a phrase: they land on the next
    /// downbeat (a march arriving mid-phrase still answers inside a
    /// bar). Release holds for its mode step's phrase boundary, and
    /// Silence always cadences out on one - it never cuts a bar short.
    fn bar_urgent(s: Section) -> bool {
        matches!(s, Section::Peak | Section::Lament)
    }

    /// The meter a section wants: festive triples, the march's 2/4,
    /// everything else 4/4.
    fn meter_for(&self, s: Section, m: &UiModel) -> Meter {
        match s {
            Section::Festive => {
                if self.phrase_idx % 2 == 0 { Meter::THREE_FOUR } else { Meter::SIX_EIGHT }
            }
            Section::Peak if m.panel.drama.live.as_ref()
                .is_some_and(|l| l.kind == "march") => Meter::TWO_FOUR,
            _ => Meter::FOUR_FOUR,
        }
    }

    /// The base tempo a section asks for, before the build's +2/phrase.
    fn tempo_for(&self, s: Section, m: &UiModel) -> f64 {
        let band = tempo_band(s);
        let base = match s {
            // Silence still ticks an ambient-paced phrase clock so the
            // ambience it leaves runs in phrase-length breaths.
            Section::Silence => 66.0,
            Section::Ambient => 66.0,
            Section::Theme => 90.0,
            Section::Build => 96.0 + 2.0 * self.build.layer as f64,
            Section::Peak => {
                if m.panel.drama.live.as_ref().is_some_and(|l| l.kind == "march") {
                    96.0 + 12.0 * m.panel.drama.live.as_ref().map(|l| l.progress).unwrap_or(0.0) as f64
                } else {
                    104.0
                }
            }
            Section::Release => 84.0,
            Section::Festive => 116.0,
            Section::Sacred => 72.0,
            Section::Lament => 66.0,
        };
        let mut bpm = band.map_or(base, |(lo, hi)| base.clamp(lo, hi));
        if s == Section::Lament {
            bpm *= 0.85; // grief slows 15%
        }
        bpm
    }

    /// Which leitmotif the section and drama call for (design's table).
    fn motif_for(&self, s: Section, m: &UiModel) -> MotifId {
        if self.pending_crackdown || m.panel.drama.pressure.leader_taken {
            return MotifId::Lord;
        }
        match s {
            Section::Lament => MotifId::Grief,
            Section::Sacred => MotifId::Church,
            Section::Festive => MotifId::Feast,
            Section::Peak | Section::Build | Section::Release => {
                if m.panel.campaign.is_some() { MotifId::Cause } else { MotifId::Village }
            }
            _ => {
                if m.panel.campaign.is_some() { MotifId::Cause } else { MotifId::Village }
            }
        }
    }

    /// One frame of direction: ease the mood, count the seconds, apply
    /// phrase-boundary changes, and publish the intent.
    pub fn update(&mut self, m: &UiModel) -> Intent {
        self.t_s += 1.0 / FPS;
        let grieving = self.grieving(m);
        let target = self.mood_target(m, grieving);
        self.mood.ease(&target);

        // A verdict we haven't staged yet queues its release, mode
        // step and stinger for the next phrase boundary.
        if let Some(o) = &m.panel.drama.last {
            let key = (o.kind.clone(), o.day);
            if self.last_outcome.as_ref() != Some(&key) {
                self.last_outcome = Some(key);
                self.pending_release = true;
                self.pending_step += verdict_step(o.verdict);
                if o.verdict == Verdict::Crackdown {
                    self.pending_crackdown = true;
                }
                self.unresolved = o.verdict == Verdict::Stall;
                if o.verdict != Verdict::Stall {
                    self.unresolved = false;
                }
                self.pending_stinger = Some(verdict_stinger(o.verdict));
            }
        }

        // The build: inside the 4-phrase window - measured in build
        // phrases (~20 s at 96 bpm), the clock the arrival lands on -
        // one layer per phrase; outside it, hold whatever's been laid
        // (a postponed event never tears the build down abruptly).
        let secs_to = Self::seconds_to_event(m);
        let tempo_now = self.tempo_for(self.section, m);
        let meter_now = self.meter_for(self.section, m);
        let phrase_s = Self::phrase_seconds(meter_now, tempo_now);
        let build_phrase_s = Self::phrase_seconds(
            Meter::FOUR_FOUR, 96.0 + 2.0 * self.build.layer as f64);
        let want_build = secs_to
            .is_some_and(|s| s <= BUILD_PHRASES as f64 * build_phrase_s);
        if want_build {
            self.build.holding = false;
        } else if self.build.layer > 0 || self.section == Section::Build {
            self.build.holding = true;
        } else {
            self.build = Build::default();
        }

        let cand = self.candidate(m, grieving, want_build || self.build.holding);

        // Downbeats first: a stinger or an urgent section (a live
        // event, a lament, running out of presence) lands at the next
        // bar boundary, inside a bar of the thing that caused it.
        self.phrase_pos += 1.0 / FPS;
        self.bar_pos += 1.0 / FPS;
        let bar_s = phrase_s / BARS_PER_PHRASE as f64;
        let bar_hit = self.bar_pos >= bar_s - 1e-9;
        if bar_hit {
            self.bar_pos = 0.0;
        }
        let mut stinger = None;
        if bar_hit {
            stinger = self.pending_stinger.take();
            if cand != self.section && Self::bar_urgent(cand) {
                self.section = cand;
                self.section_held = 0;
                self.stable = None;
            }
        }

        // Phrase-boundary work: ordinary section switches (a stable
        // candidate after a held minimum), mode steps, the drift back
        // to Dorian, and the build's per-phrase layer.
        let phrase_hit = self.phrase_pos >= phrase_s - 1e-9;
        if phrase_hit {
            self.phrase_pos = 0.0;
            self.phrase_idx += 1;
            self.section_held += 1;
            self.drift_count += 1;

            // presence debt, in phrase units: the phrase that just
            // ended spent `1 - share` if it sounded, earned `share` if
            // it was silence - the running share converges to the
            // budget by accounting, not by policing every frame
            if self.section == Section::Silence {
                self.debt -= self.presence;
                self.silence_left = self.silence_left.saturating_sub(1);
            } else {
                self.debt += 1.0 - self.presence;
            }
            self.debt = self.debt.clamp(-8.0, 8.0);

            // leaving Silence needs no hold: the cadence is the budget's
            // 1-3 phrases, spent and done
            let leave_silence = self.section == Section::Silence
                && self.silence_left == 0 && self.debt <= 0.0;
            if cand != self.section
                && (leave_silence
                    || self.section_held >= MIN_SECTION_PHRASES
                        && self.stable == Some(cand)
                    || Self::urgent(cand)
                    || cand == Section::Build && want_build)
            {
                let was = self.section;
                self.section = cand;
                self.section_held = 0;
                self.stable = None;
                // a budget silence runs 1-3 phrases of ambience, then
                // the score cadences back in
                if cand == Section::Silence && was != Section::Silence && self.debt > 0.0 {
                    self.silence_left = 1 + (self.phrase_idx % 3) as u32;
                }
            } else if cand != self.section {
                self.stable = Some(cand);
            } else {
                self.stable = None;
            }

            if self.pending_step != 0 {
                self.rung = (self.rung - self.pending_step).clamp(0, 5);
                self.pending_step = 0;
            }
            if self.drift_count >= DRIFT_PHRASES {
                self.drift_count = 0;
                if self.rung < BASELINE_RUNG {
                    self.rung += 1;
                } else if self.rung > BASELINE_RUNG {
                    self.rung -= 1;
                }
            }
            if self.section == Section::Build && want_build && !self.build.holding {
                self.build.layer = (self.build.layer + 1).min(BUILD_PHRASES);
            }
            if self.pending_release && self.section == Section::Release {
                self.pending_release = false;
            } else if self.pending_release && self.section_held >= 1 {
                // the release gave way to something urgent - clear it
                self.pending_release = false;
            }
        }

        // The mode the phrase plays in: ladder rung, +1 brightness while
        // a sky wonder lasts, Aeolian under grief.
        let mut eff = self.rung;
        if m.sky.is_some() {
            eff -= 1;
        }
        if grieving {
            eff = rung(Mode::Aeolian);
        }
        let mode = LADDER[eff.clamp(0, 5) as usize];

        let meter = self.meter_for(self.section, m);
        let mut tempo = self.tempo_for(self.section, m);
        if self.section == Section::Build {
            tempo = (96.0 + 2.0 * self.build.layer as f64).min(108.0);
        }
        let mut density = density_for(self.section, self.build.layer, self.phrase_idx % 2 == 0);
        if self.pending_crackdown {
            density[1] = (density[1] + 0.3).min(1.0); // Pulse
            density[2] = (density[2] + 0.3).min(1.0); // Ground
        }
        // Conversation drops the melody but keeps drone and harp
        // (Harmony); paused just holds quieter.
        let convo = m.convo.is_some() || m.talk.in_talk;
        if convo {
            density[4] = 0.0;
        }
        let mut duck = Ducks::default();
        if convo {
            duck.music_db = -9.0;
        }
        if m.paused {
            duck.music_db = (duck.music_db - 6.0).min(-6.0);
        }
        if self.section == Section::Peak {
            duck.ambience_db = -3.0;
        }

        // The build's last phrase before the event rests its final bar:
        // inside one phrase of the arrival, flag it for the composer.
        let rest_bar = self.section == Section::Build
            && secs_to.is_some_and(|s| s <= phrase_s);

        Intent {
            mood: self.mood,
            section: self.section,
            mode,
            tonic: TONIC,
            tempo_bpm: tempo,
            density,
            motif: self.motif_for(self.section, m),
            stinger,
            duck,
            meter,
            resolved: !self.unresolved && self.section != Section::Build,
            pressure_layer: self.pending_crackdown,
            motif_step: self.build.layer as i32,
            rest_bar,
            phrase_idx: self.phrase_idx,
        }
    }
}
