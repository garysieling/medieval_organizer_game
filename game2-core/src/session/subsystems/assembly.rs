//! The accountability assembly: the campaign's own session on its own
//! calendar. It calls itself on cadence, publishes the agenda ahead,
//! sits once it's had its days, and reads each pledge's deadline
//! against what the village saw. The organizer can call it early -
//! the institution doesn't need permission to keep its own dates.

use crate::session::*;

impl Game {
    /// The day's assembly work, for every campaign under way
    pub fn assembly_tick(&mut self, tick: u64) {
        let campaigns = self.world.query::<crate::game::campaign::Campaign>();
        for c in campaigns {
            // the institution establishes itself a while into the fight
            if self.world.get_component::<crate::game::campaign::Assembly>(c).is_none() {
                let founded = self.world.get_component::<crate::game::campaign::Campaign>(c)
                    .map(|cc| cc.founded_day).unwrap_or(tick);
                if tick >= founded + crate::game::campaign::FIRST_ASSEMBLY_DELAY {
                    self.world.add_component(c, crate::game::campaign::Assembly {
                        cadence_days: crate::game::campaign::ASSEMBLY_CADENCE,
                        last_held: founded, announced: None, agenda: vec![],
                        last_turnout: 0, musterered: vec![], held: 0, pledges: vec![],
                    });
                }
            }
            // call itself when the cadence's due and no agenda stands
            let due = self.world.get_component::<crate::game::campaign::Assembly>(c)
                .map(|a| a.announced.is_none()
                    && tick.saturating_sub(a.last_held) >= a.cadence_days)
                .unwrap_or(false);
            if due {
                if let Some(questions) = crate::game::campaign::announce_assembly(&mut self.world, c, tick) {
                    self.log.push(format!("t{}: the accountability assembly is called - the agenda's posted: {}.",
                        tick, questions.join("; ")));
                }
            }
            // sit once the published agenda's had its days
            if let Some(out) = crate::game::campaign::hold_assembly(&mut self.world, c, tick) {
                let names: Vec<String> = out.attendees.iter()
                    .filter_map(|a| self.villagers.iter().position(|v| v == a).map(|i| vill_name(i).to_string()))
                    .collect();
                self.log.push(format!("t{}: the assembly sits - {} answer the call; the leader's yes and no go on record.",
                    tick, names.join(", ")));
                for p in &out.pledges {
                    self.log.push(format!("t{}: asked \"{}\" the leader says {}.",
                        tick, p.question, if p.answer { "yes" } else { "no" }));
                }
                // a member's testimony can amend the leader's word
                // before it's inked - the record shows who checked it
                for (who, question) in &out.challenges {
                    let name = self.villagers.iter().position(|v| v == who)
                        .map(|i| vill_name(i).to_string()).unwrap_or_default();
                    self.log.push(format!("t{}: {} challenges the answer on \"{}\" - the record's amended.",
                        tick, name, question));
                }
            }
            // read the pledges whose deadlines have come
            for v in crate::game::campaign::resolve_pledges(&mut self.world, c, tick) {
                self.log.push(format!("t{}: the pledge {} - \"{}\"",
                    tick, if v.kept { "held" } else { "broke" }, v.question));
            }
        }
    }
}
