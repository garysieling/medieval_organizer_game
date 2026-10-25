//! Mythic text corpus: chunked passages with a BM25 inverted index.
//!
//! Layout on disk: `<root>/<tradition>/<id>.txt`, each optionally starting with
//! `Title:` / `Tradition:` header lines (written by scripts/fetch_creativity_corpus.py).

use std::collections::HashMap;
use std::fs;
use std::path::Path;

const TARGET_CHUNK: usize = 900;
const MAX_CHUNK: usize = 1800;
const BM25_K1: f32 = 1.2;
const BM25_B: f32 = 0.75;

/// A retrievable chunk of a source text
#[derive(Debug, Clone)]
pub struct Passage {
    pub source: String,
    pub tradition: String,
    pub text: String,
}

/// In-memory corpus with an inverted index
#[derive(Default)]
pub struct Corpus {
    pub passages: Vec<Passage>,
    postings: HashMap<String, Vec<(u32, u16)>>,
    doc_len: Vec<u32>,
    avg_len: f32,
}

/// Lowercased alphanumeric tokens of length >= 2
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.chars().count() >= 2)
        .map(|t| t.to_lowercase())
        .collect()
}

impl Corpus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load every `.txt` under `root/<tradition>/`. Empty `only` means all traditions.
    pub fn load_dir(root: &str, only: &[String]) -> Result<Self, String> {
        let mut corpus = Corpus::new();
        let mut dirs: Vec<_> = fs::read_dir(root)
            .map_err(|e| format!("Cannot read corpus dir {}: {}", root, e))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        for dir in dirs {
            let tradition = dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if tradition.starts_with('.')
                || (!only.is_empty() && !only.iter().any(|t| t == &tradition))
            {
                continue;
            }
            let mut files: Vec<_> = fs::read_dir(&dir)
                .map_err(|e| e.to_string())?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|x| x == "txt").unwrap_or(false))
                .collect();
            files.sort();
            for file in files {
                let raw = fs::read(&file).map_err(|e| e.to_string())?;
                let text = String::from_utf8_lossy(&raw);
                corpus.add_document(&text, &tradition, &file);
            }
        }
        corpus.finish();
        Ok(corpus)
    }

    /// Build from in-memory `(source, tradition, text)` triples (tests, small demos)
    pub fn from_texts(docs: &[(&str, &str, &str)]) -> Self {
        let mut corpus = Corpus::new();
        for (source, tradition, text) in docs {
            for chunk in chunk_text(text) {
                corpus.push(Passage {
                    source: source.to_string(),
                    tradition: tradition.to_string(),
                    text: chunk,
                });
            }
        }
        corpus.finish();
        corpus
    }

    fn add_document(&mut self, text: &str, dir_tradition: &str, path: &Path) {
        let mut title = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let mut tradition = dir_tradition.to_string();
        let mut body_start = 0;
        for line in text.lines().take(8) {
            if let Some(v) = line.strip_prefix("Title:") {
                title = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("Tradition:") {
                tradition = v.trim().to_string();
            } else if line.trim().is_empty() && body_start > 0 {
                body_start += line.len() + 1;
                break;
            } else if !line.contains(':') {
                break;
            }
            body_start += line.len() + 1;
        }
        let body = text.get(body_start.min(text.len())..).unwrap_or(text);
        for chunk in chunk_text(body) {
            self.push(Passage {
                source: title.clone(),
                tradition: tradition.clone(),
                text: chunk,
            });
        }
    }

    fn push(&mut self, passage: Passage) {
        let id = self.passages.len() as u32;
        let mut len = 0u32;
        let mut counts: HashMap<String, u16> = HashMap::new();
        for tok in tokenize(&passage.text) {
            let c = counts.entry(tok).or_insert(0);
            *c = c.saturating_add(1);
            len += 1;
        }
        for (tok, c) in counts {
            self.postings.entry(tok).or_default().push((id, c));
        }
        self.doc_len.push(len);
        self.passages.push(passage);
    }

    fn finish(&mut self) {
        let total: u64 = self.doc_len.iter().map(|&l| l as u64).sum();
        self.avg_len = if self.doc_len.is_empty() {
            1.0
        } else {
            total as f32 / self.doc_len.len() as f32
        };
    }

    pub fn len(&self) -> usize {
        self.passages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.passages.is_empty()
    }

    /// Number of passages containing `term` (lowercase)
    pub fn doc_freq(&self, term: &str) -> usize {
        self.postings.get(term).map(|p| p.len()).unwrap_or(0)
    }

    /// Distinct traditions present, sorted
    pub fn traditions(&self) -> Vec<String> {
        let mut t: Vec<String> = self.passages.iter().map(|p| p.tradition.clone()).collect();
        t.sort();
        t.dedup();
        t
    }

    /// BM25 ranking of passages for the query terms; returns (passage index, score) best-first
    pub fn search(&self, terms: &[String], k: usize) -> Vec<(usize, f32)> {
        let n = self.passages.len() as f32;
        let mut scores: HashMap<u32, f32> = HashMap::new();
        let mut seen = std::collections::HashSet::new();
        for term in terms {
            let term = term.to_lowercase();
            if !seen.insert(term.clone()) {
                continue;
            }
            let Some(list) = self.postings.get(&term) else {
                continue;
            };
            let df = list.len() as f32;
            let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
            for &(doc, tf) in list {
                let tf = tf as f32;
                let dl = self.doc_len[doc as usize] as f32;
                let s = idf * tf * (BM25_K1 + 1.0)
                    / (tf + BM25_K1 * (1.0 - BM25_B + BM25_B * dl / self.avg_len));
                *scores.entry(doc).or_insert(0.0) += s;
            }
        }
        let mut ranked: Vec<(usize, f32)> =
            scores.into_iter().map(|(d, s)| (d as usize, s)).collect();
        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        ranked.truncate(k);
        ranked
    }
}

