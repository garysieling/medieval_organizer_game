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

use crate::game::conversation::{Relation, Rng, Speaker};
use crate::game::dialogue_trees::trade_noun;

/// The second person a line speaks in (Part 5): thou for the intimate
/// and the looked-down-on, you for the rest. `tv_for` computes it per
/// pair from the power semantic, gated by the speaker's archaism.
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
/// person and stance, the bond they speak across, and what else the
/// line may name. E4-3/E4-5 fill the remaining fields in.
pub struct Cx<'a> {
    pub speaker: &'a Speaker,
    pub listener: &'a Speaker,
    /// How the speaker addresses this listener - `tv_for` sets it on the
    /// Speaker at pair build; `Cx::plain` carries it through
    pub tv: Tv,
    /// The bond the speaker speaks across - `{addr}` reads its intimacy
    pub rel: Relation,
    pub stance: Stance,
    pub refs: &'a [Referent],
    /// 0 modern, 1 marked shifts, 2 the full thou-pattern (E4-2)
    pub archaism: u8,
}

impl<'a> Cx<'a> {
    /// The context the pair carries: the speaker's own `tv`, `rel` and
    /// `archaism` (patched at pair build, or the `Speaker::new`
    /// defaults), a neutral stance. Slots beyond the pair's own resolve
    /// through `refs`.
    pub fn plain(speaker: &'a Speaker, listener: &'a Speaker, refs: &'a [Referent]) -> Self {
        Cx { speaker, listener, tv: speaker.tv, rel: speaker.rel,
            stance: stance_of(speaker, listener), refs, archaism: speaker.archaism }
    }
}

/// The stance a pair implies (E4-5): warmth off the bond's openness
/// (E3-2's `Open`, patched per pair), guard off the speaker's traits,
/// deference off the status gap, certainty off their pull. A guarded
/// speaker's no runs long and soft; a proud, sure one's stays short.
pub fn stance_of(speaker: &Speaker, listener: &Speaker) -> Stance {
    let open = &speaker.open;
    let warmth = ((open.benevolence + open.intimacy + open.mood) / 3.0 + 0.3)
        .clamp(0.0, 1.0);
    let mut guard = 0.4 - open.safety.clamp(0.0, 1.0) * 0.3;
    for t in &speaker.traits {
        guard += match t {
            crate::game::traits::Trait::Wary => 0.35,
            crate::game::traits::Trait::Stolid => 0.2,
            crate::game::traits::Trait::SelfAbsorbed => 0.1,
            crate::game::traits::Trait::Gregarious | crate::game::traits::Trait::Kind => -0.15,
            crate::game::traits::Trait::Hotheaded => -0.25,
            crate::game::traits::Trait::Proud => -0.3,
            _ => 0.0,
        };
    }
    let certainty = (speaker.pull.max(0.0) * 0.6
        + if speaker.traits.contains(&crate::game::traits::Trait::Proud) { 0.3 } else { 0.0 }
        + if speaker.traits.contains(&crate::game::traits::Trait::Steady) { 0.1 } else { 0.0 })
        .clamp(0.0, 1.0);
    Stance {
        warmth,
        guard: (guard - certainty * 0.2).clamp(0.0, 1.0),
        deference: ((station(listener) - station(speaker)) as f32 / 3.0 + 0.5)
            .clamp(0.0, 1.0),
        certainty,
    }
}

/// The shaping operators a dispreferred response draws (E4-5, Part 5):
/// conversation analysis' delay, preface, hedge and account, applied
/// as the stance asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// "Oh…", "Well…" - the beat before a hard thing
    Delay,
    /// "It isn't that I wouldn't." - softening ahead of the no
    Preface,
    /// "maybe", "I'd think" - the edge taken off the words
    Hedge,
    /// the refusal's reason said aloud, when one is known
    Account,
}

/// The ops an act and stance call for. Dispreferred responses
/// (push-backs; refusals passed as `PushBack`) pad as guard rises: a
/// guarded no carries delay, preface, hedge and whatever `account` the
/// caller knows, while a proud, certain no stays bare. An `Agree`
/// from a guarded, cold speaker pads the same way - the insincere yes
/// an attentive player shouldn't trust (heuristic, not event-driven).
pub fn shape(act: crate::game::conversation::Act, stance: &Stance, account: Option<&str>) -> Vec<Op> {
    use crate::game::conversation::Act;
    let g = match act {
        Act::PushBack => stance.guard,
        Act::Agree => {
            if stance.warmth >= 0.4 { return vec![]; }
            stance.guard * (1.0 - stance.warmth)
        }
        _ => return vec![],
    };
    let mut ops = vec![];
    if g >= 0.3 { ops.push(Op::Delay); }
    if g >= 0.45 { ops.push(Op::Preface); }
    if g >= 0.6 {
        if account.is_some() { ops.push(Op::Account); }
        ops.push(Op::Hedge);
    }
    ops
}

const DELAYS: [&str; 4] = ["Oh…", "Well…", "Hm…", "Ah…"];
const PREFACES: [&str; 3] = [
    "It isn't that I wouldn't.",
    "I hear you.",
    "There's that to think on.",
];
const PREFACES_DEFERENT: [&str; 2] = [
    "I wouldn't presume to say.",
    "If it please you to hear it.",
];
const HEDGES: [&str; 3] = ["maybe.", "I'd think.", "I suppose."];

/// The line the ops build around `line`: delay and preface ahead of
/// it, the account and hedge after. `account` is the reason the caller
/// knows ("the reeve counts heads at the gate"); an `Op::Account`
/// without one renders nothing. Draws come from the talk's rng so a
/// replayed talk re-says the same words.
pub fn shaped(ops: &[Op], line: &str, account: Option<&str>, stance: &Stance, rng: &mut Rng) -> String {
    if ops.is_empty() { return line.to_string(); }
    let mut out = String::new();
    if ops.contains(&Op::Delay) {
        out.push_str(DELAYS[rng.below(DELAYS.len())]);
        out.push(' ');
    }
    if ops.contains(&Op::Preface) {
        let prefaces = if stance.deference > 0.6 { &PREFACES_DEFERENT[..] } else { &PREFACES[..] };
        out.push_str(prefaces[rng.below(prefaces.len())]);
        out.push(' ');
    }
    out.push_str(line);
    if ops.contains(&Op::Account) {
        if let Some(reason) = account {
            out.push_str(&format!(" Only {reason}."));
        }
    }
    if ops.contains(&Op::Hedge) {
        out.push(' ');
        out.push_str(HEDGES[rng.below(HEDGES.len())]);
    }
    out
}

/// The six style markers a speaker's temperament sets (E4-6, #1433;
/// PERSONAGE's parameters ported lightly): which of them a line rolls
/// is the speaker's own signature. Hard cap two a line; most lines
/// stay plain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// "and that's the long of it" - the line runs on past its need
    Verbose,
    /// "or so I hear" - the edge off a plain statement
    Hedge,
    /// a hot line ends on the bang of it
    Exclaim,
    /// "or am I mistook?" - the question flung back at the room
    AskBack,
    /// "for my part" - the self held up beside the matter
    SelfTalk,
    /// "saints keep us" - the oath that opens their mouth
    Oath,
}
const MARKERS: [Marker; 6] = [Marker::Verbose, Marker::Hedge,
    Marker::Exclaim, Marker::AskBack, Marker::SelfTalk, Marker::Oath];

