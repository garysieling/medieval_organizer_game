use crate::session::*;

#[test]
fn test_a_maker_lays_down_a_masterwork() {
    let mut g = Game::new();
    // a sure pair of hands
    let maker = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c|
            matches!(c.vocation, Vocation::Luthier | Vocation::Glazier | Vocation::Toolmaker
                | Vocation::Weaver | Vocation::Tailor)).unwrap_or(false)).unwrap();
    g.world.get_component_mut::<Craft>(maker).unwrap().skill = 0.9;
    g.heirloom_tick(60);
    assert!(!g.heirlooms.is_empty(), "a thing made to outlast its maker");
    let h = g.world.get_component::<villager::Heirloom>(g.heirlooms[0]).unwrap();
    assert_eq!(h.maker, maker);
    assert_eq!(h.owner, maker, "the maker keeps it first");
}

#[test]
fn test_the_printers_masterwork_pulls_a_run() {
    let mut g = Game::new();
    // a print shop with a master at the press: any villager will do,
    // the vocation and the sure hands are the fixture
    let shop = household::spawn_building(&mut g.world,
        BuildingKind::Shop(Vocation::Printmaker), "the print shop", 5.0, 5.0, 400);
    let maker = g.villagers[0];
    g.world.get_component_mut::<Craft>(maker).unwrap().vocation = Vocation::Printmaker;
    g.world.get_component_mut::<Craft>(maker).unwrap().workplace = Some(shop);
    g.world.get_component_mut::<Craft>(maker).unwrap().skill = 0.9;
    // and no other hands in the village out-work him this round
    for &v in g.villagers.clone().iter().skip(1) {
        g.world.get_component_mut::<Craft>(v).map(|c| c.skill = c.skill.min(0.5));
    }
    g.heirloom_tick(60);
    // the block is the heirloom - and the block's whole worth is the run:
    // sellable sheets on the shop's shelf, and a name for the road
    assert_eq!(g.famed_printer, Some(maker), "the run makes the name");
    let sheets = g.world.get_component::<Building>(shop)
        .map(|b| b.stock.get("prints").copied().unwrap_or(0)).unwrap_or(0);
    assert!(sheets >= 10, "a run of sellable prints off the block: {}", sheets);
    assert!(g.log.iter().any(|l| l.contains("the block keeps giving")),
        "the run's logged: {:?}", g.log.last());
}

#[test]
fn test_the_dead_pass_their_things_on() {
    let mut g = Game::new();
    let owner = g.villagers[0];
    let item = g.world.create_entity();
    g.world.add_component(item, villager::Heirloom {
        maker: owner, owner, name: "a lute", born_tick: 10, stories: vec![] });
    g.heirlooms.push(item);
    // someone under the same roof inherits
    let home = household::home_of(&g.world, owner).unwrap();
    let kin = g.world.get_component::<Building>(home).unwrap()
        .occupants.iter().copied().find(|&o| o != owner).unwrap();
    g.world.get_component_mut::<Health>(owner).unwrap().condition = 0.0;
    g.arrive(owner, Goal::Wander); // a tick to mark him dead? no - die directly
    g.world.add_component(owner, villager::Dead);
    g.heirloom_tick(11);
    let h = g.world.get_component::<villager::Heirloom>(item).unwrap();
    assert_eq!(h.owner, kin, "a bequest picks kin");
    assert!(!h.stories.is_empty(), "it gathers the story of the passing");
}

#[test]
fn test_who_holds_it_shows_on_the_card() {
    let mut g = Game::new();
    let i = 2;
    let v = g.villagers[i];
    let player = g.seats[0].player;
    // known to the player: a tie, and three pieces revealed
    crate::game::relationships::record_interaction(&mut g.world, player,
        crate::game::relationships::Interaction { other: v,
            kind: crate::game::relationships::InteractionKind::Conversation, intensity: 1.0 });
    let __pe = g.player; for __r in vec![crate::game::desires::Reveal::Problem(crate::game::desires::Problem::Broke),
                       crate::game::desires::Reveal::Want(Want::Joy),
                       crate::game::desires::Reveal::Want(Want::GoodFood)] { crate::game::desires::note(&mut g.world, __pe, v, __r); }
    // and two heirlooms in his hands - both of them named
    for name in ["a lute", "a fine toolset"] {
        let item = g.world.create_entity();
        g.world.add_component(item, villager::Heirloom {
            maker: v, owner: v, name, born_tick: 10,
            stories: vec!["played at the Harvest Home".into()] });
        g.heirlooms.push(item);
    }
    g.seats[0].selected_target = i;
    g.seats[0].picked = true;
    let m = g.ui_model_at(0);
    let ins = m.inspector.expect("no card for him");
    // two in hand shows as one name and the count, inside one line
    assert!(ins.lines.iter().any(|l| l.text.contains("a lute +1")),
        "the card doesn't say what he holds: {:?}",
        ins.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}

#[test]
fn test_the_printer_lays_down_a_codex() {
    let mut g = Game::new();
    // the surest hands in the village are the printer's this day
    for &v in &g.villagers {
        if let Some(c) = g.world.get_component_mut::<Craft>(v) { c.skill = 0.1; }
    }
    let maker = g.villagers[0];
    { let c = g.world.get_component_mut::<Craft>(maker).unwrap();
      c.vocation = Vocation::Printmaker; c.skill = 0.95; }
    g.heirloom_tick(60);
    assert!(!g.heirlooms.is_empty(), "the master printer lays nothing down");
    let h = g.world.get_component::<villager::Heirloom>(g.heirlooms[0]).unwrap();
    assert_eq!(h.maker, maker);
    assert_eq!(h.name, "a codex");
    assert!(h.stories.iter().any(|s| s.contains("in the margin")),
        "a codex keeps its grotesques in the margin");
}
