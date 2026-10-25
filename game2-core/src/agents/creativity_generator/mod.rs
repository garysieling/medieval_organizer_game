//! Creativity Generator: a creativity-biased RAG over world myth and ancient texts.
//!
//! Pipeline for a request like "I need a character name":
//! 1. Parse the concept kind and gather randomness (numbers, rare corpus words,
//!    trending topics, random Wikipedia/Wiktionary entries).
//! 2. Retrieve passages with BM25, perturbed by the random sparks and spread across traditions.
//! 3. Mine candidates directly from the passages and have the LLM invent more
//!    (high temperature, a fresh seed per batch, different passages per batch).
//! 4. Have the LLM rank everything from most predictable to most surprising,
//!    combine with consensus/frequency/cliche heuristics, cut the most obvious head,
//!    and return ~100 ideas with provenance.

pub mod corpus;
pub mod extract;
pub mod randomness;
pub mod ranking;

use crate::infra::agent_kb::AgentKB;
use crate::infra::ollama::OllamaClient;
use corpus::{tokenize, Corpus, Passage};
use extract::ConceptKind;
use randomness::{Rng, Sparks};
use ranking::{Candidate, Origin};
use std::collections::HashSet;

pub const DEFAULT_CORPUS_DIR: &str = "data/creativity-corpus";
pub const DEFAULT_GENRE: &str =
    "a medieval fantasy simulation game in the spirit of the sagas, epics and myths that inspired The Lord of the Rings";

/// Anything that can sample text with a temperature and seed (Ollama, or a mock in tests)
pub trait IdeaModel {
    fn sample(&self, prompt: &str, temperature: f32, seed: u64) -> Result<String, String>;
}

impl IdeaModel for OllamaClient {
    fn sample(&self, prompt: &str, temperature: f32, seed: u64) -> Result<String, String> {
        self.generate(prompt, temperature, seed)
    }
}

#[derive(Debug, Clone)]
pub struct CreativityConfig {
    /// Ideas to return
    pub count: usize,
    /// Fraction of the ranked pool (most obvious first) to discard
    pub head_cut: f32,
    /// Share of returned ideas mined verbatim from the corpus
    pub corpus_share: f32,
    /// Ideas requested per LLM call
    pub batch_size: usize,
    /// Passages shown to the LLM per call
    pub passages_per_batch: usize,
    /// Fetch trending topics / random words from the internet
    pub online: bool,
    /// Use the LLM to rank obviousness (otherwise heuristics only)
    pub rank_with_llm: bool,
    pub min_temperature: f32,
    pub max_temperature: f32,
    /// Fixed seed for reproducible runs (None = clock)
    pub seed: Option<u64>,
    pub genre: String,
}

impl Default for CreativityConfig {
    fn default() -> Self {
        Self {
            count: 100,
            head_cut: 0.2,
            corpus_share: 0.3,
            batch_size: 25,
            passages_per_batch: 4,
            online: true,
            rank_with_llm: true,
            min_temperature: 0.9,
            max_temperature: 1.3,
            seed: None,
            genre: DEFAULT_GENRE.into(),
        }
    }
}

/// One returned idea
#[derive(Debug, Clone)]
pub struct Idea {
    pub text: String,
    pub note: String,
    pub source: String,
    pub tradition: String,
    pub origin: Origin,
    pub obviousness: f32,
}

