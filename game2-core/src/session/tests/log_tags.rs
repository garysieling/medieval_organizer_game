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
