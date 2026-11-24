//! I1's golden traces (#1509): the per-bar records each scenario
//! writes, committed as JSON lines under `tests/golden/<name>.json`.
//! A run compares its fresh trace against the golden and fails naming
//! the scenario and the divergent bar; `GAME2_GOLDEN=update` rewrites
//! the files - the PR that changes them explains the diff. Traces are
//! the cross-machine check: the PCM byte-equality proof runs only
//! inside one process (floating point differs across machines).
mod common;

use serde_json::Value;
use std::path::PathBuf;

fn golden_dir() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// Numbers compare within a hair (round-tripped text is exact on one
/// machine; the epsilon only covers cross-machine float edge cases);
/// strings, ids and hashes are exact.
fn same(a: &Value, b: &Value, path: &str) -> Result<(), String> {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap_or(f64::NAN),
                y.as_f64().unwrap_or(f64::NAN));
            if x == y || (x - y).abs() <= 1e-6 * x.abs().max(1.0) {
                Ok(())
            } else {
                Err(format!("{path}: {x} vs {y}"))
            }
        }
        (Value::Array(x), Value::Array(y)) => {
            if x.len() != y.len() {
                return Err(format!("{path}: len {} vs {}", x.len(), y.len()));
            }
            for (i, (x, y)) in x.iter().zip(y).enumerate() {
                same(x, y, &format!("{path}[{i}]"))?;
            }
            Ok(())
        }
        (Value::Object(x), Value::Object(y)) => {
            if x.len() != y.len() {
                return Err(format!("{path}: keys {} vs {}", x.len(), y.len()));
            }
            for (k, xv) in x {
                let yv = y.get(k).ok_or_else(|| format!("{path}.{k}: missing"))?;
                same(xv, yv, &format!("{path}.{k}"))?;
            }
            Ok(())
        }
        _ => if a == b { Ok(()) } else { Err(format!("{path}: {a} vs {b}")) },
    }
}

/// The rendered trace as JSON lines, one per bar.
fn trace_json(r: &common::Render) -> String {
    let mut buf = vec![];
    game2_audio::trace::write_trace(&mut buf, &r.traces).unwrap();
    String::from_utf8(buf).unwrap()
}

#[test]
fn golden_traces_match() {
    let update = std::env::var("GAME2_GOLDEN").as_deref() == Ok("update");
    assert_eq!(game2_audio::scenarios::registry().len(), 10,
        "the library should hold the 10 named scenarios");
    for sc in game2_audio::scenarios::registry() {
        let r = common::render_scenario(sc.name, 3600);
        let got = trace_json(&r);
        let path = golden_dir().join(format!("{}.json", sc.name));
        if update {
            std::fs::create_dir_all(golden_dir()).unwrap();
            std::fs::write(&path, &got).unwrap();
            eprintln!("golden: wrote {}", path.display());
            continue;
        }
        let want = std::fs::read_to_string(&path).unwrap_or_else(|_|
            panic!("{}: no golden file - run with GAME2_GOLDEN=update", sc.name));
        let (gl, wl) = (got.lines().count(), want.lines().count());
        assert_eq!(gl, wl, "{}: traced {gl} bars, golden holds {wl}", sc.name);
        for (i, (g, w)) in got.lines().zip(want.lines()).enumerate() {
            let g: Value = serde_json::from_str(g).unwrap();
            let w: Value = serde_json::from_str(w).unwrap();
            same(&g, &w, &format!("{} bar {i}", sc.name))
                .unwrap_or_else(|e| panic!(
                    "{}: golden trace diverged ({e}) - if this change is \
                     intended, regenerate with GAME2_GOLDEN=update and \
                     explain the diff in the PR", sc.name));
        }
    }
}

/// PCM is bit-exact only inside one run: two renders of the same
/// scenario in this process must produce the same bytes.
#[test]
fn render_is_bit_identical_within_run() {
    let a = common::render_scenario("assembly_build", 1200);
    let b = common::render_scenario("assembly_build", 1200);
    assert_eq!(a.pcm, b.pcm,
        "the same scenario rendered twice must be bit-identical");
}
