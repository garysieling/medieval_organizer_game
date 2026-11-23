//! Dialogue Generator & Evaluator: generate N characters with backstories and
//! M conversations per character (N * M total conversations), formatted with
//! an AI scoring rubric for automated grading.
//!
//! Usage:
//!   cargo run --bin dialogue_gen [-- -n 10 -m 5 --seed 42 -o evidence/dialogue_sample.md]
//!
//! `--json` writes the conversations as one JSON array instead (no rubric),
//! each turn carrying its act and tree - the conversation review tool
//! (tools/convo-review) reads this.

use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};

use game2_core::game::conversation::{act_fit, converse, Rng, Speaker, Turn};
use game2_core::game::dialogue_trees::{backstory, village_library};
use game2_core::game::talk_gen::{generate_roster, gen_conversations, kind_for,
    RELATIONS};

fn write_rubric<W: Write>(w: &mut W) -> io::Result<()> {
    writeln!(w, "# Conversation System Benchmark & AI Scoring Rubric\n")?;
    writeln!(w, "This document contains a generated benchmark of procedurally synthesized characters\n\
and simulated conversations across various social relations and circumstances.\n\
An AI evaluator or reviewer should grade the conversations against the rubric below.\n")?;

    writeln!(w, "## Scoring Rubric (Total 100 Points)\n")?;
    writeln!(w, "| Category | Max Points | Evaluation Criteria |")?;
    writeln!(w, "| :--- | :---: | :--- |")?;
    writeln!(w, "| **1. Topical Coherence & Discourse Flow** | 25 | Does each reply naturally respond to the preceding discourse act (greet->greet, remark->agree/pushback, question->answer)? Do turns stay on topic without abrupt non-sequiturs? |")?;
    writeln!(w, "| **2. Character & Backstory Alignment** | 25 | Does the character's dialogue match their backstory, traits, and status? E.g., does a sick character cough or mention the fever? Does a cleric invoke blessings? Does a hothead push back? |")?;
    writeln!(w, "| **3. Specificity Cascade & Fallback** | 20 | Does the most specific circumstantial tree fire first (e.g., sickness, bereavement, rivalry), and does the conversation smoothly degrade into generic smalltalk rather than terminating abruptly? |")?;
    writeln!(w, "| **4. Exclusivity & Repetition Avoidance** | 15 | Are lines strictly non-repetitive within the same conversation? Does material exhaustion naturally conclude the dialogue without robotic looping? |")?;
    writeln!(w, "| **5. Relationship & Ending Dynamics** | 15 | Do relationships (rivals vs friends vs strangers) color the tone? Do conversations reach plausible natural closures (polite farewell, sudden walk-off, or mutual exhaustion)? |\n")?;

    writeln!(w, "### Grading Scale")?;
    writeln!(w, "- **90-100 (Grade A):** Superb verisimilitude, sharp character voice, seamless fallback flow.")?;
    writeln!(w, "- **80-89 (Grade B):** Strong coherence, appropriate personality cues, minor phrasing quirks.")?;
    writeln!(w, "- **70-79 (Grade C):** Functional dialogue with occasional tonal mismatch.")?;
    writeln!(w, "- **< 70 (Grade D/F):** Repetition, broken topic flow, or ignored character facts.\n")?;
    writeln!(w, "---\n")?;
    Ok(())
}

