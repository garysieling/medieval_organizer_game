use crate::session::*;

#[test]
fn test_the_lords_own_measure_stands_on_the_site() {
    let g = Game::new();
    let castle = g.sites.iter().find(|s| s.kind == country::SiteKind::Castle).unwrap();
    assert!(castle.persona.cruelty > 0.0, "the persona never rolled");
}

#[test]
fn test_the_sheriff_answers_a_poacher_in_the_chase() {
    let mut g = Game::new();
    let ci = g.sites.iter().position(|s| s.kind == country::SiteKind::Castle).unwrap();
    let (cx, cy) = g.sites[ci].centre;
    // put a poacher on a wooded tile inside the chase's ring
    let v = g.villagers[0];
    g.world.get_component_mut::<Pos>(v).map(|p| { p.x = cx as f32 + 2.0; p.y = cy as f32 + 2.0; });
    g.map.trees.push((cx + 2, cy + 2));
    g.world.get_component_mut::<Purse>(v).map(|p| p.gold = 20.0);
    g.sheriff_tick(50);
    let gold = g.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(99.0);
    assert!(gold < 20.0, "the book never answered the poacher");
}
