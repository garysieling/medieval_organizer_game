//! The probe: what the world hides from its players, exposed for tests.
//!
//! Players see `UiModel`; agents see `AgentObs`. Neither carries the
//! things the sim keeps close - which mushroom is death, whose truth is
//! still unsaid, how strong the org really is under the lord's guess, what
//! anyone secretly wants. This is a SEPARATE API on purpose: agents get
//! exactly what a player gets and no more, and the probe socket (when the
//! server runs one) is a different listener they never connect to.

use serde::{Deserialize, Serialize};
use crate::game::desires::InnerLife;
use crate::game::livelihood::Purse;
use crate::game::villager::{self, Hunger};
use crate::sim::illness::Disease;
use super::Game;

/// One person's hidden disease
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseV {
    /// The patient's name
    pub patient: String,
    /// The disease's name
    pub name: String,
    /// Its stage (Incubating/Moderate/Critical/Recovering)
    pub stage: String,
    pub severity: f32,
    pub contagious: bool,
}

/// One person's inner life, laid bare: wants, grievances, satisfactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerV {
    /// The villager's name
    pub name: String,
    /// The wants they carry
    pub wants: Vec<String>,
    /// The grievances pinned on them (problem, until day)
    pub grievances: Vec<(String, u64)>,
    /// The wants currently met (want, until day)
    pub satisfied: Vec<(String, u64)>,
    /// Their purse, to the penny
    pub gold: f32,
    /// Their hunger, raw
    pub hunger: f32,
    /// Dead or alive
    pub dead: bool,
    /// In the castle's cells
    pub held: bool,
}

/// Everything the probe reports: the frame a test gets that no player
/// ever should
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeView {
    pub tick: u64,
    /// A truth sitting unsaid, if one's ripening: (finder, what they found)
    pub discovery: Option<(String, String)>,
    /// Whether the word's out
    pub disclosure_made: bool,
    /// Which mushroom patches are death-caps (the lookalike test)
    pub poison_mushrooms: Vec<(i32, i32)>,
    /// What every mushroom patch really is (tile, the true kind) - hidden
    /// knowledge, for the tests that check the lookalike traps
    pub mushroom_species: Vec<(i32, i32, String)>,
    /// The lord's read on the org's power - lags the truth
    pub lord_esteem: f32,
    /// The org's hidden strength, unlagged
    pub org_power: f32,
    /// What the castle's still owed for defied writs
    pub repression_debt: u32,
    /// How the village counts
    pub numeracy: String,
    /// The castle's prisoners: (name, the day they were taken)
    pub dungeon: Vec<(String, u64)>,
    /// The ladder of shame: (name, the rung they're on, the day it ends)
    pub sanctioned: Vec<(String, String, u64)>,
    /// Every disease abroad
    pub diseases: Vec<DiseaseV>,
    /// Every villager's inner life
    pub inner: Vec<InnerV>,
    /// Rare events afoot: (id, name, until)
    pub rare_active: Vec<(String, String, Option<u64>)>,
    /// The event schedule: (id, the day it's due)
    pub rare_due: Vec<(String, u64)>,
    /// What's already fired: (id, the day)
    pub rare_fired: Vec<(String, u64)>,
    /// The winter's running long
    pub harsh_winter: bool,
    /// Reckonings that didn't add up - a clerk emerges past three
    pub reckonings: u32,
    /// The herb-wife's tally of cures
    pub cured: u32,
    /// The hermit keeps his cave
    pub hermit_abroad: bool,
    /// The martyr's grave stands
    pub martyr: bool,
    /// What the castle's due next (the writ kind, the day window)
    pub next_writ: u64,
    /// The ink-boiler, pedlar, minstrel and circus: who's afoot and due
    pub visitors: Vec<String>,
    /// How much food the market holds (raw servings)
    pub food_servings: u32,
    /// The village's gold: total across living purses
    pub gold_total: f32,
    /// Days each garden's grown to (home, tile, growth)
    pub gardens: Vec<(i32, i32, f32)>,
    /// Fires burning and ground scorched
    pub burning: Vec<(i32, i32)>,
    pub scorched: Vec<(i32, i32)>,
}

