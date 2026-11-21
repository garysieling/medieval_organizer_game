use crate::*;
use game2_core::session::layout::iso_to_screen_cam;

/// The village commons (#688): the well and the meeting hall stand on
/// the grass by the market square, drawn from the generated pack; the
/// shops wear their own looks instead of a cottage
#[test]
fn test_the_well_and_hall_draw_by_the_market() {
    let game = Game::new_seeded(MAP_SEED);
    let m = game.ui_model();
    let (Some((wx, wy)), Some((hx, hy))) = (m.well, m.hall) else {
        panic!("no commons props on the model")
    };
    let art = Art::load_fs("../assets", 0);
    assert!(art.well_s.is_some(), "well.png didn't load");
    assert!(art.hall_s.is_some(), "meeting-hall.png didn't load");
    let mut view = View::new(art);
    (view.cam_x, view.cam_y) = cam_centred_on((wx + hx) as f32 / 2.0, (wy + hy) as f32 / 2.0);
    view.zoom = 2.0;
    let with = view.render(&m, (-1, -1));
    // the same frame with the commons art missing: the diff must hug
    // the two props' tiles and nowhere else
    let art_plain = Art::load(&|rel: &str| {
        if rel.contains("well") || rel.contains("meeting-hall") { None } else {
            std::fs::read(std::path::Path::new("../assets").join(rel)).ok()
        }
    }, 0);
    let mut view_plain = View::new(art_plain);
    (view_plain.cam_x, view_plain.cam_y) = (view.cam_x, view.cam_y);
    view_plain.zoom = 2.0;
    let without = view_plain.render(&m, (-1, -1));
    let mut near = 0;
    let mut far = 0;
    for y in 0..with.height {
        for x in 0..with.width {
            let i = (y * with.width + x) * 4;
            if with.pixels[i..i + 3] == without.pixels[i..i + 3] { continue; }
            // a diff pixel counts as "near" when its screen spot sits in
            // either prop's drawn rect (the sprites tower up from the
            // tile's front corner; the hall is the wider of the two)
            let (xi, yi) = (x as i32, y as i32);
            let near_prop = [(wx, wy), (hx, hy)].iter().any(|&(px, py)| {
                let (sx, sy) = iso_to_screen_cam(px as f32, py as f32, view.cam_x, view.cam_y, 2.0);
                (xi - sx).abs() <= 80 && (yi - sy) <= 40 && (yi - sy) >= -140
            });
            if near_prop { near += 1 } else { far += 1 }
        }
    }
    assert!(near > 0, "the commons never drew");
    assert_eq!(far, 0, "the commons diff escaped its tiles");
}

/// The build flourish (#694): a house mid-rise draws the staged sheet
/// (foundation, frame, roof) instead of cropping the finished sprite;
/// without the sheet the crop path still draws, and the last stretch
/// always belongs to the house itself
#[test]
fn test_a_house_mid_rise_draws_the_stage_sheet() {
    let game = Game::new_seeded(MAP_SEED);
    let mut m = game.ui_model();
    let b = m.buildings.iter().find(|b| !b.derelict).copied().expect("a house on the model");
    m.buildings.iter_mut().for_each(|bb| bb.risen = if (bb.x, bb.y) == (b.x, b.y) { 0.4 } else { 1.0 });
    let art = Art::load_fs("../assets", 0);
    assert_eq!(art.constr.len(), 4, "the construction stages sliced");
    let mut view = View::new(art);
    (view.cam_x, view.cam_y) = cam_centred_on(b.x, b.y);
    view.zoom = 2.0;
    let staged = view.render(&m, (-1, -1));
    // no sheet: the flourish falls back to cropping the finished sprite
    let art_plain = Art::load(&|rel: &str| {
        if rel.contains("construction") { None } else {
            std::fs::read(std::path::Path::new("../assets").join(rel)).ok()
        }
    }, 0);
    let mut view_plain = View::new(art_plain);
    (view_plain.cam_x, view_plain.cam_y) = (view.cam_x, view.cam_y);
    view_plain.zoom = 2.0;
    let cropped = view_plain.render(&m, (-1, -1));
    // near the tile the two must differ - the sheet drew; far off the
    // frames must match - nothing else touched
    let (sx, sy) = iso_to_screen_cam(b.x, b.y, view.cam_x, view.cam_y, 2.0);
    let mut near = 0;
    for y in 0..staged.height {
        for x in 0..staged.width {
            let i = (y * staged.width + x) * 4;
            if staged.pixels[i..i + 3] == cropped.pixels[i..i + 3] { continue; }
            // the stage is shorter than the cropped house, so a piece
            // standing behind it can show through - count the tile's own
            // rect as near and bound the rest to the building's block
            if (x as i32 - sx).abs() <= 80 && (y as i32 - sy) <= 40 && (y as i32 - sy) >= -140 { near += 1 }
            else {
                assert!((x as i32 - sx).abs() <= 200 && (y as i32 - sy) <= 80 && (y as i32 - sy) >= -260,
                    "the flourish leaked off the block: diff at {x},{y} vs tile at {sx},{sy}");
            }
        }
    }
    assert!(near > 0, "the stage sheet never drew");
    // the last stretch shows the house rising to its ridge, not a stage
    m.buildings.iter_mut().find(|bb| (bb.x, bb.y) == (b.x, b.y)).unwrap().risen = 0.9;
    let top = view.render(&m, (-1, -1));
    assert!(top.pixels.iter().zip(staged.pixels.iter()).any(|(a, bb)| a != bb),
        "the last stretch still shows a stage, not the house itself");
}

/// The six new shop looks (#688) load their sliced sprites, summer and
/// snowed winter sets alike; the look names match the sim's WorkLook
#[test]
fn test_the_new_shop_looks_load() {
    let art = Art::load_fs("../assets", 0);
    for w in [WorkLook::Tailor, WorkLook::Glazier, WorkLook::Printshop,
              WorkLook::MasonsYard, WorkLook::Luthier, WorkLook::Farmhouse] {
        assert!(!art.works[w as usize].is_empty(), "{:?} look has no art", w);
        assert_eq!(art.works[w as usize].len(), art.works_winter[w as usize].len());
        assert!(art.works_winter[w as usize].iter().all(|s| s.is_some()),
            "{:?} look has no winter set", w);
    }
}

/// The generated chimney wisps (#688): homes nearest the market smoke
/// with the sprite frames when the pack's there, drawn puffs when not
#[test]
fn test_chimney_smoke_uses_the_generated_wisps() {
    let game = Game::new_seeded(MAP_SEED);
    let m = game.ui_model();
    let Some(smoker) = m.buildings.iter().find(|b| b.smoke) else {
        panic!("no smoking cottage on the model")
    };
    let art = Art::load_fs("../assets", 0);
    assert!(!art.smoke.is_empty(), "smoke frames didn't load");
    let mut view = View::new(art);
    (view.cam_x, view.cam_y) = cam_centred_on(smoker.x, smoker.y);
    view.zoom = 2.0;
    let with = view.render(&m, (-1, -1));
    let art_plain = Art::load(&|rel: &str| {
        if rel.contains("smoke") { None } else {
            std::fs::read(std::path::Path::new("../assets").join(rel)).ok()
        }
    }, 0);
    let mut view_plain = View::new(art_plain);
    (view_plain.cam_x, view_plain.cam_y) = (view.cam_x, view.cam_y);
    view_plain.zoom = 2.0;
    let without = view_plain.render(&m, (-1, -1));
    assert!(with.pixels.iter().zip(without.pixels.iter()).any(|(a, b)| a != b),
        "the smoke frames never drew");
}
