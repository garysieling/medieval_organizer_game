use crate::session::*;

#[test]
fn test_the_tuning_panel_reads_the_innards() {
    let g = Game::new();
    let gauges = g.telemetry();
    assert!(gauges.iter().any(|(k, _)| *k == "org.power"), "the power matrix's there");
    assert!(gauges.iter().any(|(k, _)| *k == "trails.heat"), "the trails heat's there");
    assert!(gauges.iter().any(|(k, _)| *k == "env.tree_cover"), "the land's score's there");
    assert!(gauges.iter().any(|(k, _)| *k == "price.fish"), "the economy's there");
    let lines = g.telemetry_lines();
    assert!(lines.iter().any(|l| l.starts_with("villagers.alive")), "a line per gauge");
}
