//! The arc-scale metrics (E0-5, #1409): the Appendix A table's
//! measurable rows, read off `run_arc` transcripts and the shipped
//! content tables - own-voice share, distinct word pairs and Self-BLEU
//! across an arc and across villagers, rule-of-three coverage, and the
//! callback-truth helpers `episodes::mem_kind` feeds.
//!
//! Own-voice share (Appendix A's "flat" check): the fraction of a
//! villager's lines that only this villager could say - the tree or the
//! fork that produced them keys on the speaker's own fact table. A line
//! off `village_smalltalk` is anybody's; a `tell:ghost:*` slip, a
//! persona-seeded worry, joke or leak, and a fork gated `self:`/`any:`
//! on their facts are theirs. `open:`/`closed:` tier reads are the
//! pair's state, not the persona's - they don't count.

use crate::engine::det::{HashMap, HashSet};
use crate::game::arc::{run_arc, ArcTalk, Policy};
use crate::game::conversation::{
    Act, Criterion, DialogueTree, Exchange, Relation, Speaker,
};
use crate::game::fact_content::content_for_fact;
use crate::game::persona::{BELIEFS, EVENT_KINDS};
use crate::game::relationships::RelKind;
use crate::game::setting::Setting;

/// The talks where "after 10 talks with one villager" reads: the back
/// half of a twenty-talk arc, where the Appendix's 30% target applies
pub const LATE_FROM: usize = 10;

/// The speaker's own facts, split the way `Facts::for_roles` folds
/// them: `self:` reads own facts plus the hidden half; `any:` reads
/// the open half only. The `open:`/`closed:` area-tier strings are the
/// pair's state and `status:wx-*` the whole sky's - both shared facts,
/// not the persona's, so both sets drop them
fn own_sets(s: &Speaker) -> (HashSet<String>, HashSet<String>) {
    let drop_tiers = |f: &String| !f.starts_with("open:")
        && !f.starts_with("closed:") && !f.starts_with("status:wx-");
    let self_set: HashSet<String> = s.own_facts().into_iter()
        .chain(s.facts.hidden.iter().cloned()).filter(drop_tiers).collect();
    let any_set: HashSet<String> = s.own_facts().into_iter().filter(drop_tiers).collect();
    (self_set, any_set)
}

/// Do the criteria key on the speaker's own fact table? A `self:`
/// criterion reads their facts (positive or negative - a Lacks still
/// keys on what their table lacks); an `any:` criterion reads both
/// sides, so it counts only when the fact is one this speaker carries.
fn own_keyed(criteria: &[Criterion], self_set: &HashSet<String>, any_set: &HashSet<String>) -> bool {
    criteria.iter().any(|c| {
        let (fact, neg) = match c {
            Criterion::Has(x) => (*x, false),
            Criterion::Lacks(x) => (*x, true),
        };
        match fact.split_once(':') {
            Some(("self", f)) => if neg { !f.starts_with("open:") && !f.starts_with("closed:") }
                else { self_set.contains(f) },
            Some(("any", f)) => if neg { !f.starts_with("open:") && !f.starts_with("closed:") }
                else { any_set.contains(f) },
            _ => false,
        }
    })
}

/// Does the exchange hold `line`, and if so does the node that owns it
/// key on the speaker's own facts? Plain `calls`/`lines` pools carry no
/// criteria of their own - only a Reply fork or a Voice register can
/// gate a line on facts below the tree's
fn exchange_keyed(ex: &Exchange, line: &str, self_set: &HashSet<String>, any_set: &HashSet<String>) -> bool {
    if ex.voice.iter().any(|v| v.lines.contains(&line) && own_keyed(v.criteria, self_set, any_set)) {
        return true;
    }
    for r in ex.replies {
        if r.lines.contains(&line) && own_keyed(r.criteria, self_set, any_set) { return true; }
        if r.voice.iter().any(|v| v.lines.contains(&line) && own_keyed(v.criteria, self_set, any_set)) {
            return true;
        }
        if let Some(t) = r.then.as_ref() {
            if exchange_keyed(t, line, self_set, any_set) { return true; }
        }
    }
    false
}

