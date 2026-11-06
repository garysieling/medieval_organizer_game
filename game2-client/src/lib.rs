//! The client: everything that turns a `UiModel` into pixels. It sits beside
//! `session`, not inside it, and uses only `session`'s public API (the model
//! types, `layout`, names) - the step before it becomes its own crate.
//! Nothing in here may reach into `Game` for state: a frame is drawn from the
//! seat's model alone, so a remote client draws what a local one does.
//! (`bench` is the exception: the headless runners drive a `Game` to time it.)

// What the client draws with. Child modules take these via `use super::*`.
use game2_core::session::*;
use game2_core::engine::ecs::Entity;
use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::{Assets, Sprite};
use game2_core::game::pets::Species;
use game2_core::game::tilemap::{Tile, TileMap, WallPiece, NEIGHBOURS, noise};
use game2_core::sim::climate::Season;
use game2_core::sim::precipitation::PrecipKind;
use game2_core::ui::font::{draw_text, draw_text_outlined, draw_text_scaled};
use game2_core::ui::weather_fx;

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
pub(crate) use draw::*;
pub use hud::*;
pub use record::*;
pub use view::*;
