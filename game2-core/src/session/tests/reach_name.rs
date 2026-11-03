use crate::session::*;

#[test]
fn test_names_are_bright_only_within_reach() {
    let mut game = Game::new();
    let view = View::new(Art::load_fs("../assets", 0));
    // one villager beside the player, one far across the village
    let (px, py) = game.player_pos();
    let (near, far) = (game.villagers[0], game.villagers[4]);
    *game.world.get_component_mut::<Pos>(near).unwrap() = Pos { x: px + 1.5, y: py };
    *game.world.get_component_mut::<Pos>(far).unwrap() = Pos { x: px + 6.0, y: py + 6.0 }; // 8.5 tiles: just out of reach, still on screen
    game.selected_target = 2;
    let buf = view.render(&game.ui_model(), (-1, -1));
    let name_colour = |v: Entity, i: usize| {
        let p = *game.world.get_component::<Pos>(v).unwrap();
        let (sx, sy) = iso_to_screen_cam(p.x, p.y, view.cam_x, view.cam_y, view.zoom);
        let (lx, ly) = name_label_pos(sx, sprite_label_anchor(&view.art.villager_sprites[i], sy, view.zoom), vill_name(i));
        // labels may be nudged up a line or two; look over that band
        let (mut bright, mut dim) = (0, 0);
        for y in (ly - 40).max(0)..ly + GLYPH_H { for x in lx.max(0)..(lx + vill_name(i).len() as i32 * GLYPH_W).min(MAP_W as i32) {
            let k = (y as usize * W + x as usize) * 4;
            match (buf.pixels[k], buf.pixels[k + 1], buf.pixels[k + 2]) {
                c if c == NAME_IN_REACH => bright += 1,
                c if c == NAME_OUT_OF_REACH => dim += 1,
                _ => {}
            }
        }}
        (bright, dim)
    };
    let (b, d) = name_colour(near, 0);
    assert!(b > 10 && d == 0, "{} is in reach but not bright ({} bright, {} dim)", vill_name(0), b, d);
    let (_, d) = name_colour(far, 4);
    assert!(d > 10, "{} is out of reach but not dimmed", vill_name(4));
}
