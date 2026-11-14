//! Power and its pushback: sanctions and reputation, edicts, the dungeon and the guards, marks and the martyr, discoveries, organized power, and the festival.

use crate::session::*;

impl Game {
    /// The ladder of shame: when a villager's grievances pile past a
    /// rung, the village acts - one rung at a time, and the crowd's
    /// answer decides whether it lands
    pub fn sanction_tick(&mut self, tick: u64) {
        if tick % 7 != 4 { return; }
        // drop lapsed sanctions; the cold shoulder doesn't last forever
        self.sanctioned.retain(|(_, _, until)| tick < *until);
        // the most-grieved soul in the village - and how far clear of
        // the rest they stand: shame needs a wretch apart, not a village
        // that's all miserable together
        let mut best: Option<(Entity, usize)> = None;
        let mut second = 0usize;
        for v in self.villagers.iter().copied() {
            if crate::game::dungeon::unavailable(&self.world, v)
                || self.world.get_component::<TownRole>(v).is_some() { continue; }
            let n = self.world.get_component::<InnerLife>(v)
                .map(|l| l.grievances.iter().filter(|(_, u)| tick < *u).count())
                .unwrap_or(0);
            if n < 3 { continue; }
            match best {
                None => best = Some((v, n)),
                Some((_, m)) if n > m => { second = m; best = Some((v, n)); }
                Some(_) => second = second.max(n),
            }
        }
        let Some((v, n)) = best else { return };
        if n < second + 2 { return; }
        let current = self.sanctioned.iter().filter(|(e, _, _)| *e == v)
            .map(|(_, s, _)| *s).max();
        let next = sanctions::rung_for(n);
        // the pile the village already answered for - a lapsed shame
        // only reopens on a bigger heap, not the same sins twice
        let seen = self.sanction_seen.get(&v).copied().unwrap_or(0);
        // one rung at a time, and only upward
        let step = match (current, next) {
            // blame earns a bad word first; the ladder is climbed
            (None, Some(_)) if n > seen => Some(Sanction::Gossip),
            (Some(cur), Some(want)) if want > cur => {
                let i = sanctions::LADDER.iter().position(|s| *s == cur).unwrap_or(0);
                Some(sanctions::LADDER[(i + 1).min(5)])
            }
            _ => None,
        };
        if let Some(s) = step {
            self.sanction_seen.insert(v, n);
            self.apply_sanction(v, s, tick);
        }
    }

