//! The asset ledger gate (#1495, A1): every sound file in this public
//! repo must be CC0 or our own synthesis, and provably so - committing
//! a file redistributes it as-is. `assets/sound/LEDGER.json` is the
//! ledger, mirroring `art-prompts/generated.json`; `check_ledger` is the
//! test's hammer. Nothing here touches audio rendering.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Decision 7's cap: 20 MB of OGG at most - the web build downloads
/// them all
pub const BUDGET_BYTES: u64 = 20 * 1024 * 1024;

/// The licences allowed in the repo (decision 6): CC0, a quoted
/// "unrestricted" grant, or our own synthesis
pub const ALLOWED_LICENSES: [&str; 3] = ["CC0-1.0", "unrestricted", "own-synth"];

/// One line of `LEDGER.json`. `grant_quote` is required only when the
/// licence is `unrestricted` - the grant's own words are the evidence
#[derive(Debug, Deserialize)]
pub struct Entry {
    pub id: String,
    /// Path under `assets/sound/` (forward slashes)
    pub path: String,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub author: String,
    pub license: String,
    #[serde(default)]
    pub license_evidence_url: String,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub processing: String,
    #[serde(default)]
    pub grant_quote: String,
}

/// A way the ledger can fail the build
#[derive(Debug, PartialEq)]
pub enum Violation {
    /// `LEDGER.json` itself won't parse
    Unparseable(String),
    /// A file under `assets/sound/` (skipping README, the ledger and
    /// `_incoming/`) with no entry for it
    Unledgered(String),
    /// An entry whose file isn't on disk
    MissingFile(String),
    /// A licence off the allowlist
    BadLicense { id: String, license: String },
    /// `unrestricted` without the grant quoted
    UnrestrictedNoQuote { id: String },
    /// The file's sha256 isn't what the ledger swears to
    HashMismatch { id: String },
    /// The whole `assets/sound/` tree over the 20 MB budget
    OverBudget { total: u64 },
}

fn sha256_of(path: &Path) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    Some(format!("{:x}", Sha256::digest(&data)))
}

/// Every file under `dir`, recursively, as forward-slash paths relative
/// to `dir`
fn walk(dir: &Path, at: &Path, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(at) else { return };
    let mut names: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
    names.sort();
    for p in names {
        if p.is_dir() { walk(dir, &p, out); continue; }
        let rel = p.strip_prefix(dir).unwrap_or(&p)
            .to_string_lossy().replace('\\', "/");
        // the ledger's own paperwork and the untracked staging area
        if rel == "LEDGER.json" || rel == "README.md" || rel == "manifest.json"
            || rel.starts_with("_incoming/") { continue; }
        out.push(rel);
    }
}

/// The gate: check `sound_dir` (`assets/sound/`) against its ledger.
/// An empty vec is a pass; every violation found is a failure the test
/// names plainly.
pub fn check_ledger(sound_dir: &Path) -> Vec<Violation> {
    let mut out = vec![];
    let ledger_path = sound_dir.join("LEDGER.json");
    let entries: Vec<Entry> = match std::fs::read_to_string(&ledger_path)
        .map_err(|e| e.to_string())
        .and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string())) {
        Ok(e) => e,
        Err(e) => { out.push(Violation::Unparseable(e)); return out; }
    };

    // every file has an entry
    let mut files = vec![];
    walk(sound_dir, sound_dir, &mut files);
    for f in &files {
        if !entries.iter().any(|e| e.path == *f) {
            out.push(Violation::Unledgered(f.clone()));
        }
    }

    // every entry's file exists, is licensed, and hashes true
    let mut total = 0u64;
    for e in &entries {
        if !ALLOWED_LICENSES.contains(&e.license.as_str()) {
            out.push(Violation::BadLicense {
                id: e.id.clone(), license: e.license.clone() });
        } else if e.license == "unrestricted" && e.grant_quote.trim().is_empty() {
            out.push(Violation::UnrestrictedNoQuote { id: e.id.clone() });
        }
        let path = sound_dir.join(&e.path);
        if !path.is_file() {
            out.push(Violation::MissingFile(e.path.clone()));
            continue;
        }
        total += path.metadata().map(|m| m.len()).unwrap_or(0);
        let ok = sha256_of(&path).is_some_and(|h| h == e.sha256);
        if !ok { out.push(Violation::HashMismatch { id: e.id.clone() }); }
    }

    if total > BUDGET_BYTES { out.push(Violation::OverBudget { total }); }
    out
}
