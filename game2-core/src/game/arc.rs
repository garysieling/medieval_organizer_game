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

use crate::engine::det::HashSet;
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
    let you = you.clone();
    // the village's memory of what's said (E0-4): every template the
    // arc's spent stays spent - the repetition metric measures what's
    // left to say
    let mut recent: HashSet<&'static str> = HashSet::default();
    let mut out = Vec::with_capacity(talks);
    for i in 0..talks {
        let tseed = seed.wrapping_mul(31).wrapping_add(i as u64);
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
        recent.extend(tr.turns.iter().filter_map(|t| t.line));
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
            let heard: HashSet<&'static str> = blank.turns.iter()
                .filter_map(|t| t.line).collect();
            let again = converse_with_at(&them0, &you, Relation::Acquaintance, None,
                &lib, Some(&heard), 1, &STREET, &mut |_| Some(0), &mut Rng::new(t.seed));
            if again.lines() != blank.lines() { moved += 1; }
        }
        assert!(moved > 0, "the spent lines never stayed spent");
    }
}
