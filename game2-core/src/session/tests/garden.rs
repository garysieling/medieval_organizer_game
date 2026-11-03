use crate::session::*;

#[test]
fn test_the_cottages_have_gardens() {
    let g = Game::new();
    assert!(g.gardens.len() >= 4, "some homes keep a bed");
    for (home, _, _, _) in &g.gardens {
        assert_eq!(g.world.get_component::<Building>(*home).unwrap().kind,
            household::BuildingKind::Home);
    }
}

#[test]
fn test_a_full_bed_yields_into_the_house() {
    let mut g = Game::new();
    let gi = 0;
    g.gardens[gi].2 = 1.0;
    let home = g.gardens[gi].0;
    g.arrive(g.villagers[0], Goal::TendGarden(gi));
    let b = g.world.get_component::<Building>(home).unwrap();
    let picked = b.stock.get("marigold").copied().unwrap_or(0)
        + b.stock.get("feverfew").copied().unwrap_or(0);
    assert_eq!(picked, 1, "a full bed gives a bunch");
    assert_eq!(g.gardens[gi].2, 0.0, "the bed's picked clean");
}

#[test]
fn test_the_bed_fills_in_over_the_weeks() {
    let mut g = Game::new();
    let g0 = g.gardens[0].2;
    g.garden_tick(5);
    assert!(g.gardens[0].2 > g0, "it grows between pickings");
}

#[test]
fn test_a_kitchen_bed_grows_dinner_not_flowers() {
    let mut g = Game::new();
    // every other bed keeps a kitchen garden
    let gi = g.gardens.iter().position(|(_, _, _, k)| *k)
        .expect("some beds are kitchen beds");
    g.gardens[gi].2 = 1.0;
    let home = g.gardens[gi].0;
    g.tick = 0;
    g.arrive(g.villagers[0], Goal::TendGarden(gi));
    let b = g.world.get_component::<Building>(home).unwrap();
    let pot = b.stock.get("roots").copied().unwrap_or(0)
        + b.stock.get("potherbs").copied().unwrap_or(0)
        + b.stock.get("peas").copied().unwrap_or(0);
    assert_eq!(pot, 1, "a kitchen bed gives the pot, not the still");
    assert_eq!(b.stock.get("marigold").copied().unwrap_or(0)
        + b.stock.get("feverfew").copied().unwrap_or(0), 0);
}

#[test]
fn test_the_pot_turns_three_staples_into_pottage() {
    let mut g = Game::new();
    let home = g.building_entities.iter().copied().find(|&b| {
        g.world.get_component::<Building>(b)
            .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false)
    }).unwrap();
    {
        let b = g.world.get_component_mut::<Building>(home).unwrap();
        for good in ["peas", "roots", "potherbs"] { household::add_stock(b, good, 1); }
    }
    g.kitchen_tick(3); // pot day
    let b = g.world.get_component::<Building>(home).unwrap();
    assert_eq!(b.stock.get("pottage").copied().unwrap_or(0), 2);
    assert_eq!(b.stock.get("peas").copied().unwrap_or(0), 0);
}
