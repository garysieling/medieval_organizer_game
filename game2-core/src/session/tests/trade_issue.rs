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