    /// Bad blood made visible: a feud that turns hot is the village's
    /// talk and its slander costs the offender good name; and on the
    /// holy day the priest's arbitration cools the hottest of them, the
    /// way the sagas end - talks first, a settlement if it's cooled enough
    pub(in crate::session) fn feud_tick(&mut self, tick: u64) {
        let rows: Vec<_> = self.world.query::<crate::game::feuds::Feud>().into_iter()
            .filter_map(|e| self.world.get_component::<crate::game::feuds::Feud>(e)
                .map(|f| (e, f.status, f.offender, f.aggrieved, f.offense.clone())))
            .collect();
        for (e, status, offender, aggrieved, offense) in rows {
            let last = self.feud_told.get(&e).copied();
            match status {
                crate::game::feuds::FeudStatus::Burning if last != Some(crate::game::feuds::FeudStatus::Burning) => {
                    self.feud_told.insert(e, crate::game::feuds::FeudStatus::Burning);
                    let (an, bn) = (self.name_of(aggrieved), self.name_of(offender));
                    self.log_entity_ev(
                        EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Notable),
                        offender,
                        format!(
                            "t{}: bad blood runs hot between {} and {} - {}.",
                            tick, an, bn, offense
                        ),
                    );
                    self.rep_apply(offender, RepAction::BadDeed);
                    for who in [offender, aggrieved] {
                        if let Some(i) = self.villagers.iter().position(|v| *v == who) {
                            note_story(&mut self.tales, &mut self.tales_told, &mut self.log, i, tick,
                                crate::sim::story::EventKind::Conflict, format!("{} - bad blood burns", offense));
                        }
                    }
                    // open conflict breaks the offender's sworn word:
                    // you can't aid a neighbour with one fist raised
                    for other in crate::game::contracts::break_all_pacts(&mut self.world, offender)
                    {
                        self.log_entity_ev(
                            EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Notable),
                            other,
                            format!(
                                "t{}: the pact between {} and {} is torn.",
                                tick,
                                bn,
                                self.name_of(other)
                            ),
                        );
                    }
                }
                crate::game::feuds::FeudStatus::Resolved if last != Some(crate::game::feuds::FeudStatus::Resolved) => {
                    self.feud_told.insert(e, crate::game::feuds::FeudStatus::Resolved);
                    let (an, bn) = (self.name_of(aggrieved), self.name_of(offender));
                    self.log_entity_ev(
                        EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Notable),
                        offender,
                        format!(
                            "t{}: the feud between {} and {} is put to rest.",
                            tick, an, bn
                        ),
                    );
                    for who in [offender, aggrieved] {
                        if let Some(i) = self.villagers.iter().position(|v| *v == who) {
                            note_story(&mut self.tales, &mut self.tales_told, &mut self.log, i, tick,
                                crate::sim::story::EventKind::Peace, "the bad blood is settled".into());
                        }
                    }
                }
                _ => {}
            }
        }
        if !routine::is_holy(tick) { return; }
        let Some(priest) = self.villagers.iter().copied().find(|&v|
            !villager::is_dead(&self.world, v)
                && !crate::game::dungeon::unavailable(&self.world, v)
                && self.site_of(v).is_none()
                && self.world.get_component::<Craft>(v)
                    .map(|c| c.vocation == Vocation::Priest).unwrap_or(false)) else { return };
        // the hottest unsettled blood gets the priest's word first
        let hot = self.world.query::<crate::game::feuds::Feud>().into_iter()
            .filter(|&e| self.world.get_component::<crate::game::feuds::Feud>(e)
                .map(|f| f.status != crate::game::feuds::FeudStatus::Resolved).unwrap_or(false))
            .max_by(|&a, &b| {
                let ia = self.world.get_component::<crate::game::feuds::Feud>(a).unwrap().intensity;
                let ib = self.world.get_component::<crate::game::feuds::Feud>(b).unwrap().intensity;
                ia.partial_cmp(&ib).unwrap_or(std::cmp::Ordering::Equal)
            });
        if let Some(fe) = hot {
            crate::game::feuds::peacemaking(&mut self.world, fe, priest);
        }
    }

    /// The holy-day office, run once the day's plans are laid: everyone
    /// keeping Worshipping today - church rows or the old rite, priest
    /// or no priest - gathers a little devotion and a bond, and the
    //  parish's poor box feeds the hungriest of the flock
    pub(in crate::session) fn service_tick(&mut self, tick: u64) {
        if !routine::is_holy(tick) { return; }
        let attendees: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v) && self.site_of(v).is_none()
                && self.world.get_component::<routine::Routine>(v)
                    .map(|r| r.mode == routine::Mode::Worshipping && r.day == tick)
                    .unwrap_or(false)).collect();
        if attendees.len() >= 2 {
            crate::sim::religion::perform_service(&mut self.world, self.congregation,
                crate::sim::religion::Service {
                    name: "the holy-day office".into(), frequency: "weekly".into(),
                    participants: attendees.clone(), cohesion_boost: 0.25, charity_output: 0.2 });
            // and the office moves a few souls a stage deeper now and then
            for &v in &attendees {
                crate::sim::religion::stage_up(&mut self.world, v, routine::mix(v, tick));
            }
        }
        // the green fellowship keeps its own word on the same holy day:
        // its own congregation, its own service - but the poor box
        // stays the parish's
        if let Some(sg) = self.splinter {
            crate::sim::religion::perform_service(&mut self.world, sg,
                crate::sim::religion::Service {
                    name: "the green word".into(), frequency: "weekly".into(),
                    participants: attendees, cohesion_boost: 0.3, charity_output: 0.0 });
        }
    }

    /// A provocation lands on the parish: the first spark kindles while
    /// the reformation_spark burns; later strikes change nothing. Sparks
    /// are authored events - a heretic's word, a failed cure, the tithe -
    /// never a periodic roll
    pub(in crate::session) fn spark_reformation(&mut self, tick: u64, kind: &'static str) {
        if self.splinter.is_some() { return; }
        if self.reformation_spark.is_none() {
            self.reformation_spark = Some((tick, kind));
        }
    }

    /// Days a struck spark stays lit waiting for susceptible soil
    const SPARK_DAYS: u64 = 30;
    /// The village needs a season to become a parish before it can
    /// reform one
    const REFORMATION_EARLIEST: u64 = 60;

    /// The reformation's slow burn, run daily beside the office: a lit
    /// spark finds the most susceptible soul and they take the green
    /// word; once the fellowship stands, parish hearts drift over a few
    /// at a time. Every roll is a mix() of the soul and the day - no
    /// rng stream moves
    pub(in crate::session) fn reformation_tick(&mut self, tick: u64) {
        if tick < Self::REFORMATION_EARLIEST { return; }
        // the fuse burns out if no soil took the spark
        if let Some((when, _)) = self.reformation_spark {
            if tick > when + Self::SPARK_DAYS { self.reformation_spark = None; }
        }
        if self.splinter.is_none() {
            if let Some((_, kind)) = self.reformation_spark {
                if let Some(v) = self.most_susceptible(tick) {
                    self.ignite_reformation(v, kind, tick);
                    self.reformation_spark = None;
                }
            }
            return;
        }
        // the fellowship grows: the faithful drift to the green word
        self.grow_splinter(tick);
        // a dead or gone leader hands the word to the next soul
        let Some(sg) = self.splinter else { return };
        let healthy = self.world.get_component::<crate::sim::religion::Congregation>(sg)
            .map(|c| c.leader.filter(|l| !villager::is_dead(&self.world, *l)
                && c.members.contains(l)).is_some())
            .unwrap_or(false);
        if !healthy {
            let next = self.world.get_component::<crate::sim::religion::Congregation>(sg)
                .map(|c| c.members.iter().copied()
                    .find(|&m| !villager::is_dead(&self.world, m)))
                .flatten();
            if let Some(c) = self.world.get_component_mut::<crate::sim::religion::Congregation>(sg) {
                c.leader = next;
            }
        }
    }

    /// Who the spark can land on: the parish heart most ready to turn -
    /// live grievances, a thinning devotion, a weak bond with the
    /// priest. The old faith, the clergy and the young never reform
    fn reformation_eligible(&self, v: Entity, need_grievance: bool, tick: u64) -> bool {
        use crate::sim::religion::{Apostate, Reformed};
        if villager::is_dead(&self.world, v)
            || crate::game::dungeon::unavailable(&self.world, v)
            || self.world.get_component::<Kid>(v).is_some()
            || self.world.get_component::<TownRole>(v).is_some()
            || self.world.get_component::<Reformed>(v).is_some()
            || self.world.get_component::<crate::sim::religion::Faith>(v).is_none()
            || routine::keeps_old_ways(&self.world, v)
            || self.site_of(v).is_some()
        { return false; }
        // the priest doesn't preach against himself
        let priest = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation)
            .and_then(|c| c.leader);
        if priest == Some(v) { return false; }
        // of the fold: a parish member, or one who already walked out
        let cong = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation);
        let of_fold = cong.map(|c| c.members.contains(&v)).unwrap_or(false)
            || self.world.get_component::<Apostate>(v).is_some();
        if !of_fold { return false; }
        if need_grievance {
            // the reformer needs the grievances plural - a slow burn,
            // not a single bad day
            return self.world.get_component::<InnerLife>(v)
                .map(|l| l.grievances.iter().filter(|(_, u)| *u > tick).count() >= 2)
                .unwrap_or(false);
        }
        true
    }

    /// How ready a soul is to turn: grievances weigh heaviest, then a
    /// devotion thinner than the flock's and a weak tie to the priest,
    /// with the pious just that much readier. A per-soul nudge keeps the
    /// pick lively without a roll
    fn susceptibility(&self, v: Entity, priest: Option<Entity>, tick: u64) -> f32 {
        let faith = self.world.get_component::<crate::sim::religion::Faith>(v)
            .map(|f| f.devotion).unwrap_or(0.5);
        let grievances = self.world.get_component::<InnerLife>(v)
            .map(|l| l.grievances.iter().filter(|(_, u)| *u > tick).count()).unwrap_or(0);
        let bond = priest.and_then(|p| crate::game::relationships::find_relationship(&self.world, v, p))
            .and_then(|r| self.world.get_component::<crate::game::relationships::Relationship>(r))
            .map(|r| r.strength).unwrap_or(0.0);
        let pious = routine::has_trait(&self.world, v, crate::game::traits::Trait::Pious) as u8 as f32;
        grievances as f32 * 0.25
            + (0.55 - faith).max(0.0) * 0.8
            + (0.4 - bond).max(0.0) * 0.5
            + pious * 0.1
            + (routine::mix(v, tick) % 100) as f32 / 1000.0
    }

    /// The most susceptible eligible soul, or none if the village's
    /// soil is too thin for the spark
    fn most_susceptible(&self, tick: u64) -> Option<Entity> {
        let priest = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation)
            .and_then(|c| c.leader);
        let mut best: Option<(Entity, f32)> = None;
        for &v in &self.villagers {
            if !self.reformation_eligible(v, true, tick) { continue; }
            let score = self.susceptibility(v, priest, tick);
            if best.map(|(_, s)| score > s).unwrap_or(true) { best = Some((v, score)); }
        }
        best.filter(|(_, s)| *s > 0.30).map(|(v, _)| v)
    }

    /// The spark catches: a new fellowship stands on the green, its
    /// founder walks out of the fold, and the parish answers - a
    /// denouncement always, the ladder's first rung when the parish is
    /// orthodox enough to care
    fn ignite_reformation(&mut self, v: Entity, kind: &str, tick: u64) {
        use crate::sim::religion::{Apostate, Congregation, Reformed};
        let name = self.name_of(v);
        let cong = self.world.create_entity();
        self.world.add_component(cong, Congregation {
            routinization: 0.0,
            absolutism: 0.0,
            name: "the green fellowship".into(),
            members: vec![v],
            leader: Some(v),
            orthodoxy: 0.25,
            services: vec![],
            cultish: false,
        });
        self.splinter = Some(cong);
        if let Some(c) = self.world.get_component_mut::<Congregation>(self.congregation) {
            c.members.retain(|&m| m != v);
        }
        self.world.add_component(v, Apostate);
        self.world.add_component(v, Reformed);
        if let Some(f) = self.world.get_component_mut::<crate::sim::religion::Faith>(v) {
            f.religion = "the green fellowship".into();
            f.devotion = f.devotion.max(0.55);
            if !f.traditions.iter().any(|t| t == "the green word") {
                f.traditions.push("the green word".into());
            }
        }
        desires::add_grievance(&mut self.world, v, desires::Problem::Doctrine, tick + 180);
        let at = self.world.get_component::<Pos>(v).map(|p| (p.x, p.y));
        self.log.push_ev_at_who(
            EvTag::new(Thread::Org, Valence::Positive, Sev::Notable),
            format!(
                "t{}: {} takes the green word - a fellowship is born of {}.",
                tick, name, kind
            ),
            at,
            None,
        );
        self.say(
            v,
            routine::GREEN_WORD[(routine::mix(v, tick) as usize) % routine::GREEN_WORD.len()],
            TALK_BUBBLE_SECS * 2,
        );
        // the parish answers: the priest denounces from the door, and a
        // strict parish reaches for the first rung of shame
        if let Some(p) = self.world.get_component::<Congregation>(self.congregation).and_then(|c| c.leader) {
            if !villager::is_dead(&self.world, p) {
                self.say(p, "A crow on the green - the parish will not have it.", TALK_BUBBLE_SECS * 2);
            }
        }
        let orth = self.world.get_component::<Congregation>(self.congregation)
            .map(|c| c.orthodoxy).unwrap_or(0.0);
        if orth >= 0.7 { self.apply_sanction(v, Sanction::Gossip, tick); }
    }

    /// Hearts drift to the green word: each eligible parish soul rolls
    /// a deterministic chance scaled by how ready they already are
    fn grow_splinter(&mut self, tick: u64) {
        use crate::sim::religion::{Apostate, Congregation, Reformed};
        let Some(sg) = self.splinter else { return };
        let priest = self.world.get_component::<Congregation>(self.congregation).and_then(|c| c.leader);
        let defectors: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| self.reformation_eligible(v, false, tick))
            .filter(|&v| {
                let score = self.susceptibility(v, priest, tick);
                let chance = (score * 80.0).min(60.0) as u64;
                routine::mix(v, tick.wrapping_add(0x6A71)) % 1000 < chance
            }).collect();
        for v in defectors {
            let name = self.name_of(v);
            if let Some(c) = self.world.get_component_mut::<Congregation>(self.congregation) {
                c.members.retain(|&m| m != v);
            }
            self.world.add_component(v, Apostate);
            self.world.add_component(v, Reformed);
            if let Some(f) = self.world.get_component_mut::<crate::sim::religion::Faith>(v) {
                f.religion = "the green fellowship".into();
                if !f.traditions.iter().any(|t| t == "the green word") {
                    f.traditions.push("the green word".into());
                }
            }
            desires::add_grievance(&mut self.world, v, desires::Problem::Doctrine, tick + 90);
            if let Some(c) = self.world.get_component_mut::<Congregation>(sg) {
                if !c.members.contains(&v) { c.members.push(v); }
            }
            let at = self.world.get_component::<Pos>(v).map(|p| (p.x, p.y));
            self.log.push_ev_at_who(
                EvTag::new(Thread::Org, Valence::Positive, Sev::Minor),
                format!("t{}: {} takes the green word.", tick, name),
                at,
                None,
            );
        }
    }

    /// A rung lands: what the village does to its own
    pub(in crate::session) fn apply_sanction(&mut self, v: Entity, s: Sanction, tick: u64) {
        let name = self.villagers.iter().position(|x| *x == v).map(|i| vill_name(i)).unwrap_or("someone");
        // a bad word travels ahead of every rung
        self.rep_apply(v, RepAction::BadDeed);
        self.sanctioned.push((v, s, tick + 60));
        match s {
            Sanction::Gossip => {
                self.log_entity_ev(
                    EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                    v,
                    format!("t{}: the talk turns on {}.", tick, name),
                );
                // the words they reach for, flourished
                self.say(v, &format!("They call me {}.", gossip_insult(v as usize)), TALK_BUBBLE_SECS);
            }
            Sanction::Shun => {
                self.log_entity_ev(
                    EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Notable),
                    v,
                    format!(
                        "t{}: {} gets the cold shoulder - no one sits with them.",
                        tick, name
                    ),
                );
            }
            Sanction::Boycott => {
                self.log_entity_ev(
                    EvTag::new(Thread::Economy, Valence::Negative, Sev::Notable),
                    v,
                    format!("t{}: nobody buys from {} - the boycott bites.", tick, name),
                );
            }
            Sanction::Picket => {
                self.log_entity_ev(
                    EvTag::new(Thread::Org, Valence::Negative, Sev::Notable),
                    v,
                    format!(
                        "t{}: a crowd at {}'s door - they can't work for the press.",
                        tick, name
                    ),
                );
                self.say(v, "Let me through - I've work to do!", TALK_BUBBLE_SECS);
                // the log line made good: the neighbours come, signs in hand
                self.call_picket(v, tick);
            }
            Sanction::Stocks => {
                // the crowd's answer decides: a respected soul shamed is a
                // scandal, a disliked one shamed is a lesson
                if self.rep_of(v) > 0.5 {
                    self.rep_apply(v, RepAction::GreatDeed); // sympathy lifts them
                    // the rung was recorded before the crowd answered -
                    // a sentence never served is no rung at all
                    self.sanctioned.retain(|(e, s, _)| !(*e == v && *s == Sanction::Stocks));
                    self.log_entity_ev(EvTag::new(Thread::Law, Valence::Positive, Sev::Notable), v, format!("t{}: the crowd would have none of it - {} walks free of the stocks, and the shame lands on the accusers.", tick, name));
                    self.say(v, "They wronged me, and you all saw it.", TALK_BUBBLE_SECS);
                } else {
                    self.rep_apply(v, RepAction::Betrayal);
                    self.log_entity_ev(
                        EvTag::new(Thread::Law, Valence::Negative, Sev::Notable),
                        v,
                        format!("t{}: {} sits in the stocks by the store.", tick, name),
                    );
                    // the stocks stand by the market - pick their tile the
                    // first time a sentence needs it, and it stands after
                    if self.stocks_tile.is_none() {
                        let mp = self.world.get_component::<Pos>(self.market_food)
                            .map(|p| (p.x.round() as i32, p.y.round() as i32));
                        self.stocks_tile = mp.and_then(|(mx, my)|
                            (0..4).map(|i| (mx + [1, -1, 0, 0][i], my + [0, 0, 1, -1][i]))
                                .find(|&(tx, ty)| self.map.passable(tx, ty) && !self.map.is_occupied(tx, ty))
                                .or(Some((mx + 1, my))));
                    }
                    self.send(v, Goal::GoStocks);
                    self.say(v, "...all right. All right.", TALK_BUBBLE_SECS);
                }
            }
            Sanction::Exile => {
                self.log_entity_ev(
                    EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Major),
                    v,
                    format!(
                        "t{}: the village turns {} out - nothing left for them here.",
                        tick, name
                    ),
                );
                self.send(v, Goal::Exile);
            }
        }
    }

    /// Their standing with the crowd (seeded mid if never tracked)
    pub(in crate::session) fn rep_of(&mut self, v: Entity) -> f32 {
        if self.world.get_component::<ReputationState>(v).is_none() {
            self.world.add_component(v, ReputationState {
                stage: RepStage::Known, score: 0.45, transition_rates: vec![],
            });
        }
        self.world.get_component::<ReputationState>(v).map(|r| r.score).unwrap_or(0.45)
    }

    /// New traders joining late - migrants, grown children - carry the
    /// Merchant the guild's compact wants; a day of arrivals sees them
    /// tagged and enrolled on its next run
    pub(in crate::session) fn guild_tick(&mut self, _tick: u64) {
        for (i, &v) in self.villagers.iter().enumerate().collect::<Vec<_>>() {
            if self.world.get_component::<crate::game::guilds::Merchant>(v).is_none()
                && is_trader(&self.world, v) {
                tag_trader(&mut self.world, v, vill_name(i));
            }
        }
    }

    pub(in crate::session) fn rep_apply(&mut self, v: Entity, a: RepAction) {
        let _ = self.rep_of(v);
        if let Some(r) = self.world.get_component_mut::<ReputationState>(v) {
            crate::sim::reputation_qsp::apply_action(r, a);
        }
    }

    /// Is this shop's worker under boycott or worse? Their trade dries up
    pub fn boycotted(&self, shop: Entity) -> bool {
        let Some(worker) = self.world.get_component::<Building>(shop).and_then(|b| b.worker) else { return false };
        self.sanctioned.iter().any(|(e, s, u)| *e == worker && *s >= Sanction::Boycott && self.tick < *u)
    }

    /// Is this villager under the cold shoulder or worse?
    pub fn shunned(&self, v: Entity) -> bool {
        self.sanctioned.iter().any(|(e, s, u)| *e == v && *s >= Sanction::Shun && self.tick < *u)
    }

    /// The lord's arm: every season a writ comes down - a levy of grain,
    /// a fee in gold, a summons for one of the village's own. An org
    /// worth the name bargains it down; a strong one may stand against
    /// a summons - and pay for it when the next writ comes double.
    pub fn edict_tick(&mut self, tick: u64) {
        // word rides ahead of the bailiff: the village hears the writ's
        // coming a few days before it lands - time to gather, to bargain,
        // or to hide the grain
        if tick % 120 == 76 {
            self.log.push_ev(EvTag::new(Thread::Law, Valence::Neutral, Sev::Notable), format!("t{}: word on the road - the bailiff rides this way. A writ comes down in a few days.", tick));
        }
        if tick % 120 != 80 { return; }
        let kind = (tick / 120) % 4;
        // the debt counts the defiances - a summons reads it before the
        // writ's own weight consumes it
        let debt = self.repression_debt;
        let mult = if self.repression_debt > 0 { let m = 1 + self.repression_debt; self.repression_debt = 0; m } else { 1 };
        // the org's answer rides its standing with the lord
        let bargained = self.lord_esteem > 0.5;
        match kind {
            0 => {
                // a levy: grain off the store
                let want = (8 * mult) as f32;
                let take = if bargained { want / 2.0 } else { want } as u32;
                if let Some(b) = self.world.get_component_mut::<Building>(self.market_food) {
                    let _ = household::take_stock(b, "grain", take);
                }
                self.log.push_ev_at_who(
                    EvTag::new(Thread::Law, Valence::Negative, Sev::Major),
                    format!(
                        "t{}: a bailiff's writ - the castle levies {} grain{}.{}",
                        tick,
                        take,
                        if bargained {
                            " - the org talks him down"
                        } else {
                            ""
                        },
                        if mult > 1 {
                            " Double, for the last defiance."
                        } else {
                            ""
                        }
                    ),
                    toast_spot(&self.world, self.market_food),
                    None,
                );
                self.notices.push((
                    "writ".into(),
                    format!("the castle levies {} grain", take),
                    tick + 7,
                ));
                if let Some(b) = self.world.get_component_mut::<Building>(self.market_food) {
                    b.flyer = Some(format!("levy: {} grain due", take));
                }
            }
            1 => {
                // a fee: gold off every purse that has it
                let want = (12 * mult) as f32;
                let each = if bargained { want / 2.0 } else { want };
                let mut taken = 0.0f32;
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) { continue; }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) {
                        let cut = (each / self.villagers.len() as f32).min(p.gold);
                        p.gold -= cut; taken += cut;
                    }
                }
                self.log.push_ev(
                    EvTag::new(Thread::Law, Valence::Negative, Sev::Major),
                    format!(
                        "t{}: a bailiff's writ - a fee of {:.0} gold off the village.{}",
                        tick,
                        taken,
                        if bargained {
                            " Halved - the org's word counts."
                        } else {
                            ""
                        }
                    ),
                );
                self.notices.push((
                    "writ".into(),
                    format!("a fee of {:.0} gold off the village", taken),
                    tick + 7,
                ));
            }
            3 => {
                // the drink levy: the castle's share of every barrel in
                // the village - a quarter of what pours, off every stock;
                // and the grudge reads its weight here too
                let frac = if bargained { 0.125 } else { 0.25 };
                let share = |q: u32| (((q as f32) * frac).ceil() as u32).saturating_mul(mult).min(q);
                let mut taken = 0u32;
                for &be in &self.building_entities.clone() {
                    if let Some(b) = self.world.get_component_mut::<Building>(be) {
                        for g in ["ale", "beer", "wine", "spirits"] {
                            let q = b.stock.get(g).copied().unwrap_or(0);
                            if q > 0 { taken += household::take_stock(b, g, share(q)); }
                        }
                    }
                }
                self.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Major), format!("t{}: a bailiff's writ - the castle's share of the barrel, {} measures off the village's drink.{}",
                    tick, taken,
                    if bargained { " Halved - the org's word counts." } else { "" }));
                self.notices.push(("writ".into(), format!("the castle's share of the barrel: {} measures", taken), tick + 7));
            }
            _ => {
                // a summons: one of the village's own for 'questioning' -
                // a strong org can stand against it
                let mark = self.villagers.iter().copied()
                    .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<TownRole>(v).is_none()
                        // the guild's mutual protection: the bailiff's
                        // summons finds no hold on a member
                        && !self.world.query::<crate::game::guilds::Guild>().iter()
                            .any(|&g| self.world.get_component::<crate::game::guilds::Guild>(g)
                                .map(|gg| gg.members.contains(&v)).unwrap_or(false)))
                    .max_by_key(|&v| self.world.get_component::<InnerLife>(v)
                        .map(|l| l.grievances.len()).unwrap_or(0));
                if let Some(m) = mark {
                    let member = self.world.get_component::<Organization>(self.org)
                        .map(|o| o.members.contains(&m)).unwrap_or(false);
                    if member && self.lord_esteem > 0.7 && debt >= 2 {
                        // the third defiance: the bailiff doesn't come back
                        // alone - a writ at max repression is a gibbet
                        let name = self.name_of(m);
                        let at = self.world.get_component::<Pos>(m).map(|p| (p.x, p.y));
                        self.world.add_component(m, villager::Dead);
                        if let Some(i) = self.villagers.iter().position(|&x| x == m) {
                            self.died_on[i] = Some(tick);
                        }
                        if let Some(p) = self.world.get_component::<Pos>(m) {
                            self.martyr_grave = Some((p.x.round() as i32, p.y.round() as i32));
                        }
                        self.repression_debt = 0;
                        self.log_entity_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Major), m, format!("t{}: the bailiff returns with knights - {} is hanged for the defiance.", tick, name));
                        // mass grief: everyone saw it
                        for v in self.villagers.iter().take(8).copied().collect::<Vec<_>>() {
                            if !villager::is_dead(&self.world, v) {
                                desires::add_grievance(&mut self.world, v, desires::Problem::Grief, tick + 60);
                            }
                        }
                        // and the wronged remember: the spark of recruits
                        // arrives on a delay - repression's deferred debt
                        if let Some(at) = at { self.mark_radicalized(at, tick); }
                    } else if member && self.lord_esteem > 0.7 {
                        self.repression_debt += 1;
                        self.log_entity_ev(EvTag::new(Thread::Law, Valence::Positive, Sev::Major), m, format!("t{}: a writ for {} - and the village stands against it. The bailiff rides home empty - and the castle won't forget it.", tick, self.name_of(m)));
                    } else {
                        self.log_entity_ev(
                            EvTag::new(Thread::Law, Valence::Negative, Sev::Notable),
                            m,
                            format!(
                                "t{}: a summons - {} is taken for questioning.",
                                tick,
                                self.name_of(m)
                            ),
                        );
                        for _ in 0..2 {
                            desires::add_grievance(
                                &mut self.world,
                                m,
                                desires::Problem::WritBorne,
                                tick + 60,
                            );
                        }
                        // taken to the castle - held, not gone: the org
                        // ransoms them or the ladder climbs. The march
                        // out goes first - it's the last errand they run
                        // as a free pair of legs, then the mark lands
                        self.dungeon.push((m, tick));
                        self.travels.retain(|t| t.who != m);
                        // held at the castle: the walk goes to the keep's
                        // own cell block, not the road out - picked once
                        // at the site's centre, and it stands after
                        if self.castle_gaol.is_none() {
                            let castle = self.sites.iter().find(|s| s.kind == country::SiteKind::Castle)
                                .map(|s| s.centre);
                            self.castle_gaol = castle.map(|(cx, cy)|
                                (0..4).map(|i| (cx + [1, -1, 0, 0][i], cy + [0, 0, 1, -1][i]))
                                    .find(|&(tx, ty)| self.map.passable(tx, ty) && !self.map.is_occupied(tx, ty))
                                    .unwrap_or((cx, cy)));
                        }
                        if let Some(g) = self.castle_gaol {
                            self.send_far(m, g, Goal::GoGaol);
                        } else {
                            self.send(m, Goal::Exile);
                        }
                        self.world.add_component(m, crate::game::dungeon::CastleHeld { since: tick });
                    }
                }
            }
        }
        // a writ that lands heavier than it should - the debt coming due -
        // is seen at the market, and seen is remembered. The summons kind
        // doesn't scale with the debt, so only the levies mark the crowd
        if mult > 1 && kind != 2 {
            let at = self.world.get_component::<Pos>(self.market_food).map(|p| (p.x, p.y));
            if let Some(at) = at { self.mark_radicalized(at, tick); }
        }
        // and the writ itself is a thing to organize about
        for v in self.villagers.iter().take(3).copied().collect::<Vec<_>>() {
            if !villager::is_dead(&self.world, v) {
                desires::add_grievance(&mut self.world, v, desires::Problem::WritBorne, tick + 30);
            }
        }
    }

    /// Repression's deferred debt: whoever watched a hanging or a writ
    /// land heavier than it should carries the cause's spark for weeks -
    /// the tactical win's strategic loss arrives as recruits, not as the
    /// bailiff's ledger
    fn mark_radicalized(&mut self, at: (f32, f32), tick: u64) {
        let until = tick + 80;
        let mut marked = 0;
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            let near = self.world.get_component::<Pos>(v)
                .map(|p| (p.x - at.0).abs() + (p.y - at.1).abs() < 12.0).unwrap_or(false);
            if near {
                self.world.add_component(v, crate::game::reactions::Radicalized { until });
                marked += 1;
            }
        }
        if marked > 0 {
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                format!(
                    "t{}: the wronged remember - {} will hear the cause's call.",
                    tick, marked
                ),
            );
        }
    }

    /// The spark of the wronged fades if the cause doesn't catch it first
    pub(in crate::session) fn radical_decay(&mut self, tick: u64) {
        for &v in &self.villagers.clone() {
            let spent = self.world.get_component::<crate::game::reactions::Radicalized>(v)
                .map(|r| tick >= r.until).unwrap_or(false);
            if spent { self.world.remove_component::<crate::game::reactions::Radicalized>(v); }
        }
    }

    /// The castle's prisoners: a month to ransom them, or the ladder
    /// climbs to its top. The org's purse answers for its own.
    pub fn dungeon_tick(&mut self, tick: u64) {
        if self.dungeon.is_empty() || tick % 15 != 5 { return; }
        let mut resolved: Vec<(Entity, bool)> = vec![]; // (who, ransomed?)
        for &(who, taken) in &self.dungeon.clone() {
            let held = tick - taken;
            // the village's cause answers for the held one: the first
            // seat whose org has hands and a purse for it (#962 - before,
            // seat 0's purse answered for every prisoner)
            let payer = (0..self.seats.len()).find(|&s|
                self.world.get_component::<Organization>(self.seats[s].org)
                    .map(|o| !o.members.is_empty()).unwrap_or(false)
                && self.world.get_component::<PlayerCharacter>(self.seats[s].player)
                    .map(|p| p.resources >= 20.0).unwrap_or(false));
            if held >= 30 && held < 60 && payer.is_some() {
                // the org pays the ransom: the purse answers for its own
                if let Some(p) = self.world.get_component_mut::<PlayerCharacter>(self.seats[payer.unwrap()].player) {
                    p.resources -= 20.0;
                }
                resolved.push((who, true));
            } else if held >= 60 {
                // a month unanswered: the ladder climbs to its top
                resolved.push((who, false));
            }
        }
        for (who, ransomed) in resolved {
            self.dungeon.retain(|(e, _)| *e != who);
            let name = self.name_of(who);
            if ransomed {
                self.world.remove_component::<crate::game::dungeon::CastleHeld>(who);
                // the walk out is spent with the ransom - else the exile
                // leg outruns the pardon and arrives a driving-out
                self.travels.retain(|t| t.who != who);
                self.log_entity_ev(
                    EvTag::new(Thread::Org, Valence::Positive, Sev::Notable),
                    who,
                    format!("t{}: the org pays the ransom - {} walks home.", tick, name),
                );
                if let Some(home) = household::home_of(&self.world, who).or(Some(self.market_food))
                {
                    self.send(who, Goal::HeadHome(home));
                }
            } else {
                self.world.remove_component::<crate::game::dungeon::CastleHeld>(who);
                self.world.add_component(who, villager::Dead);
                if let Some(i) = self.villagers.iter().position(|&x| x == who) {
                    self.died_on[i] = Some(tick);
                }
                self.log_entity_ev(
                    EvTag::new(Thread::Law, Valence::Negative, Sev::Major),
                    who,
                    format!(
                        "t{}: no ransom came - {} is hanged at the castle.",
                        tick, name
                    ),
                );
            }
        }
    }

    /// The king's patience has a length: posted guards stand their tour,
    /// then ride home - the streets are the village's again, and a
    /// crackdown that comes after has to post them fresh
    /// The cell lets go of one prisoner: the components clear, and a
    /// release campaign raised just for them is spent with the release
    pub(in crate::session) fn free_prisoner(&mut self, m: Entity, tick: u64) {
        self.world.remove_component::<crate::game::dungeon::Imprisoned>(m);
        self.world.remove_component::<crate::game::dungeon::ReleaseCampaign>(m);
        // the cell lets go - the walk home starts where the door swings
        if let Some(home) = household::home_of(&self.world, m).or(Some(self.market_food)) {
            if let Some(door) = self.world.get_component::<Pos>(home).copied()
                .and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32)) {
                self.send_far(m, door, Goal::HeadHome(home));
            }
        }
        if let Some(rc) = self
            .world
            .get_component::<crate::game::dungeon::ReleaseOf>(m)
            .map(|r| r.campaign)
        {
            self.world
                .remove_component::<crate::game::campaign::Campaign>(rc);
            self.world
                .remove_component::<crate::game::campaign::CampaignMeeting>(rc);
            self.world
                .remove_component::<crate::game::dungeon::ReleaseOf>(m);
        }
        self.log_entity_ev(
            EvTag::new(Thread::Law, Valence::Positive, Sev::Notable),
            m,
            format!(
                "t{}: the cell stands open - {} walks home.",
                tick,
                self.name_of(m)
            ),
        );
    }

    pub fn guards_tick(&mut self, tick: u64) {
        // a stint inside is a stint inside: nobody sits past one, whatever
        // fresh crackdowns restamp a tour
        for &m in &self.villagers.clone() {
            let Some(since) = self.world.get_component::<crate::game::dungeon::Imprisoned>(m)
                .map(|i| i.since) else { continue; };
            if tick >= since + GUARD_TOUR { self.free_prisoner(m, tick); }
        }
        for seat in 0..self.seats.len() {
            let org = self.seats[seat].org;
            let stood_down = self.world.get_component::<crate::game::reactions::GuardsPosted>(org)
                .map(|g| tick >= g.since + GUARD_TOUR).unwrap_or(false);
            if !stood_down { continue; }
            self.world.remove_component::<crate::game::reactions::GuardsPosted>(org);
            // they stand where they were posted no more: this org's
            // knights ride home (another's may still be on their tour;
            // the gate watch is a different corps entirely)
            self.posted.retain(|&(o, _, _)| o != org);
            // and without the knights the cells can't hold: this org's held
            // walk free early, petitions answered or not
            let members = self.world.get_component::<Organization>(org)
                .map(|o| o.members.clone()).unwrap_or_default();
            for &m in &self.villagers.clone() {
                if self.world.get_component::<crate::game::dungeon::Imprisoned>(m).is_none() { continue; }
                // whose cell is it? the org that founded the release
                // campaign owns the prisoner; without one, membership says
                let ours = self.world.get_component::<crate::game::dungeon::ReleaseOf>(m)
                    .and_then(|r| self.world.get_component::<crate::game::campaign::Campaign>(r.campaign)
                        .map(|c| c.org == org))
                    .unwrap_or_else(|| members.contains(&m));
                if ours { self.free_prisoner(m, tick); }
            }
            self.log.push_ev(
                EvTag::new(Thread::Law, Valence::Neutral, Sev::Minor),
                format!("t{}: the knights ride home - the tour is done.", tick),
            );
        }
    }

    /// What a cause leaves on people fades on its own clock: a season
    /// cools a moral shock (reactions::cool_off), a month dulls a
    /// blow-off (actions::lifts_with_time). Coaching is still the fast
    /// way back - this is just the floor under "stuck forever".
    pub fn marks_tick(&mut self, tick: u64) {
        for v in self.villagers.clone() {
            crate::game::reactions::cool_off(&mut self.world, v, tick);
            crate::game::actions::lifts_with_time(&mut self.world, v, tick);
            // and a name fades the same way: esteem cools without upkeep,
            // even rock-bottom disgrace is eventually forgotten
            if let Some(r) = self.world.get_component_mut::<ReputationState>(v) {
                crate::sim::reputation_qsp::transition_tick(r);
            }
            // folk don't forget the man inside: a little pressure gathers
            // every day even when nobody's pressing - a cell can't hold
            // a name forever on suspicion alone
            if !villager::is_dead(&self.world, v)
                && self.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some() {
                crate::game::dungeon::press(&mut self.world, v, 0.02);
                if crate::game::dungeon::release_check(&mut self.world, v, 1.0) {
                    let name = self.name_of(v);
                    self.log_entity_ev(
                        EvTag::new(Thread::Law, Valence::Positive, Sev::Notable),
                        v,
                        format!(
                            "t{}: the village never stopped asking - {} walks free.",
                            tick, name
                        ),
                    );
                    if let Some(home) =
                        household::home_of(&self.world, v).or(Some(self.market_food))
                    {
                        self.send(v, Goal::HeadHome(home));
                    }
                }
            }
        }
    }

    /// The martyr's grave: they gather there, and the remembering is a
    /// kind of organizing no castle can stamp out
    pub fn martyr_tick(&mut self, tick: u64) {
        let Some(_) = self.martyr_grave else { return };
        if tick % 30 != 20 { return; }
        // a vigil: the org's members keep the grave - the remembering
        // feeds them where they stand; those free to walk, walk out to it
        let Some(grave) = self.martyr_grave else { return };
        let members = self.world.get_component::<Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        let mut roster: Vec<Entity> = self.villagers.iter().take(5).copied().collect();
        for &m in &members { if !roster.contains(&m) { roster.push(m); } }
        for v in roster {
            if villager::is_dead(&self.world, v) { continue; }
            desires::satisfy(&mut self.world, v, Want::Faith, tick + 15);
            // a walk already under way is theirs, and a seat's own
            // villager answers only its commands - the vigil takes neither
            if self.travels.iter().any(|t| t.who == v) { continue; }
            if self.seats.iter().any(|s| s.player == v) { continue; }
            self.send_tile_walk(v, grave, Goal::Wander, 40);
        }
        self.record_power(crate::sim::organized_power::ActionQuality::Transactional);
        if tick % 90 == 20 {
            self.log.push_ev_at_who(
                EvTag::new(Thread::Org, Valence::Positive, Sev::Minor),
                format!("t{}: they keep a vigil at the martyr's grave.", tick),
                self.martyr_grave
                    .map(|(x, y)| (x as f32 + 0.5, y as f32 + 0.5)),
                None,
            );
        }
    }

    /// The player is the org, not a body: when the first pair of hands
    /// stops, the work passes to the member who knew them best - or the
    /// kin under the same roof - and goes on. Each seat carries its own
    /// passing: a second player's death never ran this at all (#962)
    pub fn continuation_tick(&mut self, tick: u64) {
        for seat in 0..self.seats.len() {
            let dead = self.seats[seat].player;
            if self.seats[seat].player_passed || !villager::is_dead(&self.world, dead) { continue; }
            // the successor: an org member first - the organizing's
            // institutional - then kin under the same roof
            let members = self.world.get_component::<Organization>(self.seats[seat].org)
                .map(|o| o.members.clone()).unwrap_or_default();
            let org_pick = members.iter().copied()
                .filter(|&m| !villager::is_dead(&self.world, m) && m != dead)
                .max_by_key(|&m| relationships::find_relationship(&self.world, dead, m)
                    .map(|r| self.world.get_component::<relationships::Relationship>(r)
                        .map(|rr| (rr.strength * 100.0) as u32).unwrap_or(0)).unwrap_or(0));
            let kin_pick = household::home_of(&self.world, dead)
                .and_then(|h| self.world.get_component::<Building>(h)
                    .map(|b| b.occupants.iter().copied()
                        .find(|&o| o != dead && !villager::is_dead(&self.world, o))))
                .flatten();
            let succ = org_pick.or(kin_pick);
            self.seats[seat].player_passed = true;
            let Some(succ) = succ else {
                self.log.push_ev(
                    EvTag::new(Thread::Org, Valence::Negative, Sev::Notable),
                    format!(
                        "t{}: the work dies with them - no hands left to take it up.",
                        tick
                    ),
                );
                continue;
            };
            // the purse passes; the name and the work are theirs to make
            let res = self.world.get_component::<PlayerCharacter>(dead)
                .map(|p| p.resources).unwrap_or(0.0);
            self.world.add_component(succ, PlayerCharacter {
                name: self.name_of(succ),
                resources: res,
            });
            let name = self.name_of(succ);
            self.seats[seat].player = succ;
            self.log_entity_ev(
                EvTag::new(Thread::Org, Valence::Positive, Sev::Notable),
                succ,
                format!(
                    "t{}: the work goes on - {} takes up the organizing.",
                    tick, name
                ),
            );
            self.say(
                succ,
                "The work goes on. Someone has to.",
                CHAT_BUBBLE_SECS * 2,
            );
        }
    }

    /// The disclosure: a truth sits unsaid until it ripens, then it
    /// comes out - and the power it threatens answers. A strong org
    /// shields the discoverer; a weak one leaves him to the ladder.
    pub fn discovery_tick(&mut self, tick: u64) {
        if tick % 30 != 20 || self.disclosure_made { return; }
        if let Some((who, found, day)) = self.pending_discovery {
            // it ripens a fortnight, then comes out in a whisper
            if tick >= day + 14 {
                self.pending_discovery = None;
                self.disclosure_made = true;
                let name = self.name_of(who);
                self.log_entity_ev(
                    EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                    who,
                    format!(
                        "t{}: {} whispers it about - {}. The word's out now.",
                        tick, name, found
                    ),
                );
                if self.lord_esteem > 0.4 {
                    // an org worth the name stands with its own
                    self.log_entity_ev(
                        EvTag::new(Thread::Org, Valence::Positive, Sev::Notable),
                        who,
                        format!(
                            "t{}: the village stands with {} - the truth was theirs to say.",
                            tick, name
                        ),
                    );
                    self.say(
                        who,
                        "I said what I saw. That's all I did.",
                        CHAT_BUBBLE_SECS * 2,
                    );
                } else {
                    // no one to stand with him: the threatened move - and
                    // the ladder's there for a man who talks treason
                    for _ in 0..3 {
                        desires::add_grievance(&mut self.world, who, desires::Problem::Blamed, tick + 90);
                    }
                    self.say(who, "I only said what I saw.", CHAT_BUBBLE_SECS * 2);
                }
            }
            return;
        }
        // what there is to find: the clerk's tallies, the old cures
        let found = if let Some(t) = self.translator {
            Some((t, "the lord's fifth is really a quarter"))
        } else if self.cured >= 3 {
            self.herbologist.map(|h| (h, "the old cures work, doctrine or no"))
        } else { None };
        if let Some((who, found)) = found {
            self.pending_discovery = Some((who, found, tick));
        }
    }

    /// People power: it builds as the org spends it and atrophies daily
    /// when idle - and the lord's estimate lags behind the truth, for he
    /// only learns what the street shows him
    pub fn power_tick(&mut self, tick: u64) {
        if tick % 7 != 3 { return; }
        let capacity = self.world.get_component::<crate::sim::organized_power::OrganizedPower>(self.org)
            .map(|p| p.capacity).unwrap_or(0.0);
        // a season of the clerk's books and the count is everyone's:
        // the village keeps it the positional way for good, clerk or no
        if let Some(c) = self.translator {
            // the books count the days he's free to keep them: the held
            // teach nothing till they're out and back at the tallies
            if !crate::game::dungeon::unavailable(&self.world, c)
                && !crate::game::villager::is_dead(&self.world, c) {
                self.translator_days += 7;
            }
            if self.numeracy != Numeracy::Positional && self.translator_days >= 90 {
                self.numeracy = Numeracy::Positional;
                self.log_entity_ev(EvTag::new(Thread::Economy, Valence::Positive, Sev::Notable), c, format!("t{}: the village counts the clerk's way now - the foreign reckonings are done.", tick));
            }
        }
        // the lord's estimate creeps toward what's shown - perceived, not raw
        let was = self.lord_esteem;
        self.lord_esteem += (capacity - self.lord_esteem) * 0.08;
        // and the word of it reaches the village: not a number, the mood
        // the castle gives off - heard when it tips a step either way
        let tier = |e: f32| if e >= 0.7 { 3 } else if e >= 0.5 { 2 } else if e >= 0.3 { 1 } else { 0 };
        if tier(self.lord_esteem) != tier(was) {
            let up = self.lord_esteem > was;
            // it's the Harvesters' name on his lips, whichever seat reads it
            let oname = self.world.get_component::<Organization>(self.org)
                .map(|o| o.name.clone()).unwrap_or_else(|| "the cause".into());
            let word = match (tier(self.lord_esteem), up) {
                (3, true) => format!("word comes down - the lord frets over {}", oname),
                (2, true) => format!("word comes down - the lord keeps an ear on {} now", oname),
                (1, true) => format!("word comes down - the castle's heard the name {}", oname),
                (2, false) => format!("word comes down - the castle's worry about {} eases a step", oname),
                (1, false) => format!("word comes down - {} slips their minds", oname),
                _ => format!("word comes down - the castle's forgot {} ever met", oname),
            };
            let ev = if up {
                EvTag::new(Thread::Law, Valence::Negative, Sev::Notable)
            } else {
                EvTag::new(Thread::Law, Valence::Positive, Sev::Minor)
            };
            self.log.push_ev(ev, format!("t{}: {}.", tick, word));
        }
    }

    /// An org action in the street: power builds as it's spent
    pub fn record_power(&mut self, q: crate::sim::organized_power::ActionQuality) {
        let org = self.org;
        crate::sim::organized_power::record_action(&mut self.world, org, q);
    }

    /// The year's feast days: one mid-season each quarter, the village
    /// gathers at the market, the good stock comes out, and for a day
    /// the wants that never quite get fed are fed
    /// The year's own days: a birthday's observance lands where the
    /// celebrant's means can take it - a poor soul's meal, a neighborly
    /// call, a full little gathering where the gentry keeps the door
    pub(in crate::session) fn birthday_tick(&mut self, tick: u64) {
        let today = (tick % 360) as u16;
        for v in self.villagers.clone() {
            let Some(a) = self.world.get_component::<crate::game::lifecycle::Age>(v)
                .map(|a| a.born_day) else { continue };
            if a != today { continue; }
            let name = self.name_of(v);
            // what they can afford picks the observance: the gentry's
            // door stands open, a tradesman's neighbours call, and a
            // poor soul's day is a meal and a kind word
            let gentry = self
                .world
                .get_component::<Craft>(v)
                .map(|c| matches!(c.vocation, Vocation::Knight | Vocation::Priest))
                .unwrap_or(false);
            let gregarious = self
                .world
                .get_component::<crate::sim::psychology::Personality>(v)
                .map(|p| p.extraversion > 0.6)
                .unwrap_or(false);
            let scale = if gentry || gregarious {
                "the whole lane calls"
            } else if self.world.get_component::<Craft>(v).is_some() {
                "the neighbours drop by"
            } else {
                "a meal and a kind word"
            };
            self.log.push_ev(
                EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Minor),
                format!("t{}: it's {}'s birthday - {}.", tick, name, scale),
            );
            // and the day's own gladness: the celebrant's want for the
            // gathering's warmth finds its satisfaction
            if let Some(l) = self.world.get_component_mut::<desires::InnerLife>(v) {
                l.wants.retain(|w| *w != desires::Want::Love);
            }
        }
    }

    pub fn festival_tick(&mut self, tick: u64) {
        // the celestial wheel: each feast falls on its own anchor day,
        // the solstices and the quarter-days the year turns on - not a
        // fixed rotation, the sky's own marking
        const FEASTS: [(u64, &str); 4] = [
            (81, "May Day"),            // the first of May, the spring's quarter
            (172, "Midsummer"),         // the year's long day
            (263, "the Harvest Home"),  // the autumn's turn
            (355, "Yule"),              // midwinter's dark
        ];
        let Some(&(_, feast)) = FEASTS.iter().find(|(d, _)| tick % 365 == *d)
            else { return };
        // word of the feast travels the roads: a traveller whose round
        // was near comes for the crowd (#1073)
        self.draw_for_the_gathering(tick);
        self.log.push_ev_at_who(
            EvTag::new(Thread::Visitor, Valence::Positive, Sev::Notable),
            format!("t{}: {} - the village gathers at the market.", tick, feast),
            toast_spot(&self.world, self.market_food),
            None,
        );
        self.notices.push((
            "feast".into(),
            format!("{} - the village gathers at the market", feast),
            tick + 3,
        ));
        // the good stock comes out: bread, honey, whatever the stalls held
        let mut loaves = 0u32;
        for &be in &self.building_entities.clone() {
            if let Some(b) = self.world.get_component_mut::<Building>(be) {
                loaves += household::take_stock(b, "bread", 3);
                loaves += household::take_stock(b, "honey", 2);
            }
        }
        // and the barrel's tapped: whatever the village pours, the feast pours
        let mut poured = 0u32;
        for &be in &self.building_entities.clone() {
            if let Some(b) = self.world.get_component_mut::<Building>(be) {
                for d in ["ale", "beer", "wine", "spirits"] {
                    poured += household::take_stock(b, d, 2);
                }
            }
        }
        let gathered: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v)
                && self.world.get_component::<TownRole>(v).is_none()
                // the village's feast: the country keeps its own
                && self.site_of(v).is_none()
                && !self.shunned(v))
            .collect();
        // the village walks it in first: the column takes the streets
        // and the feast waits for it at the cross - unless something
        // else already has the streets
        let mut folk: Vec<Entity> = gathered.iter().copied()
            // a player walks themselves - the column doesn't take them
            .filter(|&v| !self.seats.iter().any(|s| s.player == v)).collect();
        folk.sort();
        let head = folk.first().and_then(|&v| self.world.get_component::<Pos>(v).copied())
            .map(|p| (p.x, p.y)).unwrap_or((self.village.0 as f32, self.village.1 as f32));
        let route = self.street_route(head, 30);
        if self.formation.is_none() && route.len() >= 4 {
            self.start_parade(folk, route, Some((feast, loaves, poured, tick)));
            return;
        }
        // something else has the streets today: feast anyway, but
        // whoever's marching keeps step - the feast doesn't pin them
        let free: Vec<Entity> = gathered.iter().copied()
            .filter(|&v| self.world.get_component::<villager::InFormation>(v).is_none())
            .collect();
        self.feast_day(tick, feast, &free, loaves, poured);
    }

    /// The feast itself: the column home, the ring forms round the
    /// cross, the good stock and the warmth go round - called the moment
    /// a feast parade finishes (or straight off, when it never marched)
    pub(in crate::session) fn feast_day(&mut self, tick: u64, feast: &str,
            gathered: &[Entity], loaves: u32, poured: u32) {
        // everyone comes to market for the day, in a ring round the cross
        let store = self.market_food;
        for &v in gathered { self.travels.retain(|t| t.who != v); }
        self.gather_feast(tick, gathered);
        // and somebody says so out loud for whoever's within earshot
        let voice = self.tavern.and_then(|t| self.world.get_component::<Building>(t))
            .and_then(|b| b.worker)
            .filter(|w| gathered.contains(w))
            .or_else(|| gathered.first().copied());
        if let Some(v) = voice {
            self.say(v, &format!("{} - the good stock's out!", feast), TALK_BUBBLE_SECS * 2);
        }
        for &v in gathered {
            // the day feeds the wants that never quite get fed
            desires::satisfy(&mut self.world, v, Want::Joy, tick + 90);
            desires::satisfy(&mut self.world, v, Want::Music, tick + 90);
            if loaves > 0 {
                desires::satisfy(&mut self.world, v, Want::GoodFood, tick + 30);
                if let Some(h) = self.world.get_component_mut::<Hunger>(v) { h.value = (h.value + 25.0).min(100.0); }
            }
            if poured > 0 {
                desires::satisfy(&mut self.world, v, Want::Drink, tick + 30);
            }
        }
        // the feast bonds: kept company at the table warms a tie - each
        // soul bumps into a few random others for a half heart a pair,
        // the same slow warming a spell in the pews gives
        let n = gathered.len();
        if n >= 2 {
            let mut bumps: Vec<(Entity, Entity)> = vec![];
            for &v in gathered {
                for _ in 0..3 {
                    let j = self.rng.below(n as i32) as usize;
                    if j < n { bumps.push((v, gathered[j])); }
                }
            }
            // and the players at the feast share the warmth: anyone
            // within sight of the market cross counts as a participant
            let mp = self.world.get_component::<Pos>(self.market_food)
                .map(|p| (p.x, p.y))
                .unwrap_or((self.village.0 as f32, self.village.1 as f32));
            for seat in &self.seats {
                let pe = seat.player;
                // standing near isn't attending: the dead and the held
                // can't join the feast, so they can't warm it either
                if villager::is_dead(&self.world, pe)
                    || crate::game::dungeon::unavailable(&self.world, pe) { continue; }
                let near = self.world.get_component::<Pos>(pe)
                    .map(|p| (p.x - mp.0).powi(2) + (p.y - mp.1).powi(2) <= 18.0 * 18.0)
                    .unwrap_or(false);
                if !near { continue; }
                for _ in 0..3 {
                    let j = self.rng.below(n as i32) as usize;
                    if j < n { bumps.push((pe, gathered[j])); }
                }
            }
            let mut seen: crate::engine::det::HashSet<(Entity, Entity)> = crate::engine::det::HashSet::default();
            for (a, b) in bumps {
                if a == b { continue; }
                let (lo, hi) = if a < b { (a, b) } else { (b, a) };
                if !seen.insert((lo, hi)) { continue; }
                relationships::record_interaction(&mut self.world, a, relationships::Interaction {
                    other: b, kind: relationships::InteractionKind::Conversation, intensity: 1.0 });
            }
        }
        // a few toasts surface what folk carry - the truth comes out in drink
        for &v in gathered.iter().take(4) {
            let line = self.world.get_component::<InnerLife>(v)
                .and_then(|l| l.grievances.first().map(|(p, _)| *p))
                .map(|p| match p {
                    desires::Problem::Blamed => "To my bread - whatever they say of it!",
                    desires::Problem::Taxed | desires::Problem::DeathDuty => "To what's left after the fifth!",
                    desires::Problem::Broke => "To better days!",
                    desires::Problem::Smelly => "To the fold - it smells of honest work!",
                    _ => "To the village!",
                }).unwrap_or("To the harvest!");
            self.say(v, line, CHAT_BUBBLE_SECS * 2);
        }
        // the tax collector is never invited but always shows - unless
        // he's forty miles out with the lord's gold on his saddle
        if let Some(c) = self.collector {
            if !villager::is_dead(&self.world, c)
                && !self.travels.iter().any(|t| t.who == c && matches!(t.goal, Goal::Remit(..))) {
                self.travels.retain(|t| t.who != c);
                self.send(c, Goal::HeadHome(store));
                self.say(c, "Uninvited. As ever.", CHAT_BUBBLE_SECS * 2);
            }
        }
    }

    /// The parade's done: a feast column hands its stock and its people
    /// to the ring; a victory column just breaks up at the cross
    pub(in crate::session) fn parade_resolved(&mut self,
            feast: Option<(&'static str, u32, u32, u64)>, members: &[Entity]) {
        if let Some((name, loaves, poured, day)) = feast {
            self.feast_day(day, name, members, loaves, poured);
        }
    }

    /// A carried demand sends the village out in the open: the column
    /// walks the streets and breaks up at the cross. Nothing else it
    /// could be doing is more worth the day
    pub(in crate::session) fn victory_parade(&mut self) {
        if self.formation.is_some() { return; }
        let mut folk: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v)
                && self.world.get_component::<TownRole>(v).is_none()
                // the village's day: the country keeps its own
                && self.site_of(v).is_none()
                && !self.shunned(v)
                // a player walks themselves - the column doesn't take them
                && !self.seats.iter().any(|s| s.player == v)
                && self.world.get_component::<Pos>(v).is_some())
            .collect();
        if folk.is_empty() { return; }
        folk.sort();
        let head = self.world.get_component::<Pos>(folk[0]).copied()
            .map(|p| (p.x, p.y)).unwrap_or((self.village.0 as f32, self.village.1 as f32));
        let route = self.street_route(head, 30);
        if route.len() < 4 { return; }
        self.start_parade(folk, route, None);
        self.log.push_ev(
            EvTag::new(Thread::Org, Valence::Positive, Sev::Major),
            format!(
                "t{}: the cause carries it - the village walks the win in.",
                self.tick
            ),
        );
    }
}

