//! Hooks laid to rest (#1243): asked after by the organizer at a deep
//! bond, or off-screen by a villager's own close friend or kin.

use crate::session::*;
use crate::game::hooks::{self, HOOKS};
use crate::game::relationships::{self, Interaction, InteractionKind, RelKind, Relationship};

/// The bond a friend needs to lay another's hook to rest
pub const FRIEND_TIE: f32 = 0.7;
/// A season's chance a close friend lays it to rest: one in this many
pub const FRIEND_IN: u64 = 4;

impl Game {
    /// Lay a hook to rest: the standing it shifts, the log that marks it
    fn lay_hook(&mut self, v: Entity, by: Entity) -> Option<usize> {
        let tick = self.tick;
        let id = hooks::lay(&mut self.world, v, by, tick)?;
        if let Some(c) = self.world.get_component_mut::<crate::sim::community::Citizen>(v) {
            c.influence = (c.influence + HOOKS[id].standing).clamp(0.0, 1.0);
        }
        let (who, friend) = (self.name_of(v), self.name_of(by));
        self.log.push_ev(EvTag::new(Thread::Player, Valence::Positive, Sev::Minor),
            format!("t{}: {} laid {}'s old trouble to rest - {}.", tick, friend, who, HOOKS[id].subject));
        Some(id)
    }

    /// The organizer asks after the picked villager's old wound
    pub fn ask_after_at(&mut self, seat: usize) -> String {
        let v = self.villagers[self.seats[seat].selected_target];
        let pe = self.seats[seat].player;
        if crate::game::organizing::stage_of(&self.world, pe, v) < crate::game::organizing::RelStage::Follower {
            return "They'd not speak of that to you - not yet.".into();
        }
        let Some(id) = self.lay_hook(v, pe) else { return "There's nothing they're carrying.".into() };
        self.say(v, HOOKS[id].laid, TALK_BUBBLE_SECS * 2);
        relationships::record_interaction(&mut self.world, pe,
            Interaction { other: v, kind: InteractionKind::Aid, intensity: 1.0 });
        format!("{} speaks at last of {}.", self.name_of(v), HOOKS[id].subject)
    }

    /// Each season, a villager with an open hook and a close friend or
    /// kin in the village may have it laid to rest by them
    pub fn hooks_tick(&mut self, tick: u64) {
        use crate::game::mentorship::{roll, SEASON};
        if tick == 0 || tick % SEASON != 0 { return; }
        for v in self.villagers.clone() {
            if villager::is_dead(&self.world, v) || hooks::open(&self.world, v).is_none() { continue; }
            let friend = self.villagers.iter().copied().find(|&f| {
                f != v && !villager::is_dead(&self.world, f) && !crate::game::dungeon::unavailable(&self.world, f)
                    && relationships::find_relationship(&self.world, v, f)
                        .and_then(|r| self.world.get_component::<Relationship>(r))
                        .map(|r| matches!(r.kind, RelKind::Friend | RelKind::Family) && r.strength >= FRIEND_TIE)
                        .unwrap_or(false)
            });
            let Some(f) = friend else { continue };
            if roll(tick, v, 0x400C ^ f as u64, FRIEND_IN) { self.lay_hook(v, f); }
        }
    }
}
