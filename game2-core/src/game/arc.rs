//! The arc scale (E0-4, #1405): one villager, twenty talks, a scripted
//! player policy - the harness Appendix A measures an arc on: deepening,
//! repetition across talks, false callbacks, and whether pressing every
//! beat pays off the way responding does.
//!
//! Between talks the runner does the session's bookkeeping at lab scale:
//! the said templates join the village's memory (`recent`), and the
//! villager's `Open` carries what the arc has earned - talks counted,
//! mood from the last vibe, disclosures banked, and the trust parts
//! nudged by how the picks landed. That's what the metrics read off: a
//! greedy arc that stalls next to a responsive one that opens.
//!
//! Determinism is the done-when: everything draws on the run's seed -
//! the villager, the per-talk seeds, the policy's picks - so a seed and
//! a policy give one byte-identical transcript of the whole arc.

use crate::game::conversation::{
    converse_with_at, Act, Choice, DialogueTree, Open, Relation, Respond,
    Rng, Speaker, Transcript, Vibe,
};
use crate::game::relationships::RelKind;
use crate::game::setting::Setting;

/// A scripted player policy (E0-4): what the bot does when the walk
/// stops for a pick. Four, per Appendix A
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Asks every topic it can
    Curious,
    /// Turns toward bids, reflects, shares back
    Responsive,
    /// Pivots to an ask after every disclosure
    Greedy,
    /// Never responds beyond the minimum
    Silent,
}

/// The four policies, for `--policy` and the JSON record
pub const POLICIES: [Policy; 4] =
    [Policy::Curious, Policy::Responsive, Policy::Greedy, Policy::Silent];

impl Policy {
    pub fn name(self) -> &'static str {
        match self {
            Policy::Curious => "curious",
            Policy::Responsive => "responsive",
            Policy::Greedy => "greedy",
            Policy::Silent => "silent",
        }
    }
}

pub fn policy_of(s: &str) -> Option<Policy> {
    POLICIES.iter().copied().find(|p| p.name() == s)
}

/// One talk in the arc: the transcript, what the policy picked, and the
/// openness the villager carried out of it
pub struct ArcTalk {
    pub index: usize,
    pub seed: u64,
    pub transcript: Transcript,
    /// The option indices the policy took, in order
    pub picks: Vec<usize>,
    /// What those indices were - (act, how it lands, the line they'd
    /// say): the record the metrics read manipulation off
    pub choices_made: Vec<(Act, Respond, String)>,
    /// The villager's `Open` after this talk's bookkeeping
    pub open_after: Open,
}

/// Which of the offered choices the policy takes: the pick is authored
/// order on ties so a policy is one deterministic function of the slate
fn pick(policy: Policy, ch: &[Choice]) -> Option<usize> {
    if ch.is_empty() { return None; }
    match policy {
        // asks every topic it can: a question fork first, else the lead
        Policy::Curious => Some(ch.iter().position(|c| c.act == Act::Question).unwrap_or(0)),
        // pivots to an ask after every disclosure: the PivotAsk tag is
        // the manipulation itself, whatever act carries it
        Policy::Greedy => Some(
            ch.iter().position(|c| c.respond == Respond::PivotAsk)
                .or_else(|| ch.iter().position(|c| c.act == Act::Question))
                .unwrap_or(0)),
        // turns toward bids, reflects, shares back: the warm responds
        // first, a pivot or a step-around last
        Policy::Responsive => {
            let score = |r: Respond| match r {
                Respond::Care => 4,
                Respond::Reflect => 3,
                Respond::Validate | Respond::Advise => 2,
                Respond::Deflect => 0,
                Respond::PivotAsk => -1,
            };
            Some((0..ch.len()).max_by_key(|&i| score(ch[i].respond)).unwrap_or(0))
        }
        // never beyond the minimum: the shortest thing on offer
        Policy::Silent => Some((0..ch.len()).min_by_key(|&i| ch[i].preview.len()).unwrap_or(0)),
    }
}

