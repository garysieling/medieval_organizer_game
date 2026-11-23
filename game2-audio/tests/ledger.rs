//! The license gate (#1495, A1): temp-dir fixtures drive
//! `check_ledger` through each violation, and `ledger_covers_every_sound`
//! also runs the real `assets/sound/` of the repo.
use game2_audio::ledger::{check_ledger, Violation, BUDGET_BYTES};
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("game2-ledger-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                .unwrap().as_nanos()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn put(dir: &Path, rel: &str, bytes: &[u8]) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, bytes).unwrap();
}

fn sha(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(&std::fs::read(path).unwrap()))
}

fn entry(dir: &Path, id: &str, rel: &str, license: &str, grant: &str) -> String {
    format!(r#"{{ "id": "{id}", "path": "{rel}", "source_url": "https://x/{id}",
        "author": "a", "license": "{license}", "license_evidence_url": "https://l",
        "sha256": "{}", "processing": "test", "grant_quote": "{grant}" }}"#,
        sha(&dir.join(rel)))
}

fn ledger(dir: &Path, entries: &[String]) {
    std::fs::write(dir.join("LEDGER.json"),
        format!("[\n{}\n]", entries.join(",\n"))).unwrap();
}

#[test]
fn ledger_covers_every_sound() {
    // a temp fixture first: one ledgered file passes clean
    let dir = fixture();
    put(&dir, "beds/river.ogg", b"fake ogg bytes");
    ledger(&dir, &[entry(&dir, "river", "beds/river.ogg", "CC0-1.0", "")]);
    assert!(check_ledger(&dir).is_empty(), "a ledgered file is clean");
    let _ = std::fs::remove_dir_all(&dir);

    // the real repo's assets/sound must pass on its own tree
    let real = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/sound");
    let v = check_ledger(&real);
    assert!(v.is_empty(), "assets/sound fails the gate: {v:?}");
}

#[test]
fn ledger_rejects_non_cc0_license() {
    let dir = fixture();
    put(&dir, "oneshots/bell.ogg", b"bell");
    ledger(&dir, &[entry(&dir, "bell", "oneshots/bell.ogg", "CC-BY-4.0", "")]);
    assert!(check_ledger(&dir).iter().any(|v| matches!(v,
        Violation::BadLicense { license, .. } if license == "CC-BY-4.0")),
        "CC-BY is redistribution-limited, off the list");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn ledger_rejects_unrestricted_without_grant() {
    let dir = fixture();
    put(&dir, "instruments/viol.ogg", b"violin");
    ledger(&dir, &[entry(&dir, "viol", "instruments/viol.ogg", "unrestricted", "")]);
    assert_eq!(check_ledger(&dir), vec![Violation::UnrestrictedNoQuote {
        id: "viol".into() }], "a bare 'unrestricted' isn't evidence");
    // with the grant quoted it passes
    ledger(&dir, &[entry(&dir, "viol", "instruments/viol.ogg", "unrestricted",
        "may be used without restriction")]);
    assert!(check_ledger(&dir).is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn ledger_flags_hash_mismatch() {
    let dir = fixture();
    put(&dir, "beds/wind.ogg", b"first bytes");
    ledger(&dir, &[entry(&dir, "wind", "beds/wind.ogg", "CC0-1.0", "")]);
    // the file changed under the ledger's feet
    std::fs::write(dir.join("beds/wind.ogg"), b"different bytes").unwrap();
    assert_eq!(check_ledger(&dir), vec![Violation::HashMismatch { id: "wind".into() }]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn ledger_flags_missing_file() {
    let dir = fixture();
    put(&dir, "oneshots/door.ogg", b"door");
    ledger(&dir, &[entry(&dir, "door", "oneshots/door.ogg", "CC0-1.0", ""),
        r#"{ "id": "ghost", "path": "oneshots/ghost.ogg", "source_url": "https://x",
        "author": "a", "license": "CC0-1.0", "license_evidence_url": "https://l",
        "sha256": "deadbeef", "processing": "test" }"#.into()]);
    // ghost.ogg never landed: the entry can't hash it, but the flag is
    // the missing file
    let v = check_ledger(&dir);
    assert!(v.contains(&Violation::MissingFile("oneshots/ghost.ogg".into())),
        "an entry without a file: {v:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn ledger_flags_unledgered_file() {
    let dir = fixture();
    put(&dir, "beds/rain.ogg", b"rain");
    put(&dir, "beds/sneaky.ogg", b"no ledger for me");
    ledger(&dir, &[entry(&dir, "rain", "beds/rain.ogg", "CC0-1.0", "")]);
    assert_eq!(check_ledger(&dir),
        vec![Violation::Unledgered("beds/sneaky.ogg".into())]);
    // and _incoming/ + the paperwork don't count as unledgered sounds
    put(&dir, "_incoming/12345.ogg", b"raw fetch");
    assert_eq!(check_ledger(&dir),
        vec![Violation::Unledgered("beds/sneaky.ogg".into())]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn budget_20mb_enforced() {
    let dir = fixture();
    // a sparse 21 MB file: write once at the end, no real bytes needed
    let big = dir.join("beds/huge.ogg");
    std::fs::create_dir_all(big.parent().unwrap()).unwrap();
    std::fs::write(&big, b"x").unwrap();
    let f = std::fs::OpenOptions::new().write(true).open(&big).unwrap();
    f.set_len(BUDGET_BYTES + 1024).unwrap();
    drop(f);
    // the ledger can't hold the real hash of a sparse file twice -
    // compute it after the stretch
    ledger(&dir, &[entry(&dir, "huge", "beds/huge.ogg", "own-synth", "")]);
    let v = check_ledger(&dir);
    assert!(v.iter().any(|x| matches!(x, Violation::OverBudget { .. })),
        "21 MB is over the 20 MB budget: {v:?}");
    let _ = std::fs::remove_dir_all(&dir);
}
