//! Shared test setup for the session's own test modules.

use crate::session::*;

/// Earn a meeting for a test, the way a player would have to: `host`
/// lends a venue at `at` and everyone in `coming` has said they'll show -
/// enough for call_meeting_at to actually call the meeting, same as a
/// real plan built up through asks. The venue sits right at `at` (not
/// wherever `host`'s own home happens to be) so a guest placed near `at`
/// draws no distance penalty on the turnout roll, and the organizer's
/// turnout skill is given a boost so that roll isn't what a test not
/// about turnout itself is left depending on.
pub(crate) fn plan_ready_meeting(game: &mut Game, host: Entity, at: (f32, f32), coming: &[Entity]) {
    let venue = game.world.create_entity();
    game.world.add_component(venue, Pos { x: at.0, y: at.1 });
    game.world.add_component(game.org, crate::game::organizing::MeetingPlan {
        venue: Some((host, venue)), coming: coming.to_vec(), ..Default::default()
    });
    let player = game.player;
    if game.world.get_component::<crate::game::org_skills::Organizing>(player).is_none() {
        game.world.add_component(player, crate::game::org_skills::Organizing::default());
    }
    game.world.get_component_mut::<crate::game::org_skills::Organizing>(player).unwrap().turnout = 2.0;
}
