use crate::session::*;

#[test]
fn test_the_agent_who_asks_for_food_gets_a_meal_not_a_stand() {
    let mut g = Game::new();
    let who = g.villagers[0];
    g.world.get_component_mut::<Hunger>(who).unwrap().value = 20.0;
    let servings = g.world.get_component::<villager::FoodSource>(g.market_food).unwrap().servings;
    // Eat queues a real meal at the store, not just a stand beside it
    g.seats.push(Seat { player: who, ..Seat::default() });
    g.agent_command(1, AgentCmd::Eat).unwrap();
    assert!(g.travels.iter().any(|t| t.who == who && matches!(t.goal, Goal::Eat)));
    // and on arrival - the walk done, standing at the table - a
    // serving goes down and the belly fills
    let sp = g.world.get_component::<Pos>(g.market_food).copied().unwrap();
    *g.world.get_component_mut::<Pos>(who).unwrap() = sp;
    g.arrive(who, Goal::Eat);
    assert_eq!(g.world.get_component::<Hunger>(who).unwrap().value, 100.0);
    assert_eq!(g.world.get_component::<villager::FoodSource>(g.market_food).unwrap().servings,
        servings - 1, "a real serving went down");
}

#[test]
fn test_the_hungry_walk_to_the_store_not_just_stand_about() {
    let mut g = Game::new();
    let who = g.villagers[0];
    // low enough that a stray egg breakfast still leaves them hungry
    g.world.get_component_mut::<Hunger>(who).unwrap().value = 2.0;
    let sp = g.world.get_component::<Pos>(g.market_food).copied().unwrap();
    *g.world.get_component_mut::<Pos>(who).unwrap() = Pos { x: sp.x + 6.0, y: sp.y };
    g.livelihood_tick(1);
    assert!(g.travels.iter().any(|t| t.who == who && matches!(t.goal, Goal::Eat)),
        "a hungry villager is sent for the meal, not left standing");
}

#[test]
fn test_a_dead_hirer_pays_no_coin() {
    let mut g = Game::new();
    let kid = g.villagers[26];
    let hirer = g.villagers[0];
    let to = g.building_entities[0];
    g.world.get_component_mut::<Satchel>(kid).unwrap().goods.insert("wheat".into(), 2);
    if g.world.get_component::<Purse>(kid).is_none() {
        g.world.add_component(kid, Purse { gold: 0.0 });
    }
    g.world.add_component(kid, OddJob {
        hirer, good: "wheat".into(), qty: 2, from: hirer, to, pay: 5.0, loaded: true });
    g.world.add_component(hirer, villager::Dead);
    let stock_before = g.world.get_component::<Building>(to).unwrap().stock.get("wheat").copied().unwrap_or(0);
    let kid_gold = g.world.get_component::<Purse>(kid).unwrap().gold;
    g.arrive(kid, Goal::RunOddJob);
    assert_eq!(g.world.get_component::<Purse>(kid).unwrap().gold, kid_gold,
        "a corpse pays no coin");
    assert_eq!(g.world.get_component::<Building>(to).unwrap().stock.get("wheat").copied().unwrap_or(0),
        stock_before + 2, "the goods land in the place's stock");
    assert!(g.world.get_component::<OddJob>(kid).is_none(), "the errand ends there");
}

#[test]
fn test_the_dead_finish_no_errands() {
    let mut g = Game::new();
    let who = g.villagers[0];
    g.world.get_component_mut::<Hunger>(who).unwrap().value = 20.0;
    let sp = g.world.get_component::<Pos>(g.market_food).copied().unwrap();
    g.travels.push(Travel { who, path: vec![(sp.x.round() as i32, sp.y.round() as i32)], goal: Goal::Eat });
    g.world.add_component(who, villager::Dead);
    g.advance_travels();
    assert!(!g.travels.iter().any(|t| t.who == who), "the errand ends where they fell");
    assert_ne!(g.world.get_component::<Hunger>(who).unwrap().value, 100.0, "a corpse eats nothing");
}

#[test]
fn test_no_conversation_is_had_with_the_dead() {
    let mut g = Game::new();
    let (who, e) = (g.villagers[0], g.villagers[1]);
    g.world.add_component(e, villager::Dead);
    g.arrive(who, Goal::TalkTo(e));
    assert!(g.world.query::<relationships::Relationship>().iter()
        .filter(|&&r| g.world.get_component::<relationships::Relationship>(r)
            .map(|rr| rr.from == who && rr.other == e).unwrap_or(false))
        .count() == 0, "a grave isn't a conversation");
}

#[test]
fn test_the_hungry_at_the_store_eat_where_they_stand() {
    let mut g = Game::new();
    let who = g.villagers[0];
    g.world.get_component_mut::<Hunger>(who).unwrap().value = 2.0;
    let sp = g.world.get_component::<Pos>(g.market_food).copied().unwrap();
    *g.world.get_component_mut::<Pos>(who).unwrap() = Pos { x: sp.x, y: sp.y };
    g.livelihood_tick(1);
    assert_eq!(g.world.get_component::<Hunger>(who).unwrap().value, 100.0,
        "already at the store: the meal goes down now");
}

#[test]
fn test_the_agent_who_the_cause_reached_signs_on_and_writes_for_the_held() {
    let mut g = Game::new();
    // a villager outside the taxed four: a stranger to the cause
    let who = g.villagers[6];
    g.seats.push(Seat { player: who, org: g.org, agent: true, ..Seat::default() });
    // a cause to sign on with
    for &v in &g.villagers[..4] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..4].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    // a stranger to it is refused
    assert!(g.agent_command(1, AgentCmd::JoinOrg).is_err());
    // but the woken sign on without being convinced
    g.world.add_component(who, crate::game::reactions::Sympathized {
        issues: vec![("the taxes".into(), g.tick)] });
    g.agent_command(1, AgentCmd::JoinOrg).unwrap();
    assert!(g.world.get_component::<Organization>(g.org).unwrap().members.contains(&who));
    // and letters reach a prisoner, whoever writes them
    let held = g.villagers[1];
    assert!(g.agent_command(1, AgentCmd::Press { prisoner: held }).is_err(), "no one inside yet");
    g.world.add_component(held, crate::game::dungeon::Imprisoned { since: g.tick, charge: "sedition".into() });
    g.agent_command(1, AgentCmd::Press { prisoner: held }).unwrap();
    assert!(g.world.get_component::<crate::game::dungeon::ReleaseCampaign>(held)
        .map(|r| r.pressure > 0.0).unwrap_or(false), "the letters pile up");
}