fn main() -> io::Result<()> {
    game2_core::infra::crash::install();
    let args: Vec<String> = env::args().collect();
    let mut num_characters = 10;
    let mut num_conversations = 5;
    let mut seed = 42u64;
    let mut out_path = None;
    let mut json = false;
    let mut coherence = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--characters" if i + 1 < args.len() => {
                num_characters = args[i + 1].parse().unwrap_or(10);
                i += 2;
            }
            "-m" | "--conversations" if i + 1 < args.len() => {
                num_conversations = args[i + 1].parse().unwrap_or(5);
                i += 2;
            }
            "-s" | "--seed" if i + 1 < args.len() => {
                seed = args[i + 1].parse().unwrap_or(42);
                i += 2;
            }
            "-o" | "--output" if i + 1 < args.len() => {
                out_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            "--coherence" => {
                coherence = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    if num_characters < 2 {
        eprintln!("Error: --characters must be at least 2 to generate distinct-partner conversations.");
        std::process::exit(1);
    }

    let mut rng = Rng::new(seed);
    let library = village_library();

    if coherence {
        // score the reply adjacencies that never follow the call (#1367):
        // the same seeded roster and pairings as --json, so the rate a
        // fresh batch would show the review tool
        let roster = generate_roster(num_characters, &mut rng);
        let mut adjacencies = 0usize;
        let mut incoherent = 0usize;
        for (char_idx, char_a) in roster.iter().enumerate() {
            for _ in 0..num_conversations {
                let char_b = &roster[(char_idx + 1 + rng.below(num_characters - 1)) % num_characters];
                let relation = RELATIONS[rng.below(RELATIONS.len())];
                let t = converse(char_a, char_b, relation, kind_for(&mut rng, relation), &library, None, &mut rng);
                // the responder's turn answers the caller's immediately
                // before it; a fresh call after a reply is a new beat,
                // not an adjacency
                for w in t.turns.windows(2) {
                    if w[0].speaker == 0 && w[1].speaker == 1 {
                        adjacencies += 1;
                        if act_fit(w[0].act, w[1].act) == 0 {
                            incoherent += 1;
                        }
                    }
                }
            }
        }
        println!("coherence: {}/{} reply adjacencies follow ({:.1}% non-sequitur)",
            adjacencies - incoherent, adjacencies,
            100.0 * incoherent as f64 / adjacencies.max(1) as f64);
        return Ok(());
    }

    if json {
        let convs = json_conversations(num_characters, num_conversations, seed, &mut rng, &library);
        let text = serde_json::to_string(&convs).expect("conversations serialize");
        match &out_path {
            Some(p) => std::fs::write(p, text)?,
            None => println!("{}", text),
        }
        return Ok(());
    }

    let mut out: Box<dyn Write> = if let Some(path) = &out_path {
        Box::new(File::create(path)?)
    } else {
        Box::new(io::stdout())
    };

    write_rubric(&mut out)?;

    writeln!(out, "## Generated Character Roster (N = {})\n", num_characters)?;
    let roster = generate_roster(num_characters, &mut rng);
    for (idx, speaker) in roster.iter().enumerate() {
        writeln!(out, "### Character #{}: {}", idx + 1, speaker.name)?;
        writeln!(out, "- **Ideology:** {}", speaker.ideology)?;
        writeln!(out, "- **Traits:** {:?}", speaker.traits)?;
        writeln!(out, "- **Status:** {:?}", speaker.status)?;
        if !speaker.facts.open.is_empty() {
            writeln!(out, "- **Facts:** {}", speaker.facts.open.join(", "))?;
        }
        writeln!(out, "- **Backstory:** {}\n", backstory(speaker))?;
    }

    let total_conversations = num_characters * num_conversations;
    writeln!(out, "---\n")?;
    writeln!(out, "## Generated Conversations (N x M = {} x {} = {} Total)\n",
        num_characters, num_conversations, total_conversations)?;

    let mut conv_count = 0;
    for (char_idx, char_a) in roster.iter().enumerate() {
        for _conv_idx in 0..num_conversations {
            conv_count += 1;

            // Pick a partner distinct from char_a
            let other_offset = 1 + rng.below(num_characters - 1);
            let partner_idx = (char_idx + other_offset) % num_characters;
            let char_b = &roster[partner_idx];

            let relation = RELATIONS[rng.below(RELATIONS.len())];

            let transcript = converse(char_a, char_b, relation, kind_for(&mut rng, relation), &library, None, &mut rng);

            writeln!(out, "### Conversation #{:02}: {} and {}", conv_count, char_a.name, char_b.name)?;
            writeln!(out, "- **Speakers:** {} ({}) & {} ({})",
                char_a.name, char_a.ideology, char_b.name, char_b.ideology)?;
            writeln!(out, "- **Backstory A:** {}", backstory(char_a))?;
            writeln!(out, "- **Backstory B:** {}", backstory(char_b))?;
            writeln!(out, "- **Relation:** {:?}", relation)?;
            writeln!(out, "- **Trees Cascade:** {:?}", transcript.trees_walked)?;
            writeln!(out, "- **Ending:** {:?}", transcript.ending.unwrap_or(game2_core::game::conversation::Ending::Exhausted))?;
            writeln!(out, "- **Turns Count:** {}", transcript.turns.len())?;
            writeln!(out, "\n```text")?;
            for turn in &transcript.turns {
                writeln!(out, "{:10} [{:?} in {:<18}]: \"{}\"",
                    transcript.speakers[turn.speaker], turn.act, turn.tree, turn.text)?;
            }
            writeln!(out, "```\n")?;
        }
    }

    writeln!(out, "## Benchmark Summary\n")?;
    writeln!(out, "- Total Characters Generated: {}", num_characters)?;
    writeln!(out, "- Total Conversations Generated: {}", total_conversations)?;
    writeln!(out, "- Generator Seed: {}", seed)?;
    writeln!(out, "\nBenchmark generation complete.")?;

    if let Some(path) = &out_path {
        println!("Generated {} characters and {} conversations -> {}", num_characters, total_conversations, path);
    }

    Ok(())
}

/// One speaker, as the review tool shows them
fn speaker_json(s: &Speaker) -> serde_json::Value {
    serde_json::json!({
        "name": s.name,
        "ideology": s.ideology,
        "traits": s.traits.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>(),
        "status": s.status,
        "facts": s.facts.open,
        "backstory": backstory(s),
    })
}

/// A stable fingerprint of a conversation's actual lines, so a reviewer's
/// feedback for one transcript never gets silently inherited by a
/// different transcript that lands at the same roster position after the
/// dialogue trees change.
fn transcript_fingerprint(speakers: &[String; 2], turns: &[Turn]) -> u64 {
    let mut hasher = DefaultHasher::new();
    speakers.hash(&mut hasher);
    for turn in turns {
        turn.speaker.hash(&mut hasher);
        turn.act.hash(&mut hasher);
        turn.tree.hash(&mut hasher);
        turn.text.hash(&mut hasher);
    }
    hasher.finish()
}

/// The same roster and pairings as the Markdown report, as JSON records
fn json_conversations(
    num_characters: usize,
    num_conversations: usize,
    seed: u64,
    _rng: &mut Rng,
    _library: &[game2_core::game::conversation::DialogueTree],
) -> Vec<serde_json::Value> {
    let mut out = vec![];
    let corpus = gen_conversations(num_characters, num_conversations, seed);
    for (idx, gen) in corpus.talks.into_iter().enumerate() {
        let char_idx = idx / num_conversations;
        let conv_idx = idx % num_conversations;
        let (relation, kind, t) = (gen.relation, gen.kind, gen.transcript);
        let (char_a, char_b) = (&corpus.roster[gen.a], &corpus.roster[gen.b]);
        {
            let fp = transcript_fingerprint(&t.speakers, &t.turns);
            out.push(serde_json::json!({
                // the roster and pairings depend on the batch's shape too,
                // plus a content fingerprint so a changed dialogue tree
                // doesn't inherit feedback filed against the old lines
                "id": format!("gen-s{}-n{}m{}-{}-{}-{:x}", seed, num_characters, num_conversations, char_idx, conv_idx, fp),
                "source": "generated",
                "kind": "Chat",
                "relation": format!("{:?}", relation),
                "rel_kind": kind.map(|k| format!("{:?}", k)),
                "speakers": [speaker_json(char_a), speaker_json(char_b)],
                "trees": t.trees_walked,
                "ending": format!("{:?}", t.ending.unwrap_or(game2_core::game::conversation::Ending::Exhausted)),
                "lines": t.turns.iter().map(|turn| serde_json::json!({
                    "speaker": t.speakers[turn.speaker],
                    "act": format!("{:?}", turn.act),
                    "tree": turn.tree,
                    "text": turn.text,
                })).collect::<Vec<_>>(),
            }));
        }
    }
    out
}
