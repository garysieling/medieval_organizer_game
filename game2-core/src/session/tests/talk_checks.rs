//! E0-3 (#1402): structural talk checks over the `dialogue_gen`
//! corpus (Appendix A's talk scale) - adjacency pairs, cascade
//! order, no repeats, endings. The rubric items code can score,
//! as assertions that fail CI; what needs a reader stays with
//! the #1415 judge.

use crate::game::conversation::{act_fit, Act, Ending};
use crate::game::talk_gen::{gen_conversations, GenCorpus};

/// The seeded corpora `dialogue_gen --json` serializes: enough talks
/// that the structural checks see the cascade's range
fn corpus() -> Vec<GenCorpus> {
    [42, 7, 1337].iter().map(|&s| gen_conversations(12, 8, s)).collect()
}

fn turn_ctx(t: &crate::game::conversation::Transcript, i: usize) -> String {
    let lo = i.saturating_sub(1);
    let hi = (i + 2).min(t.turns.len());
    t.turns[lo..hi].iter().map(|u| format!("{:?}({})", u.act, u.speaker))
        .collect::<Vec<_>>().join(" -> ")
}

#[test]
fn test_a_call_gets_a_fitting_reply() {
    // adjacency pairs (rubric 1): in converse's ping-pong the initiator
    // speaks every call and the other answers, so a 0->1 turn pair is a
    // call and its reply - act_fit is the production coherence rule
    // (a question its answer, a goodbye its goodbye, a remark anything
    // but an early farewell). Reply->next-call boundaries are free:
    // an unanswered dodge or a counter-question is an insertion
    // sequence, not a non-sequitur.
    let mut bad = vec![];
    for g in corpus().iter().flat_map(|c| &c.talks) {
        let t = &g.transcript;
        for (i, w) in t.turns.windows(2).enumerate() {
            if w[0].speaker == 0 && w[1].speaker == 1 && act_fit(w[0].act, w[1].act) == 0 {
                bad.push(format!("[{}] {:?} -> {:?} at {}\n   {:?} <- {:?}",
                    t.speakers[1], w[0].act, w[1].act,
                    turn_ctx(t, i), w[1].text, w[0].text));
            }
        }
    }
    assert!(bad.is_empty(), "non-sequitur replies:\n{}", bad.join("\n"));
}

#[test]
fn test_the_cascade_degrades_and_never_returns() {
    // the specificity cascade (rubric 3): a talk opens on a greet,
    // walks each matched tree once, and never steps back to a tree it
    // already left - the generic smalltalk tail, when it runs, runs
    // last
    let mut bad = vec![];
    for g in corpus().iter().flat_map(|c| &c.talks) {
        let t = &g.transcript;
        // opens on an opener: a greet, or a cold-open remark or
        // question (a sweetheart needn't say hello) - never an
        // answering act with nothing to answer
        if !matches!(t.turns.first().map(|u| u.act),
            Some(Act::Greet) | Some(Act::Remark) | Some(Act::Question)) {
            bad.push(format!("{}: opens on {:?}", t.speakers[0],
                t.turns.first().map(|u| u.act)));
        }
        for (i, tree) in t.trees_walked.iter().enumerate() {
            if t.trees_walked[..i].contains(tree) {
                bad.push(format!("{}: re-walked {tree} in {:?}",
                    t.speakers[0], t.trees_walked));
            }
        }
        if let Some(pos) = t.trees_walked.iter().position(|n| *n == "village_smalltalk") {
            if pos != t.trees_walked.len() - 1 {
                bad.push(format!("{}: smalltalk mid-walk in {:?}",
                    t.speakers[0], t.trees_walked));
            }
        }
        let mut deepest = 0usize;
        for (i, u) in t.turns.iter().enumerate() {
            match t.trees_walked.iter().position(|n| *n == u.tree) {
                Some(idx) => {
                    if idx < deepest {
                        bad.push(format!("{}: turn {} on {tree} after {:?}",
                            t.speakers[0], i, t.trees_walked,
                            tree = u.tree));
                    }
                    deepest = deepest.max(idx);
                }
                None => bad.push(format!("{}: turn {} tree {} never walked in {:?}",
                    t.speakers[0], i, u.tree, t.trees_walked)),
            }
        }
    }
    assert!(bad.is_empty(), "cascade faults:\n{}", bad.join("\n"));
}

#[test]
fn test_no_line_repeats_within_a_talk() {
    // exclusivity (rubric 4): a spent line never returns - neither the
    // rendered text nor the template it was filled from
    let mut bad = vec![];
    for g in corpus().iter().flat_map(|c| &c.talks) {
        let t = &g.transcript;
        let mut texts = std::collections::BTreeSet::new();
        let mut templates = std::collections::BTreeSet::new();
        for u in &t.turns {
            if !texts.insert(&u.text) {
                bad.push(format!("{}: repeats {:?}", t.speakers[0], u.text));
            }
            if let Some(l) = u.line {
                if !templates.insert(l) {
                    bad.push(format!("{}: re-spends template {l:?}", t.speakers[0]));
                }
            }
        }
    }
    assert!(bad.is_empty(), "repeats:\n{}", bad.join("\n"));
}

#[test]
fn test_every_talk_ends_in_bounds() {
    // endings (rubric 5, and item 1's tail): every talk ends, inside a
    // sane length, and closes the way its ending says - a parting ends
    // on a farewell line, a walk-off on the pushback that gave offence
    let mut bad = vec![];
    for g in corpus().iter().flat_map(|c| &c.talks) {
        let t = &g.transcript;
        let last = t.turns.last().map(|u| u.act);
        match t.ending {
            Some(Ending::Farewell) if last == Some(Act::Farewell) => {}
            Some(Ending::WalksOff) if last == Some(Act::PushBack) => {}
            Some(Ending::Exhausted) => {}
            Some(e) => bad.push(format!("{}: {:?} ends on {:?} at {}",
                t.speakers[0], e, last, turn_ctx(t, t.turns.len() - 1))),
            None => bad.push(format!("{}: no ending at {} turns",
                t.speakers[0], t.turns.len())),
        }
        if !(2..=24).contains(&t.turns.len()) {
            bad.push(format!("{}: {} turns, out of bounds",
                t.speakers[0], t.turns.len()));
        }
    }
    assert!(bad.is_empty(), "ending faults:\n{}", bad.join("\n"));
}

/// E5-1's done-when (#1430): the reveal acts are real content, not
/// enum furniture - worry, bid, joke, errand, evade and slip each
/// reach a transcript in the seeded corpus
#[test]
fn test_each_reveal_act_appears_in_the_corpus() {
    let mut seen = crate::engine::det::HashSet::default();
    for g in corpus().iter().flat_map(|c| &c.talks) {
        for t in &g.transcript.turns { seen.insert(t.act); }
    }
    for a in [Act::Worry, Act::Bid, Act::Joke, Act::Errand, Act::Evade, Act::Slip] {
        assert!(seen.contains(&a), "{:?} never appears in the corpus", a);
    }
}