impl Game {
    /// The frame a test gets: all of it, secrets included
    pub fn probe(&self) -> ProbeView {
        let name_of = |e: crate::engine::ecs::Entity| {
            self.villagers.iter().position(|&v| v == e)
                .map(|i| crate::session::vill_name(i).to_string())
                .unwrap_or_else(|| format!("e{}", e))
        };
        let org_power = self.world.get_component::<crate::sim::organized_power::OrganizedPower>(self.org)
            .map(|p| p.capacity).unwrap_or(0.0);
        let diseases: Vec<DiseaseV> = self.world.query::<Disease>().iter()
            .filter_map(|&d| self.world.get_component::<Disease>(d))
            .map(|d| DiseaseV {
                patient: name_of(d.patient), name: d.name.clone(),
                stage: format!("{:?}", d.stage), severity: d.severity, contagious: d.contagious })
            .collect();
        let inner: Vec<InnerV> = self.villagers.iter().map(|&v| {
            let life = self.world.get_component::<InnerLife>(v);
            InnerV {
                name: name_of(v),
                wants: life.map(|l| l.wants.iter().map(|w| format!("{:?}", w)).collect()).unwrap_or_default(),
                grievances: life.map(|l| l.grievances.iter().map(|(p, u)| (format!("{:?}", p), *u)).collect()).unwrap_or_default(),
                satisfied: life.map(|l| l.satisfied.iter().map(|(w, u)| (format!("{:?}", w), *u)).collect()).unwrap_or_default(),
                gold: self.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0),
                hunger: self.world.get_component::<Hunger>(v).map(|h| h.value).unwrap_or(0.0),
                dead: villager::is_dead(&self.world, v),
                held: self.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some(),
            }
        }).collect();
        let mut visitors = vec![];
        if self.minstrel.is_some() { visitors.push("minstrel".to_string()); }
        if self.pedlar.is_some() { visitors.push("pedlar".to_string()); }
        if self.circus.is_some() { visitors.push("circus".to_string()); }
        if self.ink_boiler.is_some() { visitors.push("ink-boiler".to_string()); }
        ProbeView {
            tick: self.tick,
            discovery: self.pending_discovery.map(|(w, f, _)| (name_of(w), f.to_string())),
            disclosure_made: self.disclosure_made,
            poison_mushrooms: self.mush_patches.iter().filter(|p| p.deadly()).map(|p| p.tile).collect(),
            mushroom_species: self.mush_patches.iter()
                .map(|p| (p.tile.0, p.tile.1, p.kind.spec().name.to_string())).collect(),
            lord_esteem: self.lord_esteem,
            org_power,
            repression_debt: self.repression_debt,
            numeracy: format!("{:?}", self.numeracy),
            dungeon: self.dungeon.iter().map(|&(e, d)| (name_of(e), d)).collect(),
            sanctioned: self.sanctioned.iter()
                .map(|&(e, s, u)| (name_of(e), format!("{:?}", s), u)).collect(),
            diseases,
            inner,
            rare_active: self.rare.iter()
                .map(|r| (format!("{:?}", r.id), super::events::spec(r.id).name.to_string(), r.until)).collect(),
            rare_due: self.next_rare.iter()
                .map(|(id, d)| (format!("{:?}", id), *d)).collect(),
            rare_fired: self.rare_fired.iter()
                .map(|(id, d)| (format!("{:?}", id), *d)).collect(),
            harsh_winter: self.harsh_winter,
            reckonings: self.reckonings,
            cured: self.cured,
            hermit_abroad: self.hermit.is_some(),
            martyr: self.martyr_grave.is_some(),
            next_writ: {
                let m = self.tick % 120;
                if m < 80 { self.tick + (80 - m) } else { self.tick + (200 - m) }
            },
            visitors,
            food_servings: self.world.get_component::<villager::FoodSource>(self.market_food)
                .map(|f| f.servings).unwrap_or(0),
            gold_total: self.villagers.iter()
                .filter(|&&v| !villager::is_dead(&self.world, v))
                .filter_map(|&v| self.world.get_component::<Purse>(v).map(|p| p.gold)).sum(),
            gardens: self.gardens.iter().map(|&(_, t, g)| (t.0, t.1, g)).collect(),
            burning: self.fire.burning.keys().copied().collect(),
            scorched: self.fire.scorched.keys().copied().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::illness::Stage;

    /// The probe sees what the frame hides: the unsaid truth, the death-cap,
    /// the org's real strength. The UiModel and the AgentObs carry none of it.
    #[test]
    fn test_the_probe_sees_what_the_frame_hides() {
        let mut g = Game::new();
        // a truth sits unsaid
        g.pending_discovery = Some((g.villagers[0], "the lord's fifth is really a quarter", g.tick));
        // a death-cap among the mushrooms
        if let Some(p) = g.mush_patches.first_mut() { p.kind = crate::game::mycology::MushKind::DeathCap; }
        // a fever nobody's told about
        let d = g.world.create_entity();
        g.world.add_component(d, Disease { patient: g.villagers[1], name: "the sweats".into(),
            stage: Stage::Latent, severity: 0.3, contagious: true });

        let pv = g.probe();
        assert_eq!(pv.discovery.as_ref().map(|(_, f)| f.as_str()), Some("the lord's fifth is really a quarter"));
        assert!(!pv.poison_mushrooms.is_empty() || g.mush_patches.is_empty(),
            "the probe names which cap kills");
        assert!(pv.diseases.iter().any(|d| d.name == "the sweats"));
        assert!(pv.inner.iter().all(|i| !i.name.is_empty()));

        // the player's frame carries none of it
        let ui = serde_json::to_string(&g.ui_model()).unwrap();
        assert!(!ui.contains("the lord's fifth"), "the frame hides the unsaid");
        assert!(!ui.contains("the sweats"), "the frame hides the incubating");
        assert!(!ui.contains("poison"), "the frame hides the death-cap");
        // nor does an agent's observation
        if g.free_villager().is_some() {
            let seat = g.add_agent_seat(g.villagers[0]);
            let obs = serde_json::to_string(&g.agent_obs(seat).unwrap()).unwrap();
            assert!(!obs.contains("the lord's fifth"), "agents see what players see");
            assert!(!obs.contains("the sweats"));
        }
    }
}
