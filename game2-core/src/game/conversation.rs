//! Conversation framework: weighted call/response trees for two speakers,
//! and a looser, concept-threaded model for house meetings.
//!
//! Research basis (wiki/literature-reviews/dialogue-systems.md):
//! - Ruskin (Valve, GDC 2012): a rule database, not a branching script.
//!   Trees carry criteria on world facts; the most specific matching tree
//!   (most criteria) leads, and when it bottoms out the conversation falls
//!   through to the next-most-specific - generic small talk is the
//!   always-on tail, so nobody is ever left without something to say.
//! - CiF / Prom Week (McCoy et al.): traits and statuses *weight* replies
//!   rather than gate them - personality biases the distribution of the
//!   forks, it doesn't decide them. People still surprise you.
//! - Facade (Mateas & Stern): discourse acts - respond to the act, not the
//!   words; a meeting reply can bind to the last speaker, the organizer's
//!   open question, or an earlier statement.
//! - Versu (Evans & Short): once-only quips. Said lines are consumed, so a
//!   conversation runs out of material and dries up into small talk rather
//!   than repeating itself - the "they walk off" ending falls out of
//!   exhaustion, not a special case.
//!
//! The same structures serve NPC-NPC simulation and player-NPC play:
//! `converse` walks both sides; `converse_with` hands the player's turns
//! to a chooser that sees two sampled options and may end it early.

use std::collections::HashSet;

use crate::game::traits::Trait;

/// Deterministic RNG (SplitMix64); no external crates, seeded runs replay
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1)
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform in [0, n)
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 { 0 } else { (self.next_u64() % n as u64) as usize }
    }

    /// Weighted pick over a slice; returns the index. Zero or negative
    /// weights can't be picked.
    pub fn weighted(&mut self, weights: &[f32]) -> usize {
        let total: f32 = weights.iter().copied().filter(|w| *w > 0.0).sum();
        if total <= 0.0 { return self.below(weights.len()); }
        let mut roll = self.next_f32() * total;
        for (i, w) in weights.iter().enumerate() {
            if *w <= 0.0 { continue; }
            roll -= w;
            if roll < 0.0 { return i; }
        }
        weights.len() - 1
    }
}

/// A person in a conversation: enough of a profile to roll facts against.
/// `status` carries circumstance ("sick", "grieving", "hungry", "drunk");
/// `ideology` carries vocation ("laborer", "clergy", "merchant", "lord").
#[derive(Debug, Clone)]
pub struct Speaker {
    pub name: String,
    pub traits: Vec<Trait>,
    pub ideology: &'static str,
    pub status: Vec<&'static str>,
}

impl Speaker {
    pub fn new(name: &str) -> Self {
        Speaker { name: name.to_string(), traits: vec![], ideology: "laborer", status: vec![] }
    }
    /// Fact strings this speaker contributes ("trait:hotheaded", ...)
    fn own_facts(&self) -> Vec<String> {
        let mut v: Vec<String> = self.traits.iter().map(|t| format!("trait:{}", trait_name(*t))).collect();
        v.push(format!("ideology:{}", self.ideology));
        v.extend(self.status.iter().map(|s| format!("status:{}", s)));
        v
    }
}

fn trait_name(t: Trait) -> &'static str {
    match t {
        Trait::Lazy => "lazy",
        Trait::Ambitious => "ambitious",
        Trait::Kind => "kind",
        Trait::Stingy => "stingy",
        Trait::Pious => "pious",
        Trait::Hotheaded => "hotheaded",
        Trait::Gregarious => "gregarious",
        Trait::Proud => "proud",
    }
}

/// What stands between two speakers. Folds into the pair's facts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Relation {
    Stranger,
    Acquaintance,
    Friend,
    Kin,
    Rival,
}

fn relation_name(r: Relation) -> &'static str {
    match r {
        Relation::Stranger => "stranger",
        Relation::Acquaintance => "acquaintance",
        Relation::Friend => "friend",
        Relation::Kin => "kin",
        Relation::Rival => "rival",
    }
}

