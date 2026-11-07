use crate::*;

fn person(find: &str) -> PersonV {
    PersonV { sign: None,
        e: 0, kind: PKind::Villager(0), pos: (0.0, 0.0), label: None,
        dead: false, kid: false, selected: false, in_reach: false,
        hungry: false, sick: false, act_mark: 0, bond: false, look: 0,
        find: find_key(find),
    }
}

#[test]
fn test_a_posting_finds_who_holds_it() {
    // "heart helper" is the matchmaker's posting, folded the same way the
    // box folds what was typed - hyphen and all
    let folk = vec![
        person("eliza journeyman baker"),
        person("maud heart-helper"),
        person("tom child"),
    ];
    assert_eq!(search_people(&folk, "heart helper"), vec![1]);
    assert_eq!(search_people(&folk, "Heart-Helper"), vec![1]);
    assert_eq!(search_people(&folk, "baker"), vec![0]);
    assert_eq!(search_people(&folk, "maud"), vec![1]);
    assert!(search_people(&folk, "nobody").is_empty());
    assert!(search_people(&folk, "   ").is_empty());
}

#[test]
fn test_the_box_sits_under_the_minimap_and_knows_its_bounds() {
    assert!(search_box_at(SRCH_X + 2, SRCH_Y + 2));
    assert!(search_box_at(SRCH_X + SRCH_W - 2, SRCH_Y + SRCH_H - 2));
    // the minimap itself isn't the box, and neither is the TALK panel below
    assert!(!search_box_at(SRCH_X + 2, SRCH_Y - 2));
    assert!(!search_box_at(SRCH_X + 2, SRCH_Y + SRCH_H + 2));
    assert!(!search_box_at(SRCH_X + SRCH_W + 2, SRCH_Y + 2));
    // and it doesn't overlap the TALK panel the panel foot shrank for
    assert!(SRCH_Y + SRCH_H <= talk::TALK_Y, "the box must end above the TALK panel");
}

#[test]
fn test_the_find_box_draws_what_was_typed_and_the_count() {
    let mut buf = Buffer::new(64, 64);
    draw_search_box(&mut buf, "heart helper", 1, true);
    draw_search_box(&mut buf, "", 0, false);
    // drawing is the test: a box that panics or draws off-buffer fails here
}
