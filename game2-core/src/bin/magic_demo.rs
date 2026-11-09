//! Magic discovery: ingest ancient texts, find magic, enable mechanics.

use game2_core::sim::magic::{ancient_sources, ingest_text, magic_effects, Grimoire};

fn main() {
    game2_core::infra::crash::install();
    println!("=== MAGIC DISCOVERY ===\n");
    println!("The world is written in an ancient language...\n");

    let mut grimoire = Grimoire { discoveries: vec![], domains_known: Default::default(), total_power: 0.0 };

    // Ingest ancient sources
    for (source, domain, strength) in ancient_sources() {
        let d = ingest_text(&mut grimoire, source, domain, strength);
        println!("Discovered: {}", d.name);
        println!("  effects: {:?}", magic_effects(domain));
        println!();
    }

    println!("=== Grimoire Summary ===");
    println!("discoveries: {}", grimoire.discoveries.len());
    println!("domains known: {:?}", grimoire.domains_known);
    println!("total power: {:.2}", grimoire.total_power);
    println!("\n=== MAGIC COMPLETE ===");
}
