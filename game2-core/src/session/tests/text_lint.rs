//! The E4-2 lint (#1414): no authored or realized line may cross the
//! second-person forms - no thou with "are", no you with "art" - and a
//! lord gets thee only when flagged (kin, friend, or a rival's slight).

use crate::game::conversation::{Relation, Rng, Speaker};
use crate::game::text::{agreement_faults, authored_lines, authored_lines_by_source,
    realize, tv_for, typed_second_person, Cx};

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
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 60_000, "only {checked} renderings checked");
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
