//! The cell organizes: the held folk of a castle site run their own
//! meetings behind bars - no errands, no outside callers, just their
//! own business: what to ask of the keep this week, and how to bear
//! the answer.

use crate::session::*;
use crate::game::dungeon::{CellLetter, Demand, Grant};

/// The road between cell and keep: a week each way
const CELL_ROAD_DAYS: u64 = 7;
/// The cell meets on its own day of the week
const CELL_MEET_DAYS: u64 = 7;
/// Slop under this mark breeds the gripes
const SICK_RATIONS: f32 = 0.5;

impl Game {
    /// The cell's day: keep the org honest to who's held, hold its
    /// weekly meeting, walk the letters both ways, and let thin
    /// rations do their quiet work
    pub(in crate::session) fn cell_tick(&mut self, tick: u64) {
        // who each castle holds: villagers taken on a charge go to the
        // nearest castle; an outlander to his own
        let mut held: Vec<(usize, Entity)> = vec![];
        for &v in &self.villagers {
            if self.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none() { continue; }
            // the dead take their leave of the cell too
            if villager::is_dead(&self.world, v) { continue; }
            let si = self.site_of(v)
                .filter(|&si| self.sites[si].kind == country::SiteKind::Castle)
                .unwrap_or_else(|| self.castle_site());
            held.push((si, v));
        }
        // the cell is an honest org: it forms when the first soul lands
        let sites: Vec<usize> = held.iter().map(|(si, _)| *si).collect();
        for &si in sites.iter() {
            if self.cells.iter().all(|c| c.site != si) {
                let members: Vec<Entity> = held.iter()
                    .filter(|(s, _)| *s == si).map(|(_, v)| *v).collect();
                let org = self.world.create_entity();
                self.world.add_component(org, Organization {
                    name: "the cell".into(), members: members.clone(),
                    leader: members[0], influence_pool: 0.0,
                    resources: 0.0, charter: "out".into(), successors: vec![] });
                self.cells.push(crate::game::dungeon::CellOrg {
                    org, site: si, outstanding: None, rations: 0.5 });
            }
        }
        for c in &mut self.cells {
            let mut members: Vec<Entity> = held.iter()
                .filter(|(s, _)| *s == c.site).map(|(_, v)| *v).collect();
            members.sort();
            if let Some(o) = self.world.get_component_mut::<Organization>(c.org) {
                o.members = members;
            }
        }
        // and the letters walk the road in both directions
        let mut arrived: Vec<CellLetter> = vec![];
        self.cell_mail.retain(|l| if l.arrive <= tick { arrived.push(*l); false } else { true });
        for l in arrived {
            match l.grant {
                // the ask reached the keep: the warden makes his answer
                None => {
                    let p = self
                        .world
                        .get_component::<crate::game::traits::Personality>(l.warden)
                        .map(|p| p.traits.clone())
                        .unwrap_or_default();
                    let grant = if p.contains(&crate::game::traits::Trait::Kind) {
                        Grant::Full
                    } else if p.contains(&crate::game::traits::Trait::Hotheaded)
                        || p.contains(&crate::game::traits::Trait::Stingy)
                    {
                        Grant::Refused
                    } else {
                        Grant::Part
                    };
                    self.cell_mail.push(CellLetter {
                        site: l.site,
                        demand: l.demand,
                        grant: Some(grant),
                        warden: l.warden,
                        arrive: tick + CELL_ROAD_DAYS,
                    });
                    self.log.push_ev(
                        EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                        format!(
                            "t{}: word from {} - the keep has read the cell's ask.",
                            tick, self.sites[l.site].name
                        ),
                    );
                }
                // the answer reached the cell: it lands on the held
                Some(g) => self.cell_answered(tick, l.site, l.demand, g),
            }
        }
        // the weekly meeting: no ask still out, the table picks one
        for i in 0..self.cells.len() {
            if tick % CELL_MEET_DAYS != (self.cells[i].site as u64) % CELL_MEET_DAYS { continue; }
            if self.cells[i].outstanding.is_some() { continue; }
            let members = self.world.get_component::<Organization>(self.cells[i].org)
                .map(|o| o.members.clone()).unwrap_or_default();
            if members.is_empty() { continue; }
            // the table's leaning: the sick get seen to first, thin
            // rations get fed, and there's always someone asking out
            let sick = members.iter().any(|&m| self.world.get_component::<villager::Condition>(m).is_some());
            let thin = self.cells[i].rations < SICK_RATIONS;
            let demand = if sick { Demand::Care } else if thin { Demand::Rations } else { Demand::Release };
            let Some(warden) = self.site_knight(self.cells[i].site) else { continue };
            self.cells[i].outstanding = Some(demand);
            self.cell_mail.push(CellLetter { site: self.cells[i].site, demand,
                grant: None, warden, arrive: tick + CELL_ROAD_DAYS });
            let what = match demand {
                Demand::Release => "they want out",
                Demand::Rations => "the slop's a sentence of its own",
                Demand::Care => "the sick need seeing to",
            };
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                format!("t{}: the cell meets - {}.", tick, what),
            );
        }
        // and thin rations do their quiet work
        for c in &self.cells {
            if c.rations >= SICK_RATIONS { continue; }
            let members = self.world.get_component::<Organization>(c.org)
                .map(|o| o.members.clone()).unwrap_or_default();
            for m in members {
                if self.world.get_component::<crate::sim::illness::Disease>(m).is_some() { continue; }
                if self.rng.below(100) < 5 {
                    self.world.add_component(m, crate::sim::illness::Disease::new(
                        crate::sim::illness::DiseaseKind::Gripes, m, 0.5));
                }
            }
        }
    }

    /// The keep's answer lands: pressure on the sentence, better bread,
    /// or the sick seen to - or nothing, and the cell bears it
    fn cell_answered(&mut self, tick: u64, site: usize, demand: Demand, grant: Grant) {
        let Some(i) = self.cells.iter().position(|c| c.site == site) else { return };
        self.cells[i].outstanding = None;
        let members = self.world.get_component::<Organization>(self.cells[i].org)
            .map(|o| o.members.clone()).unwrap_or_default();
        let word = match grant {
            Grant::Full => "granted",
            Grant::Part => "a part of it",
            Grant::Refused => "refused",
        };
        match demand {
            Demand::Release => {
                let amt = match grant { Grant::Full => 0.4, Grant::Part => 0.2, Grant::Refused => 0.0 };
                for &m in &members {
                    if self.world.get_component::<crate::game::dungeon::ReleaseCampaign>(m).is_some() {
                        crate::game::dungeon::press(&mut self.world, m, amt);
                    }
                }
            }
            Demand::Rations => {
                let up = match grant { Grant::Full => 0.3, Grant::Part => 0.15, Grant::Refused => 0.0 };
                self.cells[i].rations = (self.cells[i].rations + up).min(1.0);
            }
            Demand::Care => {
                let lift = match grant { Grant::Full => tick + 3, Grant::Part => tick + 7, Grant::Refused => u64::MAX };
                for &m in &members {
                    if let Some(c) = self.world.get_component_mut::<villager::Condition>(m) {
                        c.until = c.until.min(lift);
                    }
                }
            }
        }
        self.log.push_ev(
            EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
            format!("t{}: the keep answers the cell - {}.", tick, word),
        );
        self.toasts
            .push((format!("the keep answers: {}", word), tick + 40));
    }

    /// The knight-lord of a castle site: its Knight-vocation member
    fn site_knight(&self, site: usize) -> Option<Entity> {
        self.villagers.iter().copied().find(|&v|
            self.site_of(v) == Some(site)
                && self.world.get_component::<Craft>(v)
                    .map(|c| c.vocation == Vocation::Knight).unwrap_or(false)
                && !villager::is_dead(&self.world, v))
    }

    /// The castle nearest the village - where its taken are carried
    fn castle_site(&self) -> usize {
        self.sites.iter().enumerate()
            .filter(|(_, s)| s.kind == country::SiteKind::Castle)
            .map(|(i, s)| { let d = (s.centre.0 - self.village.0).pow(2) + (s.centre.1 - self.village.1).pow(2); (d, i) })
            .min().map(|(_, i)| i).unwrap_or(0)
    }
}