/// Is `line` one of the tellings authored for a fact this speaker
/// carries? `the_asking`'s secret answers draw from `telling` - the
/// fact's own words under the ask - and ride a synthetic tree the
/// library doesn't hold, so the share names them by content
fn telling_keyed(line: &str, self_set: &HashSet<String>) -> bool {
    crate::game::fact_content::FACT_CONTENTS.iter()
        .any(|c| self_set.contains(c.fact) && c.tiers.iter().any(|&(_, l)| l == line))
}

/// Was this turn keyed to the speaker's own facts? The tree's own
/// criteria first - then, when the line's template is known, the fork
/// or register that let it through, then the authored tellings. Turns
/// without a template (a composed line) judge on the tree alone. A
/// slip or a worry speaks persona-seeded content (the ghost's leak,
/// the doorknob fear); a joke counts only when it's theirs - the
/// `joke_receipt` echo answers the other side's joke from a stock
/// table anyone can draw
pub fn turn_keyed(turn: &crate::game::conversation::Turn, lib: &[DialogueTree],
                  self_set: &HashSet<String>, any_set: &HashSet<String>) -> bool {
    if matches!(turn.act, Act::Slip | Act::Worry)
        || (turn.act == Act::Joke && !turn.concepts.contains(&"joke_receipt")) { return true; }
    let Some(tree) = lib.iter().find(|t| t.name == turn.tree) else {
        // a synthetic tree (the_asking) still tells true by its line
        return turn.line.map(|l| telling_keyed(l, self_set)).unwrap_or(false);
    };
    if own_keyed(tree.criteria, self_set, any_set) { return true; }
    let Some(line) = turn.line else { return false; };
    exchange_keyed(tree.root, line, self_set, any_set)
        || tree.rejoin.map(|e| exchange_keyed(e, line, self_set, any_set)).unwrap_or(false)
        || telling_keyed(line, self_set)
}

/// The own-voice share over one speaker's turns: (keyed, total) for
/// `side`'s turns in talks `from..to` - `side` 0 is the arc's
/// villager. Read the pair: share = keyed / total
pub fn own_voice(arc: &[ArcTalk], lib: &[DialogueTree], them: &Speaker,
                 side: usize, from: usize, to: usize) -> (usize, usize) {
    let (self_set, any_set) = own_sets(them);
    let (mut keyed, mut total) = (0, 0);
    for t in arc.iter().take(to).skip(from) {
        for turn in &t.transcript.turns {
            if turn.speaker != side { continue; }
            total += 1;
            if turn_keyed(turn, lib, &self_set, &any_set) { keyed += 1; }
        }
    }
    (keyed, total)
}

/// Lowercased words, punctuation stripped - the unit both repetition
/// metrics count
fn words(line: &str) -> Vec<String> {
    line.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|w| !w.is_empty()).collect()
}

/// Distinct adjacent word pairs in a line corpus - the arc's
/// vocabulary-productivity number (Appendix A: "distinct word pairs in
/// one villager's lines over 20 talks", recorded as a baseline)
pub fn word_pairs(lines: &[String]) -> usize {
    let mut pairs: HashSet<(String, String)> = HashSet::default();
    for l in lines {
        for w in words(l).windows(2) {
            pairs.insert((w[0].clone(), w[1].clone()));
        }
    }
    pairs.len()
}

/// n-gram counts of a token stream
fn ngrams(toks: &[String], n: usize) -> HashMap<String, usize> {
    let mut m: HashMap<String, usize> = HashMap::default();
    for w in toks.windows(n) { *m.entry(w.join(" ")).or_insert(0) += 1; }
    m
}

