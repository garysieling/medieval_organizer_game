//! docs/protocol.md's examples are real: every ```json client block parses
//! as a ClientMsg, every ```json server block as a ServerMsg, and every
//! ```json obs block as an AgentObs. Change the protocol and the doc fails
//! here until it's brought along.

use game2_core::session::{AgentObs, ClientMsg, ServerMsg};

/// (tag, body) of each fenced ```json <tag> block
fn blocks(doc: &str) -> Vec<(String, String)> {
    let mut out = vec![];
    let mut lines = doc.lines();
    while let Some(l) = lines.next() {
        if let Some(tag) = l.trim().strip_prefix("```json ") {
            let body: Vec<&str> = lines.by_ref().take_while(|l| l.trim() != "```").collect();
            out.push((tag.trim().to_string(), body.join("\n")));
        }
    }
    out
}

#[test]
fn the_protocol_docs_examples_parse() {
    let doc = include_str!("../../docs/protocol.md");
    let bs = blocks(doc);
    let count = |t: &str| bs.iter().filter(|(tag, _)| tag == t).count();
    assert!(count("client") >= 8 && count("server") >= 3 && count("obs") >= 1, "the doc lost its examples");
    for (tag, body) in &bs {
        let err = match tag.as_str() {
            "client" => serde_json::from_str::<ClientMsg>(body).err(),
            "server" => serde_json::from_str::<ServerMsg>(body).err(),
            "obs" => serde_json::from_str::<AgentObs>(body).err(),
            other => panic!("unknown example tag `{other}`"),
        };
        assert!(err.is_none(), "a `{tag}` example doesn't parse ({:?}):\n{body}", err);
    }
}
