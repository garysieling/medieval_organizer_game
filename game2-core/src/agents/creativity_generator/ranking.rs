//! Candidate pool, LLM output parsing, obviousness scoring, and the head cut:
//! the most predictable ideas (as judged by the LLM + heuristics) are dropped.

use super::extract::ConceptKind;
use super::randomness::Rng;

/// Where a candidate came from
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    Corpus,
    Llm,
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub text: String,
    pub note: String,
    pub source: String,
    pub tradition: String,
    pub origin: Origin,
    /// Times this idea was produced independently (LLM batches / corpus hits)
    pub votes: usize,
    /// 0 = most predictable, 1 = most surprising, per the LLM ranker
    pub llm_surprise: Option<f32>,
    /// Higher = more obvious; computed by `score`
    pub obviousness: f32,
}

/// Names, words, and tropes LLMs (and fantasy) reach for first
pub const CLICHES: &[&str] = &[
    "gandalf",
    "aragorn",
    "frodo",
    "legolas",
    "gimli",
    "elrond",
    "sauron",
    "galadriel",
    "bilbo",
    "merlin",
    "arthur",
    "lancelot",
    "conan",
    "drizzt",
    "elara",
    "lyra",
    "kael",
    "kaelen",
    "seraphina",
    "thorne",
    "aldric",
    "elowen",
    "zephyr",
    "sylas",
    "vex",
    "rowan",
    "lysander",
    "isolde",
    "eldrin",
    "theron",
    "aelric",
    "corvin",
    "mira",
    "elric",
    "aria",
    "luna",
    "nyx",
    "draven",
    "raven",
    "shadow",
    "ember",
    "aurora",
    "valerius",
    "caelan",
    "finnian",
    "silas",
    "blacksmith",
    "farmer",
    "innkeeper",
    "chosen one",
    "dark lord",
    "ancient evil",
    "long-lost heir",
    "lost heir",
    "secretly the king",
    "amulet of",
    "orb of",
    "shadowfell",
    "eldoria",
    "whispering woods",
    "evil sorcerer",
];

/// Normalised dedupe key
pub fn normalize_key(text: &str, kind: &ConceptKind) -> String {
    let base = if kind.is_short_form() {
        text.split(" - ").next().unwrap_or(text)
    } else {
        text
    };
    base.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse a numbered / bulleted LLM list into (text, note) pairs
pub fn parse_list(output: &str, kind: &ConceptKind) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in output.lines() {
        let l = line.trim();
        let stripped = l.trim_start_matches(|c: char| {
            c.is_ascii_digit() || matches!(c, '.' | ')' | '-' | '*' | '•' | '#' | ' ' | '\t')
        });
        if stripped.len() == l.len()
            && !l
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            && out.is_empty()
            && l.ends_with(':')
        {
            continue;
        }
        let item = stripped
            .replace("**", "")
            .replace("__", "")
            .trim()
            .to_string();
        if item.len() < 2 || item.to_lowercase().starts_with("here are") || item.ends_with(':') {
            continue;
        }
        if kind.is_short_form() {
            let (name, note) = split_note(&item);
            if name.is_empty() || name.split_whitespace().count() > 6 {
                continue;
            }
            out.push((name, note));
        } else {
            out.push((item, String::new()));
        }
    }
    out
}

fn split_note(item: &str) -> (String, String) {
    for sep in [" - ", " — ", " – ", ": "] {
        if let Some((a, b)) = item.split_once(sep) {
            return (a.trim().trim_matches('"').to_string(), b.trim().to_string());
        }
    }
    (item.trim_matches('"').to_string(), String::new())
}

/// Insert or merge a candidate (merging bumps votes)
pub fn merge(pool: &mut Vec<Candidate>, cand: Candidate, kind: &ConceptKind) {
    let key = normalize_key(&cand.text, kind);
    if key.is_empty() {
        return;
    }
    if let Some(existing) = pool
        .iter_mut()
        .find(|c| normalize_key(&c.text, kind) == key)
    {
        existing.votes += cand.votes;
        if existing.note.is_empty() {
            existing.note = cand.note;
        }
        return;
    }
    pool.push(cand);
}