/// The six marker strengths a speaker's traits imply (E4-6 reads the
/// trait layer, not the Big Five floats - those integrate when E1-4's
/// persona-derived values land). Stolid damps every marker.
fn style_strengths(speaker: &Speaker) -> [f32; 6] {
    use crate::game::traits::Trait;
    let mut s = [0.0f32; 6];
    for t in &speaker.traits {
        match t {
            Trait::Gregarious => { s[0] += 0.30; s[2] += 0.15; s[3] += 0.20; }
            Trait::Kind => { s[0] += 0.15; s[1] += 0.10; }
            Trait::Wary => { s[1] += 0.35; }
            Trait::Stolid => for v in s.iter_mut() { *v -= 0.30; }
            Trait::Hotheaded => { s[2] += 0.40; }
            Trait::Cruel => { s[2] += 0.15; }
            Trait::SelfAbsorbed => { s[3] += 0.30; s[4] += 0.45; }
            Trait::Proud => { s[3] += 0.15; s[4] += 0.20; }
            Trait::Ambitious => { s[0] += 0.10; s[4] += 0.15; }
            Trait::Pious => { s[5] += 0.50; }
            Trait::Steady => { s[1] += 0.10; s[5] += 0.10; }
            Trait::Lazy => { s[0] -= 0.10; }
            Trait::Stingy => { s[4] += 0.10; }
        }
    }
    s.map(|v| v.clamp(0.0, 1.0))
}

/// The seed a marker pass runs on: the line and whose mouth it's in -
/// so a line renders the same markers wherever it's met (a preview and
/// its saying agree), and no draw is owed the walk's own stream.
fn marker_seed(template: &str, speaker: &Speaker) -> u64 {
    let hash = |s: &str| {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for b in s.bytes() { h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3); }
        h
    };
    hash(template) ^ hash(&speaker.name)
}

const OATH_OPENERS: [&str; 3] = ["Saints keep us. ", "By the rood. ", "God witness. "];
const EXCLAIM_OPENERS: [&str; 2] = ["Hah! ", "Why, "];
const VERBOSE_TAILS: [&str; 3] = [", and that's the long of it.", ", and more besides.", ", as I've cause to say."];
const HEDGE_TAILS: [&str; 3] = [", or so I hear.", ", as like as not.", ", by my reckoning."];
const ASKBACK_TAILS: [&str; 3] = [", or am I mistook?", ", or is that not the way of it?", ", or do I hear wrong?"];
const SELFTALK_TAILS: [&str; 3] = [", for my part.", ", as I see it.", ", or so I make it."];

/// The line with the speaker's markers on it: each fires as its
/// strength asks, two at most, so a pious mouth swears and the
/// self-absorbed one can't leave themselves out of it. Runs inside
/// `realize`, so a guarded no still wears them under its shape (E4-5's
/// ops wrap the marked line).
fn marked(text: &str, template: &str, speaker: &Speaker) -> String {
    let strengths = style_strengths(speaker);
    let max_s = strengths.iter().copied().fold(0.0f32, f32::max);
    if max_s <= 0.0 { return text.to_string(); }
    // one gate per line - the strongest signature decides whether the
    // line is signed at all; even the fullest mouth stays plain on
    // most of what it says
    let mut r = Rng::new(marker_seed(template, speaker));
    if r.next_f32() >= max_s * 0.6 { return text.to_string(); }
    let first = r.weighted(&strengths);
    let mut fired = vec![MARKERS[first]];
    // a second marker rides along only as the rest of the style is hot
    let mut rest = strengths;
    rest[first] = 0.0;
    let max2 = rest.iter().copied().fold(0.0f32, f32::max);
    if max2 > 0.0 && r.next_f32() < max2 * 0.5 {
        fired.push(MARKERS[r.weighted(&rest)]);
    }
    let mut out = text.to_string();
    if fired.contains(&Marker::Oath) {
        out = format!("{}{}", OATH_OPENERS[r.below(OATH_OPENERS.len())], out);
    }
    if fired.contains(&Marker::Exclaim) {
        if out.ends_with('.') {
            out.replace_range(out.len() - 1.., "!");
        } else {
            out = format!("{}{}", EXCLAIM_OPENERS[r.below(EXCLAIM_OPENERS.len())], out);
        }
    }
    let tail = |out: &mut String, set: &[&'static str], r: &mut Rng| {
        // a tail joins on a comma; a line without its stop is left be
        if out.ends_with('.') {
            out.replace_range(out.len() - 1.., set[r.below(set.len())]);
        }
    };
    for m in &fired {
        match m {
            Marker::Verbose => tail(&mut out, &VERBOSE_TAILS, &mut r),
            Marker::Hedge => tail(&mut out, &HEDGE_TAILS, &mut r),
            Marker::AskBack => tail(&mut out, &ASKBACK_TAILS, &mut r),
            Marker::SelfTalk => tail(&mut out, &SELFTALK_TAILS, &mut r),
            _ => {}
        }
    }
    out
}

/// Render a template for the pair. One pass over the text: each `{slot}`
/// resolves against the pair or the refs, and a slot nobody owns stays
/// literal, so the fills that still own theirs (the campaign's
/// `{issue}`, a meeting's `{name}`) keep working downstream.
///
/// The rng is the talk's own stream - the grammar layers draw choices
/// (thou or you, a hedge or none) from it so a replayed talk re-says
/// the same words. The style markers (E4-6) seed off the line and the
/// speaker instead: no draw comes out of the walk's stream, and a
/// preview shows the marks the saying wears.
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
    marked(&out, template, cx.speaker)
}

/// One slot's words: the pair's own names and trades, the second-person
/// and address slots of the grammar layer, then the refs; a slot nobody
/// owns goes back in the line untouched.
fn push_slot(out: &mut String, slot: &str, cx: &Cx) {
    // a slot opening a sentence answers with a capital
    let cap = out.is_empty()
        || matches!(out.trim_end().chars().last(), Some('.' | '!' | '?'));
    let push = |out: &mut String, word: &str| {
        if cap {
            let mut ch = word.chars();
            if let Some(c) = ch.next() {
                out.extend(c.to_uppercase());
                out.push_str(ch.as_str());
                return;
            }
        }
        out.push_str(word);
    };
    match slot {
        // the legacy fills replace verbatim - no cap, or a trade
        // after a full stop would render "The mill" where the old
        // chain left "the mill"
        "other" => out.push_str(&cx.listener.name),
        "self" => out.push_str(&cx.speaker.name),
        "trade" => out.push_str(&trade_noun(cx.speaker)),
        "other_trade" => out.push_str(&trade_noun(cx.listener)),
        // E4-2: the address a relation earns - "m'lord", "goodwife Maud"
        "addr" => push(out, &address_form(cx)),
        // E4-2: second person - pronouns and verb agreement read `cx.tv`
        _ if slot.starts_with("2:") => push(out, &second_person(&slot[2..], cx.tv)),
        // E4-4: second person plural - a room, a camp, the whole side;
        // ye is the plural of thou, so the pair's tv picks it the way
        // the singular path picks thee; verbs stay bare either way
        _ if slot.starts_with("2p:") => push(out, &second_person_plural(&slot[3..], cx.tv)),
        // E4-3: the lexicon - third person, tense, number and lists,
        // each tail a referent slot ("{he:someone}", "{n:fingers}")
        _ if slot.starts_with("he:") => push(out, &third_person(&slot[3..], cx, Case::Subj)),
        _ if slot.starts_with("him:") => push(out, &third_person(&slot[4..], cx, Case::Obj)),
        _ if slot.starts_with("his:") => push(out, &third_person(&slot[4..], cx, Case::Poss)),
        _ if slot.starts_with("is:") => push(out, &copula(&slot[3..], cx, false)),
        _ if slot.starts_with("was:") => push(out, &copula(&slot[4..], cx, true)),
        _ if slot.starts_with("n:") => match named_ref(&slot[2..], cx) {
            Some(r) => push(out, &format!("{} {}", number_word(r.count), plural(&r.text))),
            None => keep(out, slot),
        },
        _ if slot.starts_with("a:") => {
            let noun = named_ref(&slot[2..], cx).map(|r| r.text.clone())
                .unwrap_or_else(|| slot[2..].to_string());
            push(out, &format!("{} {}", article(&noun), noun));
        }
        _ if slot.starts_with("list:") => match named_ref(&slot[5..], cx) {
            Some(r) => push(out, &list_join(&r.text)),
            None => keep(out, slot),
        },
        _ => match cx.refs.iter().find(|r| r.slot == slot) {
            Some(r) => push(out, &r.text),
            None => {
                out.push('{');
                out.push_str(slot);
                out.push('}');
            }
        },
    }
}

