use crate::session::*;

#[test]
fn test_a_weaver_keeps_the_loom() {
    let g = Game::new();
    let w = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Weaver).unwrap_or(false));
    assert!(w.is_some(), "one weaver works the loom");
    let w = w.unwrap();
    let loom = g.world.get_component::<Craft>(w).unwrap().workplace;
    if let Some(l) = loom {
        assert_eq!(g.world.get_component::<Building>(l).unwrap().kind,
            household::BuildingKind::Shop(Vocation::Weaver));
    }
    assert_eq!(vocation::inputs(Vocation::Weaver), &["wool"]);
}

#[test]
fn test_the_loom_turns_wool_to_cloth() {
    let mut g = Game::new();
    let w = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Weaver).unwrap_or(false)).unwrap();
    // a bolt of wool in the satchel, a day's work, cloth out
    if let Some(s) = g.world.get_component_mut::<Satchel>(w) {
        *s.goods.entry("wool".to_string()).or_insert(0) += 2;
    }
    let (made, _) = livelihood::work_day(&mut g.world, w, 10);
    assert!(made > 0.0, "the loom runs");
    let cloth = g.world.get_component::<Satchel>(w)
        .map(|s| s.goods.get("cloth").copied().unwrap_or(0)).unwrap_or(0)
        + g.world.get_component::<Craft>(w).and_then(|c| c.workplace)
            .and_then(|b| g.world.get_component::<Building>(b))
            .map(|b| b.stock.get("cloth").copied().unwrap_or(0)).unwrap_or(0);
    assert!(cloth > 0, "wool in, cloth out");
}
