//! The E4-2 lint (#1414): no authored or realized line may cross the
//! second-person forms - no thou with "are", no you with "art" - and a
//! lord gets thee only when flagged (kin, friend, or a rival's slight).

use crate::game::conversation::{Relation, Rng, Speaker};
use crate::game::campaign_dialogue::fill;
use crate::game::text::{agreement_faults, authored_lines, authored_lines_by_source,
    marker_seam_fault, realize, tv_for, typed_second_person, Cx, Referent};

#[test]
fn test_no_authored_line_crosses_the_forms() {
    let mut bad = vec![];
    for t in authored_lines() {
        for f in agreement_faults(t) {
            bad.push(format!("{f} <- {t}"));
        }
    }
    assert!(bad.is_empty(), "agreement faults:\n{}", bad.join("\n"));
}

#[test]
fn test_every_realized_line_agrees() {
    // every line rendered for each side of the stations ladder, across
    // the bonds - the realized text agrees as the authored text does
    let stations: Vec<Speaker> = {
        let mut lord = Speaker::new("Edmund");
        lord.ideology = "lord";
        let mut priest = Speaker::new("Tobias");
        priest.ideology = "clergy";
        let mut trader = Speaker::new("Sibb");
        trader.ideology = "merchant";
        trader.sex = Some("female");
        let mut hand = Speaker::new("Hob");
        hand.archaism = 2;
        let mut child = Speaker::new("Will");
        child.status.push("child");
        vec![lord, priest, trader, hand, child]
    };
    let mut rng = Rng::new(11);
    let mut checked = 0;
    for t in authored_lines() {
        for a in &stations {
            for b in &stations {
                for rel in [Relation::Stranger, Relation::Friend, Relation::Rival] {
                    let mut cx = Cx::plain(a, b, &[]);
                    cx.tv = tv_for(a, b, rel, matches!(rel, Relation::Rival));
                    let got = realize(t, &cx, &mut rng);
                    assert!(agreement_faults(&got).is_empty(), "{:?}->{:?} {:?}: {got} <- {t}",
                        a.ideology, b.ideology, rel);
                    // #1691: a menu label is menu-speak, never spoken text -
                    // any slot resolving to a topic's slug is a leak (the
                    // allowlist holds labels that are also plain speech)
                    const PLAIN: &[&str] = &["the village", "the road", "their work",
                        "their sickness", "their hunger", "their grief", "themselves",
                        "the old days"];
                    for tp in crate::game::dialogue_trees::ASK_TOPICS {
                        assert!(PLAIN.contains(&tp.label) || !got.contains(tp.label),
                            "topic slug {:?} reached a line: {got} <- {t}", tp.label);
                    }
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 60_000, "only {checked} renderings checked");
}

/// #1694's seam lint: a lean-in marker can't front a clause it can't
/// carry (`Why,` wants a lowercase tail), and no opener stands before a
/// greeting's head - over every authored line, every station, every bond
#[test]
fn test_no_marker_stammers_onto_a_clause() {
    let stations: Vec<Speaker> = {
        let mut lord = Speaker::new("Edmund");
        lord.ideology = "lord";
        let mut priest = Speaker::new("Tobias");
        priest.ideology = "clergy";
        let mut trader = Speaker::new("Sibb");
        trader.ideology = "merchant";
        trader.sex = Some("female");
        let mut hand = Speaker::new("Hob");
        hand.archaism = 2;
        let mut child = Speaker::new("Will");
        child.status.push("child");
        vec![lord, priest, trader, hand, child]
    };
    let mut rng = Rng::new(17);
    let mut bad = vec![];
    for t in authored_lines() {
        for a in &stations {
            for b in &stations {
                for rel in [Relation::Stranger, Relation::Friend, Relation::Rival] {
                    let mut cx = Cx::plain(a, b, &[]);
                    cx.tv = tv_for(a, b, rel, matches!(rel, Relation::Rival));
                    let got = realize(t, &cx, &mut rng);
                    if marker_seam_fault(&got) {
                        bad.push(format!("{got} <- {t}"));
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "marker seams:\n{}", bad.join("\n"));
}

#[test]
fn test_a_lord_gets_thee_only_when_flagged() {
    let thee = ["thee", "thou", "thy", "thine", "thyself", "art", "hast", "dost",
        "canst", "wilt", "shalt", "mayst", "sayest", "shouldst", "ye"];
    let mut rng = Rng::new(13);
    let mut hob = Speaker::new("Hob");
    hob.archaism = 2;
    let mut edmund = Speaker::new("Edmund");
    edmund.ideology = "lord";
    // every slotted line rendered onto the lord
    let slotted: Vec<&'static str> = authored_lines().into_iter()
        .filter(|t| t.contains("{2:") || t.contains("{2p:") || t.contains("{addr}")).collect();
    assert!(slotted.len() > 15, "the migration should cover the lord lines");
    for t in slotted {
        // unflagged: a stranger's courtesy is you and a title
        let mut cx = Cx::plain(&hob, &edmund, &[]);
        cx.tv = tv_for(&hob, &edmund, Relation::Stranger, false);
        let got = realize(t, &cx, &mut rng).to_lowercase();
        for w in thee {
            assert!(!got.split(|c: char| !(c.is_alphabetic() || c == '\''))
                .map(|tok| tok.trim_matches('\''))
                .any(|tok| tok == w), "unflagged lord got '{w}': {got} <- {t}");
        }
        if t.contains("{addr}") {
            assert!(got.contains("m'lord"), "unflagged lord lost the title: {got} <- {t}");
        }
        // flagged: a friend says thee, and the bare name for a title
        let mut cx = Cx::plain(&hob, &edmund, &[]);
        cx.tv = tv_for(&hob, &edmund, Relation::Friend, false);
        cx.rel = Relation::Friend;
        let got = realize(t, &cx, &mut rng).to_lowercase();
        if t.contains("{addr}") {
            assert!(got.contains("edmund"), "flagged lord kept the title: {got} <- {t}");
        }
        if t.contains("{2:") {
            let toks: Vec<&str> = got.split(|c: char| !(c.is_alphabetic() || c == '\''))
                .map(|w| w.trim_matches('\'')).filter(|w| !w.is_empty()).collect();
            assert!(thee.iter().any(|w| toks.contains(w)), "flagged lord got no thou-form: {got} <- {t}");
        }
    }
}

#[test]
fn test_the_migration_leaves_no_typed_forms_in_the_trees() {
    // E4-4's leftover lint: typed second-person forms counted per source
    // file. The trees are migrated; the beats and scenes count down to
    // zero on the second pass.
    let mut counts: std::collections::BTreeMap<&str, usize> = Default::default();
    let mut samples: std::collections::BTreeMap<&str, Vec<String>> = Default::default();
    for (src, line) in authored_lines_by_source() {
        let hits = typed_second_person(line);
        if hits.is_empty() { continue; }
        *counts.entry(src).or_default() += hits.len();
        let s = samples.entry(src).or_default();
        if s.len() < 3 {
            s.push(format!("{hits:?} <- {line}"));
        }
    }
    assert_eq!(counts.get("trees").copied().unwrap_or(0), 0,
        "typed forms left in the trees: {:?}", samples.get("trees"));
    assert_eq!(counts.get("asks").copied().unwrap_or(0), 0,
        "typed forms left in the asks: {:?}", samples.get("asks"));
    // PR2's pass: the beats, scenes and hog strings are migrated too -
    // every source counts down to zero (E4-4's done-when)
    for src in ["meeting", "arc", "scenes", "hog"] {
        assert_eq!(counts.get(src).copied().unwrap_or(0), 0,
            "typed forms left in {src}: {:?}", samples.get(src));
    }
    eprintln!("typed second-person forms by source: {counts:?}");
}

/// E0-2 (#1401): the line lint over every authored line - unfilled
/// slots, a/an, doubled words, anachronisms, hardcoded terms of
/// address a {addr}/{trade} slot would resolve right (#1371/#1372)
// the realizer's own grammar - exact names, then the keyed families
// `2:`/`2p:` second person, `he:`/`him:`/`his:`/`is:`/`was:`/`n:`/`a:`/
// `list:` referent grammar - plus the names a `refs`/`fill` carries
const SLOT_FAMILIES: &[&str] = &["2", "2p", "he", "him", "his", "is", "was",
    "n", "a", "list", "other", "other_trade", "trade", "addr",
    "someone", "lads", "grandda", "issue", "self", "grievance", "mystery"];

fn slot_known(s: &str) -> bool {
    let s = s.trim_start_matches('{').trim_end_matches('}');
    SLOT_FAMILIES.contains(&s.split(':').next().unwrap_or(s))
}

fn lint_tokens(line: &str) -> Vec<String> {
    line.split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '{' || c == '}' || c == ':'))
        .filter(|t| !t.is_empty()).map(|t| t.to_string()).collect()
}

#[test]
fn test_authored_lines_have_no_unknown_or_unfilled_slots() {
    let mut bad = vec![];
    for t in authored_lines() {
        // a slot is any {x} or {x:y} run - names must be known or the
        // typed second-person families
        let bytes = t.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'{' {
                let end = t[i..].find('}').map(|e| i + e);
                match end {
                    Some(e) => {
                        if !slot_known(&t[i..=e]) {
                            bad.push(format!("unknown slot {} <- {}", &t[i..=e], t));
                        }
                        i = e + 1;
                        continue;
                    }
                    None => { bad.push(format!("unclosed slot <- {t}")); break; }
                }
            }
            i += 1;
        }
        // a realizer must not leave any slot standing - `{someone}`
        // comes from refs, `{issue}` from the meeting's own fill
        let a = Speaker::new("Hob");
        let b = Speaker::new("Sibb");
        let mut rng = Rng::new(3);
        let refs = [Referent { slot: "someone", text: "Old Nan".into(),
            sex: Some("female"), alive: true, count: 1 },
            Referent { slot: "lads", text: "the lads".into(),
                sex: None, alive: true, count: 3 },
            Referent { slot: "grandda", text: "grandda".into(),
                sex: Some("male"), alive: false, count: 1 },
            Referent { slot: "grievance", text: "the tithe".into(),
                sex: None, alive: true, count: 1 }];
        let got = fill(&realize(t, &Cx::plain(&a, &b, &refs), &mut rng), "the hunger");
        if got.contains('{') || got.contains('}') {
            bad.push(format!("unfilled after realize: {got} <- {t}"));
        }
    }
    assert!(bad.is_empty(), "slot faults:\n{}", bad.join("\n"));
}

#[test]
fn test_authored_lines_keep_a_an_and_carry_no_doubled_words() {
    // takes "an" on a silent h, "a" on the yoo-glide
    let vowel = |w: &str| {
        let w = w.to_lowercase();
        if ["hour", "honest", "honor", "honour", "heir"].iter().any(|&x| w.starts_with(x)) { return true; }
        if ["uni", "use", "ewe", "u.", "one"].iter().any(|&x| w.starts_with(x)) { return false; }
        w.chars().next().map(|c| "aeiou".contains(c)).unwrap_or(false)
    };
    let mut bad = vec![];
    for t in authored_lines() {
        let toks = lint_tokens(t);
        for w in toks.windows(2) {
            let (a, b) = (w[0].to_lowercase(), w[1].to_lowercase());
            if a == "a" && !b.starts_with('{') && vowel(&b) {
                bad.push(format!("'a {b}' <- {t}"));
            }
            if a == "an" && !b.starts_with('{') && !vowel(&b) {
                bad.push(format!("'an {b}' <- {t}"));
            }
        }
        // a doubled word with a mark between is a voice's emphasis; only
        // whitespace-joined doubles are copy faults
        let mut pos = 0usize;
        let mut last_end = 0usize;
        let mut last_word = String::new();
        while pos < t.len() {
            let rest = &t[pos..];
            let wl = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '\'').count();
            if wl == 0 { pos += 1; continue; }
            let word = &rest[..wl];
            let gap = &t[last_end..pos];
            if word.to_lowercase() == last_word && gap.chars().all(|c| c.is_whitespace()) {
                bad.push(format!("doubled '{word}' <- {t}"));
            }
            last_word = word.to_lowercase();
            last_end = pos + wl;
            pos += wl;
        }
    }
    assert!(bad.is_empty(), "a/an + doubled faults:
{}", bad.join("
"));
}

#[test]
fn test_authored_lines_carry_no_anachronisms() {
    const MODERN: &[&str] = &["okay", "ok", "yeah", "yep", "nope", "hi", "hello",
        "weekend", "percent", "guy", "guys", "awesome", "scientist",
        "machine", "phone", "television", "internet", "email", "software",
        "computer", "rocket", "airplane", "bicycle", "motor", "electric",
        "battery", "plastic", "commute", "deadline", "hobby", "teenager",
        "kilometers", "liters", "dollars",
        "cents", "budget", "schedule", "boss", "employee",
        "manager", "interview", "candidate", "statistics", "vaccine", "virus",
        "bacteria", "germ", "germs", "microscope", "surgery", "therapist"];
    let mut bad = vec![];
    for t in authored_lines() {
        let toks: Vec<String> = lint_tokens(t).iter().map(|s| s.to_lowercase()).collect();
        for w in &toks {
            if MODERN.contains(&w.as_str()) {
                bad.push(format!("modern '{w}' <- {t}"));
            }
        }
    }
    assert!(bad.is_empty(), "anachronisms:\n{}", bad.join("\n"));
}

#[test]
fn test_authored_lines_use_slots_for_terms_of_address() {
    // a hardcoded station word where a slot would fit the actual
    // listener (#1371/#1372's wrong-address check): the census runs
    // per source, lines deduped - one line keyed on a trade may sit
    // in the table many times over
    const ADDRESS: &[&str] = &["m'lord", "milord", "goodman", "goodwife",
        "merchant", "trader", "pedlar", "father"];
    let mut bad = vec![];
    let mut seen = std::collections::BTreeSet::new();
    let mut census = std::collections::BTreeMap::new();
    for (source, t) in authored_lines_by_source() {
        let toks: Vec<String> = lint_tokens(t).iter().map(|s| s.to_lowercase()).collect();
        for w in &toks {
            if ADDRESS.contains(&w.as_str()) && seen.insert((w.clone(), source, t)) {
                *census.entry((w.clone(), source)).or_insert(0u32) += 1;
            }
        }
    }
    // today's census is pinned - a new address word, a new line using
    // one, or a fix removing one all move the count and get flagged.
    // The full line list is on #1401; the 'goodman' six are the open
    // wrong-address cases (a lord's voice, the listener's sex and
    // station not guaranteed)
    let known: &[(&str, &str, u32)] = &[
        ("father", "idiolect", 5),
        ("father", "meeting", 6), ("father", "scenes", 1), ("father", "trees", 31),
        ("goodman", "trees", 6),
        ("merchant", "arc", 5), ("merchant", "meeting", 1), ("merchant", "trees", 7),
        ("pedlar", "meeting", 2), ("pedlar", "scenes", 2), ("pedlar", "trees", 4),
        ("trader", "trees", 9)];
    let mut pinned = std::collections::BTreeSet::new();
    for ((w, source), n) in &census {
        pinned.insert((w.as_str(), *source));
        match known.iter().find(|(kw, ks, _)| kw == w && ks == source) {
            Some((_, _, want)) if *n == *want => {}
            Some((_, _, want)) => bad.push(format!(
                "{source}: '{w}' x{n}, expected x{want} - new or removed use?")),
            None => bad.push(format!("{source}: '{w}' x{n} - is this tree keyed to it?")),
        }
    }
    for (w, source, _) in known {
        if !pinned.contains(&(*w, *source)) {
            bad.push(format!("{source}: '{w}' gone entirely - clear the pin"));
        }
    }
    assert!(bad.is_empty(), "new hardcoded address terms:
{}", bad.join("
"));
}
/// #1692's done-when: once an exchange binds a third party with a
/// `{*:dead}`-style referent slot, every pronoun for it goes through
/// the slot - a bare "his"/"she" beside the slots is another rolled
/// referent, the condolence that grieves two
#[test]
fn test_a_bound_referent_owns_its_pools_pronouns() {
    use crate::game::conversation::Exchange;
    fn lines_of<'a>(ex: &'a Exchange) -> Vec<&'static str> {
        let mut ls: Vec<&'static str> = ex.calls.to_vec();
        for v in ex.voice { ls.extend(v.lines.iter().copied()); }
        for r in ex.replies {
            ls.extend(r.lines.iter().copied());
            for v in r.voice { ls.extend(v.lines.iter().copied()); }
            if let Some(t) = &r.then { ls.extend(lines_of(t)); }
        }
        ls
    }
    let bare = |l: &str| {
        // slots bind what they name - scan only the line's own words
        let mut stripped = String::with_capacity(l.len());
        let mut depth = 0;
        for c in l.chars() {
            match c { '{' => depth += 1, '}' => depth -= 1,
                _ if depth == 0 => stripped.push(c), _ => {} }
        }
        stripped.split(|c: char| !c.is_ascii_alphabetic())
            .any(|w| matches!(w.to_ascii_lowercase().as_str(),
                "he" | "him" | "his" | "she" | "her" | "hers"))
    };
    let mut n = 0;
    for tree in crate::game::dialogue_trees::village_library() {
        for ex in [Some(tree.root), tree.rejoin].into_iter().flatten() {
            let ls = lines_of(ex);
            if !ls.iter().any(|l| l.contains(":dead}")) { continue; }
            n += 1;
            for l in ls {
                assert!(!bare(l), "a bare pronoun beside the bound dead: {l}");
            }
        }
    }
    assert!(n >= 2, "the grief pool should bind the dead ({n} pools found)");
}
