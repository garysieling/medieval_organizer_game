use crate::session::*;

#[test]
fn test_the_bench_days_work_reads_on_the_card() {
    let mut g = Game::new();
    // run days till somebody's bench day lands (work needs inputs in hand)
    while g.tick < 6 && g.produced.is_empty() { g.update(); }
    assert!(!g.produced.is_empty(), "nobody made anything in six days");
    // the card says what they made today
    let (&maker, _) = g.produced.iter().next().unwrap();
    let i = g.villagers.iter().position(|&v| v == maker).unwrap();
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    let insp = g.villager_inspector_at(0);
    assert!(insp.lines.iter().any(|l| l.text.contains("made")),
        "the day's work doesn't show: {:?}",
        insp.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());
}