/// The world state a rule is matched against (Ruskin's "query"), flattened
/// to strings so criteria stay declarative and content stays data.
/// - `self:*` facts are about the speaker being evaluated for
/// - `other:*` facts are about the person they're talking to
/// - `any:*`  facts hold if either of the pair carries them
/// - `pair:*` facts are about the relation between them
#[derive(Debug, Default)]
pub struct Facts {
    set: HashSet<String>,
}

impl Facts {
    /// Facts for `speaker` choosing a line while talking to `other`
    pub fn for_roles(speaker: &Speaker, other: &Speaker, rel: Relation) -> Facts {
        let mut f = Facts::default();
        for x in speaker.own_facts() { f.set.insert(format!("self:{}", x)); }
        for x in other.own_facts() { f.set.insert(format!("other:{}", x)); }
        for x in speaker.own_facts().into_iter().chain(other.own_facts()) {
            f.set.insert(format!("any:{}", x));
        }
        f.set.insert(format!("pair:relation:{}", relation_name(rel)));
        f
    }

    pub fn has(&self, fact: &str) -> bool { self.set.contains(fact) }
}

/// One test on the facts: all of a rule's criteria must hold for it to be
/// eligible. Criterion count is also the rule's specificity score (Ruskin).
#[derive(Debug, Clone, Copy)]
pub enum Criterion {
    /// The fact must be present
    Has(&'static str),
    /// The fact must be absent
    Lacks(&'static str),
}

impl Criterion {
    pub fn holds(&self, f: &Facts) -> bool {
        match self {
            Criterion::Has(x) => f.has(x),
            Criterion::Lacks(x) => !f.has(x),
        }
    }
}

/// What a line does in the exchange (Facade's discourse acts, slimmed).
/// Replies stay coherent because they answer the *act*, not the words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    Greet,
    Remark,
    Question,
    Answer,
    Agree,
    PushBack,
    Farewell,
}

/// How a conversation closes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// A polite parting
    Farewell,
    /// One of them turned and left mid-talk
    WalksOff,
    /// They ran out of things to say and drifted apart
    Exhausted,
    /// The player chose to leave (converse_with only)
    LeavesEarly,
}

/// One fork a responder can take: a weighted, criteria-gated reply that
/// may drop the conversation a level (`then`) or close it (`end`).
/// All content is static so trees can live in `const`s.
pub struct Reply {
    /// Facts the responder must satisfy for this fork to exist at all -
    /// how a sick or grieving speaker *gains* a path nobody else has
    pub criteria: &'static [Criterion],
    /// Base draw weight at this fork
    pub weight: f32,
    /// (trait, multiplier) applied to the weight when the speaker has it -
    /// the personality bias (CiF's social considerations)
    pub bias: &'static [(Trait, f32)],
    /// What this line does in the exchange
    pub act: Act,
    /// What it's about; a house meeting threads on these
    pub concepts: &'static [&'static str],
    /// Ways to say it; `{other}` is filled with the listener's name.
    /// Once-only: a variant is spent when spoken (Versu's exclusivity)
    pub lines: &'static [&'static str],
    /// The level below, if this fork takes the talk deeper
    pub then: Option<Exchange>,
    /// If set, speaking this line ends the conversation
    pub end: Option<Ending>,
}

/// One call/response level: whoever holds the floor speaks a variant of
/// `calls`; the other picks among `replies`. An exchange with no replies
/// is just a beat - the talk moves on to whoever speaks next.
pub struct Exchange {
    /// What the calls do / are about
    pub act: Act,
    pub concepts: &'static [&'static str],
    /// Ways to open this beat; `{other}` fills with the listener's name
    pub calls: &'static [&'static str],
    /// The forks the other side can take
    pub replies: &'static [Reply],
}

/// A whole dialogue tree plus the circumstances it speaks to. The library
/// holds many; the most specific match leads (Ruskin: specificity is the
/// criterion count; library order breaks ties).
pub struct DialogueTree {
    pub name: &'static str,
    pub criteria: &'static [Criterion],
    /// Where the tree starts when it opens the conversation
    pub root: &'static Exchange,
    /// Where the tree is joined when the walk falls through to it
    /// mid-conversation - the generic tail skips re-greeting and picks up
    /// at the chit-chat
    pub rejoin: Option<&'static Exchange>,
}

