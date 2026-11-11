//! Balance Tester: scores long headless runs against the four research
//! rubrics - fun/pacing, balance methodology, narrative arcs, reward
//! schedules (see wiki/literature-reviews/). The game is meant hard, even
//! unfair; the tester's job is to tell "hard but rewarding" from "hard
//! and hollow". Each KPI is a number pulled off the day's gauges and the
//! village's own log, checked against a band; the bands come out of the
//! lit reviews, not thin air.

use std::collections::HashMap;

pub use crate::session::{EvTag, Sev, Thread, Valence};

// ---------- recorded runs ----------

/// One log line as the scorer reads it: its story tag when the village
/// wrote one (plain pushes carry none - disclaimers, harness notes) and
/// the text with the "t{n}:" prefix stripped.
#[derive(Debug, Clone)]
pub struct EvLine {
    pub tag: Option<EvTag>,
    pub text: String,
}

/// One day of a run: the gauges at day's end and what the log said.
#[derive(Debug, Clone)]
pub struct DaySample {
    pub tick: u64,
    pub gauges: Vec<(String, f64)>,
    /// Log lines the village wrote this day, with their story tags.
    pub events: Vec<EvLine>,
}

/// A whole run, day by day.
#[derive(Debug)]
pub struct RunRecord {
    pub seed: u64,
    pub days: u64,
    /// Which driver policy steered the seat: "idle", "organizer",
    /// "agitator" - org-growth KPIs only measure the active personas.
    pub persona: String,
    pub samples: Vec<DaySample>,
}

impl RunRecord {
    /// One gauge's whole series, as (tick, value).
    pub fn series(&self, name: &str) -> Vec<(u64, f64)> {
        self.samples.iter().filter_map(|s| {
            s.gauges.iter().find(|(k, _)| k == name).map(|(_, v)| (s.tick, *v))
        }).collect()
    }

    /// Every event line in order, with the day it was logged.
    pub fn events(&self) -> Vec<(u64, &EvLine)> {
        self.samples.iter()
            .flat_map(|s| s.events.iter().map(move |e| (s.tick, e)))
            .collect()
    }
}

// ---------- event reading ----------

/// The tag an untagged line reads as: neutral context in no story.
fn val(e: &EvLine) -> Valence {
    e.tag.map(|t| t.valence).unwrap_or(Valence::Neutral)
}
fn thr(e: &EvLine) -> Thread {
    e.tag.map(|t| t.thread).unwrap_or(Thread::Other)
}

/// How heavily a line lands for severity-weighted scores: a rainbow is
/// not a writ. Untagged lines weigh nothing - they're not village news.
fn sev_w(e: &EvLine) -> f64 {
    match e.tag.map(|t| t.sev) {
        Some(Sev::Minor) => 1.0,
        Some(Sev::Notable) => 2.0,
        Some(Sev::Major) => 4.0,
        None => 0.0,
    }
}

/// A log line worth counting as a beat - charged news in a real thread:
/// weather patter, command feedback and forewarnings are context, not
/// beats; untagged lines (disclaimers, harness notes) never are.
pub fn is_beat(e: &EvLine) -> bool {
    e.tag.map(|t| t.is_beat()).unwrap_or(false)
}

/// A substantial beat - the kind a pacing metric counts: a cat choosing
/// a hearth is texture, a chimney fire is an event. Minor beats still
/// read as "the world moved" for flatline/dead-air, but density and
/// rhythm measure the news that lands.
fn is_notable_beat(e: &EvLine) -> bool {
    is_beat(e) && e.tag.map(|t| t.sev != Sev::Minor).unwrap_or(false)
}

// ---------- small stats ----------

fn mean(xs: &[f64]) -> f64 { if xs.is_empty() { 0.0 } else { xs.iter().sum::<f64>() / xs.len() as f64 } }

fn stdev(xs: &[f64]) -> f64 {
    if xs.len() < 2 { return 0.0; }
    let m = mean(xs);
    (xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / xs.len() as f64).sqrt()
}

/// Coefficient of variation: how spiky a series is beside its own mean.
fn cv(xs: &[f64]) -> f64 { let m = mean(xs); if m.abs() < 1e-9 { 0.0 } else { stdev(xs) / m.abs() } }

/// Days between successive true positions in `days`.
fn intervals(days: &[u64]) -> Vec<f64> {
    days.windows(2).map(|w| (w[1] - w[0]) as f64).collect()
}

/// Longest run of consecutive days satisfying `f`.
fn longest_streak(days: u64, f: impl Fn(u64) -> bool) -> u64 {
    let (mut best, mut cur) = (0, 0);
    for t in 0..days {
        if f(t) { cur += 1; best = best.max(cur); } else { cur = 0; }
    }
    best
}

// ---------- the suite ----------

