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

#[test]
fn test_an_agents_bench_day_reads_on_the_card() {
    let mut g = Game::new();
    // find a villager whose hands can actually turn a bench day
    let mut found = false;
    for i in 1..g.villagers.len() {
        let v = g.villagers[i];
        let seat = g.add_agent_seat(v);
        let _ = g.agent_command(seat, AgentCmd::WorkDay);
        if g.produced.get(&v).map(|(d, m)| *d == g.tick && *m > 0.0).unwrap_or(false) {
            g.seats[0].selected_target = i;
            g.seats[0].picked = true;
            let insp = g.villager_inspector_at(0);
            assert!(insp.lines.iter().any(|l| l.text.contains("made")),
                "the agent's bench day doesn't show: {:?}",
                insp.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());
            found = true;
            break;
        }
    }
    assert!(found, "no agent could work a bench day");
}

#[test]
fn test_the_shop_card_says_what_flowed_through_today() {
    let mut g = Game::new();
    while g.tick < 6 && g.shop_flow.is_empty() { g.update(); }
    let shop = g.shop_flow.iter().find(|(_, (d, f))| *d == g.tick && !f.is_empty())
        .map(|(&b, _)| b);
    let Some(be) = shop else { panic!("no shop's ledger ran today") };
    let bi = g.building_entities.iter().position(|b| *b == be).unwrap();
    g.seats[0].selected_building = Some(bi);
    let insp = g.building_inspector_at(0).unwrap();
    assert!(insp.lines.iter().any(|l| l.text.contains("today:")),
        "the chain doesn't show: {:?}",
        insp.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());
}

#[test]
fn test_the_offer_says_dear_and_cheap_against_the_ordinary() {
    let mut g = Game::new();
    // any shop with goods on the shelf
    let (bi, good) = g.building_entities.iter().enumerate().find_map(|(i, &b)| {
        g.world.get_component::<Building>(b).and_then(|bb|
            bb.stock.iter().find(|(_, q)| **q > 0).map(|(g, _)| (i, g.clone())))
    }).expect("nothing on offer anywhere");
    g.seats[0].selected_building = Some(bi);
    // run the felt worth into the ground: a glut reads cheap
    for _ in 0..20 { goods::nudge(&mut g.world, &good, -0.1); }
    let insp = g.building_inspector_at(0).unwrap();
    assert!(insp.lines.iter().any(|l| l.text.contains("cheap")),
        "a glut should read cheap: {:?}",
        insp.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());
    // and driven scarce it reads dear
    for _ in 0..40 { goods::nudge(&mut g.world, &good, 0.1); }
    let insp = g.building_inspector_at(0).unwrap();
    assert!(insp.lines.iter().any(|l| l.text.contains("dear")),
        "bare shelves should read dear: {:?}",
        insp.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());
}
