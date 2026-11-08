use crate::session::*;

#[test]
fn test_the_road_stamps_its_strangers() {
    let mut g = Game::new();
    g.maybe_migrant(50);
    assert!(g.villagers.iter().any(|&v|
        g.world.get_component::<villager::Migrant>(v).is_some()),
        "the newcomer isn't marked");
}

#[test]
fn test_the_strangers_share_sends_the_petition() {
    let mut g = Game::new();
    // a score of newcomers and the wives go up
    for d in 0..26 { g.maybe_migrant(50 + d); }
    g.petition_tick(60);
    assert!(g.petition.is_some(), "the strangers passed and nobody spoke");
}

#[test]
fn test_the_lords_esteem_decides_the_answer() {
    let mut g = Game::new();
    g.petition = Some(Petition { started: 50, answer_on: 57 });
    g.lord_esteem = 0.8;
    g.petition_tick(58);
    assert!(g.ordinance.is_some(), "the esteemed lord refused");
    let mut g = Game::new();
    g.petition = Some(Petition { started: 50, answer_on: 57 });
    g.lord_esteem = 0.2;
    g.petition_tick(58);
    assert!(g.ordinance.is_none(), "the scorned lord granted");
}

#[test]
fn test_the_ordinance_dries_the_tap() {
    let mut g = Game::new();
    g.ordinance = Some(Ordinance { passed_on: 50, price_mult: 1.5, licensed: true });
    let Some(tavern) = g.tavern else { return };
    let Some(keeper) = g.world.get_component::<Building>(tavern).and_then(|b| b.worker) else { return };
    // the holy day goes dark
    g.tavern_bench(4, tavern, keeper); // tick%7==4, but the gate's in tavern_tick -
    // call the tick itself to prove the dark
    let bar_before = g.world.get_component::<Building>(tavern)
        .map(|b| b.stock.get("ale").copied().unwrap_or(0)).unwrap_or(0);
    g.tavern_tick(4 + 0); // the 4th is the dark day - and 4%2==0 so restock would run if it weren't dark
    let bar_after = g.world.get_component::<Building>(tavern)
        .map(|b| b.stock.get("ale").copied().unwrap_or(0)).unwrap_or(0);
    assert_eq!(bar_before, bar_after, "the holy day's tap ran anyway");
}
