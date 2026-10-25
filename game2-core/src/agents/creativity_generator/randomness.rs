//! Sources of randomness: a seeded RNG, random numbers, rare corpus words,
//! and words off the internet (Google Trends, Wikipedia top/random, Wiktionary).

use super::corpus::{tokenize, Corpus};
use crate::infra::ollama::json;
use std::process::Command;

const USER_AGENT: &str = "game2-creativity-generator/0.1";

/// Small deterministic RNG (SplitMix64); no external crates
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    /// Seed from the wall clock
    pub fn from_time() -> Self {
        Rng::new(time_seed())
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1)
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform in [0, n)
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i + 1);
            items.swap(i, j);
        }
    }
}

pub fn time_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42)
}

/// Random material injected into retrieval and prompts
#[derive(Debug, Clone, Default)]
pub struct Sparks {
    pub numbers: Vec<u32>,
    pub corpus_words: Vec<String>,
    pub trending: Vec<String>,
    pub random_titles: Vec<String>,
    pub random_words: Vec<String>,
    pub errors: Vec<String>,
}

impl Sparks {
    /// A random subset of sparks rendered for a prompt
    pub fn prompt_line(&self, rng: &mut Rng) -> String {
        let mut parts = Vec::new();
        let mut pick = |label: &str, items: &[String], n: usize, rng: &mut Rng| {
            if items.is_empty() {
                return;
            }
            let mut v: Vec<String> = items.to_vec();
            rng.shuffle(&mut v);
            v.truncate(n);
            parts.push(format!("{}: {}", label, v.join(", ")));
        };
        let nums: Vec<String> = self.numbers.iter().map(|n| n.to_string()).collect();
        pick("numbers", &nums, 2, rng);
        pick("old words", &self.corpus_words, 4, rng);
        pick("trending today", &self.trending, 2, rng);
        pick("random encyclopedia entries", &self.random_titles, 2, rng);
        pick("random dictionary words", &self.random_words, 2, rng);
        parts.join("; ")
    }

    pub fn total(&self) -> usize {
        self.numbers.len()
            + self.corpus_words.len()
            + self.trending.len()
            + self.random_titles.len()
            + self.random_words.len()
    }
}

/// Gather all spark sources. `online` enables internet sources (best-effort; failures recorded in `errors`).
pub fn gather(corpus: &Corpus, rng: &mut Rng, online: bool) -> Sparks {
    let mut sparks = Sparks {
        numbers: (0..8).map(|_| (rng.next_u64() % 10_000) as u32).collect(),
        corpus_words: random_corpus_words(corpus, rng, 16),
        ..Default::default()
    };
    if online {
        let take = |r: Result<Vec<String>, String>, label: &str, errors: &mut Vec<String>| match r {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("{}: {}", label, e));
                vec![]
            }
        };
        let mut errors = Vec::new();
        let mut trending = take(fetch_google_trends(15), "google trends", &mut errors);
        trending.extend(take(fetch_wikipedia_top(15), "wikipedia top", &mut errors));
        sparks.trending = trending;
        sparks.random_titles = take(
            fetch_wiki_random("en.wikipedia.org", 10),
            "wikipedia random",
            &mut errors,
        );
        sparks.random_words = take(
            fetch_wiki_random("en.wiktionary.org", 30),
            "wiktionary random",
            &mut errors,
        )
        .into_iter()
        .filter(|w| {
            w.chars()
                .all(|c| c.is_ascii_alphabetic() || c == ' ' || c == '-')
        })
        .take(10)
        .collect();
        sparks.errors = errors;
    }
    sparks
}

/// Uncommon-looking words drawn from random passages
pub fn random_corpus_words(corpus: &Corpus, rng: &mut Rng, n: usize) -> Vec<String> {
    let mut out = Vec::new();
    if corpus.is_empty() {
        return out;
    }
    let mut attempts = 0;
    while out.len() < n && attempts < n * 20 {
        attempts += 1;
        let p = &corpus.passages[rng.below(corpus.len())];
        let toks: Vec<String> = tokenize(&p.text)
            .into_iter()
            .filter(|t| t.len() >= 6 && t.chars().all(|c| c.is_alphabetic()))
            .collect();
        if toks.is_empty() {
            continue;
        }
        let w = toks[rng.below(toks.len())].clone();
        if corpus.doc_freq(&w) <= corpus.len() / 200 + 2 && !out.contains(&w) {
            out.push(w);
        }
    }
    out
}