/// The session's after-talk bookkeeping, at lab scale: the bond counts
/// the talk, last talk's feel becomes next talk's mood, disclosures
/// bank, and how the picks landed nudges the trust parts - a press
/// (PivotAsk) bleeds a little safety every time while being met warms
/// it, which is the transactional penalty the metrics compare against.
/// The nudges are proportional to what's left (`+= rate * (1 - v)`),
/// so a 20-talk arc approaches a part instead of pinning at it
fn settle(them: &mut Speaker, tr: &Transcript, made: &[(Act, Respond, String)]) {
    let o = &mut them.open;
    o.talks += 1;
    o.mood = match tr.vibe {
        Some(Vibe::Safe) => o.mood + 0.2 * (1.0 - o.mood),
        Some(Vibe::Mismatch) => o.mood - 0.2 * (1.0 + o.mood),
        None => o.mood * 0.8,
    };
    o.shared = o.shared.max(tr.shared);
    for &(_, r, _) in made {
        match r {
            Respond::Care => {
                o.benevolence += 0.04 * (1.0 - o.benevolence);
                o.safety += 0.02 * (1.0 - o.safety);
            }
            Respond::Validate => {
                o.benevolence += 0.03 * (1.0 - o.benevolence);
                o.safety += 0.01 * (1.0 - o.safety);
            }
            Respond::Reflect => {
                o.intimacy += 0.04 * (1.0 - o.intimacy);
                o.safety += 0.01 * (1.0 - o.safety);
            }
            Respond::Advise => o.integrity += 0.04 * (1.0 - o.integrity),
            // the press itself reads unsafe: each ask where warmth was
            // due bleeds a share of what's been built
            Respond::PivotAsk => o.safety *= 0.95,
            Respond::Deflect => {
                o.benevolence *= 0.95;
                o.safety *= 0.97;
            }
        }
    }
    // a talk that reaches a farewell warms the parts a little regardless
    if matches!(tr.ending, Some(crate::game::conversation::Ending::Farewell)) {
        o.intimacy += 0.02 * (1.0 - o.intimacy);
    }
}

