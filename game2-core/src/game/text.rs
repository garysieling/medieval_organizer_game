//! The realizer behind `fill()` (E4-1, #1403; Part 5 of
//! docs/plans/procedural-characters.md).
//!
//! One pass turns an authored template into a spoken line for one
//! speaker facing one listener. Today it owns the pair's slots -
//! `{other}`, `{trade}`, `{other_trade}` - and whatever the context's
//! `refs` carry (`{someone}`). A slot nobody owns stays literal, so the
//! fills that still own theirs (the campaign's `{issue}`, the meeting's
//! `{name}`) keep working.
//!
//! The grammar layers ride `Cx` from the start so later tickets don't
//! churn the signature: `Tv` and `archaism` land in E4-2 (#1414), the
//! lexicon and `refs`' body fields in E4-3 (#1408), `stance` and the
//! shaping operators in E4-5 (#1429).

use crate::game::conversation::{Rng, Speaker};
use crate::game::dialogue_trees::trade_noun;

/// The second person a line speaks in (Part 5): thou for the intimate
/// and the looked-down-on, you for the rest. `tv_for` (status gap,
/// intimacy, contempt) lands with E4-2; until then every line talks you.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tv {
    Thou,
    #[default]
    You,
}

/// How a line is held (Part 5): warmth toward the listener, guard over
/// what mustn't show, deference to their station, certainty in the thing
/// said. The shaping operators of E4-5 read it; until then it is the
/// neutral default.
#[derive(Debug, Clone, Copy, Default)]
pub struct Stance {
    pub warmth: f32,
    pub guard: f32,
    pub deference: f32,
    pub certainty: f32,
}

/// Who and what a line names beyond the pair: one entry per extra slot
/// the realizer may meet - a villager a tale is about (`{someone}`), an
/// issue a meeting weighs. `text` is what stands in the line; the body
/// fields are for the grammar layers (E4-3 reads sex for pronouns,
/// alive for tense, count for plurals).
#[derive(Debug, Clone)]
pub struct Referent {
    pub slot: &'static str,
    pub text: String,
    pub sex: Option<&'static str>,
    pub alive: bool,
    pub count: u32,
}

/// The realizing context for one line: who talks to whom, in which
/// person and stance, and what else the line may name. E4-2/E4-3/E4-5
/// fill the grammar fields in; until then callers build the default -
/// you, a neutral stance, modern wording.
pub struct Cx<'a> {
    pub speaker: &'a Speaker,
    pub listener: &'a Speaker,
    pub tv: Tv,
    pub stance: Stance,
    pub refs: &'a [Referent],
    /// 0 modern, 1 marked shifts, 2 the full thou-pattern (E4-2)
    pub archaism: u8,
}

impl<'a> Cx<'a> {
    /// The plain context: you-form, a neutral stance, modern wording.
    /// Slots beyond the pair's own resolve through `refs`.
    pub fn plain(speaker: &'a Speaker, listener: &'a Speaker, refs: &'a [Referent]) -> Self {
        Cx { speaker, listener, tv: Tv::You, stance: Stance::default(), refs, archaism: 0 }
    }
}

