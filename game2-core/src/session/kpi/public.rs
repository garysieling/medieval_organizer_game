//! Public KPIs: what a player or an AI agent could work out about how
//! they're doing, from their own view and nothing else.
//!
//! These functions take a `UiModel` or an `AgentObs`, never the `Game`.
//! That's the guarantee: a public KPI can't read what the seat isn't shown.
//! `tests/layering.rs` fails if this file mentions the sim.

use crate::session::{AgentObs, PKind, UiModel};

/// The gauges a player's own frame supports: the HUD, the panel, the
/// figures on the map and the marks over their heads
pub fn public_kpis(m: &UiModel) -> Vec<(&'static str, f64)> {
    let folk = || m.people.iter().filter(|p| matches!(p.kind, PKind::Villager(_)));
    let standing = folk().filter(|p| !p.dead).count();
    let graves = folk().filter(|p| p.dead).count();
    let marked_hungry = folk().filter(|p| !p.dead && p.hungry).count();
    let marked_sick = folk().filter(|p| !p.dead && p.sick).count();
    vec![
        ("day", m.tick as f64),
        ("you.gold", m.panel.gold as f64),
        ("you.influence", m.panel.influence as f64),
        ("org.members", m.panel.members.len() as f64),
        ("org.issues", m.panel.issues.len() as f64),
        ("org.campaign", if m.panel.campaign.is_some() { 1.0 } else { 0.0 }),
        ("guilds", m.panel.guilds as f64),
        ("market.food", m.panel.food as f64),
        ("folk.seen", standing as f64),
        ("folk.graves", graves as f64),
        ("folk.marked_hungry", marked_hungry as f64),
        ("folk.marked_sick", marked_sick as f64),
        ("fires.seen", m.burning.len() as f64),
    ]
}

/// The gauges an agent's observation supports
pub fn agent_kpis(o: &AgentObs) -> Vec<(&'static str, f64)> {
    vec![
        ("day", o.tick as f64),
        ("you.gold", o.gold as f64),
        ("you.hunger", o.hunger as f64),
        ("you.held", if o.held { 1.0 } else { 0.0 }),
        ("you.orgs", o.orgs.len() as f64),
        ("you.campaigns", o.campaigns.len() as f64),
        ("you.carrying", o.satchel.iter().map(|(_, n)| *n as f64).sum()),
        ("near.folk", o.near.iter().filter(|p| !p.dead).count() as f64),
        ("near.buildings", o.near_buildings.len() as f64),
        ("near.fires", o.burning.len() as f64),
    ]
}

/// KPIs as overlay lines: "you.gold 12.00"
pub fn kpi_lines(kpis: &[(&'static str, f64)]) -> Vec<String> {
    kpis.iter().map(|(k, v)| format!("{} {:.2}", k, v)).collect()
}