/// Which rubric a KPI answers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain { Fun, Balance, Narrative, Reward }

impl Domain {
    pub fn label(&self) -> &'static str {
        match self {
            Domain::Fun => "fun",
            Domain::Balance => "balance",
            Domain::Narrative => "narrative",
            Domain::Reward => "reward",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status { Pass, Warn, Fail }

/// One key performance indicator: a number pulled off the run, the band
/// it should land in, and the edge past which it's a red flag outright.
pub struct Kpi {
    pub name: &'static str,
    pub domain: Domain,
    /// Only meaningful under a persona that plays the org game: skipped
    /// on the idle run, where it would read as a permanent failure.
    pub active_only: bool,
    /// What "good" looks like, inclusive.
    pub target: (f64, f64),
    /// Below this the KPI doesn't just miss - it flags the whole run.
    pub flag_below: Option<f64>,
    /// Above this the KPI doesn't just miss - it flags the whole run.
    pub flag_above: Option<f64>,
    pub unit: &'static str,
    pub why: &'static str,
    pub measure: fn(&RunRecord) -> f64,
}

#[derive(Debug)]
pub struct KpiResult {
    pub name: &'static str,
    pub domain: Domain,
    pub value: f64,
    pub target: (f64, f64),
    pub status: Status,
    pub unit: &'static str,
    pub why: &'static str,
}

/// An OKR key result: the KPI it watches and the share of the run it
/// demands be in band (for per-window KRs) - kept simple: the KR is met
/// when the KPI passes.
pub struct Okr {
    pub domain: Domain,
    pub objective: &'static str,
    pub key_results: &'static [&'static str],
}

/// The whole run, marked.
#[derive(Debug)]
pub struct BalanceReport {
    pub seed: u64,
    pub days: u64,
    pub persona: String,
    pub results: Vec<KpiResult>,
    pub domain_scores: Vec<(Domain, f64)>,
    pub okr_hits: Vec<(&'static str, usize, usize)>, // (objective, met, of)
    pub flags: Vec<String>,
    pub overall: f64,
    pub verdict: String,
}

// ---------- the measures ----------
// Each pulls one number off a run. All f64, all defined so an empty run
// reads as the worst sane value rather than panicking.

/// Days till the first good thing happens to/for the village.
fn m_first_reward(r: &RunRecord) -> f64 {
    r.samples
        .iter()
        .find(|s| s.events.iter().any(|e| val(e) == Valence::Positive))
        .map(|s| s.tick)
        .unwrap_or(r.days) as f64
}

/// Notable+ beats per 10 days - substantial events, not texture.
fn m_beat_density(r: &RunRecord) -> f64 {
    let beats = r
        .events()
        .into_iter()
        .filter(|(_, e)| is_notable_beat(e))
        .count();
    if r.days == 0 { 0.0 } else { beats as f64 * 10.0 / r.days as f64 }
}

/// Longest stretch where nothing changed: no beat and every gauge flat.
fn m_flatline(r: &RunRecord) -> f64 {
    longest_streak(r.days, |t| {
        let s = match r.samples.iter().find(|s| s.tick == t) { Some(s) => s, None => return true };
        s.events.iter().all(|e| !is_beat(e))
            && r.samples.iter().find(|p| p.tick == t + 1)
                .map(|n| n.gauges == s.gauges).unwrap_or(false)
    }) as f64
}

/// Days till the org wins its first member (the first real growth signal).
fn m_first_growth(r: &RunRecord) -> f64 {
    r.series("org.members").iter().find(|(_, v)| *v >= 1.0)
        .map(|(t, _)| *t).unwrap_or(r.days) as f64
}

/// Mean share of the village below the sim's own seek-food line -
/// sustained pressure, not a single winter wave's peak.
fn m_hunger_pressure(r: &RunRecord) -> f64 {
    let alive = r.series("villagers.alive");
    let hungry: HashMap<u64, f64> = r.series("villagers.hungry").into_iter().collect();
    mean(&alive.iter().map(|(t, a)| if *a <= 0.0 { 0.0 } else { hungry.get(t).copied().unwrap_or(0.0) / a })
        .collect::<Vec<_>>())
}

/// Worst starving share on any day (Hunger <= 10) - the hard red line.
fn m_starving_peak(r: &RunRecord) -> f64 {
    let alive = r.series("villagers.alive");
    let st: HashMap<u64, f64> = r.series("villagers.starving").into_iter().collect();
    alive.iter().map(|(t, a)| if *a <= 0.0 { 0.0 } else { st.get(t).copied().unwrap_or(0.0) / a })
        .fold(0.0f64, f64::max)
}

/// Peak wealth concentration: richest purse over the village mean.
fn m_gold_spread(r: &RunRecord) -> f64 {
    r.series("econ.gold_spread").iter().map(|(_, v)| *v).fold(0.0f64, f64::max)
}

/// Share of days the village table (the market store and the crop
/// fields, the pools a hungry villager walks to first) stood empty.
fn m_stockout_share(r: &RunRecord) -> f64 {
    let food: HashMap<u64, f64> = r.series("stock.food").into_iter().collect();
    let n = r.samples.iter().filter(|s| food.get(&s.tick).copied().unwrap_or(0.0) <= 0.0).count();
    if r.days == 0 { 0.0 } else { n as f64 / r.days as f64 }
}

/// Median days of cover on the village table (store + crop fields):
/// servings on hand per villager, read as "how long could the village
/// eat if all supply stopped".
fn m_food_cover(r: &RunRecord) -> f64 {
    let alive: HashMap<u64, f64> = r.series("villagers.alive").into_iter().collect();
    let mut covers: Vec<f64> = r.series("stock.food").iter()
        .map(|(t, s)| s / alive.get(t).copied().unwrap_or(1.0).max(1.0)).collect();
    covers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    covers.get(covers.len() / 2).copied().unwrap_or(0.0)
}

/// Villagers lost per 100 days of run.
fn m_mortality(r: &RunRecord) -> f64 {
    let a = r.series("villagers.alive");
    let (Some(first), Some(last)) = (a.first(), a.last()) else { return 0.0 };
    if r.days == 0 { 0.0 } else { (first.1 - last.1).max(0.0) * 100.0 / r.days as f64 }
}

/// Share of days at least one gauge moved - a living world fidgets.
fn m_liveliness(r: &RunRecord) -> f64 {
    if r.samples.len() < 2 { return 0.0; }
    let moved = r.samples.windows(2).filter(|w| w[0].gauges != w[1].gauges).count();
    moved as f64 / (r.samples.len() - 1) as f64
}

/// Beats that belong to a story thread, days between them: CV reads the
/// rhythm - near 0 is a metronome, near 1 is weather of the soul.
/// Notable+ only: the arc rhythm lives on events that land, not patter.
fn m_beat_interval_cv(r: &RunRecord) -> f64 {
    let days: Vec<u64> = r
        .events()
        .into_iter()
        .filter(|(_, e)| is_notable_beat(e))
        .map(|(t, _)| t)
        .collect();
    if days.len() < 3 {
        return 0.0;
    }
    cv(&intervals(&days))
}

/// How many story threads stayed live (3+ beats each).
fn m_thread_count(r: &RunRecord) -> f64 {
    let mut by_thread: HashMap<Thread, usize> = HashMap::new();
    for (_, e) in r.events() {
        if is_beat(e) {
            *by_thread.entry(thr(e)).or_default() += 1;
        }
    }
    by_thread.values().filter(|&&n| n >= 3).count() as f64
}

/// Trouble that found its answer: share of negative beats followed by a
/// positive beat inside 30 days.
fn m_resolution(r: &RunRecord) -> f64 {
    let ev = r.events();
    let pos_days: Vec<u64> = ev
        .iter()
        .filter(|(_, e)| val(e) == Valence::Positive)
        .map(|(t, _)| *t)
        .collect();
    let negs: Vec<u64> = ev
        .iter()
        .filter(|(_, e)| val(e) == Valence::Negative)
        .map(|(t, _)| *t)
        .collect();
    if negs.is_empty() {
        return 1.0;
    }
    let resolved = negs
        .iter()
        .filter(|t| pos_days.iter().any(|p| *p > **t && *p - **t <= 30))
        .count();
    resolved as f64 / negs.len() as f64
}

/// Longest dead air: days between beats, or at either end of the run.
fn m_dead_air(r: &RunRecord) -> f64 {
    let mut days: Vec<u64> = r.events().into_iter().filter(|(_, e)| is_beat(e)).map(|(t, _)| t).collect();
    days.insert(0, 0); days.push(r.days);
    intervals(&days).into_iter().fold(0.0, f64::max)
}

/// Rewards arrive on a variable schedule: CV of days between positive
/// beats. Near 0 reads as a drip-feed; the sweet spot is irregular.
fn m_reward_interval_cv(r: &RunRecord) -> f64 {
    let days: Vec<u64> = r
        .events()
        .into_iter()
        .filter(|(_, e)| val(e) == Valence::Positive)
        .map(|(t, _)| t)
        .collect();
    if days.len() < 3 {
        return 0.0;
    }
    cv(&intervals(&days))
}

/// Longest run with no positive beat - the extinction window a real
/// player walks away inside.
fn m_reward_drought(r: &RunRecord) -> f64 {
    let mut days: Vec<u64> = r
        .events()
        .into_iter()
        .filter(|(_, e)| val(e) == Valence::Positive)
        .map(|(t, _)| t)
        .collect();
    days.insert(0, 0);
    days.push(r.days);
    intervals(&days).into_iter().fold(0.0, f64::max)
}

/// Good days per bad one, severity-weighted: a wedding and a hanging
/// don't count the same as a kind word and a cold shoulder. A hard game
/// still pays out more than it takes.
fn m_valence_ratio(r: &RunRecord) -> f64 {
    let ev = r.events();
    let pos: f64 = ev
        .iter()
        .filter(|(_, e)| val(e) == Valence::Positive)
        .map(|(_, e)| sev_w(e))
        .sum();
    let neg: f64 = ev
        .iter()
        .filter(|(_, e)| val(e) == Valence::Negative)
        .map(|(_, e)| sev_w(e))
        .sum();
    if neg < 1.0 {
        return if pos > 0.0 { 10.0 } else { 0.0 };
    }
    pos / neg
}

/// A fair punishment is one you could see coming: share of Notable+
/// negative beats whose own thread was already active (non-negative
/// event) inside the 45 days before. Natural-consequences logic - writ
/// before knights. Minor blows need no herald; they're weather.
fn m_punish_warn(r: &RunRecord) -> f64 {
    let ev = r.events();
    let negs: Vec<(u64, Thread)> = ev
        .iter()
        .filter_map(|(t, e)| {
            (val(e) == Valence::Negative && e.tag.map(|t| t.sev != Sev::Minor).unwrap_or(false))
                .then_some((*t, thr(e)))
        })
        .collect();
    if negs.is_empty() {
        return 1.0;
    }
    let warned = negs
        .iter()
        .filter(|(t, th)| {
            ev.iter().any(|(t2, e2)| {
                *t2 < *t && *t - *t2 <= 45 && thr(e2) == *th && val(e2) != Valence::Negative
            })
        })
        .count();
    warned as f64 / negs.len() as f64
}

/// Did the world push back harder as the run went on - shaping that
/// raises the criterion as the player learns. Second-half negatives over
/// first-half negatives (1.0 = flat), severity-weighted and per head.
fn m_shaping_slope(r: &RunRecord) -> f64 {
    let half = r.days / 2;
    let alive: std::collections::HashMap<u64, f64> = r.series("villagers.alive").into_iter().collect();
    let (mut early, mut late) = (0.0, 0.0);
    for (t, e) in r.events() {
        if val(e) == Valence::Negative {
            // per head, not per day: a village that grows makes more
            // negatives honestly - growth itself isn't the world rigging
            let w = sev_w(e) / alive.get(&t).copied().unwrap_or(1.0).max(1.0);
            if t < half { early += w; } else { late += w; }
        }
    }
    if early < f64::EPSILON { return if late > 0.0 { 3.0 } else { 0.0 }; }
    late / early
}

/// The seat's craft: total growth across the org.skill.* gauges over the
/// run (end - start, summed). Only meaningful under an active persona.
fn m_skill_growth(r: &RunRecord) -> f64 {
    let (Some(first), Some(last)) = (r.samples.first(), r.samples.last()) else {
        return 0.0;
    };
    first
        .gauges
        .iter()
        .filter(|(k, _)| k.starts_with("org.skill."))
        .map(|(k, v0)| {
            last.gauges
                .iter()
                .find(|(k2, _)| k2 == k)
                .map(|(_, v1)| v1 - v0)
                .unwrap_or(0.0)
        })
        .sum()
}

/// Highest organized power the seat reached (0-1 scale).
fn m_org_power_peak(r: &RunRecord) -> f64 {
    r.series("org.power")
        .iter()
        .map(|(_, v)| *v)
        .fold(0.0f64, f64::max)
}

/// Share of days at least one villager stood sanctioned - how often the
/// world's cold shoulder is actually felt.
fn m_sanction_share(r: &RunRecord) -> f64 {
    let n = r
        .series("sanctioned")
        .iter()
        .filter(|(_, v)| *v >= 1.0)
        .count();
    if r.days == 0 {
        0.0
    } else {
        n as f64 / r.days as f64
    }
}

// ---------- the suite, as data ----------

/// The KPI table, first cut of the bands from the lit reviews
/// (wiki/literature-reviews/game-design-fun-supply-chain.md,
/// balance-testing-methodology.md, narrative-storytelling-arcs.md,
/// reward-punishment-conditioning.md). All bands are research-anchored
/// priors - the reviews themselves call for calibration over a baseline
/// ensemble of seeded runs; expect them to move.
pub fn default_kpis() -> Vec<Kpi> {
    vec![
        // -- fun / pacing (game-design-fun-supply-chain) --
        Kpi { active_only: false, name: "time_to_first_reward", domain: Domain::Fun, unit: "days",
            target: (0.0, 40.0), flag_below: None, flag_above: Some(80.0),
            why: "Civ's one-more-turn hook needs an early payoff; stream 1 puts it <=40t, red >80t",
            measure: m_first_reward },
        Kpi { active_only: false, name: "beat_density", domain: Domain::Fun, unit: "beats/10d",
            target: (2.0, 8.0), flag_below: Some(0.5), flag_above: Some(15.0),
            why: "decision_density 2-6/10d on Notable+ beats (Minor lines are texture, not events); #1132's tags made the count honest",
            measure: m_beat_density },
        Kpi { active_only: false, name: "flatline_max", domain: Domain::Fun, unit: "days",
            target: (0.0, 20.0), flag_below: None, flag_above: Some(40.0),
            why: "the fun rubric's standing rule: 10+ ticks with no change is the 'would alt-tab here' line",
            measure: m_flatline },
        Kpi { active_only: true, name: "time_to_first_growth", domain: Domain::Fun, unit: "days",
            target: (0.0, 120.0), flag_below: None, flag_above: Some(250.0),
            why: "org_growth_shape S-curve: first member by ~day 80-150; no growth at 250 reads as stalled",
            measure: m_first_growth },
        // -- balance methodology (balance-testing-methodology) --
        Kpi { active_only: false, name: "econ_gold_spread", domain: Domain::Balance, unit: "top/mean",
            target: (0.5, 8.0), flag_below: None, flag_above: Some(15.0),
            why: "wealth concentration: some spread is natural; >8x the mean is a sealed economy (prices sit on a fixed table, so the purse is the pulse)",
            measure: m_gold_spread },
        Kpi { active_only: false, name: "hunger_pressure", domain: Domain::Balance, unit: "share",
            target: (0.05, 0.30), flag_below: None, flag_above: Some(0.6),
            why: "the review's pressure band: 0.05-0.30 of the village seeking food on a mean day; ~0 reads as a food system that never bites, >0.6 sustained is famine",
            measure: m_hunger_pressure },
        Kpi { active_only: false, name: "starving_peak", domain: Domain::Balance, unit: "share",
            target: (0.0, 0.05), flag_below: None, flag_above: Some(0.15),
            why: "the hard red line: anyone at <=10 hunger is in real trouble; >15% of the village there is a crisis",
            measure: m_starving_peak },
        Kpi { active_only: false, name: "stockout_share", domain: Domain::Balance, unit: "share",
            target: (0.0, 0.2), flag_below: None, flag_above: Some(0.5),
            why: "empty store most days is a food-chain softlock; scarcity is pressure, absence is a wall",
            measure: m_stockout_share },
        Kpi { active_only: false, name: "food_cover_days", domain: Domain::Balance, unit: "days",
            target: (2.0, 30.0), flag_below: Some(0.5), flag_above: None,
            why: "stock buffer coverage: under half a day per villager is famine-adjacent; the review wants a real buffer",
            measure: m_food_cover },
        Kpi { active_only: false, name: "mortality_per_100d", domain: Domain::Balance, unit: "deaths/100d",
            target: (0.0, 3.0), flag_below: None, flag_above: Some(8.0),
            why: "starvation+illness mortality ~1-8%/yr: >3/100d sustained is attrition a player can't attribute",
            measure: m_mortality },
        Kpi { active_only: false, name: "world_liveliness", domain: Domain::Balance, unit: "share",
            target: (0.6, 1.0), flag_below: Some(0.2), flag_above: None,
            why: "stuck-state detection: <5% flat windows is the review's line; a frozen sim can't teach",
            measure: m_liveliness },
        // -- narrative arcs (narrative-storytelling-arcs) --
        Kpi { active_only: false, name: "beat_interval_cv", domain: Domain::Narrative, unit: "cv",
            target: (0.3, 1.2), flag_below: Some(0.15), flag_above: Some(3.0),
            why: "the arcs review: CV 0.3-0.8 reads as drama, ~0 is a metronome, very high is dead air + pile-ups",
            measure: m_beat_interval_cv },
        Kpi { active_only: false, name: "live_threads", domain: Domain::Narrative, unit: "count",
            target: (3.0, 9.0), flag_below: Some(2.0), flag_above: None,
            why: "braided subplots: median 2-4 concurrent threads is the soap-opera floor, >6 is soup",
            measure: m_thread_count },
        Kpi { active_only: false, name: "resolution_rate", domain: Domain::Narrative, unit: "share",
            target: (0.3, 1.0), flag_below: Some(0.1), flag_above: None,
            why: ">=0.7 of threats resolve is the arcs review's bar; below a third answered, stories dangle",
            measure: m_resolution },
        Kpi { active_only: false, name: "dead_air_max", domain: Domain::Narrative, unit: "days",
            target: (0.0, 60.0), flag_below: None, flag_above: Some(100.0),
            why: "event_drought <=60t with a >100t flag from the fun review - a valley, not a void",
            measure: m_dead_air },
        // -- reward schedules (reward-punishment-conditioning) --
        Kpi { active_only: false, name: "reward_interval_cv", domain: Domain::Reward, unit: "cv",
            target: (0.5, 1.2), flag_below: None, flag_above: Some(3.0),
            why: "VI-schedule flesh over the FI skeleton: reward intervals CV 0.5-1.2, metronome goes flat",
            measure: m_reward_interval_cv },
        Kpi { active_only: false, name: "reward_drought_max", domain: Domain::Reward, unit: "days",
            target: (0.0, 60.0), flag_below: None, flag_above: Some(90.0),
            why: "the extinction window: >90d rewardless is where real players walk (K5/K6)",
            measure: m_reward_drought },
        Kpi { active_only: false, name: "valence_ratio", domain: Domain::Reward, unit: "pos/neg",
            target: (2.5, 5.0), flag_below: Some(1.0), flag_above: None,
            why: "loss aversion ~2:1 + the 5:1 praise line: 2.5-5 good per bad is hard-but-fair; <1 is cruelty",
            measure: m_valence_ratio },
        Kpi { active_only: false, name: "punishment_warn", domain: Domain::Reward, unit: "share",
            target: (0.5, 1.0), flag_below: Some(0.2), flag_above: None,
            why: "natural consequences need warning: writ before knights; <0.5 reads as arbitrary, breeds helplessness",
            measure: m_punish_warn },
        Kpi { active_only: false, name: "shaping_slope", domain: Domain::Reward, unit: "ratio",
            target: (0.5, 5.0), flag_below: None, flag_above: None,
            why: "successive approximations: pressure ramps with the run - per head now, so growth doesn't read as rigging; >5x back-half spike does",
            measure: m_shaping_slope },
        // -- the seat itself (active personas only; gauges existed but
        //    no KPI read them - #1132) --
        Kpi { active_only: true, name: "organizer_skill_growth", domain: Domain::Fun, unit: "pts",
            target: (1.0, 30.0), flag_below: None, flag_above: None,
            why: "earned verbs: the seat's craft should grow over a run - org.skill.* end-start; band is a prior pending the matrix calibration",
            measure: m_skill_growth },
        Kpi { active_only: true, name: "org_power_peak", domain: Domain::Balance, unit: "0-1",
            target: (0.05, 0.95), flag_below: None, flag_above: None,
            why: "an active organizer should reach real organized power sometime in a run - ceiling sits under saturation; prior pending calibration",
            measure: m_org_power_peak },
        Kpi { active_only: false, name: "sanction_share", domain: Domain::Balance, unit: "share",
            target: (0.0, 0.3), flag_below: None, flag_above: Some(0.7),
            why: "the cold shoulder should be felt sometimes but not constantly - sanctioned on >70% of days reads as run repression",
            measure: m_sanction_share },
    ]
}

/// OKRs: one objective per rubric, its key results named by KPI.
pub fn default_okrs() -> Vec<Okr> {
    vec![
        Okr { domain: Domain::Fun,
            objective: "the run holds attention - payoff early, world alive, no dead stretches",
            key_results: &["time_to_first_reward", "beat_density", "flatline_max", "time_to_first_growth"] },
        Okr { domain: Domain::Balance,
            objective: "hard but legible - scarcity without softlock, prices that breathe, losses that attribute",
            key_results: &["econ_gold_spread", "hunger_pressure", "starving_peak", "stockout_share", "food_cover_days", "mortality_per_100d", "world_liveliness"] },
        Okr { domain: Domain::Narrative,
            objective: "the village tells stories - braided threads on an irregular rhythm that resolve",
            key_results: &["beat_interval_cv", "live_threads", "resolution_rate", "dead_air_max"] },
        Okr { domain: Domain::Reward,
            objective: "rewards train the player - variable cadence, no extinction drought, fair warning before foul",
            key_results: &["reward_interval_cv", "reward_drought_max", "valence_ratio", "punishment_warn", "shaping_slope"] },
    ]
}

// ---------- scoring ----------

fn status_of(value: f64, k: &Kpi) -> Status {
    if k.flag_below.map(|b| value < b).unwrap_or(false)
        || k.flag_above.map(|a| value > a).unwrap_or(false) { return Status::Fail; }
    if value >= k.target.0 && value <= k.target.1 { Status::Pass } else { Status::Warn }
}

/// The tester itself: a fixed suite over a recorded run.
pub struct BalanceTester {
    pub name: String,
    pub kpis: Vec<Kpi>,
    pub okrs: Vec<Okr>,
}

impl BalanceTester {
    pub fn new() -> Self {
        Self { name: "Balance Tester".into(), kpis: default_kpis(), okrs: default_okrs() }
    }