/// Is this villager a trader the guild's compact wants? The
    /// trade-vocation folk and the stall's trader count; raw producers,
    /// officials and the country folk don't - the guild's the village's
    pub(in crate::session) fn is_trader(world: &World, v: Entity) -> bool {
        if world.get_component::<crate::session::country::Outlander>(v).is_some() { return false; }
        world.get_component::<TownRole>(v)
            .map(|r| matches!(r, TownRole::Trader))
            .unwrap_or(false)
            || world.get_component::<Craft>(v)
                .map(|c| !matches!(c.vocation,
                    Vocation::Priest | Vocation::Knight | Vocation::Farmer
                    | Vocation::Shepherd | Vocation::Fisher))
                .unwrap_or(false)
    }

    /// Give a trader the Merchant the guild formation reads: their name,
    /// their purse plus the stock and tools the writs can strip, their
    /// stake marked exposed to the bailiff
    pub(in crate::session) fn tag_trader(world: &mut World, v: Entity, name: &str) {
        let wealth = world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0) + 60.0;
        world.add_component(v, crate::game::guilds::Merchant { name: name.into(),
            wealth, trade_routes: vec!["the market".into()], vulnerable: true });
    }

impl Game {
    /// The gaol's day: the held hold at their tile, and now and then
    /// one slips the cell - the door's honest, the wall's not
    pub(in crate::session) fn gaol_tick(&mut self, tick: u64) {
        let village_tile = self.village_gaol;
        let castle_tile = self.castle_gaol;
        for v in self.villagers.clone() {
            let held = self.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some();
            let castle_held = self.world.get_component::<crate::game::dungeon::CastleHeld>(v).is_some();
            if !held && !castle_held { continue; }
            let tile = if castle_held { castle_tile } else { village_tile };
            // the escape: a small seeded roll slips them free - the door's
            // honest and the wall's not, and a release's effort ends with it
            if self.rng.below(100) < 2 {
                self.world.remove_component::<crate::game::dungeon::Imprisoned>(v);
                self.world.remove_component::<crate::game::dungeon::CastleHeld>(v);
                self.world.remove_component::<crate::game::dungeon::ReleaseCampaign>(v);
                let name = self.name_of(v);
                self.log_entity_ev(EvTag::new(Thread::Law, Valence::Neutral, Sev::Notable), v, format!("t{}: {} slips the cell and is gone - the road takes whoever the cell couldn't.", tick, name));
                self.toasts.push(("one slipped the cell".into(), tick + 40));
                if let Some(home) = household::home_of(&self.world, v).or(Some(self.market_food)) {
                    if let Some(p) = self.world.get_component::<Pos>(home).copied() {
                        self.send_far(v, (p.x.round() as i32, p.y.round() as i32), Goal::HeadHome(home));
                    }
                }
                continue;
            }
            // pinned at the gaol: the Pos holds where the escort left them
            if let Some(t) = tile {
                if let Some(p) = self.world.get_component_mut::<Pos>(v) {
                    p.x = t.0 as f32 + 0.5; p.y = t.1 as f32 + 0.5;
                }
            }
        }
    }

