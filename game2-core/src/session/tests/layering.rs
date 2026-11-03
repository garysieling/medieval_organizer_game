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
