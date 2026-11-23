use crate::session::*;

/// Stand the player at their bench with a day's worth of presence
/// banked - the next ambient beat lands the batch (#1338)
fn at_the_bench(g: &mut Game) {
    let bench = g.world.get_component::<Craft>(g.seats[0].player).unwrap().workplace.unwrap();
    let (bx, by) = { let p = g.world.get_component::<Pos>(bench).unwrap(); (p.x, p.y) };
    let p = g.world.get_component_mut::<Pos>(g.seats[0].player).unwrap();
    p.x = bx; p.y = by;
    g.seats[0].ambient_bench = crate::session::verbs::AMBIENT_WORK_DAYS;
}

fn a_shop(g: &Game) -> crate::engine::ecs::Entity {
    g.building_entities.iter().copied().find(|&b|
        g.world.get_component::<Building>(b)
            .map(|bb| matches!(bb.kind, BuildingKind::Shop(_))).unwrap_or(false)).unwrap()
}

#[test]
fn test_an_idle_bench_names_what_is_missing() {
    let mut g = Game::new();
    // a baker with neither wheat in hand nor a shop's stock to draw on
    let shop = a_shop(&g);
    let mut craft = vocation::craft(Vocation::Baker, 0.3);
    craft.workplace = Some(shop);
    g.world.add_component(g.player, craft);
    at_the_bench(&mut g);
    g.ambient_bench_at(0);
    let last = g.log.last().unwrap();
    assert!(last.contains("no wheat"), "{}", last);
    // the want is a once-a-day word, not a per-frame nag
    g.seats[0].ambient_bench = crate::session::verbs::AMBIENT_WORK_DAYS;
    let n = g.log.len();
    g.ambient_bench_at(0);
    assert_eq!(g.log.len(), n, "the bench doesn't nag twice in a day");
    // with wheat in the satchel the day's presence makes bread, and
    // the log says where it went
    g.world.get_component_mut::<Satchel>(g.player).unwrap().goods.insert("wheat".into(), 2);
    g.seats[0].ambient_bench = crate::session::verbs::AMBIENT_WORK_DAYS;
    g.ambient_bench_at(0);
    let last = g.log.last().unwrap();
    assert!(last.contains("1 bread made"), "{}", last);
}

#[test]
fn test_a_tasked_heir_reports_the_inherited_shop_once() {
    let mut g = Game::new();
    // the player's bench is the shop their senior left them - the
    // senior's card still names it as their workplace too
    let senior = g.villagers[5];
    let shop = a_shop(&g);
    let mut craft = vocation::craft(Vocation::Fisher, 0.3);
    craft.workplace = Some(shop);
    craft.tasked_by = Some(senior);
    g.world.add_component(g.player, craft);
    if g.world.get_component::<Craft>(senior).is_none() {
        g.world.add_component(senior, vocation::craft(Vocation::Fisher, 0.9));
    }
    g.world.get_component_mut::<Craft>(senior).unwrap().workplace = Some(shop);
    at_the_bench(&mut g);
    g.ambient_bench_at(0);
    let last = g.log.last().unwrap();
    assert!(last.contains("1 fish made"), "{}", last);
    // the shared shelf's take is named once - not again as the senior's
    assert!(!last.contains("through"), "{}", last);
}

#[test]
fn test_the_players_promotion_names_the_craft() {
    let mut g = Game::new();
    // a bench-day short of Apprentice, wheat in hand for the day
    let shop = a_shop(&g);
    let mut craft = vocation::craft(Vocation::Baker, 0.396);
    craft.workplace = Some(shop);
    g.world.add_component(g.player, craft);
    g.world.get_component_mut::<Satchel>(g.player).unwrap().goods.insert("wheat".into(), 2);
    at_the_bench(&mut g);
    g.ambient_bench_at(0);
    let last = g.log.last().unwrap();
    assert!(last.contains("reach Apprentice Baker!"), "{}", last);
}
