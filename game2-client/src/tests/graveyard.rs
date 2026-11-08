use crate::*;
use game2_core::engine::render::Buffer;
use game2_core::session::layout::iso_to_screen_cam;
use game2_core::sim::illness::Health;

/// The churchyard end to end (#673): the dead lie in, the funeral
/// carries them to the plots, the buried wear tombstones on worn earth,
/// and the grave answers the mouse with who it keeps
#[test]
fn test_the_churchyard_draws_its_dead() {
    let mut game = Game::new_seeded(MAP_SEED);
    game.speed = 8;
    // two village deaths on different days
    let (a, b) = (game.villagers[0], game.villagers[1]);
    for v in [a, b] {
        game.world.get_component_mut::<Health>(v).unwrap().condition = 0.0;
    }
    while game.tick < 6 { game.update(); }
    if let Some(h) = game.world.get_component_mut::<Health>(b) { h.condition = 0.0; }
    while game.tick < 30 { game.update(); }
    let m = game.ui_model();
    assert!(!m.graveyard.is_empty(), "no churchyard on the model");
    for (i, v) in [(0usize, a), (1, b)] {
        let pv = m.people.iter().find(|p| p.kind == PKind::Villager(i)).unwrap();
        assert!(pv.dead, "villager {i} isn't dead on the model");
        assert!(pv.buried, "villager {i} never reached the churchyard");
        assert!(pv.epitaph.as_deref().unwrap_or("").contains("died day"),
            "no epitaph for villager {i}");
        let _ = v;
    }
    // camera on the churchyard's first plot
    let (gx, gy) = m.graveyard[0];
    let art = Art::load_fs("../assets", 0);
    assert!(art.tombstones.len() >= 2, "the tombstone pack didn't load");
    let mut view = View::new(art);
    (view.cam_x, view.cam_y) = cam_centred_on(gx as f32, gy as f32);
    view.zoom = 1.0;
    let frame = view.render(&m, (-1, -1));
    // an empty plot is worn earth, not grass - sample the ground layer
    // itself, since sprites and the church legitimately cover centres
    let ground = view.render_ground(&m, 1.0, (view.cam_x, view.cam_y)).buf;
    let want = graveyard_dirt(m.season);
    let taken: std::collections::HashSet<(i32, i32)> = m.people.iter()
        .filter(|p| p.dead).map(|p| (p.pos.0.floor() as i32, p.pos.1.floor() as i32))
        .collect();
    let mut worn = 0;
    let mut total = 0;
    for &(tx, ty) in m.graveyard.iter().rev() {
        if taken.contains(&(tx, ty)) { continue; }
        let (sx, sy) = iso_to_screen_cam(tx as f32, ty as f32, view.cam_x, view.cam_y, 1.0);
        if (0..ground.width as i32).contains(&sx) && (0..ground.height as i32).contains(&sy) {
            total += 1;
            if px(&ground, sx, sy) == want { worn += 1; }
        }
    }
    assert!(total > 0 && worn * 2 >= total,
        "the churchyard's earth isn't worn dirt ({worn}/{total} plots)");
    // and the buried figure's tombstone stands on its plot - the frame
    // with the tombstone pack differs from the one without it
    let art_plain = Art::load(&|rel: &str| {
        if rel.contains("tomb") { None } else {
            std::fs::read(std::path::Path::new("../assets").join(rel)).ok()
        }
    }, 0);
    let mut view_plain = View::new(art_plain);
    (view_plain.cam_x, view_plain.cam_y) = (view.cam_x, view.cam_y);
    view_plain.zoom = 1.0;
    let plain = view_plain.render(&m, (-1, -1));
    // the tombstones must draw ON the graves, not just differ somewhere
    let (mut x0, mut y0, mut x1, mut y1, mut n) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN, 0);
    for y in 0..frame.height {
        for x in 0..frame.width {
            let i = (y * frame.width + x) * 4;
            if frame.pixels[i..i + 3] != plain.pixels[i..i + 3] {
                n += 1;
                x0 = x0.min(x as i32); y0 = y0.min(y as i32);
                x1 = x1.max(x as i32); y1 = y1.max(y as i32);
            }
        }
    }
    assert!(n > 0, "the tombstones never drew");
    // every differing pixel sits by a grave or its name - the camera is
    // on the churchyard, so the diff box should hug the plots
    let (gx, gy) = iso_to_screen_cam(gx as f32, gy as f32, view.cam_x, view.cam_y, 1.0);
    assert!(x0 >= gx as i32 - 220 && x1 <= gx as i32 + 220
        && y0 >= gy as i32 - 160 && y1 <= gy as i32 + 80,
        "tombstone diff escaped the churchyard: ({x0},{y0})-({x1},{y1}) vs grave ({gx},{gy})");
    // hover the first grave: the epitaph card appears
    let first_dead = m.people.iter().find(|p| p.kind == PKind::Villager(0)).unwrap();
    let (hx, hy) = iso_to_screen_cam(first_dead.pos.0, first_dead.pos.1,
        view.cam_x, view.cam_y, 1.0);
    let hovered = view.render(&m, (hx, hy - 8));
    assert!(frames_differ(&frame, &hovered), "the grave's epitaph never showed");
}

