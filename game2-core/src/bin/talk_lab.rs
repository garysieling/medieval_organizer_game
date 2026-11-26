//! Talk lab: play the game's conversations from the command line.
//!
//! The same conversations the UI shows, driven through the session's own
//! machinery (`Game::command` + `ui_model().convo`) so what you see is
//! what a player sees - then a feedback prompt at the end files your
//! verdict as a GitHub issue, so sample runs build the redesign corpus.
//!
//! Modes:
//!   talk      a 1:1 Talk - real villager (a seeded Game) or synthetic
//!             speakers (flags), 1/2/x picks like the UI
//!   meeting   watch a house meeting (synthetic roster)
//!   sandbox   a free command shell over a real session - play your way
//!             to any panel: founding calls, agenda slates, dilemma and
//!             lord's-eyes boxes all surface through the convo box
//!   batch     N auto-picked synthetic conversations, transcript dump
//!   smoke     one scripted run, exit status = it worked (the test hook)
//!
//! Every run ends with a `REPRO:` line - re-running those args replays
//! the run exactly (the sim's seeded; a Talk replays from seed + picks).
//!
//! Examples:
//!   cargo run -p game2-core --bin talk_lab -- talk --villager 3
//!   cargo run -p game2-core --bin talk_lab -- talk --seed 42 --villager eliza
//!   cargo run -p game2-core --bin talk_lab -- talk --name Eliza --status sick \
//!         --trait kind --rel acquaintance --seed 7
//!   cargo run -p game2-core --bin talk_lab -- sandbox --stage strongorg
//!   cargo run -p game2-core --bin talk_lab -- batch -n 10 --out evidence/talks.md
//!   cargo run -p game2-core --bin talk_lab -- replay --seed 7 --picks 0,2,1 \
//!         --name Eliza --status sick

use std::collections::VecDeque;
use std::io::{self, BufRead, Write};

use game2_core::game::conversation::{
    converse_with, converse_with_at, run_meeting, Act, Choice, Ending, MeetingLine, Relation, Rng, Speaker,
};
use game2_core::game::dialogue_trees::{backstory, meeting_pool, village_library, MEET_PROMPT, MEET_RITES};
use game2_core::game::traits::Trait;
use game2_core::session::stage::{run_days, stage, Stage};
use game2_core::session::{Command, Game, NAMES};
use game2_core::session::MAP_SEED;

// ---------------------------------------------------------------- args

/// Everything a run needs, parsed from argv. `picks` pre-loads the pick
/// queue (the REPRO line's tail); `auto` takes option 1 when the queue's
/// empty so a machine never blocks on stdin.
struct Args {
    mode: String,
    seed: u64,
    villager: Option<String>,
    name: Option<String>,
    ideology: Option<String>,
    rel: Option<Relation>,
    traits: Vec<Trait>,
    statuses: Vec<String>,
    stage: Option<Stage>,
    /// --plan: a ready MeetingPlan sits on the org at start
    plan: bool,
    days: u64,
    n: usize,
    /// The arc runner's talk count (E0-4, #1405): 20, the Appendix A
    /// arc scale
    talks: usize,
    /// The scripted player policy an arc runs (E0-4)
    policy: Option<String>,
    picks: VecDeque<usize>,
    auto: bool,
    feedback: bool,
    out: Option<String>,
    folk: usize,
    warm: bool,
    seeded_cast: bool,
}

fn parse_args() -> Args {
    let mut a = Args {
        mode: "help".into(), seed: MAP_SEED, villager: None, name: None,
        ideology: None, rel: None, traits: vec![], statuses: vec![],
        stage: None, days: 0, n: 10, talks: 20, policy: None, plan: false,
        picks: VecDeque::new(), auto: false,
        feedback: true, out: None, folk: 4, warm: false, seeded_cast: false,
    };
    let argv: Vec<String> = std::env::args().collect();
    let mut i = 1;
    if argv.len() > 1 && !argv[1].starts_with('-') { a.mode = argv[1].clone(); i = 2; }
    while i < argv.len() {
        let take = |i: &mut usize| -> String { *i += 1; argv.get(*i).cloned().unwrap_or_default() };
        match argv[i].as_str() {
            "--seed" | "-s" => a.seed = take(&mut i).parse().unwrap_or(MAP_SEED),
            // a seeded cast carries personas (E3-1): their facts print
            "--cast" => a.seeded_cast = take(&mut i) == "seeded",
            "--villager" => a.villager = Some(take(&mut i)),
            "--name" => a.name = Some(take(&mut i)),
            "--ideology" => a.ideology = Some(take(&mut i)),
            "--status" => a.statuses.push(take(&mut i)),
            "--trait" => {
                let t = take(&mut i);
                match trait_of(&t) {
                    Some(t) => a.traits.push(t),
                    None => { eprintln!("unknown trait '{t}' - try lazy/ambitious/kind/stingy/pious/hotheaded/gregarious/proud"); std::process::exit(2); }
                }
            }
            "--rel" => a.rel = Some(rel_of(&take(&mut i))),
            "--stage" => a.stage = Some(stage_of(&take(&mut i))),
            // the asks' yield, staged (#1621): the plan a courting run
            // builds to, so the meeting arc's reachable from the shell
            "--plan" => a.plan = true,
            "--days" => a.days = take(&mut i).parse().unwrap_or(0),
            "-n" => a.n = take(&mut i).parse().unwrap_or(10),
            "--talks" => a.talks = take(&mut i).parse().unwrap_or(20),
            "--policy" => a.policy = Some(take(&mut i)),
            "--picks" => {
                for p in take(&mut i).split(',') {
                    if let Ok(n) = p.trim().parse() { a.picks.push_back(n); }
                }
            }
            "--auto" => a.auto = true,
            "--warm" => a.warm = true,
            "--help" | "-h" => { /* falls through to usage */ }
            "--feedback" => a.feedback = take(&mut i) != "off",
            "--no-feedback" => a.feedback = false,
            "--out" | "-o" => a.out = Some(take(&mut i)),
            "--folk" => a.folk = take(&mut i).parse().unwrap_or(4),
            _ => { eprintln!("unknown flag {}", argv[i]); std::process::exit(2); }
        }
        i += 1;
    }
    a
}

