use crate::session::*;
use crate::session::events::EventId;

#[test]
fn test_a_comet_rides_and_everyone_reads_it() {
    let mut g = Game::new();
    g.sky_tick(150);
    assert!(g.sky.is_some(), "a comet rides the walls");
    assert!(g.log.iter().any(|l| l.contains("rides over the walls")));
    let omens = g.bubbles.iter().filter(|b| b.0 != g.player).count();
    assert!(omens > 10, "a crowd of small opinions");
    // the priest reads it his way
    let priest = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false));
    if let Some(p) = priest {
        let said = g.bubbles.iter().find(|b| b.0 == p).map(|b| b.1.as_str());
        assert_eq!(said, Some("A sign. Pray."), "the priest sees warning");
    }
    // and it burns out
    g.sky_tick(160);
    assert!(g.sky.is_none(), "it burns for three nights, no more");
}

#[test]
fn test_every_booked_day_has_its_word() {
    let mut g = Game::new();
    // a sky event five days out - the group omen lands where no
    // event-specific sign exists
    g.next_rare = vec![(EventId::BloodMoon, g.tick + 5)];
    g.events_tick(g.tick);
    assert!(g.log.iter().any(|l| l.contains("the sky's got a strange look")),
        "the booking went silent: {:?}", g.log.last());
    // and the wolf still gets her own sign
    g.next_rare = vec![(EventId::WolfAtFold, g.tick + 5)];
    g.log.clear();
    g.events_tick(g.tick);
    assert!(g.log.iter().any(|l| l.contains("wolf tracks")),
        "the wolf lost her sign: {:?}", g.log.last());
}
