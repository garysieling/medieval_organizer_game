//! The synthetic-talk recipe behind `dialogue_gen` (#1402/E0-3): a
//! seeded roster of plausible villagers, seeded pairings across the
//! relations, run through `converse` - shared so the `--json` corpus
//! and the talk checks generate the same talks.

use crate::game::conversation::{converse_at, DialogueTree, Relation, Rng, Speaker, Transcript};
use crate::game::dialogue_trees::village_library;
use crate::game::relationships::RelKind;
use crate::game::traits::Trait;
use crate::session::NAMES;

pub const IDEOLOGIES: [&str; 4] = ["laborer", "merchant", "clergy", "lord"];
pub const STATUSES: [&str; 5] = ["well", "sick", "hungry", "grieving", "drunk"];
pub const ALL_TRAITS: [Trait; 8] = [
    Trait::Lazy,
    Trait::Ambitious,
    Trait::Kind,
    Trait::Stingy,
    Trait::Pious,
    Trait::Hotheaded,
    Trait::Gregarious,
    Trait::Proud,
];
pub const RELATIONS: [Relation; 5] = [
    Relation::Stranger,
    Relation::Acquaintance,
    Relation::Friend,
    Relation::Kin,
    Relation::Rival,
];

/// A plausible fine bond for the coarse relation the draw picked (#1372):
/// kin reads family or lover, a friend may stand as a mentor, an
/// acquaintance may be an apprentice - so the kind-keyed material shows
pub fn kind_for(rng: &mut Rng, relation: Relation) -> Option<RelKind> {
    Some(match relation {
        Relation::Kin => if rng.below(2) == 0 { RelKind::Family } else { RelKind::Lover },
        Relation::Friend => if rng.below(3) == 0 { RelKind::Mentor } else { RelKind::Friend },
        Relation::Acquaintance => [RelKind::Acquaintance, RelKind::Mentor, RelKind::Apprentice][rng.below(3)],
        Relation::Rival => RelKind::Rival,
        Relation::Stranger => RelKind::Stranger,
    })
}

/// One generated conversation: the pair's roster seats, who they
/// stood as, and the talk
pub struct GenTalk {
    pub a: usize,
    pub b: usize,
    pub relation: Relation,
    pub kind: Option<RelKind>,
    /// Where the talk stood (E5-9, #1446): the corpus seeds a spread of
    /// places so the same speaker is heard in both transcripts
    pub setting: crate::game::setting::Setting,
    pub transcript: Transcript,
}

/// A seeded batch: the roster and every talk drawn across it
pub struct GenCorpus {
    pub roster: Vec<Speaker>,
    pub talks: Vec<GenTalk>,
}

