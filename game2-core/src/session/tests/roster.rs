//! The starting roster: a seed deals the same premade wanderers anywhere,
//! and picking one rewrites the seat's player to the card.
use crate::session::*;
use crate::session::journal::{replay, Input, Journal};
use crate::session::stage;

#[test]
fn a_seed_deals_the_same_roster_anywhere() {
    let a = roster::roster(42);
    let b = roster::roster(42);
    let c = roster::roster(43);
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.name, y.name);
        assert_eq!(x.backstory, y.backstory);
        assert_eq!(x.kid, y.kid);
        assert_eq!(x.look, y.look);
        assert_eq!(x.vocation, y.vocation);
        assert_eq!(x.gold.to_bits(), y.gold.to_bits());
    }
    // a different seed deals a different hand
    assert!(a.iter().zip(c.iter()).any(|(x, y)| x.name != y.name || x.kid != y.kid));
}

#[test]
fn the_roster_offers_adults_and_children() {
    let r = roster::roster(MAP_SEED);
    assert!(r.iter().any(|s| !s.kid), "the roster offers a grown-up");
    assert!(r.iter().any(|s| s.kid), "the roster offers a child");
    // children only wear the young looks; grown-ups spread over the pack
    assert!(r.iter().filter(|s| s.kid).all(|s| s.look < 2));
    let looks: crate::engine::det::HashSet<usize> =
        r.iter().filter(|s| !s.kid).map(|s| s.look).collect();
    assert!(looks.len() > 1, "grown-ups don't all wear one face");
}

#[test]
fn picking_a_card_rewrites_the_player() {
    let mut g = Game::new_seeded(MAP_SEED);
    let seat = 0;
    let cards = roster::roster(g.seed);
    let kid_idx = cards.iter().position(|s| s.kid).unwrap();
    let adult_idx = cards.iter().position(|s| !s.kid).unwrap();
    let e = g.seats[seat].player;
    // the child: no trade, the young sprite, pocket money
    assert!(g.pick_character_at(seat, kid_idx));
    assert!(g.world.get_component::<vocation::Kid>(e).is_some());
    assert!(g.world.get_component::<vocation::Craft>(e).is_none());
    assert_eq!(g.world.get_component::<PlayerCharacter>(e).unwrap().name, cards[kid_idx].name);
    assert!(g.world.get_component::<crate::game::player::Avatar>(e).unwrap().kid);
    assert_eq!(g.world.get_component::<Purse>(e).unwrap().gold.to_bits(), cards[kid_idx].gold.to_bits());
    // a second pick is refused - one character per seat, or the purse
    // would mint itself gold
    assert!(!g.pick_character_at(seat, adult_idx));
    // an adult card on a fresh seat wears a trade, no childhood
    let mut g2 = Game::new_seeded(MAP_SEED);
    assert!(g2.pick_character_at(0, adult_idx));
    let e2 = g2.seats[0].player;
    assert!(g2.world.get_component::<vocation::Kid>(e2).is_none());
    assert!(g2.world.get_component::<vocation::KidGrowth>(e2).is_none());
    assert!(g2.world.get_component::<vocation::Craft>(e2).is_some());
    assert!(!g2.world.get_component::<crate::game::player::Avatar>(e2).unwrap().kid);
    // off the roster or off the seats: refused
    assert!(!g.pick_character_at(seat, 99));
    assert!(!g.pick_character_at(99, 0));
}

/// A picked childhood isn't forever: the seat's kid comes of age on the
/// same day the village's do, takes a trade, and grows up in the figure
/// too
#[test]
fn a_picked_child_comes_of_age() {
    let mut g = Game::new_seeded(MAP_SEED);
    let kid_idx = roster::roster(g.seed).iter().position(|s| s.kid).unwrap();
    assert!(g.pick_character_at(0, kid_idx));
    let e = g.seats[0].player;
    // nearly grown: the next day's lifecycle is the day
    g.world.get_component_mut::<vocation::KidGrowth>(e).unwrap().age_days = vocation::COMING_OF_AGE - 1;
    stage::run_days(&mut g, 2);
    assert!(g.world.get_component::<vocation::Kid>(e).is_none(), "the kid grew up");
    assert!(g.world.get_component::<vocation::KidGrowth>(e).is_none());
    assert!(g.world.get_component::<vocation::Craft>(e).is_some(), "a trade was taken up");
    assert!(!g.world.get_component::<crate::game::player::Avatar>(e).unwrap().kid,
        "and the figure is a grown one");
}

#[test]
fn a_picked_character_survives_the_journal() {
    let mut g = Game::new_seeded(MAP_SEED);
    let mut j = Journal::new(MAP_SEED);
    g.apply_input(&Input::AddSeat);
    j.record(0, Input::AddSeat);
    let idx = roster::roster(g.seed).iter().position(|s| s.kid).unwrap();
    g.apply_input(&Input::PickCharacter { seat: 1, idx });
    j.record(0, Input::PickCharacter { seat: 1, idx });
    let r = replay(&j, 5);
    let name = |g: &Game, seat: usize| {
        g.world.get_component::<PlayerCharacter>(g.seats[seat].player).unwrap().name.clone()
    };
    assert_eq!(name(&g, 1), name(&r, 1));
    assert!(r.world.get_component::<vocation::Kid>(r.seats[1].player).is_some());
}
