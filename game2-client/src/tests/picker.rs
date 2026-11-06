//! The character picker deals the seed's roster as cards.

use crate::*;

#[test]
fn the_picker_deals_the_roster_as_cards() {
    let art = Art::load_fs("../assets", MAP_SEED);
    let cards: Vec<roster::RosterCard> =
        roster::roster(MAP_SEED).iter().map(roster::RosterCard::from).collect();
    assert_eq!(cards.len(), roster::roster(MAP_SEED).len());
    let buf = render_picker(&art, &cards, 0);
    // names, works and figures leave plenty of lit pixels on the plate
    let bg = (15u8, 15u8, 25u8);
    let lit = buf.pixels.chunks_exact(4)
        .filter(|c| (c[0], c[1], c[2]) != bg).count();
    assert!(lit > 2000, "the picker drew almost nothing ({lit} lit px)");
}
