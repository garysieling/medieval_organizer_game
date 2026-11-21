//! The paper that proves your standing: the servant's pass (the
//! livret ouvrier's medieval kin). A hired-out hand carries the last
//! master's mark; without it you're masterless - the watch sweeps you
//! first, no lawful master takes you on, and the cause's own work gets
//! harder while you hide. The org's answers are the hidden scribe's
//! forgery, and the cruel one: reporting an informer masterless (#1261).

use crate::engine::ecs::Entity;

/// The lord's or the last master's mark in a worker's book: proof of
/// where you belong. `marked_by` is whoever last signed the line.
#[derive(Debug, Clone, Copy)]
pub struct Pass {
    pub marked_by: Option<Entity>,
    pub since: u64,
    /// Written by a hand that wasn't a master's - holds until checked
    pub forged: bool,
}

/// No mark, no master: the masterless can't take lawful work and the
/// watch reaches for them first. Landed by losing the last mark, or
/// by somebody saying so to the right ears.
#[derive(Debug, Clone, Copy)]
pub struct Masterless;

/// A hired hand's book: does their pass hold them to a master?
/// Held folk organize at half weight - their book's a leash until
/// somebody marks them free.
pub fn held(world: &crate::engine::ecs::World, v: Entity) -> bool {
    world.get_component::<Pass>(v).is_some()
        && world.get_component::<crate::game::vocation::Craft>(v)
            .and_then(|c| c.tasked_by).is_some()
}
