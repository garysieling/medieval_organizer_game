use crate::session::*;

fn give_case(g: &mut Game, who: Entity, sev: f32) {
    let d = g.world.create_entity();
    g.world.add_component(d, crate::sim::illness::Disease {
        kind: crate::sim::illness::DiseaseKind::Gripes,
        name: "the gripes".into(), patient: who,
        stage: crate::sim::illness::Stage::Moderate, severity: sev,
        contagious: false, incubation: 0,
    });
}

#[test]
fn test_three_deep_and_he_rides_in() {
    let mut g = Game::new();
    let three: Vec<Entity> = g.villagers.iter().copied().take(3).collect();
    for v in three { give_case(&mut g, v, 0.5); }
    g.doctor_tick(50);
    assert!(g.doctor.is_some(), "three deep and nobody came");
}

#[test]
fn test_the_village_mends_and_he_rides_home() {
    let mut g = Game::new();
    g.doctor = Some(Doctor { e: g.world.create_entity(), rounds: vec![], fee_bank: 0.0 });
    // nobody sick - the road takes him back
    g.doctor_tick(50);
    assert!(g.doctor.is_none(), "the sickness passed and he stayed anyway");
}

#[test]
fn test_his_hands_cost_and_sometimes_help() {
    let mut g = Game::new();
    let three: Vec<Entity> = g.villagers.iter().copied().take(3).collect();
    let patient = three[0];
    for v in three { give_case(&mut g, v, 0.5); }
    g.world.get_component_mut::<Purse>(patient).unwrap().gold = 10.0;
    g.doctor_tick(50);
    g.doctor_tick(51);
    // he visited somebody: the purse or the sickness moved
    let moved = g.doctor.as_ref().map(|d| d.fee_bank > 0.0).unwrap_or(false)
        || g.world.get_component::<Purse>(patient).unwrap().gold < 10.0;
    assert!(moved, "he walked in and touched nobody's purse");
}
