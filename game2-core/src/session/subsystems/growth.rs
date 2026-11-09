//! Settlement growth, first tier: a village that prospers for a season
//! fills its vacant street-fronting plots and re-opens the benches that
//! stand empty; a country town whose treasury runs deep raises another
//! roof on its own ring. One step a month, and the log says why.

use crate::session::*;

/// Consecutive prosperous months before the village reaches for new roofs
const PROSPER_MONTHS: u32 = 3;
/// The prosperity line: coin in the people's purses and grain in the
/// store, each clamped 0..1, coin the heavier share. The founding
/// village starts above it and a recovered one crosses it again - the
/// long middling stretch of a hard year stays under
const PROSPERITY_LINE: f32 = 0.6;
/// Servings that read as a full granary (setup stocks five hundred)
const FULL_STORE: f32 = 500.0;
/// Gold a head that reads as a full purse
const FULL_PURSE: f32 = 50.0;
/// A country site grows a roof when its treasury tops this
const COUNTRY_TREASURY: f32 = 150.0;
/// What a new roof over a country site costs its treasury
const ROOF_COST: f32 = 60.0;

impl Game {
    /// The month's growth turn, after the country's books are done: the
    /// village's own ledger first, then each site's
    pub(in crate::session) fn growth_tick(&mut self, tick: u64) {
        // the month turns a few days after market day, when the week's
        // takings are counted and the store's level is plain
        if tick % 30 != 25 { return; }
        if self.village_prosperity() >= PROSPERITY_LINE {
            self.growth_streak += 1;
        } else {
            self.growth_streak = 0;
        }
        if self.growth_streak >= PROSPER_MONTHS {
            self.village_growth(tick);
        }
        self.country_growth(tick);
    }

