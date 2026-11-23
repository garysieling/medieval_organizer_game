//! The audio layer holds (wiki/software-design/sound-system.md): the
//! crate reads a seat's public view, never the sim - no `Game`, no
//! `World`, no `probe` - and no clocks or OS randomness. `src/scenarios.rs`
//! is the one allowlisted file: the scenario library is a test harness
//! by design (feature `scenarios`).

/// Source with `//` comments stripped (string literals here don't hold `//`)
fn code_of(src: &str) -> String {
    src.lines().map(|l| l.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n")
}

/// The file with `#[cfg(test)]` items removed: a `#[cfg(test)]` attribute
/// starts a skip that ends at the next column-0 `}` (test mods are
/// top-level). Good enough for a lint - it can only weaken, never accuse.
fn without_tests(src: &str) -> String {
    let mut out = String::new();
    let mut skipping = false;
    for l in src.lines() {
        if l.contains("#[cfg(test)]") { skipping = true; }
        if !skipping { out.push_str(l); out.push('\n'); }
        else if l == "}" { skipping = false; }
    }
    out
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
fn the_audio_crate_never_reaches_the_sim_or_the_clock() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() { walk(&p, out) }
            else if p.extension().is_some_and(|x| x == "rs") { out.push(p) }
        }
    }
    let mut files = vec![];
    walk(&src, &mut files);
    assert!(!files.is_empty());
    for f in &files {
        if f.file_name().is_some_and(|n| n == "scenarios.rs") { continue; } // the test harness
        let code = without_tests(&code_of(&std::fs::read_to_string(f).unwrap()));
        for word in ["Game", "World", "probe", "SystemTime", "Instant", "getrandom"] {
            assert!(!mentions(&code, word),
                "{} mentions `{word}`: audio reads UiModel only, no sim, no clock",
                f.strip_prefix(&src).unwrap_or(f).display());
        }
        for path in ["std::time", "rand::"] {
            assert!(!code.contains(path),
                "{} mentions `{path}`: randomness comes from rng::Rng, and there are no clocks",
                f.strip_prefix(&src).unwrap_or(f).display());
        }
    }
}

/// The sim must never depend on the audio crate - the dependency arrow
/// points one way only.
#[test]
fn game2_core_does_not_depend_on_game2_audio() {
    let core = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../game2-core/Cargo.toml");
    let toml = std::fs::read_to_string(&core).unwrap();
    assert!(!toml.contains("game2-audio"),
        "game2-core/Cargo.toml mentions game2-audio - the arrow points the other way");
}