    /// The stocks' day: a body pinned there holds its spot, gets fed by
    /// the village watching, and walks free when the sentence runs out
    pub(in crate::session) fn stocks_tick(&mut self, tick: u64) {
        let Some(tile) = self.stocks_tile else { return };
        for v in self.villagers.clone() {
            let Some(u) = self.world.get_component::<villager::InStocks>(v).map(|s| s.until) else { continue };
            if tick >= u {
                self.world.remove_component::<villager::InStocks>(v);
                self.log_entity_ev(
                    EvTag::new(Thread::Law, Valence::Neutral, Sev::Notable),
                    v,
                    format!(
                        "t{}: {} walks free of the stocks - the sentence's run, the shame's not.",
                        tick,
                        self.name_of(v)
                    ),
                );
                continue;
            }
            // pinned and fed: the body stays, the belly's seen to - the
            // village's theater doesn't starve its player
            if let Some(p) = self.world.get_component_mut::<Pos>(v) {
                p.x = tile.0 as f32 + 0.5; p.y = tile.1 as f32 + 0.5;
            }
            if let Some(h) = self.world.get_component_mut::<villager::Hunger>(v) {
                h.value = 100.0;
            }
        }
    }
}

impl Game {
    /// A caught wrong answered by the book: the fine takes the purse,
    /// the dungeon takes the body, the stocks take the shame - routed
    /// through whatever enforcement's already standing. The seam
    /// #653's sheriffs call when they arrive.
    #[allow(dead_code)]
    pub(in crate::session) fn apply_penalty(&mut self, site: usize, who: Entity, crime: &str, tick: u64) {
        let _law = self.law_book.law_for(site);
        match crime {
            "theft" | "smuggling" => {
                if let Some(p) = self.world.get_component_mut::<Purse>(who) {
                    let cut = (p.gold * 0.25).min(p.gold);
                    p.gold -= cut;
                    self.log_entity_ev(
                        EvTag::new(Thread::Law, Valence::Negative, Sev::Notable),
                        who,
                        format!(
                            "t{}: fined {:.0} gold under the law - {}.",
                            tick,
                            cut,
                            self.name_of(who)
                        ),
                    );
                }
            }
            "assembly" => {
                // over the cap the law calls it a mob: the stocks for a
                // ringleader, a fine for the rest
                self.apply_sanction(who, crate::game::sanctions::Sanction::Stocks, tick);
            }
            "poaching" => {
                // taken in the chase: a fine for the purse, and the
                // stocks if the purse can't pay
                if let Some(p) = self.world.get_component_mut::<Purse>(who) {
                    let cut = (p.gold * 0.4).min(p.gold);
                    p.gold -= cut;
                    if cut < 1.0 { self.apply_sanction(who, crate::game::sanctions::Sanction::Stocks, tick); }
                }
            }
            "defiance" => {
                // a writ defied: the dungeon's own answer
                self.world.add_component(who, crate::game::dungeon::Imprisoned {
                    since: tick, charge: crime.into() });
                if let Some(g) = self.village_gaol.or(self.castle_gaol) {
                    self.send_far(who, g, Goal::GoGaol);
                }
            }
            _ => {}
        }
    }
}

impl Game {
    /// The cult's day: a strict orthodoxy, a charismatic head, and a
    /// flock cut off tip the parish into a cult - the tithe drains the
    /// members to the leader, and a congregation that relaxes comes
    /// back to itself
    pub(in crate::session) fn cult_tick(&mut self, tick: u64) {
        let ce = self.congregation;
        let (mut cult, orthodoxy, leader, members) = {
            let c = self.world.get_component::<crate::sim::religion::Congregation>(ce)
                .expect("the parish has a congregation");
            (c.cultish, c.orthodoxy, c.leader, c.members.clone())
        };
        // the three factors: strict, charismatic, cut off. Charisma's a
        // leader who holds the flock's hearts; isolation's the flock's
        // bonds staying inside itself
        let charismatic = leader.map(|l| {
            let bonds = self.world.query::<crate::sim::psychology::Bond>().iter()
                .filter(|&&e| self.world.get_component::<crate::sim::psychology::Bond>(e)
                    .map(|b| b.target == l).unwrap_or(false)).count();
            bonds >= 4
        }).unwrap_or(false);
        let isolated = {
            let inside: usize = self.world.query::<crate::sim::psychology::Bond>().iter()
                .filter(|&&e| members.contains(&e)
                    && self.world.get_component::<crate::sim::psychology::Bond>(e)
                        .map(|b| members.contains(&b.target)).unwrap_or(false)).count();
            let outside: usize = self.world.query::<crate::sim::psychology::Bond>().iter()
                .filter(|&&e| members.contains(&e)
                    && self.world.get_component::<crate::sim::psychology::Bond>(e)
                        .map(|b| !members.contains(&b.target)).unwrap_or(false)).count();
            inside as f32 > (outside as f32) * 1.5
        };
        if !cult && orthodoxy > 0.7 && charismatic && isolated {
            cult = true;
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Notable), format!("t{}: the parish turns inward - the flock follows its shepherd past the village's edge.", tick));
            for m in &members { self.world.add_component(*m, crate::sim::religion::Cultish); }
        } else if cult && orthodoxy < 0.5 {
            // institutionalized: the fire burns out, the flock comes
            // back to the village's own fold
            cult = false;
            for m in &members {
                self.world
                    .remove_component::<crate::sim::religion::Cultish>(*m);
            }
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                format!(
                    "t{}: the fire burns out - the parish is the village's own again.",
                    tick
                ),
            );
        }
        self.world.get_component_mut::<crate::sim::religion::Congregation>(ce)
            .map(|c| c.cultish = cult);
        // the tithe: the members' gold flows to the shepherd while the
        // cult holds - a quarter of what each has
        if cult && tick % 7 == 0 {
            let take = members.iter().filter_map(|&m| {
                let p = self.world.get_component_mut::<Purse>(m)?;
                let cut = p.gold * 0.25;
                p.gold -= cut; Some(cut)
            }).sum::<f32>();
            if let Some(l) = leader {
                if let Some(p) = self.world.get_component_mut::<Purse>(l) { p.gold += take; }
            }
        }
    }
}

impl Game {
    /// The relic's day: the pilgrims leave their pennies at whichever
    /// shrine holds a saint, and once in a long while a denunciation
    /// crashes every relic's market at once
    pub(in crate::session) fn relic_tick(&mut self, tick: u64) {
        for e in self.world.query::<crate::game::relic::Relic>() {
            let (rev, exposed) = self.world.get_component::<crate::game::relic::Relic>(e)
                .map(|r| (r.revenue, r.exposed)).unwrap_or((0.0, true));
            if !exposed {
                if let Some(p) = self.world.get_component_mut::<Purse>(e) { p.gold += rev * 0.05; }
            }
        }
        // the exposé's a rare morning - the market's faith crashes all
        // at once, the Reformation's own day
        if self.rng.below(50000) == 0 {
            crate::game::relic::expose_all(&mut self.world);
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Negative, Sev::Major),
                format!(
                    "t{}: a denunciation! The relics are exposed - the pilgrims' faith empties.",
                    tick
                ),
            );
        }
    }
}

