//! The illuminated marginalia (#1323): the manuscript frame round the
//! conversation box and the sidebar, the seasonal vine corners, and the
//! fleuron that marks who's speaking.

use game2_core::session::*;
use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::Sprite;
use game2_core::session::model::{ConvoPanelV, TalkRowV};
use crate::talk::*;
use crate::{Art, UiFrame, View};
use game2_core::sim::climate::Season;

fn solid(w: usize, h: usize, rgb: (u8, u8, u8)) -> Sprite {
    Sprite { width: w, height: h,
        pixels: (0..w * h).flat_map(|_| [rgb.0, rgb.1, rgb.2, 255]).collect() }
}

fn stub_frame() -> UiFrame {
    let mut vines: [[Option<Sprite>; 4]; 4] = std::array::from_fn(|_| [None, None, None, None]);
    for (i, rgb) in [(255, 160, 190), (40, 190, 60), (220, 160, 40), (140, 190, 230)].iter().enumerate() {
        vines[i] = [Some(solid(20, 20, *rgb)), Some(solid(20, 20, *rgb)),
            Some(solid(20, 20, *rgb)), Some(solid(20, 20, *rgb))];
    }
    UiFrame {
        corners: [Some(solid(6, 6, (200, 160, 60))), Some(solid(6, 6, (200, 160, 60))),
            Some(solid(6, 6, (200, 160, 60))), Some(solid(6, 6, (200, 160, 60)))],
        run_n: Some(solid(30, 4, (150, 60, 60))),
        run_s: Some(solid(30, 4, (150, 60, 60))),
        tile_e: Some(solid(4, 12, (60, 80, 150))),
        tile_w: Some(solid(4, 12, (60, 80, 150))),
        vines,
        accents: vec![solid(10, 10, (255, 215, 0)), solid(10, 10, (0, 120, 255))],
        sidebar: None,
    }
}

fn panel() -> ConvoPanelV {
    ConvoPanelV {
        title: "TALK - ELIZA".into(),
        rows: vec![
            TalkRowV { name: Some("Eliza".into()),
                text: "good morrow to thee".into(),
                gist: false, you: false, rule: false, who: Some(3), mood: 0 },
            TalkRowV { name: None, text: "she waits on thy word".into(),
                gist: false, you: false, rule: false, who: None, mood: 0 },
        ],
        options: vec!["greet her".into()],
        decides: false, leave: true,
    }
}

fn px(buf: &Buffer, x: i32, y: i32) -> [u8; 4] {
    let i = (y as usize * buf.width + x as usize) * 4;
    buf.pixels[i..i + 4].try_into().unwrap()
}

/// The box draws its border pieces: corner blocks and edge runs land in
/// the band round the fill, where an empty frame leaves only its gilt rule
#[test]
fn test_the_conversation_box_carries_a_manuscript_border() {
    let p = panel();
    let mut framed = Buffer::new(W, H);
    draw_convo_panel(&mut framed, &p, (-1, -1), &mut ConvoPin::default(), 30,
        &|_| None, &stub_frame(), Season::Summer);
    let mut bare = Buffer::new(W, H);
    draw_convo_panel(&mut bare, &p, (-1, -1), &mut ConvoPin::default(), 30,
        &|_| None, &UiFrame::default(), Season::Summer);
    // the side tile down the box's left edge: the stub's blue scrollwork
    // where the vine doesn't reach
    assert_eq!(&px(&framed, CONVO_X - 2 + 2, CONVO_Y - 2 + 40)[..3], &[60, 80, 150]);
    // the bare frame leaves its gilt rule in the same band instead
    assert_eq!(&px(&bare, CONVO_X - 2 + 1, CONVO_Y - 2)[..3], &[172, 136, 68]);
    // the top edge run under it: the stub's red scrollwork
    assert_eq!(&px(&framed, CONVO_X - 2 + 30, CONVO_Y - 2 + 2)[..3], &[150, 60, 60]);
}

/// The vine flourishes at the corners swap with the season
#[test]
fn test_the_vine_corners_swap_with_the_season() {
    let p = panel();
    let (mut spring, mut winter) = (Buffer::new(W, H), Buffer::new(W, H));
    draw_convo_panel(&mut spring, &p, (-1, -1), &mut ConvoPin::default(), 30,
        &|_| None, &stub_frame(), Season::Spring);
    draw_convo_panel(&mut winter, &p, (-1, -1), &mut ConvoPin::default(), 30,
        &|_| None, &stub_frame(), Season::Winter);
    // the flourish's top-left corner pixels: spring's pink, winter's ice
    assert_eq!(&px(&spring, CONVO_X - 2 + 4, CONVO_Y - 2 + 4)[..3], &[255, 160, 190]);
    assert_eq!(&px(&winter, CONVO_X - 2 + 4, CONVO_Y - 2 + 4)[..3], &[140, 190, 230]);
}

/// A fleuron beside the title marks who's speaking: the player's star
/// when the last line was yours, a villager's own mark when it was theirs
#[test]
fn test_a_fleuron_marks_who_is_speaking() {
    let f = stub_frame();
    let (ax, ay) = (CONVO_X + CONVO_W - 14 - 7 + 4, CONVO_Y + 5 + 4);
    // last speaker a villager: their mark (1 + who % (len - 1))
    let mut buf = Buffer::new(W, H);
    draw_convo_panel(&mut buf, &panel(), (-1, -1), &mut ConvoPin::default(), 30,
        &|_| None, &f, Season::Summer);
    assert_eq!(&px(&buf, ax, ay)[..3], &[0, 120, 255], "villager 3 keeps the second mark");
    // last line the player's: the star takes the slot instead
    let mut p2 = panel();
    p2.rows.push(TalkRowV { name: Some("You".into()), text: "and to thee".into(),
        gist: false, you: true, rule: false, who: None, mood: 0 });
    let mut buf2 = Buffer::new(W, H);
    draw_convo_panel(&mut buf2, &p2, (-1, -1), &mut ConvoPin::default(), 30,
        &|_| None, &f, Season::Summer);
    assert_eq!(&px(&buf2, ax, ay)[..3], &[255, 215, 0], "your line carries the star");
}

/// The sidebar's frame draws behind its panels: with the real art the
/// border's gold shows inside the panel's margins
#[test]
fn test_the_sidebar_carries_its_manuscript_frame() {
    let mut g = Game::new_seeded(MAP_SEED);
    let art = Art::load_fs("../assets", 0);
    if art.ui_frame.sidebar.is_none() {
        return; // no generated marginalia on this checkout - the gilt rule stands in
    }
    let view = View::new(art);
    while g.tick < 3 { g.update(); }
    let buf = view.render(&g.ui_model(), (-1, -1));
    // the frame band inside the panel's edge: something other than the
    // flat panel fill (32,32,48) drew there
    let band = (0..8).flat_map(|dy| (0..200).map(move |dx| (UI_X as i32 + 6 + dx, 40 + dy)));
    let drawn = band.filter(|&(x, y)| px(&buf, x, y)[..3] != [32, 32, 48]).count();
    assert!(drawn > 50, "the sidebar's frame drew nothing ({} px)", drawn);
}