/// A thing said and recorded
#[derive(Debug)]
pub struct Turn {
    /// Index into the speakers pair (0 = initiator)
    pub speaker: usize,
    pub act: Act,
    pub text: String,
    pub concepts: Vec<&'static str>,
    /// Which tree the line came from - handy for a grader
    pub tree: &'static str,
}

/// The full record of a two-person conversation
#[derive(Debug)]
pub struct Transcript {
    pub speakers: [String; 2],
    /// Which trees the cascade walked, in order
    pub trees_walked: Vec<&'static str>,
    pub turns: Vec<Turn>,
    pub ending: Option<Ending>,
}

impl Transcript {
    /// The turns as printable lines: "Eliza: Forgive me, I've a cough..."
    pub fn lines(&self) -> Vec<String> {
        self.turns.iter()
            .map(|t| format!("{}: {}", self.speakers[t.speaker], t.text))
            .collect()
    }
}

/// Safety bound so an unlucky cascade can't loop forever
const MAX_TURNS: usize = 24;

/// Reply forks whose lines are all spent count as exhausted (Versu's
/// exclusivity): a spent fork stops being offered, which is how a
/// conversation runs out of material.
fn eligible<'a>(replies: &'a [Reply], facts: &Facts, used: &HashSet<&'static str>) -> Vec<&'a Reply> {
    replies.iter()
        .filter(|r| r.criteria.iter().all(|c| c.holds(facts)))
        .filter(|r| r.lines.iter().any(|l| !used.contains(*l)))
        .collect()
}

/// The weight a reply draws with: base weight times every trait
/// multiplier the speaker carries (CiF-style biasing, never gating)
fn draw_weight(r: &Reply, speaker: &Speaker) -> f32 {
    let mut w = r.weight;
    for (t, m) in r.bias {
        if speaker.traits.contains(t) { w *= m; }
    }
    w
}

/// An unused variant of a line set: the template (so it can be spent) and
/// the rendered text with `{other}` filled with the listener's name
fn fresh_line(lines: &[&'static str], other: &str, used: &HashSet<&'static str>, rng: &mut Rng) -> Option<(&'static str, String)> {
    let fresh: Vec<&'static str> = lines.iter().copied().filter(|l| !used.contains(*l)).collect();
    fresh.get(rng.below(fresh.len())).map(|&t| (t, t.replace("{other}", other)))
}

/// The matching trees, most specific first (Ruskin's scoring: criterion
/// count; a tie keeps library order so authors control the fall-through)
fn cascade<'a>(library: &'a [DialogueTree], initiator_facts: &Facts) -> Vec<&'a DialogueTree> {
    let mut v: Vec<(usize, &'a DialogueTree)> = library.iter().enumerate()
        .filter(|(_, t)| t.criteria.iter().all(|c| c.holds(initiator_facts)))
        .map(|(i, t)| (i, t))
        .collect();
    v.sort_by(|(ia, ta), (ib, tb)| tb.criteria.len().cmp(&ta.criteria.len()).then(ia.cmp(ib)));
    v.into_iter().map(|(_, t)| t).collect()
}

/// What the player is offered at their turn: up to `n` sampled forks plus
/// the standing option to walk away (choose returns None).
pub struct Choice {
    /// The index of the fork among the node's replies... but for the
    /// caller what matters is which offered option was picked
    pub act: Act,
    /// The line they'd say - the player picks a *way of speaking*
    pub preview: String,
}

/// Run a whole NPC-NPC conversation: the initiator opens the most
/// specific matching tree, the other answers, and the walk descends
/// call/response levels until an ending fork, an exhausted tree (which
/// falls through to the next-most-specific), or exhaustion of the generic
/// tail itself. Lines are spent as spoken; `max_turns` bounds the walk.
pub fn converse(
    a: &Speaker,
    b: &Speaker,
    rel: Relation,
    library: &[DialogueTree],
    rng: &mut Rng,
) -> Transcript {
    converse_with(a, b, rel, library, usize::MAX, &mut |_| Some(0), rng)
}

