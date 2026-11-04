//! The client: everything that turns a `UiModel` into pixels. It sits beside
//! `session`, not inside it, and uses only `session`'s public API (the model
//! types, `layout`, names) - the step before it becomes its own crate.
//! Nothing in here may reach into `Game` for state: a frame is drawn from the
//! seat's model alone, so a remote client draws what a local one does.
//! (`bench` is the exception: the headless runners drive a `Game` to time it.)

// What the client draws with. Child modules take these via `use super::*`.
use crate::session::*;
use crate::engine::ecs::Entity;
use crate::engine::render::Buffer;
use crate::engine::sprite::{Assets, Sprite};
use crate::game::pets::Species;
use crate::game::player::recruit_influence_needed;
use crate::game::tilemap::{Tile, TileMap, WallPiece, NEIGHBOURS, noise};
use crate::sim::climate::Season;
use crate::sim::precipitation::PrecipKind;
use crate::ui::font::{draw_text, draw_text_outlined, draw_text_scaled};
use crate::ui::weather_fx;

mod art;
mod bench;
mod draw;
mod hud;
pub mod record;
pub mod talk;
#[cfg(test)]
mod tests;
mod view;

pub use art::*;
pub use bench::*;
pub(super) use draw::*;
pub use hud::*;
pub use record::*;
pub use view::*;
