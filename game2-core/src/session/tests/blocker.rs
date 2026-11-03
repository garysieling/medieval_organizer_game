use crate::session::*;

#[test]
fn test_invest_is_greyed_out_with_no_gold() {
    let mut game = Game::new();
    game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().resources = 5.0;
    assert_eq!(game.action_blocker(2).as_deref(), Some("need 10 gold"));
    game.act(2);
    assert!(game.log.last().unwrap().ends_with("Can't invest: need 10 gold."), "{}", game.log.last().unwrap());
    game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().resources = 10.0;
    assert_eq!(game.action_blocker(2), None);
}

#[test]
fn test_recruit_is_greyed_out_for_a_member() {
    let mut game = Game::new();
    let view = View::new(Art::load_fs("../assets", 0));
    let v = game.villagers[2];
    game.world.get_component_mut::<Organization>(game.org).unwrap().members.push(v);
    // beside them, so Recruit doesn't turn to someone nearer (reach_target)
    let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: vx + 0.5, y: vy };
    game.selected_target = 2;
    assert_eq!(game.action_blocker(1).as_deref(), Some("already a member"));
    // the log line mustn't read as a success (log_colour greens "joined")
    game.act(1);
    assert_ne!(log_colour(game.log.last().unwrap()), (120, 255, 120), "{}", game.log.last().unwrap());
    // and the button is drawn disabled, with the reason beside it
    let buf = view.render(&game.ui_model(), (-1, -1));
    let b = &action_buttons()[1];
    let i = ((b.y + 4) as usize * W + (b.x + 4) as usize) * 4;
    assert_ne!((buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]), b.color, "Recruit still drawn in its live colour");
}

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
