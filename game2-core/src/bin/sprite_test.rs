use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::Sprite;
use game2_core::ui::font::draw_text;
use std::path::Path;

fn main() {
    let mut buf = Buffer::new(400, 300);
    // dark background
    buf.rect(0, 0, 400, 300, 30, 30, 45);

    let peasant = Sprite::load(Path::new("assets/MinifolksVillagers/MinifolksVillagers/Without Outline/MiniPeasant.png"));
    let house = Sprite::load(Path::new("assets/Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/house/Wood houses/Wood house/wood_house(1).png"));

    match (&peasant, &house) {
        (Ok(p), Ok(h)) => {
            println!("peasant: {}x{} house: {}x{}", p.width, p.height, h.width, h.height);
            h.blit(&mut buf, 100, 100);
            p.blit_scaled(&mut buf, 220, 120, 3); // scale up the tiny villager
            draw_text(&mut buf, 10, 10, "Sprite Test OK", 255, 255, 255);
        }
        (Err(e), _) => println!("peasant load failed: {}", e),
        (_, Err(e)) => println!("house load failed: {}", e),
    }
    buf.save_bmp("evidence/sprite-test.bmp").unwrap();
    println!("saved evidence/sprite-test.bmp");
}