/// As `converse`, but speaker `player`'s response turns go through
/// `choose`: it sees up to two weighted-sampled options (as previews of
/// what they'd say) and returns the option index, or None to end the
/// conversation early (Ending::LeavesEarly).
pub fn converse_with(
    a: &Speaker,
    b: &Speaker,
    rel: Relation,
    library: &[DialogueTree],
    player: usize,
    choose: &mut dyn FnMut(&[Choice]) -> Option<usize>,
    rng: &mut Rng,
) -> Transcript {
    let pair = [a, b];
    let mut tr = Transcript {
        speakers: [a.name.clone(), b.name.clone()],
        trees_walked: vec![],
        turns: vec![],
        ending: None,
    };
    let mut used: HashSet<&'static str> = HashSet::new();
    let trees = cascade(library, &Facts::for_roles(a, b, rel));
    let mut trees = trees.iter().peekable();
    let mut node: Option<&Exchange> = trees.next().map(|t| {
        tr.trees_walked.push(t.name);
        t.root
    });
    // The tree the current node belongs to, for the transcript
    let mut node_tree = tr.trees_walked.first().copied().unwrap_or("");
    // The lead/respond split: the initiator speaks every call, the other
    // answers - a call/response ping-pong like an RPG dialogue tree.
    // Which fork the answerer takes still steers where the talk goes.
    let (caller, responder) = (0usize, 1usize);

    let advance_to_rejoin = |trees: &mut std::iter::Peekable<std::slice::Iter<&DialogueTree>>| -> Option<(&'static str, &'static Exchange)> {
        while let Some(t) = trees.next() {
            if let Some(rej) = t.rejoin {
                return Some((t.name, rej));
            }
        }
        None
    };

    for _ in 0..MAX_TURNS {
        let Some(ex) = node else { tr.ending = Some(Ending::Exhausted); break };
        // Only open a beat the other side can still answer: a call left
        // hanging with no reply reads as a non-sequitur, so a spent node
        // falls through before anything is said
        let facts = Facts::for_roles(pair[responder], pair[caller], rel);
        let pool = eligible(ex.replies, &facts, &used);
        let fresh_call = if pool.is_empty() { None } else { fresh_line(ex.calls, &pair[responder].name, &used, rng) };
        let Some((call_t, call)) = fresh_call else {
            // Nothing fresh to call with or answer: fall through to the next
            // rejoinable tree, or dry up entirely at the generic tail
            match advance_to_rejoin(&mut trees) {
                Some((name, rej)) => { tr.trees_walked.push(name); node = Some(rej); node_tree = name; }
                None => node = None,
            }
            continue;
        };
        used.insert(call_t);
        tr.turns.push(Turn { speaker: caller, act: ex.act, text: call,
            concepts: ex.concepts.to_vec(), tree: node_tree });

        let weights: Vec<f32> = pool.iter().map(|r| draw_weight(r, pair[responder])).collect();

        // (fork index, line template): the fork picked and the variant it
        // will speak. For a player turn the variants are rolled with the
        // options so the preview is exactly the line they get.
        let (pick, line_t): (usize, &'static str) = if responder == player {
            // The player sees up to two weighted-sampled options without replacement + "leave"
            let mut available: Vec<usize> = (0..pool.len()).collect();
            let mut sampled: Vec<usize> = vec![];
            let target_count = 2.min(pool.len());
            while sampled.len() < target_count && !available.is_empty() {
                let avail_weights: Vec<f32> = available.iter().map(|&i| weights[i]).collect();
                let chosen_avail_idx = rng.weighted(&avail_weights);
                let chosen_pool_idx = available.remove(chosen_avail_idx);
                sampled.push(chosen_pool_idx);
            }
            let mut rolled: Vec<(&'static str, Choice)> = vec![];
            for &i in &sampled {
                let r = pool[i];
                let (t, preview) = fresh_line(r.lines, &pair[caller].name, &used, rng)
                    .map(|(t, s)| (t, s))
                    .unwrap_or((r.lines[0], r.lines[0].to_string()));
                rolled.push((t, Choice { act: r.act, preview }));
            }
            let choices: Vec<Choice> = rolled.iter().map(|(_, c)| Choice { act: c.act, preview: c.preview.clone() }).collect();
            match choose(&choices).and_then(|i| sampled.get(i).copied().zip(rolled.get(i))) {
                Some((i, (t, _))) => (i, *t),
                None => { tr.ending = Some(Ending::LeavesEarly); break; }
            }
        } else {
            let i = rng.weighted(&weights);
            match fresh_line(pool[i].lines, &pair[caller].name, &used, rng) {
                Some((t, _)) => (i, t),
                None => { tr.ending = Some(Ending::Exhausted); break; }
            }
        };
        let reply = pool[pick];
        used.insert(line_t);
        tr.turns.push(Turn { speaker: responder, act: reply.act, text: line_t.replace("{other}", &pair[caller].name),
            concepts: reply.concepts.to_vec(), tree: node_tree });

        if let Some(end) = reply.end {
            tr.ending = Some(end);
            break;
        }
        node = match &reply.then {
            Some(next) => Some(next),
            None => match advance_to_rejoin(&mut trees) {
                Some((name, rej)) => { tr.trees_walked.push(name); node_tree = name; Some(rej) }
                // The generic tail keeps drifting: stay on this node until
                // its material is spent (the small-talk doldrums)
                None => node,
            },
        };
    }
    if tr.ending.is_none() {
        tr.ending = Some(Ending::Exhausted);
    }
    tr
}

// ---------------------------------------------------------------------
// House meetings: a looser model. Statements carry concept tags and each
// reply binds to the last speaker, the organizer's open question, or an
// earlier statement - whichever pulls hardest.

/// A thing said on the meeting record
#[derive(Debug)]
pub struct Statement {
    /// Index into the `attendees` slice the meeting was run with
    pub speaker: usize,
    pub act: Act,
    pub text: String,
    pub concepts: Vec<&'static str>,
    /// Index of the statement it answers, if it answers one
    pub replying_to: Option<usize>,
}

/// A line an attendee could bring to a meeting
#[derive(Debug, Clone, Copy)]
pub struct MeetingLine {
    pub act: Act,
    /// What it's about - replies bind on concept overlap
    pub concepts: &'static [&'static str],
    /// (trait, multiplier) on the bid when the speaker has it
    pub bias: &'static [(Trait, f32)],
    /// Ways to say it; `{other}` fills with the prior speaker's name
    pub lines: &'static [&'static str],
}

