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
            stance: Stance::default(), refs, archaism: speaker.archaism }
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
        "trade" => out.push_str(&trade_noun(cx.speaker)),
        "other_trade" => out.push_str(&trade_noun(cx.listener)),
        // E4-2: the address a relation earns - "m'lord", "goodwife Maud"
        "addr" => push(out, &address_form(cx)),
        // E4-2: second person - pronouns and verb agreement read `cx.tv`
        _ if slot.starts_with("2:") => push(out, &second_person(&slot[2..], cx.tv)),
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
        "ought" => "oughtest",
        "wit" => "wost",
        v => return if v.ends_with('e') { format!("{v}st") } else { format!("{v}est") },
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
    ("may", "mayst"), ("might", "mightst"),
    ("ought", "oughtest"), ("wit", "wost"),
];

/// Every authored line that can reach a mouth: the village library's
/// exchanges, the ask topics' tells, the meeting and arc beats, the
/// scenes and the hog's run-ons. The lint and the render-equivalence
/// tests sweep it.
#[doc(hidden)]
pub fn authored_lines() -> Vec<&'static str> {
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
    for tree in crate::game::dialogue_trees::village_library() {
        gather(tree.root, &mut all);
        if let Some(rejoin) = tree.rejoin { gather(rejoin, &mut all); }
    }
    for t in crate::game::dialogue_trees::ASK_TOPICS {
        all.extend(t.tell.iter().copied());
    }
    for b in crate::game::campaign_meeting_beats::MEETING_BEATS {
        all.extend(b.lines.iter().copied());
        for m in b.moves { all.extend(m.lines.iter().copied()); }
    }
    for b in crate::game::campaign_arc_beats::ARC_BEATS {
        all.extend(b.lines.iter().copied());
        for m in b.moves { all.extend(m.lines.iter().copied()); }
    }
    for s in crate::game::scenes::SCENES {
        for v in s.variants {
            all.extend(v.lines.iter().map(|(_, l)| *l));
        }
    }
    all.extend(crate::game::conversation::HOG_RUN.iter().copied());
    all.extend(crate::game::conversation::HOG_GISTS.iter().copied());
    all.extend(crate::game::conversation::HOG_FLOOR.iter().copied());
    all
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
            if t.contains("{2:") || t.contains("{addr}") {
                // E4-2 migrated this line - the old fills never owned
                // these slots, so there is no old render to match;
                // it must only resolve every owned slot
                migrated += 1;
                let got = realize(t, &Cx::plain(&alys, &corvin, &[]), &mut rng);
                assert!(!got.contains("{2:") && !got.contains("{addr}"), "line: {t}");
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
}
