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

#[test]
fn an_agent_knows_itself_and_no_one_elses_wants() {
    let mut g = Game::new();
    while g.tick < 2 { g.update(); }
    // a villager with a trade and a bench
    let i = (0..g.villagers.len()).find(|&i| {
        let v = g.villagers[i];
        !villager::is_dead(&g.world, v)
            && g.world.get_component::<crate::game::vocation::TownRole>(v).is_none()
            && g.world.get_component::<crate::game::vocation::Kid>(v).is_none()
            && g.world.get_component::<crate::game::vocation::Craft>(v).is_some_and(|c| c.workplace.is_some())
    }).expect("a free tradesman");
    let v = g.villagers[i];
    let seat = g.add_agent_seat(v);
    let obs = g.agent_obs(seat).unwrap();
    let craft = g.world.get_component::<crate::game::vocation::Craft>(v).unwrap();
    assert_eq!(obs.me.trade.as_deref(), Some(crate::game::vocation::vocation_name(craft.vocation)));
    assert_eq!(obs.me.bench, craft.workplace);
    assert!(obs.me.level.is_some());
    // its own wants, as the sim holds them
    let own: Vec<String> = g.world.get_component::<crate::game::desires::InnerLife>(v)
        .map(|l| l.wants.iter().filter(|w| !crate::game::desires::is_satisfied(&g.world, v, **w, g.tick))
            .map(|w| format!("{:?}", w)).collect()).unwrap_or_default();
    assert_eq!(obs.me.wants, own);
    // and nobody else's: the near list is names and distances only
    let wire = serde_json::to_string(&obs).unwrap();
    for n in &obs.near {
        assert!(!wire.contains(&format!("\"{}\",\"wants\"", n.name)), "someone else's wants on the wire");
    }
    assert!(serde_json::to_value(&obs).unwrap()["near"][0].get("wants").is_none(), "a neighbour's wants in the obs");
}

#[test]
fn an_agent_fetches_what_its_bench_is_short_of() {
    let mut g = Game::new();
    while g.tick < 2 { g.update(); }
    // a free tradesman whose bench is short of its raw good
    let short = (0..g.villagers.len()).map(|i| g.villagers[i]).find(|&v| {
        !villager::is_dead(&g.world, v)
            && g.world.get_component::<crate::game::vocation::TownRole>(v).is_none()
            && g.world.get_component::<crate::game::vocation::Kid>(v).is_none()
            && crate::game::livelihood::needs_input(&g.world, v).is_some()
            && !g.travels.iter().any(|t| t.who == v)
    }).expect("a tradesman short of inputs");
    let seat = g.add_agent_seat(short);
    let need = g.agent_obs(seat).unwrap().me.needs.expect("the obs says what's short");
    assert_eq!(Some(need.as_str()), crate::game::livelihood::needs_input(&g.world, short));
    let walking_before = g.travels.len();
    let r = g.agent_command(seat, AgentCmd::FetchInput);
    match r {
        // the errand's under way: a kid or the tradesman is walking for it
        Ok(()) => {
            assert!(g.travels.len() > walking_before, "a fetch that sent no one");
            assert_eq!(g.agent_command(seat, AgentCmd::FetchInput), Err("someone's already fetching it".into()),
                "a second fetch while the first is out");
            // an errand abandoned (a hungry child breaks off: the walk goes,
            // the OddJob stays) no longer blocks the next fetch
            g.travels.clear();
            assert_ne!(g.agent_command(seat, AgentCmd::FetchInput), Err("someone's already fetching it".into()),
                "an abandoned errand still blocks fetching");
        }
        Err(e) => assert!(e.starts_with("no "), "an unexpected refusal: {e}"),
    }
    // and a bench with all it needs is told so
    let full = (0..g.villagers.len()).map(|i| g.villagers[i]).find(|&v| {
        !villager::is_dead(&g.world, v) && crate::game::livelihood::needs_input(&g.world, v).is_none()
            && g.world.get_component::<crate::game::vocation::TownRole>(v).is_none()
            && g.world.get_component::<crate::game::vocation::Kid>(v).is_none()
            && !g.seats.iter().any(|s| s.player == v)
    }).expect("someone wanting for nothing");
    let seat = g.add_agent_seat(full);
    assert_eq!(g.agent_command(seat, AgentCmd::FetchInput), Err("the bench wants for nothing".into()));
}