/// BLEU of one corpus against reference corpora (unigram + bigram,
/// counts clipped per reference, standard brevity penalty). The
/// Self-BLEU measure (Zhu et al. 2018, per Appendix A): score each
/// sample against the rest and average - higher means less diverse
fn bleu(cand: &[String], refs: &[&Vec<String>]) -> f64 {
    let c_len = cand.len();
    if c_len == 0 || refs.is_empty() { return 0.0; }
    let r_len = refs.iter().map(|r| r.len())
        .min_by_key(|&l| l.abs_diff(c_len)).unwrap_or(0);
    let bp = if c_len > r_len { 1.0 }
        else { (1.0 - r_len as f64 / c_len as f64).exp() };
    let ref_grams: Vec<_> = refs.iter().map(|r| (ngrams(r, 1), ngrams(r, 2))).collect();
    let mut logp = 0.0;
    for (n, cn) in [(1, ngrams(cand, 1)), (2, ngrams(cand, 2))] {
        let total: usize = cn.values().sum();
        if total == 0 { return 0.0; }
        let clipped: usize = cn.iter().map(|(g, c)| {
            let m = ref_grams.iter()
                .map(|(g1, g2)| (if n == 1 { g1 } else { g2 }).get(g).copied().unwrap_or(0))
                .max().unwrap_or(0);
            (*c).min(m)
        }).sum();
        logp += (clipped as f64 / total as f64).ln() / 2.0;
    }
    bp * logp.exp()
}

/// Self-BLEU over a set of line corpora: each corpus scored against
/// the others, averaged. Within an arc the corpora are the talks'
/// villager lines; across the cast they're each villager's arc's lines
pub fn self_bleu(corpora: &[Vec<String>]) -> f64 {
    let toks: Vec<Vec<String>> = corpora.iter()
        .map(|c| c.iter().flat_map(|l| words(l)).collect()).collect();
    let n = toks.len();
    if n < 2 { return 0.0; }
    (0..n).map(|i| {
        let refs: Vec<&Vec<String>> = toks.iter().enumerate()
            .filter(|(j, _)| *j != i).map(|(_, r)| r).collect();
        bleu(&toks[i], &refs)
    }).sum::<f64>() / n as f64
}

/// The rule-of-three coverage row (Appendix A): of the fact types the
/// cast ships - every ghost kind and every lie - how many carry a
/// tell, a telling and a pressure choice. Returns (covered, shipped,
/// missing): 100% is the target, `missing` names what's not there
pub fn rule_of_three() -> (usize, usize, Vec<String>) {
    let shipped: Vec<String> = EVENT_KINDS.iter().map(|k| format!("ghost:{}", k.tag()))
        .chain(BELIEFS.iter().map(|b| format!("lie:{}", b.tag()))).collect();
    let missing: Vec<String> = shipped.iter().filter(|f| match content_for_fact(f) {
        Some(c) => c.tell.is_empty() || c.slip.is_empty() || c.gossip.is_empty()
            || c.pressure.is_empty() || c.tiers.iter().all(|(_, l)| l.is_empty()),
        None => true,
    }).cloned().collect();
    (shipped.len() - missing.len(), shipped.len(), missing)
}

/// One arc measured: the villager's own-voice share early and late
/// ("after 10 talks" is the Appendix's row), the distinct word pairs
/// over the whole arc, the Self-BLEU between its talks, and the
/// villager's line corpus per talk - the caller keeps the corpus for
/// the cast-level Self-BLEU
pub struct ArcMeasure {
    pub own_early: f64,
    pub own_late: f64,
    pub word_pairs: usize,
    pub bleu: f64,
    /// The villager's rendered lines per talk - the cast metric's input
    pub corpora: Vec<Vec<String>>,
}

