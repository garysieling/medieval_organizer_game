use crate::session::*;
use crate::game::contracts::{self, Contract, ContractStatus, PACT_AID};
use crate::game::feuds;
use crate::engine::ecs::System;

#[test]
fn test_the_pact_button_seals_and_the_card_says_so() {
    let mut g = Game::new();
    g.seats[0].picked = true;
    g.seats[0].selected_target = 1;
    let pe = g.seats[0].player;
    let v = g.villagers[1];
    if let Some(p) = g.world.get_component_mut::<Purse>(pe) { p.gold = 20.0; }
    let gold0 = 20.0;
    g.update_ctx_actions_at(0);
    let i = g.seats[0].ctx_actions.iter().position(|a| *a == CtxAct::Pact)
        .expect("no Swear aid button");
    g.ctx_act_at(0, i);
    assert!(contracts::has_pact(&g.world, pe, v), "no pact sealed");
    assert_eq!(g.world.get_component::<Purse>(pe).unwrap().gold, gold0 - PACT_AID);
    let insp = g.villager_inspector_at(0);
    assert!(insp.lines.iter().any(|l| l.text.contains("sworn to you")));
    // and it's one wax each: the button's gone once they're bound
    g.update_ctx_actions_at(0);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::Pact));
}

#[test]
fn test_a_burning_feud_tears_the_offenders_pact() {
    let mut g = Game::new();
    g.seats[0].picked = true;
    g.seats[0].selected_target = 1;
    let pe = g.seats[0].player;
    let v = g.villagers[1];
    if let Some(p) = g.world.get_component_mut::<Purse>(pe) { p.gold = 20.0; }
    if let Some(p) = g.world.get_component_mut::<PlayerCharacter>(pe) { p.resources = 20.0; }
    g.update_ctx_actions_at(0);
    let i = g.seats[0].ctx_actions.iter().position(|a| *a == CtxAct::Pact).expect("no Swear aid button");
    g.ctx_act_at(0, i);
    assert!(contracts::pact_live(&g.world, pe, v), "no pact sealed");
    assert_eq!(g.world.get_component::<PlayerCharacter>(pe).unwrap().resources, 10.0,
        "the panel's purse still shows the old gold");
    // the sworn villager turns on a third party and it burns: open
    // conflict breaks the aid they swore
    feuds::spawn_feud(&mut g.world, v, g.villagers[2], "a slight");
    for _ in 0..20 { feuds::FeudSystem.run(&mut g.world); }
    g.feud_tick(g.tick);
    let c = g.world.query::<Contract>()[0];
    assert_eq!(g.world.get_component::<Contract>(c).unwrap().status, ContractStatus::Breached);
    let rep = g.world.get_component::<contracts::Reputation>(v).unwrap();
    assert_eq!(rep.contracts_breached, 1);
    assert!(rep.trust_score < 0.5, "breaking faith cost nothing");
    // the torn pact shows no more, and seals no more
    let insp = g.villager_inspector_at(0);
    assert!(!insp.lines.iter().any(|l| l.text.contains("sworn to you")));
}
