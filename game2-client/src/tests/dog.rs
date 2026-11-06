use crate::*;
use game2_core::game::pets::Pet;
use game2_core::game::relationships::Relationship;
use game2_core::game::villager::Pos;
use game2_core::sim::illness::Health;

#[test]
fn test_dog_goes_with_its_new_owner_when_aldric_dies() {
    let mut game = Game::new_seeded(MAP_SEED);
    game.speed = 8; // a couple of months of frames, sampled thinner
    let view = View::new(Art::load_fs("../assets", 0));
    while game.tick < 60 { game.update(); }
    let aldric = game.villagers[0];
    // Aldric is close to Eliza, so she's who takes his dog in
    let rel = game.world.create_entity();
    game.world.add_component(rel, Relationship {
        from: aldric, other: game.villagers[1], kind: game2_core::game::relationships::RelKind::Friend, strength: 0.6, interactions: 5,
    });
    game.world.get_component_mut::<Health>(aldric).unwrap().condition = 0.0;
    let t = game.tick;
    while game.tick < t + 3 { game.update(); }
    let owner = game.world.get_component::<Pet>(game.pets[0].0).unwrap().owner.expect("someone should take the dog in");
    assert_ne!(owner, aldric, "the dog should be rehomed");

    // stand them on a clear tile - nothing taller around to hide the
    // dog (the town is built up enough that a fixed spot won't do)
    let clear = (2..game.map.width as i32 - 2).flat_map(|x| (2..game.map.height as i32 - 2).map(move |y| (x, y)))
        .find(|&(x, y)| game.map.passable(x, y)
            && !game.map.trees.iter().any(|&(tx, ty)| (tx - x).abs() <= 1 && (ty - y).abs() <= 1)
            && !game.stones.contains(&(x, y))
            && !game.map.walls.iter().any(|(w, _)| (w.0 - x).abs() <= 1 && (w.1 - y).abs() <= 1)
            && !game.buildings.iter().any(|&(bx, by, _)| (bx.round() as i32 - x).abs() <= 1 && (by.round() as i32 - y).abs() <= 1)
            && x > 12 && x < 30 && y > 12 && y < 28)
        .expect("a clear tile");
    *game.world.get_component_mut::<Pos>(owner).unwrap() = Pos { x: clear.0 as f32, y: clear.1 as f32 };
    let buf = view.render(&game.ui_model(), (-1, -1));
    // the dog's coat: its sprite's commonest colour (whichever art loaded)
    let dog_s = view.art.dog.as_ref().unwrap();
    let mut counts: game2_core::engine::det::HashMap<(u8, u8, u8), usize> = game2_core::engine::det::HashMap::default();
    for p in dog_s.pixels.chunks(4).filter(|p| p[3] > 0) { *counts.entry((p[0], p[1], p[2])).or_default() += 1; }
    let (coat_rgb, coat_n) = counts.into_iter().max_by_key(|&(c, n)| (n, c)).unwrap();
    let need = 20.min(coat_n / 2);
    let dog_at = |v: Entity| {
        let p = *game.world.get_component::<Pos>(v).unwrap();
        let (dx, dy) = iso_to_screen_cam(p.x + 0.8, p.y + 0.5, view.cam_x, view.cam_y, view.zoom);
        let mut coat = 0;
        for y in (dy - 24)..dy { for x in (dx - 12)..(dx + 12) {
            let i = (y as usize * W + x as usize) * 4;
            if (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == coat_rgb { coat += 1; }
        }}
        coat >= need
    };
    assert!(dog_at(owner), "the dog isn't drawn beside its new owner");
    assert!(!dog_at(aldric), "the dog is still drawn at Aldric's grave");
}