    /// Mark a run. Overall is the mean domain score, 0-100. KPIs that
    /// only an active seat can move are skipped on the idle run.
    pub fn score(&self, run: &RunRecord) -> BalanceReport {
        let results: Vec<KpiResult> = self
            .kpis
            .iter()
            .filter(|k| !(k.active_only && run.persona == "idle"))
            .map(|k| {
                let v = (k.measure)(run);
                KpiResult {
                    name: k.name,
                    domain: k.domain,
                    value: v,
                    target: k.target,
                    status: status_of(v, k),
                    unit: k.unit,
                    why: k.why,
                }
            })
            .collect();

        let domain_scores: Vec<(Domain, f64)> = [
            Domain::Fun,
            Domain::Balance,
            Domain::Narrative,
            Domain::Reward,
        ]
        .iter()
        .map(|d| {
            let rs: Vec<&KpiResult> = results.iter().filter(|r| r.domain == *d).collect();
            let s = mean(
                &rs.iter()
                    .map(|r| match r.status {
                        Status::Pass => 1.0,
                        Status::Warn => 0.5,
                        Status::Fail => 0.0,
                    })
                    .collect::<Vec<_>>(),
            );
            (*d, s)
        })
        .collect();

        let okr_hits: Vec<(&'static str, usize, usize)> = self
            .okrs
            .iter()
            .map(|o| {
                let met = o
                    .key_results
                    .iter()
                    .filter(|kr| {
                        results
                            .iter()
                            .any(|r| r.name == **kr && r.status == Status::Pass)
                    })
                    .count();
                (o.objective, met, o.key_results.len())
            })
            .collect();

        let flags: Vec<String> = results
            .iter()
            .filter(|r| r.status == Status::Fail)
            .map(|r| {
                format!(
                    "{} = {:.2} {} (target {:.2}-{:.2})",
                    r.name, r.value, r.unit, r.target.0, r.target.1
                )
            })
            .collect();

        let overall = mean(&domain_scores.iter().map(|(_, s)| *s).collect::<Vec<_>>()) * 100.0;
        let verdict = if overall >= 80.0 {
            "rewarding - the game pays its debts".into()
        } else if overall >= 60.0 {
            "playable - rewarding in parts, thin in others".into()
        } else if overall >= 40.0 {
            "hard and hollow - needs tuning before it teaches".into()
        } else {
            "punishing - the world takes more than it gives".into()
        };

        BalanceReport {
            seed: run.seed,
            days: run.days,
            persona: run.persona.clone(),
            results,
            domain_scores,
            okr_hits,
            flags,
            overall,
            verdict,
        }
    }

    /// The report as text: KPI lines by domain, then the OKR rollup.
    pub fn render(&self, report: &BalanceReport) -> String {
        let mut out = format!(
            "=== BALANCE TEST (seed {}, {} days, persona: {}) ===\n",
            report.seed, report.days, report.persona
        );
        for d in [
            Domain::Fun,
            Domain::Balance,
            Domain::Narrative,
            Domain::Reward,
        ] {
            let s = report
                .domain_scores
                .iter()
                .find(|(dd, _)| *dd == d)
                .map(|(_, s)| *s)
                .unwrap_or(0.0);
            out += &format!("\n-- {} ({:.0}%) --\n", d.label(), s * 100.0);
            for r in report.results.iter().filter(|r| r.domain == d) {
                let mark = match r.status { Status::Pass => " OK ", Status::Warn => "WARN", Status::Fail => "FAIL" };
                out += &format!("  [{mark}] {:<22} {:>8.2} {:<8} target {:.2}-{:.2}   {}\n",
                    r.name, r.value, r.unit, r.target.0, r.target.1, r.why);
            }
        }
        out += "\n-- OKRs --\n";
        for (obj, met, of) in &report.okr_hits {
            out += &format!("  {}/{}  {}\n", met, of, obj);
        }
        if !report.flags.is_empty() {
            out += "\n-- red flags --\n";
            for f in &report.flags { out += &format!("  ! {}\n", f); }
        }
        out += &format!("\noverall: {:.0}/100 - {}\n", report.overall, report.verdict);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(tag: EvTag, text: &str) -> EvLine {
        EvLine {
            tag: Some(tag),
            text: text.into(),
        }
    }
    fn beat(t: Thread, v: Valence) -> EvTag {
        EvTag::new(t, v, Sev::Notable)
    }

    fn gauge_day(tick: u64, alive: f64, events: Vec<EvLine>) -> DaySample {
        DaySample {
            tick,
            gauges: vec![("villagers.alive".into(), alive), ("org.members".into(), 3.0),
                ("price.grain".into(), 10.0 + (tick % 7) as f64 * 0.1)],
            events,
        }
    }

    fn run(days: u64, events: &[(u64, EvLine)]) -> RunRecord {
        let mut samples = vec![];
        for t in 0..days {
            let day: Vec<EvLine> = events
                .iter()
                .filter(|(et, _)| *et == t)
                .map(|(_, e)| e.clone())
                .collect();
            samples.push(gauge_day(t, 30.0, day));
        }
        RunRecord {
            seed: 1,
            days,
            persona: "test".into(),
            samples,
        }
    }

    #[test]
    fn test_tags_read_beats_not_phrasing() {
        // the line's tag decides - the scorer never guesses from words
        let tagged = ev(
            EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Major),
            "Bram died.",
        );
        assert!(is_beat(&tagged) && val(&tagged) == Valence::Negative);
        // forewarnings and patter aren't beats even when charged-sounding
        let warn = ev(
            EvTag::new(Thread::Law, Valence::Neutral, Sev::Notable),
            "riders asking after a deserter",
        );
        assert!(!is_beat(&warn) && thr(&warn) == Thread::Law);
        let weather = ev(
            EvTag::new(Thread::Weather, Valence::Positive, Sev::Minor),
            "a rainbow!",
        );
        assert!(!is_beat(&weather));
        // untagged lines are invisible to every event measure
        let plain = EvLine {
            tag: None,
            text: "a hint line".into(),
        };
        assert!(!is_beat(&plain) && sev_w(&plain) == 0.0);
    }

    #[test]
    fn test_valence_ratio_weighs_severity() {
        // one Major negative vs two Minor positives: the blow outweighs
        let events = vec![
            (
                3u64,
                ev(beat(Thread::Lifecycle, Valence::Positive), "a kind word"),
            ),
            (
                4u64,
                ev(beat(Thread::Lifecycle, Valence::Positive), "a small win"),
            ),
            (
                5u64,
                ev(
                    EvTag::new(Thread::Law, Valence::Negative, Sev::Major),
                    "Bram hanged.",
                ),
            ),
        ];
        let r = run(10, &events);
        // pos weight 2+2=4, neg weight 4 -> ratio 1.0, not 2.0
        assert!(
            (m_valence_ratio(&r) - 1.0).abs() < 0.01,
            "ratio {}",
            m_valence_ratio(&r)
        );
    }

    #[test]
    fn test_minor_blows_need_no_herald() {
        // a Minor negative with no warning still shouldn't drag
        // punishment_warn down - only Notable+ punishment needs a herald
        let events = vec![(
            5u64,
            ev(
                EvTag::new(Thread::Law, Valence::Negative, Sev::Minor),
                "a fine",
            ),
        )];
        let r = run(20, &events);
        assert!((m_punish_warn(&r) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_idle_persona_skips_active_kpis() {
        let mut r = run(100, &[]);
        r.persona = "idle".into();
        let t = BalanceTester::new();
        let report = t.score(&r);
        assert!(
            !report
                .results
                .iter()
                .any(|k| k.name == "time_to_first_growth"
                    || k.name == "organizer_skill_growth"
                    || k.name == "org_power_peak"),
            "active-only KPIs should be skipped on idle"
        );
    }

    #[test]
    fn test_a_quiet_run_scores_poorly() {
        let t = BalanceTester::new();
        let report = t.score(&run(100, &[]));
        assert!(report.overall < 60.0, "dead air should not pass: {}", report.overall);
        assert!(!report.flags.is_empty());
    }

    #[test]
    fn test_a_lively_run_scores_well() {
        let mut events: Vec<(u64, EvLine)> = vec![];
        for t in (3..200).step_by(9) {
            events.push((
                t,
                ev(
                    beat(Thread::Visitor, Valence::Positive),
                    "the village gathers at the market.",
                ),
            ));
            events.push((
                t + 3,
                ev(
                    beat(Thread::Law, Valence::Negative),
                    "the tax collector takes his cut (5 gold).",
                ),
            ));
            events.push((
                t + 5,
                ev(
                    beat(Thread::Visitor, Valence::Positive),
                    "the pedlar comes up the road.",
                ),
            ));
        }
        let t = BalanceTester::new();
        let report = t.score(&run(200, &events));
        let fun = report.domain_scores.iter().find(|(d, _)| *d == Domain::Fun).unwrap().1;
        assert!(fun >= 0.5, "a storied run should score on fun: {fun}");
        assert!(report.overall >= 50.0, "overall {}", report.overall);
    }

    #[test]
    fn test_extinction_drought_flags() {
        let mut events: Vec<(u64, EvLine)> = vec![(
            5,
            ev(
                beat(Thread::Visitor, Valence::Positive),
                "the pedlar comes up the road.",
            ),
        )];
        for t in (10..200).step_by(7) {
            events.push((
                t,
                ev(
                    beat(Thread::Illness, Valence::Negative),
                    "Bram is taken with la grippe",
                ),
            ));
        }
        let t = BalanceTester::new();
        let report = t.score(&run(200, &events));
        assert!(report.results.iter().any(|r| r.name == "reward_drought_max" && r.status == Status::Fail));
    }
}