fn curl_get(url: &str) -> Result<String, String> {
    let out = Command::new("curl")
        .args(["-s", "-L", "-m", "15", "-A", USER_AGENT, url])
        .output()
        .map_err(|e| format!("curl not available: {}", e))?;
    if !out.status.success() {
        return Err(format!("curl exit {}", out.status));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Today's trending searches (Google Trends daily RSS)
pub fn fetch_google_trends(limit: usize) -> Result<Vec<String>, String> {
    let xml = curl_get("https://trends.google.com/trending/rss?geo=US")?;
    let titles = parse_rss_titles(&xml);
    if titles.is_empty() {
        return Err("no titles in feed".into());
    }
    Ok(titles.into_iter().take(limit).collect())
}

/// Item `<title>`s from an RSS document (skips the channel title)
pub fn parse_rss_titles(xml: &str) -> Vec<String> {
    xml.split("<title>")
        .skip(2)
        .filter_map(|s| s.split("</title>").next())
        .map(|t| {
            t.replace("&amp;", "&")
                .replace("&#39;", "'")
                .replace("&quot;", "\"")
                .trim()
                .to_string()
        })
        .filter(|t| !t.is_empty())
        .collect()
}

/// Most-viewed English Wikipedia articles yesterday
pub fn fetch_wikipedia_top(limit: usize) -> Result<Vec<String>, String> {
    let days = (time_seed() / 1_000_000_000 / 86_400) as i64 - 1;
    let (y, m, d) = civil_from_days(days);
    let url = format!(
        "https://wikimedia.org/api/rest_v1/metrics/pageviews/top/en.wikipedia/all-access/{:04}/{:02}/{:02}",
        y, m, d
    );
    let doc = curl_get(&url)?;
    let titles: Vec<String> = json::extract_all_strings(&doc, "article")
        .into_iter()
        .filter(|a| !a.contains(':') && a != "Main_Page")
        .map(|a| a.replace('_', " "))
        .take(limit)
        .collect();
    if titles.is_empty() {
        return Err("no articles".into());
    }
    Ok(titles)
}

/// Random page titles from a MediaWiki site (Wikipedia, Wiktionary)
pub fn fetch_wiki_random(host: &str, n: usize) -> Result<Vec<String>, String> {
    let url = format!(
        "https://{}/w/api.php?action=query&list=random&rnnamespace=0&rnlimit={}&format=json",
        host, n
    );
    let titles = json::extract_all_strings(&curl_get(&url)?, "title");
    if titles.is_empty() {
        return Err("no titles".into());
    }
    Ok(titles)
}

/// Days since 1970-01-01 -> (year, month, day). Howard Hinnant's algorithm.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_is_deterministic() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        assert_eq!(a.next_u64(), b.next_u64());
        let f = a.next_f32();
        assert!((0.0..1.0).contains(&f));
        assert!(a.below(10) < 10);
    }

    #[test]
    fn test_civil_from_days() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_386), (2025, 10, 25));
    }

    #[test]
    fn test_parse_rss_titles() {
        let xml = "<rss><channel><title>Daily Search Trends</title><item><title>lamine yamal</title></item><item><title>A &amp; B</title></item></channel></rss>";
        assert_eq!(parse_rss_titles(xml), vec!["lamine yamal", "A & B"]);
    }

    #[test]
    fn test_offline_sparks() {
        let c = Corpus::from_texts(&[(
            "x",
            "norse",
            "The berserker wandered through the moonlit fjords and the shieldwall broke.",
        )]);
        let s = gather(&c, &mut Rng::new(1), false);
        assert_eq!(s.numbers.len(), 8);
        assert!(s.trending.is_empty());
        assert!(!s.prompt_line(&mut Rng::new(2)).is_empty());
    }
}
