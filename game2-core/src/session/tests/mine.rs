use crate::session::*;

fn mine_site(g: &Game) -> Option<usize> {
    g.sites.iter().position(|s| s.kind == country::SiteKind::Mine)
}

#[test]
fn test_the_workings_stand_on_the_hill() {
    let g = Game::new();
    let Some(si) = mine_site(&g) else { return }; // no vein in this seed
    let s = &g.sites[si];
    // high ground: above water, out on its own
    assert!(s.centre.0 > 4 && s.centre.1 > 4);
    // a crew of miners keeps it, and a hand carts the ore
    assert!(s.carrier.is_some(), "the pit runs no cart");
    let miners = g.villagers.iter().filter(|&&v|
        g.world.get_component::<country::Outlander>(v).map(|o| o.site == si).unwrap_or(false)
        && g.world.get_component::<Craft>(v).map(|c| c.vocation == vocation::Vocation::Miner).unwrap_or(false)
    ).count();
    assert!(miners >= 2, "a crew of {} can't work a face", miners);
    // the pit's store keeps a sack for the ore it stockpiles
    assert!(g.world.get_component::<Satchel>(s.store).is_some(),
        "the pit stockpiles nothing");
}

#[test]
fn test_the_face_yields_ore() {
    let mut g = Game::new();
    let Some(si) = mine_site(&g) else { return };
    for d in 0..5 { g.country_tick(200 + d); }
    let ore = g.world.get_component::<Satchel>(g.sites[si].store)
        .and_then(|s| s.goods.get("ore").copied()).unwrap_or(0);
    assert!(ore >= 5, "the face gave up only {} in a week", ore);
}

#[test]
fn test_the_smith_takes_ore() {
    assert!(vocation::inputs(vocation::Vocation::Toolmaker).contains(&"ore"),
        "the smith has no use for the pit's iron");
}