/// The station a speaker's standing implies (Brown & Gilman's power
/// axis): the manor over the church over the moneyed over the common
/// sort; a child sits under all of it
fn station(s: &Speaker) -> i8 {
    if s.status.contains(&"child") { return -1; }
    match s.ideology {
        "lord" => 3,
        "clergy" => 2,
        "merchant" => 1,
        _ => 0,
    }
}

/// Thou or you for this speaker to this listener (E4-2, #1414; Brown
/// and Gilman's power semantic): superiors give thou and receive you;
/// peers keep thou for kin and friends and give strangers you.
/// `contempt` marks the deliberate slight - thou where you is owed,
/// which a rival's talk carries. The speaker's own archaism gates the
/// pattern: 0 is modern you always, 1 keeps thou for the marked shifts
/// (intimacy, contempt), 2 lets bare rank give thou too. Prayers are
/// the caller's business - God takes thou whatever this returns.
pub fn tv_for(speaker: &Speaker, listener: &Speaker, rel: Relation, contempt: bool) -> Tv {
    if speaker.archaism == 0 { return Tv::You; }
    let intimate = matches!(rel, Relation::Kin | Relation::Friend);
    if intimate || contempt { return Tv::Thou; }
    if speaker.archaism >= 2 && station(speaker) > station(listener) { return Tv::Thou; }
    Tv::You
}

/// What `{addr}` stands in for (E4-2): the courtesy the listener's
/// station, body and bond earn in the speaker's mouth
fn address_form(cx: &Cx) -> String {
    let l = cx.listener;
    let name = l.name.as_str();
    let female = l.sex == Some("female");
    let close = matches!(cx.rel, Relation::Kin | Relation::Friend);
    match station(l) {
        // the high-born get their title - from intimates, their name
        3.. => return if close { name.into() } else if female { "m'lady".into() } else { "m'lord".into() },
        2 => return if female { format!("sister {name}") } else { "father".into() },
        _ => {}
    }
    if l.status.contains(&"child") {
        return if close { name.into() } else if female { "lass".into() } else { "lad".into() };
    }
    if close {
        return match cx.rel {
            Relation::Kin => name.into(),
            _ => format!("friend {name}"),
        };
    }
    if matches!(cx.rel, Relation::Rival) { return name.into(); }
    if l.status.contains(&"elder") {
        return if female { format!("goody {name}") } else { format!("gaffer {name}") };
    }
    // a caller off the road is a traveler, and the village's own
    // shopkeeper a trader - the named courtesy, not a bare title (#1371)
    if l.status.contains(&"visitor") { return "traveler".into(); }
    if l.ideology == "merchant" { return "trader".into(); }
    // a child minds their betters however they stand
    let courtesy = if station(l) == 1 || cx.speaker.status.contains(&"child") {
        if female { format!("mistress {name}") } else { format!("master {name}") }
    } else if female {
        format!("goodwife {name}")
    } else if l.sex == Some("male") {
        format!("goodman {name}")
    } else {
        name.into()
    };
    courtesy
}

/// The word a `{2:…}` slot stands in for, in the form `tv` asks for:
/// pronouns straight, verbs agreeing - "art"/"are", "hast"/"have",
/// "sayest"/"say".
fn second_person(word: &str, tv: Tv) -> String {
    let thou = tv == Tv::Thou;
    match word {
        "subj" => if thou { "thou" } else { "you" }.into(),
        "obj" => if thou { "thee" } else { "you" }.into(),
        "poss" => if thou { "thy" } else { "your" }.into(),
        "own" => if thou { "thine" } else { "yours" }.into(),
        "self" => if thou { "thyself" } else { "yourself" }.into(),
        w if thou => thou_form(w),
        w => you_form(w),
    }
}

/// The word a `{2p:…}` slot stands in for (E4-4): the plural second
/// person a line speaks to a room or a side. Ye is the plural of
/// thou, so the pair's tv picks it exactly as the singular path picks
/// thee; objects and possessives are "you"/"your" in both. Verbs
/// after a plural subject keep the bare form ("ye go"), so only the
/// pronouns need slots.
fn second_person_plural(word: &str, tv: Tv) -> String {
    match word {
        "subj" => if tv == Tv::Thou { "ye" } else { "you" },
        "obj" => "you",
        "poss" => "your",
        "own" => "yours",
        "self" => "yourselves",
        w => w,
    }.to_string()
}

/// A verb's you-form: the lemma itself, save the copula and the
/// auxiliaries that inflect for person - "are", "have", "do".
fn you_form(verb: &str) -> String {
    match verb {
        "be" | "is" | "am" | "are" => "are",
        "was" | "were" => "were",
        "has" | "have" => "have",
        "does" | "do" => "do",
        v => v,
    }.to_string()
}

/// A slot nobody owns goes back in the line untouched (E4-3: the lexicon
/// slots share the rule with the rest)
fn keep(out: &mut String, slot: &str) {
    out.push('{');
    out.push_str(slot);
    out.push('}');
}

/// The referent a lexicon slot's tail names (E4-3): first the talk's
/// refs, then the pair's own words - "other" is the listener,
/// "self"/"speaker" the speaker - so `{he:other}` says "she" of the
/// one faced. Unknown tails find no referent and stay literal.
fn named_ref<'a>(tail: &str, cx: &'a Cx<'a>) -> Option<Referent> {
    if let Some(r) = cx.refs.iter().find(|r| r.slot == tail) {
        return Some(r.clone());
    }
    match tail {
        "other" => Some(Referent { slot: "other", text: cx.listener.name.clone(),
            sex: cx.listener.sex, alive: true, count: 1 }),
        "self" | "speaker" => Some(Referent { slot: "self", text: cx.speaker.name.clone(),
            sex: cx.speaker.sex, alive: true, count: 1 }),
        _ => None,
    }
}

/// Nominative / object / possessive, for the third-person slots
enum Case { Subj, Obj, Poss }

/// The third-person pronoun of the referent the tail names (E4-3): the
/// ref's `sex` picks he/she; a group - or a body of unrecorded sex -
/// takes they
fn third_person(tail: &str, cx: &Cx, case: Case) -> String {
    let Some(r) = named_ref(tail, cx) else { return format!("{{{}:{}}}",
        match case { Case::Subj => "he", Case::Obj => "him", Case::Poss => "his" }, tail) };
    let word = if r.count > 1 || r.sex.is_none() {
        match case { Case::Subj => "they", Case::Obj => "them", Case::Poss => "their" }
    } else if r.sex == Some("female") {
        match case { Case::Subj => "she", Case::Obj => "her", Case::Poss => "her" }
    } else {
        match case { Case::Subj => "he", Case::Obj => "him", Case::Poss => "his" }
    };
    word.into()
}

