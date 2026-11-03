use crate::session::*;

#[test]
fn test_carried_demands_lighten_the_tax_round() {
    let mut g = Game::new();
    let pe = g.player;
    for &v in &g.villagers[..4] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
        for _ in 0..3 {
            crate::game::relationships::record_interaction(&mut g.world, v,
                crate::game::relationships::Interaction {
                    other: pe, kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
        }
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..4].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();

    // a man with his own taxing to complain of carries the demand;
    // the doors stay shut on the ones who don't share the fight
    let reeve = g.collector.unwrap();
    desires::add_grievance(&mut g.world, reeve, desires::Problem::Taxed, g.tick + 200);
    g.seats[0].selected_target = COLLECTOR_IDX;
    g.seats[0].picked = true;
    g.seats[0].ctx_actions.push(CtxAct::Townhall);
    g.ctx_act_at(0, 0);
    assert!(g.log.last().unwrap().contains("king will hear"), "{:?}", g.log.last());

    // the grant lands: the cause's people stop counting the taxing
    assert!(g.world.get_component::<crate::game::reactions::LordGrant>(g.org).is_some());
    // and the campaign talks it over as a win - an outcome beat from
    // the arc library, on the TALK panel, read off the grant
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    assert!(crate::game::campaign_arc::arc_facts(&g.world, c, g.tick).has("arc:won"));
    let talk = g.talk_log.last().expect("no campaign talk recorded");
    assert!(talk.lines.len() >= 2 && talk.tick == g.tick, "{:?}", talk.lines.len());
    let arc_lines: std::collections::HashSet<String> = crate::game::campaign_dialogue::ARC_BEATS.iter()
        .flat_map(|b| b.lines.iter().chain(b.moves.iter().flat_map(|m| m.lines)))
        .map(|l| crate::game::campaign_dialogue::fill(l, "the taxes")).collect();
    assert!(arc_lines.contains(&talk.lines[0].text), "not an arc line: {:?}", talk.lines[0].text);
    assert!(!desires::problems_of(&g.world, g.villagers[1], g.tick)
        .contains(&desires::Problem::Taxed), "the grievance is answered");
}
