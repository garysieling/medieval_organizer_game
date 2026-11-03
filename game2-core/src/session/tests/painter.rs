use crate::session::*;

#[test]
fn test_a_hall_gets_its_colors() {
    let mut g = Game::new();
    let painter = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Painter).unwrap_or(false));
    let Some(p) = painter else { panic!("no painter") };
    let shop = g.world.get_component::<Craft>(p).unwrap().workplace.unwrap();
    g.world.get_component_mut::<Building>(shop).unwrap()
        .stock.insert("paint".to_string(), 2);
    g.painter_tick(7);
    assert!(!g.painted.is_empty(), "a hall gets its colors");
    assert!(g.log.iter().any(|l| l.contains("gets its colors")));
}

#[test]
fn test_the_color_fades() {
    let mut g = Game::new();
    g.painted.push((g.building_entities[0], 10));
    g.painter_tick(400);
    assert!(g.painted.is_empty(), "the seasons take the color back");
}
