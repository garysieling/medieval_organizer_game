use crate::*;

/// A rare visitor's `person` slug draws his own figure over the generic
/// courier (#693): a slugged traveller and a plain one must differ, and
/// the slug must resolve to loaded art
#[test]
fn test_a_visitor_draws_his_own_figure() {
    let game = Game::new_seeded(MAP_SEED);
    let mut view = View::new(Art::load_fs("../assets", 0));
    let mut m = game.ui_model();
    let p = game.village;
    m.people.push(PersonV {
        e: u32::MAX, kind: PKind::Traveller, pos: (p.0 as f32, p.1 as f32),
        label: None, dead: false, kid: false, selected: false, in_reach: false,
        hungry: false, sick: false, act_mark: 0, bond: false, look: 0,
        find: String::new(), person: Some("tinker".into()),
        sign: None, mounted: false, epitaph: None, buried: false, stale: 0.0,
    });
    (view.cam_x, view.cam_y) = cam_centred_on(p.0 as f32, p.1 as f32);
    view.zoom = 2.0;
    let slugged = view.render(&m, (-1, -1));
    m.people.last_mut().unwrap().person = None;
    let generic = view.render(&m, (-1, -1));
    let diff = slugged.pixels.iter().zip(generic.pixels.iter())
        .filter(|(a, b)| a != b).count();
    assert!(diff > 0, "the tinker's own sprite never drew");
    // and the slug resolves to real art, not a silent fallback
    let art = Art::load_fs("../assets", 0);
    assert!(art.person("tinker").is_some(), "the tinker sprite never loaded");
}

/// A rare visitor's `person` still is 1x art (~36px) drawn at the
/// villagers' stills' scale - not through draw_character's 2x pack-frame
/// path, which left them twice the height of everyone else (#1153)
#[test]
fn test_a_visitor_draws_at_his_sprite_s_height() {
    let game = Game::new_seeded(MAP_SEED);
    let mut view = View::new(Art::load_fs("../assets", 0));
    let mut m = game.ui_model();
    // the map's his alone: a diff with the empty frame is his figure and
    // nothing else - no walk-frame noise from the crowd
    let p = game.village;
    let (tx, ty) = (p.0 as f32 + 12.0, p.1 as f32 + 12.0);
    m.people.clear();
    m.people.push(PersonV {
        e: u32::MAX, kind: PKind::Traveller, pos: (tx, ty),
        label: None, dead: false, kid: false, selected: false, in_reach: false,
        hungry: false, sick: false, act_mark: 0, bond: false, look: 0,
        find: String::new(), person: Some("refugee-family".into()),
        sign: None, mounted: false, epitaph: None, buried: false, stale: 0.0,
    });
    (view.cam_x, view.cam_y) = cam_centred_on(tx, ty);
    view.zoom = 2.0;
    // the first render draws the ground straight in; later ones blit the
    // compact cache - compare two frames on the same path or the ground
    // pass itself diffs across the whole map
    let _prime = view.render(&m, (-1, -1));
    m.people.clear();
    let without = view.render(&m, (-1, -1));
    m.people.push(PersonV {
        e: u32::MAX, kind: PKind::Traveller, pos: (tx, ty),
        label: None, dead: false, kid: false, selected: false, in_reach: false,
        hungry: false, sick: false, act_mark: 0, bond: false, look: 0,
        find: String::new(), person: Some("refugee-family".into()),
        sign: None, mounted: false, epitaph: None, buried: false, stale: 0.0,
    });
    let with = view.render(&m, (-1, -1));
    let (mut top, mut bot) = (usize::MAX, 0usize);
    for (i, (a, b)) in with.pixels.iter().zip(without.pixels.iter()).enumerate() {
        if a != b {
            let y = (i / 4) / W;
            top = top.min(y); bot = bot.max(y);
        }
    }
    // ~36px of art at zoom 2 is ~72px on screen; the 2x path ran ~144px
    assert!(bot > top && bot - top < 110,
        "a visitor's figure runs {}px on screen - drawn at the pack frame's 2x?", bot - top);
}

/// The travellers' names ride the model (`PersonV.label`) - they should
/// reach the screen, so a click's target reads who he is (#1153)
#[test]
fn test_a_travellers_label_draws() {
    let game = Game::new_seeded(MAP_SEED);
    let mut view = View::new(Art::load_fs("../assets", 0));
    let mut m = game.ui_model();
    let p = game.village;
    let (tx, ty) = (p.0 as f32 + 12.0, p.1 as f32 + 12.0);
    // a still for the figure, so the only change between frames is the
    // label itself - a walk sheet's stride would flap the diff either way
    m.people.clear();
    m.people.push(PersonV {
        e: u32::MAX, kind: PKind::Traveller, pos: (tx, ty),
        label: Some("the tinker".into()), dead: false, kid: false, selected: false, in_reach: false,
        hungry: false, sick: false, act_mark: 0, bond: false, look: 0,
        find: String::new(), person: Some("tinker".into()),
        sign: None, mounted: false, epitaph: None, buried: false, stale: 0.0,
    });
    (view.cam_x, view.cam_y) = cam_centred_on(tx, ty);
    view.zoom = 2.0;
    // as above: compare frames on the same ground path (the first render
    // draws it straight, later ones blit the compact cache)
    let _prime = view.render(&m, (-1, -1));
    m.people.last_mut().unwrap().label = None;
    let bare = view.render(&m, (-1, -1));
    m.people.last_mut().unwrap().label = Some("the tinker".into());
    let named = view.render(&m, (-1, -1));
    let diff = named.pixels.iter().zip(bare.pixels.iter())
        .filter(|(a, b)| a != b).count();
    assert!(diff > 0, "a traveller's label never drew");
}
