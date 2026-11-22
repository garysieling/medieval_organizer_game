//! Talk dump: run a seeded village forward and write what folk said to
//! each other - chats in passing, house meetings, meals - as one JSON
//! array. The real in-game conversations, for the conversation review
//! tool (tools/convo-review) to flip through. Each speaker who's a
//! villager carries their full character sheet - secrets and all - and
//! the figure the map draws them as, a PNG data URI, so the reviewer
//! sees who's talking.
//!
//! Lives in the client crate for the art: the portrait is resolved the
//! way the conversation box resolves it (#1319).
//!
//! Usage:
//!   cargo run -p game2-client --bin talk_dump -- [--seed 7] [--days 20] [--max 100] [-o out.json]

use std::collections::{HashMap, HashSet};
use std::env;

use game2_client::Art;
use game2_core::engine::sprite::Sprite;
use game2_core::session::dialogue::TalkKind;
use game2_core::session::stage::run_days;
use game2_core::session::Game;

fn main() -> std::io::Result<()> {
    game2_core::infra::crash::install();
    let args: Vec<String> = env::args().collect();
    let (mut seed, mut days, mut max, mut out_path) = (7u64, 20u64, 100usize, None);
    let mut assets = "assets".to_string();
    let mut i = 1;
    while i < args.len() {
        let val = args.get(i + 1).cloned();
        match (args[i].as_str(), val) {
            ("-s" | "--seed", Some(v)) => { seed = v.parse().unwrap_or(seed); i += 2; }
            ("-d" | "--days", Some(v)) => { days = v.parse().unwrap_or(days); i += 2; }
            ("-n" | "--max", Some(v)) => { max = v.parse().unwrap_or(max); i += 2; }
            ("-o" | "--output", Some(v)) => { out_path = Some(v); i += 2; }
            ("--assets", Some(v)) => { assets = v; i += 2; }
            _ => { i += 1; }
        }
    }

    let art = Art::load_fs(&assets, seed);
    let mut g = Game::new_seeded(seed);
    let mut seen: HashSet<u64> = HashSet::new();
    let mut out: Vec<serde_json::Value> = vec![];
    // each figure encodes once: the same faces come up again and again
    let mut portraits: HashMap<usize, Option<String>> = HashMap::new();
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
            // speakers in the order they first speak, each sheet read as
            // it stood the day the talk's harvested
            let mut speakers: Vec<serde_json::Value> = vec![];
            let mut named: Vec<&str> = vec![];
            for l in &r.lines {
                let Some(who) = l.who else { continue };
                if named.contains(&l.name.as_str()) { continue; }
                named.push(&l.name);
                let sheet = match g.villagers.iter().position(|&v| v == who) {
                    Some(vi) => {
                        let notes = g.character_notes(vi);
                        let portrait = portraits.entry(vi)
                            .or_insert_with(|| figure_of(&art, &notes).map(data_uri)).clone();
                        let mut j = serde_json::to_value(&notes).expect("notes serialize");
                        j["portrait"] = serde_json::json!(portrait);
                        j
                    }
                    None => serde_json::json!({ "name": l.name }),
                };
                speakers.push(sheet);
            }
            out.push(serde_json::json!({
                "id": format!("village-s{}-{}", seed, r.id),
                "source": "village",
                "kind": format!("{:?}", r.kind),
                "day": r.tick,
                "speakers": speakers,
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

/// The still the map draws villager `n` as: their trade's own figure
/// where the people art has one (sexed first), else their look - the
/// conversation box's lookup, whole figure rather than the head
fn figure_of<'a>(art: &'a Art, n: &game2_core::session::view_model::CharacterNotes) -> Option<&'a Sprite> {
    let look = art.figure_look(n.kid, n.fig);
    n.person.as_deref()
        .and_then(|t| art.person(&format!("{}-{}", t, art.look_sex(look))).or_else(|| art.person(t)))
        .or_else(|| art.villager_sprites.get(look))
}

/// A sprite as a PNG data URI, for the page to show as-is
fn data_uri(s: &Sprite) -> String {
    let mut png = vec![];
    {
        let mut enc = png::Encoder::new(&mut png, s.width as u32, s.height as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().expect("png header");
        w.write_image_data(&s.pixels).expect("png data");
    }
    format!("data:image/png;base64,{}", base64(&png))
}

fn base64(b: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(b.len().div_ceil(3) * 4);
    for c in b.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            s.push(if k <= c.len() { T[(n >> (18 - 6 * k)) as usize & 63] as char } else { '=' });
        }
    }
    s
}