/// The lychgate (#674): the churchyard's gate stands on its tile at the
/// plots' entrance, drawn from the generated pack
#[test]
fn test_the_lychgate_stands_at_the_churchyard_gate() {
    let mut game = Game::new_seeded(MAP_SEED);
    game.speed = 8;
    // the graveyard (and its gate) is planned on the first day's check
    while game.tick < 2 { game.update(); }
    let m = game.ui_model();
    let Some((gx, gy)) = m.lychgate else { panic!("no lychgate on the model") };
    assert!(!m.graveyard.contains(&(gx, gy)), "the gate sits on a grave plot");
    let art = Art::load_fs("../assets", 0);
    assert!(art.lychgate.len() >= 2, "the lychgate pack didn't load");
    let (sw, sh) = { let s = pick(&art.lychgate, gx, gy, 55).expect("no lychgate sprite picked");
        (s.width, s.height) };
    let mut view = View::new(art);
    (view.cam_x, view.cam_y) = cam_centred_on(gx as f32, gy as f32);
    view.zoom = 2.0;
    let with = view.render(&m, (-1, -1));
    let art_plain = Art::load(&|rel: &str| {
        if rel.contains("lychgate") { None } else {
            std::fs::read(std::path::Path::new("../assets").join(rel)).ok()
        }
    }, 0);
    let mut view_plain = View::new(art_plain);
    (view_plain.cam_x, view_plain.cam_y) = (view.cam_x, view.cam_y);
    view_plain.zoom = 2.0;
    let without = view_plain.render(&m, (-1, -1));
    // the gate must draw ON its tile: the diff hugs the gate's screen spot
    let (mut x0, mut y0, mut x1, mut y1, mut n) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN, 0);
    for y in 0..with.height {
        for x in 0..with.width {
            let i = (y * with.width + x) * 4;
            if with.pixels[i..i + 3] != without.pixels[i..i + 3] {
                n += 1;
                x0 = x0.min(x as i32); y0 = y0.min(y as i32);
                x1 = x1.max(x as i32); y1 = y1.max(y as i32);
            }
        }
    }
    assert!(n > 0, "the lychgate never drew");
    // the diff stays inside the sprite's drawn rect: draw_standing anchors
    // the sprite's bottom just past the tile centre and lets it tower up
    let (sx, sy) = iso_to_screen_cam(gx as f32, gy as f32, view.cam_x, view.cam_y, 2.0);
    let w = (sw as f32 * 2.0).round() as i32;
    let h = (sh as f32 * 2.0).round() as i32;
    let bottom = sy + ((sw as f32 / 4.0).min(game2_core::session::layout::TILE_H as f32) * 2.0).round() as i32;
    let (dx, dy) = (sx - w / 2, bottom - h);
    assert!(x0 >= dx - 4 && x1 <= dx + w + 4 && y0 >= dy - 4 && y1 <= bottom + 4,
        "lychgate diff escaped its tile: ({x0},{y0})-({x1},{y1}) vs drawn ({dx},{dy})-({},{bottom})",
        dx + w);
}

fn px(b: &Buffer, x: i32, y: i32) -> (u8, u8, u8) {
    let i = (y as usize * b.width + x as usize) * 4;
    (b.pixels[i], b.pixels[i + 1], b.pixels[i + 2])
}

fn frames_differ(a: &Buffer, b: &Buffer) -> bool {
    a.pixels.iter().zip(b.pixels.iter()).any(|(x, y)| x != y)
}