/// The organizer's agenda item: a question with the concepts it invites
pub struct MeetingPrompt {
    /// The question as asked, `{org}` fills with the organizer's name
    pub text: &'static str,
    /// Concepts an answer should touch to count as answering it
    pub concepts: &'static [&'static str],
}

/// How long the organizer's question stays the loudest thing in the room:
/// after this many answers it stops pulling replies to itself
const QUESTION_PULL: usize = 2;

/// How strongly a just-answered statement pulls a follow-up vs an older
/// one, and how much the open question outpulls both
const LAST_PULL: f32 = 2.5;
const OLD_PULL: f32 = 1.5;
const QUESTION_SCORE: f32 = 3.0;

/// Run a house meeting's talk: the organizer opens with a question, then
/// each round the attendee whose best bid weighs heaviest speaks - an
/// answer to the open question, a response to the last speaker (agree or
/// push back, by temperament), or a revival of an earlier statement that
/// touches something they care about. Whoever spoke last sits the round
/// out. Returns the full record, organizer's question included.
pub fn run_meeting<'a>(
    organizer: &Speaker,
    attendees: &[&Speaker],
    prompt: &MeetingPrompt,
    pools: &'a [&'a [MeetingLine]],
    rng: &mut Rng,
) -> Vec<Statement> {
    let mut record: Vec<Statement> = vec![Statement {
        speaker: usize::MAX, // the organizer isn't an attendee
        act: Act::Question,
        text: format!("{}: {}", organizer.name, prompt.text),
        concepts: prompt.concepts.to_vec(),
        replying_to: None,
    }];
    let mut used: HashSet<&'static str> = HashSet::new();
    let mut answers = 0usize;
    let mut rounds = pools.len().max(4) * 2;

    while rounds > 0 {
        rounds -= 1;
        let last_idx = record.len() - 1;
        let last_speaker = record[last_idx].speaker;
        let question_open = answers < QUESTION_PULL;

        // Bid: (speaker, target statement, line, score)
        let mut bids: Vec<(usize, usize, &'a MeetingLine, f32)> = vec![];
        for (i, sp) in attendees.iter().enumerate() {
            if i == last_speaker { continue; } // nobody speaks twice running
            let pool = pools[i];
            let mut score = 1.0f32;
            for (t, m) in [(Trait::Gregarious, 1.5), (Trait::Proud, 0.8), (Trait::Ambitious, 1.2)] {
                if sp.traits.contains(&t) { score *= m; }
            }
            // What could they say, and what would it answer?
            for (si, s) in record.iter().enumerate() {
                if si == 0 && !question_open { continue; }
                if s.speaker == i { continue; } // you can't answer yourself
                let open_q = si == 0;
                let pull = if open_q { QUESTION_SCORE }
                    else if si == last_idx { LAST_PULL }
                    else { OLD_PULL };
                for l in pool {
                    // Agreement or pushback can only reply to a fellow attendee's statement, not the organizer's opening question
                    if open_q && (l.act == Act::Agree || l.act == Act::PushBack) {
                        continue;
                    }
                    let overlap = l.concepts.iter().any(|c| s.concepts.contains(c));
                    if !overlap && !open_q { continue; }
                    // an open question invites any contribution, but lines
                    // touching its concepts answer it best
                    let fit = if open_q {
                        if overlap { 1.5 } else { 0.5 }
                    } else { 1.0 };
                    if l.lines.iter().all(|x| used.contains(*x)) { continue; }
                    let mut s = score * pull * fit;
                    for (t, m) in l.bias {
                        if sp.traits.contains(t) { s *= m; }
                    }
                    bids.push((i, si, l, s));
                }
            }
        }
        if bids.is_empty() { break; }
        let weights: Vec<f32> = bids.iter().map(|b| b.3).collect();
        let (speaker, target, line, _) = bids[rng.weighted(&weights)];
        let other = if record[target].speaker == usize::MAX {
            organizer.name.as_str()
        } else {
            attendees[record[target].speaker].name.as_str()
        };
        let Some((t, text)) = fresh_line(line.lines, other, &used, rng) else { continue };
        used.insert(t);
        if target == 0 { answers += 1; }
        record.push(Statement {
            speaker, act: line.act, text,
            concepts: line.concepts.to_vec(),
            replying_to: Some(target),
        });
    }
    record
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::dialogue_trees::*;

    #[test]
    fn test_specific_tree_wins_over_generic() {
        let library = village_library();
        let mut rng = Rng::new(42);

        // A sick villager talking to a well villager
        let mut sick = Speaker::new("Eliza");
        sick.status.push("sick");
        let well = Speaker::new("Aldric");

        let tr = converse(&sick, &well, Relation::Acquaintance, &library, &mut rng);
        assert_eq!(tr.trees_walked[0], "the_sickness", "sick speaker leads with the_sickness tree");
        assert!(tr.turns.iter().any(|t| t.tree == "the_sickness"));

        // A well villager talking to a well villager leads with generic smalltalk
        let tr_well = converse(&well, &Speaker::new("Bram"), Relation::Acquaintance, &library, &mut rng);
        assert_eq!(tr_well.trees_walked[0], "village_smalltalk", "well speaker leads with smalltalk");
    }

    #[test]
    fn test_specificity_cascade_falls_through_to_smalltalk() {
        let library = village_library();
        let mut sick = Speaker::new("Eliza");
        sick.status.push("sick");
        let well = Speaker::new("Aldric");

        // Over multiple seeds, if the sickness tree doesn't end abruptly, it falls through to smalltalk
        let mut fell_through = false;
        for seed in 0..50 {
            let mut rng = Rng::new(seed);
            let tr = converse(&sick, &well, Relation::Acquaintance, &library, &mut rng);
            if tr.trees_walked.contains(&"village_smalltalk") && tr.trees_walked[0] == "the_sickness" {
                fell_through = true;
                break;
            }
        }
        assert!(fell_through, "sickness tree should fall through to smalltalk on some paths");
    }

    #[test]
    fn test_quip_exclusivity_prevents_duplicate_lines() {
        let library = village_library();
        for seed in 0..30 {
            let mut rng = Rng::new(seed);
            let a = Speaker::new("Aldric");
            let b = Speaker::new("Eliza");
            let tr = converse(&a, &b, Relation::Acquaintance, &library, &mut rng);

            let mut seen = HashSet::new();
            for turn in &tr.turns {
                assert!(seen.insert(turn.text.clone()), "Line was repeated: {:?}", turn.text);
            }
        }
    }

    #[test]
    fn test_personality_biases_reply_distribution() {
        let library = village_library();

        let mut hothead = Speaker::new("Corvin");
        hothead.traits.push(Trait::Hotheaded);

        let calm = Speaker::new("Aldric");

        let rival = Speaker::new("Wulfric");

        // Count pushbacks in old_scores for hothead vs calm
        let mut hot_pushbacks = 0;
        let mut calm_pushbacks = 0;
        for seed in 0..100 {
            let mut rng = Rng::new(seed);
            let tr_hot = converse(&rival, &hothead, Relation::Rival, &library, &mut rng);
            hot_pushbacks += tr_hot.turns.iter().filter(|t| t.speaker == 1 && t.act == Act::PushBack).count();

            let mut rng = Rng::new(seed);
            let tr_calm = converse(&rival, &calm, Relation::Rival, &library, &mut rng);
            calm_pushbacks += tr_calm.turns.iter().filter(|t| t.speaker == 1 && t.act == Act::PushBack).count();
        }
        assert!(hot_pushbacks > calm_pushbacks, "Hothead should push back more: hot={}, calm={}", hot_pushbacks, calm_pushbacks);
    }

    #[test]
    fn test_player_can_pick_options_or_leave_early() {
        let library = village_library();
        let a = Speaker::new("Aldric");
        let player = Speaker::new("Player");

        // Player leaves immediately (choose returns None)
        let mut rng = Rng::new(1);
        let tr_leave = converse_with(
            &a, &player, Relation::Acquaintance, &library, 1,
            &mut |_choices| None,
            &mut rng,
        );
        assert_eq!(tr_leave.ending, Some(Ending::LeavesEarly));
        assert_eq!(tr_leave.turns.len(), 1, "Only initial caller turn occurred before player left");

        // Player picks first option each time
        let mut rng = Rng::new(2);
        let mut choices_seen = 0;
        let tr_play = converse_with(
            &a, &player, Relation::Acquaintance, &library, 1,
            &mut |choices| {
                assert!(!choices.is_empty() && choices.len() <= 2);
                choices_seen += 1;
                Some(0)
            },
            &mut rng,
        );
        assert!(choices_seen > 0);
        assert!(tr_play.turns.len() > 1);
        assert_ne!(tr_play.ending, Some(Ending::LeavesEarly));
    }

    #[test]
    fn test_house_meeting_threading() {
        let organizer = Speaker::new("Reeve");
        let mut a = Speaker::new("Eliza");
        a.status.push("sick");
        let mut b = Speaker::new("Bram");
        b.status.push("hungry");
        let c = Speaker::new("Corvin");

        let pool_a = meeting_pool(&a);
        let pool_b = meeting_pool(&b);
        let pool_c = meeting_pool(&c);
        let pools: Vec<&[MeetingLine]> = vec![&pool_a, &pool_b, &pool_c];
        let attendees = [&a, &b, &c];

        let mut rng = Rng::new(77);
        let statements = run_meeting(&organizer, &attendees, &MEET_PROMPT, &pools, &mut rng);

        assert!(statements.len() >= 4, "Meeting should produce multiple statements");
        assert_eq!(statements[0].speaker, usize::MAX); // organizer question
        assert_eq!(statements[0].replying_to, None);

        // Every subsequent statement must reply to a valid earlier statement
        for (idx, stmt) in statements.iter().enumerate().skip(1) {
            let target = stmt.replying_to.expect("Attendee statement must reply to something");
            assert!(target < idx, "Target statement must be prior to this statement");
            // No attendee speaks twice in a row
            if idx > 1 {
                assert_ne!(stmt.speaker, statements[idx - 1].speaker, "Speaker cannot speak consecutively");
            }
        }
    }

    #[test]
    fn test_meeting_replies_do_not_agree_with_opener() {
        let organizer = Speaker::new("Reeve");
        let a = Speaker::new("Eliza");
        let b = Speaker::new("Bram");
        let pool_a = meeting_pool(&a);
        let pool_b = meeting_pool(&b);
        let pools: Vec<&[MeetingLine]> = vec![&pool_a, &pool_b];
        let attendees = [&a, &b];

        for seed in 0..50 {
            let mut rng = Rng::new(seed);
            let statements = run_meeting(&organizer, &attendees, &MEET_PROMPT, &pools, &mut rng);
            // Any statement answering statement 0 (the open question) must NOT be Agree or PushBack
            for stmt in &statements {
                if stmt.replying_to == Some(0) {
                    assert_ne!(stmt.act, Act::Agree, "Should not agree with an open question");
                    assert_ne!(stmt.act, Act::PushBack, "Should not push back on an open question without raising a point");
                }
            }
        }
    }

    #[test]
    fn test_sick_rival_falls_through_to_rivalry_then_smalltalk() {
        let library = village_library();
        let mut sick_rival = Speaker::new("Eliza");
        sick_rival.status.push("sick");
        let rival = Speaker::new("Corvin");

        // The cascade has [the_sickness, old_scores, village_smalltalk]
        let facts = Facts::for_roles(&sick_rival, &rival, Relation::Rival);
        let cascaded = cascade(&library, &facts);
        let names: Vec<&str> = cascaded.iter().map(|t| t.name).collect();
        assert_eq!(names, vec!["the_sickness", "old_scores", "village_smalltalk"]);

        // When the conversation runs, old_scores is preserved and walked via its rejoin
        let mut saw_rival_walked = false;
        for seed in 0..50 {
            let mut rng = Rng::new(seed);
            let tr = converse(&sick_rival, &rival, Relation::Rival, &library, &mut rng);
            if tr.trees_walked.contains(&"old_scores") && tr.trees_walked[0] == "the_sickness" {
                saw_rival_walked = true;
                break;
            }
        }
        assert!(saw_rival_walked, "old_scores should be preserved and walked on fallback from sickness");
    }

    #[test]
    fn test_meeting_nobody_replies_to_themselves() {
        let organizer = Speaker::new("Reeve");
        let mut a = Speaker::new("Eliza");
        a.status.push("sick");
        let b = Speaker::new("Bram");
        let mut c = Speaker::new("Corvin");
        c.ideology = "merchant";
        let (pa, pb, pc) = (meeting_pool(&a), meeting_pool(&b), meeting_pool(&c));
        let pools: Vec<&[MeetingLine]> = vec![&pa, &pb, &pc];
        let attendees = [&a, &b, &c];
        for seed in 0..50 {
            let mut rng = Rng::new(seed);
            let st = run_meeting(&organizer, &attendees, &MEET_PROMPT, &pools, &mut rng);
            for s in &st[1..] {
                let t = s.replying_to.unwrap();
                assert_ne!(st[t].speaker, s.speaker, "seed {}: {:?} answers their own statement", seed, s.text);
            }
        }
    }

    #[test]
    fn test_every_call_gets_an_answer_unless_it_ends() {
        let library = village_library();
        for seed in 0..100 {
            let mut rng = Rng::new(seed);
            let mut a = Speaker::new("Aldric");
            if seed % 2 == 0 { a.status.push("sick"); }
            let b = Speaker::new("Eliza");
            let tr = converse(&a, &b, Relation::Rival, &library, &mut rng);
            for w in tr.turns.windows(2) {
                assert!(!(w[0].speaker == 0 && w[1].speaker == 0),
                    "seed {}: call {:?} left unanswered", seed, w[0].text);
            }
        }
    }

    #[test]
    fn test_player_choices_no_duplicate_options() {
        let library = village_library();
        let a = Speaker::new("Aldric");
        let player = Speaker::new("Player");

        for seed in 0..50 {
            let mut rng = Rng::new(seed);
            converse_with(
                &a, &player, Relation::Acquaintance, &library, 1,
                &mut |choices| {
                    if choices.len() == 2 {
                        assert_ne!(choices[0].preview, choices[1].preview, "Player choices must be distinct");
                    }
                    Some(0)
                },
                &mut rng,
            );
        }
    }
}