    /// How well the village is doing: mean coin in living adults' purses
    /// and the granary's level, each clamped 0..1
    fn village_prosperity(&self) -> f32 {
        let (mut folk, mut coin) = (0usize, 0.0f32);
        for &v in &self.villagers {
            if villager::is_dead(&self.world, v)
                || self.world.get_component::<country::Outlander>(v).is_some()
                || self.world.get_component::<Kid>(v).is_some() { continue; }
            folk += 1;
            coin += self.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0);
        }
        let coin_score = if folk > 0 { (coin / folk as f32 / FULL_PURSE).min(1.0) } else { 0.0 };
        let food_score = self.world.get_component::<villager::FoodSource>(self.market_food)
            .map(|f| (f.servings as f32 / FULL_STORE).min(1.0)).unwrap_or(0.0);
        coin_score * 0.6 + food_score * 0.4
    }

    /// One growth step for the village: a free hand re-opens a derelict
    /// bench if one fits, else a new roof on a vacant street plot
    fn village_growth(&mut self, tick: u64) {
        if self.restaff_derelict(tick) { return; }
        self.raise_roof(tick);
    }

    /// A workplace standing empty takes a hand when a free practitioner
    /// of its trade exists - the bench re-opens without new mortar
    fn restaff_derelict(&mut self, tick: u64) -> bool {
        for i in 0..self.building_entities.len() {
            let e = self.building_entities[i];
            let voc = match self.world.get_component::<Building>(e) {
                Some(b) if household::derelict(b) => match b.kind {
                    BuildingKind::Shop(v) => v,
                    _ => continue,
                },
                _ => continue,
            };
            let hand = self.villagers.iter().copied().find(|&v| {
                !villager::is_dead(&self.world, v)
                    && self.world.get_component::<Kid>(v).is_none()
                    && self.world.get_component::<country::Outlander>(v).is_none()
                    && self.world.get_component::<Craft>(v)
                        .map(|c| c.vocation == voc && c.workplace.is_none()).unwrap_or(false)
            });
            let Some(w) = hand else { continue };
            if let Some(c) = self.world.get_component_mut::<Craft>(w) { c.workplace = Some(e); }
            if let Some(b) = self.world.get_component_mut::<Building>(e) { b.worker = Some(w); }
            let wname = self.villagers.iter().position(|&v| v == w).map(|i| vill_name(i)).unwrap_or("a hand");
            self.log_entity(w, format!("t{}: {} takes up the {} again - a prosperous village needs every hand.",
                tick, wname, vocation::trade_name(voc)));
            return true;
        }
        false
    }

    /// A new roof on the vacant street-fronting plot nearest the square:
    /// a home when heads lack roofs or no bench is wanted, else the
    /// workshop a free hand has been waiting for
    fn raise_roof(&mut self, tick: u64) {
        // town_plots reads the map as it stands: occupied tiles fall out,
        // so the head of the list is the best plot still going begging
        let Some(&(x, y)) = town_plots(&self.map, self.town_wall, self.village).first() else { return };
        let mut homeless = false;
        let mut bench: Option<(Entity, Vocation)> = None;
        for &v in &self.villagers {
            if villager::is_dead(&self.world, v)
                || self.world.get_component::<country::Outlander>(v).is_some()
                || self.world.get_component::<Kid>(v).is_some() { continue; }
            if household::home_of(&self.world, v).is_none() { homeless = true; }
            if bench.is_none() {
                if let Some(c) = self.world.get_component::<Craft>(v) {
                    // the land and water trades work their fields and the
                    // shore, not a street-fronting bench
                    if c.workplace.is_none()
                        && !matches!(c.vocation, Vocation::Farmer | Vocation::Fisher) {
                        let kind = household::kind_for(c.vocation);
                        if matches!(kind, BuildingKind::Shop(_)) { bench = Some((v, c.vocation)); }
                    }
                }
            }
        }
        let (kind, name, hand) = if homeless || bench.is_none() {
            (BuildingKind::Home, "a cottage".to_string(), None)
        } else {
            let (v, voc) = bench.unwrap();
            (BuildingKind::Shop(voc), vocation::trade_name(voc).to_string(), Some(v))
        };
        self.map.build_at(x, y);
        let e = household::spawn_building(&mut self.world, kind, &name, x as f32, y as f32, 20);
        self.building_entities.push(e);
        // the sprite and the flourish: the road reaches out, then the
        // roof rises - the same Works a player's own build gets
        let h = self.growth_rng.below(97) as usize;
        let sprite = match WorkLook::of(kind) {
            Some(look) => BSprite::Work(look, h),
            None => BSprite::House(h),
        };
        self.buildings.push((x as f32, y as f32, sprite));
        let bi = self.buildings.len() - 1;
        let road = self.map.plan_road((x, y));
        self.works.push(Works { building: bi, road, frames: 0 });
        if let Some(w) = hand {
            if let Some(c) = self.world.get_component_mut::<Craft>(w) { c.workplace = Some(e); }
            if let Some(b) = self.world.get_component_mut::<Building>(e) { b.worker = Some(w); }
            let wname = self.villagers.iter().position(|&v| v == w).map(|i| vill_name(i)).unwrap_or("a hand");
            self.log_entity(w, format!("t{}: {} takes up the {} - a prosperous village needs every hand.",
                tick, wname, name));
        }
        self.log.push_at_who(format!("t{}: the village prospers - new roofs rise.", tick),
            Some((x as f32, y as f32)), None);
        self.toasts.push_at((format!("{} rises in the village", name), tick + 5),
            Some((x as f32, y as f32)));
    }

    /// A flush site's own ledger buys another roof: towns and castles
    /// raise a cottage on their ring, paid for from the treasury, on a
    /// staggered month so the country grows a roof at a time
    fn country_growth(&mut self, tick: u64) {
        let month = tick / 30;
        for si in 0..self.sites.len() {
            let (grows, sname, centre, reach, treasury) = {
                let s = &self.sites[si];
                (matches!(s.kind, country::SiteKind::Town | country::SiteKind::Castle),
                    s.name, s.centre, s.radius.max(4), s.treasury)
            };
            if !grows || treasury < COUNTRY_TREASURY { continue; }
            // each site takes its turn on its own month of the season
            if (month + si as u64) % 4 != 0 { continue; }
            let dx = self.growth_rng.below(2 * reach + 1) - reach;
            let dy = self.growth_rng.below(2 * reach + 1) - reach;
            let Some((bx, by)) = self.map.nearest_buildable(centre.0 + dx, centre.1 + dy) else { continue };
            // a roof belongs on the site's own ground, not a neighbour's
            if (bx - centre.0).abs().max((by - centre.1).abs()) > reach + 2 { continue; }
            let e = self.country_building(BuildingKind::Home,
                format!("a cottage at {}", sname), bx, by);
            self.sites[si].homes.push(e);
            self.buildings.push((bx as f32, by as f32,
                BSprite::House(self.growth_rng.below(97) as usize)));
            self.map.connect((bx, by));
            self.sites[si].treasury -= ROOF_COST;
            self.log.push_at_who(format!("t{}: {} prospers - a new roof rises on its ground.", tick, sname),
                Some((bx as f32, by as f32)), None);
        }
    }
}
