//! Balance Tester: scores long headless runs against the four research
//! rubrics - fun/pacing, balance methodology, narrative arcs, reward
//! schedules (see wiki/literature-reviews/). The game is meant hard, even
//! unfair; the tester's job is to tell "hard but rewarding" from "hard
//! and hollow". Each KPI is a number pulled off the day's gauges and the
//! village's own log, checked against a band; the bands come out of the
//! lit reviews, not thin air.

use std::collections::HashMap;

// ---------- recorded runs ----------

/// One day of a run: the gauges at day's end and what the log said.
#[derive(Debug, Clone)]
pub struct DaySample {
    pub tick: u64,
    pub gauges: Vec<(String, f64)>,
    /// Log lines the village wrote this day (the "t{n}:" prefix stripped).
    pub events: Vec<String>,
}

/// A whole run, day by day.
#[derive(Debug)]
pub struct RunRecord {
    pub seed: u64,
    pub days: u64,
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
    pub fn events(&self) -> Vec<(u64, &str)> {
        self.samples.iter()
            .flat_map(|s| s.events.iter().map(move |e| (s.tick, e.as_str())))
            .collect()
    }
}

// ---------- event reading ----------

/// What a log line does to a player's day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Valence { Positive, Negative, Neutral }

/// Which story the line belongs to - the braids a narrative score counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Thread {
    Weather, Illness, Fire, Law, Beast, Visitor, Org, Economy,
    Lifecycle, Craft, Player, Other,
}

/// Read one log line (sans the "t{n}:" prefix) for its charge and its
/// story. Keyword matching on the village's own phrasing - a thin but
/// honest classifier, and every word here is a string the game writes.
pub fn classify(entry: &str) -> (Valence, Thread) {
    let e = entry.to_lowercase();
    let has = |w: &[&str]| w.iter().any(|k| e.contains(k));

    // command feedback isn't a village event: "Can't recruit: need
    // influence 0.40" is the game answering a button press, not news
    if e.starts_with("can't ") || e.starts_with("you can't") || e.starts_with("no one is near") {
        return (Valence::Neutral, Thread::Player);
    }

    // grief first: a death or a hanging outweighs anything else in the line
    if has(&[" died", "hanged", "driven out", "turns ", "run off"]) {
        return (Valence::Negative, Thread::Lifecycle);
    }
    // word of the law riding in is neither a writ nor a defiance: the
    // warning beat itself is neutral news on the Law thread
    if has(&["rides this way", "rides out this way"]) {
        return (Valence::Neutral, Thread::Law);
    }
    if has(&["stocks", "summons", "taken for questioning", "held at the castle", "bailiff", "writ", "reckoning"]) {
        let v = if has(&["stands against", "rides home empty", "walks free"]) { Valence::Positive } else { Valence::Negative };
        return (v, Thread::Law);
    }
    if has(&["flu", "fever", "illness", "recovered", "treats", "cure", "taken with", "grippe", "scarlatina", "gripes", "sweats", "consumption", "ague"]) {
        let v = if has(&["recovered", "treats", "cure"]) { Valence::Positive } else { Valence::Negative };
        return (v, Thread::Illness);
    }
    // word of the beast riding ahead of it is forewarning, not the blow:
    // tracks and restless hens are neutral news on the Beast thread
    if has(&["wolf tracks", "won't settle", "prowls the hedges"]) {
        return (Valence::Neutral, Thread::Beast);
    }
    if has(&["wolf", "fox"]) { return (Valence::Negative, Thread::Beast); }
    if has(&["fire", "alight", "burnt", "burning"]) { return (Valence::Negative, Thread::Fire); }
    if has(&["tax", "collector takes", "levy", "levies"]) { return (Valence::Negative, Thread::Law); }
    if has(&["rainbow", "festival", "gathers at the market", "circus comes", "minstrel", "pedlar comes",
             "ink-maker comes", "shearing day", "swarm took a liking", "pale stone",
             "letter comes", "brought", "together"]) {
        return (Valence::Positive, Thread::Visitor);
    }
    if has(&["skeps yield", "market day", "grant covers"]) {
        // a yield, a selling day, a rebate: the economy's own good news,
        // and the context a shortage lands in - before the Org branch,
        // whose "member" would otherwise claim market day's pay line
        return (Valence::Positive, Thread::Economy);
    }
    if has(&["guild formed", "stands with", "pulls together", "shares out", "ransom", "walks home",
             "takes up the organizing", "tread lightly", "passes you by", "vigil"]) {
        return (Valence::Positive, Thread::Org);
    }
    if has(&["recruit", "joined", "member"]) { return (Valence::Positive, Thread::Org); }
    if has(&["boycott", "shuttered", "derelict", "thin stores", "drowned", "spoiled", "starv"]) {
        return (Valence::Negative, Thread::Economy);
    }
    if has(&["takes up the", "bench", "colors", "tallies straight", "lays down", "outlast"]) {
        return (Valence::Positive, Thread::Craft);
    }
    if has(&["cold shoulder", "talk turns", "tells ", "gossip", "word spreads"]) {
        return (Valence::Neutral, Thread::Lifecycle);
    }
    if has(&["begins", "stops", "winter sets", "drizzle", "snow", "rain"]) {
        let v = if e.contains("winter sets in hard") { Valence::Negative } else { Valence::Neutral };
        return (v, Thread::Weather);
    }
    if has(&["you ", "your "]) { return (Valence::Neutral, Thread::Player); }
    (Valence::Neutral, Thread::Other)
}

