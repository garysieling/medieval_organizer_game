use crate::session::*;

#[test]
fn test_a_maker_lays_down_a_masterwork() {
    let mut g = Game::new();
    // a sure pair of hands
    let maker = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c|
            matches!(c.vocation, Vocation::Luthier | Vocation::Glazier | Vocation::Toolmaker
                | Vocation::Weaver | Vocation::Tailor)).unwrap_or(false)).unwrap();
    g.world.get_component_mut::<Craft>(maker).unwrap().skill = 0.9;
    g.heirloom_tick(60);
    assert!(!g.heirlooms.is_empty(), "a thing made to outlast its maker");
    let h = g.world.get_component::<villager::Heirloom>(g.heirlooms[0]).unwrap();
    assert_eq!(h.maker, maker);
    assert_eq!(h.owner, maker, "the maker keeps it first");
}

#[test]
fn test_the_dead_pass_their_things_on() {
    let mut g = Game::new();
    let owner = g.villagers[0];
    let item = g.world.create_entity();
    g.world.add_component(item, villager::Heirloom {
        maker: owner, owner, name: "a lute", born_tick: 10, stories: vec![] });
    g.heirlooms.push(item);
    // someone under the same roof inherits
    let home = household::home_of(&g.world, owner).unwrap();
    let kin = g.world.get_component::<Building>(home).unwrap()
        .occupants.iter().copied().find(|&o| o != owner).unwrap();
    g.world.get_component_mut::<Health>(owner).unwrap().condition = 0.0;
    g.arrive(owner, Goal::Wander); // a tick to mark him dead? no - die directly
    g.world.add_component(owner, villager::Dead);
    g.heirloom_tick(11);
    let h = g.world.get_component::<villager::Heirloom>(item).unwrap();
    assert_eq!(h.owner, kin, "a bequest picks kin");
    assert!(!h.stories.is_empty(), "it gathers the story of the passing");
}
