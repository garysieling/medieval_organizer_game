use crate::session::*;

#[test]
fn test_work_names_what_is_missing_instead_of_nothing() {
    let mut g = Game::new();
    // a baker with neither wheat in hand nor a shop's stock to draw on
    g.world.add_component(g.player, vocation::craft(Vocation::Baker, 0.3));
    g.seats[0].ctx_actions = vec![CtxAct::Work];
    g.ctx_act_at(0, 0);
    let last = g.log.last().unwrap();
    assert!(last.contains("no wheat"), "{}", last);
    // with wheat in the satchel the day makes bread (a street bench
    // works at half rate: two days a loaf) and says where it went
    g.world.get_component_mut::<Satchel>(g.player).unwrap().goods.insert("wheat".into(), 2);
    g.ctx_act_at(0, 0);
    g.ctx_act_at(0, 0);
    let last = g.log.last().unwrap();
    assert!(last.contains("1 bread made") && last.contains("in your satchel"), "{}", last);
}

#[test]
fn test_a_tasked_heir_reports_the_inherited_shop_once() {
    let mut g = Game::new();
    // the player's bench is the shop their senior left them - the
    // senior's card still names it as their workplace too
    let senior = g.villagers[5];
    let shop = g.building_entities.iter().copied().find(|&b|
        g.world.get_component::<Building>(b)
            .map(|bb| matches!(bb.kind, BuildingKind::Shop(_))).unwrap_or(false)).unwrap();
    let mut craft = vocation::craft(Vocation::Fisher, 0.3);
    craft.workplace = Some(shop);
    craft.tasked_by = Some(senior);
    g.world.add_component(g.player, craft);
    if g.world.get_component::<Craft>(senior).is_none() {
        g.world.add_component(senior, vocation::craft(Vocation::Fisher, 0.9));
    }
    g.world.get_component_mut::<Craft>(senior).unwrap().workplace = Some(shop);
    g.seats[0].ctx_actions = vec![CtxAct::Work];
    g.ctx_act_at(0, 0);
    let last = g.log.last().unwrap();
    assert!(last.contains("1 fish made"), "{}", last);
    // the shared shelf's take is named once - not again as the senior's
    assert!(!last.contains("through"), "{}", last);
}

#[test]
fn test_the_players_promotion_names_the_craft() {
    let mut g = Game::new();
    // a bench-day short of Apprentice, wheat in hand for the day
    g.world.add_component(g.player, vocation::craft(Vocation::Baker, 0.396));
    g.world.get_component_mut::<Satchel>(g.player).unwrap().goods.insert("wheat".into(), 2);
    g.seats[0].ctx_actions = vec![CtxAct::Work];
    g.ctx_act_at(0, 0);
    let last = g.log.last().unwrap();
    assert!(last.contains("reach Apprentice Baker!"), "{}", last);
}
