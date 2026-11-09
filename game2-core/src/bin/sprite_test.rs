use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::Sprite;
use game2_core::ui::font::draw_text;
use std::path::Path;

fn main() {
    game2_core::infra::crash::install();
    let mut buf = Buffer::new(400, 300);
    buf.rect(0, 0, 400, 300, 30, 30, 45);

    // Spritesheets are grids of frames - crop a single frame
    let sheet = Sprite::load(Path::new("assets/MinifolksVillagers/MinifolksVillagers/Without Outline/MiniPeasant.png")).unwrap();
    println!("sheet {}x{} -> frames of 32x32", sheet.width, sheet.height);
    let frame = sheet.crop(0, 0, 32, 32);

    let house = Sprite::load(Path::new("assets/Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/house/Wood houses/Wood house/wood_house(1).png")).unwrap();

    house.blit(&mut buf, 60, 120);
    frame.blit_scaled(&mut buf, 200, 140, 3);
    draw_text(&mut buf, 10, 10, "Cropped Frame Test", 255, 255, 255);

    buf.save_bmp("evidence/sprite-crop.bmp").unwrap();
    println!("saved evidence/sprite-crop.bmp");
}
