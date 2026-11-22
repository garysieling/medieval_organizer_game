//! Talk dump: run a seeded village forward and write what folk said to
//! each other - chats in passing, house meetings, meals - as one JSON
//! array. The real in-game conversations, for the conversation review
//! tool (tools/convo-review) to flip through.
//!
//! Usage:
//!   cargo run -p game2-core --bin talk_dump -- [--seed 7] [--days 20] [--max 100] [-o out.json]

use std::collections::HashSet;
use std::env;

use game2_core::session::dialogue::TalkKind;
use game2_core::session::stage::run_days;
use game2_core::session::Game;

fn main() -> std::io::Result<()> {
    game2_core::infra::crash::install();
    let args: Vec<String> = env::args().collect();
    let (mut seed, mut days, mut max, mut out_path) = (7u64, 20u64, 100usize, None);
    let mut i = 1;
    while i < args.len() {
        let val = args.get(i + 1).cloned();
        match (args[i].as_str(), val) {
            ("-s" | "--seed", Some(v)) => { seed = v.parse().unwrap_or(seed); i += 2; }
            ("-d" | "--days", Some(v)) => { days = v.parse().unwrap_or(days); i += 2; }
            ("-n" | "--max", Some(v)) => { max = v.parse().unwrap_or(max); i += 2; }
            ("-o" | "--output", Some(v)) => { out_path = Some(v); i += 2; }
            _ => { i += 1; }
        }
    }

    let mut g = Game::new_seeded(seed);
    let mut seen: HashSet<u64> = HashSet::new();
    let mut out: Vec<serde_json::Value> = vec![];
    // a day at a time: the talk log is capped, so harvest as it fills.
    // Only records from days already over are taken - a meeting's lines
    // are put on the record as they're said
    for _ in 0..=days {
        if out.len() >= max { break; }
        run_days(&mut g, 1);
        for r in &g.talk_log {
            if out.len() >= max { break; }
            if r.tick >= g.tick || seen.contains(&r.id) { continue; }
            // the player's own talks wait on their picks - there's no
            // player at the keys here
            if r.kind == TalkKind::Talk { continue; }
            seen.insert(r.id);
            let spoken: Vec<_> = r.lines.iter().filter(|l| l.who.is_some()).collect();
            if spoken.len() < 2 { continue; }
            let mut speakers: Vec<String> = vec![];
            for l in &r.lines {
                if l.who.is_some() && !speakers.contains(&l.name) { speakers.push(l.name.clone()); }
            }
            out.push(serde_json::json!({
                "id": format!("village-s{}-{}", seed, r.id),
                "source": "village",
                "kind": format!("{:?}", r.kind),
                "day": r.tick,
                "speakers": speakers.iter().map(|n| serde_json::json!({ "name": n })).collect::<Vec<_>>(),
                "lines": r.lines.iter().map(|l| serde_json::json!({
                    "speaker": if l.who.is_some() { l.name.clone() } else { String::new() },
                    "narration": l.who.is_none(),
                    "gist": l.gist,
                    "text": l.text,
                })).collect::<Vec<_>>(),
            }));
        }
    }
    let text = serde_json::to_string(&out).expect("talks serialize");
    match out_path {
        Some(p) => std::fs::write(p, text)?,
        None => println!("{}", text),
    }
    Ok(())
}
