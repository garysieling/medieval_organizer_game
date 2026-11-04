//! The layers hold (docs/architecture.md): the render client draws from a
//! `UiModel` alone, never the authoritative `Game` or its ECS `World`.
//! `render/bench.rs` is exempt - the headless runners drive a Game to time it.

/// Source with `//` comments stripped (string literals here don't hold `//`)
fn code_of(src: &str) -> String {
    src.lines().map(|l| l.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n")
}

fn mentions(code: &str, word: &str) -> bool {
    code.match_indices(word).any(|(i, _)| {
        let before = code[..i].chars().next_back();
        let after = code[i + word.len()..].chars().next();
        let ident = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
        !ident(before) && !ident(after)
    })
}

#[test]
fn render_and_public_kpis_never_reach_into_the_sim() {
    let files = [
        ("render/art.rs", include_str!("../render/art.rs")),
        ("render/draw.rs", include_str!("../render/draw.rs")),
        ("render/hud.rs", include_str!("../render/hud.rs")),
        ("render/view.rs", include_str!("../render/view.rs")),
        // public KPIs read a seat's view, never the sim
        ("kpi/public.rs", include_str!("../kpi/public.rs")),
    ];
    for (name, src) in files {
        let code = code_of(src);
        for word in ["Game", "World", "world", "Scheduler"] {
            assert!(!mentions(&code, word),
                "{name} mentions `{word}`: the render layer and public KPIs read UiModel/AgentObs only");
        }
    }
}

/// Sim state iterates its maps (fire, trails, the probe's lists, draw
/// order), so it must use `engine::det`'s fixed-key maps: std's are seeded
/// randomly per map and two runs of one seed would diverge.
#[test]
fn sim_code_uses_deterministic_maps() {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() { walk(&p, out) } else if p.extension().is_some_and(|x| x == "rs") { out.push(p) }
        }
    }
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = vec![src.join("session.rs")];
    for d in ["game", "sim", "session", "engine"] { walk(&src.join(d), &mut files); }
    let mut bad = vec![];
    for f in &files {
        if f.ends_with("engine/det.rs") || f.ends_with("layering.rs") { continue; }
        let code = code_of(&std::fs::read_to_string(f).unwrap());
        let random_map = code.contains("std::collections::HashMap") || code.contains("std::collections::HashSet")
            || code.lines().any(|l| l.contains("use std::collections::{") && (l.contains("HashMap") || l.contains("HashSet")));
        if random_map { bad.push(f.strip_prefix(&src).unwrap_or(f).display().to_string()); }
    }
    assert!(bad.is_empty(), "use crate::engine::det::{{HashMap, HashSet}} (fixed-key) in sim code, not std's: {bad:?}");
}
