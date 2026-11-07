use crate::*;
use game2_core::game::relationships::{self, Interaction, InteractionKind};
use game2_core::game::villager::Pos;

/// The lit heart red, and only the heart strip paints it on the map
const HEART_RED: (u8, u8, u8) = (232, 92, 112);

fn heart_reds(buf: &Buffer, upto_x: usize) -> usize {
    buf.pixels.chunks(4).enumerate()
        .filter(|(i, px)| *i % W < upto_x && (px[0], px[1], px[2]) == HEART_RED)
        .count()
}

/// Warmth floats over a friend's head: the five-heart strip draws above
/// their name once the seat's bond has hearts to show - and a stranger's
/// name carries none
#[test]
fn hearts_float_over_a_friends_head() {
    let mut game = Game::new_seeded(MAP_SEED);
    let view = View::new(Art::load_fs("../assets", 0));
    let v = game.villagers[0];
    // beside them, so the friend is on screen with the player
    let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: vx + 0.5, y: vy };
    // a stranger's frame: no hearts anywhere on the map
    let buf = view.render(&game.ui_model(), (-1, -1));
    assert_eq!(heart_reds(&buf, UI_X as usize), 0, "a stranger wore hearts");
    // a kindness warms the tie
    let pe = game.player;
    relationships::record_interaction(&mut game.world, pe,
        Interaction { other: v, kind: InteractionKind::Aid, intensity: 1.0 });
    let m = game.ui_model();
    assert!(m.bonds.iter().any(|b| b.0 == v && b.1 > 0.0), "the bond isn't on the model");
    let buf = view.render(&m, (-1, -1));
    assert!(heart_reds(&buf, UI_X as usize) >= 10,
        "no hearts drawn over the friend's head");
}

/// For eyes: a friend's hearts over their head, rendered as
/// --dump-frame would. `HEARTS_FRAME=<path.bmp> cargo test -p
/// game2-client --lib hearts_over_head_frame -- --ignored`
#[test]
#[ignore]
fn hearts_over_head_frame() {
    let Ok(path) = std::env::var("HEARTS_FRAME") else { return };
    let mut game = Game::new_seeded(MAP_SEED);
    let v = game.villagers[0];
    let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
    *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: vx + 0.5, y: vy };
    let pe = game.player;
    for _ in 0..2 {
        relationships::record_interaction(&mut game.world, pe,
            Interaction { other: v, kind: InteractionKind::Aid, intensity: 1.0 });
    }
    let view = View::new(Art::load_fs("../assets", 0));
    view.render(&game.ui_model(), (-1, -1)).save_bmp(&path).unwrap();
}
