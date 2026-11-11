use crate::session::*;
use crate::sim::magic;

#[test]
fn test_the_priests_reading_teaches_the_parish() {
    let mut g = Game::new();
    // roll until the priest opens a book - a discovery lands and the
    // log names it
    let mut found = false;
    for t in 1..200 {
        g.magic_tick(t);
        if !g.grimoire.discoveries.is_empty() { found = true; break; }
    }
    assert!(found, "two hundred days and the priest never opened a book");
    assert!(g.log.iter().any(|l| l.contains("the priest's reading turns up")),
        "the reading went unsaid: {:?}", g.log.last());
    // and a page of the healing arts mends the sick a little faster
    let v = g.villagers[0];
    if g.world.get_component::<crate::sim::illness::Health>(v).is_some() {
        g.world.get_component_mut::<crate::sim::illness::Health>(v).unwrap().condition = 0.5;
        g.world.add_component(v, crate::sim::illness::Disease {
            kind: crate::sim::illness::DiseaseKind::Gripes, name: "the gripes".into(),
            patient: v, stage: crate::sim::illness::Stage::Mild, severity: 0.3,
            contagious: false, incubation: 0, recovering: 0 });
        magic::ingest_text(&mut g.grimoire, "Medieval herbals - healing",
            magic::MagicDomain::Healing, 0.5);
        let c0 = g.world.get_component::<crate::sim::illness::Health>(v).unwrap().condition;
        g.magic_tick(201);
        let c1 = g.world.get_component::<crate::sim::illness::Health>(v).unwrap().condition;
        assert!(c1 > c0, "the physick did nothing: {} -> {}", c0, c1);
    }
}
