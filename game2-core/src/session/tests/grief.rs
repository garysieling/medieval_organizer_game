use crate::session::*;

#[test]
fn test_the_closest_ones_mourn_the_dead() {
    let mut g = Game::new();
    let mourner = g.villagers[0];
    let dead = g.villagers[1];
    g.world.add_component(mourner, crate::sim::psychology::Bond { target: dead, security: 0.8 });
    g.world.add_component(mourner, crate::sim::psychology::AttachmentHistory {
        anxiety_episodes: 0, warmth_episodes: 2, episodes: vec![] });
    let before = g.world.get_component::<crate::sim::psychology::AttachmentHistory>(mourner)
        .unwrap().anxiety_episodes;
    g.world.get_component_mut::<Health>(dead).unwrap().condition = 0.0;
    g.grief_tick(1);
    // the dead are mourned
    assert!(g.world.get_component::<crate::sim::psychology::Mourned>(dead).is_some());
    // the bond takes a Loss - and the grief's a grievance the village reads
    let h = g.world.get_component::<crate::sim::psychology::AttachmentHistory>(mourner).unwrap();
    assert!(h.anxiety_episodes > before, "no loss recorded");
    let life = g.world.get_component::<crate::game::desires::InnerLife>(mourner).unwrap();
    assert!(life.grievances.iter().any(|(p, _)| *p == crate::game::desires::Problem::Bereaved),
        "the grief carried no grievance");
    assert!(g.log.iter().any(|l| l.contains("mourn")), "the mourning went untold");
    // and only once
    g.grief_tick(2);
    let h2 = g.world.get_component::<crate::sim::psychology::AttachmentHistory>(mourner).unwrap();
    assert_eq!(h2.anxiety_episodes, h2.anxiety_episodes, "they mourned twice");
}