impl From<Candidate> for Idea {
    fn from(c: Candidate) -> Self {
        Idea {
            text: c.text,
            note: c.note,
            source: c.source,
            tradition: c.tradition,
            origin: c.origin,
            obviousness: c.obviousness,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreativityReport {
    pub request: String,
    pub kind: ConceptKind,
    pub seed: u64,
    pub ideas: Vec<Idea>,
    /// The obvious head that was cut (most obvious first)
    pub cut: Vec<Idea>,
    pub sparks: Sparks,
    pub pool_size: usize,
    pub passages_used: usize,
    pub llm_calls: usize,
    pub errors: Vec<String>,
}

impl CreativityReport {
    pub fn to_markdown(&self) -> String {
        let mut s = format!(
            "# Creativity Generator: {}\n\n- kind: {}\n- seed: {}\n- returned: {} (pool {}, cut {} most obvious)\n- passages used: {}, LLM calls: {}\n",
            self.request, self.kind.label(), self.seed, self.ideas.len(), self.pool_size, self.cut.len(), self.passages_used, self.llm_calls
        );
        if !self.sparks.trending.is_empty() {
            s.push_str(&format!(
                "- trending sparks: {}\n",
                self.sparks.trending.join(", ")
            ));
        }
        if !self.sparks.random_titles.is_empty() || !self.sparks.random_words.is_empty() {
            s.push_str(&format!(
                "- random sparks: {}\n",
                [
                    self.sparks.random_titles.clone(),
                    self.sparks.random_words.clone()
                ]
                .concat()
                .join(", ")
            ));
        }
        if !self.sparks.corpus_words.is_empty() {
            s.push_str(&format!(
                "- corpus sparks: {}\n",
                self.sparks.corpus_words.join(", ")
            ));
        }
        for e in &self.errors {
            s.push_str(&format!("- warning: {}\n", e));
        }
        s.push_str("\n## Ideas\n\n");
        for (i, idea) in self.ideas.iter().enumerate() {
            s.push_str(&format!("{}. {}", i + 1, idea.text));
            if !idea.note.is_empty() {
                s.push_str(&format!(" - {}", idea.note));
            }
            let origin = match idea.origin {
                Origin::Corpus => format!("from {} ({})", idea.source, idea.tradition),
                Origin::Llm => format!("llm, inspired by {}", idea.source),
            };
            s.push_str(&format!("  _[{}]_\n", origin));
        }
        if !self.cut.is_empty() {
            s.push_str("\n## Cut as too obvious\n\n");
            let cut: Vec<&str> = self.cut.iter().map(|c| c.text.as_str()).collect();
            s.push_str(&cut.join(" | "));
            s.push('\n');
        }
        s
    }
}

/// The Creativity Generator agent
pub struct CreativityGenerator {
    pub name: String,
    pub corpus: Corpus,
    pub model: Option<Box<dyn IdeaModel>>,
    pub config: CreativityConfig,
}

impl CreativityGenerator {
    pub fn new(corpus: Corpus, model: Option<Box<dyn IdeaModel>>) -> Self {
        Self {
            name: "Creativity Generator".into(),
            corpus,
            model,
            config: CreativityConfig::default(),
        }
    }

    /// Load the corpus from disk and use an Ollama model
    pub fn with_ollama(corpus_dir: &str, model: &str) -> Result<Self, String> {
        let corpus = Corpus::load_dir(corpus_dir, &[])?;
        Ok(Self::new(corpus, Some(Box::new(OllamaClient::new(model)))))
    }

    /// Generate `config.count` ideas for a free-text request
    pub fn generate(&self, request: &str, context: &str) -> CreativityReport {
        let kind = ConceptKind::parse(request);
        let seed = self.config.seed.unwrap_or_else(randomness::time_seed);
        let mut rng = Rng::new(seed);
        let sparks = randomness::gather(&self.corpus, &mut rng, self.config.online);
        let mut errors = sparks.errors.clone();
        let mut pool: Vec<Candidate> = Vec::new();
        let mut used_passages: HashSet<usize> = HashSet::new();
        let mut llm_calls = 0;

        let base_terms = self.query_terms(&kind, request, context, &sparks);

        // 1. Mine candidates straight from the texts
        let mut mine_ids: Vec<usize> = self
            .corpus
            .search(&base_terms, 250)
            .into_iter()
            .map(|(i, _)| i)
            .collect();
        for _ in 0..150.min(self.corpus.len()) {
            mine_ids.push(rng.below(self.corpus.len()));
        }
        let mine_passages: Vec<&Passage> =
            mine_ids.iter().map(|&i| &self.corpus.passages[i]).collect();
        let mut mined = extract::mine(&kind, &mine_passages);
        if !kind.is_short_form() {
            rng.shuffle(&mut mined);
        }
        let corpus_share = if self.model.is_some() {
            self.config.corpus_share
        } else {
            1.0
        };
        let max_freq = mined.iter().map(|m| m.frequency).max().unwrap_or(1).max(1) as f32;
        let mined_cap = ((self.config.count as f32 * corpus_share) / (1.0 - self.config.head_cut)
            * 2.0)
            .ceil() as usize
            + 10;
        for m in mined.into_iter().take(mined_cap) {
            let freq_obv = (m.frequency as f32).ln_1p() / max_freq.ln_1p().max(1.0);
            ranking::merge(
                &mut pool,
                Candidate {
                    text: m.text,
                    note: m.note,
                    source: m.source,
                    tradition: m.tradition,
                    origin: Origin::Corpus,
                    votes: 1,
                    llm_surprise: None,
                    obviousness: freq_obv,
                },
                &kind,
            );
        }

        // 2. LLM invention, a different slice of the corpus + sparks per batch
        if let Some(model) = &self.model {
            let llm_target = (self.config.count as f32 * (1.0 - self.config.corpus_share)
                / (1.0 - self.config.head_cut)
                * 1.3)
                .ceil() as usize;
            let batches = llm_target.div_ceil(self.config.batch_size.max(1)).max(1);
            for b in 0..batches {
                let passages = self.pick_passages(&base_terms, &mut rng, &mut used_passages);
                let prompt = self.generation_prompt(
                    &kind,
                    request,
                    context,
                    &passages,
                    &sparks.prompt_line(&mut rng),
                );
                let temp = self.config.min_temperature
                    + rng.next_f32() * (self.config.max_temperature - self.config.min_temperature);
                llm_calls += 1;
                match model.sample(&prompt, temp, rng.next_u64() & 0x7fff_ffff) {
                    Ok(out) => {
                        let inspired_by = passages
                            .iter()
                            .map(|p| p.source.clone())
                            .collect::<Vec<_>>()
                            .join("; ");
                        let tradition = passages
                            .iter()
                            .map(|p| p.tradition.clone())
                            .collect::<Vec<_>>()
                            .join(", ");
                        for (text, note) in ranking::parse_list(&out, &kind) {
                            ranking::merge(
                                &mut pool,
                                Candidate {
                                    text,
                                    note,
                                    source: inspired_by.clone(),
                                    tradition: tradition.clone(),
                                    origin: Origin::Llm,
                                    votes: 1,
                                    llm_surprise: None,
                                    obviousness: 0.3,
                                },
                                &kind,
                            );
                        }
                    }
                    Err(e) => errors.push(format!("generation batch {}: {}", b + 1, e)),
                }
            }
        }

        // 3. LLM ranks obviousness in shuffled batches
        if self.config.rank_with_llm {
            if let Some(model) = &self.model {
                rng.shuffle(&mut pool);
                for chunk in pool.chunks_mut(60) {
                    let prompt = {
                        let refs: Vec<&Candidate> = chunk.iter().collect();
                        ranking::rank_prompt(&kind, &self.config.genre, &refs)
                    };
                    llm_calls += 1;
                    match model.sample(&prompt, 0.2, rng.next_u64() & 0x7fff_ffff) {
                        Ok(out) => {
                            let order = ranking::parse_rank_order(&out, chunk.len());
                            let mut refs: Vec<&mut Candidate> = chunk.iter_mut().collect();
                            ranking::apply_rank(&mut refs, &order);
                        }
                        Err(e) => errors.push(format!("ranking: {}", e)),
                    }
                }
            }
        }

        // mined candidates carry their corpus-frequency signal in `obviousness`
        let freq: Vec<(String, f32)> = pool
            .iter()
            .map(|c| {
                (
                    ranking::normalize_key(&c.text, &kind),
                    if c.origin == Origin::Corpus {
                        c.obviousness
                    } else {
                        0.3
                    },
                )
            })
            .collect();
        let mut scored = pool;
        ranking::score(&mut scored, &|c: &Candidate| {
            let key = ranking::normalize_key(&c.text, &kind);
            freq.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, f)| *f)
                .unwrap_or(0.3)
        });
        let pool_size = scored.len();
        let (kept, mut cut) = ranking::select(
            scored,
            self.config.count,
            self.config.head_cut,
            corpus_share,
            &mut rng,
        );
        cut.sort_by(|a, b| {
            b.obviousness
                .partial_cmp(&a.obviousness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        CreativityReport {
            request: request.into(),
            kind,
            seed,
            ideas: kept.into_iter().map(Idea::from).collect(),
            cut: cut.into_iter().map(Idea::from).collect(),
            sparks,
            pool_size,
            passages_used: used_passages.len() + mine_ids.len(),
            llm_calls,
            errors,
        }
    }

    /// Append a summary of a run to the agent KB (.agents/creativity-generator/kb)
    pub fn log_to_kb(&self, report: &CreativityReport) -> Result<(), String> {
        let kb = AgentKB::new("creativity-generator");
        kb.init()?;
        let top: Vec<&str> = report
            .ideas
            .iter()
            .take(10)
            .map(|i| i.text.as_str())
            .collect();
        let cut: Vec<&str> = report
            .cut
            .iter()
            .take(10)
            .map(|i| i.text.as_str())
            .collect();
        kb.add_kb_entry(
            "generation-history.md",
            &format!(
                "### {}\n**Kind:** {} | **Seed:** {} | **Returned:** {} of {}\n**Sample:** {}\n**Cut as obvious:** {}\n",
                report.request,
                report.kind.label(),
                report.seed,
                report.ideas.len(),
                report.pool_size,
                top.join(", "),
                cut.join(", ")
            ),
        )
    }

    fn query_terms(
        &self,
        kind: &ConceptKind,
        request: &str,
        context: &str,
        sparks: &Sparks,
    ) -> Vec<String> {
        const STOP: &[&str] = &[
            "need", "want", "some", "give", "with", "that", "this", "from", "have", "make", "idea",
            "ideas", "name", "names",
        ];
        let mut terms = kind.seed_terms();
        for t in tokenize(request).into_iter().chain(tokenize(context)) {
            if t.len() > 3 && !STOP.contains(&t.as_str()) {
                terms.push(t);
            }
        }
        terms.extend(sparks.corpus_words.iter().take(4).cloned());
        for topic in sparks
            .trending
            .iter()
            .chain(sparks.random_words.iter())
            .take(6)
        {
            terms.extend(tokenize(topic).into_iter().filter(|t| t.len() > 3));
        }
        terms
    }

    /// Passages for one LLM batch: BM25 hits on a randomised query plus a wildcard,
    /// preferring traditions not used yet in this run
    fn pick_passages(
        &self,
        base_terms: &[String],
        rng: &mut Rng,
        used: &mut HashSet<usize>,
    ) -> Vec<&Passage> {
        let mut out: Vec<&Passage> = Vec::new();
        if self.corpus.is_empty() {
            return out;
        }
        let mut terms = base_terms.to_vec();
        rng.shuffle(&mut terms);
        terms.truncate((terms.len() / 2).max(3));
        terms.extend(randomness::random_corpus_words(&self.corpus, rng, 2));
        let mut hits: Vec<usize> = self
            .corpus
            .search(&terms, 60)
            .into_iter()
            .map(|(i, _)| i)
            .filter(|i| !used.contains(i))
            .collect();
        rng.shuffle(&mut hits);
        let used_traditions: HashSet<String> = used
            .iter()
            .map(|&i| self.corpus.passages[i].tradition.clone())
            .collect();
        hits.sort_by_key(|&i| used_traditions.contains(&self.corpus.passages[i].tradition));
        let mut seen_traditions = HashSet::new();
        for i in hits {
            if out.len() + 1 >= self.config.passages_per_batch {
                break;
            }
            if seen_traditions.insert(self.corpus.passages[i].tradition.clone()) {
                used.insert(i);
                out.push(&self.corpus.passages[i]);
            }
        }
        let wildcard = rng.below(self.corpus.len());
        used.insert(wildcard);
        out.push(&self.corpus.passages[wildcard]);
        out
    }

    fn generation_prompt(
        &self,
        kind: &ConceptKind,
        request: &str,
        context: &str,
        passages: &[&Passage],
        sparks: &str,
    ) -> String {
        let n = self.config.batch_size;
        let mut s = format!(
            "You are the Creativity Generator for {}.\nRequest: \"{}\"\nInvent {} distinct {} ideas.\n",
            self.config.genre,
            request.trim(),
            n,
            kind.label()
        );
        if !context.trim().is_empty() {
            s.push_str(&format!("Context: {}\n", context.trim()));
        }
        s.push_str("\nSource fragments from sagas, scripture, epics and myths. Borrow their sounds, customs, images and story shapes; transform rather than copy, and blend traditions:\n");
        for (i, p) in passages.iter().enumerate() {
            let excerpt: String = p.text.chars().take(700).collect();
            s.push_str(&format!(
                "[{}] ({} - {}) {}\n",
                i + 1,
                p.tradition,
                p.source,
                excerpt
            ));
        }
        if !sparks.is_empty() {
            s.push_str(&format!("\nRandom sparks (let some of these bend a few ideas in odd directions, ignore the rest): {}\n", sparks));
        }
        s.push_str(&format!(
            "\nAvoid the obvious: no Tolkien or D&D names, nothing like Elara, Kael, Lyra, Thorne, 'blacksmith', 'chosen one' or 'dark lord'.\n{}\nOutput exactly {} lines numbered 1-{}. No introduction, no commentary.\n",
            kind.format_hint(),
            n,
            n
        ));
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct MockModel {
        calls: Cell<usize>,
    }

    impl IdeaModel for MockModel {
        fn sample(&self, prompt: &str, _t: f32, _s: u64) -> Result<String, String> {
            self.calls.set(self.calls.get() + 1);
            if prompt.starts_with("You are judging") {
                // rank in listed order: item 1 most predictable
                let n = prompt
                    .lines()
                    .filter(|l| {
                        l.split('.')
                            .next()
                            .map(|d| d.parse::<usize>().is_ok())
                            .unwrap_or(false)
                    })
                    .count();
                return Ok((1..=n)
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", "));
            }
            let call = self.calls.get();
            Ok((1..=25)
                .map(|i| format!("{}. Name{}x{} - a test person", i, call, i))
                .chain(std::iter::once("26. Elara - cliche".into()))
                .collect::<Vec<_>>()
                .join("\n"))
        }
    }

    fn corpus() -> Corpus {
        Corpus::from_texts(&[
            ("Grettir", "norse", "Then came Grettir son of Asmund, and Thorbjorn the priest of Frey rode from Bjarg.\n\nGrettir slew the bear and the berserks at Haramarsey."),
            ("Ramayana", "hindu", "Then spake Vibhishana unto Rama, and Hanuman leapt from Mahendra.\n\nThe rakshasa Kumbhakarna slept in Lanka."),
            ("Maui", "polynesian", "Then Maui snared the sun with ropes of his sister Hina, and fished up Aotearoa."),
        ])
    }

    #[test]
    fn test_generate_with_mock_model() {
        let model = MockModel {
            calls: Cell::new(0),
        };
        let mut agent = CreativityGenerator::new(corpus(), Some(Box::new(model)));
        agent.config.online = false;
        agent.config.seed = Some(11);
        agent.config.count = 40;
        let report = agent.generate("I need a character name", "");
        assert_eq!(report.kind, ConceptKind::CharacterName);
        assert_eq!(report.ideas.len(), 40);
        assert!(report.cut.iter().any(|i| i.text == "Elara"));
        assert!(!report.ideas.iter().any(|i| i.text == "Elara"));
        assert!(report.ideas.iter().any(|i| i.origin == Origin::Corpus));
        assert!(report.llm_calls >= 2);
        assert!(report.to_markdown().contains("## Ideas"));
    }

    #[test]
    fn test_generate_corpus_only_is_reproducible() {
        let mut agent = CreativityGenerator::new(corpus(), None);
        agent.config.online = false;
        agent.config.seed = Some(5);
        let a: Vec<String> = agent
            .generate("i need a vocation", "")
            .ideas
            .into_iter()
            .map(|i| i.text)
            .collect();
        let b: Vec<String> = agent
            .generate("i need a vocation", "")
            .ideas
            .into_iter()
            .map(|i| i.text)
            .collect();
        assert_eq!(a, b);
        assert!(a.contains(&"priest".to_string()));
    }
}
