use crate::session::*;

#[test]
fn test_the_contact_glosses_the_words() {
    let mut g = Game::new();
    // an outlander in the ear accrues the fluency
    if g.villagers.is_empty() { return; }
    let m = g.villagers[0];
    g.world.add_component(m, crate::session::country::Outlander { site: 0 });
    g.fluent = 0.0;
    g.language_tick(39);
    assert!(g.fluent > 0.0, "the contact never glossed: {}", g.fluent);
}

#[test]
fn test_the_ledger_grows_on_the_threshold() {
    let mut g = Game::new();
    if g.villagers.is_empty() { return; }
    let m = g.villagers[0];
    g.world.add_component(m, crate::session::country::Outlander { site: 0 });
    g.fluent = 0.5;
    g.learned_words = 0;
    g.language_tick(39);
    assert!(g.learned_words > 0, "the ledger never grew");
    assert!(g.log.iter().any(|l| l.contains("a word settles")),
        "the glossing never logged");
}
