use crate::session::*;

#[test]
fn test_the_woken_sign_on_without_being_convinced() {
    let mut g = Game::new();
    for &v in &g.villagers[..4] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    // a stranger saw the march and felt the cause wake in them - but
    // they don't know the organizer from a hole in the wall
    let stranger = g.villagers[5];
    g.world.add_component(stranger, crate::game::reactions::Sympathized {
        issues: vec![("the taxes".into(), 1)] });
    let before = g.world.get_component::<Organization>(g.org).unwrap().members.len();
    let (pe, oe) = (g.player, g.org);
    let r = crate::game::player::act_recruit(&mut g.world, pe, oe, stranger);
    assert!(r.success, "the woken sign on without a bond: {}", r.description);
    assert_eq!(g.world.get_component::<Organization>(g.org).unwrap().members.len(), before + 1);
    // and the map reads them over their heads
    let m = g.ui_model();
    let pv = m.people.iter().find(|p| p.e == stranger).unwrap();
    assert!(pv.label.as_deref().unwrap_or("").contains("(woke)"), "{:?}", pv.label);
    // and the card reads them too, once they're picked
    g.seats[0].selected_target = 5;
    g.seats[0].picked = true;
    let insp = g.villager_inspector_at(0);
    assert!(insp.lines.iter().any(|l| l.text.contains("woke")), "{:?}", insp.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
}
