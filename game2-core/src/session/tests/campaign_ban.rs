use crate::session::*;

#[test]
fn test_a_rule_is_written_against_the_crowd_that_wont_stop() {
    let mut g = Game::new();
    let pe = g.player;
    for &v in &g.villagers[..5] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
        for _ in 0..3 {
            crate::game::relationships::record_interaction(&mut g.world, v,
                crate::game::relationships::Interaction {
                    other: pe, kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
        }
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..5].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    g.seats[0].ctx_actions.push(CtxAct::March);

    // first crowd: the knights come out and take the loudest
    g.ctx_act_at(0, 0);
    for _ in 0..12000 { g.update(); if g.formation.is_none() { break; } }
    assert!(g.log.iter().any(|l| l.contains("knights come out")), "{:?}", g.log);
    // second crowd: the law is written - the street answers once a day
    g.tick += 1;
    g.seats[0].ctx_actions = vec![CtxAct::March];
    g.ctx_act_at(0, 0);
    for _ in 0..12000 { g.update(); if g.formation.is_none() { break; } }
    assert!(g.log.iter().any(|l| l.contains("no more than two assembled")), "{:?}", g.log);
    assert!(g.world.get_component::<crate::game::reactions::AssemblyBan>(g.org).is_some());
    // and under the ban even a handful is broken up on sight
    g.tick += 1;
    g.seats[0].ctx_actions = vec![CtxAct::March];
    g.ctx_act_at(0, 0);
    for _ in 0..12000 { g.update(); if g.formation.is_none() { break; } }
    assert!(g.log.iter().any(|l| l.contains("knights come out")), "{:?}", g.log);
}