/// Run `talks` talks between `them` and the policy bot `you`, at `rel`
/// in `setting`, all seeded off `seed`. Returns the villager as the arc
/// left them (the openness they carry out) and each talk's record.
///
/// The bot answers through `converse_with_at`'s pick point: the walk
/// stops, the policy reads the slate, and the choice it makes is what
/// `choices_made` records - the same mechanism `talk_lab talk` uses for
/// a human's picks.
pub fn run_arc(
    them: &Speaker,
    you: &Speaker,
    rel: Relation,
    kind: Option<RelKind>,
    talks: usize,
    policy: Policy,
    library: &[DialogueTree],
    setting: &Setting,
    seed: u64,
) -> (Speaker, Vec<ArcTalk>) {
    let mut them = them.clone();
    let mut you = you.clone();
    // the village's memory of what's said (E0-4): a spent template
    // stays spent - with a decay window, the same span the session's
    // `said_lines` holds (FRESH_LINE_DAYS). The arc's days run about
    // two talks apiece, so the window spans the last six talks'
    // lines (#1690): a saying returns once its days are out
    const FRESH_TALKS: usize = 6;
    let mut said_at: crate::engine::det::HashMap<&'static str, u64> =
        crate::engine::det::HashMap::default();
    let mut out = Vec::with_capacity(talks);
    for i in 0..talks {
        let tseed = seed.wrapping_mul(31).wrapping_add(i as u64);
        let recent: crate::engine::det::HashMap<&'static str, u64> = said_at.iter()
            .filter(|&(_, &at)| (i as u64) - at < FRESH_TALKS as u64)
            .map(|(&l, &at)| (l, at)).collect();
        let mut picks: Vec<usize> = vec![];
        let mut made: Vec<(Act, Respond, String)> = vec![];
        let tr = converse_with_at(&them, &you, rel, kind, library,
            Some(&recent), 1, setting, &mut |ch: &[Choice]| {
                pick(policy, ch).map(|p| {
                    picks.push(p);
                    made.push((ch[p].act, ch[p].respond, ch[p].preview.clone()));
                    p
                })
            }, &mut Rng::new(tseed));
        for l in tr.turns.iter().filter_map(|t| t.line) {
            said_at.insert(l, i as u64);
        }
        // the arc's one slip (E5-8, #1439): the engine reports what
        // fired, the caller spends the pair's budget on it - once to
        // a relationship's whole run of talks
        for &(i, _) in &tr.slips {
            let sp = if i == 0 { &mut them } else { &mut you };
            sp.slips_left = sp.slips_left.saturating_sub(1);
        }
        settle(&mut them, &tr, &made);
        out.push(ArcTalk { index: i, seed: tseed, transcript: tr,
            picks, choices_made: made, open_after: them.open.clone() });
    }
    (them, out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::dialogue_trees::village_library;
    use crate::game::setting::STREET;

    fn pair() -> (Speaker, Speaker) {
        let mut them = Speaker::new("Hob");
        them.trade = Some("carter");
        (them, Speaker::new("Wanderer"))
    }

    /// the done-when (E0-4): a seed and a policy give one arc - the same
    /// inputs replay to byte-identical output
    #[test]
    fn a_seed_and_a_policy_give_one_arc() {
        let (them, you) = pair();
        let lib = village_library();
        let render = |arc: &[ArcTalk]| serde_json::to_string(
            &arc.iter().map(|t| serde_json::json!({
                "lines": t.transcript.lines(),
                "picks": t.picks,
                "ending": format!("{:?}", t.transcript.ending),
                "open": format!("{:?}", t.open_after),
            })).collect::<Vec<_>>()).unwrap();
        let a = run_arc(&them, &you, Relation::Acquaintance, None, 20,
            Policy::Curious, &lib, &STREET, 7);
        let b = run_arc(&them, &you, Relation::Acquaintance, None, 20,
            Policy::Curious, &lib, &STREET, 7);
        assert_eq!(render(&a.1), render(&b.1));
    }

    /// the policies are different instruments (E0-4): the greedy arc
    /// presses every beat into an ask where the responsive one meets
    /// what's offered - the comparison the manipulation check needs
    #[test]
    fn the_policies_diverge() {
        let (them, you) = pair();
        let lib = village_library();
        let asks = |arc: &[ArcTalk]| arc.iter()
            .flat_map(|t| t.choices_made.iter())
            .filter(|(_, r, _)| *r == Respond::PivotAsk).count();
        let warm = |arc: &[ArcTalk]| arc.iter()
            .flat_map(|t| t.choices_made.iter())
            .filter(|(_, r, _)| matches!(r, Respond::Care | Respond::Reflect)).count();
        let greedy = run_arc(&them, &you, Relation::Acquaintance, None, 20,
            Policy::Greedy, &lib, &STREET, 7).1;
        let responsive = run_arc(&them, &you, Relation::Acquaintance, None, 20,
            Policy::Responsive, &lib, &STREET, 7).1;
        assert!(asks(&greedy) > asks(&responsive),
            "greedy {} vs responsive {}", asks(&greedy), asks(&responsive));
        assert!(warm(&responsive) >= warm(&greedy));
    }

    /// the arc's bookkeeping carries between talks (E0-4): the village
    /// remembers what's been said and the bond counts its talks
    #[test]
    fn the_arc_carries_between_talks() {
        let (them0, you) = pair();
        let lib = village_library();
        let (them, arc) = run_arc(&them0, &you, Relation::Acquaintance, None, 20,
            Policy::Responsive, &lib, &STREET, 7);
        assert_eq!(arc.len(), 20);
        assert_eq!(them.open.talks, 20, "the bond counted every talk");
        // and the village's memory carries: replay a talk's own seed
        // with its spent lines held recent - wherever a re-drawn line
        // has a fresher sibling the walk speaks differently. (Where a
        // pool's drained the talk repeats rather than starve, so the
        // probe runs a handful of seeds)
        let mut moved = 0;
        for t in arc.iter().take(6) {
            let blank = converse_with_at(&them0, &you, Relation::Acquaintance, None,
                &lib, None, 1, &STREET, &mut |_| Some(0), &mut Rng::new(t.seed));
            let heard: crate::engine::det::HashMap<&'static str, u64> = blank.turns.iter()
                .filter_map(|t| t.line).map(|l| (l, 1)).collect();
            let again = converse_with_at(&them0, &you, Relation::Acquaintance, None,
                &lib, Some(&heard), 1, &STREET, &mut |_| Some(0), &mut Rng::new(t.seed));
            if again.lines() != blank.lines() { moved += 1; }
        }
        assert!(moved > 0, "the spent lines never stayed spent");
    }

    /// the done-when (E5-8, #1439): at most one slip in a
    /// relationship's whole run of talks - the pair's budget spends
    /// on the first leak and a prone speaker's arc never shows two
    #[test]
    fn an_arc_never_shows_two_slips() {
        let (_, you) = pair();
        let lib = village_library();
        let mut saw = 0;
        for seed in 0..12 {
            let mut them = Speaker::new("Hob");
            them.trade = Some("carter");
            // a prone soul with the arc's budget still unspent
            them.slip = Some(crate::game::conversation::Slip {
                prone: 1.0,
                dead: Some(crate::game::conversation::Dead {
                    kin: "brother", name: "Corvin".into(), trade: "carter" }) });
            them.slips_left = 1;
            let (spent, arc) = run_arc(&them, &you, Relation::Acquaintance, None, 20,
                Policy::Responsive, &lib, &STREET, seed);
            let slips = arc.iter()
                .map(|t| t.transcript.slips.len()).sum::<usize>();
            assert!(slips <= 1, "two slips in one arc: {}", slips);
            let tagged = arc.iter().flat_map(|t| t.transcript.turns.iter())
                .filter(|t| t.act == crate::game::conversation::Act::Slip).count();
            assert_eq!(slips, tagged, "the record counts what the lines show");
            if slips == 1 {
                saw += 1;
                assert_eq!(spent.slips_left, 0, "the leak spent the budget");
            }
        }
        assert!(saw > 0, "a prone ghost never leaked in 12 arcs");
    }

    /// the landmark keeps its freshness (#1690): a whole-line habit
    /// airs a fresh variant while one remains and goes quiet when the
    /// pool's spent - it can't open every talk of an arc with the same
    /// saying, and nothing the villager speaks airs in back-to-back
    /// talks (the parting rite and stock courtesies excepted - they're
    /// liturgy, not speech)
    #[test]
    fn a_signature_line_stays_spent_between_talks() {
        let (mut them, you) = pair();
        them.idiolect = Some(crate::game::idiolect::Idiolect {
            phrase: "as the mill grinds",
            habit: crate::game::idiolect::Habit::ProverbPatience });
        let habit: crate::engine::det::HashSet<&'static str> =
            crate::game::idiolect::Habit::ProverbPatience.lines()
                .iter().copied().collect();
        let lib = village_library();
        for &seed in &[3u64, 42, 99] {
            for &policy in &[Policy::Curious, Policy::Greedy, Policy::Responsive] {
                let (_them, arc) = run_arc(&them, &you, Relation::Acquaintance, None,
                    20, policy, &lib, &STREET, seed);
                let mut last_habit: Option<&'static str> = None;
                // template -> (last talk it aired, consecutive run)
                let mut runs: crate::engine::det::HashMap<&'static str, (usize, usize)> =
                    crate::engine::det::HashMap::default();
                for (i, talk) in arc.iter().enumerate() {
                    let tpls: crate::engine::det::HashSet<&'static str> = talk.transcript.turns.iter()
                        .filter(|t| t.speaker == 0)
                        .filter_map(|t| t.line).collect();
                    let aired: Vec<&'static str> =
                        habit.intersection(&tpls).copied().collect();
                    assert!(aired.len() <= 1,
                        "two proverbs in one talk ({i})");
                    if let Some(&v) = aired.first() {
                        assert!(Some(&v) != last_habit.as_ref(),
                            "the same proverb {v} aired in back-to-back \
                             talks ({i}; seed {seed} {policy:?})");
                    }
                    last_habit = aired.first().copied();
                    for l in tpls.iter().copied()
                        .filter(|l| !crate::game::dialogue_trees::STOCK_LINES.contains(l)
                            && !crate::game::conversation::PART_LINES.contains(l)) {
                        let (last, run) = runs.get(&l).copied().unwrap_or((usize::MAX, 0));
                        let run = if last == i.wrapping_sub(1) { run + 1 } else { 1 };
                        assert!(run <= 2,
                            "a line aired {run} talks running (talk {i}): {l}");
                        runs.insert(l, (i, run));
                    }
                }
            }
        }
    }
}