/// Render a template for the pair. One pass over the text: each `{slot}`
/// resolves against the pair or the refs, and a slot nobody owns stays
/// literal, so the fills that still own theirs (the campaign's
/// `{issue}`, a meeting's `{name}`) keep working downstream.
///
/// The rng is the talk's own stream - the grammar layers draw choices
/// (thou or you, a hedge or none) from it so a replayed talk re-says
/// the same words. E4-1 makes no draws yet.
pub fn realize(template: &str, cx: &Cx, _rng: &mut Rng) -> String {
    let mut out = String::with_capacity(template.len() + 8);
    let mut rest = template;
    while let Some(o) = rest.find('{') {
        out.push_str(&rest[..o]);
        let after = &rest[o + 1..];
        match after.find('}') {
            Some(c) => {
                push_slot(&mut out, &after[..c], cx);
                rest = &after[c + 1..];
            }
            None => {
                out.push_str(&rest[o..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// One slot's words: the pair's own names and trades, then the refs;
/// a slot nobody owns goes back in the line untouched.
fn push_slot(out: &mut String, slot: &str, cx: &Cx) {
    match slot {
        "other" => out.push_str(&cx.listener.name),
        "trade" => out.push_str(trade_noun(cx.speaker)),
        "other_trade" => out.push_str(trade_noun(cx.listener)),
        _ => match cx.refs.iter().find(|r| r.slot == slot) {
            Some(r) => out.push_str(&r.text),
            None => {
                out.push('{');
                out.push_str(slot);
                out.push('}');
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::conversation::Speaker;
    use crate::game::dialogue_trees::village_library;

    fn pair() -> (Speaker, Speaker) {
        let mut alys = Speaker::new("Alys");
        alys.status.push("miller");
        let corvin = Speaker::new("Corvin");
        (alys, corvin)
    }

    fn realize_plain(t: &str, speaker: &Speaker, listener: &Speaker, refs: &[Referent]) -> String {
        realize(t, &Cx::plain(speaker, listener, refs), &mut Rng::new(7))
    }

    #[test]
    fn pair_slots_fill_as_the_old_replaces_did() {
        let (alys, corvin) = pair();
        let t = "{other}, the mill keeps me - and how goes {other_trade}? My own is {trade}.";
        let got = realize_plain(t, &alys, &corvin, &[]);
        assert_eq!(got, "Corvin, the mill keeps me - and how goes the day's work? My own is the mill.");
    }

    #[test]
    fn refs_fill_the_extra_slots() {
        let (alys, corvin) = pair();
        let refs = [Referent { slot: "someone", text: "Old Nan".into(), sex: Some("female"), alive: true, count: 1 }];
        let got = realize_plain("{someone}? Lovely sort.", &alys, &corvin, &refs);
        assert_eq!(got, "Old Nan? Lovely sort.");
    }

    #[test]
    fn a_slot_nobody_owns_stays_for_the_next_fill() {
        let (alys, corvin) = pair();
        let got = realize_plain("{other}, about {issue}: I say {name}.", &alys, &corvin, &[]);
        assert_eq!(got, "Corvin, about {issue}: I say {name}.");
        // and a brace that never closes renders as it always did
        assert_eq!(realize_plain("an open {", &alys, &corvin, &[]), "an open {");
    }

    /// The done-when: every authored line the library holds renders
    /// through the realizer exactly as the old replace chain made it
    fn all_library_lines() -> Vec<&'static str> {
        fn gather<'a>(ex: &'a crate::game::conversation::Exchange, out: &mut Vec<&'static str>) {
            out.extend(ex.calls.iter().copied());
            for v in ex.voice { out.extend(v.lines.iter().copied()); }
            for r in ex.replies {
                out.extend(r.lines.iter().copied());
                for v in r.voice { out.extend(v.lines.iter().copied()); }
                if let Some(then) = &r.then { gather(then, out); }
            }
        }
        let mut all = vec![];
        for tree in village_library() {
            gather(tree.root, &mut all);
            if let Some(rejoin) = tree.rejoin { gather(rejoin, &mut all); }
        }
        for t in crate::game::dialogue_trees::ASK_TOPICS {
            all.extend(t.tell.iter().copied());
        }
        all
    }

    #[test]
    fn every_library_line_renders_as_the_old_fills() {
        let (alys, corvin) = pair();
        let mut rng = Rng::new(3);
        let mut n = 0;
        for t in all_library_lines() {
            let want = t.replace("{other}", "Corvin")
                .replace("{trade}", trade_noun(&alys))
                .replace("{other_trade}", trade_noun(&corvin));
            assert_eq!(realize(t, &Cx::plain(&alys, &corvin, &[]), &mut rng), want, "line: {t}");
            n += 1;
        }
        // the sweep must actually reach the corpus, not vacuously pass
        assert!(n > 1500, "only {n} lines swept");
    }

    #[test]
    fn no_pair_slot_survives_the_pass() {
        let (alys, corvin) = pair();
        let mut rng = Rng::new(3);
        let refs = [Referent { slot: "someone", text: "Old Nan".into(), sex: Some("female"), alive: true, count: 1 }];
        for t in all_library_lines() {
            let got = realize(t, &Cx::plain(&alys, &corvin, &refs), &mut rng);
            for slot in ["{other}", "{trade}", "{other_trade}", "{someone}"] {
                assert!(!got.contains(slot), "{slot} left in: {t}");
            }
        }
    }
}