/// The copula the referent's count and aliveness ask for (E4-3):
/// `{is:x}` is present of the living, past of the dead ("is" of a dead
/// man is the fault this prevents); `{was:x}` stays past either way
fn copula(tail: &str, cx: &Cx, past: bool) -> String {
    let Some(r) = named_ref(tail, cx) else { return format!("{{{}:{}}}",
        if past { "was" } else { "is" }, tail) };
    let past = past || !r.alive;
    match (past, r.count > 1) {
        (false, false) => "is",
        (false, true) => "are",
        (true, false) => "was",
        (true, true) => "were",
    }.into()
}

/// "a" or "an" for the word that follows (E4-3): vowel sounds take an,
/// save the closed list - an honest man, an heir, but a unicorn and a
/// one-eyed dog. Absorbs `vocation::article`.
pub fn article(word: &str) -> &'static str {
    let w = word.trim_start().to_lowercase();
    const AN: &[&str] = &["honest", "honor", "honour", "heir", "herb", "hour"];
    const A: &[&str] = &["unicorn", "unique", "use", "user", "eulogy", "ewe"];
    // the short a-words need a word's end after them, or "one" would
    // eat "onset"
    const A_WORD: &[&str] = &["one", "once"];
    let word_end = |x: &str| w.starts_with(x) && (w.len() == x.len()
        || !w.as_bytes()[x.len()].is_ascii_alphabetic());
    if A_WORD.iter().any(|x| w.len() >= x.len() && word_end(x)) { return "a"; }
    if A.iter().any(|x| w.starts_with(x)) { return "a"; }
    if AN.iter().any(|x| w.starts_with(x)) { return "an"; }
    match w.chars().next() {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    }
}

/// The closed lexicon's irregular plurals (E4-3): bodies and folk,
/// kith and house, beasts and field - what the four rules can't reach.
/// Kept at module scope so the tests can exercise every entry.
const IRREG: &[(&str, &str)] = &[
    // bodies and folk
    ("man", "men"), ("woman", "women"), ("child", "children"),
    ("lad", "lads"), ("lass", "lasses"), ("foot", "feet"),
    ("tooth", "teeth"), ("goose", "geese"), ("mouse", "mice"),
    ("louse", "lice"), ("die", "dice"), ("penny", "pence"),
    // kith and house
    ("wife", "wives"), ("life", "lives"), ("knife", "knives"),
    ("self", "selves"), ("calf", "calves"), ("half", "halves"),
    ("loaf", "loaves"), ("leaf", "leaves"), ("sheaf", "sheaves"),
    ("thief", "thieves"), ("wolf", "wolves"), ("shelf", "shelves"),
    ("wreath", "wreaths"), ("brother", "brethren"),
    // beasts and field
    ("ox", "oxen"), ("sheep", "sheep"), ("swine", "swine"),
    ("deer", "deer"), ("fish", "fish"), ("folk", "folk"),
    ("cow", "kye"),
    // -f words that keep the f: the -ves rule would spoil them
    ("roof", "roofs"), ("cliff", "cliffs"), ("staff", "staffs"),
    ("chief", "chiefs"), ("belief", "beliefs"), ("proof", "proofs"),
];

/// The plural of a village noun (E4-3): the closed lexicon's irregulars
/// first, then the four suffix rules - sibilant -es, consonant+y -ies,
/// f/fe -ves, plain -s
pub fn plural(noun: &str) -> String {
    let w = noun.trim_end().to_lowercase();
    if let Some((_, p)) = IRREG.iter().find(|(s, _)| *s == w) {
        return p.to_string();
    }
    let chars: Vec<char> = w.chars().collect();
    let last = chars.last().copied().unwrap_or(' ');
    let before = chars.get(chars.len().saturating_sub(2)).copied().unwrap_or(' ');
    let vowel = |c: char| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u');
    if matches!(last, 's' | 'x' | 'z') || w.ends_with("sh") || w.ends_with("ch") {
        // the sibilants take -es: bench, ash, fox, mass
        format!("{w}es")
    } else if last == 'y' && !vowel(before) {
        // consonant+y drops the y: berry, story, fly
        format!("{}ies", &w[..w.len() - 1])
    } else if w.ends_with("fe") {
        // -fe lets the f go voiced: wife, knife, life (the table
        // holds the ones a village says)
        format!("{}ves", &w[..w.len() - 2])
    } else if last == 'f' {
        format!("{}ves", &w[..w.len() - 1])
    } else {
        format!("{w}s")
    }
}

/// A small count as a word (E4-3): one to twenty written out, anything
/// past that as digits - "three men", "23 sheaves"
pub fn number_word(n: u32) -> String {
    const WORDS: &[&str] = &["no", "one", "two", "three", "four", "five",
        "six", "seven", "eight", "nine", "ten", "eleven", "twelve",
        "thirteen", "fourteen", "fifteen", "sixteen", "seventeen",
        "eighteen", "nineteen", "twenty"];
    match WORDS.get(n as usize) {
        Some(w) => w.to_string(),
        None => n.to_string(),
    }
}

/// A roster rendered as a list (E4-3): the ref's comma-joined text
/// comes back "Hob, Maud and Agnes" - two names pair with "and", one
/// stands alone
pub fn list_join(text: &str) -> String {
    let items: Vec<&str> = text.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    match items.len() {
        0 => String::new(),
        1 => items[0].to_string(),
        2 => format!("{} and {}", items[0], items[1]),
        n => format!("{} and {}", items[..n - 1].join(", "), items[n - 1]),
    }
}

/// A verb's thou-form (E4-2): the auxiliaries by table, the rest by the
/// -est/-st rule - makest, knowest, goest. "must" does not bend.
pub fn thou_form(verb: &str) -> String {
    match verb {
        "be" | "is" | "are" | "am" => "art",
        "was" | "were" => "wast",
        "have" | "has" => "hast",
        "had" => "hadst",
        "do" | "does" => "dost",
        "did" => "didst",
        "can" => "canst",
        "could" => "couldst",
        "will" => "wilt",
        "would" => "wouldst",
        "shall" => "shalt",
        "should" => "shouldst",
        "may" => "mayst",
        "might" => "mightst",
        "must" => "must",
        "said" => "saidst",
        "ought" => "oughtest",
        "wit" => "wost",
        v if v.ends_with('e') => return format!("{v}st"),
        // carriest, spiest - a consonant's y goes to i before -est
        v if v.ends_with('y') && !matches!(v.as_bytes()[v.len() - 2],
            b'a' | b'e' | b'i' | b'o' | b'u') => return format!("{}iest", &v[..v.len() - 1]),
        v => return format!("{v}est"),
    }.to_string()
}

/// Verbs the lint knows a thou-form for; the irregulars' keys join them.
/// The lexicon is closed by design - E4-3 grows the table.
const VERBS: &[&str] = &[
    "know", "keep", "say", "think", "make", "give", "take", "speak", "tell",
    "hear", "see", "get", "bid", "hold", "find", "bring", "stand", "understand",
    "owe", "seem", "love", "hate", "fear", "hope", "pray", "live", "lie",
    "stay", "wait", "follow", "lead", "rest", "work", "ask", "answer",
    "remember", "forget", "believe", "doubt", "go", "come", "want", "wish",
    "mean", "call", "leave", "sit", "walk", "talk", "drink", "eat", "sleep",
    "wake", "rise", "fall", "grow", "show", "draw", "drive", "ride", "write",
    "read", "sing", "run", "break", "bear", "wear", "swear", "choose", "lose",
    "seek", "hide", "bind", "wind", "fight", "strike", "win", "begin",
];