pub fn is_cliche(text: &str) -> bool {
    let t = text.to_lowercase();
    CLICHES.iter().any(|c| {
        if c.contains(' ') {
            t.contains(c)
        } else {
            t.split(|ch: char| !ch.is_alphanumeric()).any(|w| w == *c)
        }
    })
}

/// Prompt asking the LLM to order items from most predictable to most surprising
pub fn rank_prompt(kind: &ConceptKind, genre: &str, items: &[&Candidate]) -> String {
    let mut s = format!(
        "You are judging {} ideas for {}.\nOrder ALL the items below from MOST predictable, generic or cliched (first) to MOST surprising, fresh and evocative (last).\nReply with only the item numbers, comma-separated, most predictable first. No commentary.\n\n",
        kind.label(),
        genre
    );
    for (i, c) in items.iter().enumerate() {
        s.push_str(&format!("{}. {}\n", i + 1, c.text));
    }
    s
}

/// Parse "3, 7, 1, ..." into zero-based indices (deduped, in range)
pub fn parse_rank_order(output: &str, n: usize) -> Vec<usize> {
    let mut seen = vec![false; n];
    let mut order = Vec::new();
    for tok in output.split(|c: char| !c.is_ascii_digit()) {
        if let Ok(v) = tok.parse::<usize>() {
            if v >= 1 && v <= n && !seen[v - 1] {
                seen[v - 1] = true;
                order.push(v - 1);
            }
        }
    }
    order
}

/// Apply a parsed order to a batch: first = surprise 0.0, last = 1.0
pub fn apply_rank(batch: &mut [&mut Candidate], order: &[usize]) {
    if order.len() < 2 {
        return;
    }
    let denom = (order.len() - 1) as f32;
    for (pos, &idx) in order.iter().enumerate() {
        if let Some(c) = batch.get_mut(idx) {
            c.llm_surprise = Some(pos as f32 / denom);
        }
    }
}

/// Combine LLM ranking, consensus, corpus frequency, and cliche hits into `obviousness`
pub fn score(pool: &mut [Candidate], corpus_freq: &dyn Fn(&Candidate) -> f32) {
    let max_votes = pool.iter().map(|c| c.votes).max().unwrap_or(1).max(2) as f32;
    for c in pool.iter_mut() {
        let llm = 1.0 - c.llm_surprise.unwrap_or(0.5);
        let consensus = (c.votes.saturating_sub(1)) as f32 / (max_votes - 1.0);
        let freq = corpus_freq(c).clamp(0.0, 1.0);
        let cliche = if is_cliche(&c.text) { 1.0 } else { 0.0 };
        c.obviousness = 0.55 * llm + 0.25 * consensus + 0.2 * freq + cliche;
    }
}

