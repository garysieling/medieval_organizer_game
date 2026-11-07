use crate::*;

/// A model carrying just the given toasts, on the given frame
fn model_with(toasts: Vec<ToastV>, frame: u32) -> UiModel {
    let game = Game::new_seeded(MAP_SEED);
    let mut m = game.ui_model();
    m.toasts = toasts;
    m.frame = frame;
    m
}

#[test]
fn test_a_toast_with_a_spot_is_hit_where_it_is_drawn() {
    let view = View::new(Art::load_fs("../assets", 0));
    let m = model_with(vec![
        ToastV { text: "plain banner".into(), at: None },
        ToastV { text: "a fire!".into(), at: Some((3.0, 4.0)) },
    ], 0);
    // the feed stacks a pitch apart from the top: the first banner's box
    // is y 44..64, the second's y 74..94, both centred on the map
    let boxes = view.toast_layout(&m.toasts, m.frame);
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[0].2, 44);
    assert_eq!(boxes[1].2, 74);
    // dead centre of the second banner hits it
    let (_, x, y, w, _) = boxes[1];
    assert_eq!(view.toast_hit(&m, x + w / 2, y + 10), Some(1));
    // the gap between the banners hits nothing
    assert_eq!(view.toast_hit(&m, x + w / 2, 69), None);
    // off to the side of the map hits nothing
    assert_eq!(view.toast_hit(&m, 5, 45), None);
}

#[test]
fn test_a_toast_moves_up_and_fades_out() {
    let view = View::new(Art::load_fs("../assets", 0));
    // a is up first; "b" joins partway through its life
    let mut m = model_with(vec![ToastV { text: "a".into(), at: None }], 0);
    view.toast_layout(&m.toasts, m.frame);
    m.frame = 100;
    m.toasts.push(ToastV { text: "b".into(), at: None });
    let boxes = view.toast_layout(&m.toasts, m.frame);
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[1].2, 74, "the newcomer pops in under the rest");
    // at frame 370 a's life (born 0) is up and b's (born 100) is in its
    // fade: b moves up to the top slot and dims on the way out
    m.frame = 370;
    let boxes = view.toast_layout(&m.toasts, m.frame);
    assert_eq!(boxes.len(), 1, "the oldest has faded out: {:?}", boxes);
    assert_eq!(boxes[0].0, 1, "and it's b left standing");
    assert_eq!(boxes[0].2, 35, "moved up to the top slot, climbed a little");
    assert!(boxes[0].4 < 1.0, "and it fades on the way out: {}", boxes[0].4);
    // and once its life's up it's gone entirely - hit or no hit
    m.frame = 430;
    assert!(view.toast_layout(&m.toasts, m.frame).is_empty());
    assert_eq!(view.toast_hit(&m, 500, 50), None);
}