/// Scan a line - authored source or realized output - for second-person
/// agreement faults (E4-2's lint): "thou are", "you art", and their kin.
/// A modal licenses the bare infinitive ("dost thou have"), so it is no
/// fault. Returns a short note per fault, empty when the line agrees.
pub fn agreement_faults(line: &str) -> Vec<String> {
    // thou-forms the lint knows: the irregulars' yields plus every
    // regular verb conjugated - minus any that equal their plain form
    let mut thou_forms: Vec<String> = THOU_IRREG.iter().map(|(_, t)| t.to_string()).collect();
    for v in VERBS { let t = thou_form(v); if t != *v { thou_forms.push(t); } }
    let words: Vec<String> = line
        .split(|c: char| !(c.is_alphabetic() || c == '\''))
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect();
    let mut faults = vec![];
    for (i, w) in words.iter().enumerate() {
        let next = words.get(i + 1).map(|s| s.as_str()).unwrap_or("");
        match w.as_str() {
            // only the subject forms bind a verb - "thee" is object
            // case, and names like Will would read as its verb
            "thou" => {
                // a modal before thou licenses the bare infinitive:
                // "dost thou have", "wouldst thou go" agree fine
                let licensed = i > 0 && matches!(words[i - 1].as_str(),
                    "dost" | "didst" | "canst" | "wilt" | "wouldst" | "shalt"
                    | "shouldst" | "mayst" | "mightst" | "must" | "needst" | "darest");
                if !licensed && matches!(next,
                    "are" | "is" | "have" | "has" | "do" | "does" | "did"
                    | "can" | "can't" | "cannot" | "will" | "shall" | "may"
                    | "was" | "were" | "could" | "would" | "should" | "might")
                {
                    faults.push(format!("{w} {next}"));
                }
            }
            "you" | "ye" => {
                if thou_forms.iter().any(|t| t == next) {
                    faults.push(format!("{w} {next}"));
                }
            }
            _ => {}
        }
    }
    faults
}

/// The irregular verb pairs: (plain, thou-form). `thou_form` serves the
/// realizer; this table serves the lint's "you X" check.
const THOU_IRREG: &[(&str, &str)] = &[
    ("be", "art"), ("is", "art"), ("are", "art"), ("am", "art"),
    ("was", "wast"), ("were", "wast"),
    ("have", "hast"), ("has", "hast"), ("had", "hadst"),
    ("do", "dost"), ("does", "dost"), ("did", "didst"),
    ("can", "canst"), ("could", "couldst"),
    ("will", "wilt"), ("would", "wouldst"),
    ("shall", "shalt"), ("should", "shouldst"),
    ("may", "mayst"), ("might", "mightst"), ("said", "saidst"),
    ("ought", "oughtest"), ("wit", "wost"),
];

/// Every authored line that can reach a mouth: the village library's
/// exchanges, the ask topics' tells, the meeting and arc beats, the
/// scenes and the hog's run-ons. The lint and the render-equivalence
/// tests sweep it.
#[doc(hidden)]
pub fn authored_lines() -> Vec<&'static str> {
    authored_lines_by_source().into_iter().map(|(_, l)| l).collect()
}

/// The corpus with each line tagged by the file it was authored in
/// (E4-4): the migration lint counts typed second-person forms per
/// source so a file's leftovers are its own to clear.
#[doc(hidden)]
pub fn authored_lines_by_source() -> Vec<(&'static str, &'static str)> {
    fn gather<'a>(src: &'static str, ex: &'a crate::game::conversation::Exchange,
                  out: &mut Vec<(&'static str, &'static str)>) {
        out.extend(ex.calls.iter().map(|l| (src, *l)));
        for v in ex.voice { out.extend(v.lines.iter().map(|l| (src, *l))); }
        for r in ex.replies {
            out.extend(r.lines.iter().map(|l| (src, *l)));
            for v in r.voice { out.extend(v.lines.iter().map(|l| (src, *l))); }
            if let Some(then) = &r.then { gather(src, then, out); }
        }
    }
    let mut all = vec![];
    for tree in crate::game::dialogue_trees::village_library() {
        gather("trees", tree.root, &mut all);
        if let Some(rejoin) = tree.rejoin { gather("trees", rejoin, &mut all); }
    }
    for t in crate::game::dialogue_trees::ASK_TOPICS {
        all.extend(t.tell.iter().map(|l| ("asks", *l)));
    }
    for b in crate::game::campaign_meeting_beats::MEETING_BEATS {
        all.extend(b.lines.iter().map(|l| ("meeting", *l)));
        for m in b.moves { all.extend(m.lines.iter().map(|l| ("meeting", *l))); }
    }
    for b in crate::game::campaign_arc_beats::ARC_BEATS {
        all.extend(b.lines.iter().map(|l| ("arc", *l)));
        for m in b.moves { all.extend(m.lines.iter().map(|l| ("arc", *l))); }
    }
    for s in crate::game::scenes::SCENES {
        for v in s.variants {
            all.extend(v.lines.iter().map(|(_, l)| ("scenes", *l)));
        }
    }
    all.extend(crate::game::conversation::HOG_RUN.iter().map(|l| ("hog", *l)));
    all.extend(crate::game::conversation::HOG_GISTS.iter().map(|l| ("hog", *l)));
    all.extend(crate::game::conversation::HOG_FLOOR.iter().map(|l| ("hog", *l)));
    for h in crate::game::idiolect::HABITS {
        all.extend(h.lines().iter().map(|l| ("idiolect", *l)));
    }
    for set in crate::game::conversation::EVADE_SETS {
        all.extend(set.iter().map(|l| ("evade", *l)));
    }
    for set in crate::game::conversation::worry_sets() {
        all.extend(set.iter().map(|l| ("worry", *l)));
    }
    for set in crate::game::conversation::joke_sets() {
        all.extend(set.iter().map(|l| ("joke", *l)));
    }
    all
}

