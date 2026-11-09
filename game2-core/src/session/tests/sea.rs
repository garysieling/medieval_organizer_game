
#[test]
fn test_the_sea_gives_its_own_goods() {
    // the catch's variety and the strand's finds ride the goods table
    for g in ["eel", "prawn", "squid", "lobster", "jet"] {
        assert!(crate::game::goods::spec(g).is_some(), "no spec for {}", g);
    }
    assert!(crate::game::goods::worth("amber") > crate::game::goods::worth("driftwood"),
        "the rare find should outworth the leavings");
}

#[test]
fn test_the_deep_one_is_a_moment_that_speaks_of_the_sea() {
    use crate::session::events::*;
    let s = spec(EventId::SeaMonster);
    assert!(matches!(s.kind, EventKind::Moment), "the deep one's no visitation");
    assert_eq!(s.name, "the deep one");
    assert!(matches!(group(EventId::SeaMonster), Group::Sky));
}
