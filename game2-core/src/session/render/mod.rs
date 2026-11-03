//! The client side of the session: everything that turns a `UiModel` into
//! pixels. Nothing in here may reach into `Game` for state - a frame is drawn
//! from the seat's model alone, so a remote client draws what a local one does.
//! (`bench` is the exception: the headless runners drive a `Game` to time it.)

mod art;
mod bench;
mod draw;
mod hud;
mod view;

pub use art::*;
pub use bench::*;
pub(super) use draw::*;
pub use hud::*;
pub use view::*;