impl Game {
    /// The pick a dilemma carries home: the choice writes its debt and
    /// the debt lands when its day comes - someone remembers what you
    /// carried out first
    pub(in crate::session) fn dilemma_tick(&mut self, tick: u64) {
        let mut fired: Vec<usize> = vec![];
        for (i, d) in self.dilemma_debts.iter().enumerate() {
            if tick < d.due { continue }
            // the witness remembers the pick - and the victim's owed the
            // weight of it, for good or ill. The answering soul carries
            // it on their character too: sparing the innocent grows the
            // merciful, laying the weight on hardens them (#950)
            let act = if d.weight > 0.4 {
                crate::game::virtue::MoralAct::LaidTheWeight
            } else {
                crate::game::virtue::MoralAct::SparedTheInnocent
            };
            crate::game::virtue::witness(&mut self.world, d.witness, act, 1.0);
            crate::sim::psychology::record_episode(&mut self.world, d.witness,
                crate::sim::psychology::Episode { with: d.victim, kind: crate::sim::psychology::EpisodeKind::Remembrance });
            if d.weight > 0.4 {
                desires::add_grievance(
                    &mut self.world,
                    d.victim,
                    desires::Problem::Loss,
                    d.due + 30,
                );
            }
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Negative, Sev::Notable),
                format!(
                    "t{}: the choice comes home - someone remembers what was carried out first.",
                    tick
                ),
            );
            fired.push(i);
        }
        for i in fired.into_iter().rev() { self.dilemma_debts.remove(i); }
    }

    /// The ordinary folk's hard moments (#950): when a bonded neighbor
    /// goes hungry under a roof that isn't, the better-off faces the
    /// share/keep fork. The traits they carry lean the pick, their
    /// settled character weights it - a kind soul shares more often
    /// than a stingy one, and each choice bends the soul a little. A
    /// rare thing: one fork a day out of every few hundred villagers
    pub(in crate::session) fn virtue_tick(&mut self, tick: u64) {
        use crate::game::virtue::{self, MoralAct, Virtue};
        use crate::game::{desires, relationships, traits};
        for &v in &self.villagers {
            if crate::session::routine::mix(v, tick) % 241 != 0 { continue; }
            // nothing to give if the pot's bare at home
            if desires::problems_of(&self.world, v, tick).contains(&desires::Problem::Hungry) { continue; }
            // a bonded neighbor who IS hungry: the fork needs a face
            let mut needy = None;
            for r in relationships::bonds_of(&self.world, v) {
                let Some(rel) = self.world.get_component::<relationships::Relationship>(r) else { continue };
                if rel.strength < 0.4 { continue; }
                let o = if rel.from == v { rel.other } else { rel.from };
                if o == v || !self.villagers.contains(&o) { continue; }
                if desires::problems_of(&self.world, o, tick).contains(&desires::Problem::Hungry) { needy = Some(o); break; }
            }
            let Some(o) = needy else { continue };
            // the pick: charity pulls to share, prudence to keep, and the
            // traits lean - a kind soul's table is open more often
            let (charity, prudence) = self.world.get_component::<virtue::Virtues>(v)
                .map(|vs| (vs.get(Virtue::Charity), vs.get(Virtue::Prudence)))
                .unwrap_or((0.5, 0.5));
            let lean = self.world.get_component::<traits::Personality>(v)
                .map(|p| p.traits.iter().map(|t| match t {
                    traits::Trait::Kind => 0.15,
                    traits::Trait::Stingy => -0.15,
                    traits::Trait::Pious => 0.05,
                    _ => 0.0,
                }).sum::<f32>()).unwrap_or(0.0);
            let roll = (crate::session::routine::mix(v, tick ^ 0x5EED) % 1000) as f32 / 1000.0 - 0.5;
            if charity - prudence + lean + roll * 0.1 > 0.0 {
                // a place at their table: the want fed a week, a warm
                // memory between them, the soul bent toward charity
                desires::satisfy(&mut self.world, o, desires::Want::GoodFood, tick + 7);
                crate::sim::psychology::record_episode(&mut self.world, o,
                    crate::sim::psychology::Episode { with: v, kind: crate::sim::psychology::EpisodeKind::Warmth });
                virtue::witness(&mut self.world, v, MoralAct::SharedAid, 1.0);
                // whoever stood near enough to see carries the smaller share
                if let Some(at) = self.world.get_component::<Pos>(v).map(|p| (p.x, p.y)) {
                    let near: Vec<Entity> = self.villagers.iter().copied().filter(|&w| {
                        w != v && w != o && self.world.get_component::<Pos>(w)
                            .map(|p| (p.x - at.0).abs() + (p.y - at.1).abs() <= 6.0).unwrap_or(false)
                    }).collect();
                    for w in near { virtue::witness(&mut self.world, w, MoralAct::WitnessedKindness, 1.0); }
                }
            } else {
                virtue::witness(&mut self.world, v, MoralAct::RefusedAid, 1.0);
            }
        }
    }

    /// The shared story: folk who witnessed the same thing carry a bit
    /// between them, and the retelling warms the pair - an untold bit
    /// fades inside the month
    pub(in crate::session) fn bits_tick(&mut self, tick: u64) {
        use crate::sim::psychology::{AttachmentHistory, SharedBit, SharedBits};
        // what everyone carries: group each villager's memories by the
        // thing witnessed - two names on the same memory is a bit
        let mut by_memory: crate::engine::det::HashMap<(Entity, crate::sim::psychology::EpisodeKind), Vec<Entity>> =
            crate::engine::det::HashMap::default();
        for &v in &self.villagers {
            let Some(h) = self.world.get_component::<AttachmentHistory>(v) else { continue };
            // the fresh end of memory is what folk still trade in -
            // scanning the whole of a life every day would drag. And a
            // Warmth beat is a bond moment, not a witnessed thing - letting
            // retellings seed new bits would run the loop away
            for e in h.episodes.iter().rev().take(40) {
                if matches!(e.kind, crate::sim::psychology::EpisodeKind::Warmth) { continue; }
                by_memory.entry((e.with, e.kind)).or_default().push(v);
            }
        }
        for ((subject, kind), mut holders) in by_memory {
            if holders.len() < 2 { continue; }
            holders.sort_unstable();
            for i in 0..holders.len() {
                for j in (i + 1)..holders.len() {
                    let (a, b) = (holders[i], holders[j]);
                    for (x, y) in [(a, b), (b, a)] {
                        let mut sb = self.world.get_component::<SharedBits>(x)
                            .map(|s| SharedBits { bits: s.bits.clone() }).unwrap_or_default();
                        if !sb.bits.iter().any(|bit| bit.partner == y
                            && bit.subject == subject && bit.kind == kind) {
                            sb.bits.push(SharedBit {
                                partner: y, subject, kind, told: false, born: tick });
                            self.world.add_component(x, sb);
                        }
                    }
                }
            }
        }
        // the telling: when next the pair meet they trade it - a day's
        // grace, then the small warmth lands; a bit never told fades
        const BIT_TTL: u64 = 30;
        for &v in &self.villagers {
            let Some(bits) = self.world.get_component::<SharedBits>(v)
                .map(|s| s.bits.clone()) else { continue };
            let mut keep = vec![];
            for mut bit in bits {
                if bit.told {
                    // a story once told is done its work - keeping every
                    // one forever would drag the daily scan
                    continue;
                }
                if tick >= bit.born + 1
                    && !crate::game::villager::is_dead(&self.world, bit.partner) {
                    // a warmth beat, not a new relationship entity -
                    // spawning one inside the daily tick would shift the
                    // entity order and starve the store
                    crate::sim::psychology::record_episode(&mut self.world, v,
                        crate::sim::psychology::Episode { with: bit.partner,
                            kind: crate::sim::psychology::EpisodeKind::Warmth });
                    bit.told = true;
                } else if tick >= bit.born + BIT_TTL {
                    continue;
                }
                keep.push(bit);
            }
            self.world.add_component(v, SharedBits { bits: keep });
        }
    }

    /// The hidden school's own mirror: the flag puts the cover on the
    /// org - its own trainings run slower but no shutter reaches them
    pub(in crate::session) fn hidden_venue_tick(&mut self, tick: u64) {
        if tick % 90 != 5 { return; }
        let has = self.world.get_component::<crate::game::org_skills::HiddenVenue>(self.org).is_some();
        if self.hidden_venue && !has {
            self.world.add_component(self.org, crate::game::org_skills::HiddenVenue);
        } else if !self.hidden_venue && has {
            self.world.remove_component::<crate::game::org_skills::HiddenVenue>(self.org);
        }
    }

    /// The channels' own work: the org's own practiced reach - the
    /// pulpit's borrowed reach moderated by the fold's own orthodoxy,
    /// the press slow but its own, the signal-net fast and leaky, the
    /// symbol everywhere and thin. The levels accrue by the work they
    /// do, and the fast channel's leaks feed the castle's own eyes
    pub(in crate::session) fn channel_tick(&mut self, tick: u64) {
        if tick % 30 != 21 { return; }
        // the channels' own work accrues on the org's own members - the
        // leader and the coordinators, the ones who carry the word
        let members = self.world.get_component::<crate::sim::community::Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        let orth = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation)
            .map(|c| c.orthodoxy).unwrap_or(0.3);
        let mut leaky = false;
        for m in members {
            let Some(mut o) = self.world.get_component::<crate::game::org_skills::Organizing>(m).cloned() else { continue };
            let org_work = o.get(crate::game::org_skills::OrgSkill::Turnout)
                + o.get(crate::game::org_skills::OrgSkill::Agitation);
            o.bump(crate::game::org_skills::OrgSkill::Pulpit, org_work * 0.003 * (1.0 + orth));
            o.bump(crate::game::org_skills::OrgSkill::Press, org_work * 0.001);
            o.bump(crate::game::org_skills::OrgSkill::SignalNet, org_work * 0.004);
            o.bump(crate::game::org_skills::OrgSkill::Symbol, org_work * 0.006);
            if o.get(crate::game::org_skills::OrgSkill::SignalNet) > 0.5 { leaky = true; }
            self.world.add_component(m, o);
        }
        // the signal-net's own leak: the fast channel's relays reach
        // the castle's own ears
        if leaky {
            for site in &self.sites {
                let Some(l) = site.lord else { continue };
                if let Some(w) = self.world.get_component_mut::<Watcher>(l) {
                    w.paranoia = (w.paranoia + 0.01).min(1.0);
                }
            }
        }
    }

    /// The lord's own reach: the authority over the village - one
    /// strength, derived monthly from the flow his writ actually
    /// controls, the village still standing with him, and the drag
    /// his own style costs. Below the bar the writ runs out and the
    /// strongest claim left standing fills the vacuum; the crown
    /// answers late and dear, at a reach the village can contest
    /// again (#971)
    pub(in crate::session) fn authority_tick(&mut self, tick: u64) {
        if tick % 30 != 19 { return; }
        // a dissolved writ answers only to the crown's own clock: the
        // reimposition rides in late and dear - a new bailiff, a
        // harsher hand, and a reach the village broke once already
        if let Some(gone) = self.lord_gone {
            if tick >= gone + RETURN_DAYS {
                self.lord_gone = None;
                self.vacuum_filler = None;
                self.lord_strength = RETURN_STRENGTH;
                self.lord_flow = RETURN_FLOW;
                self.repression_debt += 2;
                self.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Notable), format!("t{}: a new bailiff rides in under the old writ - the reach reimposed at a price, and the village remembers how it broke.", tick));
            }
            return;
        }
        // the claim's own reality, re-derived: the loyalty is the
        // village's own contentment; the drag is the nearest lord's
        // own cruelty and vanity; the flow is what the road let
        // through since the last accounting
        let living: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v))
            .filter(|&v| self.world.get_component::<crate::session::country::Outlander>(v).is_none())
            .collect();
        let contented = living.iter().filter(|&&v|
            self.world.get_component::<crate::game::desires::InnerLife>(v)
                .map(|l| l.grievances.iter().all(|(_, u)| *u <= tick)).unwrap_or(true)).count();
        let loyalty = if living.is_empty() { 0.5 } else { contented as f32 / living.len() as f32 };
        let drag = self.sites.iter()
            .filter(|s| matches!(s.kind, crate::session::country::SiteKind::Castle))
            .map(|s| s.persona.cruelty * 0.6 + s.persona.vanity * 0.4)
            .fold(0.0f32, f32::max);
        let derived = claim_strength(self.lord_flow, loyalty, drag);
        self.lord_strength += (derived - self.lord_strength) * 0.12;
        // the dissolution's grind: the org's own capacity still
        // weakens the reach - the struggle over the rate, not just
        // the taking
        let capacity = self.world.get_component::<crate::sim::organized_power::OrganizedPower>(self.org)
            .map(|o| o.capacity).unwrap_or(0.0);
        self.lord_strength = (self.lord_strength - capacity * 0.01).max(0.0);
        // and the window moves: a flow counted last month fades
        self.lord_flow *= 0.97;
        // the vacuum: below the bar the writ runs out, and the
        // strongest claim left standing takes the village's own
        // answer - the cause if it's the strongest, a faction if not
        if self.lord_strength < DISSOLVE_AT {
            self.lord_gone = Some(tick);
            let filler = self.vacuum_pick();
            self.vacuum_filler = filler;
            let name = filler.and_then(|f| self.world.get_component::<crate::sim::community::Organization>(f)
                .map(|o| o.name.clone())).unwrap_or_else(|| "no one".into());
            self.log.push_ev(EvTag::new(Thread::Law, Valence::Positive, Sev::Major), format!("t{}: the writ runs out - the lord's arm no longer reaches the village, and the village answers to {}.", tick, name));
            return;
        }
        // and the self-organization: the lord's own low reach feeds
        // the village's own answer - the thesis's own loop
        if self.lord_strength < 0.4 {
            if let Some(o) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(self.org) {
                o.adjust(0.01);
            }
            if let Some(org) = self.world.get_component_mut::<crate::sim::community::Organization>(self.org) {
                org.resources += 2.0;
            }
        }
    }

    /// Who the vacuum fills to: the cause stands on its own capacity;
    /// a faction's pull is the influence its people actually carry,
    /// discounted for the body everyone already belongs to - the
    /// parish holding every soul pulls no harder for it. The
    /// strongest claim takes the village - no favorites (#971)
    fn vacuum_pick(&self) -> Option<Entity> {
        let mut best: Option<(Entity, f32)> = None;
        let pull_of = |e: Entity| -> f32 {
            self.world.get_component::<crate::sim::factions::Faction>(e).map(|f| {
                let members = crate::sim::factions::members_of(&self.world, f);
                if members.is_empty() { return 0.0; }
                let mean = crate::sim::factions::pooled_influence(&self.world, &members)
                    / members.len() as f32;
                mean * (members.len() as f32 / 20.0).sqrt()
            }).unwrap_or(0.0)
        };
        for &f in &self.factions {
            let pull = pull_of(f);
            if pull > best.map(|(_, p)| p).unwrap_or(0.0) { best = Some((f, pull)); }
        }
        let cause = self.world.get_component::<crate::sim::organized_power::OrganizedPower>(self.org)
            .map(|o| o.capacity).unwrap_or(0.0);
        if cause >= best.map(|(_, p)| p).unwrap_or(0.0) && cause > 0.0 {
            best = Some((self.org, cause));
        }
        best.map(|(e, _)| e)
    }

    /// The teller's own arc: the drama's own scheduling - the setup
    /// looms, the escalation builds, the climax fires, the resolution
    /// closes. The persona's own on/off cycle paces the threat to
    /// the village's own state
    pub(in crate::session) fn storyteller_tick(&mut self, tick: u64) {
        if tick % 30 != 15 { return; }
        // the persona's own cycle: the teller's on, then off - the
        // threat's own pacing, not a continuous pressure
        if self.storyteller.is_none() && self.arc_phase == 0 {
            self.storyteller = Some(tick);
            self.arc_phase = 1;
            self.log.push_ev(
                EvTag::new(Thread::Other, Valence::Neutral, Sev::Minor),
                format!(
                    "t{}: the teller's own setup - a threat looms on the horizon.",
                    tick
                ),
            );
            return;
        }
        // the arc's own phases: the setup escalates, the climax
        // fires, the resolution closes
        match self.arc_phase {
            1 => { // the escalation's own build
                self.arc_phase = 2;
                if let Some(o) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(self.org) {
                    o.adjust(0.05);
                }
                self.log.push_ev(
                    EvTag::new(Thread::Other, Valence::Neutral, Sev::Minor),
                    format!(
                        "t{}: the arc's own escalation - the pressure's own build.",
                        tick
                    ),
                );
            }
            2 => { // the climax's own firing
                self.arc_phase = 3;
                self.log.push_ev(
                    EvTag::new(Thread::Other, Valence::Neutral, Sev::Minor),
                    format!("t{}: the arc's own climax - the event's own firing.", tick),
                );
            }
            _ => { // the resolution's own close - the teller rests
                if self.arc_phase == 3 {
                    self.arc_phase = 0;
                    self.storyteller = None;
                    self.log.push_ev(EvTag::new(Thread::Other, Valence::Neutral, Sev::Minor), format!("t{}: the arc's own resolution - the story closes, and the teller rests.", tick));
                }
            }
        }
    }

    /// The calendar's own rhythm: the dry-spell counter accrues, and
    /// at the long stretch the drought-breaker fires - the payoff
    /// that keeps the calendar's own promise. A win fires the
    /// celebration - the conditioning's own juice
    pub(in crate::session) fn event_pace_tick(&mut self, tick: u64) {
        if tick % 30 != 5 { return; }
        // the drought's own accrual: the days since the last event
        self.drought += 1;
        // a win's own moment: the comeback's own firing or the
        // celebration's own juice - the dry spell's own answer
        if self.comeback.is_some() && self.celebration.is_none() {
            self.celebration = Some(tick);
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: the village's own celebration - the win's own moment, the drought's own answer.", tick));
        }
        // the drought-breaker: at the long stretch the calendar's own
        // payoff fires - the "end on success" that keeps the promise
        if self.drought >= 60 {
            self.drought = 0;
            if let Some(o) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(self.org) {
                o.adjust(0.1);
            }
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Positive, Sev::Notable),
                format!(
                    "t{}: the drought-breaker lands - the long stretch ends on a win.",
                    tick
                ),
            );
        }
    }

    /// The stranger's own tongue: the outlander's own speech glosses
    /// itself over the contact - the ear's own learning accrues, and
    /// the words join the player's own ledger
    pub(in crate::session) fn language_tick(&mut self, tick: u64) {
        if tick % 30 != 9 { return; }
        // the contact's own learning: an outlander in the village's
        // own ear accrues the fluency - the words gloss over time
        let outlanders = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v))
            .filter(|v| self.world.get_component::<crate::session::country::Outlander>(*v).is_some())
            .count();
        if outlanders == 0 { return; }
        self.fluent = (self.fluent + outlanders as f32 * 0.005).min(1.0);
        // and the words join the ledger: the fluency's own threshold
        // marks a word learned
        let words = (self.fluent * 20.0) as usize;
        if words > self.learned_words {
            self.learned_words = words;
            self.log.push_ev(
                EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Minor),
                format!(
                    "t{}: a word settles in the ear - the outlander's own speech glossing itself.",
                    tick
                ),
            );
        }
    }

    /// The mob's own rising: the grievance's own boil-over - the
    /// pitchforks and the torches march on the seats of power, the
    /// soldiers ride in after, and the peasants always lose. The
    /// bystanders' own doors bar; the caught are taken or killed
    pub(in crate::session) fn uprising_tick(&mut self, tick: u64) {
        if tick % 60 != 55 { return; }
        if self.uprising.is_some() { return; }
        // the boil-over's own line: the villagers' own grievances past
        // the threshold - the legal channel's own failure
        let grieved = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v))
            .filter(|&v| self.world.get_component::<crate::game::desires::InnerLife>(v)
                .map(|l| !l.grievances.is_empty()).unwrap_or(false))
            .count();
        if grieved < 8 { return; }
        self.uprising = Some(tick);
        // the soldiers' own answer: the lord's own arm rides in - the
        // mob's own suppression, the peasants always lose. The
        // repression's own debt lands on the next writ
        self.repression_debt += 1;
        // a rising is the writ contested in the open: the reach thins
        // for having to answer it (#971)
        self.lord_strength = (self.lord_strength - 0.1).max(0.0);
        // and the mob's own cost: the caught are taken or killed -
        // a few of the grieved go to the castle or the grave
        let mut taken = 0;
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            if taken >= 2 { break; }
            if self.world.get_component::<crate::game::desires::InnerLife>(v)
                .map(|l| !l.grievances.is_empty()).unwrap_or(false) {
                self.world.add_component(v, crate::game::dungeon::Imprisoned { since: tick, charge: "the rising".into() });
                taken += 1;
            }
        }
        self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Major), format!("t{}: the mob rises - the pitchforks and the torches, and the soldiers ride in after. The peasants never win.", tick));
    }

    /// The road's own caravan: the trade between two sites - the
    /// price's own differential worth the walking, the bandit's own
    /// risk on the way, and the far market's own pay
    pub(in crate::session) fn caravan_tick(&mut self, tick: u64) {
        if tick % 60 != 33 { return; }
        if self.caravan.is_some() { return; }
        // the caravan's own forming: two sites to run the route - the
        // price's own differential worth the walking
        let sites = self.sites.iter().filter(|s| s.lord.is_some()).count();
        if sites < 2 { return; }
        self.caravan = Some(tick);
        // the bandit's own risk: a watching castle's own paranoia
        // catches the caravan - the road's own danger
        let bandits = self.sites.iter().any(|s| s.lord
            .and_then(|l| self.world.get_component::<Watcher>(l))
            .map(|w| w.paranoia >= 0.7).unwrap_or(false));
        if bandits {
            self.log.push_ev(EvTag::new(Thread::Economy, Valence::Negative, Sev::Notable), format!("t{}: the caravan's taken on the road - the bandits' own risk, the goods never reached the far market.", tick));
            return;
        }
        // the far market's own pay: the caravan's own goods sell at
        // the other end - the org's own resources lifted
        if let Some(org) = self.world.get_component_mut::<crate::sim::community::Organization>(self.org) {
            org.resources += 15.0;
        }
        self.log.push_ev(
            EvTag::new(Thread::Economy, Valence::Positive, Sev::Notable),
            format!(
                "t{}: the caravan reaches the far market - the road's own trade, the goods sold.",
                tick
            ),
        );
    }

    /// The horizon's own war: the conflict between two sites - the
    /// levies called, the battle's own light attrition (the bigger
    /// host wins), the loser's tribute paid, and the levied
    /// villagers' own grievance
    pub(in crate::session) fn war_tick(&mut self, tick: u64) {
        if tick % 60 != 51 { return; }
        if self.war.is_some() { return; }
        // the war's own seed: two castle sites host knights - the
        // conflict forms between the strong
        let castles: Vec<_> = self.sites.iter().enumerate()
            .filter(|(_, s)| s.lord.is_some()).map(|(i, _)| i).collect();
        if castles.len() < 2 { return; }
        self.war = Some(tick);
        // the battle's own attrition: the bigger host wins, the
        // loser's own standing pays the tribute
        for site in &self.sites {
            let Some(l) = site.lord else { continue };
            if let Some(w) = self.world.get_component_mut::<Watcher>(l) {
                w.legitimacy = (w.legitimacy - 0.1).max(0.0);
            }
        }
        // and the levy's own cost: the villagers called carry the
        // grievance - the due's own taking
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            if let Some(l) = self.world.get_component_mut::<crate::game::desires::InnerLife>(v) {
                l.grievances.push((crate::game::desires::Problem::Loss, tick + 60));
            }
        }
        self.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Notable), format!("t{}: the levies are called - the lords' own conflict, and the villagers' own due paid.", tick));
    }

    /// The village's own comings and goings: the hunger's own push
    /// drives a household off the map (the villagers' own shrink),
    /// and the prosperity's own pull draws a stranger (the villagers'
    /// own swell). A household leaving's a log-worthy event - the
    /// mirror of a stranger arriving
    pub(in crate::session) fn migration_tick(&mut self, tick: u64) {
        if tick % 30 != 27 { return; }
        // the hunger's own push: at the village's own low ebb a
        // household walks off the map - the villagers' own shrink
        let starving = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v))
            .filter_map(|v| self.world.get_component::<crate::session::Hunger>(v))
            .filter(|h| h.value >= 0.8).count();
        if starving >= 5 && self.villagers.len() > 10 {
            if let Some(gone) = self.villagers.pop() {
                // the leaver's own going: dead to the village - the
                // roll keeps them, the map doesn't
                self.world.add_component(gone, crate::game::villager::Dead);
                self.world.remove_component::<crate::game::villager::Villager>(gone);
            }
            self.emigration += 1;
            self.log.push_ev(EvTag::new(Thread::Economy, Valence::Negative, Sev::Notable), format!("t{}: a household walks off the map - the hunger's own push takes them to the next town.", tick));
            return;
        }
        // the prosperity's own pull: the stores' own plenty draws a
        // stranger - the villagers' own swell (an arrival counts as
        // the emigration's own negative - the swell's the mirror)
        let plenty = self.soil_quality >= 0.7 && starving == 0;
        if plenty && tick % 90 == 27 {
            self.emigration = self.emigration.saturating_sub(1);
            self.log.push_ev(
                EvTag::new(Thread::Visitor, Valence::Positive, Sev::Minor),
                format!(
                    "t{}: a stranger comes down the road - the stores' own plenty draws them in.",
                    tick
                ),
            );
        }
    }

    /// The fold's own hardness: the extremism's own confluence - the
    /// weak bonds, the high orthodoxy, the cultish seed - drifts the
    /// absolutism up; at the high ebb the fold's own hardness shows,
    /// the orthodoxy's own line and the separation's own pressure
    pub(in crate::session) fn fundamentalism_tick(&mut self, tick: u64) {
        if tick % 30 != 26 { return; }
        // the risk's own confluence: the weak bonds + the high
        // orthodoxy + the cultish seed drifts the absolutism up
        let weak_bonds = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v))
            .filter_map(|v| self.world.get_component::<crate::sim::psychology::Bond>(v))
            .filter(|b| b.security < 0.4).count();
        let Some(c) = self.world.get_component_mut::<crate::sim::religion::Congregation>(self.congregation) else { return };
        if c.orthodoxy >= 0.6 && (c.cultish || weak_bonds >= 3) {
            c.absolutism = (c.absolutism + 0.01).min(1.0);
        }
        // at the high ebb the fold's own hardness shows - the
        // orthodoxy's own line hardens and the separation's own
        // pressure rises on the radicalism
        if c.absolutism >= 0.6 {
            c.orthodoxy = (c.orthodoxy + 0.005).min(1.0);
            self.separation_pressure = (self.separation_pressure + 0.005).min(1.0);
        }
    }

    /// The misery's own answer: the windfall that favors the losing -
    /// at the village's own low ebb a comeback event fires, the
    /// bountiful harvest or the relic found, the purse and the stores
    /// lifted - a story's own relief, not a fix
    pub(in crate::session) fn comeback_tick(&mut self, tick: u64) {
        if tick % 60 != 44 { return; }
        if self.comeback.is_some() { return; }
        // the misery's own line: the capacity's own low ebb fires the
        // windfall - the rubber band's own answer
        let misery = self.world.get_component::<crate::sim::organized_power::OrganizedPower>(self.org)
            .map(|o| o.capacity < 0.2).unwrap_or(false);
        if !misery { return; }
        self.comeback = Some(tick);
        // the windfall's own relief: the purse's own lift and the
        // stores' own answer - a story's own turn, not a fix
        if let Some(o) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(self.org) {
            o.adjust(0.2);
        }
        if let Some(org) = self.world.get_component_mut::<crate::sim::community::Organization>(self.org) {
            org.resources += 10.0;
        }
        self.log.push_ev(EvTag::new(Thread::Economy, Valence::Positive, Sev::Notable), format!("t{}: the windfall's own turn - a bountiful harvest, and the cause's own ebb lifts.", tick));
    }

    /// The fiber's own making: the weaver's own skill scales the
    /// cloth's own quality - the same fiber plus the skill spans the
    /// ten-times price range. And the good cloth comforts the wearer
    /// - the texture's own ease
    pub(in crate::session) fn textile_tick(&mut self, tick: u64) {
        if tick % 30 != 22 { return; }
        // the weaver's own skill scales the cloth's own quality - the
        // same fiber plus the skill spans the price's own range
        let skill = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v))
            .filter_map(|v| self.world.get_component::<crate::game::vocation::Craft>(v))
            .filter(|c| c.vocation == crate::game::vocation::Vocation::Weaver)
            .map(|c| c.skill).fold(0.0f32, f32::max);
        if skill > 0.0 {
            self.cloth_quality = (self.cloth_quality + skill * 0.01).min(1.0);
        }
        // and the good cloth comforts the wearer - the texture's own
        // ease on the body's own condition
        if self.cloth_quality >= 0.6 {
            for &v in &self.villagers.clone() {
                if villager::is_dead(&self.world, v) { continue; }
                if let Some(h) = self.world.get_component_mut::<crate::sim::illness::Health>(v) {
                    h.condition = (h.condition + 0.003).min(1.0);
                }
            }
        }
    }

    /// The kettle's own leaf: the pedlar's own import stocks the tea,
    /// the cup's own lift raises the villagers' own energy (the
    /// caffeine's own psychometric), and the ceremony's own bond
    /// accrues when the brew's shared
    pub(in crate::session) fn tea_tick(&mut self, tick: u64) {
        if tick % 30 != 13 { return; }
        // the pedlar's own import stocks the leaf - the spice route's
        // own middleman brings it
        self.tea = (self.tea + 0.02).min(1.0);
        if self.tea <= 0.1 { return; }
        // the cup's own lift: a villager drinking lifts the energy -
        // the caffeine's own psychometric on the day's own work
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            if let Some(h) = self.world.get_component_mut::<crate::sim::illness::Health>(v) {
                h.condition = (h.condition + 0.005).min(1.0);
            }
        }
        self.tea = (self.tea - 0.01).max(0.0);
        // and the ceremony's own bond: a shared brew accrues the
        // sociability - the ritual's own gathering
        if self.tea >= 0.3 {
            let members = self.villagers.iter().copied()
                .filter(|&v| !villager::is_dead(&self.world, v))
                .take(2).collect::<Vec<_>>();
            for m in &members {
                if let Some(b) = self.world.get_component_mut::<crate::sim::psychology::Bond>(*m) {
                    b.security = (b.security + 0.005).min(1.0);
                }
            }
        }
    }

    /// The church's own resin: the incense stock that marks the
    /// sacred - the pedlar's own route keeps it stocked, the services
    /// consume it, and at zero the church goes dark - the sacred's own
    /// mark fades, the separation's own pressure rises
    pub(in crate::session) fn incense_tick(&mut self, tick: u64) {
        if tick % 30 != 18 { return; }
        // the pedlar's own route keeps the resin stocked - the
        // spice-trade's own middleman brings it in
        self.incense = (self.incense + 0.02).min(1.0);
        // and the services consume it - the sacred's own marking
        // burns what the route brings
        let services = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation)
            .map(|c| c.services.len()).unwrap_or(0);
        self.incense = (self.incense - services as f32 * 0.01).max(0.0);
        // at zero the church goes dark - the sacred's own mark fades,
        // and the separation's own pressure rises on the lapse
        if self.incense <= 0.0 {
            self.separation_pressure = (self.separation_pressure + 0.02).min(1.0);
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Minor), format!("t{}: the censer hangs cold - the church's own supply's lapsed, and the sacred's own mark's faded.", tick));
        }
    }

    /// The horizon's own threat: the raiders land - the take falls on
    /// the villagers' own purses and stores, the castle's own watching
    /// halves it (the lord's own obligation, or its own failure), and
    /// the raided carry the grievance
    pub(in crate::session) fn raid_tick(&mut self, tick: u64) {
        // the raiders' own landing: rare - the horizon's own threat on
        // the rarest day, and only once
        if self.raid.is_some() { return; }
        if tick % 400 != 200 { return; }
        self.raid = Some(tick);
        // the take falls on the purses and the stores - the castle's
        // own watching halves what the raiders can carry
        let defended = self.sites.iter().any(|s| s.lord
            .and_then(|l| self.world.get_component::<Watcher>(l))
            .map(|w| w.paranoia >= 0.5).unwrap_or(false));
        let take = if defended { 2.0 } else { 5.0 };
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            if let Some(p) = self.world.get_component_mut::<crate::session::Purse>(v) {
                p.gold = (p.gold - take).max(0.0);
            }
            // and the raided carry the grievance - the horizon's own
            // threat answered or not
            if let Some(l) = self.world.get_component_mut::<crate::game::desires::InnerLife>(v) {
                l.grievances.push((crate::game::desires::Problem::Loss, tick + 60));
            }
        }
        self.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Major), format!("t{}: the raiders come over the ridge - the take falls on the village's own purses, and the castle's own answer's what it is.", tick));
    }

    /// The field's own memory: the three-field's own cycle - every
    /// third year the fallow restores the soil, the other years the
    /// crop takes it. And the hunger's own gap: the spring's own thin
    /// stores before the first harvest - the leanest stretch
    pub(in crate::session) fn rotation_tick(&mut self, tick: u64) {
        if tick % 30 != 6 { return; }
        // the three-field's own cycle: a year in three fallow restores
        // the soil, the rest the crop drains
        let year = tick / 360;
        if year % 3 == 2 {
            self.soil_quality = (self.soil_quality + 0.05).min(1.0);
        } else {
            self.soil_quality = (self.soil_quality - 0.02).max(0.3);
        }
        // the hunger's own gap: the spring's own thin stores before
        // the first harvest - the leanest stretch, and the soil's own
        // quality scales what the yield can answer
        let day_in_year = tick % 360;
        if day_in_year >= 90 && day_in_year < 120 && self.soil_quality < 0.5 {
            self.log.push_ev(EvTag::new(Thread::Economy, Valence::Negative, Sev::Notable), format!("t{}: the spring's own gap - the stores run thin before the first harvest, and the soil's own answer's thin with them.", tick));
        }
    }

    /// The work's own stages: the multi-stage construction accrues on
    /// a master's own guidance - the frame, the soundboard, the
    /// stringing, the finish. At three the masterwork's own making
    /// lands - the skill's own lift
    pub(in crate::session) fn craft_stage_tick(&mut self, tick: u64) {
        if tick % 30 != 8 { return; }
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            let Some(c) = self.world.get_component_mut::<crate::game::vocation::Craft>(v) else { continue };
            // the master's own guidance moves the stages - the lone
            // apprentice's own work stays at the frame
            if c.master.is_none() { continue; }
            c.stage = (c.stage + 1).min(3);
            // the masterwork's own making: at three the piece's own
            // finish lifts the skill - the masterwork's own worth
            if c.stage >= 3 && c.skill < 0.6 {
                c.skill = 0.6;
                self.log.push_ev(EvTag::new(Thread::Craft, Valence::Positive, Sev::Notable), format!("t{}: the masterwork's own finish - the stages' own making lands, the piece worth the name.", tick));
            }
        }
    }

    /// The term's own counting: the years under a master accrue, and
    /// at seven the journeyman's own ceremony lifts the skill - the
    /// apprenticeship's own graduation, the piece made
    pub(in crate::session) fn apprentice_tick(&mut self, tick: u64) {
        if tick % 30 != 24 { return; }
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            let Some(c) = self.world.get_component_mut::<crate::game::vocation::Craft>(v) else { continue };
            if c.master.is_none() { continue; }
            c.apprentice_years += 1;
            // the graduation: at seven years the journeyman's own
            // ceremony - the piece made, the skill lifted past the
            // apprentice's own cap
            if c.apprentice_years >= 7 && c.skill < 0.4 {
                c.skill = 0.4;
                self.log.push_ev(
                    EvTag::new(Thread::Craft, Valence::Positive, Sev::Notable),
                    format!(
                        "t{}: the journeyman's own ceremony - the term's done, the piece's made.",
                        tick
                    ),
                );
            }
        }
    }

    /// The prophet's own vulnerability: the revelation-built org has
    /// no internal way to remove him - the takeover, the co-option,
    /// the knife-fight over the succession. And the cover's own
    /// withdrawal: the shield that grants the governing capacity can
    /// be pulled at the worst moment
    pub(in crate::session) fn capture_tick(&mut self, tick: u64) {
        if tick % 60 != 41 { return; }
        let org = self.org;
        // the prophet's own capture: the revelation's own org can't
        // remove its own head - the takeover takes the leader
        if self.prophet {
            if let Some(o) = self.world.get_component_mut::<crate::sim::community::Organization>(org) {
                // the succession's own knife-fight: the members split
                let n = o.members.len() / 4;
                for _ in 0..n { o.members.pop(); }
            }
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Major), format!("t{}: the prophet's own capture - the succession's knife-fight costs a quarter of the cause.", tick));
        }
        // the cover's own withdrawal: the shield that grants the
        // governing capacity can be pulled - the dependency's the
        // vulnerability
        if self.shield_cover.is_some() && self.institutions > 0 {
            self.shield_cover = None;
            self.institutions = self.institutions.saturating_sub(1);
            if let Some(o) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(org) {
                o.adjust(-0.15);
            }
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Notable), format!("t{}: the cover's withdrawn - the institution pulls its own shield, and the governing capacity goes with it.", tick));
        }
    }

    /// The fold's own deadlock: the stall that won't break on its own
    /// - the council's own call, the Constantine's own move, the
    /// influence that settles what the stall can't
    pub(in crate::session) fn council_tick(&mut self, tick: u64) {
        if tick % 60 != 53 { return; }
        if self.council.is_some() { return; }
        // the deadlock's own signal: the split's own pressure past the
        // stall's own line, or the creed's own heresies grown long
        let stalled = self.separation_pressure >= 0.7
            || self.world.get_component::<crate::sim::religion::Doctrine>(self.congregation)
                .map(|d| d.heresy_vector.len() >= 3).unwrap_or(false);
        if !stalled { return; }
        self.council = Some(tick);
        self.log.push_ev(
            EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
            format!(
                "t{}: the fold's own stall - a council's called to break what the silence can't.",
                tick
            ),
        );
    }

    /// The spirit's own marking: a villager whose own devotion burns
    /// past the fold's own telling becomes the mystic the institution
    /// must answer - the license to prophesy's the gate
    pub(in crate::session) fn revelation_tick(&mut self, tick: u64) {
        if tick % 60 != 47 { return; }
        if self.mystic.is_some() { return; }
        // the spirit's own marking: a villager whose own faith burns
        // brightest becomes the mystic the fold must answer
        let best = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v))
            .max_by(|&a, &b| {
                let fa = self.world.get_component::<crate::sim::religion::Faith>(a)
                    .map(|f| f.devotion).unwrap_or(0.0);
                let fb = self.world.get_component::<crate::sim::religion::Faith>(b)
                    .map(|f| f.devotion).unwrap_or(0.0);
                fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
            });
        let Some(_m) = best else { return };
        self.mystic = Some(tick);
        self.log.push_ev(EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable), format!("t{}: a mystic walks the lanes - the spirit's own marking, and the fold must answer.", tick));
    }

    /// The desk's own making: the school produces the subjects it
    /// teaches - the kids' own dispositions accrue the domination
    /// the crown's own classroom instills. And the fold's own school
    /// counters it - the org-run desk produces the agency instead.
    /// The subject-production's the real battleground
    pub(in crate::session) fn discipline_tick(&mut self, tick: u64) {
        if tick % 30 != 3 { return; }
        // the classroom's own output: the kids at the desk accrue the
        // domination - the crown's own subject made
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            if self.world.get_component::<crate::session::Kid>(v).is_none() { continue; }
            let at_desk = self.world.get_component::<crate::session::routine::Routine>(v)
                .map(|r| r.day == tick - 1 || r.day == tick).unwrap_or(false);
            if !at_desk { continue; }
            let mut d = self.world.get_component::<crate::game::desires::Domination>(v)
                .copied().unwrap_or_default();
            d.internalized_domination = (d.internalized_domination + 0.005).min(1.0);
            self.world.add_component(v, d);
        }
        // and the fold's own counter: the org-run desk produces the
        // agency instead - the counter-institution's own making
        if self.folk_school {
            let members = self.world.get_component::<crate::sim::community::Organization>(self.org)
                .map(|o| o.members.clone()).unwrap_or_default();
            for m in members {
                let mut d = self.world.get_component::<crate::game::desires::Domination>(m)
                    .copied().unwrap_or_default();
                d.critical_agency = (d.critical_agency + 0.005).min(1.0);
                self.world.add_component(m, d);
            }
        }
    }

    /// The frame's own weapon: the legitimacy that shields the lord
    /// also obligates him - the petition's own count, framed in his
    /// own terms, extracts the concession or the standing. The
    /// protection's own promise made enforceable
    pub(in crate::session) fn paternalism_tick(&mut self, tick: u64) {
        if tick % 30 != 11 { return; }
        if self.petition_count == 0 { return; }
        for site in &self.sites {
            let Some(l) = site.lord else { continue };
            let legit = self.world.get_component::<Watcher>(l)
                .map(|w| w.legitimacy).unwrap_or(0.0);
            if legit < 0.5 { continue; }
            // the frame's own claim: a petition citing the lord's own
            // protection extracts the concession - the standing's own
            // obligation made real
            if let Some(o) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(self.org) {
                o.adjust(self.petition_count as f32 * 0.001);
            }
            // and the ignoring's own cost: a frame-cited demand
            // unanswered costs the standing more than the demand
            // asked - the shield's own two-sidedness
            if let Some(w) = self.world.get_component_mut::<Watcher>(l) {
                w.legitimacy = (w.legitimacy - self.petition_count as f32 * 0.002).max(0.0);
            }
        }
    }

    /// The canon's own curation: the crown's own Bible cuts the
    /// liberation framings - the members' own edge dulls on the
    /// official telling, unless the vernacular's own transmission
    /// reads past it. The fold's own letters gate how far the
    /// redaction reaches
    pub(in crate::session) fn canon_tick(&mut self, tick: u64) {
        if tick % 30 != 16 { return; }
        if !self.redacted_canon { return; }
        // the fold's own letters: a numerate fold reads past the
        // official telling - the redaction's own reach stops at the
        // literacy the crown can't take
        let literate = matches!(self.numeracy, crate::session::Numeracy::Positional);
        let members = self.world.get_component::<crate::sim::community::Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        for m in &members {
            let Some(mut o) = self.world.get_component::<crate::game::org_skills::Organizing>(*m).cloned() else { continue };
            if self.vernacular || literate {
                // the illicit telling restores the edge - at the cost
                // of the reading itself being a risk
                o.agitation = (o.agitation + 0.005).min(1.0);
            } else {
                // the official telling dulls the edge - the liberation
                // framings cut, the obedience kept
                o.agitation = (o.agitation - 0.01).max(0.0);
            }
            self.world.add_component(*m, o);
        }
        // and the vernacular's own risk: the illicit telling feeds the
        // castle's own ears
        if self.vernacular && !literate {
            for site in &self.sites {
                let Some(l) = site.lord else { continue };
                if let Some(w) = self.world.get_component_mut::<Watcher>(l) {
                    w.paranoia = (w.paranoia + 0.005).min(1.0);
                }
            }
        }
    }

    /// The crown's own offer: while the fold's own work's a threat,
    /// the legal status lands - the gold for the routinization, the
    /// edge for the refusal
    pub(in crate::session) fn incorporation_tick(&mut self, tick: u64) {
        if tick % 40 != 23 { return; }
        if self.incorporation_offer.is_some() { return; }
        let watched = self.sites.iter().any(|s| s.lord
            .and_then(|l| self.world.get_component::<Watcher>(l))
            .map(|w| w.paranoia >= 0.5).unwrap_or(false));
        if !watched { return; }
        self.incorporation_offer = Some(tick);
        self.log.push_ev(EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable), format!("t{}: a charter offered - the crown's own legal status for the fold, and the gold that comes with it.", tick));
    }

    /// The stack's own work: the logistics accrue on the org's own
    /// supply hands, and the endurance consumes them; the discipline
    /// trains on the members' own practiced meetings, and the
    /// provocateur's own test finds the untrained
    pub(in crate::session) fn campaign_stack_tick(&mut self, tick: u64) {
        if tick % 20 != 4 { return; }
        let members = self.world.get_component::<crate::sim::community::Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        if members.is_empty() { return; }
        // the supply's own hands: the organizers' own practiced work
        // feeds the logistics the endurance needs
        let supply: f32 = members.iter().map(|&m|
            self.world.get_component::<crate::game::org_skills::Organizing>(m)
                .map(|o| o.get(crate::game::org_skills::OrgSkill::Training)
                    + o.get(crate::game::org_skills::OrgSkill::SignalNet)).unwrap_or(0.0)).sum();
        self.campaign_logistics = (self.campaign_logistics + supply * 0.01 - 0.005).max(0.0).min(1.0);
        // and the discipline's own training: the practiced meetings
        // build the habit that holds against the provocateur
        let practice: f32 = members.iter().map(|&m|
            self.world.get_component::<crate::game::org_skills::Organizing>(m)
                .map(|o| o.get(crate::game::org_skills::OrgSkill::SecurityCulture)).unwrap_or(0.0)).sum();
        self.discipline = (self.discipline + practice * 0.005).min(1.0);
        // the provocateur's own test: a plant's own call to violence
        // finds the untrained - the breaking converts the asset into
        // the liability
        if self.plant.is_some() && self.discipline < 0.3 {
            if let Some(o) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(self.org) {
                o.adjust(-0.05);
            }
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Notable), format!("t{}: the untrained room takes the plant's bait - the discipline wasn't there to hold.", tick));
        }
    }

    /// The pillars' own work: the member institutions pay their dues
    /// and lend their names - but their own agendas constrain the
    /// cause, and the powerful's own patronage can pull one loose
    /// unless the founding's own sponsorship holds
    pub(in crate::session) fn institution_tick(&mut self, tick: u64) {
        if tick % 30 != 13 { return; }
        if self.institutions == 0 { return; }
        // the dues: the institutions' own recurring income, and the
        // names lend the cause their standing
        if let Some(o) = self.world.get_component_mut::<crate::sim::community::Organization>(self.org) {
            o.resources += self.institutions as f32 * 2.0;
        }
        if let Some(o) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(self.org) {
            o.adjust(self.institutions as f32 * 0.005);
        }
        // the agenda's own constraint: the institutions' own caution
        // slows the agitation they do not own
        let members = self.world.get_component::<crate::sim::community::Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        for m in &members {
            if let Some(mut o) = self.world.get_component::<crate::game::org_skills::Organizing>(*m).cloned() {
                o.agitation = (o.agitation - self.institutions as f32 * 0.002).max(0.0);
                self.world.add_component(*m, o);
            }
        }
        // the patronage's own pull: the powerful's own offer - an
        // institution drifts unless the founding's sponsorship holds
        let offered = self.sites.iter().any(|s| s.lord
            .and_then(|l| self.world.get_component::<Watcher>(l))
            .map(|w| w.paranoia >= 0.7).unwrap_or(false));
        if offered && !self.locally_sponsored && self.institutions > 0 {
            self.institutions -= 1;
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
                format!(
                    "t{}: a patron's own offer - one of the pillars drifts toward the powerful.",
                    tick
                ),
            );
        }
    }

    /// The canvass's own work: the petition's own count grows the
    /// signatures and develops the hands that carry it - and a
    /// suppression's own backfire, if the frame's ready, turns the
    /// gag into the bigger story
    pub(in crate::session) fn canvass_tick(&mut self, tick: u64) {
        if tick % 20 != 8 { return; }
        let members = self.world.get_component::<crate::sim::community::Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        if members.is_empty() { return; }
        // the canvass's own count: every carried signature's a name
        // the opponent can read
        self.petition_count = self.petition_count.saturating_add(members.len() as u32);
        // and the carrying develops the hands - the canvass is the
        // practice, not just the tally
        for m in &members {
            if let Some(mut o) = self.world.get_component::<crate::game::org_skills::Organizing>(*m).cloned() {
                o.one_on_one = (o.one_on_one + 0.005).min(1.0);
                self.world.add_component(*m, o);
            }
        }
        // the suppression's own backfire: a ban with a frame ready
        // becomes the bigger story - the castle pays its own standing
        let suppressed = self.sites.iter().any(|s| s.lord
            .and_then(|l| self.world.get_component::<Watcher>(l))
            .map(|w| w.paranoia >= 0.7).unwrap_or(false));
        if suppressed && self.frame_readiness {
            self.petition_count = self.petition_count.saturating_add(members.len() as u32 * 2);
            for site in &self.sites {
                let Some(l) = site.lord else { continue };
                if let Some(w) = self.world.get_component_mut::<Watcher>(l) {
                    w.legitimacy = (w.legitimacy - 0.05).max(0.0);
                }
            }
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Positive, Sev::Notable),
                format!(
                    "t{}: they ban the posting, and the ban's the story - the frame was ready.",
                    tick
                ),
            );
        }
    }

    /// The iron rule's own work: the leader's own weight - the
    /// dependence's a vulnerability when the leader's taken, and the
    /// cold-anger screen keeps the brittle ones off the ladder
    pub(in crate::session) fn leader_tick(&mut self, tick: u64) {
        if tick % 40 != 17 { return; }
        let org = self.org;
        let Some(leader) = self.world.get_component::<crate::sim::community::Organization>(org)
            .map(|o| o.leader) else { return };
        // the cold-anger screen: the leader whose own agency's brittle
        // can't take the criticism - screened off the ladder
        let brittle = self.world.get_component::<crate::game::desires::Domination>(leader)
            .map(|d| d.critical_agency < 0.2 && d.internalized_domination > 0.5)
            .unwrap_or(false);
        if brittle {
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor), format!("t{}: the cold-anger screen - the brittle one can't take the criticism, and steps back.", tick));
        }
        // the dependence's own growth: the longer the leader holds,
        // the more the cause leans - and the more the taking would cost
        self.leader_dependence = (self.leader_dependence + 0.005).min(1.0);
        // the dependence's own bite: the capacity's own strength caps
        // at what the cause can do without the leader - the dependence
        // is the vulnerability, not the strength
        if self.leader_dependence > 0.7 {
            if let Some(o) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(org) {
                o.adjust(-0.01);
            }
        }
    }

    /// The movement school's own work: the community teacher - the
    /// organic intellectual - grows the members' own agency; the
    /// imported expert grows the dependency instead. And the folk
    /// school's own memory: the skill doesn't decay across a defeat
    pub(in crate::session) fn movement_school_tick(&mut self, tick: u64) {
        if tick % 30 != 5 { return; }
        if !self.folk_school { return; }
        let members = self.world.get_component::<crate::sim::community::Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        for m in members {
            let Some(mut d) = self.world.get_component::<crate::game::desires::Domination>(m).copied() else { continue };
            if self.community_teacher {
                // the community teacher's own slow work: the agency
                // grows, the domination's own hold loosens
                d.critical_agency = (d.critical_agency + 0.01).min(1.0);
                d.internalized_domination = (d.internalized_domination - 0.005).max(0.0);
            } else {
                // the imported expert's own fast path: knowledge, but
                // the dependency grows - they lean on the outsider
                d.internalized_domination = (d.internalized_domination + 0.01).min(1.0);
            }
            self.world.add_component(m, d);
        }
    }

    /// The pedagogy's own work: the domination's taken in lowered by
    /// the consciousness-raising, the agency grown on it, and the
    /// praxis loop - the experience the action leaves turned to skill
    /// only by the reflection alternating with it. Action alone
    /// plateaus; reflection alone makes theorists
    pub(in crate::session) fn pedagogy_tick(&mut self, tick: u64) {
        if tick % 30 != 9 { return; }
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            let Some(mut d) = self.world.get_component::<crate::game::desires::Domination>(v).copied() else { continue };
            // the consciousness-raising: a meeting or a witnessed win
            // lowers the domination's own hold, grows the agency
            let met = self.world.get_component::<crate::game::desires::InnerLife>(v)
                .map(|i| !i.wants.is_empty()).unwrap_or(false);
            if met {
                d.internalized_domination = (d.internalized_domination - 0.01).max(0.0);
                d.critical_agency = (d.critical_agency + 0.01).min(1.0);
            }
            // the praxis loop: action's own experience + reflection's
            // own agency, alternating, turns into the skill - neither
            // alone gets there
            if d.experience > 0.1 && d.critical_agency > 0.3 {
                if let Some(mut o) = self.world.get_component::<crate::game::org_skills::Organizing>(v).cloned() {
                    o.one_on_one = (o.one_on_one + d.experience * d.critical_agency * 0.01).min(1.0);
                    self.world.add_component(v, o);
                    d.experience = (d.experience - 0.01).max(0.0);
                }
            }
            self.world.add_component(v, d);
        }
    }

    /// The inquisition's own cycle: the summons, the harvest, the
    /// informers, the prosecution, the relapse - the medieval
    /// repression ladder rung by rung. The hidden fold's own
    /// dissimulation halves what the prosecution can take
    pub(in crate::session) fn inquisition_tick(&mut self, tick: u64) {
        // the ladder's rung comes at a season's own pace, not a
        // constant drain on the cause
        if tick % 90 != 32 { return; }
        let watched = self.sites.iter().any(|s| s.lord
            .and_then(|l| self.world.get_component::<Watcher>(l))
            .map(|w| w.paranoia >= 0.5).unwrap_or(false));
        if !watched { return; }
        // the lord doesn't run the ladder on a watcher's own nerves
        // alone - the cause has to be a real one, its own work loud
        let members = self.world.get_component::<crate::sim::community::Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        let organizing: f32 = members.iter().map(|&m|
            self.world.get_component::<crate::game::org_skills::Organizing>(m)
                .map(|o| o.get(crate::game::org_skills::OrgSkill::Turnout)
                    + o.get(crate::game::org_skills::OrgSkill::Agitation)).unwrap_or(0.0)).sum();
        if organizing < 0.5 { return; }
        let stage = self.inquisition.unwrap_or(0);
        let org = self.org;
        match stage {
            0 => {
                // the summons: the call to answer lands
                self.log.push_ev(
                    EvTag::new(Thread::Law, Valence::Neutral, Sev::Notable),
                    format!(
                        "t{}: a summons to answer - the lord's own questions begin.",
                        tick
                    ),
                );
            }
            1 => { // the harvest: the grace/amnesty takes the weak
                if let Some(o) = self.world.get_component_mut::<crate::sim::community::Organization>(org) {
                    let n = o.members.len() / 10;
                    for _ in 0..n { o.members.pop(); }
                }
            }
            2 => { // the informers: the eyes inside grow
                self.penetration = (self.penetration + 0.1).min(1.0);
            }
            3 => { // the prosecution: the raid lands - the hidden fold
                // takes half what an open one would
                let damage = if self.hidden_institution { 0.05 } else { 0.15 };
                if let Some(o) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(org) {
                    o.adjust(-damage);
                }
            }
            _ => { // the relapse: the maintenance stage - the cycle's
                // run, the penitents' own leverage stays until it's cut
                self.penetration = (self.penetration + 0.05).min(1.0);
            }
        }
        // the ladder runs once to the relapse - the maintenance stage,
        // not a perpetual re-harvest
        self.inquisition = Some((stage + 1).min(4));
    }

    /// The lord's own parley: while the org's a threat, an offer to
    /// talk lands - the negotiate-and-decapitate play, or a real
    /// settlement. Riding to it risks the ambush; holding the
    /// distance's safe; demanding hostages makes the guarantee cost
    /// the castle itself
    pub(in crate::session) fn parley_tick(&mut self, tick: u64) {
        if tick % 40 != 31 { return; }
        if self.parley.is_some() { return; }
        // the threat's on the members' own work, not the org's own
        // record: Turnout + Agitation on whoever's carrying the word
        let members = self.world.get_component::<crate::sim::community::Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        let organizing: f32 = members.iter().map(|&m|
            self.world.get_component::<crate::game::org_skills::Organizing>(m)
                .map(|o| o.get(crate::game::org_skills::OrgSkill::Turnout)
                    + o.get(crate::game::org_skills::OrgSkill::Agitation)).unwrap_or(0.0)).sum();
        if organizing < 0.5 { return; }
        for site in &self.sites {
            let Some(l) = site.lord else { continue };
            let Some(w) = self.world.get_component::<Watcher>(l) else { continue };
            if w.paranoia < 0.5 { continue; }
            self.parley = Some(tick);
            self.log.push_ev(EvTag::new(Thread::Law, Valence::Neutral, Sev::Notable), format!("t{}: a herald under a white flag - the lord offers to talk. The flag's own honor's the question.", tick));
            return;
        }
    }

    /// The sign's own spread: the org's symbol rides the channels'
    /// own reach - carried further the stronger they run, and fading
    /// without the work of spreading it. More answer the call when
    /// the sign's known
    pub(in crate::session) fn brand_tick(&mut self, tick: u64) {
        if tick % 20 != 14 { return; }
        let members = self.world.get_component::<crate::sim::community::Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        // the channels' own reach carries the sign: symbol + signal_net
        // + pulpit's borrowed standing
        let mut reach = 0.0;
        for m in &members {
            if let Some(o) = self.world.get_component::<crate::game::org_skills::Organizing>(*m) {
                reach += o.get(crate::game::org_skills::OrgSkill::Symbol)
                    + o.get(crate::game::org_skills::OrgSkill::SignalNet)
                    + o.get(crate::game::org_skills::OrgSkill::Pulpit);
            }
        }
        // it rises on the channels' own work, and fades unreinforced
        self.brand_saturation = (self.brand_saturation + reach * 0.02 - 0.01)
            .max(0.0).min(1.0);
        // and the sign's own pull: more answer the call when it's known -
        // the cause's own reach scales by it
        if self.brand_saturation > 0.3 {
            if let Some(o) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(self.org) {
                o.adjust(self.brand_saturation * 0.01);
            }
        }
    }

    /// The lord's own counter-rite: a de-oathing ceremony on the
    /// threat-watch - the oath's own binding frays, the members' own
    /// hands loosen, but nobody's taken. The front's own armor: an
    /// open face halves what the exposure costs the secret wing
    pub(in crate::session) fn counter_ritual_tick(&mut self, tick: u64) {
        if tick % 40 != 27 { return; }
        let watched = self.sites.iter().any(|s| s.lord
            .and_then(|l| self.world.get_component::<Watcher>(l))
            .map(|w| w.paranoia >= 0.5).unwrap_or(false));
        if !watched { return; }
        let org = self.org;
        let members = self.world.get_component::<crate::sim::community::Organization>(org)
            .map(|o| o.members.clone()).unwrap_or_default();
        // the de-oathing: the oath's own binding frays a notch, the
        // members' own hands loosen - commitment damage, not removal
        self.oath_intensity = self.oath_intensity.saturating_sub(1);
        let damage = if self.front { 0.02 } else { 0.05 };
        for m in members {
            if let Some(b) = self.world.get_component_mut::<crate::sim::psychology::Bond>(m) {
                b.security = (b.security - damage).max(0.0);
            }
        }
        self.log.push_ev(
            EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
            format!(
                "t{}: the de-oathing ceremony - the members' hands loosen, but nobody's taken.",
                tick
            ),
        );
    }

    /// The creed's own scoring: a council settles a position - the
    /// canon grows by the win, the defeated side lands on the
    /// heresy_vector where it can resurface as a splinter. And the
    /// suffering the faith answers converts the grievances to
    /// devotion - the theodicy's own work
    pub(in crate::session) fn doctrine_tick(&mut self, tick: u64) {
        if tick % 30 != 19 { return; }
        // the council's own settling: a position lands on the canon,
        // the loser on the heresy_vector
        if (tick / 90) % 4 == 1 {
            let positions = ["the eucharist's own presence", "the saints' own intercession",
                "the tithe's own due", "the priest's own celibacy"];
            let i = (tick as usize / 90) % positions.len();
            let Some(d) = self.world.get_component_mut::<crate::sim::religion::Doctrine>(self.congregation) else { return };
            let pos = positions[i].to_string();
            if !d.canon.contains(&pos) {
                d.canon.push(pos.clone());
                d.heresy_vector.push(format!("the {} denied", pos));
            }
        }
        // the theodicy's own work: the grievances the faith answers
        // convert to the devotion it holds
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            let grieved = self.world.get_component::<crate::game::desires::InnerLife>(v)
                .map(|i| !i.grievances.is_empty()).unwrap_or(false);
            if !grieved { continue; }
            if let Some(f) = self.world.get_component_mut::<crate::sim::religion::Faith>(v) {
                f.devotion = (f.devotion + self.suffering_meaning * 0.005).min(1.0);
            }
        }
    }

    /// The church's own instruments: the parallel fold accrues its
    /// own weight until the schism fires; and the lord's own silence -
    /// the interdict - drains what the services would hold
    pub(in crate::session) fn sacrament_tick(&mut self, tick: u64) {
        if tick % 30 != 26 { return; }
        // the parallel fold's own weight: the org's own size is the
        // infrastructure the church cannot see
        let members = self.world.get_component::<crate::sim::community::Organization>(self.org)
            .map(|o| o.members.len()).unwrap_or(0);
        if members > 6 {
            self.separation_pressure = (self.separation_pressure
                + members as f32 * 0.005).min(1.0);
        }
        // and the schism fires whether the leadership wants it or not
        if self.separation_pressure >= 1.0 {
            self.separation_pressure = 0.0;
            if let Some(c) = self.world.get_component_mut::<crate::sim::religion::Congregation>(self.congregation) {
                c.orthodoxy = (c.orthodoxy - 0.2).max(0.0);
            }
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Notable), format!("t{}: the fold splits on its own weight - a second way of praying walks out the door.", tick));
        }
        // the lord's own silence: the interdict's own drain on the flock
        if self.interdict {
            let flock = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation)
                .map(|c| c.members.clone()).unwrap_or_default();
            for m in flock {
                if let Some(f) = self.world.get_component_mut::<crate::sim::religion::Faith>(m) {
                    f.devotion = (f.devotion - 0.01).max(0.0);
                }
            }
        }
    }

    /// The lord's own postings: while the org's a threat, the
    /// reporting/audit burden lands - the compliance work itself is
    /// the weapon. Unmet, the burden ticks the cause's own capacity
    /// down; the bigger the cause, the more the eyes demand
    pub(in crate::session) fn compliance_tick(&mut self, tick: u64) {
        if tick % 30 != 5 { return; }
        let org = self.org;
        let capacity = self.world.get_component::<crate::sim::organized_power::OrganizedPower>(org)
            .map(|o| o.capacity).unwrap_or(0.0);
        let members = self.world.get_component::<crate::sim::community::Organization>(org)
            .map(|o| o.members.len()).unwrap_or(0);
        // the burden's laid while the cause's a threat and grows with
        // its own prominence
        let organizing = self.world.get_component::<crate::game::org_skills::Organizing>(org)
            .map(|o| o.get(crate::game::org_skills::OrgSkill::Turnout)
                + o.get(crate::game::org_skills::OrgSkill::Agitation)).unwrap_or(0.0);
        if organizing >= 0.5 {
            for site in &self.sites {
                let Some(l) = site.lord else { continue };
                let Some(w) = self.world.get_component::<Watcher>(l) else { continue };
                if w.paranoia < 0.5 { continue; }
                self.compliance_burden = (self.compliance_burden
                    + 0.05 + capacity * 0.01 + members as f32 * 0.001).min(1.0);
            }
        }
        // the unmet burden ticks the cause's own capacity down
        if self.compliance_burden > 0.0 {
            if let Some(o) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(org) {
                o.adjust(-self.compliance_burden * 0.02);
            }
        }
    }

    /// The lord's own suit: while the org's a threat, a manor court
    /// action lands - the fees accrue day on day, the longer contested
    /// the deeper the drain. Winning the case and losing the org's a
    /// real outcome - a favorable ruling still leaves the cause spent
    pub(in crate::session) fn lawfare_tick(&mut self, tick: u64) {
        if let Some((filed, _)) = self.lawfare {
            let days = tick.saturating_sub(filed);
            // the fees escalate by the day contested - the org's own
            // practice in the courts blunts the ladder some
            let defense = self.world.get_component::<crate::game::org_skills::Organizing>(self.org)
                .map(|o| o.get(crate::game::org_skills::OrgSkill::Training)).unwrap_or(0.0);
            let drain = 0.02 * days as f32 * (1.0 - defense * 0.5).max(0.3);
            if let Some(o) = self.world.get_component_mut::<crate::sim::community::Organization>(self.org) {
                o.resources = (o.resources - drain).max(0.0);
            }
            // a favorable ruling ends the case on day 15 - the drain's
            // already spent, the org's thinner for the winning
            if days >= 15 {
                self.lawfare = None;
                self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: the court finds for the cause - but the org's thinner for the winning of it.", tick));
            }
            return;
        }
        // the file: a threat + a castle's eyes + on the cooldown
        if tick % 40 != 34 { return; }
        let organizing = self.world.get_component::<crate::game::org_skills::Organizing>(self.org)
            .map(|o| o.get(crate::game::org_skills::OrgSkill::Turnout)
                + o.get(crate::game::org_skills::OrgSkill::Agitation)).unwrap_or(0.0);
        if organizing < 0.5 { return; }
        for site in &self.sites {
            let Some(l) = site.lord else { continue };
            let Some(w) = self.world.get_component::<Watcher>(l) else { continue };
            if w.paranoia < 0.5 { continue; }
            self.lawfare = Some((tick, tick));
            self.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Notable), format!("t{}: a summons on parchment - the lord's own court calls the cause to answer. The fees begin.", tick));
            return;
        }
    }

    /// The org's own secrecy: the oath binds the members' own hands
    /// (the stronger the oath, the surer they keep - and the steeper
    /// the fall for the one who breaks it), the cover's own truth
    /// keeps the eyes' surface thin, and the eyes get in with size
    /// and age - vetting and compartmentalization move the tradeoff,
    /// not the accrual itself
    pub(in crate::session) fn clandestine_tick(&mut self, tick: u64) {
        if tick % 30 != 22 { return; }
        let org = self.org;
        let members = self.world.get_component::<crate::sim::community::Organization>(org)
            .map(|o| o.members.clone()).unwrap_or_default();
        // the oath's own bind: each member's own hands hold tighter
        // by the oath's strength
        for &m in &members {
            if let Some(b) = self.world.get_component_mut::<crate::sim::psychology::Bond>(m) {
                b.security = (b.security + self.oath_intensity as f32 * 0.01).min(1.0);
            }
        }
        // the eyes get in with size and age - the cover's own truth
        // slows them, no cover at all lets them run
        let age = (tick / 90).min(8) as f32;
        let accrual = members.len() as f32 * 0.002 + age * 0.004;
        let rate = if self.cover { accrual * 0.5 } else if self.oath_intensity > 0 { accrual * 1.4 } else { accrual };
        self.penetration = (self.penetration + rate).min(1.0);
        // the oath's own stake: at high penetration and a curse-backed
        // oath, the discovered oath's a crime against the org itself
        if self.penetration > 0.6 && self.oath_intensity >= 2 {
            self.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Major), format!("t{}: the eyes find the oath's own writing - the secret's a charge now, not a whisper.", tick));
        }
    }

    /// The cult's own end: when the dials run past sixty and the
    /// idealism's spent, the cage breaks one of four ways - the
    /// implosion of its own split, the capture by a harder head, the
    /// deadline's date, or the rare routinization into a sect
    pub(in crate::session) fn cult_collapse_tick(&mut self, tick: u64) {
        if self.cult_collapse.is_some() { return; }
        if tick % 30 != 15 { return; }
        let t = self.world.get_component::<crate::sim::religion::Totalism>(self.congregation);
        let Some(t) = t else { return };
        let sum = t.dials.iter().map(|&d| d as u16).sum::<u16>();
        if sum < 60 || t.idealism > 0.2 { return; }
        // the mode reads the fold's own shape: an ambitious head
        // captures, a strong member implodes, the rare one's the sect
        let ambitious = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation)
            .and_then(|c| c.leader)
            .and_then(|l| self.world.get_component::<crate::game::traits::Personality>(l)
                .map(|p| p.traits.contains(&crate::game::traits::Trait::Ambitious)))
            .unwrap_or(false);
        let roll = (tick.wrapping_mul(43).wrapping_add(self.congregation as u64)) % 100;
        let mode = if roll < 10 { crate::sim::religion::CollapseMode::Institutionalization }
            else if ambitious { crate::sim::religion::CollapseMode::Capture }
            else if roll < 60 { crate::sim::religion::CollapseMode::Implosion }
            else { crate::sim::religion::CollapseMode::Deadline };
        self.cult_collapse = Some(mode);
        let name = match mode {
            crate::sim::religion::CollapseMode::Implosion => "the fold splits on itself",
            crate::sim::religion::CollapseMode::Capture => "a harder head takes the fold",
            crate::sim::religion::CollapseMode::Deadline => "the fold's own date comes",
            crate::sim::religion::CollapseMode::Institutionalization => "the fold becomes a sect",
        };
        self.log.push_ev(
            EvTag::new(Thread::Org, Valence::Positive, Sev::Major),
            format!("t{}: {} - the cage breaks its own way.", tick, name),
        );
    }

    /// The cult's own drift: a hard head and a strict orthodoxy push
    /// the dials up; the idealism's spent down to the leader's own
    /// purse; the flock's own cutting-off multiplies what the dials
    /// take and raises what it costs a member to leave
    pub(in crate::session) fn totalism_tick(&mut self, tick: u64) {
        if tick % 30 != 10 { return; }
        let cong = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation)
            .map(|c| (c.leader, c.orthodoxy, c.services.len()));
        let Some((leader, orthodoxy, services)) = cong else { return };
        // the head's own heat: ambitious, hot, proud - the kind that
        // tightens a fold
        let heat = leader.and_then(|l| self.world.get_component::<crate::game::traits::Personality>(l)
            .map(|p| p.traits.iter().filter(|&&t| matches!(t,
                crate::game::traits::Trait::Ambitious
                | crate::game::traits::Trait::Hotheaded
                | crate::game::traits::Trait::Proud)).count() as f32)).unwrap_or(0.0);
        // the flock's own cutting-off: the members' outside bonds run thin
        let members = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation)
            .map(|c| c.members.clone()).unwrap_or_default();
        let outside = members.iter()
            .filter(|&&m| self.world.get_component::<crate::sim::psychology::AttachmentHistory>(m)
                .map(|h| h.episodes.len() < 4).unwrap_or(false)).count();
        let isolation = if members.is_empty() { 0.0 } else { outside as f32 / members.len() as f32 };
        let take = {
            let Some(t) = self.world.get_component_mut::<crate::sim::religion::Totalism>(self.congregation) else { return };
            // a pressured hard head and a strict orthodoxy push a dial
            let push = heat + orthodoxy + isolation;
            if (tick.wrapping_mul(31).wrapping_add(self.congregation as u64)) % 100 < (push * 12.0) as u64 {
                let i = (tick as usize * 7 + self.congregation as usize) % 8;
                t.dials[i] = t.dials[i].saturating_add(1).min(10);
            }
            // the extraction: the idealism's spent down to the leader
            let take = t.idealism * 0.05;
            t.idealism -= take;
            // and the isolation multiplies what the dials take - and
            // what it costs to leave
            t.exit_cost += t.dials.iter().map(|&d| d as f32).sum::<f32>() * 0.01 * (1.0 + isolation);
            // each rite's its own rung in the cage
            t.exit_cost += services as f32 * 0.005;
            t.isolation = isolation;
            take
        };
        if take > 0.0 {
            if let Some(l) = leader {
                if let Some(p) = self.world.get_component_mut::<Purse>(l) {
                    p.gold += take * 0.1;
                }
            }
        }
        // the flag reads off the dial total
        let sum = self.world.get_component::<crate::sim::religion::Totalism>(self.congregation)
            .map(|t| t.dials.iter().map(|&d| d as u16).sum::<u16>()).unwrap_or(0);
        if let Some(c) = self.world.get_component_mut::<crate::sim::religion::Congregation>(self.congregation) {
            c.cultish = sum > 40;
        }
    }

    /// The broker's own squeeze: the intermediaries between the castle
    /// and the ground carry the pressure both sides put on - and at
    /// enough squeeze, the community's own pull flips them to the cause
    pub(in crate::session) fn broker_tick(&mut self, tick: u64) {
        if tick % 30 != 12 { return; }
        let threat = self.sites.iter().any(|s| s.lord
            .and_then(|l| self.world.get_component::<Watcher>(l))
            .map(|w| w.paranoia >= 0.5).unwrap_or(false));
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            let Some(mut b) = self.world.get_component::<crate::game::vocation::Broker>(v).copied() else { continue };
            if threat { b.pressure = (b.pressure + 0.05).min(1.0); }
            // the flip: enough squeeze and the community's own pull
            // outweighs the castle's - the broker comes over
            if b.pressure > 0.8 && b.community_loyalty > b.power_loyalty {
                // the prophet's own won't hold a seat - same bar as recruit
                if self.world.get_component::<crate::game::prophetic::Follower>(v).is_some() {
                    self.world.add_component(v, b);
                    continue;
                }
                let already = self.world.get_component::<crate::sim::community::Organization>(self.org)
                    .map(|o| o.members.contains(&v)).unwrap_or(false);
                if !already {
                    if let Some(o) = self.world.get_component_mut::<crate::sim::community::Organization>(self.org) {
                        o.members.push(v);
                    }
                    self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Minor), format!("t{}: an intermediary comes over - the broker reads the ground for the cause.", tick));
                    self.world.remove_component::<crate::game::vocation::Broker>(v);
                    continue;
                }
            }
            self.world.add_component(v, b);
        }
    }

    /// A soul's own standing: ordination lifts a priest to clergy, a
    /// knighting lifts a knight to nobility, and a wedding pulls the
    /// lower house to the higher. The clergy's own read of the law
    /// brings its dues - the devotion they tend accrues a touch faster
    pub(in crate::session) fn estate_tick(&mut self, tick: u64) {
        if tick % 20 != 8 { return; }
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            let est = self.world.get_component::<crate::game::vocation::Estate>(v).copied();
            // ordination and knighthood lift the standing
            if let Some(c) = self.world.get_component::<crate::game::vocation::Craft>(v) {
                let lift = match c.vocation {
                    crate::game::vocation::Vocation::Priest => Some(crate::game::vocation::Estate::Clergy),
                    crate::game::vocation::Vocation::Knight => Some(crate::game::vocation::Estate::Nobility),
                    _ => None,
                };
                if let Some(l) = lift {
                    if est != Some(l) {
                        self.world.add_component(v, l);
                    }
                }
            }
            // a wedding pulls the lower house to the higher
            if let Some(m) = self.world.get_component::<crate::game::relationships::Married>(v) {
                if m.form == 0 {
                    if let (Some(me), Some(se)) = (
                        self.world.get_component::<crate::game::vocation::Estate>(v).copied(),
                        self.world.get_component::<crate::game::vocation::Estate>(m.spouse).copied()) {
                        let rank = |e: crate::game::vocation::Estate| match e {
                            crate::game::vocation::Estate::Laity => 0,
                            crate::game::vocation::Estate::Clergy => 1,
                            crate::game::vocation::Estate::Nobility => 2 };
                        if rank(se) > rank(me) { self.world.add_component(v, se); }
                    }
                }
            }
            // the clergy's own read of the law brings its dues: their
            // devotion accrues a touch faster - a bonus, not a wall
            if est == Some(crate::game::vocation::Estate::Clergy) {
                if let Some(f) = self.world.get_component_mut::<crate::sim::religion::Faith>(v) {
                    f.devotion = (f.devotion + 0.002).min(1.0);
                }
            }
        }
    }

    /// A marriage's own end: a bond worn thin and a grievance kept
    /// makes the separation likely - church-declared, the union never
    /// was (the annulment); a spouse's own walking, the deserter's
    /// claim falls to the stayer (the desertion). Children follow the
    /// staying parent's roof
    pub(in crate::session) fn separation_tick(&mut self, tick: u64) {
        if tick % 30 != 17 { return; }
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            let Some(m) = self.world.get_component::<crate::game::relationships::Married>(v).copied() else { continue };
            if m.form != 0 { continue; }
            // a bond worn thin: the union's own strain
            let thin = self.world.get_component::<crate::sim::psychology::Bond>(v)
                .map(|b| b.target == m.spouse && b.security < 0.3).unwrap_or(false);
            if !thin { continue; }
            // the roll: the church's own read (the annulment) vs the
            // spouse's own walking (the desertion)
            let annulled = (tick.wrapping_mul(37).wrapping_add(v as u64)) % 100 < 30;
            if let Some(mm) = self.world.get_component_mut::<crate::game::relationships::Married>(v) {
                mm.form = if annulled { 1 } else { 2 };
            }
            if let Some(mm) = self.world.get_component_mut::<crate::game::relationships::Married>(m.spouse) {
                mm.form = if annulled { 1 } else { 2 };
            }
            if annulled {
                self.log.push_ev(
                    EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Notable),
                    format!(
                        "t{}: the union's declared never-was - the church's own reading of it.",
                        tick
                    ),
                );
            } else {
                // the deserter's own claim falls to the stayer: the
                // purse and the stock stay with the one who kept the roof
                if let Some(p) = self.world.get_component_mut::<Purse>(v) {
                    let share = p.gold * 0.5;
                    p.gold -= share;
                    if let Some(sp) = self.world.get_component_mut::<Purse>(m.spouse) {
                        sp.gold += share;
                    }
                }
                self.log.push_ev(EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Notable), format!("t{}: a spouse walks - the deserter's claim falls to the one who kept the roof.", tick));
            }
        }
    }

    /// The trade's own toll: the dusty and dangerous trades accrue
    /// their own exposure day on day - past the threshold the trade's
    /// own disease lands, chronic, the craft's own price. A miner's
    /// dust, a smith's heat, a miller's mill-stone grit
    pub(in crate::session) fn trade_exposure_tick(&mut self, tick: u64) {
        if tick % 10 != 3 { return; }
        let heavy = |v: crate::game::vocation::Vocation| matches!(v,
            crate::game::vocation::Vocation::Mason
            | crate::game::vocation::Vocation::Toolmaker
            | crate::game::vocation::Vocation::Weaver);
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            let craft = self.world.get_component_mut::<crate::game::vocation::Craft>(v);
            let Some(c) = craft else { continue };
            if !heavy(c.vocation) { continue; }
            c.exposure += 0.005;
            // past the threshold the trade's own disease lands - a hit
            // the body keeps, not an illness it shakes
            if c.exposure >= 0.5 && c.exposure < 0.51 {
                if let Some(h) = self.world.get_component_mut::<crate::sim::illness::Health>(v) {
                    h.condition = (h.condition - 0.15).max(0.1);
                }
                self.log.push_ev(
                    EvTag::new(Thread::Illness, Valence::Negative, Sev::Notable),
                    format!(
                        "t{}: the trade's own toll shows - the dust's in the bones now.",
                        tick
                    ),
                );
            }
        }
    }

    /// The org's own eyes: the level grows with its own discipline
    /// (Turnout + SecurityCulture) and the density of its one-on-ones -
    /// the deep org reads what the lord can't see
    pub(in crate::session) fn counter_intel_tick(&mut self, tick: u64) {
        if tick % 90 != 8 { return; }
        let org = self.org;
        let skill = self.world.get_component::<crate::game::org_skills::Organizing>(org)
            .map(|o| o.get(crate::game::org_skills::OrgSkill::SecurityCulture)
                + o.get(crate::game::org_skills::OrgSkill::Training)).unwrap_or(0.0);
        let density = self.world.get_component::<crate::sim::community::Organization>(org)
            .map(|o| {
                let inside: usize = o.members.iter()
                    .filter(|&&m| {
                        o.members.iter().any(|&n| n != m &&
                            self.world.get_component::<crate::sim::psychology::Bond>(n)
                                .map(|b| b.target == m).unwrap_or(false))
                    }).count();
                inside
            }).unwrap_or(0);
        self.counter_intel = ((skill * 2.0 + density as f32 * 0.5) as u8).min(3);
    }

    /// The staged incident: while the org's a threat AND at a campaign's
    /// own peak, the lord declares the emergency - assemblies banned,
    /// the square held - and the clock runs on the village's answer
    pub(in crate::session) fn emergency_tick(&mut self, tick: u64) {
        if let Some((_, deadline)) = self.emergency {
            // the window's run out: the repression's just how things are
            // now - the paranoia's the accrual of a normalized ban
            if tick >= deadline {
                self.emergency = None;
                self.witnesses = 0;
                for site in &self.sites {
                    let Some(l) = site.lord else { continue };
                    if let Some(w) = self.world.get_component_mut::<Watcher>(l) {
                        w.paranoia = (w.paranoia + 0.3).min(1.0);
                    }
                }
                self.log.push_ev(EvTag::new(Thread::Law, Valence::Neutral, Sev::Minor), format!("t{}: the emergency's just how things are now - the ban's the season, not the news.", tick));
            }
            return;
        }
        // the declare: a threat + a campaign's own action peak
        if tick % 40 != 30 { return; }
        let org = self.org;
        let organizing = self.world.get_component::<crate::game::org_skills::Organizing>(org)
            .map(|o| o.get(crate::game::org_skills::OrgSkill::Turnout)
                + o.get(crate::game::org_skills::OrgSkill::Agitation)).unwrap_or(0.0);
        if organizing < 0.5 { return; }
        let acting = self.world.query::<crate::game::campaign::Campaign>().iter()
            .any(|&c| self.world.get_component::<crate::game::campaign::Campaign>(c)
                .map(|cc| cc.phase == crate::game::campaign::CampaignPhase::Action).unwrap_or(false));
        if !acting { return; }
        for site in &self.sites {
            let Some(l) = site.lord else { continue };
            let Some(w) = self.world.get_component::<Watcher>(l) else { continue };
            if w.paranoia < 0.5 { continue; }
            // the documentation widens the window: the deeper the
            // org's own eyes, the longer its witnesses have to come
            self.emergency = Some((tick, tick + 20 + self.counter_intel as u64 * 4));
            self.witnesses = 0;
            self.suppression_arc = 3;
            self.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Major), format!("t{}: a brawl at the granary - the lord calls it the cause's own doing and bans the square. The clock runs.", tick));
            return;
        }
    }

    /// The plant's work: the lord's infiltration puts a member in the
    /// org while it's a threat, and at the campaign's action peak that
    /// member pushes the violent step - the proposal's the weapon, not
    /// the deed (deferred to #842)
    pub(in crate::session) fn provocateur_tick(&mut self, tick: u64) {
        // plant the member while the org shows and a castle's eyes are on it
        if self.plant.is_none() && tick % 40 == 20 {
            let org = self.org;
            let organizing = self.world.get_component::<crate::game::org_skills::Organizing>(org)
                .map(|o| o.get(crate::game::org_skills::OrgSkill::Turnout)
                    + o.get(crate::game::org_skills::OrgSkill::Agitation)).unwrap_or(0.0);
            if organizing >= 0.5 {
                for site in &self.sites {
                    let Some(l) = site.lord else { continue };
                    let Some(w) = self.world.get_component::<Watcher>(l) else { continue };
                    if w.paranoia < 0.5 { continue; }
                    // a member of theirs, the least-marked, goes in
                    let plant = self.world.get_component::<crate::sim::community::Organization>(org)
                        .and_then(|o| o.members.first().copied());
                    if let Some(m) = plant {
                        self.world.add_component(m, crate::game::campaign::Plant(l));
                        self.plant = Some(m);
                        self.suppression_arc = 2;
                        self.log.push_ev(
                            EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                            format!(
                                "t{}: a new face joins the cause - quiet, keeps to the edges.",
                                tick
                            ),
                        );
                    }
                    return;
                }
            }
        }
        // the proposal: at the campaign's own action peak, the plant
        // speaks for the violent step - the player's call after that
        if self.plant.is_some() && self.proposal.is_none() {
            let acting = self.world.query::<crate::game::campaign::Campaign>().iter()
                .any(|&c| self.world.get_component::<crate::game::campaign::Campaign>(c)
                    .map(|cc| cc.phase == crate::game::campaign::CampaignPhase::Action).unwrap_or(false));
            if acting {
                self.proposal = Some(tick);
                self.log.push_ev(EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable), format!("t{}: one of them's for the torch - burn the lord's stores, they say, now while the eyes are on us.", tick));
            }
        }
    }

    /// The lord's counter: while the org's a threat (its organizing
    /// shows and a castle's eyes are on it), a seat gets offered on the
    /// board - binding behind it, or the performative kind that absorbs
    /// the cause's own energy if taken
    pub(in crate::session) fn seat_offer_tick(&mut self, tick: u64) {
        if self.seat_offer.is_some() { return; }
        // on the cooldown, while the org shows enough to look like a threat
        if tick % 40 != 12 { return; }
        let org = self.org;
        let organizing = self.world.get_component::<crate::game::org_skills::Organizing>(org)
            .map(|o| o.get(crate::game::org_skills::OrgSkill::Turnout)
                + o.get(crate::game::org_skills::OrgSkill::Agitation)).unwrap_or(0.0);
        if organizing < 0.5 { return; }
        // a castle's eyes mark it: the watcher whose paranoia's high enough
        for site in &self.sites {
            let Some(l) = site.lord else { continue };
            let Some(w) = self.world.get_component::<Watcher>(l) else { continue };
            if w.paranoia < 0.5 { continue; }
            // the offer: binding on a det roll - the deeper the cause's
            // cut, the likelier the lord means it
            let binding = (tick.wrapping_mul(37).wrapping_add(l as u64)) % 100 < 40;
            self.seat_offer = Some(crate::game::campaign::SeatOffer {
                from: l, role: crate::game::campaign::Role::Chair,
                binding, offered: tick });
            self.suppression_arc = 1;
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
                format!(
                    "t{}: the lord's man carries an offer - a seat on the board for the cause.",
                    tick
                ),
            );
            return;
        }
    }

    /// The exit: a villager whose live grievances pass what their
    /// devotion can bear walks out of the fold - the mark the leaving
    /// leaves on them, the congregation's roll, and the temple's books
    pub(in crate::session) fn exit_tick(&mut self, tick: u64) {
        use crate::sim::religion::Apostate;
        let cong = self.congregation;
        // the week's tithe comes due on its day: the call's own weight
        // is the grievance it accrues on the purses it bites - the coin
        // stays out of it, a real drain starved the till
        if tick % 7 == 3 {
            let members: Vec<Entity> = self.world.get_component::<crate::sim::religion::Congregation>(cong)
                .map(|c| c.members.clone()).unwrap_or_default();
            for m in &members {
                let thin = self.world.get_component::<Purse>(*m).map(|p| p.gold < 1.0).unwrap_or(false);
                if thin {
                    desires::add_grievance(&mut self.world, *m, desires::Problem::Taxed, tick + 30);
                    // the parish's due is a provocation: it can kindle
                    // the green word in a susceptible soul
                    self.spark_reformation(tick, "the parish's due");
                }
            }
        }
        // the tolerance: a devout heart bears more than a drifting one -
        // past it, a det-mix roll decides whether today they walk
        let members: Vec<Entity> = self.world.get_component::<crate::sim::religion::Congregation>(cong)
            .map(|c| c.members.clone()).unwrap_or_default();
        for m in members {
            let live = self.world.get_component::<desires::InnerLife>(m)
                .map(|l| l.grievances.iter().filter(|(_, u)| *u > tick).count()).unwrap_or(0);
            let devotion = self.world.get_component::<crate::sim::religion::Faith>(m)
                .map(|f| f.devotion).unwrap_or(0.0);
            let tolerance = 2 + (devotion * 4.0) as usize;
            if live <= tolerance { continue; }
            let roll = (tick.wrapping_mul(31).wrapping_add(m as u64)) % 1000;
            if roll < 120 {
                let Some(c) = self.world.get_component_mut::<crate::sim::religion::Congregation>(cong)
                    else { continue };
                c.members.retain(|&x| x != m);
                self.world.add_component(m, Apostate);
                self.log.push_ev(
                    EvTag::new(Thread::Org, Valence::Negative, Sev::Notable),
                    format!(
                        "t{}: {} walks out of the fold - the grievances outgrew the faith.",
                        tick,
                        self.name_of(m)
                    ),
                );
            }
        }
    }

    /// The rival's push: an org leans on a congregation's weak hearts -
    /// grievances injected where the faith already thins. The seam the
    /// org-side consumer rides - rivals and bots call it, not the tick
    #[allow(dead_code)]
    pub(in crate::session) fn exit_push(&mut self, tick: u64, targets: &[Entity]) {
        for &m in targets {
            desires::add_grievance(&mut self.world, m, desires::Problem::OverseerCut, tick + 20);
        }
    }

    /// The campaign's own season: it listens before it cuts, it reads
    /// the room before it acts - a dossier moves it past the cutting,
    /// a directive past the research, and a settled campaign loops
    /// back to the listening or ends
    pub(in crate::session) fn phase_tick(&mut self, tick: u64) {
        let mut done: Vec<Entity> = vec![];
        for e in self.world.query::<crate::game::campaign::Campaign>() {
            let c = match self.world.get_component_mut::<crate::game::campaign::Campaign>(e) {
                Some(c) => c, None => continue };
            let next = match c.phase {
                crate::game::campaign::CampaignPhase::Listening =>
                    if c.target.is_some() { Some(c.phase.step()) } else { None },
                crate::game::campaign::CampaignPhase::IssueCut =>
                    if c.dossier.is_some() { Some(c.phase.step()) } else { None },
                crate::game::campaign::CampaignPhase::Research =>
                    if c.directive.is_some() { Some(c.phase.step()) } else { None },
                crate::game::campaign::CampaignPhase::Action =>
                    if tick > c.founded_day + 30 { Some(c.phase.step()) } else { None },
                crate::game::campaign::CampaignPhase::Evaluate => Some(c.phase.step()),
            };
            if let Some(n) = next {
                c.phase = n;
                if n == crate::game::campaign::CampaignPhase::Action {
                    self.log.push_ev(
                        EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
                        format!(
                            "t{}: the campaign's season turns - the demands go to the street.",
                            tick
                        ),
                    );
                } else if n == crate::game::campaign::CampaignPhase::Listening {
                    done.push(e); // a settled campaign ends its season
                }
            }
        }
        for e in done { self.world.remove_component::<crate::game::campaign::Campaign>(e); }
    }
}