pub fn generate_roster(n: usize, rng: &mut Rng) -> Vec<Speaker> {
    let mut roster = Vec::with_capacity(n);
    // one sky for the batch, like a real day's (#1388): weather-gated
    // lines show in the generated corpus, keyed on the wx-* statuses
    // speaker_of pushes in-game
    let wx = ["wx-rain", "wx-snow", "wx-fog", "wx-fair"][rng.below(4)];
    for i in 0..n {
        let name = NAMES[i % NAMES.len()];
        let ideology = IDEOLOGIES[rng.below(IDEOLOGIES.len())];

        // 1 to 3 distinct traits
        let num_traits = 1 + rng.below(3);
        let mut traits = Vec::new();
        let mut trait_indices: Vec<usize> = (0..ALL_TRAITS.len()).collect();
        for _ in 0..num_traits {
            let pick = rng.below(trait_indices.len());
            let idx = trait_indices.remove(pick);
            traits.push(ALL_TRAITS[idx]);
        }

        // Status
        let status_pick = STATUSES[rng.below(STATUSES.len())];
        let mut status = Vec::new();
        status.push(wx);
        if status_pick != "well" {
            status.push(status_pick);
        }

        roster.push(Speaker {
            name: name.to_string(),
            facts: crate::game::conversation::lite_facts(&traits, i as u64),
            traits,
            ideology,
            status,
            trade: None,
            pull: 0.8,
            sex: None,
            // a spread of bonds so the corpus walks the openness tiers
            // (E3-2): strangers to warm friends, a few within earshot
            open: crate::game::conversation::Open {
                intimacy: rng.below(900) as f32 / 1000.0,
                benevolence: rng.below(900) as f32 / 1000.0,
                integrity: rng.below(900) as f32 / 1000.0,
                safety: rng.below(1000) as f32 / 1000.0,
                mood: (rng.below(1000) as f32 / 500.0) - 1.0,
                talks: rng.below(4) as u32,
                earshot: rng.below(4) == 0,
                shared: rng.below(3) as u8,
            },
            tv: crate::game::text::Tv::You,
            archaism: 1,
            rel: Relation::Stranger,
            worry: None,
            slip: None,
            slips_left: 0,
            joke: None,
            story: None,
            // a spread of habits so the corpus hears the idiolect pass
            // (E4-7, #1416): roster index deals each speaker a distinct
            // phrase and habit without replacement, like the cast deal
            idiolect: Some(crate::game::idiolect::Idiolect {
                phrase: crate::game::idiolect::PHRASES[i % crate::game::idiolect::PHRASES.len()],
                habit: crate::game::idiolect::HABITS[(i * 7 + 3) % crate::game::idiolect::HABITS.len()],
            }),
        });
    }
    roster
}

/// The seeded corpus: `n` characters, `m` conversations each with a
/// seeded distinct partner - the same talks `--json` serializes
pub fn gen_conversations(num_characters: usize, num_conversations: usize, seed: u64) -> GenCorpus {
    let mut rng = Rng::new(seed);
    let library = village_library();
    let roster = generate_roster(num_characters, &mut rng);
    let mut talks = vec![];
    for (char_idx, char_a) in roster.iter().enumerate() {
        for _ in 0..num_conversations {
            let other_offset = 1 + rng.below(num_characters - 1);
            let b = (char_idx + other_offset) % num_characters;
            let relation = RELATIONS[rng.below(RELATIONS.len())];
            let kind = kind_for(&mut rng, relation);
            // E5-9 (#1446): a spread of places - the street and market
            // for the public transcript, the alehouse, kitchen and
            // field for the hidden one (the bond decides if it opens),
            // and the law's ear on a few backstage talks to flip them
            // public again
            let setting = spread_setting(&mut rng);
            let t = converse_at(char_a, &roster[b], relation, kind, &library, None, &setting, &mut rng);
            talks.push(GenTalk { a: char_idx, b, relation, kind, setting, transcript: t });
        }
    }
    GenCorpus { roster, talks }
}

/// The places a batch spreads over (E5-9, #1446) - shared by the
/// flat and the stratified draws
fn spread_setting(rng: &mut Rng) -> crate::game::setting::Setting {
    crate::game::setting::Setting {
        place: match rng.below(10) {
            0 | 1 => crate::game::setting::Place::Market,
            2 => crate::game::setting::Place::ChurchDoor,
            3 | 4 | 5 => crate::game::setting::Place::Alehouse,
            6 | 7 => crate::game::setting::Place::Kitchen,
            _ => crate::game::setting::Place::Field,
        },
        law_earshot: rng.below(8) == 0,
        invited: false,
    }
}

