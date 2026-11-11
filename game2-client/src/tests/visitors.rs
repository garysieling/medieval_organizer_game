use crate::*;

/// A rare visitor's `visit` slug draws his own figure over the generic
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
        find: String::new(), visit: Some("tinker".into()),
        sign: None, mounted: false, epitaph: None, buried: false, stale: 0.0,
    });
    (view.cam_x, view.cam_y) = cam_centred_on(p.0 as f32, p.1 as f32);
    view.zoom = 2.0;
    let slugged = view.render(&m, (-1, -1));
    m.people.last_mut().unwrap().visit = None;
    let generic = view.render(&m, (-1, -1));
    let diff = slugged.pixels.iter().zip(generic.pixels.iter())
        .filter(|(a, b)| a != b).count();
    assert!(diff > 0, "the tinker's own sprite never drew");
    // and the slug resolves to real art, not a silent fallback
    let art = Art::load_fs("../assets", 0);
    assert!(art.visitors.contains_key("tinker"), "the tinker sprite never loaded");
}
