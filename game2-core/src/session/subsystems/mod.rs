//! The sim's subsystems, one file per family. Each file adds an `impl Game`
//! block: its daily `*_tick`, its per-frame `advance_*`, and the helpers
//! only it uses. `Game::livelihood_tick` (the day) calls the ticks in order.
//! New subsystems go here as their own file, not into session.rs.

mod animals;
mod people;
mod visitors;
mod wood;
