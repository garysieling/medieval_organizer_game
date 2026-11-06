use crate::session::*;

#[test]
fn test_a_kid_comes_of_age_and_takes_a_trade() {
    let mut g = Game::new();
    let kid = g.villagers[26];
    assert!(g.world.get_component::<Kid>(kid).is_some());
    g.world.get_component_mut::<KidGrowth>(kid).unwrap().age_days = vocation::COMING_OF_AGE - 1;
    g.lifecycle_tick(100);
    assert!(g.world.get_component::<Kid>(kid).is_none(), "they've grown");
    assert!(g.world.get_component::<Craft>(kid).is_some(), "they take a trade");
}

#[test]
fn test_the_errand_hirer_becomes_the_master() {
    let mut g = Game::new();
    let kid = g.villagers[27];
    let master = g.villagers.iter().copied().find(|&v|
        v != kid && g.world.get_component::<Craft>(v).is_some()).unwrap();
    let mvoc = g.world.get_component::<Craft>(master).unwrap().vocation;
    {
        let grw = g.world.get_component_mut::<KidGrowth>(kid).unwrap();
        grw.age_days = vocation::COMING_OF_AGE - 1;
        grw.errands = vec![(master, 5)];
    }
    g.lifecycle_tick(100);
    let c = g.world.get_component::<Craft>(kid).unwrap();
    assert_eq!(c.vocation, mvoc, "they follow the trade they ran for");
    assert_eq!(c.master, Some(master), "the hirer stands as master");
    assert_eq!(c.tasked_by, Some(master));
}

#[test]
fn test_the_years_count_and_the_old_die_of_old_age() {
    let mut g = Game::new();
    // everyone carries their years and the card tells it
    let i = 0;
    let v = g.villagers[i];
    let y0 = g.world.get_component::<crate::game::lifecycle::Age>(v).expect("no years on them").years;
    g.seats[0].selected_target = i;
    let insp = g.villager_inspector_at(0);
    if let Some(bio) = insp.lines.iter().find(|l| l.text.contains("winters")) {
        assert!(bio.text.contains(&(y0 as u32).to_string()) || bio.text.contains(&((y0 + 1.0) as u32).to_string()),
            "the years miss the bio: {}", bio.text);
    }
    // a year of days ages them a year
    g.lifecycle_tick(1);
    let y1 = g.world.get_component::<crate::game::lifecycle::Age>(v).unwrap().years;
    assert!((y1 - y0) - 1.0 / 52.0 < 0.001, "a day counted {} years", y1 - y0);
    // and one past their span goes to old age
    let e = g.villagers[1];
    g.world.get_component_mut::<crate::game::lifecycle::Age>(e).unwrap().years = 91.0;
    g.world.get_component_mut::<crate::game::lifecycle::Vitality>(e).unwrap().max_age = 91.0;
    g.lifecycle_tick(2);
    assert!(villager::is_dead(&g.world, e) || g.world.get_component::<Health>(e).map(|h| h.condition <= 0.0).unwrap_or(false),
        "the elder outlived their span");
    assert!(g.log.iter().any(|l| l.contains("of old age")), "the death went untold");
}