fn trait_of(s: &str) -> Option<Trait> {
    Some(match s.to_ascii_lowercase().as_str() {
        "lazy" => Trait::Lazy, "ambitious" => Trait::Ambitious,
        "kind" => Trait::Kind, "stingy" => Trait::Stingy,
        "pious" => Trait::Pious, "hotheaded" => Trait::Hotheaded,
        "gregarious" => Trait::Gregarious, "proud" => Trait::Proud,
        _ => return None,
    })
}

fn rel_of(s: &str) -> Relation {
    match s.to_ascii_lowercase().as_str() {
        "stranger" => Relation::Stranger, "friend" => Relation::Friend,
        "kin" => Relation::Kin, "rival" => Relation::Rival,
        _ => Relation::Acquaintance,
    }
}

fn stage_of(s: &str) -> Stage {
    match s.to_ascii_lowercase().as_str() {
        "fever" => Stage::Fever, "famine" => Stage::Famine,
        "hardwinter" | "winter" => Stage::HardWinter,
        "strongorg" | "org" => Stage::StrongOrg,
        "unrest" => Stage::Unrest, "flush" => Stage::Flush,
        "bereft" => Stage::Bereft, _ => Stage::Ordinary,
    }
}

/// The asks' yield, staged (#1621): a ready-to-call MeetingPlan on the
/// org, the way a courting run's asks would have left it - the meeting
/// arc's reach, without an hour of setup talk
fn stage_plan(g: &mut Game) {
    let oe = g.org;
    let coming: Vec<game2_core::engine::ecs::Entity> =
        g.villagers.iter().take(8).copied().collect();
    let plan = game2_core::game::organizing::plan_for(&mut g.world, oe);
    plan.coming = coming;
    plan.venue = Some(game2_core::game::organizing::Venue::Square);
    let pe = g.seats[0].player;
    g.world.add_component(pe, game2_core::game::org_skills::Organizing::default());
    g.world.get_component_mut::<game2_core::game::org_skills::Organizing>(pe)
        .unwrap().turnout = 2.0;
}

// ------------------------------------------------------ output + picks

/// Everything shown is also kept, so a run writes itself into the
/// verdict ticket (or a transcript file) verbatim.
struct Out {
    buf: String,
    quiet: bool,
}
impl Out {
    fn emit(&mut self, s: &str) {
        println!("{s}");
        self.buf.push_str(s);
        self.buf.push('\n');
        if !self.quiet { let _ = io::stdout().flush(); }
    }
}

