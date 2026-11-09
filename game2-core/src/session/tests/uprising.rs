use crate::session::*;

#[test]
fn test_the_grievance_boils_over() {
    let mut g = Game::new();
    for i in 0..10 {
        let v = g.villagers[i % g.villagers.len()];
        g.world.add_component(v, crate::game::desires::InnerLife {
            wants: vec![], grievances: vec![(crate::game::desires::Problem::Taxed, 10)], satisfied: vec![] });
    }
    g.uprising_tick(115);
    assert!(g.uprising.is_some(), "the mob never rose");
    assert!(g.log.iter().any(|l| l.contains("the mob rises")),
        "the rising never logged");
}

#[test]
fn test_the_soldiers_take_the_caught() {
    let mut g = Game::new();
    let m = g.villagers[0];
    g.world.add_component(m, crate::game::desires::InnerLife {
        wants: vec![], grievances: vec![(crate::game::desires::Problem::Taxed, 10)], satisfied: vec![] });
    for i in 1..10 {
        let v = g.villagers[i % g.villagers.len()];
        g.world.add_component(v, crate::game::desires::InnerLife {
            wants: vec![], grievances: vec![(crate::game::desires::Problem::Taxed, 10)], satisfied: vec![] });
    }
    let before = g.repression_debt;
    g.uprising_tick(115);
    assert!(g.repression_debt > before, "the debt never landed");
    let taken = g.world.get_component::<crate::game::dungeon::Imprisoned>(m).is_some()
        || g.villagers.iter().any(|&v| g.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some());
    assert!(taken, "the soldiers never took anyone");
}

#[test]
fn test_the_quiet_village_stays_quiet() {
    let mut g = Game::new();
    g.uprising_tick(115);
    assert!(g.uprising.is_none(), "a quiet village rose anyway");
}
