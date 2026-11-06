use crate::session::*;

#[test]
fn test_the_winter_runs_long_and_the_store_thins() {
    let mut g = Game::new();
    g.winter_tick(400);
    assert!(g.harsh_winter, "every other winter's a hard one");
    assert!(g.log.iter().any(|l| l.contains("sets in hard")));
    if let Some(b) = g.world.get_component_mut::<Building>(g.market_food) {
        household::add_stock(b, "grain", 20);
    }
    for t in (400..410).step_by(5) { g.winter_tick(t); }
    let grain = g.world.get_component::<Building>(g.market_food)
        .map(|b| b.stock.get("grain").copied().unwrap_or(0)).unwrap_or(0);
    assert!(grain < 20, "the stores run thin");
}

#[test]
fn test_a_weak_village_looks_for_who_to_blame() {
    let mut g = Game::new();
    g.lord_esteem = 0.0;
    let herb = g.herbologist.unwrap();
    for t in (400..500).step_by(30) { g.winter_tick(t); } // 400,430,460,490 - all %30==10 in the window
    assert!(desires::problems_of(&g.world, herb, 500).contains(&desires::Problem::Blamed),
        "the herb-wife's hut looks different in a hard winter");
}

#[test]
fn test_an_org_worth_the_name_pulls_together() {
    let mut g = Game::new();
    g.lord_esteem = 0.6;
    for v in g.villagers.iter().take(6).copied().collect::<Vec<_>>() {
        if let Some(h) = g.world.get_component_mut::<Hunger>(v) { h.value = 40.0; }
    }
    // tick 1150: 1150 % 730 = 420 (in the harsh window), 1150 % 60 = 10 (the share-out day)
    g.winter_tick(1150);
    assert!(g.log.iter().any(|l| l.contains("pulls together")),
        "an org shares what it has");
    // and the herb-wife takes no blame for a shared winter
    let herb = g.herbologist.unwrap();
    assert!(!desires::problems_of(&g.world, herb, 1150).contains(&desires::Problem::Blamed),
        "belonging is the counterweight");
}

#[test]
fn test_a_snowball_fight_calls_the_village_out() {
    let mut g = Game::new();
    // winter's button shows, once a day
    g.world.get_component_mut::<crate::sim::climate::Climate>(g.climate).unwrap().season = crate::sim::climate::Season::Winter;
    g.update_ctx_actions();
    assert!(g.seats[0].ctx_actions.contains(&CtxAct::Snowballs), "the winter button shows");
    let i = g.seats[0].ctx_actions.iter().position(|a| *a == CtxAct::Snowballs).unwrap();
    g.ctx_act_at(0, i);
    assert_eq!(g.snowball_day, g.tick);
    assert!(g.log.iter().any(|l| l.contains("Snowball fight")), "{:?}", g.log.last());
    let playing = g.villagers.iter().filter(|&&v| g.world.get_component::<routine::Routine>(v)
        .map(|r| r.mode == routine::Mode::Playing && r.day == g.tick).unwrap_or(false)).count();
    assert!(playing > 10, "{} folk came out to play", playing);
    // and the snowballs start flying
    let mut saw = false;
    for _ in 0..200 { g.update(); saw |= !g.snowballs.is_empty(); }
    assert!(saw, "snowballs in the air");
    // once a day: the button's gone till tomorrow
    g.update_ctx_actions();
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::Snowballs), "not twice in one day");
    // and the hubbub reaches the frame, not the void between clicks
    g.snowball_cued = false;
    g.update();
    assert!(g.sfx.iter().any(|c| c.kind == "snowballs"), "the fight is heard");
}

#[test]
fn test_a_snowball_fight_too_far_out_finds_no_ears() {
    let mut g = Game::new();
    // the Wanderer's well outside town: no villager's path is in reach
    if let Some(p) = g.world.get_component_mut::<Pos>(g.player) { p.x += 60.0; }
    let msg = g.snowball_fight_at(0);
    assert!(msg.contains("doesn't carry"), "{}", msg);
    assert_ne!(g.snowball_day, g.tick, "the day's call isn't spent on nobody");
}