/// A log line worth counting as a beat - weather patter and passing
/// remarks don't carry a story, so they're left out of event metrics.
pub fn is_beat(entry: &str) -> bool {
    !matches!(classify(entry).1, Thread::Weather | Thread::Player | Thread::Other)
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
    r.samples.iter().find(|s| s.events.iter().any(|e| classify(e).0 == Valence::Positive))
        .map(|s| s.tick).unwrap_or(r.days) as f64
}

/// Beats per 10 days, weather patter excluded.
fn m_beat_density(r: &RunRecord) -> f64 {
    let beats = r.events().into_iter().filter(|(_, e)| is_beat(e)).count();
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

/// Share of days the village food store stood empty.
fn m_stockout_share(r: &RunRecord) -> f64 {
    let food: HashMap<u64, f64> = r.series("stock.food").into_iter().collect();
    let n = r.samples.iter().filter(|s| food.get(&s.tick).copied().unwrap_or(0.0) <= 0.0).count();
    if r.days == 0 { 0.0 } else { n as f64 / r.days as f64 }
}

/// Median days of cover in the store: servings on hand per villager,
/// read as "how long could the village eat if all supply stopped".
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
fn m_beat_interval_cv(r: &RunRecord) -> f64 {
    let days: Vec<u64> = r.events().into_iter().filter(|(_, e)| is_beat(e)).map(|(t, _)| t).collect();
    if days.len() < 3 { return 0.0; }
    cv(&intervals(&days))
}

/// How many story threads stayed live (3+ beats each).
fn m_thread_count(r: &RunRecord) -> f64 {
    let mut by_thread: HashMap<Thread, usize> = HashMap::new();
    for (_, e) in r.events() { if is_beat(e) { *by_thread.entry(classify(e).1).or_default() += 1; } }
    by_thread.values().filter(|&&n| n >= 3).count() as f64
}

/// Trouble that found its answer: share of negative beats followed by a
/// positive beat inside 30 days.
fn m_resolution(r: &RunRecord) -> f64 {
    let ev = r.events();
    let pos_days: Vec<u64> = ev.iter().filter(|(_, e)| classify(e).0 == Valence::Positive).map(|(t, _)| *t).collect();
    let negs: Vec<u64> = ev.iter().filter(|(_, e)| classify(e).0 == Valence::Negative).map(|(t, _)| *t).collect();
    if negs.is_empty() { return 1.0; }
    let resolved = negs.iter().filter(|t| pos_days.iter().any(|p| *p > **t && *p - **t <= 30)).count();
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
    let days: Vec<u64> = r.events().into_iter()
        .filter(|(_, e)| classify(e).0 == Valence::Positive).map(|(t, _)| t).collect();
    if days.len() < 3 { return 0.0; }
    cv(&intervals(&days))
}

/// Longest run with no positive beat - the extinction window a real
/// player walks away inside.
fn m_reward_drought(r: &RunRecord) -> f64 {
    let mut days: Vec<u64> = r.events().into_iter()
        .filter(|(_, e)| classify(e).0 == Valence::Positive).map(|(t, _)| t).collect();
    days.insert(0, 0); days.push(r.days);
    intervals(&days).into_iter().fold(0.0, f64::max)
}

/// Good days per bad one. A hard game still pays out more than it takes.
fn m_valence_ratio(r: &RunRecord) -> f64 {
    let ev = r.events();
    let pos = ev.iter().filter(|(_, e)| classify(e).0 == Valence::Positive).count() as f64;
    let neg = ev.iter().filter(|(_, e)| classify(e).0 == Valence::Negative).count() as f64;
    if neg < 1.0 { return if pos > 0.0 { 10.0 } else { 0.0 }; }
    pos / neg
}

/// A fair punishment is one you could see coming: share of negative
/// beats whose own thread was already active (non-negative event) inside
/// the 45 days before. Natural-consequences logic - writ before knights.
fn m_punish_warn(r: &RunRecord) -> f64 {
    let ev = r.events();
    let negs: Vec<(u64, Thread)> = ev.iter().filter_map(|(t, e)| {
        let (v, th) = classify(e);
        (v == Valence::Negative).then_some((*t, th))
    }).collect();
    if negs.is_empty() { return 1.0; }
    let warned = negs.iter().filter(|(t, th)| ev.iter().any(|(t2, e2)|
        *t2 < *t && *t - *t2 <= 45
            && classify(e2).1 == *th
            && classify(e2).0 != Valence::Negative
    )).count();
    warned as f64 / negs.len() as f64
}

/// Did the world push back harder as the run went on - shaping that
/// raises the criterion as the player learns. Second-half negatives over
/// first-half negatives (1.0 = flat).
fn m_shaping_slope(r: &RunRecord) -> f64 {
    let half = r.days / 2;
    let (mut early, mut late) = (0.0, 0.0);
    for (t, e) in r.events() {
        if classify(e).0 == Valence::Negative {
            if t < half { early += 1.0; } else { late += 1.0; }
        }
    }
    if early < 1.0 { return if late > 0.0 { 3.0 } else { 0.0 }; }
    late / early
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
        Kpi { name: "time_to_first_reward", domain: Domain::Fun, unit: "days",
            target: (0.0, 40.0), flag_below: None, flag_above: Some(80.0),
            why: "Civ's one-more-turn hook needs an early payoff; stream 1 puts it <=40t, red >80t",
            measure: m_first_reward },
        Kpi { name: "beat_density", domain: Domain::Fun, unit: "beats/10d",
            target: (2.0, 6.0), flag_below: Some(0.5), flag_above: Some(15.0),
            why: "decision_density from the supply-chain review: 2-6 events/10d, <0.5 sustained is idle",
            measure: m_beat_density },
        Kpi { name: "flatline_max", domain: Domain::Fun, unit: "days",
            target: (0.0, 20.0), flag_below: None, flag_above: Some(40.0),
            why: "the fun rubric's standing rule: 10+ ticks with no change is the 'would alt-tab here' line",
            measure: m_flatline },
        Kpi { name: "time_to_first_growth", domain: Domain::Fun, unit: "days",
            target: (0.0, 120.0), flag_below: None, flag_above: Some(250.0),
            why: "org_growth_shape S-curve: first member by ~day 80-150; no growth at 250 reads as stalled",
            measure: m_first_growth },
        // -- balance methodology (balance-testing-methodology) --
        Kpi { name: "econ_gold_spread", domain: Domain::Balance, unit: "top/mean",
            target: (0.5, 8.0), flag_below: None, flag_above: Some(15.0),
            why: "wealth concentration: some spread is natural; >8x the mean is a sealed economy (prices sit on a fixed table, so the purse is the pulse)",
            measure: m_gold_spread },
        Kpi { name: "hunger_pressure", domain: Domain::Balance, unit: "share",
            target: (0.05, 0.30), flag_below: None, flag_above: Some(0.6),
            why: "the review's pressure band: 0.05-0.30 of the village seeking food on a mean day; ~0 reads as a food system that never bites, >0.6 sustained is famine",
            measure: m_hunger_pressure },
        Kpi { name: "starving_peak", domain: Domain::Balance, unit: "share",
            target: (0.0, 0.05), flag_below: None, flag_above: Some(0.15),
            why: "the hard red line: anyone at <=10 hunger is in real trouble; >15% of the village there is a crisis",
            measure: m_starving_peak },
        Kpi { name: "stockout_share", domain: Domain::Balance, unit: "share",
            target: (0.0, 0.2), flag_below: None, flag_above: Some(0.5),
            why: "empty store most days is a food-chain softlock; scarcity is pressure, absence is a wall",
            measure: m_stockout_share },
        Kpi { name: "food_cover_days", domain: Domain::Balance, unit: "days",
            target: (2.0, 30.0), flag_below: Some(0.5), flag_above: None,
            why: "stock buffer coverage: under half a day per villager is famine-adjacent; the review wants a real buffer",
            measure: m_food_cover },
        Kpi { name: "mortality_per_100d", domain: Domain::Balance, unit: "deaths/100d",
            target: (0.0, 3.0), flag_below: None, flag_above: Some(8.0),
            why: "starvation+illness mortality ~1-8%/yr: >3/100d sustained is attrition a player can't attribute",
            measure: m_mortality },
        Kpi { name: "world_liveliness", domain: Domain::Balance, unit: "share",
            target: (0.6, 1.0), flag_below: Some(0.2), flag_above: None,
            why: "stuck-state detection: <5% flat windows is the review's line; a frozen sim can't teach",
            measure: m_liveliness },
        // -- narrative arcs (narrative-storytelling-arcs) --
        Kpi { name: "beat_interval_cv", domain: Domain::Narrative, unit: "cv",
            target: (0.3, 1.2), flag_below: Some(0.15), flag_above: Some(3.0),
            why: "the arcs review: CV 0.3-0.8 reads as drama, ~0 is a metronome, very high is dead air + pile-ups",
            measure: m_beat_interval_cv },
        Kpi { name: "live_threads", domain: Domain::Narrative, unit: "count",
            target: (3.0, 9.0), flag_below: Some(2.0), flag_above: None,
            why: "braided subplots: median 2-4 concurrent threads is the soap-opera floor, >6 is soup",
            measure: m_thread_count },
        Kpi { name: "resolution_rate", domain: Domain::Narrative, unit: "share",
            target: (0.3, 1.0), flag_below: Some(0.1), flag_above: None,
            why: ">=0.7 of threats resolve is the arcs review's bar; below a third answered, stories dangle",
            measure: m_resolution },
        Kpi { name: "dead_air_max", domain: Domain::Narrative, unit: "days",
            target: (0.0, 60.0), flag_below: None, flag_above: Some(100.0),
            why: "event_drought <=60t with a >100t flag from the fun review - a valley, not a void",
            measure: m_dead_air },
        // -- reward schedules (reward-punishment-conditioning) --
        Kpi { name: "reward_interval_cv", domain: Domain::Reward, unit: "cv",
            target: (0.5, 1.2), flag_below: None, flag_above: Some(3.0),
            why: "VI-schedule flesh over the FI skeleton: reward intervals CV 0.5-1.2, metronome goes flat",
            measure: m_reward_interval_cv },
        Kpi { name: "reward_drought_max", domain: Domain::Reward, unit: "days",
            target: (0.0, 60.0), flag_below: None, flag_above: Some(90.0),
            why: "the extinction window: >90d rewardless is where real players walk (K5/K6)",
            measure: m_reward_drought },
        Kpi { name: "valence_ratio", domain: Domain::Reward, unit: "pos/neg",
            target: (2.5, 5.0), flag_below: Some(1.0), flag_above: None,
            why: "loss aversion ~2:1 + the 5:1 praise line: 2.5-5 good per bad is hard-but-fair; <1 is cruelty",
            measure: m_valence_ratio },
        Kpi { name: "punishment_warn", domain: Domain::Reward, unit: "share",
            target: (0.5, 1.0), flag_below: Some(0.2), flag_above: None,
            why: "natural consequences need warning: writ before knights; <0.5 reads as arbitrary, breeds helplessness",
            measure: m_punish_warn },
        Kpi { name: "shaping_slope", domain: Domain::Reward, unit: "ratio",
            target: (0.5, 3.0), flag_below: None, flag_above: None,
            why: "successive approximations: pressure ramps with the run, but a >3x back-half spike reads as rigged",
            measure: m_shaping_slope },
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

    /// Mark a run. Overall is the mean domain score, 0-100.
    pub fn score(&self, run: &RunRecord) -> BalanceReport {
        let results: Vec<KpiResult> = self.kpis.iter().map(|k| {
            let v = (k.measure)(run);
            KpiResult { name: k.name, domain: k.domain, value: v, target: k.target,
                status: status_of(v, k), unit: k.unit, why: k.why }
        }).collect();

        let domain_scores: Vec<(Domain, f64)> = [Domain::Fun, Domain::Balance, Domain::Narrative, Domain::Reward]
            .iter().map(|d| {
                let rs: Vec<&KpiResult> = results.iter().filter(|r| r.domain == *d).collect();
                let s = mean(&rs.iter().map(|r| match r.status {
                    Status::Pass => 1.0, Status::Warn => 0.5, Status::Fail => 0.0 }).collect::<Vec<_>>());
                (*d, s)
            }).collect();

        let okr_hits: Vec<(&'static str, usize, usize)> = self.okrs.iter().map(|o| {
            let met = o.key_results.iter().filter(|kr|
                results.iter().any(|r| r.name == **kr && r.status == Status::Pass)).count();
            (o.objective, met, o.key_results.len())
        }).collect();

        let flags: Vec<String> = results.iter().filter(|r| r.status == Status::Fail)
            .map(|r| format!("{} = {:.2} {} (target {:.2}-{:.2})", r.name, r.value, r.unit, r.target.0, r.target.1)).collect();

        let overall = mean(&domain_scores.iter().map(|(_, s)| *s).collect::<Vec<_>>()) * 100.0;
        let verdict = if overall >= 80.0 { "rewarding - the game pays its debts".into() }
            else if overall >= 60.0 { "playable - rewarding in parts, thin in others".into() }
            else if overall >= 40.0 { "hard and hollow - needs tuning before it teaches".into() }
            else { "punishing - the world takes more than it gives".into() };

        BalanceReport { seed: run.seed, days: run.days, results, domain_scores, okr_hits, flags, overall, verdict }
    }

    /// The report as text: KPI lines by domain, then the OKR rollup.
    pub fn render(&self, report: &BalanceReport) -> String {
        let mut out = format!("=== BALANCE TEST (seed {}, {} days) ===\n", report.seed, report.days);
        for d in [Domain::Fun, Domain::Balance, Domain::Narrative, Domain::Reward] {
            let s = report.domain_scores.iter().find(|(dd, _)| *dd == d).map(|(_, s)| *s).unwrap_or(0.0);
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

    fn gauge_day(tick: u64, alive: f64, events: &[&str]) -> DaySample {
        DaySample {
            tick,
            gauges: vec![("villagers.alive".into(), alive), ("org.members".into(), 3.0),
                ("price.grain".into(), 10.0 + (tick % 7) as f64 * 0.1)],
            events: events.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn run(days: u64, events: &[(u64, &str)]) -> RunRecord {
        let mut samples = vec![];
        for t in 0..days {
            let ev: Vec<&str> = events.iter().filter(|(et, _)| *et == t).map(|(_, e)| *e).collect();
            samples.push(gauge_day(t, 30.0, &ev));
        }
        RunRecord { seed: 1, days, samples }
    }

    #[test]
    fn test_classify_reads_the_village() {
        assert_eq!(classify("Bram died.").0, Valence::Negative);
        assert_eq!(classify("the rain stops - a rainbow!").0, Valence::Positive);
        assert_eq!(classify("the flu spreads").1, Thread::Illness);
        assert_eq!(classify("drizzle begins").1, Thread::Weather);
        assert_eq!(classify("the Merchant Guild formed!").1, Thread::Org);
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
        let mut events: Vec<(u64, &str)> = vec![];
        for t in (3..200).step_by(9) {
            events.push((t, "the village gathers at the market."));
            events.push((t + 3, "the tax collector takes his cut (5 gold)."));
            events.push((t + 5, "the pedlar comes up the road."));
        }
        let t = BalanceTester::new();
        let report = t.score(&run(200, &events));
        let fun = report.domain_scores.iter().find(|(d, _)| *d == Domain::Fun).unwrap().1;
        assert!(fun >= 0.5, "a storied run should score on fun: {fun}");
        assert!(report.overall >= 50.0, "overall {}", report.overall);
    }

    #[test]
    fn test_extinction_drought_flags() {
        let mut events: Vec<(u64, &str)> = vec![(5, "the pedlar comes up the road.")];
        for t in (10..200).step_by(7) { events.push((t, "Bram is taken with la grippe")); }
        let t = BalanceTester::new();
        let report = t.score(&run(200, &events));
        assert!(report.results.iter().any(|r| r.name == "reward_drought_max" && r.status == Status::Fail));
    }
}