/// Drop the most obvious `head_cut` fraction, then take `count` from what's left,
/// filling `corpus_share` from corpus-mined candidates. Returns (kept, cut).
pub fn select(
    mut pool: Vec<Candidate>,
    count: usize,
    head_cut: f32,
    corpus_share: f32,
    rng: &mut Rng,
) -> (Vec<Candidate>, Vec<Candidate>) {
    // small jitter so ties and near-ties break randomly
    let mut keyed: Vec<(f32, Candidate)> = pool
        .drain(..)
        .map(|c| (c.obviousness + rng.next_f32() * 0.05, c))
        .collect();
    keyed.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let n_cut = ((keyed.len() as f32) * head_cut.clamp(0.0, 0.95)).round() as usize;
    let mut cut = Vec::new();
    let mut rest = Vec::new();
    for (i, (_, c)) in keyed.into_iter().enumerate() {
        if i < n_cut || c.obviousness >= 1.0 {
            cut.push(c);
        } else {
            rest.push(c);
        }
    }
    let want_corpus = ((count as f32) * corpus_share.clamp(0.0, 1.0)).round() as usize;
    let (corpus, llm): (Vec<Candidate>, Vec<Candidate>) =
        rest.into_iter().partition(|c| c.origin == Origin::Corpus);
    let n_corpus = want_corpus
        .min(corpus.len())
        .max(count.saturating_sub(llm.len()).min(corpus.len()));
    let n_llm = count.saturating_sub(n_corpus).min(llm.len());
    let mut kept: Vec<Candidate> = corpus
        .into_iter()
        .take(n_corpus)
        .chain(llm.into_iter().take(n_llm))
        .collect();
    rng.shuffle(&mut kept);
    (kept, cut)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(text: &str, origin: Origin) -> Candidate {
        Candidate {
            text: text.into(),
            note: String::new(),
            source: String::new(),
            tradition: String::new(),
            origin,
            votes: 1,
            llm_surprise: None,
            obviousness: 0.0,
        }
    }

    #[test]
    fn test_parse_list_variants() {
        let out = "Here are 3 names:\n1. **Hrothmund** - a ferryman of the dead\n2) Sitavarna: a temple dancer\n- Kauaʻula — wind spirit\n";
        let v = parse_list(out, &ConceptKind::CharacterName);
        assert_eq!(v.len(), 3);
        assert_eq!(
            v[0],
            (
                "Hrothmund".to_string(),
                "a ferryman of the dead".to_string()
            )
        );
        assert_eq!(v[1].0, "Sitavarna");
        assert_eq!(v[2].0, "Kauaʻula");
        let p = parse_list(
            "1. The miller's daughter sells her shadow to a river god.",
            &ConceptKind::PlotPoint,
        );
        assert_eq!(
            p[0].0,
            "The miller's daughter sells her shadow to a river god."
        );
    }

    #[test]
    fn test_merge_counts_votes() {
        let k = ConceptKind::CharacterName;
        let mut pool = Vec::new();
        merge(&mut pool, cand("Hrothmund", Origin::Llm), &k);
        merge(&mut pool, cand("hrothmund!", Origin::Llm), &k);
        assert_eq!(pool.len(), 1);
        assert_eq!(pool[0].votes, 2);
    }

    #[test]
    fn test_rank_order_parse_and_apply() {
        assert_eq!(parse_rank_order("3, 1, 3, 9, 2", 3), vec![2, 0, 1]);
        let mut a = cand("a", Origin::Llm);
        let mut b = cand("b", Origin::Llm);
        {
            let mut batch = vec![&mut a, &mut b];
            apply_rank(&mut batch, &[1, 0]);
        }
        assert_eq!(b.llm_surprise, Some(0.0));
        assert_eq!(a.llm_surprise, Some(1.0));
    }

    #[test]
    fn test_head_cut_removes_obvious_and_cliches() {
        let mut pool: Vec<Candidate> = (0..20)
            .map(|i| {
                let mut c = cand(
                    &format!("idea{}", i),
                    if i % 2 == 0 {
                        Origin::Corpus
                    } else {
                        Origin::Llm
                    },
                );
                c.llm_surprise = Some(i as f32 / 19.0);
                c
            })
            .collect();
        pool.push(cand("Elara", Origin::Llm));
        score(&mut pool, &|_| 0.0);
        let (kept, cut) = select(pool, 10, 0.2, 0.5, &mut Rng::new(3));
        assert_eq!(kept.len(), 10);
        assert!(cut.iter().any(|c| c.text == "Elara"));
        assert!(cut.iter().any(|c| c.text == "idea0"));
        assert!(!kept.iter().any(|c| c.text == "idea0" || c.text == "idea1"));
        assert!(kept.iter().filter(|c| c.origin == Origin::Corpus).count() >= 4);
    }
}
