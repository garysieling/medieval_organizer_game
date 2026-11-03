use crate::session::*;

#[test]
fn test_a_letter_comes_for_them() {
    let mut g = Game::new();
    let from = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false)).unwrap();
    let to = g.villagers[0];
    g.letters.push((from, to, 10));
    g.letter_tick(15);
    assert!(g.letters.is_empty(), "it arrived");
    assert!(g.log.iter().any(|l| l.contains("a letter comes")), "word travels");
}

#[test]
fn test_the_illiterate_pay_the_priest_a_penny() {
    let mut g = Game::new();
    let priest = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false)).unwrap();
    let reader = g.villagers.iter().copied().find(|&v| v != priest && !g.literate(v)).unwrap();
    g.world.get_component_mut::<Purse>(reader).unwrap().gold = 10.0;
    g.world.get_component_mut::<Purse>(priest).unwrap().gold = 0.0;
    g.letters.push((priest, reader, 10));
    g.letter_tick(15);
    let gold = g.world.get_component::<Purse>(reader).unwrap().gold;
    let priest_gold = g.world.get_component::<Purse>(priest).unwrap().gold;
    assert_eq!(gold, 9.0, "a penny for the reading");
    assert_eq!(priest_gold, 1.0, "and the penny's his");
}

#[test]
fn test_the_dead_dont_say_the_office() {
    let mut g = Game::new();
    let priest = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false)).unwrap();
    let reader = g.villagers.iter().copied().find(|&v| v != priest && !g.literate(v)).unwrap();
    g.world.get_component_mut::<Purse>(reader).unwrap().gold = 10.0;
    g.world.get_component_mut::<Purse>(priest).unwrap().gold = 0.0;
    g.world.add_component(priest, villager::Dead);
    g.letters.push((priest, reader, 10));
    g.letter_tick(15);
    let gold = g.world.get_component::<Purse>(reader).unwrap().gold;
    let priest_gold = g.world.get_component::<Purse>(priest).unwrap().gold;
    assert_eq!(gold, 10.0, "no office read, no penny paid");
    assert_eq!(priest_gold, 0.0, "the dead take no fee");
    assert!(g.log.iter().any(|l| l.contains("a letter comes")), "the letter still lands");
}
