//! KPIs, in two kinds that must not mix (docs/architecture.md):
//! - `public`: computed only from a seat's `UiModel` / an agent's `AgentObs`.
//!   That's what a player or an AI can know about how they're doing.
//! - `secret` (`Game::secret_kpis`): the true state, for tests, balance runs
//!   and dashboards. Never put it in a model or an observation.
//!
//! Engine perf timings are a third thing again: `infra::perf`.

pub mod public;
mod secret;

pub use public::*;