/// A stratified batch (E0-8, #1410): every tree's line count, which
/// never reached the target, and the conversations that added
/// coverage in order drawn
pub struct Coverage {
    /// turns counted per tree, in library order
    pub counts: Vec<(&'static str, usize)>,
    /// trees that never reached the target inside the cap
    pub uncovered: Vec<&'static str>,
    /// the conversations that added coverage, in order
    pub talks: Vec<GenTalk>,
    /// pairs rolled, kept or not
    pub attempts: usize,
}

/// Branch-stratified sampling: keep rolling pairs until each tree's
/// lines hit `target`; a dead branch (criteria unreachable by any
/// speaker) is reported uncovered rather than looping forever
pub fn cover_branches(roster: &[Speaker], target: usize, max_attempts: usize,
    library: &[DialogueTree], rng: &mut Rng) -> Coverage {
    assert!(roster.len() >= 2, "a pair needs two speakers");
    let mut counts: Vec<(&'static str, usize)> =
        library.iter().map(|t| (t.name, 0usize)).collect();
    let mut talks = vec![];
    let mut attempts = 0;
    while attempts < max_attempts && counts.iter().any(|&(_, c)| c < target) {
        attempts += 1;
        let a = rng.below(roster.len());
        let b = (a + 1 + rng.below(roster.len() - 1)) % roster.len();
        let relation = RELATIONS[rng.below(RELATIONS.len())];
        let kind = kind_for(rng, relation);
        let setting = spread_setting(rng);
        let t = converse_at(&roster[a], &roster[b], relation, kind,
            library, None, &setting, rng);
        let mut added = false;
        for turn in &t.turns {
            if let Some(c) = counts.iter_mut().find(|(n, _)| *n == turn.tree) {
                c.1 += 1;
                if c.1 <= target { added = true; }
            }
        }
        if added { talks.push(GenTalk { a, b, relation, kind, setting, transcript: t }); }
    }
    let uncovered = counts.iter().filter(|&&(_, c)| c < target)
        .map(|&(n, _)| n).collect();
    Coverage { counts, uncovered, talks, attempts }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::conversation::{Act, Criterion, Ending, Exchange, Reply};

    static HELLO: Exchange = Exchange {
        act: Act::Greet, concepts: &["x"], calls: &["Morrow."], voice: &[],
        replies: &[Reply {
            criteria: &[], weight: 1.0, bias: &[], act: Act::Farewell,
            respond: None, concepts: &[], lines: &["Off I go."], voice: &[],
            then: None, end: Some(Ending::LeavesEarly) }],
    };

    /// The stratified draw fills a reachable branch to the target and
    /// keeps the conversations that added coverage
    #[test]
    fn a_reachable_branch_fills_to_target() {
        let mut rng = Rng::new(7);
        let roster = generate_roster(4, &mut rng);
        let lib = vec![DialogueTree { name: "always", criteria: &[],
            root: &HELLO, rejoin: None }];
        let cov = cover_branches(&roster, 5, 500, &lib, &mut rng);
        assert!(cov.counts[0].1 >= 5, "{:?}", cov.counts);
        assert!(cov.uncovered.is_empty());
        assert!(!cov.talks.is_empty());
        assert!(cov.attempts < 500);
    }

    /// A branch no speaker can reach is reported uncovered at the cap,
    /// not hung on (E0-8, #1410)
    #[test]
    fn a_dead_branch_reports_uncovered_and_stops() {
        let mut rng = Rng::new(7);
        let roster = generate_roster(4, &mut rng);
        let lib = vec![
            DialogueTree { name: "dead",
                criteria: &[Criterion::Has("self:status:never_there")],
                root: &HELLO, rejoin: None },
            DialogueTree { name: "always", criteria: &[],
                root: &HELLO, rejoin: None }];
        let cov = cover_branches(&roster, 5, 200, &lib, &mut rng);
        assert_eq!(cov.uncovered, vec!["dead"]);
        assert_eq!(cov.attempts, 200);
        assert!(cov.counts.iter().find(|(n, _)| *n == "always").unwrap().1 >= 5);
    }

    /// Kept conversations are the coverage contributors - a talk that
    /// only repeats already-full branches isn't kept
    #[test]
    fn only_coverage_adding_talks_are_kept() {
        let mut rng = Rng::new(7);
        let roster = generate_roster(4, &mut rng);
        let lib = vec![DialogueTree { name: "always", criteria: &[],
            root: &HELLO, rejoin: None }];
        // target 2 turns: each talk says two "always" lines - the
        // second talk is the last that adds anything
        let cov = cover_branches(&roster, 2, 500, &lib, &mut rng);
        assert_eq!(cov.talks.len(), 1);
        assert!(cov.attempts >= 1);
    }
}
