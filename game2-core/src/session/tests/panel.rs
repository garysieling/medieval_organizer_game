use crate::session::*;

#[test]
fn test_wrap_text_breaks_at_words() {
    let lines = wrap_text("t280: Bram tells Dagny - someone saw the lord's tax collector", 20);
    assert!(lines.iter().all(|l| l.chars().count() <= 20), "{:?}", lines);
    assert_eq!(lines.join(" "), "t280: Bram tells Dagny - someone saw the lord's tax collector");
    assert_eq!(lines[0], "t280: Bram tells");
}

#[test]
fn test_wrap_text_splits_overlong_words() {
    assert_eq!(wrap_text("abcdefghij", 4), vec!["abcd", "efgh", "ij"]);
    assert!(wrap_text("", 10).is_empty());
}

#[test]
fn test_the_panel_shows_the_meeting_plans_yeses() {
    use crate::game::organizing::MeetingPlan;
    let mut g = Game::new();
    // no plan yet: no section
    assert!(g.ui_model().panel.meeting.is_none());
    let (host, coming, spreader) = (g.villagers[1], g.villagers[2], g.villagers[3]);
    let home = crate::game::household::home_of(&g.world, host)
        .unwrap_or_else(|| { let e = g.world.create_entity(); g.world.add_component(e, Pos { x: 16.0, y: 16.0 }); e });
    g.world.add_component(g.org, MeetingPlan {
        venue: Some((host, home)), coming: vec![coming], spreaders: vec![spreader],
        snacks: Some(host), ..Default::default()
    });
    let mp = g.ui_model().panel.meeting.expect("the plan's yeses");
    assert_eq!(mp.coming, vec![vill_name(2)]);
    assert_eq!(mp.spreaders, vec![vill_name(3)]);
    assert!(mp.roles.iter().any(|r| r.contains("house:")), "{:?}", mp.roles);
    assert!(mp.roles.iter().any(|r| r == &format!("snacks: {}", vill_name(1))), "{:?}", mp.roles);
}

#[test]
fn test_the_panel_names_the_org_s_head() {
    let g = Game::new();
    let m = g.ui_model_at(0);
    let org = g.world.get_component::<Organization>(g.seats[0].org).unwrap();
    // seat 0's own org is led by the seat itself - the panel says "you"
    assert_eq!(org.leader, g.seats[0].player);
    assert_eq!(m.panel.leader.as_deref(), Some("you"),
        "the org's head isn't named on the panel");
}
