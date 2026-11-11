use crate::session::*;
use crate::session::country::SiteKind;

fn abbey(g: &Game) -> usize {
    g.sites.iter().position(|s| s.sisterhood.is_some()).expect("no order")
}

#[test]
fn test_the_order_keeps_two_tiers() {
    let g = Game::new();
    let si = abbey(&g);
    let o = g.sites[si].sisterhood.as_ref().unwrap();
    assert!(!o.members.is_empty(), "the choir's empty");
    assert!(!o.lay.is_empty(), "the conversi's bench is empty");
    assert!(o.charter.contains("sisterhood"), "the rule's unnamed: {}", o.charter);
}

#[test]
fn test_the_charter_holds_the_roster() {
    let mut g = Game::new();
    let si = abbey(&g);
    // kill off half the house - the order doesn't bleed the way a
    // village does: the charter refills the ranks
    let dead: Vec<Entity> = {
        let o = g.sites[si].sisterhood.as_ref().unwrap();
        o.members.iter().chain(o.lay.iter()).take(3).copied().collect()
    };
    for v in dead { g.world.add_component(v, villager::Dead); }
    g.order_tick(30);
    let o = g.sites[si].sisterhood.as_ref().unwrap();
    assert_eq!(o.members.len() + o.lay.len(), 6, "the rolls didn't refill");
}

#[test]
fn test_lay_labor_keeps_the_books() {
    let mut g = Game::new();
    let si = abbey(&g);
    let lay_n = g.sites[si].sisterhood.as_ref().unwrap().lay.len() as f32;
    let before_t = g.sites[si].treasury;
    let before_s = g.world.get_component::<villager::FoodSource>(g.sites[si].store)
        .map(|fs| fs.servings).unwrap_or(0);
    g.order_tick(30);
    let earned = g.sites[si].treasury - before_t;
    assert!(earned >= lay_n * 3.0, "the grange earned {:.0}, expected {:.0}", earned, lay_n * 3.0);
    let after_s = g.world.get_component::<villager::FoodSource>(g.sites[si].store)
        .map(|fs| fs.servings).unwrap_or(0);
    assert!(after_s >= before_s, "the grange's store didn't grow");
}

#[test]
fn test_a_fat_house_plants_a_daughter() {
    let mut g = Game::new();
    let si = abbey(&g);
    // a free grange within reach - the pick's on site centres
    let fi = g.sites.iter().position(|s| s.kind == SiteKind::Farm && s.mother.is_none())
        .expect("no farm to daughter");
    g.sites[fi].centre = g.sites[si].centre;
    g.sites[si].treasury = 500.0;
    g.order_tick(30);
    assert_eq!(g.sites[fi].mother, Some(si), "the grange never took the rule");
    assert!(g.sites[fi].treasury >= 200.0, "no endowment travelled: {:.0}", g.sites[fi].treasury);
    assert!(g.log.iter().any(|l| l.contains("daughter house")), "no planting logged");
}

#[test]
fn test_a_grange_pays_the_mother() {
    let mut g = Game::new_seeded(5);
    let si = abbey(&g);
    let fi = g.sites.iter().position(|s| s.kind == SiteKind::Farm
        && s.liege.is_some() && s.mother.is_none()).expect("no tributary farm");
    g.sites[fi].mother = Some(si);
    let runner = g.sites[fi].lord.expect("the farm has no head");
    if let Some(p) = g.world.get_component_mut::<Purse>(runner) { p.gold = 200.0; }
    let r = (fi as u64 * 7) % 60;
    g.country_tick(if r == 0 { 60 } else { r });
    let t = g.travels.iter().find(|t| t.who == runner && matches!(t.goal, Goal::Owe(..)))
        .expect("no tribute rode out");
    let Goal::Owe(_, to, _) = t.goal.clone() else { unreachable!() };
    assert_eq!(to, si, "the grange's assessment went to {:?}, not the mother house", to);
}

#[test]
fn test_the_seizure_takes_the_strongboxes() {
    let mut g = Game::new();
    let si = abbey(&g);
    g.sites[si].treasury = 300.0;
    let liege = g.sites[si].liege.expect("the abbey answers to no keep");
    let keep_before = g.sites[liege].treasury;
    g.sites[si].sisterhood.as_mut().unwrap().risk = 0.98;
    g.order_tick(30);
    let o = g.sites[si].sisterhood.as_ref().unwrap();
    let left = g.sites[si].treasury;
    assert!(left < 200.0, "the strongboxes stayed fat: {:.0}", left);
    assert!(g.sites[liege].treasury > keep_before, "the keep took nothing");
    assert!(o.risk < 0.5, "the risk never reset: {:.2}", o.risk);
    assert!(g.log.iter().any(|l| l.contains("strongboxes")), "no seizure logged");
}