/// Split text into paragraph-aligned chunks of roughly TARGET_CHUNK chars
pub fn chunk_text(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for para in text.split("\n\n") {
        let para = para.split_whitespace().collect::<Vec<_>>().join(" ");
        if para.is_empty() {
            continue;
        }
        if para.len() > MAX_CHUNK {
            if !current.is_empty() {
                chunks.push(std::mem::take(&mut current));
            }
            let mut piece = String::new();
            for sentence in para.split_inclusive(['.', '!', '?']) {
                piece.push_str(sentence);
                if piece.len() >= TARGET_CHUNK {
                    chunks.push(std::mem::take(&mut piece).trim().to_string());
                }
            }
            if !piece.trim().is_empty() {
                chunks.push(piece.trim().to_string());
            }
            continue;
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(&para);
        if current.len() >= TARGET_CHUNK {
            chunks.push(std::mem::take(&mut current));
        }
    }
    if !current.trim().is_empty() {
        chunks.push(current);
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> Corpus {
        Corpus::from_texts(&[
            ("Volsunga", "norse", "Sigurd slew the dragon Fafnir on the Glittering Heath.\n\nRegin the smith forged the sword Gram."),
            ("Ramayana", "hindu", "Rama went into exile in the forest of Dandaka with Sita.\n\nRavana the rakshasa carried Sita to Lanka."),
        ])
    }

    #[test]
    fn test_chunking_and_traditions() {
        let c = tiny();
        assert!(c.len() >= 2);
        assert_eq!(
            c.traditions(),
            vec!["hindu".to_string(), "norse".to_string()]
        );
    }

    #[test]
    fn test_bm25_prefers_matching_passage() {
        let c = tiny();
        let hits = c.search(&["dragon".into(), "sword".into()], 5);
        assert!(!hits.is_empty());
        assert_eq!(c.passages[hits[0].0].tradition, "norse");
        assert_eq!(c.doc_freq("sita"), 1);
    }

    #[test]
    fn test_long_paragraph_is_split() {
        let long = "The hero walked far. ".repeat(200);
        let chunks = chunk_text(&long);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|c| c.len() <= MAX_CHUNK + 100));
    }
}
