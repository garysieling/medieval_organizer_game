use crate::*;

/// Count pixels of exactly `rgb` inside the rect around (x, y)
fn count_rgb(buf: &Buffer, x: i32, y: i32, w: i32, h: i32, rgb: (u8, u8, u8)) -> usize {
    let mut n = 0;
    for yy in y.max(0)..(y + h).min(H as i32) {
        for xx in x.max(0)..(x + w).min(W as i32) {
            let i = (yy as usize * W + xx as usize) * 4;
            if (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == rgb { n += 1; }
        }
    }
    n
}

#[test]
fn test_a_villager_with_a_sign_carries_their_board() {
    let game = Game::new_seeded(MAP_SEED);
    let mut m = game.ui_model();
    let (ve, pos) = {
        let v = m.people.iter_mut()
            .find(|p| matches!(p.kind, PKind::Villager(_)) && !p.dead).unwrap();
        v.sign = Some(1);
        (v.e, v.pos)
    };
    let mut view = View::new(Art::load_fs("../assets", 0));
    (view.cam_x, view.cam_y) = camera_on(pos.0, pos.1);
    let buf = view.render(&m, (-1, -1));
    let (sx, sy) = iso_to_screen_cam(pos.0, pos.1, view.cam_x, view.cam_y, view.zoom);
    let z = view.zoom;
    // the placard: pale face, blue print, dark rim - right of the figure's chest
    let (bx, by) = (sx + (11.0 * z).round() as i32, sy - (34.0 * z).round() as i32);
    let (bw, bh) = ((11.0 * z).round() as i32 + 3, (9.0 * z).round() as i32 + 3);
    assert!(count_rgb(&buf, bx - 1, by - 1, bw, bh, (228, 218, 196)) > 15,
        "no placard face at the sign spot");
    assert!(count_rgb(&buf, bx - 1, by - 1, bw, bh, (80, 100, 150)) > 2,
        "no printed line on the placard");
    // the wooden board reads differently: brown face, dark scrawl
    m.people.iter_mut().find(|p| p.e == ve).unwrap().sign = Some(0);
    let buf2 = view.render(&m, (-1, -1));
    assert!(count_rgb(&buf2, bx - 1, by - 1, bw, bh, (165, 115, 65)) > 15,
        "no wooden board at the sign spot");
    assert_eq!(count_rgb(&buf2, bx - 1, by - 1, bw, bh, (228, 218, 196)), 0,
        "the placard is gone with the design change");
}
