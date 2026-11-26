//! #1132's coverage guard: every timestamped log line the village writes
//! must carry a story tag. The balance tester scores runs off the tags,
//! not the strings - an untagged "t{n}:" line is invisible to it, and the
//! scorer's vocabulary silently drifts blind again. Two checks: the
//! sources can't write one, and a real run doesn't produce one.

use crate::session::*;

/// A log line's tick stamp: "t<n>:" at the front.
fn stamped(l: &str) -> bool {
    l.starts_with('t')
        && l.chars()
            .nth(1)
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        && l.contains(':')
}

/// Runtime coverage: a seeded run's log has no untagged t{n}: lines.
/// The window rolls, so this reads the lines present at day's end - new
/// phrasings are the ones that matter, and they're all recent.
#[test]
fn every_timestamped_line_carries_a_tag() {
    let mut g = Game::new();
    while g.tick < 25 {
        g.update();
    }
    let untagged: Vec<&String> = (0..g.log.len())
        .filter(|&i| stamped(&g.log[i]) && g.log.ev(i).is_none())
        .map(|i| &g.log[i])
        .collect();
    assert!(untagged.is_empty(),
        "untagged timestamped log lines - these events are invisible to the balance tester:\n{untagged:?}");
}

/// Source coverage, the layering-test idiom: no source file may push a
/// timestamped line through the plain (untagged) push API. Tagged pushes
/// (`push_ev*`) are the only way a `t{n}:` line reaches the log.
#[test]
fn no_source_pushes_a_timestamped_line_untagged() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = vec![];
    let mut stack = vec![dir];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            if p.components().any(|c| c.as_os_str() == "tests") {
                continue;
            }
            let src = std::fs::read_to_string(&p).unwrap();
            for (n, line) in src.lines().enumerate() {
                let l = line.trim();
                // an untagged push writing a "t{...}:" format is the leak
                if (l.contains(".push(")
                    || l.contains(".push_at(")
                    || l.contains(".push_who(")
                    || l.contains(".push_at_who("))
                    && l.contains("format!(\"t{")
                {
                    offenders.push(format!("{}:{}: {}", p.display(), n + 1, l));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "timestamped lines written through the untagged push API - use push_ev*:\n{}",
        offenders.join("\n")
    );
}

/// A log line's tick stamp parsed back: "t<n>:" -> n.
fn tick_of(l: &str) -> Option<u64> {
    l.strip_prefix('t')?.split(':').next()?.parse().ok()
}

/// One event, one line (#1696): a villager's passing gets one telling a
/// tick - a death can't run once from an event's own line and again
/// from the frame check, a foundling can't be a basket and a birth
/// both. The tags make it checkable: same tick, same mouth, same
/// thread = the same event told twice. Mouth-less lines legitimately
/// crowd a tick (the holy day's four beats, the visitors' comings and
/// goings), so only `who`-carrying lines are keyed.
#[test]
fn no_event_is_logged_twice() {
    let mut g = Game::new();
    while g.tick < 40 {
        g.update();
    }
    let mut seen: crate::engine::det::HashMap<(u64, usize, Thread), Vec<usize>> =
        Default::default();
    for i in 0..g.log.len() {
        let (Some(tick), Some(ev), Some(who)) =
            (tick_of(&g.log[i]), g.log.ev(i), g.log.meta(i).who)
        else {
            continue;
        };
        seen.entry((tick, who, ev.thread)).or_default().push(i);
    }
    let doubled: Vec<Vec<String>> = seen
        .values()
        .filter(|v| v.len() > 1)
        .map(|v| v.iter().map(|&i| g.log[i].clone()).collect())
        .collect();
    assert!(doubled.is_empty(),
        "same tick, same mouth, same braid - the event told twice:\n{doubled:?}");
}

/// The repro itself (#1696): a foundling is one event - the basket's
/// telling stands in for the crib's, not alongside it
#[test]
fn a_foundling_is_one_event_one_line() {
    let mut g = Game::new();
    g.force_event(crate::session::events::EventId::Foundling);
    let born: Vec<&String> = g
        .log
        .iter()
        .filter(|l| l.contains("foundling") || l.contains("is born"))
        .collect();
    assert_eq!(born.len(), 1, "one foundling, one line: {born:?}");
}

/// ...and a death (#1696): the frame check's telling is the only one -
/// the manner's own line, once
#[test]
fn a_death_is_one_event_one_line() {
    let mut g = Game::new();
    let v = g.villagers[0];
    if let Some(h) = g.world.get_component_mut::<Health>(v) {
        h.condition = 0.0;
    }
    while g.tick < 3 {
        g.update();
    }
    let name = g.cast_name(0);
    let deaths: Vec<&String> = g
        .log
        .iter()
        .filter(|l| l.contains(&name) && l.contains("die"))
        .collect();
    assert_eq!(deaths.len(), 1, "one death, one line: {deaths:?}");
}
