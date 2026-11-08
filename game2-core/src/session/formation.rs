//! A formation: folk moving together - a column on the streets for
//! the cause today, and the shape a parade or an escort takes tomorrow.

use crate::session::*;

/// What a column of folk is doing out together
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormationKind {
    /// The cause takes to the streets
    March,
    /// The whole village walks it in - on a feast day the column hands
    /// off to the market ring (carrying the feast's name, its day, and
    /// the stock: loaves, poured); on a victory it just walks and
    /// breaks up
    Parade { feast: Option<(&'static str, u32, u32, u64)> },
    /// A funeral: the column carries `Entity` (the dead) to their
    /// churchyard plot - the body rides the pallbearers' shoulders
    Funeral(Entity),
}

/// Folk walking a shared route, each in their slot of the column
pub struct Formation {
    pub kind: FormationKind,
    /// The seat the column belongs to - a march's outcome lands there
    pub seat: usize,
    /// The column, front to back - members[0] leads
    pub members: Vec<Entity>,
    /// Where the head's going, waypoint by waypoint
    pub route: Vec<(f32, f32)>,
    /// The waypoint the head's on
    pub at: usize,
    /// Frames the head's pushed at the current waypoint - a point it
    /// can't reach gets left behind rather than stood at forever
    pub stuck: u32,
    /// Frames the head's waited for the column to form - the grace
    /// runs on this, not on `stuck`, so the first leg gets a full
    /// stall budget of its own
    pub waited: u32,
    /// How far the head had to walk to the current waypoint when it
    /// became current - the stall budget scales with the leg, so a
    /// long walk to the rally point isn't mistaken for a wall
    pub leg0: f32,
    /// Who's already cheered this parade - the crowd's cheer lands once
    pub cheered: crate::engine::det::HashSet<Entity>,
}

/// The column's pace, in tiles a frame - a walk, not a run
const FORMATION_STEP: f32 = 0.035;
/// The file spacing: each holds a pace behind the one ahead
const SLOT_GAP: f32 = 0.8;

impl Game {
    /// One frame of the column: the head walks its route, everyone else
    /// keeps their slot a pace behind the one ahead. A wall in the way
    /// slides a body along it rather than stand it still; when the head
    ///'s done the route the column breaks and its outcome lands
    pub fn formation_step(&mut self) {
        let Some(mut f) = self.formation.take() else { return };
        // whoever fell out for good - died, got taken - leaves the file;
        // an empty column is no column
        let mut fallen: Vec<Entity> = vec![];
        f.members.retain(|&m| {
            let keep = self.world.get_component::<Pos>(m).is_some()
                && !villager::is_dead(&self.world, m);
            if !keep { fallen.push(m); }
            keep
        });
        for m in fallen {
            // a fallen marcher isn't marching: the marker has to leave
            // with them, or the day's drivers ignore them for good
            self.world.remove_component::<villager::InFormation>(m);
        }
        if f.members.is_empty() { self.formation = None; return; }
        // the column keeps the day's stride at any watch speed, the way
        // the walks do - turbo shouldn't leave a funeral crawling for a
        // month of game-days
        let pace = FORMATION_STEP * self.walk_pace().min(20.0);
        // the head steps toward its waypoint, then takes the next - but
        // it waits for the column to form first: a march that leaves
        // its people behind isn't a march
        if let (Some(&head), Some(&(tx, ty))) = (f.members.first(), f.route.get(f.at)) {
            if let Some(hp) = self.world.get_component::<Pos>(head).copied() {
                // a funeral can't keep the village waiting days for the
                // stragglers - the column marches with whoever's gathered
                let grace = if matches!(f.kind, FormationKind::Funeral(_)) { 360 } else { 1800 };
                let formed = f.waited > grace || f.members.iter().filter(|&&m| {
                    self.world.get_component::<Pos>(m)
                        .map(|p| (p.x - hp.x).powi(2) + (p.y - hp.y).powi(2) < 36.0).unwrap_or(false)
                }).count() * 2 > f.members.len();
                let (dx, dy) = (tx - hp.x, ty - hp.y);
                if f.leg0 < 0.0 {
                    f.leg0 = (dx * dx + dy * dy).sqrt();
                }
                if dx * dx + dy * dy < 0.35 {
                    f.at += 1; f.stuck = 0; f.waited = 0;
                    f.leg0 = f.route.get(f.at)
                        .map(|&(wx, wy)| ((wx - hp.x).powi(2) + (wy - hp.y).powi(2)).sqrt())
                        .unwrap_or(0.0);
                } else if formed || f.at > 0 {
                    self.step_toward(head, dx, dy, pace);
                    f.stuck += 1;
                    // a way it can't go, the column walks past: twice
                    // the leg's expected walk plus a slack, then the
                    // head takes the skipped leg's end so the next
                    // stretch starts from the road and not the wall it
                    // hugged. The budget rides the leg's own length, so
                    // the mourners' long walk to the body gets its
                    // patience while a boxed-in head still gets loose
                    if f.stuck as f32 > f.leg0 / pace * 2.0 + 30.0 {
                        f.at += 1; f.stuck = 0; f.waited = 0;
                        if let (Some(pp), Some(&(wx, wy))) = (
                            self.world.get_component_mut::<Pos>(head), f.route.get(f.at)) {
                            pp.x = wx; pp.y = wy;
                            f.leg0 = f.route.get(f.at + 1)
                                .map(|&(nx, ny)| ((nx - wx).powi(2) + (ny - wy).powi(2)).sqrt())
                                .unwrap_or(0.0);
                        }
                    }
                // a column that never forms still marches - the head
                // gives it a grace, then walks with whoever fell in
                } else { f.waited += 1; }
            }
        }
        // and every file keeps a pace behind the one in front, a step
        // to the side by turns so the column reads as files not a queue
        for i in 1..f.members.len() {
            let (Some(a), Some(p)) = (self.world.get_component::<Pos>(f.members[i - 1]).copied(),
                                      self.world.get_component::<Pos>(f.members[i]).copied())
                else { continue };
            let side = if i % 2 == 0 { 0.5 } else { -0.5 };
            let (gx, gy) = (a.x - p.x, a.y - p.y);
            let d = (gx * gx + gy * gy).sqrt();
            if d > SLOT_GAP {
                let want = (d - SLOT_GAP * 0.8).min(pace * 1.4);
                self.step_toward(f.members[i], gx + (-gy / d) * side * 0.3, gy + (gx / d) * side * 0.3, want);
            }
        }
        // the funeral's cargo rides the pallbearers' shoulders: the
        // body lies where it fell until a bearer reaches it, then sits
        // at the lead bearers' midpoint so a straggler at the file's
        // tail can't drag it back down the road
        if let FormationKind::Funeral(dead) = f.kind {
            if f.at >= 1 {
                let carriers: Vec<Entity> = self.world.get_component::<villager::Funeral>(dead)
                    .map(|fun| fun.carriers.clone()).unwrap_or_default();
                let head = f.members.first().and_then(|&h|
                    self.world.get_component::<Pos>(h).map(|p| (p.x, p.y)));
                // the bearers actually at the box: the carriers keeping
                // pace with the head, not the ones still back down the
                // road - the body rides them, not the whole file's mean
                let shoulders: Vec<(f32, f32)> = f.members.iter()
                    .filter(|m| carriers.contains(m))
                    .filter_map(|&c| self.world.get_component::<Pos>(c).map(|p| (p.x, p.y)))
                    .filter(|&(x, y)| head.map(|(hx, hy)|
                        (x - hx).powi(2) + (y - hy).powi(2) < 16.0).unwrap_or(true))
                    .take(4)
                    .collect();
                if !shoulders.is_empty() {
                    let n = shoulders.len() as f32;
                    let (sx, sy) = shoulders.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
                    // a bearer right at the body takes it up
                    let taken = self.world.get_component::<Pos>(dead)
                        .map(|b| shoulders.iter().any(|&(x, y)|
                            (x - b.x).powi(2) + (y - b.y).powi(2) < 9.0))
                        .unwrap_or(false);
                    if let Some(fun) = self.world.get_component_mut::<villager::Funeral>(dead) {
                        if taken { fun.carried = true; }
                        if fun.carried {
                            if let Some(dp) = self.world.get_component_mut::<Pos>(dead) {
                                dp.x = sx / n; dp.y = sy / n;
                            }
                        }
                    }
                }
            }
        }
        // a parade's crowd: whoever watches from the lane cheers a
        // little - once each, while the column's out
        if matches!(f.kind, FormationKind::Parade { .. }) {
            let spots: Vec<(f32, f32)> = f.members.iter()
                .filter_map(|&m| self.world.get_component::<Pos>(m).map(|p| (p.x, p.y)))
                .collect();
            for &v in &self.villagers.clone() {
                if f.cheered.contains(&v) { continue; }
                if self.world.get_component::<villager::InFormation>(v).is_some() { continue; }
                let Some(p) = self.world.get_component::<Pos>(v).copied() else { continue };
                if spots.iter().any(|&(x, y)| (p.x - x).powi(2) + (p.y - y).powi(2) < 64.0) {
                    f.cheered.insert(v);
                    desires::satisfy(&mut self.world, v, desires::Want::Joy, self.tick + 30);
                    desires::satisfy(&mut self.world, v, desires::Want::Music, self.tick + 30);
                }
            }
        }
        if f.at < f.route.len() {
            self.formation = Some(f);
        } else {
            let (seat, kind, members) = (f.seat, f.kind, f.members.clone());
            for &m in &members { self.world.remove_component::<villager::InFormation>(m); }
            match kind {
                FormationKind::March => self.march_resolved(seat),
                FormationKind::Parade { feast } => self.parade_resolved(feast, &members),
                FormationKind::Funeral(dead) => self.funeral_resolved(dead, &members),
            }
        }
    }

    /// The town's paved ways rung round the market cross, begun nearest
    /// the head - the route a column walks, march or parade alike
    pub(in crate::session) fn street_route(&self, from: (f32, f32), cap: usize) -> Vec<(f32, f32)> {
        let (wx0, wy0, wx1, wy1) = self.town_wall;
        let mut ring: Vec<(i32, i32)> = vec![];
        for y in wy0..=wy1 { for x in wx0..=wx1 {
            if self.map.get(x, y) == Some(Tile::Road) { ring.push((x, y)); }
        } }
        let (cx, cy) = {
            let n = ring.len().max(1) as f32;
            (ring.iter().map(|t| t.0 as f32).sum::<f32>() / n,
             ring.iter().map(|t| t.1 as f32).sum::<f32>() / n)
        };
        ring.sort_by(|a, b| (a.1 as f32 - cy).atan2(a.0 as f32 - cx)
            .partial_cmp(&(b.1 as f32 - cy).atan2(b.0 as f32 - cx)).unwrap());
        let mut route: Vec<(f32, f32)> = ring.iter().step_by(3)
            .map(|t| (t.0 as f32 + 0.5, t.1 as f32 + 0.5)).collect();
        let nearest = route.iter().enumerate()
            .min_by(|(_, a), (_, b)| ((a.0 - from.0).powi(2) + (a.1 - from.1).powi(2))
                .partial_cmp(&((b.0 - from.0).powi(2) + (b.1 - from.1).powi(2))).unwrap())
            .map(|(i, _)| i).unwrap_or(0);
        route.rotate_left(nearest);
        route.truncate(cap);
        route
    }

    /// Call the village folk out: the column forms, whoever was walking
    /// somewhere else falls in - the parade kind carries what it ends as
    pub(in crate::session) fn start_parade(&mut self, members: Vec<Entity>,
            route: Vec<(f32, f32)>, feast: Option<(&'static str, u32, u32, u64)>) {
        if self.formation.is_some() || members.is_empty() || route.is_empty() { return; }
        for &m in &members {
            self.travels.retain(|t| t.who != m);
            // whatever the day had them doing, the column has them now
            self.world.remove_component::<routine::Routine>(m);
            self.world.add_component(m, villager::InFormation);
        }
        self.formation = Some(Formation { kind: FormationKind::Parade { feast },
            seat: 0, members, route, at: 0, stuck: 0, waited: 0, leg0: -1.0, cheered: Default::default() });
    }

    /// Call the mourners out: the funeral column forms at the body and
    /// walks its route to the grave slot. False while another column is
    /// out - the burial waits for tomorrow's check
    pub(in crate::session) fn start_funeral(&mut self, dead: Entity, slot: (i32, i32),
            carriers: Vec<Entity>, officiant: Option<Entity>,
            members: Vec<Entity>, route: Vec<(f32, f32)>) -> bool {
        if self.formation.is_some() || members.is_empty() || route.len() < 2 { return false; }
        if carriers.is_empty() { return false; }
        for &m in &members {
            self.travels.retain(|t| t.who != m);
            // whatever the day had them doing, the funeral has them now
            self.world.remove_component::<routine::Routine>(m);
            self.world.add_component(m, villager::InFormation);
        }
        self.world.add_component(dead, villager::Funeral {
            slot, carriers, officiant, carried: false, since: self.tick });
        self.formation = Some(Formation { kind: FormationKind::Funeral(dead),
            seat: 0, members, route, at: 0, stuck: 0, waited: 0, leg0: -1.0, cheered: Default::default() });
        true
    }

    /// The column reaches the plot: the body is laid in the slot, the
    /// mourners' bonds and faith are touched, and the village remembers
    fn funeral_resolved(&mut self, dead: Entity, members: &[Entity]) {
        let Some(fun) = self.world.get_component::<villager::Funeral>(dead).map(|f| villager::Funeral {
            slot: f.slot, carriers: f.carriers.clone(), officiant: f.officiant,
            carried: f.carried, since: f.since })
        else { return };
        self.world.remove_component::<villager::Funeral>(dead);
        if let Some(p) = self.world.get_component_mut::<Pos>(dead) {
            *p = Pos { x: fun.slot.0 as f32, y: fun.slot.1 as f32 };
        }
        self.world.add_component(dead, villager::Buried);
        let tick = self.tick;
        // the mourners were kin by the grief: each standing friend draws
        // a little closer to the family the dead leaves
        let ties: Vec<Entity> = relationships::bonds_of(&self.world, dead).iter()
            .filter_map(|&r| self.world.get_component::<relationships::Relationship>(r)
                .map(|rel| if rel.from == dead { rel.other } else { rel.from }))
            .filter(|&t| !villager::is_dead(&self.world, t)
                && self.world.get_component::<Pos>(t).is_some())
            .take(8).collect();
        for &m in members {
            for &t in &ties {
                // a bearer is already one of the dead's ties - the
                // grief warms the living bonds, never a self-bond
                if t == m { continue; }
                relationships::record_interaction(&mut self.world, m,
                    relationships::Interaction { other: t,
                        kind: relationships::InteractionKind::Conversation,
                        intensity: 0.1 });
            }
            // and the faithful leave a funeral a little more devoted
            if let Some(fa) = self.world.get_component_mut::<crate::sim::religion::Faith>(m) {
                fa.devotion = (fa.devotion + 0.04).min(1.0);
            }
        }
        let name = self.name_of(dead);
        let officiant = fun.officiant.filter(|&o| !villager::is_dead(&self.world, o)
            && self.world.get_component::<Pos>(o).is_some());
        // priest, elder or lay leader - the log doesn't guess which
        let words = if officiant.is_some() { " The words are said." } else { "" };
        self.log.push(format!("t{}: {} is laid to rest in the churchyard.{}", tick, name, words));
        if let Some(o) = officiant {
            self.say(o, "Earth to earth.", TALK_BUBBLE_SECS * 2);
        }
    }

    /// A step along (dx, dy) toward the target, sliding along a wall in
    /// the way rather than standing still - and when the way's boxed in,
    /// slipping out by whichever neighbour still makes ground toward it
    fn step_toward(&mut self, e: Entity, dx: f32, dy: f32, want: f32) {
        let Some(p) = self.world.get_component::<Pos>(e).copied() else { return };
        let d = (dx * dx + dy * dy).sqrt();
        if d < 0.001 { return; }
        let (sx, sy) = (dx / d * want, dy / d * want);
        let (nx, ny) = if self.walkable(p.x + sx, p.y + sy) { (p.x + sx, p.y + sy) }
            else if self.walkable(p.x + sx, p.y) { (p.x + sx, p.y) }
            else if self.walkable(p.x, p.y + sy) { (p.x, p.y + sy) }
            else {
                // boxed in: every neighbour that still closes on the
                // target, nearest approach first - a column backs out of
                // a pocket rather than stall the funeral for a month
                let mut best: Option<(f32, f32, f32)> = None;
                for ay in -1i32..=1 {
                    for ax in -1i32..=1 {
                        if ax == 0 && ay == 0 { continue; }
                        let (ax, ay) = (ax as f32, ay as f32);
                        let (cx, cy) = (p.x + ax * want, p.y + ay * want);
                        if !self.walkable(cx, cy) { continue; }
                        let gain = (dx * (cx - p.x) + dy * (cy - p.y)) / d;
                        if best.map(|(_, _, g)| gain > g).unwrap_or(true) {
                            best = Some((cx, cy, gain));
                        }
                    }
                }
                match best { Some((bx, by, _)) => (bx, by), None => return }
            };
        if let Some(pp) = self.world.get_component_mut::<Pos>(e) { pp.x = nx; pp.y = ny; }
    }
}
