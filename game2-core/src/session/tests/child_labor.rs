use crate::session::*;

fn idle_kid_near(g: &mut Game, at: (i32, i32)) -> Entity {
    let k = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<vocation::Kid>(v).is_some()
        && !villager::is_dead(&g.world, v))
        .expect("no kid in the village");
    *g.world.get_component_mut::<Pos>(k).unwrap() = Pos { x: at.0 as f32, y: at.1 as f32 };
    k
}

#[test]
fn test_a_needy_field_hires_a_hand() {
    let mut g = Game::new();
    let field = *g.crop_fields.first().expect("no fields");
    let fp = g.world.get_component::<Pos>(field).unwrap().clone();
    let kid = idle_kid_near(&mut g, (fp.x.round() as i32, fp.y.round() as i32));
    // a farmer near enough to call them
    if let Some(farmer) = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Farmer).unwrap_or(false)) {
        *g.world.get_component_mut::<Pos>(farmer).unwrap() = Pos { x: fp.x + 2.0, y: fp.y };
    }
    g.kid_labor_tick(50);
    assert!(g.world.get_component::<vocation::KidWork>(kid).is_some(),
        "the idle kid wasn't hired");
    assert!(g.travels.iter().any(|t| t.who == kid), "the kid never walked to the field");
}

#[test]
fn test_the_days_work_pays_and_helps_the_crop() {
    let mut g = Game::new();
    let field = *g.crop_fields.first().expect("no fields");
    let kid = idle_kid_near(&mut g, (0, 0));
    let farmer = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Farmer).unwrap_or(false))
        .expect("no farmer");
    g.world.get_component_mut::<Purse>(farmer).unwrap().gold = 10.0;
    g.world.get_component_mut::<Purse>(kid).unwrap().gold = 0.0;
    let prog = g.world.get_component::<villager::Crop>(field).unwrap().progress;
    g.world.add_component(kid, vocation::KidWork { field, hirer: farmer, pay: 0.6 });
    g.kid_labor_tick(50);
    // wages landed and the crop felt the hands
    assert!(g.world.get_component::<Purse>(kid).unwrap().gold > 0.3, "no wages paid");
    let after = g.world.get_component::<villager::Crop>(field).unwrap().progress;
    assert!(after > prog, "the field didn't feel the hands");
    // and the day counts toward coming of age
    assert!(g.world.get_component::<vocation::KidGrowth>(kid)
        .map(|g| g.errands.iter().any(|(e, _)| *e == farmer)).unwrap_or(false),
        "the day didn't count");
}

#[test]
fn test_the_village_speaks_of_hurt_hands() {
    let mut g = Game::new();
    let field = *g.crop_fields.first().expect("no fields");
    let kid = idle_kid_near(&mut g, (0, 0));
    let farmer = g.villagers[0];
    // a parent close enough to see it
    *g.world.get_component_mut::<Pos>(g.villagers[5]).unwrap() = Pos { x: 0.5, y: 0.5 };
    // force the hurt roll: run many days' work through
    let mut saw_tag = false;
    for d in 0..40 {
        g.world.add_component(kid, vocation::KidWork { field, hirer: farmer, pay: 0.5 });
        g.kid_labor_tick(50 + d);
        if g.world.get_component::<crate::game::desires::InnerLife>(kid)
            .map(|l| l.grievances.iter().any(|g| g.0 == crate::game::desires::Problem::ChildLabor))
            .unwrap_or(false) { saw_tag = true; break; }
    }
    assert!(saw_tag, "forty days' work and the village never spoke of it");
}
