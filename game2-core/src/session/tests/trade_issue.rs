use crate::session::*;

fn set_voc(g: &mut Game, v: Entity, voc: Vocation) {
    g.world.get_component_mut::<Craft>(v).unwrap().vocation = voc;
}

#[test]
fn test_a_trades_grief_echoes_among_the_trade() {
    let mut g = Game::new();
    let pe = g.player;
    let (a, b, c) = (g.villagers[0], g.villagers[1], g.villagers[2]);
    set_voc(&mut g, a, Vocation::Mason);
    set_voc(&mut g, b, Vocation::Mason);
    set_voc(&mut g, c, Vocation::Mason);
    let r = || desires::Reveal::Problem(desires::Problem::NoInput("stone"));
    assert!(desires::hear(&mut g.world, pe, a, r(), 10).is_none());
    assert!(desires::hear(&mut g.world, pe, b, r(), 10).is_none());
    let issue = desires::hear(&mut g.world, pe, c, r(), 10);
    assert_eq!(issue.as_deref(), Some("the masons' workshops starved of materials"),
        "three masons make one issue, named for the trade");
}

#[test]
fn test_the_same_trouble_across_trades_splinters() {
    let mut g = Game::new();
    let pe = g.player;
    let (a, b, c) = (g.villagers[0], g.villagers[1], g.villagers[2]);
    set_voc(&mut g, a, Vocation::Mason);
    set_voc(&mut g, b, Vocation::Mason);
    set_voc(&mut g, c, Vocation::Baker);
    let r = || desires::Reveal::Problem(desires::Problem::NoInput("stone"));
    desires::hear(&mut g.world, pe, a, r(), 10);
    desires::hear(&mut g.world, pe, b, r(), 10);
    // two masons and a baker say "no materials" - that's two voices
    // of one trade plus one of another, never three of a kind
    assert!(desires::hear(&mut g.world, pe, c, r(), 10).is_none(),
        "a mixed room doesn't converge on a trade's grief");
    assert!(g.world.get_component::<Organizer>(pe).unwrap().issues.is_empty());
}

#[test]
fn test_everybodys_grief_converges_in_a_mixed_room() {
    let mut g = Game::new();
    let pe = g.player;
    let (a, b, c) = (g.villagers[0], g.villagers[1], g.villagers[2]);
    set_voc(&mut g, a, Vocation::Mason);
    set_voc(&mut g, b, Vocation::Baker);
    set_voc(&mut g, c, Vocation::Priest);
    let r = || desires::Reveal::Problem(desires::Problem::Taxed);
    desires::hear(&mut g.world, pe, a, r(), 10);
    desires::hear(&mut g.world, pe, b, r(), 10);
    let issue = desires::hear(&mut g.world, pe, c, r(), 10);
    assert_eq!(issue.as_deref(), Some("the tax collector's cut"),
        "the commons converge whatever the room's mix");
}

#[test]
fn test_kindred_grievances_gather_under_one_platform() {
    let mut g = Game::new();
    let pe = g.player;
    // the notebook holds the taxes, the writ and the fevers - the first
    // two fold into one platform, the third stands alone (#1072)
    desires::found_issue(&mut g.world, pe, "the taxes".into());
    desires::found_issue(&mut g.world, pe, "the bailiff's writ".into());
    desires::found_issue(&mut g.world, pe, "the fevers".into());
    let issues = g.ui_model_at(0).panel.issues;
    // the platform heads the list with its constituents beneath, the
    // thin issue keeps its own line
    assert_eq!(issues.len(), 4, "{:?}", issues);
    assert_eq!(issues[0], "hard times under the lord's hand");
    assert!(issues[1].contains("the tax collector's cut") && issues[2].contains("the bailiff's writ"),
        "the constituents read beneath the platform");
    assert_eq!(issues[3], desires::issue_name("the fevers".into()), "a lone tag keeps its own name");
}
