use crate::session::*;
use crate::engine::ecs::System;

#[test]
fn test_the_belly_empties_but_never_kills() {
    let mut g = Game::new();
    let pe = g.seats[0].player;
    g.world.get_component_mut::<Hunger>(pe).unwrap().value = 50.0;
    crate::game::villager::HungerSystem.run(&mut g.world);
    let after = g.world.get_component::<Hunger>(pe).unwrap().value;
    assert!(after < 50.0, "the wanderer's hunger never drained");
    // and with no Health it only growls - the nourishment step can't bite
    g.world.get_component_mut::<Hunger>(pe).unwrap().value = 0.0;
    crate::game::villager::nourishment_step(&mut g.world, &[]);
    assert!(g.world.get_component::<crate::sim::illness::Health>(pe).is_none());
}

#[test]
fn test_the_satchel_feeds_you_when_you_forget() {
    let mut g = Game::new();
    let pe = g.seats[0].player;
    // peckish but not starving: past the store's own trip (h < 30) so
    // the meal on trial is the satchel's
    g.world.get_component_mut::<Hunger>(pe).unwrap().value = 35.0;
    g.world.get_component_mut::<Satchel>(pe).unwrap().goods.insert("bread".into(), 2);
    g.livelihood_tick(5); // no feast day - the satchel's on its own
    let h = g.world.get_component::<Hunger>(pe).unwrap().value;
    assert!(h > 35.0, "the belly never filled: {}", h);
    assert_eq!(g.world.get_component::<Satchel>(pe).unwrap().goods.get("bread").copied().unwrap_or(0), 1);
    assert!(g.log.iter().any(|l| l.contains("you eat the bread")), "the meal went unlogged");
    // but it won't eat your tools
    g.world.get_component_mut::<Hunger>(pe).unwrap().value = 35.0;
    g.world.get_component_mut::<Satchel>(pe).unwrap().goods.retain(|k, _| k != "bread");
    g.world.get_component_mut::<Satchel>(pe).unwrap().goods.insert("tools".into(), 1);
    g.livelihood_tick(6);
    assert_eq!(g.world.get_component::<Satchel>(pe).unwrap().goods.get("tools").copied().unwrap_or(0), 1,
        "the satchel ate the tools");
}

#[test]
fn test_a_gift_feeds_the_hungry_and_warms_the_bond() {
    let mut g = Game::new();
    let pe = g.seats[0].player;
    let v = g.villagers[0];
    g.world.get_component_mut::<Satchel>(pe).unwrap().goods.insert("bread".into(), 1);
    g.world.get_component_mut::<Hunger>(v).unwrap().value = 10.0;
    // near enough to hand it over
    let (px, py) = { let p = g.world.get_component::<Pos>(pe).unwrap(); (p.x, p.y) };
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: px + 1.0, y: py };
    g.seats[0].selected_target = 0;
    g.seats[0].picked = true;
    g.seats[0].ctx_actions = vec![CtxAct::Give];
    g.ctx_act_at(0, 0);
    assert_eq!(g.world.get_component::<Satchel>(v).unwrap().goods.get("bread").copied().unwrap_or(0), 1,
        "the bread never reached their bag");
    assert_eq!(g.world.get_component::<Satchel>(pe).unwrap().goods.get("bread").copied().unwrap_or(0), 0);
    let bond = crate::game::relationships::find_relationship(&g.world, pe, v)
        .and_then(|r| g.world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    assert!(bond > 0.0, "the gift left them cold");
    assert!(g.log.iter().any(|l| l.contains("you give")), "the giving went unlogged");
}

#[test]
fn test_a_shopkeepers_talk_sells_you_what_they_make() {
    let mut g = Game::new();
    g.pending_quests.clear(); // no job offer to crowd the till's picks
    let pe = g.seats[0].player;
    g.world.get_component_mut::<Purse>(pe).unwrap().gold = 50.0;
    // a shop with a worker and bread on the shelf; the buy runs through
    // the keeper's talk now (#1339)
    let shop = g.building_entities.iter().copied().find(|&b|
        g.world.get_component::<Building>(b)
            .map(|bb| bb.worker.is_some() && matches!(bb.kind, BuildingKind::Shop(_))).unwrap_or(false))
        .expect("no working shop");
    let worker = g.world.get_component::<Building>(shop).unwrap().worker.unwrap();
    {
        let bb = g.world.get_component_mut::<Building>(shop).unwrap();
        bb.stock.clear();
        bb.stock.insert("bread".into(), 3);
        // stood beside the keeper, not the stones: the talk's what sells
        let wp = g.world.get_component::<Pos>(worker).copied().unwrap();
        *g.world.get_component_mut::<Pos>(pe).unwrap() = Pos { x: wp.x + 1.0, y: wp.y };
    }
    let wi = g.villagers.iter().position(|&v| v == worker).unwrap();
    g.talk_to_at(0, wi);
    let i = { let t = g.seats[0].talk.as_ref().expect("the talk opens");
        assert_eq!(t.wares, vec!["bread".to_string()], "the shelf's offer rides the picks");
        t.job as usize };
    g.talk_pick_at(0, Some(i));
    assert_eq!(g.world.get_component::<Satchel>(pe).unwrap().goods.get("bread").copied().unwrap_or(0), 1,
        "the bread never reached your bag");
    assert!(g.world.get_component::<Purse>(pe).unwrap().gold < 50.0, "the purse never paid");
    // and a broke purse gets told, not robbed
    g.world.get_component_mut::<Purse>(pe).unwrap().gold = 0.0;
    g.talk_pick_at(0, Some(i));
    assert_eq!(g.world.get_component::<Satchel>(pe).unwrap().goods.get("bread").copied().unwrap_or(0), 1);
    assert!(g.seats[0].talk.as_ref().unwrap().notes.iter().any(|n| n.contains("can't cover")),
        "the refusal went unsaid");
    // and a bare shelf offers no wares: clear the stock and the pick's gone
    g.world.get_component_mut::<Building>(shop).unwrap().stock.clear();
    g.talk_pick_at(0, None);
    g.talk_to_at(0, wi);
    assert!(g.seats[0].talk.as_ref().unwrap().wares.is_empty(),
        "a bare shelf still sells - the wares latched empty");
}
