//! The conversation review tool's character sheet: everything about a
//! villager, the secrets the inspector card holds back included
use crate::session::*;
use crate::game::desires;

/// The sheet carries the name, the traits the talk engine reads, and
/// every want or trouble the villager hasn't told - the inspector shows
/// a stranger none of those
#[test]
fn test_the_review_sheet_tells_what_the_card_holds_back() {
    let g = Game::new_seeded(7);
    let mut any_secret = false;
    for i in 0..g.villagers.len() {
        let v = g.villagers[i];
        let n = g.character_notes(i);
        assert_eq!(n.name, vill_name(i));
        assert_eq!(n.fig, i);
        let traits = g.world.get_component::<crate::game::traits::Personality>(v)
            .map(|p| p.traits.len()).unwrap_or(0);
        assert_eq!(n.traits.len(), traits, "every trait on the sheet, known or not");
        assert_eq!(n.secrets.len(), desires::unrevealed(&g.world, v, g.tick).len(),
            "every kept want and trouble on the sheet");
        any_secret |= !n.secrets.is_empty();
    }
    assert!(any_secret, "a fresh village keeps some secrets");
    // and it goes over the wire to the review page as JSON
    let j = serde_json::to_value(g.character_notes(0)).unwrap();
    assert!(j["secrets"].is_array() && j["traits"].is_array());
}
