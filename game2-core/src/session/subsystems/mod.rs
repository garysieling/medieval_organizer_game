//! The sim's subsystems, one file per family. Each file adds an `impl Game`
//! block: its daily `*_tick`, its per-frame `advance_*`, and the helpers
//! only it uses. `Game::livelihood_tick` (the day) calls the ticks in order.
//! New subsystems go here as their own file, not into session.rs.

mod animals;
mod assembly;
mod cascade;
mod day;
mod economy;
mod growth;
mod land;
mod meetings;
mod people;
mod prophet;
mod rites;
pub(in crate::session) mod cairns;
pub(in crate::session) mod graveyard;
pub(in crate::session) mod hearth;
pub(in crate::session) mod kids;
pub(in crate::session) mod doctor;
pub(in crate::session) mod laborers;
pub(in crate::session) mod petition;
pub(in crate::session) mod procession;
pub(in crate::session) mod levy;
mod orders;
pub(in crate::session) mod monuments;
pub(in crate::session) mod post;
pub(in crate::session) mod prison;
pub(in crate::session) mod society;
mod community_law;
mod factions;
mod travel;
pub(in crate::session) mod vaccination;
mod visitors;
mod wood;
