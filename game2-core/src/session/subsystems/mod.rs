//! The sim's subsystems, one file per family. Each file adds an `impl Game`
//! block: its daily `*_tick`, its per-frame `advance_*`, and the helpers
//! only it uses. `Game::livelihood_tick` (the day) calls the ticks in order.
//! New subsystems go here as their own file, not into session.rs.

mod animals;
mod day;
mod economy;
mod land;
mod meetings;
mod people;
mod rites;
pub(in crate::session) mod cairns;
pub(in crate::session) mod hearth;
pub(in crate::session) mod kids;
pub(in crate::session) mod levy;
pub(in crate::session) mod monuments;
pub(in crate::session) mod post;
pub(in crate::session) mod prison;
pub(in crate::session) mod society;
mod travel;
mod visitors;
mod wood;
