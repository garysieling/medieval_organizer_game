use crate::session::*;

#[test]
fn test_the_board_carries_the_writ() {
    let mut g = Game::new();
    g.notices.push(("writ".into(), "a fee of 12 gold".into(), 50));
    let m = g.ui_model();
    assert!(m.notices.iter().any(|(k, t)| k == "writ" && t.contains("12")), "the writ didn't post");
}

#[test]
fn test_a_posting_comes_down_at_its_term() {
    let mut g = Game::new();
    g.notices.push(("writ".into(), "a fee".into(), g.tick + 2));
    g.notices.push(("quest".into(), "a small job".into(), g.tick + 20));
    g.tick += 10;
    let m = g.ui_model();
    assert_eq!(m.notices.len(), 1, "the lapsed posting didn't come down");
    assert_eq!(m.notices[0].0, "quest");
}

#[test]
fn test_the_board_reads_when_you_stand_at_it() {
    let mut g = Game::new();
    g.notices.push(("feast".into(), "the village gathers".into(), g.tick + 3));
    g.notice_panel = true;
    let m = g.ui_model();
    assert!(m.notice_panel, "the panel didn't open");
    assert!(m.notices.iter().any(|(_, t)| t.contains("gathers")), "the feast didn't post");
}
