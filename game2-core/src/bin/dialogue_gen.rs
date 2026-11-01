//! Dialogue Generator & Evaluator: generate N characters with backstories and
//! M conversations per character (N * M total conversations), formatted with
//! an AI scoring rubric for automated grading.
//!
//! Usage:
//!   cargo run --bin dialogue_gen [-- -n 10 -m 5 --seed 42 -o evidence/dialogue_sample.md]

use std::env;
use std::fs::File;
use std::io::{self, Write};

use game2_core::game::conversation::{converse, Relation, Rng, Speaker};
use game2_core::game::dialogue_trees::{backstory, village_library};
use game2_core::game::traits::Trait;
use game2_core::session::NAMES;

const ROMAN: [&str; 6] = ["", "II", "III", "IV", "V", "VI"];
const IDEOLOGIES: [&str; 4] = ["laborer", "merchant", "clergy", "lord"];
const STATUSES: [&str; 5] = ["well", "sick", "hungry", "grieving", "drunk"];
const ALL_TRAITS: [Trait; 8] = [
    Trait::Lazy,
    Trait::Ambitious,
    Trait::Kind,
    Trait::Stingy,
    Trait::Pious,
    Trait::Hotheaded,
    Trait::Gregarious,
    Trait::Proud,
];
const RELATIONS: [Relation; 5] = [
    Relation::Stranger,
    Relation::Acquaintance,
    Relation::Friend,
    Relation::Kin,
    Relation::Rival,
];

fn generate_roster(n: usize, rng: &mut Rng) -> Vec<Speaker> {
    let mut roster = Vec::with_capacity(n);
    for i in 0..n {
        // Past the end of the name list, a second Aldric is "Aldric II"
        let name = match i / NAMES.len() {
            0 => NAMES[i].to_string(),
            k => format!("{} {}", NAMES[i % NAMES.len()], ROMAN.get(k).copied().unwrap_or("the Younger")),
        };
        let ideology = IDEOLOGIES[rng.below(IDEOLOGIES.len())];

        // 1 to 3 distinct traits
        let num_traits = 1 + rng.below(3);
        let mut traits = Vec::new();
        let mut trait_indices: Vec<usize> = (0..ALL_TRAITS.len()).collect();
        for _ in 0..num_traits {
            let pick = rng.below(trait_indices.len());
            let idx = trait_indices.remove(pick);
            traits.push(ALL_TRAITS[idx]);
        }

        // Status
        let status_pick = STATUSES[rng.below(STATUSES.len())];
        let mut status = Vec::new();
        if status_pick != "well" {
            status.push(status_pick);
        }

        roster.push(Speaker {
            name,
            traits,
            ideology,
            status,
        });
    }
    roster
}

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
    let args: Vec<String> = env::args().collect();
    let mut num_characters = 10;
    let mut num_conversations = 5;
    let mut seed = 42u64;
    let mut out_path = None;

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
            _ => {
                i += 1;
            }
        }
    }

    if num_characters < 2 {
        eprintln!("dialogue_gen: need at least 2 characters to hold a conversation (got -n {})", num_characters);
        std::process::exit(2);
    }

    let mut rng = Rng::new(seed);
    let library = village_library();

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

            let transcript = converse(char_a, char_b, relation, &library, &mut rng);

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
