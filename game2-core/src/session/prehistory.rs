//! Prehistory (#1419, E2-1; design `docs/plans/procedural-characters.md`
//! part 1, pass 1): the thirty years before play, walked once per world
//! seed. Each event comes off the record table - coroners' inquests,
//! muster rolls, manorial courts, parish burials - and records whom it
//! touched, which is what turns nine identical soldier brothers into
//! one levy with three fates. `persona::generate` reads `story_of` for
//! each cast slot; #1420 grounds the households and #1421 grows the
//! ghosts.

use crate::game::persona::{draw, EventId};

/// The nine record-sources the thirty years are written in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Fire,
    Levy,
    DeathDuty,
    Fever,
    MillerDebt,
    BrokenBetrothal,
    OldFaith,
    Eviction,
    HarvestFeast,
}

/// All of them, in a fixed order so a shared year sorts stable
pub const KINDS: [EventKind; 9] = [
    EventKind::Fire, EventKind::Levy, EventKind::DeathDuty,
    EventKind::Fever, EventKind::MillerDebt, EventKind::BrokenBetrothal,
    EventKind::OldFaith, EventKind::Eviction, EventKind::HarvestFeast,
];

/// Whom an event touched: a cast slot, or a name who never made the
/// roll - the infant the fever took, the deserter, the blamed peddler.
/// Ghosts cite the event, not the ephemeral person
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Who {
    Cast(usize),
    Ephemeral(&'static str),
}

/// One soul in an event's record, and the part they played in it -
/// the parts are what the ghost pass keys on
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Touched {
    pub who: Who,
    pub part: &'static str,
}

/// One prehistory event: what happened, when, and whom it touched
#[derive(Debug, Clone)]
pub struct PrehistoryEvent {
    pub kind: EventKind,
    /// Years before play began, 1 to 30
    pub years_ago: u8,
    pub touched: Vec<Touched>,
}

/// The thirty years' record, plus the proto-households it dealt the
/// cast into - pass 2 places the real homes and kin from these
#[derive(Debug, Clone)]
pub struct Prehistory {
    pub events: Vec<PrehistoryEvent>,
    /// Which household each cast slot sits in
    pub households: Vec<usize>,
}

/// Whom a cast slot's roof gave them (#1420): the elders who raised
/// them, the children raised beside them, and the rest under it. In
/// slots, not entities - `setup` grounds them when the bodies exist
#[derive(Debug, Clone, Default)]
pub struct KinPlan {
    /// The household's elders - shared across its children
    pub parents: Vec<usize>,
    /// The household's other children - shared parents
    pub siblings: Vec<usize>,
    /// The rest under the same roof, old and young
    pub housemates: Vec<usize>,
}

impl PrehistoryEvent {
    /// How many households this event reached - the done-when measure
    pub fn households_touched(&self, ph: &Prehistory) -> usize {
        let mut seen = self.touched.iter().filter_map(|t| match t.who {
            Who::Cast(s) => ph.households.get(s).copied(),
            Who::Ephemeral(_) => None,
        }).collect::<Vec<_>>();
        seen.sort_unstable();
        seen.dedup();
        seen.len()
    }
}

impl Prehistory {
    /// The events a cast slot was in - what `Past.story` carries
    pub fn story_of(&self, slot: usize) -> Vec<EventId> {
        self.events.iter().enumerate()
            .filter(|(_, e)| e.touched.iter().any(|t| t.who == Who::Cast(slot)))
            .map(|(i, _)| EventId(i as u32))
            .collect()
    }

    /// Pass 2 (#1420): each slot's kin, seeded off the world seed. All
    /// of a household's children share the same elders - that's what
    /// makes them siblings - and elders get the rest of the roof as
    /// housemates
    pub fn kin_plan(&self, seed: u64, kids: &[usize]) -> Vec<KinPlan> {
        let mut plan = vec![KinPlan::default(); self.households.len()];
        let n = self.households.iter().max().map(|m| m + 1).unwrap_or(0);
        for h in 0..n {
            let mem = members(&self.households, h);
            let elders: Vec<usize> = mem.iter().copied()
                .filter(|s| !kids.contains(s)).collect();
            let young: Vec<usize> = mem.iter().copied()
                .filter(|s| kids.contains(s)).collect();
            // the elders who raised this roof's children: two when the
            // house holds them, seeded off the household not the kid,
            // so brothers and sisters get the same pair
            let mut parents: Vec<usize> = vec![];
            for d in 0.. {
                if parents.len() >= elders.len().min(2) || elders.is_empty() { break; }
                let p = elders[(draw(seed ^ 0xA17E5, h as u64 * 61 + d)
                    * elders.len() as f32) as usize % elders.len()];
                if !parents.contains(&p) { parents.push(p); }
                if d > 40 { break; }
            }
            for &s in &mem {
                plan[s].parents = if young.contains(&s) { parents.clone() } else { vec![] };
                plan[s].siblings = if young.contains(&s) {
                    young.iter().copied().filter(|&o| o != s).collect()
                } else { vec![] };
                plan[s].housemates = mem.iter().copied()
                    .filter(|&o| o != s
                        && !plan[s].parents.contains(&o)
                        && !plan[s].siblings.contains(&o))
                    .collect();
            }
        }
        plan
    }
}

/// The years the record covers
pub const YEARS: u8 = 30;

/// A kind's mean gap in years between occurrences - plausible rates,
/// tunable: the feast is recurring, the levy comes with the wars, a
/// fever year is once a generation
fn mean_gap(kind: EventKind) -> u8 {
    match kind {
        EventKind::HarvestFeast => 3,
        EventKind::Levy => 7,
        EventKind::DeathDuty => 9,
        EventKind::MillerDebt => 10,
        EventKind::Fire => 12,
        EventKind::Eviction => 14,
        EventKind::BrokenBetrothal => 15,
        EventKind::Fever => 18,
        EventKind::OldFaith => 25,
    }
}

/// Deal the cast into proto-households of one to four: a seeded
/// shuffle cut into runs of the grown folk, then the children placed
/// so the little ones share a roof - the first pair into the smallest
/// household that can hold them, the rest into seeded roofs with room
fn deal_households(seed: u64, cast: usize, kids: &[usize]) -> Vec<usize> {
    let grown: Vec<usize> = (0..cast).filter(|s| !kids.contains(s)).collect();
    let mut order = grown.clone();
    for i in (1..order.len()).rev() {
        let j = (draw(seed ^ 0x7105EED, i as u64) * (i + 1) as f32) as usize;
        order.swap(i, j.min(i));
    }
    let mut households = vec![usize::MAX; cast];
    let mut sizes: Vec<usize> = vec![];
    let mut i = 0usize;
    while i < order.len() {
        let h = sizes.len();
        let size = (1 + (draw(seed ^ 0x7105EED ^ 0x9E37, h as u64) * 4.0) as usize)
            .min(order.len() - i);
        for k in 0..size {
            households[order[i + k]] = h;
        }
        sizes.push(size);
        i += size;
    }
    // a roof that takes the first pair of children and still has an
    // elder standing: the smallest household with room for two
    let mut placed = 0usize;
    if kids.len() >= 2 {
        if let Some(h) = (0..sizes.len())
            .filter(|&h| sizes[h] + 2 <= 4 && sizes[h] >= 1)
            .min_by_key(|&h| sizes[h] * 64 + (draw(seed ^ 0x5EED, h as u64) * 64.0) as usize)
        {
            households[kids[0]] = h;
            households[kids[1]] = h;
            sizes[h] += 2;
            placed = 2;
        }
    }
    for &k in &kids[placed..] {
        // each further child: a seeded roof with room for them
        let mut home = None;
        for d in 0..60 {
            let h = (draw(seed ^ 0x5EED ^ k as u64, d) * sizes.len() as f32) as usize
                % sizes.len().max(1);
            if !sizes.is_empty() && sizes[h] < 4 { home = Some(h); break; }
        }
        match home {
            Some(h) => { households[k] = h; sizes[h] += 1; }
            None => { households[k] = sizes.len(); sizes.push(1); }
        }
    }
    households
}

/// The slots that share a household id
fn members(households: &[usize], h: usize) -> Vec<usize> {
    households.iter().enumerate()
        .filter_map(|(s, &hh)| (hh == h).then_some(s))
        .collect()
}

/// Pick `n` distinct households, seeded off `salt`
fn pick_households(seed: u64, salt: u64, n_households: usize, n: usize) -> Vec<usize> {
    let mut picked: Vec<usize> = vec![];
    for d in 0.. {
        let h = (draw(seed ^ salt, d) * n_households as f32) as usize % n_households;
        if !picked.contains(&h) {
            picked.push(h);
            if picked.len() == n { break; }
        }
        if d > 60 { break; } // the pool's dealt; take what's there
    }
    picked
}

/// One member of household `h`, seeded off `salt`
fn member_of(households: &[usize], h: usize, seed: u64, salt: u64) -> usize {
    let m = members(households, h);
    m[(draw(seed ^ salt, h as u64) * m.len() as f32) as usize % m.len()]
}

/// The event each kind writes when its year comes - every pick is off
/// `seed ^ salt`, so the walk's draws don't shift when a kind's count
/// does
fn record(kind: EventKind, year: u8, seed: u64, salt: u64,
    households: &[usize], n_households: usize) -> PrehistoryEvent
{
    let d = |n: u64| draw(seed ^ salt, n);
    let touched: Vec<Touched> = match kind {
        EventKind::HarvestFeast => households.iter().enumerate()
            .map(|(s, _)| Touched { who: Who::Cast(s), part: "feasted" })
            .collect(),
        EventKind::Levy => {
            // two to four young men across families; most never come
            // back and only their households remember them
            let n = 2 + (d(1) * 3.0) as usize;
            let mut t = vec![];
            for (k, h) in pick_households(seed, salt ^ 11, n_households, n)
                .into_iter().enumerate()
            {
                let s = member_of(households, h, seed, salt ^ (20 + k as u64));
                if d(40 + k as u64) < 0.3 {
                    t.push(Touched { who: Who::Cast(s), part: "served and came home" });
                } else {
                    t.push(Touched { who: Who::Cast(s), part: "remembers one the levy took" });
                    t.push(Touched { who: Who::Ephemeral(match d(50 + k as u64) {
                        x if x < 0.4 => "the son who never wrote",
                        x if x < 0.7 => "the brother who died in the wars",
                        _ => "the one who deserted",
                    }), part: "taken" });
                }
            }
            t
        }
        EventKind::Fire => {
            // a household's barn, the neighbors who saw it, and someone
            // the village found to blame
            let hs = pick_households(seed, salt ^ 11, n_households, 3);
            let mut t = vec![
                Touched { who: Who::Cast(member_of(households, hs[0], seed, salt ^ 21)),
                    part: "burned out" },
                Touched { who: Who::Cast(member_of(households, hs[1], seed, salt ^ 22)),
                    part: "watched it burn" },
            ];
            if d(30) < 0.7 {
                let blamed = if d(31) < 0.5 && hs.len() > 2 {
                    Who::Cast(member_of(households, hs[2], seed, salt ^ 23))
                } else {
                    Who::Ephemeral("the peddler they blamed")
                };
                t.push(Touched { who: blamed, part: "blamed" });
            }
            t
        }
        EventKind::DeathDuty => vec![
            Touched { who: Who::Cast(member_of(households,
                pick_households(seed, salt ^ 11, n_households, 1)[0],
                seed, salt ^ 21)), part: "paid the death duty" },
            Touched { who: Who::Ephemeral("the father they buried"), part: "died" },
        ],
        EventKind::Fever => {
            // one parish burying: several households lose someone at once
            let n = 2 + (d(1) * 3.0) as usize;
            pick_households(seed, salt ^ 11, n_households, n)
                .into_iter().enumerate().flat_map(|(k, h)| [
                    Touched { who: Who::Cast(member_of(households, h,
                        seed, salt ^ (20 + k as u64))), part: "buried their own" },
                    Touched { who: Who::Ephemeral(match d(50 + k as u64) {
                        x if x < 0.5 => "the little one",
                        _ => "the old mother",
                    }), part: "taken" },
                ]).collect()
        }
        EventKind::MillerDebt => vec![
            Touched { who: Who::Cast(member_of(households,
                pick_households(seed, salt ^ 11, n_households, 1)[0],
                seed, salt ^ 21)), part: "owes the miller" },
        ],
        EventKind::BrokenBetrothal => {
            // the couple and both kin: the one who broke it, the one
            // who waited, and a cousin who still takes a side
            let hs = pick_households(seed, salt ^ 11, n_households, 3);
            let mut t = vec![
                Touched { who: Who::Cast(member_of(households, hs[0], seed, salt ^ 21)),
                    part: "broke the troth" },
                Touched { who: Who::Cast(member_of(households, hs[1], seed, salt ^ 22)),
                    part: "was promised" },
            ];
            if hs.len() > 2 && d(30) < 0.6 {
                t.push(Touched { who: Who::Cast(member_of(households, hs[2],
                    seed, salt ^ 23)), part: "took a side" });
            }
            t
        }
        EventKind::OldFaith => vec![
            Touched { who: Who::Cast(member_of(households,
                pick_households(seed, salt ^ 11, n_households, 1)[0],
                seed, salt ^ 21)), part: if d(30) < 0.5 {
                    "still keeps the stones" } else { "was made to spit on them" } },
        ],
        EventKind::Eviction => {
            // a household put out, and the neighbors who took the strip
            let hs = pick_households(seed, salt ^ 11, n_households, 2);
            let mut t = vec![
                Touched { who: Who::Cast(member_of(households, hs[0], seed, salt ^ 21)),
                    part: "put out for arrears" },
            ];
            if hs.len() > 1 {
                t.push(Touched { who: Who::Cast(member_of(households, hs[1],
                    seed, salt ^ 22)), part: "took the strip" });
            }
            t
        }
    };
    PrehistoryEvent { kind, years_ago: year, touched }
}

/// Walk the thirty years once per world seed: each kind fires at its
/// rate off its own stream, picks whom it touched, and the log is
/// kept oldest-first. `cast` is the founding cast's size - later
/// arrivals carry no story here. `kids` names which slots are the
/// children, so the deal can raise them under one roof
pub fn run(world_seed: u64, cast: usize, kids: &[usize]) -> Prehistory {
    let seed = world_seed ^ 0x7E1A_9017;
    let households = deal_households(seed, cast, kids);
    let n_households = households.iter().max().map(|m| m + 1).unwrap_or(0);
    let mut events = vec![];
    for (ki, kind) in KINDS.into_iter().enumerate() {
        let salt = (ki as u64 + 1).wrapping_mul(0x2545_F491_4F7C_DD1D);
        let gap = mean_gap(kind) as u64;
        // the first occurrence lands seeded inside the first gap
        let mut year = 1 + (draw(seed ^ salt, 0) * gap as f32) as u64;
        while year <= YEARS as u64 {
            events.push(record(kind, year as u8, seed, salt,
                &households, n_households));
            // next occurrence: the gap plus or minus a third of it
            let jitter = ((draw(seed ^ salt, year) - 0.5) * (gap as f32) * 0.66) as i64;
            year = ((year as i64 + gap as i64 + jitter).max(year as i64 + 1)) as u64;
        }
    }
    events.sort_by_key(|e| e.years_ago);
    Prehistory { events, households }
}

#[cfg(test)]
mod tests {
    use super::*;
    const KIDS: [usize; 3] = [26, 27, 28];

    #[test]
    fn the_same_seed_writes_the_same_years() {
        let a = run(42, 31, &KIDS);
        let b = run(42, 31, &KIDS);
        assert_eq!(a.events.len(), b.events.len());
        for (x, y) in a.events.iter().zip(b.events.iter()) {
            assert_eq!((x.kind, x.years_ago), (y.kind, y.years_ago));
            assert_eq!(x.touched, y.touched);
        }
        assert_eq!(a.households, b.households);
    }

    #[test]
    fn different_seeds_write_different_years() {
        let a = run(42, 31, &KIDS);
        let b = run(77, 31, &KIDS);
        assert_ne!(
            a.events.iter().map(|e| (e.kind, e.years_ago)).collect::<Vec<_>>(),
            b.events.iter().map(|e| (e.kind, e.years_ago)).collect::<Vec<_>>());
    }

    #[test]
    fn the_record_stays_in_bounds() {
        for seed in [1, 42, 77, 1234, 99_999] {
            let ph = run(seed, 31, &KIDS);
            assert!(ph.households.iter().all(|&h| h < 31));
            for e in &ph.events {
                assert!(e.years_ago >= 1 && e.years_ago <= YEARS);
                for t in &e.touched {
                    if let Who::Cast(s) = t.who { assert!(s < 31); }
                }
            }
        }
    }

    #[test]
    fn half_the_events_reach_two_households() {
        // the done-when: this is what turns nine identical soldier
        // brothers into one levy with three fates
        for seed in [1, 42, 77, 1234, 99_999] {
            let ph = run(seed, 31, &KIDS);
            let multi = ph.events.iter()
                .filter(|e| e.households_touched(&ph) >= 2).count();
            assert!(multi * 2 >= ph.events.len(),
                "seed {seed}: {multi} of {} events reach two households",
                ph.events.len());
        }
    }

    #[test]
    fn a_story_only_cites_the_events_they_were_in() {
        let ph = run(42, 31, &KIDS);
        for slot in 0..31 {
            for id in ph.story_of(slot) {
                assert!(ph.events[id.0 as usize].touched.iter()
                    .any(|t| t.who == Who::Cast(slot)));
            }
        }
        // and every cast member was in something - the feast at least
        let ph = run(7, 31, &KIDS);
        for slot in 0..31 {
            assert!(!ph.story_of(slot).is_empty(), "slot {slot} has no story");
        }
    }

    #[test]
    fn the_table_writes_every_kind() {
        // across a handful of seeds all nine records show up somewhere
        let mut seen = [false; 9];
        for seed in [1, 42, 77, 1234] {
            for e in run(seed, 31, &KIDS).events {
                seen[KINDS.iter().position(|&k| k == e.kind).unwrap()] = true;
            }
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn the_children_share_a_roof() {
        // the deal raises the little ones together: at least two of
        // them land under one roof so siblings exist
        for seed in [1, 42, 77, 1234, 99_999] {
            let ph = run(seed, 31, &KIDS);
            let shared = KIDS.iter()
                .filter(|&&a| KIDS.iter().any(|&b| a != b
                    && ph.households[a] == ph.households[b]))
                .count();
            assert!(shared >= 2, "seed {seed}: no two children share a roof");
        }
    }

    #[test]
    fn a_households_children_share_its_elders() {
        let ph = run(42, 31, &KIDS);
        let plan = ph.kin_plan(42, &KIDS);
        for (s, k) in plan.iter().enumerate() {
            // parents and siblings stay inside the slot's own household
            for &p in &k.parents { assert_eq!(ph.households[p], ph.households[s]); }
            for &b in &k.siblings { assert_eq!(ph.households[b], ph.households[s]); }
            // parents are never children, siblings are
            for &p in &k.parents { assert!(!KIDS.contains(&p)); }
            for &b in &k.siblings { assert!(KIDS.contains(&b)); }
            // the lists don't overlap and no one is their own kin
            assert!(!k.parents.contains(&s) && !k.siblings.contains(&s)
                && !k.housemates.contains(&s));
        }
        // two children under one roof get the same parents: siblings
        let (a, b) = (KIDS[0], KIDS[1]);
        if ph.households[a] == ph.households[b] {
            assert_eq!(plan[a].parents, plan[b].parents);
            assert!(plan[a].siblings.contains(&b));
        }
    }

    #[test]
    fn a_child_without_elders_is_an_orphan() {
        // a kid dealt their own household keeps no parents - the plan
        // stays honest rather than inventing a family
        let ph = run(3, 8, &[0, 1, 2]);
        let plan = ph.kin_plan(3, &[0, 1, 2]);
        for (s, k) in plan.iter().enumerate() {
            let elders = members(&ph.households, ph.households[s]).iter()
                .filter(|m| ![0, 1, 2].contains(m)).count();
            if elders == 0 { assert!(k.parents.is_empty()); }
        }
    }
}

