//! A formation: folk moving together - a column on the streets for
//! the cause today, and the shape a parade or an escort takes tomorrow.

use crate::session::*;

/// What a column of folk is doing out together
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormationKind {
    /// The cause takes to the streets
    March,
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
        // the head steps toward its waypoint, then takes the next - but
        // it waits for the column to form first: a march that leaves
        // its people behind isn't a march
        if let (Some(&head), Some(&(tx, ty))) = (f.members.first(), f.route.get(f.at)) {
            if let Some(hp) = self.world.get_component::<Pos>(head).copied() {
                let formed = f.members.iter().filter(|&&m| {
                    self.world.get_component::<Pos>(m)
                        .map(|p| (p.x - hp.x).powi(2) + (p.y - hp.y).powi(2) < 36.0).unwrap_or(false)
                }).count() * 2 > f.members.len();
                let (dx, dy) = (tx - hp.x, ty - hp.y);
                if dx * dx + dy * dy < 0.2 { f.at += 1; f.stuck = 0; }
                else if formed || f.at > 0 {
                    self.step_toward(head, dx, dy, FORMATION_STEP);
                    f.stuck += 1;
                    // a way it can't go, the column walks past
                    if f.stuck > 240 { f.at += 1; f.stuck = 0; }
                }
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
                let want = (d - SLOT_GAP * 0.8).min(FORMATION_STEP * 1.4);
                self.step_toward(f.members[i], gx + (-gy / d) * side * 0.3, gy + (gx / d) * side * 0.3, want);
            }
        }
        if f.at < f.route.len() {
            self.formation = Some(f);
        } else {
            let seat = f.seat;
            for &m in &f.members { self.world.remove_component::<villager::InFormation>(m); }
            if f.kind == FormationKind::March { self.march_resolved(seat); }
        }
    }

    /// A step along (dx, dy) toward the target, sliding along a wall in
    /// the way rather than standing still
    fn step_toward(&mut self, e: Entity, dx: f32, dy: f32, want: f32) {
        let Some(p) = self.world.get_component::<Pos>(e).copied() else { return };
        let d = (dx * dx + dy * dy).sqrt();
        if d < 0.001 { return; }
        let (sx, sy) = (dx / d * want, dy / d * want);
        let (nx, ny) = if self.walkable(p.x + sx, p.y + sy) { (p.x + sx, p.y + sy) }
            else if self.walkable(p.x + sx, p.y) { (p.x + sx, p.y) }
            else if self.walkable(p.x, p.y + sy) { (p.x, p.y + sy) }
            else { return };
        if let Some(pp) = self.world.get_component_mut::<Pos>(e) { pp.x = nx; pp.y = ny; }
    }
}
