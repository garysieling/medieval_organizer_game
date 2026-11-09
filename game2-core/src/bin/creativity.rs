//! Creativity Generator CLI.
//!
//! cargo run --release --bin creativity -- "I need a character name"
//! Options: --count N  --cut 0.2  --model gemma3:4b  --seed N  --offline  --no-llm
//!          --corpus data/creativity-corpus  --only norse,hindu  --context "..."  --out ideas.md  --log

use game2_core::agents::creativity_generator::{
    corpus::Corpus, CreativityGenerator, IdeaModel, DEFAULT_CORPUS_DIR,
};
use game2_core::infra::ollama::OllamaClient;
use std::time::Instant;

fn main() {
    game2_core::infra::crash::install();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut request = String::new();
    let mut count = 100usize;
    let mut cut = 0.2f32;
    let mut model_name = "gemma3:4b".to_string();
    let mut seed: Option<u64> = None;
    let mut online = true;
    let mut use_llm = true;
    let mut corpus_dir = DEFAULT_CORPUS_DIR.to_string();
    let mut only: Vec<String> = Vec::new();
    let mut context = String::new();
    let mut out: Option<String> = None;
    let mut log = false;

    let mut i = 0;
    while i < args.len() {
        let next = |i: &mut usize| -> String {
            *i += 1;
            args.get(*i).cloned().unwrap_or_default()
        };
        match args[i].as_str() {
            "--count" => count = next(&mut i).parse().unwrap_or(count),
            "--cut" => cut = next(&mut i).parse().unwrap_or(cut),
            "--model" => model_name = next(&mut i),
            "--seed" => seed = next(&mut i).parse().ok(),
            "--offline" => online = false,
            "--no-llm" => use_llm = false,
            "--corpus" => corpus_dir = next(&mut i),
            "--only" => {
                only = next(&mut i)
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            }
            "--context" => context = next(&mut i),
            "--out" => out = Some(next(&mut i)),
            "--log" => log = true,
            "-h" | "--help" => {
                println!("usage: creativity \"I need a plot point\" [--count 100] [--cut 0.2] [--model gemma3:4b] [--seed N] [--offline] [--no-llm] [--corpus DIR] [--only norse,hindu] [--context TEXT] [--out FILE] [--log]");
                return;
            }
            other => {
                if !request.is_empty() {
                    request.push(' ');
                }
                request.push_str(other);
            }
        }
        i += 1;
    }
    if request.is_empty() {
        request = "I need a character name".into();
    }

    if !std::path::Path::new(&corpus_dir).is_dir()
        && std::path::Path::new(&format!("../{}", corpus_dir)).is_dir()
    {
        corpus_dir = format!("../{}", corpus_dir);
    }
    let t = Instant::now();
    let corpus = match Corpus::load_dir(&corpus_dir, &only) {
        Ok(c) if !c.is_empty() => c,
        Ok(_) | Err(_) => {
            eprintln!(
                "No corpus at {}. Fetch it first: python scripts/fetch_creativity_corpus.py",
                corpus_dir
            );
            std::process::exit(1);
        }
    };
    eprintln!(
        "Loaded {} passages from {} traditions in {:.1}s",
        corpus.len(),
        corpus.traditions().len(),
        t.elapsed().as_secs_f32()
    );

    let model: Option<Box<dyn IdeaModel>> = if use_llm {
        let client = OllamaClient::new(&model_name);
        if client.is_available() {
            Some(Box::new(client))
        } else {
            eprintln!("Ollama not available; running corpus-only");
            None
        }
    } else {
        None
    };

    let mut agent = CreativityGenerator::new(corpus, model);
    agent.config.count = count;
    agent.config.head_cut = cut;
    agent.config.seed = seed;
    agent.config.online = online;

    let t = Instant::now();
    let report = agent.generate(&request, &context);
    eprintln!(
        "Generated in {:.1}s ({} LLM calls)",
        t.elapsed().as_secs_f32(),
        report.llm_calls
    );
    let md = report.to_markdown();
    match out {
        Some(path) => match std::fs::write(&path, &md) {
            Ok(_) => eprintln!("Wrote {}", path),
            Err(e) => eprintln!("Cannot write {}: {}", path, e),
        },
        None => println!("{}", md),
    }
    if log {
        if let Err(e) = agent.log_to_kb(&report) {
            eprintln!("KB log failed: {}", e);
        }
    }
}
