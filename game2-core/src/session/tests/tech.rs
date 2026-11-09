use crate::session::*;

#[test]
fn test_the_print_shop_card_says_what_its_bench_is_learning() {
    let mut g = Game::new();
    // the shop the printer works: the default village may not have
    // built one, so give it one
    let shop = household::spawn_building(&mut g.world, BuildingKind::Shop(Vocation::Printmaker),
        "the print shop", 0.0, 0.0, 10);
    g.building_entities.push(shop);
    // a sprite on the same tile, the way a real building stands: the card
    // finds the entity by where the sprite stands, not by list position
    g.buildings.push((0.0, 0.0, BSprite::House(0)));
    let bi = g.buildings.len() - 1;
    // some progress banked so the card reads a real fraction
    let next = crate::game::tech_tree::next_researchable(&g.tech).unwrap().to_string();
    crate::game::tech_tree::research(&mut g.tech, &next, 2.0);
    let cost = g.tech.iter().find(|t| t.name == next).unwrap().cost;
    g.seats[0].selected_building = Some(bi);
    let insp = g.building_inspector_at(0).unwrap();
    let line = insp.lines.iter().find(|l| l.text.contains("learning"))
        .map(|l| l.text.clone());
    assert_eq!(line.as_deref(), Some(format!("learning: {} (2/{:.0})", next, cost).as_str()), "{:?}",
        insp.lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>());
    // and when the bench works it out, the village hears
    g.tech_tick(g.tick);
    crate::game::tech_tree::research(&mut g.tech, &next, cost);
    g.tech_tick(g.tick + 1);
    assert!(g.tech.iter().any(|t| t.name == next && t.discovered), "{} never discovered", next);
}