/// One strength over every claim on the village (#971): what of the
/// flow the claim controls actually lands, how much of the village
/// still stands with it, less the drag the head's own style costs
fn claim_strength(flow: f32, loyalty: f32, drag: f32) -> f32 {
    (flow * 0.5 + loyalty * 0.5 - drag * 0.15).clamp(0.0, 1.0)
}

/// Below this the lord's writ runs out: the dissolution's own bar
const DISSOLVE_AT: f32 = 0.15;
/// Days the crown takes answering a broken writ - the reimposition
/// is always late
const RETURN_DAYS: u64 = 120;
/// The reach a new bailiff rides in with: back, but contestable
const RETURN_STRENGTH: f32 = 0.45;
/// The flow a returned writ starts on - it has to be re-won on the road
const RETURN_FLOW: f32 = 0.3;

/// The watcher behind the curtain: the lord's own eyes, and the
/// paranoia they feed - heavy eyes catch more, and catching more
/// makes the watching worse
#[derive(Debug, Clone)]
pub struct Watcher {
    pub paranoia: f32,   // what the eyes bring back feeds this
    pub legitimacy: f32, // what an exposure costs him
}

impl Game {
    /// The watcher's day: his eyes fall on the weak - a sick man, a
    /// poor man, a frightened man - and each name they carry home
    /// feeds the paranoia. A tight security culture keeps the eyes
    /// off the weak. And once in a long while the watchers are
    /// caught, and the watching costs him.
    pub(in crate::session) fn surveillance_tick(&mut self, tick: u64) {
        let org = self.org;
        let skill = self.world.get_component::<crate::game::org_skills::Organizing>(org)
            .map(|o| o.get(crate::game::org_skills::OrgSkill::SecurityCulture)).unwrap_or(0.0);
        let mut leaks = 0usize;
        for v in &self.villagers {
            if villager::is_dead(&self.world, *v) { continue; }
            let weak = self.world.get_component::<Health>(*v)
                .map(|h| h.condition < 0.5).unwrap_or(false)
                || self.world.get_component::<Purse>(*v)
                    .map(|p| p.gold < 1.0).unwrap_or(false);
            if weak { leaks += 1; }
        }
        // the discipline cuts the leaks; a practiced org keeps its own
        let mut effective = leaks as f32 * (1.0 - skill * 0.8);
        // the vetting: a deep-read org slows the eyes' accrual
        if self.counter_intel >= 2 { effective *= 0.6; }
        // but a loud cause leaks just by being big - no security culture
        // shrinks sheer size, so the muscle's bulk lands on top of the
        // vetted count: the watching scales with what it watches
        let muscle = self.world.get_component::<crate::sim::organized_power::OrganizedPower>(org)
            .map(|p| p.capacity).unwrap_or(0.0);
        effective += (muscle - 0.5).max(0.0) * 2.0;
        for site in &self.sites {
            let Some(l) = site.lord else { continue };
            let Some(w) = self.world.get_component_mut::<Watcher>(l) else { continue };
            w.paranoia = (w.paranoia + effective * 0.01).min(1.0);
            // and the watchers get caught: a high paranoia's a high
            // chance the eyes are found out, and the finding costs him
            if w.paranoia > 0.6 && (tick.wrapping_mul(31).wrapping_add(l as u64)) % 1000 < 2 {
                w.legitimacy = (w.legitimacy - 0.2).max(0.0);
                w.paranoia = 0.0;
                self.log.push_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Notable), format!("t{}: the watchers are caught in the open - the lord's eyes cost him his standing.", tick));
            }
        }
    }
}
