use crate::*;
use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::Sprite;
use game2_core::game::tilemap::Tile;
use game2_core::session::layout::TILE_H;
use game2_core::session::layout::TILE_W;
use game2_core::sim::climate::Season;

/// Every memorial piece (#703): the monuments ticket (#702) picks which
/// form a monument wears, so these only need to be found, keyed out and
/// standing on their tile at a sane size
fn memorial_pieces<'a>(art: &'a Art) -> Vec<(&'static str, &'a Sprite)> {
    let mut pieces: Vec<(&str, &Sprite)> = vec![
        ("wayside cross", art.memorial_cross.as_ref().unwrap()),
        ("picture stone", art.memorial_picture.as_ref().unwrap()),
        ("obelisk", art.memorial_obelisk.as_ref().unwrap()),
        ("trophy", art.memorial_trophy.as_ref().unwrap()),
        ("column", art.memorial_column.as_ref().unwrap()),
    ];
    pieces.extend(art.memorial_cairns.iter().map(|c| ("cairn", c)));
    pieces
}

#[test]
fn test_memorial_pieces_load_and_draw() {
    let art = Art::load_fs("../assets", 0);
    let pieces = memorial_pieces(&art);
    assert_eq!(pieces.len(), 7, "five pieces plus two cairns");
    for (name, s) in &pieces {
        assert!(s.pixels.chunks(4).any(|p| p[3] > 0), "{name} is blank");
        assert!(s.pixels.chunks(4).any(|p| p[3] == 0), "{name} has no keyed-out background");
        assert!(s.height <= 90, "{name} is {}px tall - taller than the keep's neighbours expect", s.height);
        let mut buf = Buffer::new(160, 200);
        draw_standing(&mut buf, s, 80, 150, 1.0);
        assert!(buf.pixels.chunks(4).any(|p| p[0] > 0 || p[1] > 0 || p[2] > 0),
            "{name} drew nothing");
    }
    // the column is the skyline piece: it must stand over a cottage's roofline
    assert!(art.memorial_column.as_ref().unwrap().height > 70, "column too short to tower");
}

#[test]
fn test_memorials_fall_back_when_the_pack_is_missing() {
    // the generated files absent: the fields stay empty and the caller's
    // placeholder draw (view.rs) stands in, as it did before #703
    let art = Art::load(&|rel: &str| if rel.starts_with("rd/") { None } else { std::fs::read(std::path::Path::new("../assets").join(rel)).ok() }, 0);
    assert!(art.memorial_cross.is_none() && art.memorial_column.is_none());
    assert!(art.memorial_cairns.is_empty());
}

/// A contact sheet for the PR: each piece on a grass tile at zoom 1
#[test]
fn test_memorial_contact_sheet() {
    let art = Art::load_fs("../assets", 0);
    let pieces = memorial_pieces(&art);
    let (cw, ch) = (110, 140);
    let mut buf = Buffer::new(cw * pieces.len() as usize, ch);
    let (r, g, b) = tile_colour(Tile::Grass, Season::Summer);
    for (i, (_, s)) in pieces.iter().enumerate() {
        let cx = cw * i as usize + cw / 2;
        let cy = ch - TILE_H as usize - 10;
        buf.diamond(cx as i32, cy as i32, TILE_W, TILE_H, r, g, b);
        buf.diamond(cx as i32, cy as i32 + TILE_H, TILE_W, 1, r / 2, g / 2, b / 2);
        draw_standing(&mut buf, s, cx as i32, cy as i32, 1.0);
    }
    buf.save_bmp("../evidence/memorials-703.bmp").unwrap();
}
