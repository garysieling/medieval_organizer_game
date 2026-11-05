use crate::*;

#[test]
fn test_a_toast_with_a_spot_is_hit_where_it_is_drawn() {
    let toasts = vec![
        ToastV { text: "plain banner".into(), at: None },
        ToastV { text: "a fire!".into(), at: Some((3.0, 4.0)) },
    ];
    // the first box sits at y 40..60, the second at 66..86, both centred
    let boxes = crate::view::toast_boxes(&toasts);
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[0].1, 40);
    assert_eq!(boxes[1].1, 66);
    // dead centre of the second banner hits it
    let (x, y, w) = boxes[1];
    assert_eq!(toast_hit(&toasts, x + w / 2, y + 10), Some(1));
    // the gap between the banners hits nothing
    assert_eq!(toast_hit(&toasts, x + w / 2, 62), None);
    // off to the side of the map hits nothing
    assert_eq!(toast_hit(&toasts, 5, 45), None);
}
