//! Secret KPIs: the true state, for tests, balance runs and dashboards.
//! These read hidden things (real hunger, the org's power under the lord's
//! guess) and must never reach a `UiModel` or an `AgentObs`.

use crate::session::*;

impl Game {
    /// The tuning gauges: what the village's innards really read at this
    /// tick - true hunger, the org's real power under the lord's guess, trail
    /// heat, the economy's pulse. Omniscient: tests, balance runs, dashboards
    pub fn secret_kpis(&self) -> Vec<(&'static str, f64)> {
        let alive = self.villagers.iter().filter(|&&v| !villager::is_dead(&self.world, v)).count();
        // hungry = below the sim's own seek-food line (30); a villager between
        // 30-40 is just low on the eat cycle, not in trouble
        let hungry = self.villagers.iter().filter(|&&v|
            self.world.get_component::<Hunger>(v).map(|h| h.value < 30.0).unwrap_or(false)).count();
        // starving = the hard line: a real food crisis, not the bottom of a meal cycle
        let starving = self.villagers.iter().filter(|&&v|
            self.world.get_component::<Hunger>(v).map(|h| h.value <= 10.0).unwrap_or(false)).count();
        let trail_heat: f32 = sorted_sum(self.trails.values().map(|t| t.strength));
        let trail_max = self.trails.values().map(|t| t.strength).fold(0.0f32, f32::max);
        let power = self.world.get_component::<crate::sim::organized_power::OrganizedPower>(self.org)
            .map(|p| p.capacity).unwrap_or(0.0);
        let grain = self.world.get_component::<Building>(self.market_food)
            .map(|b| b.stock.get("grain").copied().unwrap_or(0)).unwrap_or(0) as f64;
        let bread = self.world.get_component::<Building>(self.market_food)
            .map(|b| b.stock.get("bread").copied().unwrap_or(0)).unwrap_or(0) as f64;
        let food = self.world.get_component::<villager::FoodSource>(self.market_food)
            .map(|f| f.servings).unwrap_or(0) as f64;
        // the village's gold: mean purse and the top/mean spread - the
        // economy's pulse while prices sit on a fixed table
        let purses: Vec<f32> = self.villagers.iter()
            .filter(|&&v| !villager::is_dead(&self.world, v))
            .filter_map(|&v| self.world.get_component::<Purse>(v).map(|p| p.gold))
            .collect();
        let gold_mean = if purses.is_empty() { 0.0 } else { purses.iter().sum::<f32>() / purses.len() as f32 };
        let gold_top = purses.iter().copied().fold(0.0f32, f32::max);
        let org_members = self.world.get_component::<Organization>(self.org)
            .map(|o| o.members.len()).unwrap_or(0);
        let sanctioned = self.sanctioned.len();
        let tree_cover = sorted_sum(self.land.tree_cover.values().copied()) / self.land.tree_cover.len().max(1) as f32;
        vec![
            ("villagers.alive", alive as f64),
            ("villagers.hungry", hungry as f64),
            ("villagers.starving", starving as f64),
            ("econ.gold_mean", gold_mean as f64),
            ("econ.gold_spread", (gold_top / gold_mean.max(1.0)) as f64),
            ("org.members", org_members as f64),
            ("org.power", power as f64),
            ("org.lord_esteem", self.lord_esteem as f64),
            ("sanctioned", sanctioned as f64),
            ("trails.heat", trail_heat as f64),
            ("trails.max", trail_max as f64),
            ("stock.grain", grain),
            ("stock.bread", bread),
            ("stock.food", food),
            ("env.tree_cover", tree_cover as f64),
            ("price.grain", goods::worth_at(&self.world, "grain") as f64),
            ("price.wood", goods::worth_at(&self.world, "wood") as f64),
            ("price.fish", goods::worth_at(&self.world, "fish") as f64),
            ("winter.harsh", if self.harsh_winter { 1.0 } else { 0.0 }),
        ]
    }

    /// The secret gauges as lines: "org.power 0.40"
    pub fn secret_kpi_lines(&self) -> Vec<String> {
        self.secret_kpis().iter().map(|(k, v)| format!("{} {:.2}", k, v)).collect()
    }
}

/// Sum floats in a fixed order. f32 addition isn't associative, so a sum
/// over a map's values should never depend on how the map happens to be laid out
fn sorted_sum(vals: impl Iterator<Item = f32>) -> f32 {
    let mut v: Vec<f32> = vals.collect();
    v.sort_by(|a, b| a.total_cmp(b));
    v.into_iter().sum()
}
