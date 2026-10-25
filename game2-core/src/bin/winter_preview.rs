//! Winter variant of the UI preview - snow-covered map.
use game2_core::engine::ecs::World;
use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::{Sprite, Assets};
use game2_core::ui::font::draw_text;
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Pos};
use game2_core::ui::isometric::render_terrain;

const W: usize = 960;
const H: usize = 640;
const MAP_W: usize = 640;
const TILE_W: i32 = 20;
const TILE_H: i32 = 10;

fn iso(tx: f32, ty: f32) -> (i32, i32) {
    (MAP_W as i32 / 2 + ((tx - ty) * TILE_W as f32) as i32, 100 + ((tx + ty) * TILE_H as f32) as i32)
}

fn main() {
    let mut world = World::new();
    let terrain = Terrain::generate(33, 0.7, 42);
    let assets = Assets::new("assets");

    let mut villagers = vec![];
    for i in 0..5 {
        villagers.push(villager::spawn_villager(&mut world, (i*4+3) as f32, (i%3+6) as f32));
    }

    let winter_tree = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Enviroument/Winter/trees/tree_winter(1).png")).ok();
    let tree_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Enviroument/Spring/trees/tree(1).png")).ok();
    let use_tree = winter_tree.as_ref().or(tree_s.as_ref());
    let snow_house = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/house/Wood houses/Wood house/wood_house(1).png")).ok();

    let mut buf = Buffer::new(W, H);
    buf.rect(0, 0, W as i32, H as i32, 25, 25, 38);

    let board = render_terrain(&terrain, MAP_W, H);
    for y in 0..H { for x in 0..MAP_W {
        let idx = (y * MAP_W + x) * 4;
        if idx + 3 < board.pixels.len() {
            let (r, g, b) = (board.pixels[idx], board.pixels[idx+1], board.pixels[idx+2]);
            if !(r == 0 && g == 0 && b == 0) {
                // winter tint: desaturate + cool
                buf.set(x as i32, y as i32,
                    (r as f32 * 0.85).min(255.0) as u8,
                    (g as f32 * 0.9).min(255.0) as u8,
                    (b as f32 * 1.15).min(255.0) as u8);
            }
        }
    }}

    // buildings
    for (bx, by) in [(5.0f32,5.0f32),(12.0,4.0),(7.0,12.0)] {
        if let Some(h) = &snow_house { let (sx, sy) = iso(bx, by); h.blit_center(&mut buf, sx, sy); }
    }
    for (bx, by) in [(3.0f32,14.0f32),(18.0,5.0),(9.0,16.0)] {
        if let Some(t) = use_tree { let (sx, sy) = iso(bx, by); t.blit_center(&mut buf, sx, sy); }
    }
    for v in villagers.iter() {
        if let Some(p) = world.get_component::<Pos>(*v) {
            let (sx, sy) = iso(p.x, p.y);
            buf.rect(sx - 4, sy - 8, 8, 8, 200, 200, 255);
        }
    }

    // falling snow
    for i in 0..60 {
        let px = (i * 41) % W as i32;
        let py = (i * 29) % H as i32;
        buf.set(px, py, 240, 245, 255);
        buf.set(px+1, py, 240, 245, 255);
    }

    draw_text(&mut buf, 10, 8, "Winter  t150", 255, 255, 255);
    buf.save_bmp("evidence/ui-winter.bmp").unwrap();
    println!("saved evidence/ui-winter.bmp");
}