/// Run the arc and measure it. `side` 0 is `them` - the villager the
/// metrics are about
pub fn measure_arc(them: &Speaker, you: &Speaker, rel: Relation, kind: Option<RelKind>,
                   talks: usize, policy: Policy, lib: &[DialogueTree], setting: &Setting,
                   seed: u64) -> ArcMeasure {
    let (_, arc) = run_arc(them, you, rel, kind, talks, policy, lib, setting, seed);
    let corpora: Vec<Vec<String>> = arc.iter().map(|t| t.transcript.turns.iter()
        .filter(|turn| turn.speaker == 0).map(|turn| turn.text.clone()).collect()).collect();
    let all: Vec<String> = corpora.iter().flatten().cloned().collect();
    let (ke, te) = own_voice(&arc, lib, them, 0, 0, LATE_FROM.min(talks));
    let (kl, tl) = own_voice(&arc, lib, them, 0, LATE_FROM.min(talks), talks);
    ArcMeasure {
        own_early: if te == 0 { 0.0 } else { ke as f64 / te as f64 },
        own_late: if tl == 0 { 0.0 } else { kl as f64 / tl as f64 },
        word_pairs: word_pairs(&all),
        bleu: self_bleu(&corpora),
        corpora,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::arc::Policy;
    use crate::game::conversation::{Criterion::Has, Reply, Transcript, Turn};
    use crate::game::dialogue_trees::village_library;
    use crate::game::setting::STREET;

    fn tree(name: &'static str, criteria: &'static [Criterion], ex: &'static Exchange) -> DialogueTree {
        DialogueTree { name, criteria, root: ex, rejoin: None }
    }

    fn say(speaker: usize, tree: &'static str, line: Option<&'static str>) -> Turn {
        Turn { speaker, act: Act::Remark, respond: None, text: "text".into(),
            concepts: vec![], tree, line, channel: crate::game::setting::Channel::Public }
    }

    /// Twelve arc slots with `turns` riding the last one: `own_voice`
    /// counts every turn in the window, so this reads the share and the
    /// `from` cutoff without needing Turn to be Clone
    fn arc_at(turns: Vec<Turn>) -> Vec<ArcTalk> {
        let mut turns = Some(turns);
        (0..12).map(|i| ArcTalk { index: i, seed: 0,
            transcript: Transcript {
                speakers: ["Hob".into(), "Wanderer".into()],
                trees_walked: vec![], turns: if i == 11 { turns.take().unwrap() } else { vec![] },
                ending: None, vibe: None, asked: vec![], forward: 0,
                evasions: vec![], worry_told: None, slips: vec![], jokes: vec![],
                shared: 0, overshot: false, setting: crate::game::setting::STREET },
            picks: vec![], choices_made: vec![],
            open_after: crate::game::conversation::Open::default() }).collect()
    }

    /// a turn gated `self:` on the villager's own fact is own-voice; the
    /// same line with no gate - or gated on the pair's tier - is not
    #[test]
    fn a_self_keyed_turn_counts() {
        static EX: Exchange = Exchange { act: Act::Remark, concepts: &[],
            calls: &["the wind"], voice: &[],
            replies: &[Reply { criteria: &[Has("self:ghost:levy")], weight: 1.0,
                bias: &[], act: Act::Agree, respond: None, concepts: &[],
                lines: &["they took him at the gate"], voice: &[], then: None, end: None }] };
        let lib = vec![
            tree("plain", &[], &EX),
            tree("keyed", &[Has("self:ghost:levy")], &EX),
            tree("tier", &[Has("self:open:heart:2")], &EX),
        ];
        let mut them = Speaker::new("Hob");
        them.facts.hidden.push("ghost:levy".into());
        let (self_set, any_set) = own_sets(&them);
        let t_keyed = say(0, "plain", Some("they took him at the gate"));
        let t_tree = say(0, "keyed", None);
        let t_tier = say(0, "tier", Some("the wind"));
        let t_plain = say(0, "plain", Some("the wind"));
        assert!(turn_keyed(&t_keyed, &lib, &self_set, &any_set), "the gated reply is theirs");
        assert!(turn_keyed(&t_tree, &lib, &self_set, &any_set), "the keyed tree is theirs");
        assert!(!turn_keyed(&t_tier, &lib, &self_set, &any_set), "a tier read is the pair's, not theirs");
        assert!(!turn_keyed(&t_plain, &lib, &self_set, &any_set), "plain lines are anybody's");
    }

    /// the persona-seeded channels - a slip, a joke, a worry - are
    /// own-voice wherever they land, but the joke_receipt echo is the
    /// other side's joke answered from a stock table
    #[test]
    fn the_seeded_channels_count() {
        let lib = village_library();
        let them = Speaker::new("Hob");
        let (self_set, any_set) = own_sets(&them);
        for act in [Act::Slip, Act::Joke, Act::Worry] {
            let t = Turn { speaker: 0, act, respond: None, text: "…".into(),
                concepts: vec![], tree: "village_smalltalk", line: None,
                channel: crate::game::setting::Channel::Public };
            assert!(turn_keyed(&t, &lib, &self_set, &any_set), "{act:?} is own-voice");
        }
        let receipt = Turn { speaker: 0, act: Act::Joke, respond: None,
            text: "aye, and worse".into(), concepts: vec!["joke", "joke_receipt"],
            tree: "village_smalltalk", line: None,
            channel: crate::game::setting::Channel::Public };
        assert!(!turn_keyed(&receipt, &lib, &self_set, &any_set),
            "a stock receipt to the other's joke isn't theirs");
    }

    /// a fair sky is everyone's - a line gated `self:status:wx-*`
    /// alone says nothing about who carries it
    #[test]
    fn the_days_weather_is_not_theirs() {
        static EX: Exchange = Exchange { act: Act::Remark, concepts: &[],
            calls: &["bright enough for drying"], voice: &[],
            replies: &[Reply { criteria: &[Has("self:status:wx-fair")], weight: 1.0,
                bias: &[], act: Act::Agree, respond: None, concepts: &[],
                lines: &["a fine drying day"], voice: &[], then: None, end: None }] };
        let lib = vec![tree("plain", &[], &EX),
                       tree("sky", &[Has("self:status:wx-fair")], &EX)];
        let mut them = Speaker::new("Hob");
        them.status.push("wx-fair");
        let (self_set, any_set) = own_sets(&them);
        let t_reply = say(0, "plain", Some("a fine drying day"));
        let t_tree = say(0, "sky", None);
        assert!(!turn_keyed(&t_reply, &lib, &self_set, &any_set), "weather-gated reply");
        assert!(!turn_keyed(&t_tree, &lib, &self_set, &any_set), "weather-gated tree");
    }

    /// the_asking's secret answers: a tier telling from the fact they
    /// carry is own-voice; the topic's generic fallback is not
    #[test]
    fn an_authored_telling_is_theirs() {
        let lib = village_library();
        let mut them = Speaker::new("Hob");
        them.facts.hidden.push("ghost:levy".into());
        let (self_set, any_set) = own_sets(&them);
        let telling = crate::game::fact_content::FACT_CONTENTS.iter()
            .find(|c| c.fact == "ghost:levy").unwrap().tiers[0].1;
        let t_told = Turn { speaker: 0, act: Act::Answer, respond: None,
            text: "told".into(), concepts: vec![], tree: "the_asking",
            line: Some(telling), channel: crate::game::setting::Channel::Public };
        let t_flat = Turn { speaker: 0, act: Act::Answer, respond: None,
            text: "flat".into(), concepts: vec![], tree: "the_asking",
            line: Some("It's not for me to say."), channel: crate::game::setting::Channel::Public };
        assert!(turn_keyed(&t_told, &lib, &self_set, &any_set), "the levy's telling is theirs");
        assert!(!turn_keyed(&t_flat, &lib, &self_set, &any_set), "the flat answer is anybody's");
    }

    /// the metrics' own cast patch: a slip budget lets the leak
    /// reach the measure, and when it fires the share counts it -
    /// the channel live talk setup spends but `speaker_of` leaves
    /// unpatched
    #[test]
    fn a_budgeted_leak_reaches_the_share() {
        let lib = village_library();
        let you = Speaker::new("Wanderer");
        let mut saw = false;
        for seed in 0..20 {
            let mut them = Speaker::new("Hob");
            them.trade = Some("carter");
            them.slip = Some(crate::game::conversation::Slip {
                prone: 1.0,
                dead: Some(crate::game::conversation::Dead {
                    kin: "brother", name: "Corvin".into(), trade: "carter" }) });
            them.slips_left = 1;
            let (_, arc) = run_arc(&them, &you, Relation::Acquaintance, None, 20,
                Policy::Responsive, &lib, &STREET, seed);
            for t in &arc {
                for turn in &t.transcript.turns {
                    if turn.speaker == 0 && turn.act == Act::Slip {
                        saw = true;
                        let (self_set, any_set) = own_sets(&them);
                        assert!(turn_keyed(turn, &lib, &self_set, &any_set),
                            "the fired leak isn't own-voice");
                    }
                }
            }
        }
        assert!(saw, "a prone ghost never leaked in 20 arcs");
    }

    /// the share reads keyed/total over the villager's turns in the
    /// window - only `side`'s turns count, and `from` cuts the window
    #[test]
    fn the_share_counts_keyed_over_total() {
        static KEYED: Exchange = Exchange { act: Act::Remark, concepts: &[],
            calls: &["mine"], voice: &[], replies: &[] };
        static PLAIN: Exchange = Exchange { act: Act::Remark, concepts: &[],
            calls: &["theirs"], voice: &[], replies: &[] };
        let lib = vec![
            tree("keyed", &[Has("self:ghost:levy")], &KEYED),
            tree("plain", &[], &PLAIN),
        ];
        let mut them = Speaker::new("Hob");
        them.facts.hidden.push("ghost:levy".into());
        let arc = arc_at(vec![
            say(0, "keyed", Some("mine")),
            say(0, "plain", Some("theirs")),
            say(1, "keyed", Some("mine")),  // the bot's turn never counts
        ]);
        assert_eq!(own_voice(&arc, &lib, &them, 0, 0, 12), (1, 2));
        assert_eq!(own_voice(&arc, &lib, &them, 0, 0, 11), (0, 0));
        assert_eq!(own_voice(&arc, &lib, &them, 0, 11, 12), (1, 2));
    }

    /// the catalog ships whole: every ghost kind and every lie carries
    /// a tell, a telling and a pressure choice - 100% coverage
    #[test]
    fn the_shipped_facts_all_carry_the_three() {
        let (covered, shipped, missing) = rule_of_three();
        assert_eq!(shipped, 20);
        assert!(missing.is_empty(), "uncovered fact types: {missing:?}");
        assert_eq!(covered, shipped);
    }

    /// word pairs count the vocabulary's joints: repeats don't grow it,
    /// new phrasings do
    #[test]
    fn word_pairs_count_distinct_joints() {
        let few = word_pairs(&["the wind took the roof".into(),
            "the wind took the roof".into()]);
        let more = word_pairs(&["the wind took the roof".into(),
            "the rain filled the well".into()]);
        assert_eq!(few, 4);
        assert!(more > few);
    }

    /// Self-BLEU: a corpus against copies of itself reads 1.0; a corpus
    /// of fresh words reads near 0
    #[test]
    fn self_bleu_reads_overlap() {
        let same = vec![vec!["the well ran dry this year".into(); 3],
                        vec!["the well ran dry this year".into(); 3]];
        assert!(self_bleu(&same) > 0.99);
        let apart = vec![vec!["apples carts stones".into()],
                         vec!["winter mercy tallow".into()]];
        assert!(self_bleu(&apart) < 0.05);
    }

    /// a wounded villager's arc holds own-voice lines a blank one's
    /// never draws - the `tell:` tree is only theirs
    #[test]
    fn an_arc_reads_the_wound() {
        let lib = village_library();
        let you = Speaker::new("Wanderer");
        let mut wounded = Speaker::new("Hob");
        wounded.facts.hidden.push("ghost:levy".into());
        let m_wound = measure_arc(&wounded, &you, Relation::Acquaintance, None, 20,
            Policy::Responsive, &lib, &STREET, 7);
        let m_plain = measure_arc(&Speaker::new("Hob"), &you, Relation::Acquaintance,
            None, 20, Policy::Responsive, &lib, &STREET, 7);
        assert!(m_wound.own_early > m_plain.own_early,
            "the wound speaks: {:.3} vs {:.3}", m_wound.own_early, m_plain.own_early);
        assert!(m_wound.word_pairs > 0 && m_wound.bleu > 0.0);
    }

    /// the metrics are as deterministic as the arcs they read: same
    /// seed, same table
    #[test]
    fn a_seed_gives_one_table() {
        let lib = village_library();
        let them = Speaker::new("Hob");
        let you = Speaker::new("Wanderer");
        let a = measure_arc(&them, &you, Relation::Acquaintance, None, 20,
            Policy::Curious, &lib, &STREET, 7);
        let b = measure_arc(&them, &you, Relation::Acquaintance, None, 20,
            Policy::Curious, &lib, &STREET, 7);
        assert_eq!((a.own_early, a.word_pairs, a.bleu.to_bits()),
                   (b.own_early, b.word_pairs, b.bleu.to_bits()));
    }
}
