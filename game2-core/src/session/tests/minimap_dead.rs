use crate::session::*;
use crate::client::*;

#[test]
fn test_the_dead_show_as_graves_on_the_minimap() {
    let mut game = Game::new();
    let view = View::new(Art::load_fs("../assets", 0));
    while game.tick < 60 { game.update(); }
    let corvin = game.villagers[3];
    game.world.get_component_mut::<Health>(corvin).unwrap().condition = 0.0;
    let t = game.tick;
    while game.tick < t + 2 { game.update(); }
    let buf = view.render(&game.ui_model(), (-1, -1));
    let px = |v: Entity| {
        let p = *game.world.get_component::<Pos>(v).unwrap();
        let (x, y) = tile_to_minimap(p.x, p.y, mm_k(game.map.width as f32, game.map.height as f32));
        let i = ((y as i32 - 1) as usize * W + (x as i32 - 1) as usize) * 4;
        (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2])
    };
    assert_eq!(px(corvin), MM_GRAVE, "dead Corvin still shows as a living villager on the minimap");
    assert_eq!(px(game.villagers[0]), MM_VILLAGER);
}