/// A line's typed second-person forms (E4-4's leftover census): the
/// archaic set, the modern set, and any -st/-est verb a subject's "thou"
/// or "you" still binds. Everything a realizer slot can carry should be
/// a slot; what remains is what the lint counts.
#[doc(hidden)]
pub fn typed_second_person(line: &str) -> Vec<String> {
    let words: Vec<String> = line
        .split(|c: char| !(c.is_alphabetic() || c == '\''))
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect();
    let mut hits = vec![];
    for (i, w) in words.iter().enumerate() {
        let hit = match w.as_str() {
            "thee" | "thou" | "thy" | "thine" | "thyself" | "ye" | "art"
            | "wast" | "wert" | "hast" | "hadst" | "dost" | "didst" | "canst"
            | "couldst" | "wilt" | "wouldst" | "shalt" | "shouldst" | "mayst"
            | "mightst" | "saidst" | "you" | "your" | "yours" | "yourself"
            | "yourselves" => true,
            // a -st verb only counts beside its subject: "thou knowest",
            // "knowest thou" - "last" and "priest" are not verbs
            _ if w.len() > 3 && (w.ends_with("est") || w.ends_with("st"))
                && !matches!(w.as_str(), "last" | "lost" | "rest" | "best"
                    | "priest" | "quest" | "first" | "honest" | "earnest"
                    | "tempest" | "forest" | "harvest" | "interest" | "west"
                    | "east" | "feast" | "beast" | "least" | "breast" | "jest"
                    | "nest" | "test" | "pest" | "vest" | "chest" | "guest"
                    | "request" | "conquest" | "manifest" | "lest" | "just"
                    | "must" | "dust" | "thrust" | "trust" | "crust" | "worst"
                    | "against" | "amongst" | "midst" | "whilst" | "thirst") => {
                let near = |j: usize| matches!(words.get(j).map(|s| s.as_str()),
                    Some("thou") | Some("ye") | Some("you"));
                near(i.wrapping_sub(1)) || near(i + 1)
            }
            _ => false,
        };
        if hit { hits.push(w.clone()); }
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::conversation::{Relation, Speaker};

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

    /// The done-when: every authored line renders through the realizer
    /// exactly as the old replace chain made it
    fn all_library_lines() -> Vec<&'static str> {
        authored_lines()
    }

    #[test]
    fn every_library_line_renders_as_the_old_fills() {
        let (alys, corvin) = pair();
        let mut rng = Rng::new(3);
        let mut n = 0;
        let mut migrated = 0;
        for t in all_library_lines() {
            if t.contains("{2:") || t.contains("{2p:") || t.contains("{addr}")
                || t.contains("{self}") || t.contains(":self}") {
                // E4-2/E4-4/E4-7 migrated this line - the old fills never
                // owned these slots, so there is no old render to
                // match; it must only resolve every owned slot
                migrated += 1;
                let got = realize(t, &Cx::plain(&alys, &corvin, &[]), &mut rng);
                assert!(!got.contains("{2:") && !got.contains("{2p:") && !got.contains("{addr}")
                    && !got.contains("{self}") && !got.contains(":self}"), "line: {t}");
                continue;
            }
            let want = t.replace("{other}", "Corvin")
                .replace("{trade}", trade_noun(&alys))
                .replace("{other_trade}", trade_noun(&corvin));
            assert_eq!(realize(t, &Cx::plain(&alys, &corvin, &[]), &mut rng), want, "line: {t}");
            n += 1;
        }
        // the sweep must actually reach the corpus, not vacuously pass
        assert!(n > 1500, "only {n} lines swept");
        assert!(migrated > 10, "the lord lines should be slotted by now");
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

    // --- E4-2: the second person ---

    fn lord() -> Speaker {
        let mut l = Speaker::new("Edmund");
        l.ideology = "lord";
        l
    }

    #[test]
    fn the_power_semantic_gives_thou_down_and_you_up() {
        let mut reeve_man = Speaker::new("Hob");
        reeve_man.archaism = 2;
        let mut edmund = lord();
        edmund.archaism = 2;
        // superiors give thou, inferiors give you
        assert_eq!(tv_for(&edmund, &reeve_man, Relation::Acquaintance, false), Tv::Thou);
        assert_eq!(tv_for(&reeve_man, &edmund, Relation::Acquaintance, false), Tv::You);
        // peers: thou for kin and friends, you for strangers
        assert_eq!(tv_for(&reeve_man, &Speaker::new("Maud"), Relation::Friend, false), Tv::Thou);
        assert_eq!(tv_for(&reeve_man, &Speaker::new("Maud"), Relation::Kin, false), Tv::Thou);
        assert_eq!(tv_for(&reeve_man, &Speaker::new("Maud"), Relation::Stranger, false), Tv::You);
        // a marked shift: contempt gives thou where you is owed
        assert_eq!(tv_for(&reeve_man, &Speaker::new("Maud"), Relation::Rival, true), Tv::Thou);
    }

    #[test]
    fn archaism_gates_the_full_pattern() {
        let mut edmund = lord();
        let hob = Speaker::new("Hob");
        // level 1: thou only as a marked shift - bare rank earns you
        assert_eq!(tv_for(&edmund, &hob, Relation::Acquaintance, false), Tv::You);
        assert_eq!(tv_for(&edmund, &hob, Relation::Friend, false), Tv::Thou);
        // level 0: modern you always
        edmund.archaism = 0;
        assert_eq!(tv_for(&edmund, &hob, Relation::Kin, false), Tv::You);
        // level 2: the full pattern - rank itself gives thou
        edmund.archaism = 2;
        assert_eq!(tv_for(&edmund, &hob, Relation::Acquaintance, false), Tv::Thou);
    }

    #[test]
    fn second_person_slots_agree_with_the_form() {
        let (alys, corvin) = pair();
        let t = "{2:subj} {2:be} kind, and I thank {2:obj} for it - {2:poss} hand in mine.";
        let mut cx = Cx::plain(&alys, &corvin, &[]);
        cx.tv = Tv::Thou;
        assert_eq!(realize(t, &cx, &mut Rng::new(1)),
            "Thou art kind, and I thank thee for it - thy hand in mine.");
        cx.tv = Tv::You;
        assert_eq!(realize(t, &cx, &mut Rng::new(1)),
            "You are kind, and I thank you for it - your hand in mine.");
    }

    #[test]
    fn verbs_agree() {
        assert_eq!(thou_form("be"), "art");
        assert_eq!(thou_form("have"), "hast");
        assert_eq!(thou_form("do"), "dost");
        assert_eq!(thou_form("can"), "canst");
        assert_eq!(thou_form("will"), "wilt");
        assert_eq!(thou_form("shall"), "shalt");
        assert_eq!(thou_form("know"), "knowest");
        assert_eq!(thou_form("make"), "makest");
        assert_eq!(thou_form("go"), "goest");
        assert_eq!(thou_form("say"), "sayest");
        assert_eq!(thou_form("must"), "must");
    }

    #[test]
    fn address_forms_follow_station_and_bond() {
        let alys = Speaker::new("Alys");
        let edmund = lord();
        // a stranger to the manor: m'lord; his friend: his name
        let mut cx = Cx::plain(&alys, &edmund, &[]);
        assert_eq!(address_form(&cx), "m'lord");
        cx.rel = Relation::Friend;
        assert_eq!(address_form(&cx), "Edmund");
        // a father, a goodwife, a lad
        let mut priest = Speaker::new("Tobias");
        priest.ideology = "clergy";
        assert_eq!(address_form(&Cx::plain(&alys, &priest, &[])), "father");
        let mut maud = Speaker::new("Maud");
        maud.sex = Some("female");
        assert_eq!(address_form(&Cx::plain(&alys, &maud, &[])), "goodwife Maud");
        let mut will = Speaker::new("Will");
        will.status.push("child");
        assert_eq!(address_form(&Cx::plain(&alys, &will, &[])), "lad");
        // the village shopkeeper is a trader, the caller off the road a
        // traveler (#1371): the named courtesy, not a bare title
        let mut greta = Speaker::new("Greta");
        greta.ideology = "merchant"; greta.sex = Some("female");
        assert_eq!(address_form(&Cx::plain(&alys, &greta, &[])), "trader");
        let mut pedlar = Speaker::new("Nan");
        pedlar.status.push("visitor");
        assert_eq!(address_form(&Cx::plain(&alys, &pedlar, &[])), "traveler");
    }

    #[test]
    fn a_lord_gets_thee_only_when_flagged() {
        let alys = Speaker::new("Alys");
        let edmund = lord();
        // unflagged: you-forms and the title
        let mut cx = Cx::plain(&alys, &edmund, &[]);
        cx.tv = Tv::You;
        let got = realize("God keep {2:obj} and {2:poss} house, {addr}.", &cx, &mut Rng::new(1));
        assert_eq!(got, "God keep you and your house, m'lord.");
        // flagged intimate: thee and the bare name
        cx.tv = Tv::Thou;
        cx.rel = Relation::Kin;
        assert_eq!(realize("God keep {2:obj} and {2:poss} house, {addr}.", &cx, &mut Rng::new(1)),
            "God keep thee and thy house, Edmund.");
    }

    #[test]
    fn the_lint_catches_crossed_forms() {
        assert!(agreement_faults("thou art well").is_empty());
        assert!(agreement_faults("dost thou have grain").is_empty());
        assert!(agreement_faults("wouldst thou have the village do it").is_empty());
        assert_eq!(agreement_faults("thou are well"), vec!["thou are"]);
        assert_eq!(agreement_faults("if thou can find them"), vec!["thou can"]);
        assert_eq!(agreement_faults("you art mistaken"), vec!["you art"]);
        assert_eq!(agreement_faults("you knowest it"), vec!["you knowest"]);
        // and modern words that merely end in -est stay clear
        assert!(agreement_faults("an honest request, at your best").is_empty());
    }

    #[test]
    fn every_authored_second_person_line_agrees() {
        // realized output of every slotted line agrees under each form
        let (alys, corvin) = pair();
        let mut rng = Rng::new(5);
        for t in authored_lines().iter().filter(|t| t.contains("{2:")) {
            for tv in [Tv::Thou, Tv::You] {
                let mut cx = Cx::plain(&alys, &corvin, &[]);
                cx.tv = tv;
                let got = realize(t, &cx, &mut rng);
                assert!(agreement_faults(&got).is_empty(), "{tv:?}: {got}");
            }
        }
    }

    // --- E4-3: the lexicon ---

    #[test]
    fn every_irregular_plural_maps() {
        // the done-when is exhaustive: the whole closed table, exercised
        for (s, p) in IRREG {
            assert_eq!(&plural(s), p, "plural of {s}");
        }
        assert!(IRREG.len() > 30, "the table should cover the lexicon");
    }

    #[test]
    fn the_four_suffix_rules_cover_the_rest() {
        // sibilant -es
        assert_eq!(plural("bench"), "benches");
        assert_eq!(plural("ash"), "ashes");
        assert_eq!(plural("fox"), "foxes");
        assert_eq!(plural("mass"), "masses");
        // consonant+y drops the y
        assert_eq!(plural("berry"), "berries");
        assert_eq!(plural("story"), "stories");
        // but a vowel+y keeps it
        assert_eq!(plural("day"), "days");
        assert_eq!(plural("boy"), "boys");
        // f/fe voices to -ves
        assert_eq!(plural("strife"), "strives");
        assert_eq!(plural("scarf"), "scarves");
        // plain -s for the rest
        assert_eq!(plural("cart"), "carts");
        assert_eq!(plural("barley"), "barleys");
        assert_eq!(plural("field"), "fields");
        assert_eq!(plural("granary"), "granaries");
    }

    #[test]
    fn a_and_an_follow_the_sound() {
        assert_eq!(article("smith"), "a");
        assert_eq!(article("ox"), "an");
        assert_eq!(article("ale"), "an");
        assert_eq!(article("cart"), "a");
        // the closed exceptions
        assert_eq!(article("honest"), "an");
        assert_eq!(article("heir"), "an");
        assert_eq!(article("unicorn"), "a");
        assert_eq!(article("one-eyed"), "a");
        assert_eq!(article("ewe"), "a");
        assert_eq!(article("onset"), "an");
        assert_eq!(article("ewer"), "a");
        // vocation::article defers to the lexicon
        assert_eq!(crate::game::vocation::article("Apprentice"), "an");
        assert_eq!(crate::game::vocation::article("Master"), "a");
    }

    #[test]
    fn number_words_and_lists() {
        assert_eq!(number_word(0), "no");
        assert_eq!(number_word(1), "one");
        assert_eq!(number_word(3), "three");
        assert_eq!(number_word(12), "twelve");
        assert_eq!(number_word(20), "twenty");
        assert_eq!(number_word(23), "23");
        assert_eq!(list_join("Hob"), "Hob");
        assert_eq!(list_join("Hob, Maud"), "Hob and Maud");
        assert_eq!(list_join("Hob, Maud, Agnes"), "Hob, Maud and Agnes");
    }

    #[test]
    fn the_lexicon_slots_read_the_referent() {
        let (alys, corvin) = pair();
        let refs = [
            Referent { slot: "someone", text: "Old Nan".into(), sex: Some("female"), alive: true, count: 1 },
            Referent { slot: "lads", text: "lad".into(), sex: Some("male"), alive: true, count: 3 },
            Referent { slot: "grandda", text: "Corvin".into(), sex: Some("male"), alive: false, count: 1 },
        ];
        let cx = Cx::plain(&alys, &corvin, &refs);
        let mut rng = Rng::new(2);
        // pronouns from sex; they for a group or an unrecorded body
        assert_eq!(realize("{he:someone} keeps bees; I gave {him:someone} {his:someone} due.", &cx, &mut rng),
            "She keeps bees; I gave her her due.");
        assert_eq!(realize("{he:lads} run wild - {his:lads} manners.", &cx, &mut rng),
            "They run wild - their manners.");
        // tense and number read the body: the dead go past, groups go plural
        assert_eq!(realize("{is:grandda} buried past the church.", &cx, &mut rng),
            "Was buried past the church.");
        assert_eq!(realize("{was:lads} three strong arms once.", &cx, &mut rng),
            "Were three strong arms once.");
        // number + plural, article, and the roster
        assert_eq!(realize("{n:lads} came calling.", &cx, &mut rng), "Three lads came calling.");
        assert_eq!(realize("He is {a:someone}.", &cx, &mut rng), "He is an Old Nan.");
        assert_eq!(realize("He is {a:ox}.", &cx, &mut rng), "He is an ox.");
        // {other} and {self} name the pair without a ref
        let mut maud = Speaker::new("Maud");
        maud.sex = Some("female");
        let mut hob = Speaker::new("Hob");
        hob.sex = Some("male");
        let cx = Cx::plain(&hob, &maud, &[]);
        assert_eq!(realize("{he:other} keeps the inn.", &cx, &mut rng), "She keeps the inn.");
        assert_eq!(realize("And {he:self}? {he:self} keeps the forge.", &cx, &mut rng),
            "And he? He keeps the forge.");
    }

    #[test]
    fn unowned_lexicon_slots_stay_literal() {
        let (alys, corvin) = pair();
        let cx = Cx::plain(&alys, &corvin, &[]);
        let mut rng = Rng::new(2);
        assert_eq!(realize("{n:ghost} and {he:ghost} went by.", &cx, &mut rng),
            "{n:ghost} and {he:ghost} went by.");
        assert_eq!(realize("{list:folk} and {is:ghost} done.", &cx, &mut rng),
            "{list:folk} and {is:ghost} done.");
    }

    #[test]
    fn a_guarded_no_runs_longer_than_a_proud_one() {
        use crate::game::traits::Trait;
        let (alys, _) = pair();
        let mut wary = Speaker::new("Wat");
        wary.traits = vec![Trait::Wary, Trait::Stolid];
        let mut proud = Speaker::new("Hob");
        proud.traits = vec![Trait::Proud];
        proud.pull = 0.9;
        let sg = stance_of(&wary, &alys);
        let sp = stance_of(&proud, &alys);
        assert!(sg.guard > sp.guard, "guarded {sg:?} should out-guard proud {sp:?}");
        let reason = Some("the reeve counts heads at the gate");
        let ops_g = shape(crate::game::conversation::Act::PushBack, &sg, reason);
        let ops_p = shape(crate::game::conversation::Act::PushBack, &sp, reason);
        assert!(ops_g.len() > ops_p.len(), "{ops_g:?} vs {ops_p:?}");
        let mut rng = Rng::new(7);
        let long_no = shaped(&ops_g, "Not this week.", reason, &sg, &mut rng);
        let mut rng = Rng::new(7);
        let short_no = shaped(&ops_p, "Not this week.", reason, &sp, &mut rng);
        assert!(long_no.len() > short_no.len(), "{long_no} vs {short_no}");
        assert!(short_no == "Not this week.", "a proud no stays bare: {short_no}");
    }

    #[test]
    fn a_cold_guarded_yes_reads_like_a_no() {
        use crate::game::traits::Trait;
        let (alys, _) = pair();
        let mut guarded = Speaker::new("Maud");
        guarded.traits = vec![Trait::Wary];
        // a cold bond: nothing reached, nothing trusted
        guarded.open = crate::game::conversation::Open::default();
        let cold = stance_of(&guarded, &alys);
        let ops = shape(crate::game::conversation::Act::Agree, &cold, None);
        assert!(!ops.is_empty(), "a guarded, cold yes should be padded");
        let mut rng = Rng::new(3);
        let padded = shaped(&ops, "Aye.", None, &cold, &mut rng);
        assert!(padded != "Aye.", "padded: {padded}");
        // a warm, open yes stays plain
        let mut warm = Speaker::new("Maud");
        warm.open.benevolence = 0.9;
        warm.open.intimacy = 0.9;
        warm.open.mood = 0.5;
        warm.open.safety = 0.9;
        let sg = stance_of(&warm, &alys);
        assert!(shape(crate::game::conversation::Act::Agree, &sg, None).is_empty());
    }

    // E4-6 (#1433): the style markers a temperament sets - a line rolls
    // which fire, two at most, most lines plain; the done-when is the
    // marker rate tracking its trait, identifiability over the baseline
    mod e4_6 {
        use super::*;
        use crate::game::traits::Trait;

        fn speaker(name: &str, traits: &[Trait]) -> Speaker {
            let mut s = Speaker::new(name);
            s.traits = traits.to_vec();
            s
        }

        /// The fragments a fired marker can leave - one per kind, so a
        /// line's marks can be counted from the text alone
        fn marker_fragments(line: &str) -> usize {
            let mut n = 0;
            for w in OATH_OPENERS { if line.starts_with(w.trim_end()) { n += 1; } }
            for w in EXCLAIM_OPENERS { if line.starts_with(w.trim_end()) { n += 1; } }
            if line.ends_with('!') { n += 1; }
            for set in [&VERBOSE_TAILS[..], &HEDGE_TAILS[..],
                        &ASKBACK_TAILS[..], &SELFTALK_TAILS[..]] {
                for w in set { if line.ends_with(w.trim_start_matches(',')) { n += 1; } }
            }
            n
        }

        #[test]
        fn test_the_trait_loads_its_markers() {
            let pious = style_strengths(&speaker("Maud", &[Trait::Pious]));
            assert!(pious[5] >= 0.4, "pious should carry the oath: {pious:?}");
            let hog = style_strengths(&speaker("Nun", &[Trait::SelfAbsorbed]));
            assert!(hog[4] >= 0.4 && hog[3] >= 0.2, "self-absorbed: {hog:?}");
            let wary = style_strengths(&speaker("Wat", &[Trait::Wary]));
            assert!(wary[1] >= 0.3, "wary should carry the hedge: {wary:?}");
            // the quiet one damps what the others raise
            let stolid = style_strengths(&speaker("Cobb",
                &[Trait::Stolid, Trait::Pious, Trait::Gregarious]));
            assert!(stolid.iter().all(|&v| v < 0.3), "stolid should damp: {stolid:?}");
            // no traits, no voice to sign
            assert!(style_strengths(&speaker("Plain", &[])).iter().all(|&v| v == 0.0));
        }

        #[test]
        fn test_the_markers_lean_on_their_trait() {
            // the done-when: over a wide line pool the marker rate tracks
            // the trait - a pious speaker's oaths and a self-absorbed
            // one's self-reference rise over the plain baseline
            let (alys, corvin) = pair();
            let lines = authored_lines();
            let count = |s: &Speaker, frag: fn(&str) -> bool| {
                let cx = Cx::plain(s, &corvin, &[]);
                let mut rng = Rng::new(5);
                lines.iter().filter(|&&t| frag(&realize(t, &cx, &mut rng))).count()
            };
            let plain = speaker("Hob", &[]);
            let pious = speaker("Hob", &[Trait::Pious]);
            let oathed = |l: &str| OATH_OPENERS.iter().any(|w| l.starts_with(w.trim_end()));
            assert!(count(&pious, oathed) > count(&plain, oathed) + 20,
                "pious vs plain oaths: {} vs {}",
                count(&pious, oathed), count(&plain, oathed));
            let hog = speaker("Hob", &[Trait::SelfAbsorbed]);
            let selfish = |l: &str| SELFTALK_TAILS.iter()
                .any(|w| l.ends_with(w.trim_start_matches(',')))
                || ASKBACK_TAILS.iter().any(|w| l.ends_with(w.trim_start_matches(',')));
            assert!(count(&hog, selfish) > count(&plain, selfish) + 40,
                "self-absorbed vs plain: {} vs {}",
                count(&hog, selfish), count(&plain, selfish));
            let _ = alys;
        }

        #[test]
        fn test_two_markers_cap_a_line_and_most_stay_plain() {
            let (alys, _) = pair();
            // the fullest mouth in the village: five strong traits
            let loud = speaker("Hob", &[Trait::Pious, Trait::Hotheaded,
                Trait::SelfAbsorbed, Trait::Gregarious, Trait::Wary]);
            let cx = Cx::plain(&loud, &alys, &[]);
            let mut rng = Rng::new(9);
            let mut marked_lines = 0;
            for t in authored_lines() {
                let got = realize(t, &cx, &mut rng);
                let n = marker_fragments(&got);
                assert!(n <= 2, "three marks on one line: {got} <- {t}");
                if n > 0 { marked_lines += 1; }
            }
            // even the fullest mouth leaves most of its lines alone
            assert!(marked_lines * 2 < authored_lines().len(),
                "{marked_lines} of {} lines marked", authored_lines().len());
        }

        #[test]
        fn test_the_same_line_marks_the_same_way() {
            let (alys, _) = pair();
            let pious = speaker("Tobias", &[Trait::Pious]);
            let cx = Cx::plain(&pious, &alys, &[]);
            // two different streams, one rendering: a preview shows the
            // marks the saying wears
            let a = realize("The well's gone foul, {other}.", &cx, &mut Rng::new(1));
            let b = realize("The well's gone foul, {other}.", &cx, &mut Rng::new(999));
            assert_eq!(a, b);
            // and a trait-less mouth leaves the words alone
            let hob = speaker("Hob", &[]);
            let plain = Cx::plain(&hob, &alys, &[]);
            assert_eq!(realize("The well's gone foul, {other}.", &plain, &mut Rng::new(1)),
                "The well's gone foul, Alys.");
        }

        #[test]
        fn test_the_marker_and_the_shape_compose() {
            let (alys, _) = pair();
            // a guarded, wary speaker's padded no still signs itself:
            // the shape wraps the marked line, one pass through the two
            let guarded = speaker("Wat", &[Trait::Wary, Trait::Pious]);
            let st = stance_of(&guarded, &alys);
            let ops = shape(crate::game::conversation::Act::PushBack, &st, None);
            let marked_line = marked("Not this week.", "Not this week.", &guarded);
            let mut rng = Rng::new(4);
            let out = shaped(&ops, &marked_line, None, &st, &mut rng);
            for frag in ["Oh…", "Well…", "Hm…", "Ah…", "It isn't that I wouldn't.",
                         "I hear you.", "There's that to think on.",
                         "Saints keep us.", "By the rood.", "God witness.",
                         "or so I hear", "as like as not", "by my reckoning"] {
                if out.contains(frag) { return; }
            }
            panic!("no mark and no shape survived: {out}");
        }
    }
}
