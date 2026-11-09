//! The plague doctor: when the sickness runs three deep he rides in
//! with his beaked mask and his waxed coat, works the sick households
//! for a fee - mostly a little help, sometimes nothing, sometimes a
//! hurt - and rides home when the village mends.

use crate::session::*;
use crate::sim::illness::Stage;
use crate::game::herbology;

/// The doctor comes when the sickness runs this deep
const CALL_CASES: usize = 3;

impl Game {
    /// The doctor's day: the call when the sickness crosses, the rounds
    /// of the sick, the fees and the hurts, and the road home when the
    /// village mends
    pub(in crate::session) fn doctor_tick(&mut self, tick: u64) {
        // how many show it - not brewing it, not over it
        let cases = self.villagers.iter().copied().filter(|&v|
            !villager::is_dead(&self.world, v)
            && self.world.query::<crate::sim::illness::Disease>().iter().any(|&d|
                self.world.get_component::<crate::sim::illness::Disease>(d)
                    .map(|dis| dis.patient == v
                        && dis.stage != Stage::Latent && dis.stage != Stage::Recovering)
                    .unwrap_or(false))).count();
        // the call: three deep and he rides in
        if self.doctor.is_none() && cases >= CALL_CASES {
            let e = self.world.create_entity();
            let (x, y) = (self.town_wall.0 as f32, self.town_wall.3 as f32 + 2.0);
            self.world.add_component(e, Pos { x, y });
            self.doctor = Some(Doctor { e, rounds: vec![], fee_bank: 0.0 });
            self.log.push(format!("t{}: a plague doctor rides in - beaked mask, waxed coat, and a price on his hands.", tick));
        }
        // or he's done: the village mends and the road takes him back
        if self.doctor.is_some() && cases < CALL_CASES {
            let Some(d) = self.doctor.take() else { return };
            self.world.remove_component::<Pos>(d.e);
            self.log.push(format!("t{}: the sickness passes and the doctor rides home, {:.0} gold the heavier.", tick, d.fee_bank));
            return;
        }
        let Some(d) = self.doctor.as_mut() else { return };
        // his rounds: the sick in roster order, a visit each
        if d.rounds.is_empty() {
            d.rounds = self.villagers.iter().copied().filter(|&v|
                !villager::is_dead(&self.world, v)
                && self.world.query::<crate::sim::illness::Disease>().iter().any(|&dd|
                    self.world.get_component::<crate::sim::illness::Disease>(dd)
                        .map(|dis| dis.patient == v
                            && dis.stage != Stage::Latent && dis.stage != Stage::Recovering)
                        .unwrap_or(false))).collect();
        }
        let Some(patient) = d.rounds.pop() else { return };
        let de = d.e;
        // he stands with them - the stranger's hands for the grief
        if let (Some(a), Some(b)) = (self.world.get_component::<Pos>(de).map(|p| (p.x, p.y)),
            self.world.get_component::<Pos>(patient).map(|p| (p.x, p.y))) {
            if let Some(p) = self.world.get_component_mut::<Pos>(de) { p.x = b.0 + 0.5; p.y = b.1; }
            let _ = a;
        }
        // his hands, and their price: mostly a little help, sometimes
        // nothing, sometimes a hurt worse than the sickness
        let stage = self.world.query::<crate::sim::illness::Disease>().iter()
            .filter_map(|&dd| self.world.get_component::<crate::sim::illness::Disease>(dd))
            .find(|dis| dis.patient == patient
                && dis.stage != Stage::Latent && dis.stage != Stage::Recovering)
            .map(|dis| dis.stage).unwrap_or(Stage::Mild);
        let harm_odds = match stage {
            Stage::Mild => 5, Stage::Moderate => 15,
            Stage::Severe => 25, Stage::Critical => 35,
            _ => 0,
        };
        let roll = self.shroom_rng.below(100);
        if roll < harm_odds {
            // iatrogenic harm: a worse turn and a weaker body
            for dd in self.world.query::<crate::sim::illness::Disease>().iter().copied() {
                let Some(dis) = self.world.get_component_mut::<crate::sim::illness::Disease>(dd) else { continue };
                if dis.patient == patient && dis.stage != Stage::Recovering { dis.severity += 0.1; }
            }
            if let Some(h) = self.world.get_component_mut::<crate::sim::illness::Health>(patient) {
                h.condition = (h.condition - 0.1).max(0.05);
            }
            self.say(patient, "The doctor's hands left him worse.", 4);
            // a miracle that failed is a provocation: it can kindle
            // the green word in a susceptible soul
            self.spark_reformation(tick, "the physician's failure");
        } else {
            let eff = 0.15 + (self.shroom_rng.below(3) as f32) * 0.05;
            if herbology::apply_efficacy(&mut self.world, patient, eff) {
                self.say(patient, "The doctor's hands helped, some.", 4);
            }
        }
        // the fee rides off with him
        let eff2 = 0.15 + (self.shroom_rng.below(3) as f32) * 0.05;
        let fee = (2.0 + eff2 * 8.0).min(8.0);
        let paid = self.world.get_component_mut::<Purse>(patient)
            .and_then(|p| { if p.gold >= fee { p.gold -= fee; Some(fee) } else { None } })
            .unwrap_or(0.0);
        if paid > 0.0 {
            if let Some(d2) = self.doctor.as_mut() { d2.fee_bank += paid; }
        }
    }
}
