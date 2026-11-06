use crate::*;
use game2_core::sim::illness::Health;

#[test]
fn test_a_dead_villagers_card_shows_a_gravestone() {
    let mut game = Game::new_seeded(MAP_SEED);
    game.speed = 8; // a couple of months of frames, sampled thinner
    let view = View::new(Art::load_fs("../assets", 0));
    while game.tick < 60 { game.update(); }
    let corvin = game.villagers[3];
    game.world.get_component_mut::<Health>(corvin).unwrap().condition = 0.0;
    let t = game.tick;
    while game.tick < t + 3 { game.update(); }
    game.selected_target = 3;
    let buf = view.render(&game.ui_model(), (-1, -1));
    let (x0, y0) = (UI_X as i32 + 6, 312);
    let mut stone = 0;
    let fig: Vec<(u8, u8, u8)> = view.art.villager_sprites[3].pixels.chunks(4).filter(|c| c[3] > 128).map(|c| (c[0], c[1], c[2])).collect();
    let mut figure = 0;
    for y in y0..y0 + 64 { for x in x0..x0 + 64 {
        let i = (y as usize * W + x as usize) * 4;
        let px = (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]);
        if px == GRAVE_STONE { stone += 1; }
        else if fig.contains(&px) && px != GRAVE_EDGE { figure += 1; }
    }}
    assert!(stone > 200, "no gravestone in the card ({} stone px)", stone);
    assert!(figure < 20, "Corvin's living portrait is still drawn ({} px)", figure);
}