/// The player's Errand button: with nothing the player needs, a picked
/// child can run paper and ink in for the print shop - the printer hires,
/// pays and is delivered to, not the player
#[test]
fn test_a_kid_can_run_an_errand_for_the_print_shop() {
    let mut g = Game::new();
    // the player needs nothing himself, so the errand looks to the shop
    if let Some(c) = g.world.get_component_mut::<Craft>(g.seats[0].player) {
        c.vocation = Vocation::Taverner;
    }
    // a printmaker with bare shelves: no paper or ink on hand or stocked
    let pi = g.villagers.iter().position(|&v| {
        g.world.get_component::<Kid>(v).is_none() && !villager::is_dead(&g.world, v)
    }).unwrap();
    let printer = g.villagers[pi];
    let shop = g.world.get_component::<Craft>(printer).and_then(|c| c.workplace);
    if let Some(c) = g.world.get_component_mut::<Craft>(printer) {
        c.vocation = Vocation::Printmaker;
    }
    if let Some(b) = shop {
        if let Some(bb) = g.world.get_component_mut::<Building>(b) {
            bb.stock.remove("paper"); bb.stock.remove("ink");
        }
    }
    if let Some(s) = g.world.get_component_mut::<Satchel>(printer) {
        s.goods.remove("paper"); s.goods.remove("ink");
    }
    assert_eq!(livelihood::needs_input(&g.world, printer), Some("paper"),
        "the printer is short of paper first");
    // somebody's satchel holds paper to fetch
    let seller = g.villagers.iter().copied().find(|&v| {
        v != printer && g.world.get_component::<Satchel>(v).is_some()
            && !villager::is_dead(&g.world, v)
    }).unwrap();
    g.world.get_component_mut::<Satchel>(seller).unwrap().goods.insert("paper".into(), 3);
    // the kid is picked and sent
    let ki = g.villagers.iter().position(|&v| g.world.get_component::<Kid>(v).is_some()).unwrap();
    let kid = g.villagers[ki];
    g.seats[0].selected_target = ki;
    g.seats[0].ctx_actions = vec![CtxAct::Errand];
    g.ctx_act_at(0, 0);
    let job = g.world.get_component::<OddJob>(kid).expect("the errand is under way");
    assert_eq!((job.hirer, job.good.as_str(), job.to), (printer, "paper", printer),
        "the printer hires and is delivered to, not the player");
    // leg one: at the source the paper's loaded
    *g.world.get_component_mut::<Pos>(kid).unwrap() = *g.world.get_component::<Pos>(seller).unwrap();
    g.arrive(kid, Goal::RunOddJob);
    assert!(g.world.get_component::<OddJob>(kid).unwrap().loaded, "the paper's in the satchel");
    // leg two: at the printer it's delivered and paid for
    *g.world.get_component_mut::<Pos>(kid).unwrap() = *g.world.get_component::<Pos>(printer).unwrap();
    g.travels.retain(|t| t.who != kid);
    let pay = g.world.get_component::<OddJob>(kid).unwrap().pay;
    let printer_gold = g.world.get_component::<Purse>(printer).map(|p| p.gold).unwrap_or(0.0);
    let kid_gold = g.world.get_component::<Purse>(kid).map(|p| p.gold).unwrap_or(0.0);
    g.arrive(kid, Goal::RunOddJob);
    assert!(g.world.get_component::<OddJob>(kid).is_none(), "the errand ends");
    assert_eq!(g.world.get_component::<Satchel>(printer).unwrap().goods.get("paper").copied().unwrap_or(0), 2,
        "the printer's satchel gets the paper");
    assert!((printer_gold - g.world.get_component::<Purse>(printer).unwrap().gold - pay.min(printer_gold)).abs() < 0.01,
        "the printer's purse pays");
    assert!((g.world.get_component::<Purse>(kid).unwrap().gold - kid_gold - pay).abs() < 0.01,
        "the kid earns the pay");
}