/// A pick source: the scripted queue first, then auto/stdin. Returns
/// Some(pick) or None to leave/quit the conversation.
fn next_pick(opts: &[String], picks: &mut VecDeque<usize>, auto: bool, stdin: &io::Stdin) -> Option<usize> {
    if let Some(p) = picks.pop_front() {
        println!(">> (scripted pick {}) {}", p + 1, opts.get(p).cloned().unwrap_or_default());
        return Some(p.min(opts.len().saturating_sub(1)));
    }
    if auto {
        println!(">> (auto pick 1) {}", opts.first().cloned().unwrap_or_default());
        return Some(0);
    }
    loop {
        print!("pick> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 { return None; } // EOF leaves
        let line = line.trim();
        if line == "l" || line == "leave" || line == "q" { return None; }
        if let Ok(n) = line.parse::<usize>() {
            if n >= 1 && n <= opts.len() { return Some(n - 1); }
        }
        println!("  (1-{} to pick, 'l' to leave)", opts.len());
    }
}

// ------------------------------------------------------- real-mode talk

/// The seeded-session path: a real villager, the real `PlayerTalk` -
/// profile, gist, asks and all - read off the convo box each frame.
fn run_talk_real(a: &Args, out: &mut Out, taken: &mut Vec<usize>, stdin: &io::Stdin) {
    let mut pq = a.picks.clone();
    let mut g = if a.seeded_cast { Game::new_seeded_cast(a.seed) }
        else { Game::new_seeded(a.seed) };
    if let Some(s) = a.stage { stage(&mut g, s); }
    if a.days > 0 { run_days(&mut g, a.days); }

    // the villager: an index into g.villagers, or a name matched on the
    // model's labels - default whoever's nearest in reach
    let m0 = g.ui_model();
    let vi = match &a.villager {
        Some(v) => match v.parse::<usize>() {
            Ok(i) if i < g.villagers.len() => i,
            Ok(i) => { out.emit(&format!("no villager {i} - {} on the roll", g.villagers.len())); return; }
            Err(_) => {
                let want = v.to_ascii_lowercase();
                m0.people.iter().find(|p| p.label.as_deref()
                    .map(|l| l.to_ascii_lowercase().contains(&want)).unwrap_or(false))
                    .and_then(|p| g.villagers.iter().position(|&e| e == p.e))
                    .unwrap_or_else(|| { out.emit(&format!("nobody named '{v}' here")); usize::MAX })
            }
        },
        None => m0.talk_target.unwrap_or(0),
    };
    if vi == usize::MAX { return; }
    let who = g.villagers[vi];
    // --warm: the lab's heard their self-interest already (Known entries
    // non-empty), which is what puts organizing asks on the talk (#1191)
    if a.warm {
        // the seat's own ledger (E7-1): warm marks what this organizer
        // heard, kept on their player entity
        let pe = g.seats[0].player;
        for r in game2_core::game::desires::unrevealed(&g.world, pe, who, g.tick) {
            game2_core::game::desires::hear(&mut g.world, pe, who, r, g.tick);
        }
    }
    let vname = m0.people.iter().find(|p| p.e == who)
        .and_then(|p| p.label.clone()).unwrap_or_else(|| format!("villager {vi}"));
    out.emit(&format!("=== Talk: {vname} (villager {vi}, game seed {}) ===", a.seed));
    // the persona's fact table (E3-1): what the talk's criteria see -
    // open facts are voice-observable; the hidden half is theirs alone
    let vs = game2_core::session::dialogue::speaker_of(&g.world, who, &vname, g.tick);
    out.emit(&format!("  facts: {}", vs.facts.open.join(" ")));
    if !vs.facts.hidden.is_empty() {
        out.emit(&format!("  facts (self-only): {}", vs.facts.hidden.join(" ")));
    }

    g.command(Command::Select(vi));
    g.command(Command::Act(0)); // Talk

    let mut seen = 0usize;       // convo rows already printed
    let mut opened = false;      // the Talk's been on the box
    let mut walking_said = false;
    let mut talk_seed = 0u64;
    let mut frames = 0u64;
    loop {
        g.update();
        frames += 1;
        if frames > 30_000 { out.emit("(gave up waiting)"); break; }
        if let Some(t) = &g.seats[0].talk {
            opened = true;
            // the profile and gist come through the convo rows already
            // (gist_lines); printing t.profile here doubles them (#1191)
            if talk_seed == 0 { talk_seed = t.seed; }
        }
        let m = g.ui_model();
        let waiting = match &m.convo {
            Some(c) => {
                if c.rows.len() < seen { seen = 0; } // the box trimmed
                for r in &c.rows[seen..] {
                    let line = match (&r.name, r.you, r.gist) {
                        (Some(_), true, _) => format!("  you: {}", r.text),
                        (Some(n), _, true) => format!("  {n}: {}   [gist]", r.text),
                        (Some(n), _, _) => format!("  {n}: {}", r.text),
                        (None, _, _) => format!("  ({})", r.text),
                    };
                    out.emit(&line);
                }
                seen = c.rows.len();
                if !c.options.is_empty() {
                    for (i, o) in c.options.iter().enumerate() { out.emit(&format!("    {}) {}", i + 1, o)); }
                    if c.leave { out.emit("    l) take your leave"); }
                    true
                } else { false }
            }
            None => false,
        };
        if waiting {
            let c = m.convo.clone().unwrap();
            match next_pick(&c.options, &mut pq, a.auto, stdin) {
                Some(p) => {
                    taken.push(p);
                    if c.decides { g.command(Command::Decide(p)); }
                    else { g.command(Command::TalkPick(Some(p))); }
                }
                None => { g.command(Command::TalkPick(None)); break; }
            }
            continue;
        }
        if opened && g.seats[0].talk.is_none() { break; } // the talk ran its course
        if !opened {
            // the click started the walk over - or failed to: say which,
            // don't just sit on a blank screen
            if !walking_said && frames > 6 {
                out.emit("  (walking over…)");
                walking_said = true;
            }
            if g.seats[0].approach.is_none() && frames > 6 {
                match &m.blocked[0] {
                    Some(b) => out.emit(&format!("  (can't: {b})")),
                    None => out.emit("  (no way over to them - the talk never opened)"),
                }
                for e in m.log.iter().take(4) { out.emit(&format!("  | {}", e.text)); }
                break;
            }
        }
    }
    out.emit(&format!("REPRO: talk_lab talk --seed {} --villager {} --picks {} {}{}",
        a.seed, a.villager.clone().unwrap_or_else(|| vi.to_string()),
        taken.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(","),
        if a.seeded_cast { "--cast seeded " } else { "" },
        if talk_seed != 0 { format!("(talk seed {talk_seed})") } else { String::new() }));
}

// --------------------------------------------------- synthetic-mode talk

/// The flag-built path: both speakers are `Speaker`s off the args - no
/// session, so no asks/gist (those live on the org's state); the picks
/// drive `converse_with` one replay deeper each turn, which is exactly
/// how `PlayerTalk` paces the UI.
fn run_talk_synthetic(a: &Args, out: &mut Out, taken: &mut Vec<usize>, stdin: &io::Stdin) {
    let mut pq = a.picks.clone();
    let (them, you) = synthetic_pair(a);
    let rel = a.rel.unwrap_or(Relation::Acquaintance);
    out.emit(&format!("=== Talk: {} (synthetic) vs {} - {:?} ===", them.name, you.name, rel));
    out.emit(&format!("  them: {}", backstory(&them)));
    out.emit(&format!("  them's facts: {}", them.facts.open.join(" ")));
    out.emit(&format!("  you : {}", backstory(&you)));
    let lib = village_library();
    let mut picks: Vec<usize> = vec![];
    let mut shown = 0usize;
    loop {
        let mut need: Option<Vec<(Act, String)>> = None;
        let mut used_picks = 0usize;
        let tr = converse_with(&them, &you, rel, None, &lib, None, 1, &mut |ch: &[Choice]| {
            if used_picks < picks.len() {
                let p = picks[used_picks];
                used_picks += 1;
                Some(p.min(ch.len().saturating_sub(1)))
            } else {
                need = Some(ch.iter().map(|c| (c.act, c.preview.clone())).collect());
                None
            }
        }, &mut Rng::new(a.seed));
        for t in &tr.turns[shown..] {
            out.emit(&format!("  {}: {}", tr.speakers[t.speaker], t.text));
        }
        shown = tr.turns.len();
        match need {
            None => {
                let vibe = tr.vibe.map(|v| format!(", vibe: {v:?}")).unwrap_or_default();
                out.emit(&format!("  -- {:?} (trees: {:?}{vibe})", tr.ending.unwrap_or(Ending::Exhausted), tr.trees_walked));
                break;
            }
            Some(opts) => {
                let labels: Vec<String> = opts.iter().map(|(act, preview)| format!("[{act:?}] {preview}")).collect();
                for (i, o) in labels.iter().enumerate() { out.emit(&format!("    {}) {}", i + 1, o)); }
                match next_pick(&labels, &mut pq, a.auto, stdin) {
                    Some(p) => { picks.push(p); taken.push(p); }
                    None => { out.emit("  -- LeavesEarly"); break; }
                }
            }
        }
    }
    out.emit(&format!("REPRO: talk_lab talk {} --seed {} --rel {:?} --picks {}",
        speaker_flags(a), a.seed, rel,
        taken.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")));
}

/// The speaker the flags describe, plus a plain "you" for them to face.
fn synthetic_pair(a: &Args) -> (Speaker, Speaker) {
    let mut them = Speaker::new(&a.name.clone().unwrap_or_else(|| "Eliza".into()));
    them.traits = a.traits.clone();
    them.ideology = a.ideology.as_deref().unwrap_or("laborer").to_string().leak();
    for s in &a.statuses { let st: &'static str = s.clone().leak(); them.status.push(st); }
    // the persona-lite profile (E3-1): archetype:* and value:* facts
    // reach the transcript even for a flag-built speaker
    them.facts = game2_core::game::conversation::lite_facts(&them.traits, a.seed);
    // "Voice" is what the game calls the player - keeps greetings like
    // "Morrow, Voice." reading the way they do in the UI (#1190)
    let you = Speaker::new("Voice");
    (them, you)
}

/// The flags a REPRO needs to rebuild `them`.
fn speaker_flags(a: &Args) -> String {
    let mut s = format!("--name {}", a.name.clone().unwrap_or_else(|| "Eliza".into()));
    if let Some(i) = &a.ideology { s += &format!(" --ideology {i}"); }
    for t in &a.traits { s += &format!(" --trait {}", format!("{t:?}").to_ascii_lowercase()); }
    for st in &a.statuses { s += &format!(" --status {st}"); }
    s
}

// ------------------------------------------------------------- meeting

/// A house meeting, watched: the organizer puts the prompt, the floor
/// threads it. Synthetic roster off the seed; a real meeting is the
/// sandbox's job (Gather there, then watch the convo box).
fn run_meeting_mode(a: &Args, out: &mut Out) {
    let mut rng = Rng::new(a.seed);
    let roster = synthetic_roster(a.folk + 1, &mut rng);
    let organizer = Speaker::new(&a.name.clone().unwrap_or_else(|| "Reeve".into()));
    let attendees: Vec<&Speaker> = roster.iter().take(a.folk).collect();
    let pools: Vec<Vec<MeetingLine>> = attendees.iter().map(|s| meeting_pool(s)).collect();
    let pool_refs: Vec<&[MeetingLine]> = pools.iter().map(|p| p.as_slice()).collect();
    out.emit(&format!("=== House meeting: {} convenes {} (seed {}) ===",
        organizer.name, attendees.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", "), a.seed));
    for s in &attendees { out.emit(&format!("  - {} [{}]: {}", s.name, s.ideology, backstory(s))); }
    let statements = run_meeting(&organizer, &attendees, &[], &MEET_PROMPT, &pool_refs, &MEET_RITES, &mut rng);
    for (i, st) in statements.iter().enumerate() {
        let who = if st.speaker == usize::MAX { organizer.name.as_str() } else { attendees[st.speaker].name.as_str() };
        let replying = st.replying_to.map(|t| format!(" (re #{t})")).unwrap_or_default();
        // the statement text can already carry a "Name: " lead - don't say it twice
        let text = st.text.strip_prefix(&format!("{who}: ")).map(|s| s.to_string()).unwrap_or_else(|| st.text.clone());
        out.emit(&format!("  #{i} {who} [{:?}/{:?}]{replying}: {text}", st.act, st.kind));
    }
    out.emit(&format!("REPRO: talk_lab meeting --seed {} --folk {} --name {}",
        a.seed, a.folk, organizer.name));
}

/// A seeded roster in dialogue_gen's shape: name, ideology, 1-3 traits,
/// one status - enough variety that meetings and batches don't repeat.
fn synthetic_roster(n: usize, rng: &mut Rng) -> Vec<Speaker> {
    const IDEOLOGIES: [&str; 4] = ["laborer", "merchant", "clergy", "lord"];
    const STATUSES: [&str; 5] = ["well", "sick", "hungry", "grieving", "drunk"];
    const TRAITS: [Trait; 8] = [Trait::Lazy, Trait::Ambitious, Trait::Kind, Trait::Stingy,
        Trait::Pious, Trait::Hotheaded, Trait::Gregarious, Trait::Proud];
    (0..n).map(|i| {
        let mut s = Speaker::new(NAMES[i % NAMES.len()]);
        s.ideology = IDEOLOGIES[rng.below(IDEOLOGIES.len())];
        let mut ix: Vec<usize> = (0..TRAITS.len()).collect();
        for _ in 0..1 + rng.below(3) {
            let p = rng.below(ix.len());
            s.traits.push(TRAITS[ix.remove(p)]);
        }
        let st = STATUSES[rng.below(STATUSES.len())];
        if st != "well" { s.status.push(st); }
        // a griever's dead keeps one body (#1692): the condolence
        // pronouns read `mourned`, not a fresh roll per line
        if st == "grieving" { s.mourned = Some(if i % 2 == 0 { "male" } else { "female" }); }
        // the ear's landmark (E4-7, #1416): the index deal, distinct
        // per speaker like the cast's
        s.idiolect = Some(game2_core::game::idiolect::Idiolect {
            phrase: game2_core::game::idiolect::PHRASES[i % game2_core::game::idiolect::PHRASES.len()],
            habit: game2_core::game::idiolect::HABITS[(i * 7 + 3) % game2_core::game::idiolect::HABITS.len()],
        });
        s
    }).collect()
}

// ------------------------------------------------------------- sandbox

/// The free shell over a real session: play it like the game. Any convo
/// box that comes up - a Talk, a meeting's lines, the founding call, an
/// agenda slate, a dilemma - shows its rows live; numbered answers go
/// to TalkPick or Decide as the box itself says (`decides`).
fn run_sandbox(a: &Args, out: &mut Out, stdin: &io::Stdin) {
    let mut g = Game::new_seeded(a.seed);
    if let Some(s) = a.stage { stage(&mut g, s); }
    if a.plan { stage_plan(&mut g); }
    if a.days > 0 { run_days(&mut g, a.days); }
    out.emit(&format!("=== Sandbox: seed {}{} ===", a.seed,
        a.stage.map(|s| format!(", {s:?}")).unwrap_or_default()));
    out.emit("commands: who | sel N | talk | act N | pick <text> | card | meet | ctx | ctx N | 1-9 (answer the box) |");
    out.emit("          leave | frame [N] | day [N] | log | quit");
    let mut seen = 0usize;
    let mut log_head: Option<String> = None;
    let mut taken: Vec<usize> = vec![];
    let mut cmds: Vec<String> = vec![];
    loop {
        // a few frames between commands, then whatever the box says now -
        // an approach walk or a meeting's next line lands between prompts
        for _ in 0..8 { g.update(); }
        let m = g.ui_model();
        if let Some(c) = &m.convo {
            if c.rows.len() < seen { seen = 0; }
            for r in &c.rows[seen..] {
                out.emit(&format!("  {}: {}", r.name.clone().unwrap_or_else(|| "—".into()), r.text));
            }
            seen = c.rows.len();
            if !c.options.is_empty() {
                for (i, o) in c.options.iter().enumerate() { out.emit(&format!("    {}) {}", i + 1, o)); }
                if c.leave { out.emit("    l) take your leave"); }
            }
        }
        // the log's newest-first and rolling: print lines newer than the
        // last head, oldest first (rolls lose the tail - fine for a shell)
        let fresh: Vec<&str> = m.log.iter()
            .take_while(|e| Some(&e.text) != log_head.as_ref())
            .map(|e| e.text.as_str()).collect();
        for l in fresh.iter().rev() { out.emit(&format!("  | {l}")); }
        if let Some(e) = m.log.first() { log_head = Some(e.text.clone()); }

        print!("sandbox> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 { break; }
        let line = line.trim().to_string();
        if line.is_empty() { continue; }
        cmds.push(line.clone());
        let mut w = line.split_whitespace();
        match w.next().unwrap_or("") {
            "q" | "quit" => break,
            "who" => {
                let m = g.ui_model();
                for (i, e) in g.villagers.iter().enumerate() {
                    let name = m.people.iter().find(|p| p.e == *e)
                        .and_then(|p| p.label.clone()).unwrap_or_default();
                    if !name.is_empty() { out.emit(&format!("  {i:2} {name}")); }
                }
            }
            "sel" | "select" => {
                if let Some(Ok(i)) = w.next().map(|s| s.parse::<usize>()) { g.command(Command::Select(i)); }
            }
            "talk" => g.command(Command::Act(0)),
            "act" => if let Some(Ok(i)) = w.next().map(|s| s.parse::<usize>()) { g.command(Command::Act(i)); },
            "ctx" => {
                let m = g.ui_model();
                if w.next().is_some() {
                    if let Some(Ok(i)) = line.split_whitespace().nth(1).map(|s| s.parse::<usize>()) {
                        g.command(Command::CtxAct(i));
                    }
                } else {
                    for (i, c) in m.ctx.iter().enumerate() {
                        let label = m.ctx_labels.get(i).cloned().unwrap_or_else(|| format!("{c:?}"));
                        let blocked = m.ctx_blocked.get(i).cloned().flatten()
                            .map(|b| format!("   [{b}]")).unwrap_or_default();
                        out.emit(&format!("  {i}: {label}{blocked}"));
                    }
                }
            }
            "pick" => {
                // the option whose text holds the rest of the line -
                // a scripted playtest names the pick a player'd mean,
                // not the slot it happened to land in
                let want = line[4..].trim().to_ascii_lowercase();
                if let Some(c) = g.ui_model().convo.clone() {
                    match c.options.iter().position(|o| o.to_ascii_lowercase().contains(&want)) {
                        Some(p) => {
                            taken.push(p);
                            if c.decides { g.command(Command::Decide(p)); }
                            else { g.command(Command::TalkPick(Some(p))); }
                        }
                        None => out.emit(&format!("  (no option holds '{want}')")),
                    }
                }
            }
            "card" => {
                // the selection's inspector card as the client reads it
                match g.ui_model().inspector {
                    Some(ins) => for l in &ins.lines { out.emit(&format!("  | {}", l.text)); },
                    None => out.emit("  (nothing picked)"),
                }
            }
            "meet" => {
                let msg = g.call_meeting();
                out.emit(&format!("  {msg}"));
            }
            "leave" | "l" => g.command(Command::TalkPick(None)),
            "frame" | "f" => {
                let n = w.next().and_then(|s| s.parse().ok()).unwrap_or(1);
                for _ in 0..n { g.update(); }
            }
            "day" | "days" | "d" => {
                let n = w.next().and_then(|s| s.parse().ok()).unwrap_or(1);
                run_days(&mut g, n);
                out.emit(&format!("  (day {})", g.tick));
            }
            "log" => {
                for e in g.ui_model().log.iter().take(12) { out.emit(&format!("  | {}", e.text)); }
            }
            _ => {
                if let Ok(n) = line.parse::<usize>() {
                    // a bare number answers the box
                    if let Some(c) = g.ui_model().convo.clone() {
                        if n >= 1 && n <= c.options.len() {
                            let p = n - 1;
                            taken.push(p);
                            if c.decides { g.command(Command::Decide(p)); }
                            else { g.command(Command::TalkPick(Some(p))); }
                        }
                    }
                } else {
                    out.emit("  ? (who/sel/talk/act/ctx/1-9/leave/frame/day/log/quit)");
                }
            }
        }
    }
    out.emit(&format!("REPRO: talk_lab sandbox --seed {}{} -- then: {}",
        a.seed, a.stage.map(|s| format!(" --stage {s:?}")).unwrap_or_default(),
        cmds.join("; ")));
}

// --------------------------------------------------------------- batch

/// N conversations off the seeded roster, auto-picked - the headless
/// corpus builder. `--out` writes the whole dump as markdown.
fn run_batch(a: &Args, out: &mut Out) {
    let mut rng = Rng::new(a.seed);
    let roster = synthetic_roster(a.n.max(2), &mut rng);
    let lib = village_library();
    let rels = [Relation::Stranger, Relation::Acquaintance, Relation::Friend, Relation::Kin, Relation::Rival];
    out.emit(&format!("# Talk lab batch: {} conversations (seed {})\n", a.n, a.seed));
    for i in 0..a.n {
        let (ia, ib) = (i % roster.len(), (i + 1 + rng.below(roster.len() - 1)) % roster.len());
        let (x, y) = if ia < ib { (ia, ib) } else { (ib, ia) };
        let rel = rels[rng.below(rels.len())];
        // E5-9 (#1446): a spread of places - the street and market for
        // the public face, the alehouse and kitchen for the hidden one
        // (the batch's friends and kin get let backstage by the bond),
        // the law's ear on a few to flip backstage public again
        let setting = game2_core::game::setting::Setting {
            place: match rng.below(10) {
                0 | 1 => game2_core::game::setting::Place::Market,
                2 => game2_core::game::setting::Place::ChurchDoor,
                3 | 4 | 5 => game2_core::game::setting::Place::Alehouse,
                6 | 7 => game2_core::game::setting::Place::Kitchen,
                _ => game2_core::game::setting::Place::Field,
            },
            law_earshot: rng.below(8) == 0,
            invited: false,
        };
        let cseed = a.seed.wrapping_mul(31).wrapping_add(i as u64);
        // auto-pick with the run's own rng, recording the picks
        let mut prng = Rng::new(cseed);
        let mut made: Vec<usize> = vec![];
        let tr = converse_with_at(&roster[x], &roster[y], rel, None, &lib, None, 1, &setting, &mut |ch: &[Choice]| {
            let p = prng.below(ch.len());
            made.push(p);
            Some(p)
        }, &mut Rng::new(cseed));
        out.emit(&format!("## {} and {} - {:?} at {} ({:?}) (seed {cseed}, picks {:?})",
            roster[x].name, roster[y].name, rel, tr.setting.place.slug(), tr.setting.channel(), made));
        out.emit(&format!("them: {}", backstory(&roster[x])));
        out.emit(&format!("you : {}\n", backstory(&roster[y])));
        for t in &tr.turns {
            out.emit(&format!("{} [{:?} in {}]: {}", tr.speakers[t.speaker], t.act, t.tree, t.text));
        }
        let vibe = tr.vibe.map(|v| format!(", vibe: {v:?}")).unwrap_or_default();
        out.emit(&format!("-- {:?}{vibe}\n", tr.ending.unwrap_or(Ending::Exhausted)));
    }
}

/// The arc scale (E0-4, #1405): one villager, `--talks` talks against a
/// scripted `--policy`, the whole arc as JSON - the same seed and policy
/// give byte-identical output. The metrics table (E0-5) reads this.
fn run_arc_mode(a: &Args, out: &mut Out) {
    use game2_core::game::arc::{policy_of, run_arc};
    let policy = a.policy.as_deref().and_then(policy_of).unwrap_or_else(|| {
        eprintln!("arc needs --policy, one of: curious responsive greedy silent");
        std::process::exit(2);
    });
    let lib = village_library();
    // the seeded roster the batch draws on: --villager picks by index or
    // name, so a villager is reproducible across seeds and policies
    let roster = synthetic_roster(32, &mut Rng::new(a.seed));
    let them = a.villager.as_deref()
        .and_then(|v| v.parse::<usize>().ok().and_then(|i| roster.get(i))
            .or_else(|| roster.iter().find(|s| s.name.eq_ignore_ascii_case(v))))
        .unwrap_or(&roster[0]).clone();
    let you = Speaker::new("Wanderer");
    let rel = a.rel.unwrap_or(Relation::Acquaintance);
    let (them_after, arc) = run_arc(&them, &you, rel, None, a.talks, policy, &lib,
        &game2_core::game::setting::STREET, a.seed);
    let open_json = |o: &game2_core::game::conversation::Open| serde_json::json!({
        "intimacy": o.intimacy, "benevolence": o.benevolence,
        "integrity": o.integrity, "safety": o.safety,
        "mood": o.mood, "talks": o.talks, "shared": o.shared,
    });
    let talks: Vec<serde_json::Value> = arc.iter().map(|t| {
        let tr = &t.transcript;
        serde_json::json!({
            "i": t.index,
            "seed": t.seed,
            "lines": tr.turns.iter().map(|turn| serde_json::json!({
                "speaker": tr.speakers[turn.speaker],
                "act": format!("{:?}", turn.act),
                "tree": turn.tree,
                "channel": format!("{:?}", turn.channel),
                "text": turn.text,
            })).collect::<Vec<_>>(),
            "picks": t.picks,
            "choices": t.choices_made.iter().map(|(act, r, line)| serde_json::json!({
                "act": format!("{:?}", act),
                "respond": format!("{:?}", r),
                "preview": line,
            })).collect::<Vec<_>>(),
            "ending": tr.ending.map(|e| format!("{:?}", e)),
            "vibe": tr.vibe.map(|v| format!("{:?}", v)),
            "asked": tr.asked,
            "forward": tr.forward,
            "shared": tr.shared,
            "open_after": open_json(&t.open_after),
        })
    }).collect();
    let doc = serde_json::json!({
        "kind": "arc",
        "seed": a.seed,
        "policy": policy.name(),
        "relation": format!("{:?}", rel),
        "villager": {
            "name": them.name,
            "ideology": them.ideology,
            "traits": them.traits.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>(),
            "status": them.status,
            "trade": them.trade,
        },
        "talks": talks,
        "open_final": open_json(&them_after.open),
    });
    out.emit(&serde_json::to_string_pretty(&doc).unwrap());
}

/// The Appendix A metrics table (E0-5, #1409): arcs over the real
/// cast, all four policies - own-voice share, distinct word pairs and
/// Self-BLEU across talks and across villagers, the rule-of-three
/// catalog's coverage, and callback truth read off live talks.
/// `--folk` sizes the cast sample, `--talks` the arc's length.
fn run_metrics_mode(a: &Args, out: &mut Out) {
    use game2_core::game::arc::POLICIES;
    use game2_core::game::arc_metrics::{self, measure_arc, LATE_FROM};
    use game2_core::game::episodes::{self, PairMemory};
    use game2_core::session::dialogue::speaker_of;

    let lib = village_library();
    let mut g = Game::new_seeded_cast(a.seed);
    let n = a.folk.max(2).min(g.villagers.len());
    let cast: Vec<Speaker> = g.villagers.iter().take(n).map(|&v| {
        let mut s = speaker_of(&g.world, v, &g.name_of(v), g.tick);
        // what live talk setup adds: a fresh pair carries an unspent
        // slip budget, and the doorknob worry the world seeds - the
        // metrics count those channels, so the cast needs them live
        s.slips_left = 1;
        s.worry = game2_core::game::desires::worry_of(&g.world, v, g.tick, a.seed);
        s
    }).collect();
    let you = Speaker::new("Wanderer");
    let rel = Relation::Acquaintance;
    let setting = &game2_core::game::setting::STREET;

    // the arc rows: each (villager, policy) arc measured; the villager's
    // line corpora kept for the cast-level Self-BLEU
    let mut sums: Vec<(f64, f64, f64, f64)> = vec![(0.0, 0.0, 0.0, 0.0); POLICIES.len()];
    let mut corpora: Vec<Vec<Vec<String>>> = vec![vec![]; POLICIES.len()];
    for them in &cast {
        for (pi, &policy) in POLICIES.iter().enumerate() {
            let seed = a.seed.wrapping_mul(97).wrapping_add(pi as u64)
                .wrapping_add(them.name.bytes().map(|b| b as u64).sum::<u64>());
            let m = measure_arc(them, &you, rel, None, a.talks, policy, &lib, setting, seed);
            let s = &mut sums[pi];
            s.0 += m.own_early; s.1 += m.own_late;
            s.2 += m.word_pairs as f64; s.3 += m.bleu;
            corpora[pi].push(m.corpora.into_iter().flatten().collect());
        }
    }
    let cn = cast.len() as f64;
    let cast_bleu: Vec<f64> = corpora.iter().map(|cs| arc_metrics::self_bleu(cs)).collect();
    let early: Vec<f64> = sums.iter().map(|s| s.0 / cn).collect();
    let late: Vec<f64> = sums.iter().map(|s| s.1 / cn).collect();
    let pairs: Vec<f64> = sums.iter().map(|s| s.2 / cn).collect();
    let bleu: Vec<f64> = sums.iter().map(|s| s.3 / cn).collect();

    // rule of three: the shipped fact types carrying tell+line+choice
    let (covered, shipped, missing) = arc_metrics::rule_of_three();

    // callback truth: file an episode per villager, open their talk,
    // and every `mem:*` fact on the speaker must trace to a filed one -
    // two villagers go unfiled so "nothing remembered from nothing"
    // shows in the count too
    let pe = g.seats[0].player;
    let probe = (n + 2).min(g.villagers.len());
    for (i, &v) in g.villagers.iter().take(probe).enumerate() {
        if i < n {
            episodes::file(&mut g.world, v, pe,
                episodes::MEM_KINDS[i % episodes::MEM_KINDS.len()], g.tick, &["metric"]);
        }
    }
    let (mut traced, mut total) = (0usize, 0usize);
    let probed: Vec<game2_core::engine::ecs::Entity> =
        g.villagers.iter().take(probe).copied().collect();
    for v in probed {
        g.open_player_talk(0, v, Relation::Acquaintance, "the day's work");
        let t = g.seats[0].talk.as_ref().expect("a talk under way");
        let eps: Vec<episodes::PairEp> = g.world.get_component::<PairMemory>(v)
            .filter(|m| m.pe == pe).map(|m| m.eps.clone()).unwrap_or_default();
        for f in t.villager.facts.open.iter().filter(|f| f.starts_with("mem:")) {
            total += 1;
            if episodes::mem_kind(f)
                .map(|k| eps.iter().any(|e| e.kind == k)).unwrap_or(false) {
                traced += 1;
            }
        }
        g.seats[0].talk = None;
        g.world.remove_component::<game2_core::game::villager::InPlayerTalk>(v);
    }

    out.emit(&format!("=== Appendix A metrics (E0-5) - seed {}, {} villagers x {} policies x {} talks ===",
        a.seed, n, POLICIES.len(), a.talks));
    out.emit("");
    out.emit("| metric | value | target |");
    out.emit("| --- | --- | --- |");
    out.emit(&format!("| own-voice share, talks {}+ | {:.2} | >= 0.30 |",
        LATE_FROM, late.iter().sum::<f64>() / late.len() as f64));
    out.emit(&format!("| own-voice share, talks 0-{} | {:.2} | (deepening signal) |",
        LATE_FROM, early.iter().sum::<f64>() / POLICIES.len() as f64));
    out.emit(&format!("| distinct word pairs, 20 talks | {:.0} | baseline |",
        pairs.iter().sum::<f64>() / POLICIES.len() as f64));
    out.emit(&format!("| Self-BLEU, within arc | {:.3} | baseline, must not rise |",
        bleu.iter().sum::<f64>() / POLICIES.len() as f64));
    out.emit(&format!("| Self-BLEU, across villagers | {:.3} | baseline, must not rise |",
        cast_bleu.iter().sum::<f64>() / cast_bleu.len() as f64));
    out.emit(&format!("| rule-of-three coverage | {}/{} ({:.0}%) | 100% of shipped fact types |",
        covered, shipped, 100.0 * covered as f64 / shipped.max(1) as f64));
    out.emit(&format!("| callback truth | {}/{} ({:.0}%) | 100% |",
        traced, total, if total == 0 { 100.0 } else { 100.0 * traced as f64 / total as f64 }));
    out.emit("");
    out.emit(&format!("per-policy own-voice share (talks {}+):", LATE_FROM));
    for (pi, &policy) in POLICIES.iter().enumerate() {
        out.emit(&format!("  {:<10} {:.2}", policy.name(), sums[pi].1 / cn));
    }
    out.emit("per-policy Self-BLEU (arc / cast):");
    for (pi, &policy) in POLICIES.iter().enumerate() {
        out.emit(&format!("  {:<10} {:.3} / {:.3}", policy.name(),
            sums[pi].3 / cn, cast_bleu[pi]));
    }
    if !missing.is_empty() {
        out.emit(&format!("uncovered fact types: {}", missing.join(", ")));
    }
    out.emit(&format!("REPRO: talk_lab metrics --seed {} --folk {} --talks {}",
        a.seed, n, a.talks));
}

// ------------------------------------------------------------ feedback

/// The point of the lab: end of a run, ask what was wrong, file it.
/// A verdict becomes a GitHub issue with the transcript embedded
/// (falls back to a markdown file under evidence/ when gh won't).
fn feedback(out: &mut Out, stdin: &io::Stdin) {
    if out.quiet { return; }
    print!("\nverdict (what was wrong/odd, or blank to skip)> ");
    let _ = io::stdout().flush();
    let mut verdict = String::new();
    if stdin.lock().read_line(&mut verdict).unwrap_or(0) == 0 { return; }
    let verdict = verdict.trim().to_string();
    if verdict.is_empty() { return; }
    print!("title (blank = first words of verdict)> ");
    let _ = io::stdout().flush();
    let mut title = String::new();
    let _ = stdin.lock().read_line(&mut title);
    let title = title.trim();
    let title = if title.is_empty() {
        verdict.chars().take(70).collect::<String>()
    } else { title.to_string() };

    let body = format!("## Verdict\n{verdict}\n\n## Transcript\n```\n{}```\n", out.buf);
    let dir = std::env::temp_dir();
    let path = dir.join(format!("talk-lab-{}.md", std::process::id()));
    if std::fs::write(&path, &body).is_err() { return; }
    // the corpus label is best-effort: file without it if it isn't there
    let ok = std::process::Command::new("gh")
        .args(["issue", "create", "--title", &title,
               "--body-file", &path.to_string_lossy(),
               "--label", "conversation-lab"])
        .status().map(|s| s.success()).unwrap_or(false)
        || std::process::Command::new("gh")
            .args(["issue", "create", "--title", &title, "--body-file", &path.to_string_lossy()])
            .status().map(|s| s.success()).unwrap_or(false);
    if !ok {
        let dest = format!("evidence/talk-lab-{}.md", std::process::id());
        let _ = std::fs::create_dir_all("evidence");
        if std::fs::write(&dest, &body).is_ok() {
            out.emit(&format!("gh failed - wrote {dest} (file it yourself)"));
        }
    }
    let _ = std::fs::remove_file(&path);
}

// ---------------------------------------------------------------- main

fn usage() {
    println!("{}", &"talk_lab <mode> [flags]
  talk      1:1 Talk. real: --seed S --villager N|name [--stage S --days D]
            synthetic: --name X --trait T --status s --ideology i --rel R --seed S
  meeting   watch a house meeting: --seed S --folk N --name organizer
  sandbox   free session shell (all picked panels live here): --seed S --stage S --days D
  batch     N auto-picked synthetic talks: -n N --seed S --out FILE
  arc       one villager, --talks T scripted-policy talks, JSON out:
            --villager N|name --policy curious|responsive|greedy|silent --seed S
  metrics   the Appendix A table over the real cast: --folk N --talks T --seed S
  replay    any mode + --picks a,b,c replays the run those picks came from
  smoke     scripted non-interactive run - the test hook
flags: --picks a,b,c | --auto | --warm | --feedback off | --out FILE | --folk N
          --warm: mark their reveals heard on the seat's ledger first, so organizing asks show".to_string());
}

fn main() {
    game2_core::infra::crash::install();
    let a = parse_args();
    let stdin = io::stdin();
    let mut out = Out { buf: String::new(), quiet: false };
    let mut taken: Vec<usize> = vec![];

    match a.mode.as_str() {
        "talk" | "replay" => {
            if a.villager.is_some() || a.stage.is_some() || a.days > 0 {
                run_talk_real(&a, &mut out, &mut taken, &stdin);
            } else if a.name.is_none() {
                // no context at all: the nearest real villager it is
                run_talk_real(&a, &mut out, &mut taken, &stdin);
            } else {
                run_talk_synthetic(&a, &mut out, &mut taken, &stdin);
            }
        }
        "meeting" => run_meeting_mode(&a, &mut out),
        "sandbox" => run_sandbox(&a, &mut out, &stdin),
        "batch" => run_batch(&a, &mut out),
        "arc" => run_arc_mode(&a, &mut out),
        "metrics" => run_metrics_mode(&a, &mut out),
        "smoke" => {
            out.quiet = true;
            let mut a2 = parse_args();
            a2.auto = true; a2.feedback = false;
            run_talk_real(&a2, &mut out, &mut taken, &stdin);
            if out.buf.lines().count() < 3 {
                eprintln!("smoke: the talk produced no lines");
                std::process::exit(1);
            }
            out.quiet = false;
            out.emit("smoke: ok");
        }
        _ => { usage(); return; }
    }

    if let Some(path) = &a.out {
        match std::fs::write(path, &out.buf) {
            // the name alone: a corpus's logs diff clean whatever dir
            // they were written to (E0-7, #1407)
            Ok(()) => {
                let file = std::path::Path::new(path).file_name()
                    .and_then(|n| n.to_str()).unwrap_or(path);
                out.emit(&format!("(transcript -> {file})"));
            }
            // a failed write can't pass for a written transcript (E0-7)
            Err(e) => { eprintln!("could not write {path}: {e}"); std::process::exit(1); }
        }
    }
    if a.feedback && a.mode != "batch" && a.mode != "smoke" && a.mode != "arc" && a.mode != "metrics" {
        feedback(&mut out, &stdin);
    }
}
