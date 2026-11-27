use crate::*;

fn referral_model(referrals: Vec<ReferralV>, mystery: Option<MysteryV>) -> UiModel {
    let game = Game::new_seeded(MAP_SEED);
    let mut m = game.ui_model();
    m.panel.referrals = referrals;
    m.panel.mystery = mystery;
    m
}

fn door(name: &str, target: Option<usize>) -> ReferralV {
    ReferralV { name: name.into(), from: "Willa".into(), target, at: Some((10.0, 20.0)) }
}

#[test]
fn test_only_the_referred_name_is_a_button() {
    let view = View::new(Art::load_fs("../assets", 0));
    let m = referral_model(vec![door("Hob", Some(3))], None);
    // no mystery up: the block opens at y 90, "see Hob - Willa sent" -
    // the name rides after "see " (4 chars) from x UI_X+150
    let x0 = UI_X as i32 + 150 + crate::measure("see ");
    let x1 = x0 + crate::measure("Hob");
    let y = 90 + 2;
    assert_eq!(view.referral_hit(&m, x0 + 2, y), Some(3), "the name answers");
    assert_eq!(view.referral_hit(&m, x1 - 1, y), Some(3), "its last cell too");
    assert_eq!(view.referral_hit(&m, UI_X as i32 + 150, y), None,
        "the rest of the line is inert");
    assert_eq!(view.referral_hit(&m, x0 + 2, 90 + 2 * LINE_H + 2), None,
        "a rowless seat hits nothing");
    assert_eq!(view.referral_hit(&m, 40, y), None, "off the panel hits nothing");
}

#[test]
fn test_the_block_sits_under_the_mystery() {
    let view = View::new(Art::load_fs("../assets", 0));
    // a titled mystery burns its four rows plus the marks' one - the
    // referrals open on the next row, not over it
    let mz = MysteryV {
        title: "who keeps leaving gifts".into(), heard: 3, total: 5,
        dims: vec![],
        asker_name: "Matilda".into(), asker: Some(7), asker_at: None,
    };
    let m = referral_model(vec![door("Hob", Some(3))], Some(mz));
    let x0 = UI_X as i32 + 150 + crate::measure("see ");
    assert_eq!(view.referral_hit(&m, x0 + 2, 90 + 2), None,
        "the mystery's own rows aren't the door's");
    assert_eq!(view.referral_hit(&m, x0 + 2, 90 + 5 * LINE_H + 2), Some(3),
        "the door sits under the marks");
}

#[test]
fn test_a_door_off_the_roll_has_no_button() {
    let view = View::new(Art::load_fs("../assets", 0));
    // the referred one's gone: the line still draws, nothing clicks
    let m = referral_model(vec![door("Hob", None)], None);
    let x0 = UI_X as i32 + 150 + crate::measure("see ");
    assert_eq!(view.referral_hit(&m, x0 + 2, 90 + 2), None);
    // no referrals at all
    let m2 = referral_model(vec![], None);
    assert_eq!(view.referral_hit(&m2, x0 + 2, 90 + 2), None);
}
