use crate::*;
use game2_core::game::villager::{is_dead, Pos};

/// A living villager entity
fn living_villager(game: &Game) -> Entity {
    *game
        .villagers
        .iter()
        .find(|&&v| game.world.get_component::<Pos>(v).is_some() && !is_dead(&game.world, v))
        .expect("a living villager")
}

fn person_index(m: &UiModel, e: Entity) -> usize {
    m.people
        .iter()
        .position(|pv| pv.e == e && matches!(pv.kind, PKind::Villager(_)))
        .expect("the villager's figure")
}

#[test]
fn test_a_mounted_knight_rides_as_one_figure() {
    // the mounted flag swaps the walker for the horse-and-rider sprite,
    // mirrored when the ride faces left
    let game = Game::new_seeded(MAP_SEED);
    let e = living_villager(&game);
    let p = *game.world.get_component::<Pos>(e).unwrap();
    let mut view = View::new(Art::load_fs("../assets", 0));
    (view.cam_x, view.cam_y) = cam_centred_on(p.x, p.y);
    let mut m = game.ui_model();
    let i = person_index(&m, e);
    m.people[i].mounted = true;
    // face the ride right, then left: the sprite mirrors
    m.poses.retain(|p| p.0 != e);
    m.poses.push((e, 1, true));
    let right = view.render(&m, (-1, -1));
    m.poses.retain(|p| p.0 != e);
    m.poses.push((e, 3, true));
    let left = view.render(&m, (-1, -1));
    let flipped = right
        .pixels
        .iter()
        .zip(left.pixels.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        flipped > 0,
        "the mount never mirrored for the leftward ride"
    );
    m.people[i].mounted = false;
    let plain = view.render(&m, (-1, -1));
    let diff = right
        .pixels
        .iter()
        .zip(plain.pixels.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert!(diff > 0, "the mount never drew");
}

#[test]
fn test_the_mount_sorts_as_one_piece() {
    // the horse-and-rider is one figure: behind the castle keep it hides
    // exactly like the walker did - the taller sprite mustn't peek through
    let mut game = Game::new_seeded(MAP_SEED);
    let e = living_villager(&game);
    // a castle keep - the tallest thing the country raises - with the
    // villager diagonally behind it (the smaller x+y key puts the keep
    // in front)
    let (kx, ky) = *game.map.castle_keeps.first().expect("a castle keep");
    *game.world.get_component_mut::<Pos>(e).unwrap() = Pos {
        x: kx as f32 - 2.0,
        y: ky as f32 - 2.0,
    };
    let mut view = View::new(Art::load_fs("../assets", 0));
    (view.cam_x, view.cam_y) = cam_centred_on(kx as f32 - 2.0, ky as f32 - 2.0);
    let mut m = game.ui_model();
    let i = person_index(&m, e);
    // the generated person still is a different height than the pack
    // figure, so its name anchors elsewhere and lands inside the checked
    // region - pin the pack look, the sort check is about the mount
    m.people[i].person = None;
    m.people[i].mounted = true;
    let mounted = view.render(&m, (-1, -1));
    m.people[i].mounted = false;
    let plain = view.render(&m, (-1, -1));
    // only the figure's own pixels must match: its name rides a different
    // anchor mounted or not, and labels draw over everything anyway
    let (sx, sy) = iso_to_screen_cam(
        kx as f32 - 2.0,
        ky as f32 - 2.0,
        view.cam_x,
        view.cam_y,
        view.zoom,
    );
    let z = |v: f32| (v * view.zoom).round() as i32;
    let (x0, x1) = (
        (sx - z(18.0)).max(0) as usize,
        (sx + z(18.0)).min(W as i32 - 1) as usize,
    );
    let (y0, y1) = (
        (sy - z(42.0)).max(0) as usize,
        (sy - z(2.0)).min(H as i32 - 1) as usize,
    );
    let mut diff = 0;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let px = (y * W as usize + x) * 4;
            if mounted.pixels[px..px + 3] != plain.pixels[px..px + 3] {
                diff += 1;
            }
        }
    }
    assert_eq!(diff, 0, "the mount drew {} px through the keep", diff);
}
