//! The woodpile: from a month out from the cold, a thin stack gets a
//! hand out ahead of need - and a bundle come home goes on the stack
//! before the hearth has to ask. A kid's a fine hand for a light load.

use crate::session::*;

/// The first day the wise start stacking: a month before the hearths
/// need the wood
const STACK_FROM: u64 = 240;
/// Below this many bundles a household sends a hand out
const THIN_STACK: u32 = 4;

impl Game {
    /// The woodpile's day: sweep what came home onto the stack, and if
    /// the pile's thin send a free hand for a bundle ahead of the cold
    pub(in crate::session) fn woodpile_tick(&mut self, tick: u64) {
        if tick % 360 < STACK_FROM || tick % 3 != 1 { return; }
        let fuels = crate::game::goods::slot_goods(crate::game::goods::Tag::Fuel);
        for &h in &self.building_entities.clone() {
            let Some(b) = self.world.get_component::<household::Building>(h) else { continue };
            if b.kind != household::BuildingKind::Home { continue; }
            let occupants = b.occupants.clone();
            let hp = self.world.get_component::<Pos>(h).copied();
            // what came home goes on the stack
            for &o in &occupants {
                if villager::is_dead(&self.world, o) { continue; }
                let home = hp.and_then(|hp| self.world.get_component::<Pos>(o)
                    .map(|op| (op.x - hp.x).abs() + (op.y - hp.y).abs() <= 7.0)).unwrap_or(false);
                if !home { continue; }
                for g in &fuels {
                    let n = self.world.get_component::<Satchel>(o)
                        .and_then(|s| s.goods.get(*g).copied()).unwrap_or(0);
                    if n == 0 { continue; }
                    let stacked = self.world.get_component_mut::<household::Building>(h)
                        .map(|b| household::add_stock(b, g, n)).unwrap_or(0);
                    if stacked > 0 {
                        if let Some(s) = self.world.get_component_mut::<Satchel>(o) {
                            *s.goods.get_mut(*g).unwrap() -= stacked;
                        }
                    }
                }
            }
            // and a thin pile sends a hand out ahead of the cold - one
            // at a time, and grown hands only: a kid's bundle is his
            // own errand, not the house's
            let Some(b) = self.world.get_component::<household::Building>(h) else { continue };
            let stacked: u32 = fuels.iter()
                .map(|g| b.stock.get(*g).copied().unwrap_or(0)).sum();
            if stacked >= THIN_STACK { continue; }
            if occupants.iter().any(|&o| self.travels.iter()
                .any(|t| t.who == o && matches!(t.goal, Goal::Gather(..)))) { continue; }
            if let Some(hand) = occupants.iter().copied().find(|&o|
                !villager::is_dead(&self.world, o)
                && self.world.get_component::<Kid>(o).is_none()
                && !crate::game::dungeon::unavailable(&self.world, o)
                && !self.travels.iter().any(|t| t.who == o)
                && !self.seats.iter().any(|s| s.player == o)) {
                if let Some(op) = self.world.get_component::<Pos>(hand).copied() {
                    // a bundle's worth a morning's walk, not a week's
                    if let Some(t) = self.gather_site("bavin", op)
                        .filter(|&(tx, ty)| (tx - op.x as i32).abs() + (ty - op.y as i32).abs() <= 80) {
                        self.send_gather(hand, "bavin", t);
                    }
                }
            }
        }
    }

    /// Ahead of the salting the salt has to come home: through the autumn
    /// a house or fold holding perishables lays in its salt - what came
    /// home goes on the stock, and a salt-short larder sends a hand out
    /// for it while a seller's salt's still on the road
    pub(in crate::session) fn salt_layin_tick(&mut self, tick: u64) {
        // the salting season runs from the Harvest Home's turn to the
        // bee itself - after the slaughter there's nothing left to save
        let day = tick % 365;
        if !(263..314).contains(&day) || tick % 3 != 2 { return; }
        for &h in &self.building_entities.clone() {
            let Some(b) = self.world.get_component::<household::Building>(h) else { continue };
            let perish = b.stock.get("fish").copied().unwrap_or(0)
                + b.stock.get("meat").copied().unwrap_or(0);
            let occupants = b.occupants.clone();
            let worker = b.worker;
            let hp = self.world.get_component::<Pos>(h).copied();
            // what came home goes on the stock: a hand back from the
            // market or the pedlar's pitch puts the salt up
            let hands: Vec<Entity> = occupants.iter().copied().chain(worker).collect();
            for &o in &hands {
                if villager::is_dead(&self.world, o) { continue; }
                let home = hp.and_then(|hp| self.world.get_component::<Pos>(o)
                    .map(|op| (op.x - hp.x).abs() + (op.y - hp.y).abs() <= 7.0)).unwrap_or(false);
                if !home { continue; }
                let n = self.world.get_component::<Satchel>(o)
                    .and_then(|s| s.goods.get("salt").copied()).unwrap_or(0);
                if n == 0 { continue; }
                let stocked = self.world.get_component_mut::<household::Building>(h)
                    .map(|b| household::add_stock(b, "salt", n)).unwrap_or(0);
                if stocked > 0 {
                    if let Some(s) = self.world.get_component_mut::<Satchel>(o) {
                        *s.goods.get_mut("salt").unwrap() -= stocked;
                    }
                }
            }
            // a salt-short larder sends a hand out: enough for the
            // perishables waiting on it, a salt per two of them, and
            // only while somebody's selling salt at all
            if perish == 0 { continue; }
            let salt = self.world.get_component::<household::Building>(h)
                .map(|b| b.stock.get("salt").copied().unwrap_or(0)).unwrap_or(0);
            let need = ((perish + 1) / 2).saturating_sub(salt).min(4);
            if need == 0 { continue; }
            let hand = hands.iter().copied().find(|&o|
                !villager::is_dead(&self.world, o)
                && self.world.get_component::<Kid>(o).is_none()
                && !crate::game::dungeon::unavailable(&self.world, o)
                && !self.travels.iter().any(|t| t.who == o)
                && !self.seats.iter().any(|s| s.player == o));
            let Some(hand) = hand else { continue };
            let Some(op) = self.world.get_component::<Pos>(hand).copied() else { continue };
            // a household errand, not a pilgrimage: salt's only worth the
            // walk while a seller's within reach of the village
            if let Some(seller) = livelihood::seller_for(&self.world, "salt", op)
                .filter(|&s| !self.boycotted(s)
                    && self.world.get_component::<Pos>(s)
                        .map(|p| (p.x - op.x).abs() + (p.y - op.y).abs() <= 40.0)
                        .unwrap_or(false)) {
                self.send(hand, Goal::BuyFrom(seller, "salt", need));
            }
        }
    }
}
