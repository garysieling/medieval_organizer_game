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
    g.seats[0].notice_panel = true;
    let m = g.ui_model();
    assert!(m.notice_panel, "the panel didn't open");
    assert!(m.notices.iter().any(|(_, t)| t.contains("gathers")), "the feast didn't post");
}

#[test]
fn test_the_board_opens_only_for_the_seat_that_reads_it() {
    let mut g = Game::new();
    g.add_seat();
    // seat 1's camera centred so the market cross lands at (540, 400)
    let mp = *g.world.get_component::<Pos>(g.market_food).unwrap();
    let cam_x = -((mp.x - mp.y) * TILE_W as f32).round() as i32;
    let cam_y = (460.0 - (mp.x + mp.y) * TILE_H as f32).round() as i32;
    g.click_map_at(1, 540, 400, cam_x, cam_y, 1.0);
    assert!(g.ui_model_at(1).notice_panel, "the reading seat's panel didn't open");
    assert!(!g.ui_model_at(0).notice_panel, "a click by seat 1 opened seat 0's panel");
}

#[test]
fn test_the_board_command_shuts_the_panel() {
    let mut g = Game::new();
    g.seats[0].notice_panel = true;
    g.command(Command::Board(false));
    assert!(!g.seats[0].notice_panel, "the Esc's command didn't shut the panel");
    // a set, not a toggle: closing a closed panel stays closed
    g.command(Command::Board(false));
    assert!(!g.seats[0].notice_panel);
    g.command(Command::Board(true));
    assert!(g.seats[0].notice_panel);
}

#[test]
fn test_a_click_away_lets_the_board_go() {
    let mut g = Game::new();
    let mp = *g.world.get_component::<Pos>(g.market_food).unwrap();
    let cam_x = -((mp.x - mp.y) * TILE_W as f32).round() as i32;
    let cam_y = (460.0 - (mp.x + mp.y) * TILE_H as f32).round() as i32;
    // the board click still toggles - open, then shut on the board itself
    g.click_map_at(0, 540, 400, cam_x, cam_y, 1.0);
    assert!(g.seats[0].notice_panel, "the board click didn't open the panel");
    g.click_map_at(0, 540, 400, cam_x, cam_y, 1.0);
    assert!(!g.seats[0].notice_panel, "the board click didn't toggle shut");
    // open again, then a click away from it lets it go
    g.click_map_at(0, 540, 400, cam_x, cam_y, 1.0);
    g.click_map_at(0, 50, 50, cam_x, cam_y, 1.0);
    assert!(!g.seats[0].notice_panel, "a click away left the panel up");
}
