//! Relationships: attachment-informed bonds between villagers.
//! Emergent: bonds form from interactions, not assigned.

use crate::engine::ecs::{Entity, World};
use crate::engine::det::HashMap;

/// Pair index: unordered entity pair -> the earliest Relationship
/// entity between them. find_relationship consults it only while the
/// index is provably complete: `rev` tracks the storage's own mutation
/// revision, so any bond change the index didn't fold - a bare
/// add_component, a removal - leaves it behind, and lookups fall back
/// to a scan until the next indexed bond rebuilds it in one pass.
#[derive(Default)]
struct RelIndex {
    by_pair: HashMap<(Entity, Entity), Entity>,
    /// World::mutation_rev::<Relationship> the index last folded through
    rev: u64,
}

fn pair_key(a: Entity, b: Entity) -> (Entity, Entity) {
    (a.min(b), a.max(b))
}

/// One pass over every live bond -> earliest entity per pair, marked
/// complete. Runs when a bare component change put bonds the index
/// never saw into the world - rare outside hand-built test setups.
fn rebuild_index(world: &mut World) {
    let mut by_pair: HashMap<(Entity, Entity), Entity> = HashMap::default();
    world.iter::<Relationship>(|e, r| {
        by_pair.entry(pair_key(r.from, r.other))
            .and_modify(|cur| { if e < *cur { *cur = e; } })
            .or_insert(e);
    });
    let rev = world.mutation_rev::<Relationship>();
    let idx = world.resource_or_default::<RelIndex>();
    idx.by_pair = by_pair;
    idx.rev = rev;
}

/// The lowest-id live Relationship matching `pred` - the answer a sorted
/// query() + first-match always gave, without the alloc or the sort.
fn lowest_matching(world: &World, matches: impl Fn(&Relationship) -> bool) -> Option<Entity> {
    let mut best: Option<Entity> = None;
    world.iter::<Relationship>(|e, r| {
        if matches(r) && best.map(|b| e < b).unwrap_or(true) { best = Some(e); }
    });
    best
}

/// The trust half of the bond, Mayer's three faces of it (E6-1, #1431):
/// each interaction moves the face it truly touches rather than one
/// pooled warmth - the deed done for them reads benevolent *and* able
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Trust {
    /// Can they get it done (competence read)
    pub ability: f32,
    /// Do they mean you well
    pub benevolence: f32,
    /// Do they keep their word
    pub integrity: f32,
}

/// Clark's psychological-safety ladder, carried as data (E6-1, #1431) -
/// the unlocks (a come-along, meeting roles, open dissent) and the
/// "breaks when" demotions land with E8-1's act model; today it just
/// records where a bond's nerve has reached
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SafetyRung {
    /// You're one of us
    #[default]
    Inclusion,
    /// It's safe to ask and learn
    Learner,
    /// It's safe to offer and do
    Contributor,
    /// It's safe to disagree openly
    Challenger,
}

impl SafetyRung {
    /// a step back down the ladder - a broken bond's nerve goes before
    /// its warmth does
    pub fn lower(self) -> Self {
        match self {
            SafetyRung::Challenger => SafetyRung::Contributor,
            SafetyRung::Contributor => SafetyRung::Learner,
            _ => SafetyRung::Inclusion,
        }
    }
}

/// A relationship between two entities, stored as its own entity (like
/// Organization or HouseMeeting) so each person can hold many of them.
/// `strength` is the derived summary the readers know (hearts,
/// FOLLOWER_TIE); the parts under it - intimacy, the three trusts -
/// are what interactions actually move (E6-1, #1431)
#[derive(Debug, Default)]
pub struct Relationship {
    pub from: Entity,
    pub other: Entity,
    pub kind: RelKind,
    pub strength: f32,      // -1 to 1
    pub interactions: u32,  // how many times they've interacted
    /// The bond's parts (E6-1): talks, meals and time feed intimacy;
    /// deeds and betrayals feed the trusts; the rung marks their nerve
    pub intimacy: f32,
    pub trust: Trust,
    pub safety: SafetyRung,
    /// Times they turned your words aside instead of answering (E6-3):
    /// the reproach beat of E8-3 reads this later
    pub pivots: u32,
    /// Times they stepped around you; enough and the nerve closes
    pub deflects: u32,
}

impl Relationship {
    /// `strength` is a summary, not a state: derived from the parts as
    /// their weighted sum (all weights 1 - the deltas each kind pushes
    /// are sized to land the old strength delta exactly). Reconcile
    /// what the parts don't know about: warmth arriving from outside -
    /// a seeded bond, a hand-set strength - parks under intimacy;
    /// warmth above the cap is saturation, not coldness, and shaves
    /// every part alike rather than inventing a grievance
    fn reconcile(&mut self) {
        let derived = self.intimacy + self.trust.ability
            + self.trust.benevolence + self.trust.integrity;
        let diff = self.strength - derived;
        if diff.abs() <= 1e-6 {
            return;
        }
        if diff > 0.0 || derived <= 1e-6 {
            self.intimacy += diff;
        } else {
            let keep = self.strength / derived;
            self.intimacy *= keep;
            self.trust.ability *= keep;
            self.trust.benevolence *= keep;
            self.trust.integrity *= keep;
        }
    }
    /// the summary the readers know, off the parts
    fn derive(&mut self) {
        self.strength = (self.intimacy + self.trust.ability
            + self.trust.benevolence + self.trust.integrity).clamp(-1.0, 1.0);
    }
    /// reconcile what the parts don't know about, then re-derive
    pub fn recompose(&mut self) {
        self.reconcile();
        self.derive();
    }
}

/// Friendship the way the player reads it: warmth shown as hearts, in
/// half-heart steps, five at most. Cold bonds show none - a rival's
/// bad blood isn't hearts unfilled, it's the other thing entirely.
pub const MAX_HEARTS: f32 = 5.0;

/// `strength` (-1..1) read as hearts (0..MAX_HEARTS): the warm half of
/// the axis only, so a grudge never dips below empty - and snapped to
/// the half-heart grid, since the daily drift would otherwise leave the
/// meter showing 0.495 where the game only ever deals in halves
pub fn hearts_of(strength: f32) -> f32 {
    let h = strength.clamp(0.0, 1.0) * MAX_HEARTS;
    (h * 2.0).round() / 2.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// The marriage's own mark: who the bond's to, and how it ended if it
/// did - annulled (never valid) or deserted (one walked, the claim
/// stayed). Carried on each spouse while the union stands
pub struct Married {
    /// Who the union's with
    pub spouse: Entity,
    /// How it ended, if it did: 0 stands, 1 annulled, 2 deserted
    pub form: u8,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum RelKind {
    #[default]
    Stranger,
    Acquaintance,
    Friend,
    Rival,
    Family,
    Lover,
    Mentor,
    Apprentice,
}

/// An interaction event between two entities
#[derive(Debug)]
pub struct Interaction {
    pub other: Entity,
    pub kind: InteractionKind,
    pub intensity: f32,
}

#[derive(Debug)]
pub enum InteractionKind {
    Conversation,
    SharedMeal,
    /// A day of work done shoulder to shoulder (#1344): the work
    /// party's warmth - closer than a shared table, short of a deed
    SharedWork,
    Conflict,
    Aid,
    Betrayal,
    Romance,
}

/// System: interactions deepen relationships, kinds evolve
pub struct RelationshipSystem;
impl crate::engine::ecs::System for RelationshipSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Relationship>() {
            if let Some(r) = world.get_component_mut::<Relationship>(e) {
                // Decay toward neutral if no recent interactions - the
                // parts drift and the summary follows (E6-1); the first
                // recompose back-fills a seeded strength before it drifts
                r.recompose();
                r.intimacy *= 0.99;
                r.trust.ability *= 0.99;
                r.trust.benevolence *= 0.99;
                r.trust.integrity *= 0.99;
                r.derive();
            }
            if let Some(r) = world.get_component::<Relationship>(e) {
                let kind = infer_kind(r);
                world.get_component_mut::<Relationship>(e).unwrap().kind = kind;
            }
        }
    }
}

/// Create the entity a Relationship lives on and register it in the pair
/// index. Bonds that come in through here resolve in O(1) from then on -
/// prefer it over bare create_entity + add_component.
pub fn add_relationship(world: &mut World, r: Relationship) -> Entity {
    let e = world.create_entity();
    let key = pair_key(r.from, r.other);
    world.add_component(e, r);
    let rev = world.mutation_rev::<Relationship>();
    let indexed_rev = world.resource::<RelIndex>().map(|i| i.rev).unwrap_or(0);
    if indexed_rev + 1 == rev {
        // the index was complete before this bond - fold it in. An
        // existing entry already holds the pair's earliest bond, and e
        // is the newest id, so or_insert keeps the ordering a sorted
        // scan always favoured
        let idx = world.resource_or_default::<RelIndex>();
        idx.by_pair.entry(key).or_insert(e);
        idx.rev = rev;
    } else {
        // a bare add_component slipped bonds past the index - rebuild
        // once rather than scan per insert
        rebuild_index(world);
    }
    e
}

/// The bond entities `v` is a side of, in creation order - ties settle
/// the way a sorted query() always did, so picks stay reproducible.
/// For a walk over many villagers' bonds at once, build a `bond_map`
/// instead of calling this per villager.
pub fn bonds_of(world: &World, v: Entity) -> Vec<Entity> {
    let mut out = world.query::<Relationship>();
    out.retain(|&e| world.get_component::<Relationship>(e)
        .map(|r| r.from == v || r.other == v).unwrap_or(false));
    out
}

/// One pass over every bond -> who holds bonds to whom (each list in
/// creation order). For code walking many villagers' bonds in one go,
/// building this once beats a full scan per villager; strength and kind
/// still come off the components.
pub fn bond_map(world: &World) -> HashMap<Entity, Vec<Entity>> {
    let mut m: HashMap<Entity, Vec<Entity>> = HashMap::default();
    for e in world.query::<Relationship>() {
        let Some(r) = world.get_component::<Relationship>(e) else { continue };
        m.entry(r.from).or_default().push(e);
        if r.other != r.from { m.entry(r.other).or_default().push(e); }
    }
    m
}

/// Find the relationship entity between `from` and `other`, if any exists
/// (checked in both directions, since either side may have recorded the
/// first interaction).
pub fn find_relationship(world: &World, from: Entity, other: Entity) -> Option<Entity> {
    let matches = |r: &Relationship|
        (r.from == from && r.other == other) || (r.from == other && r.other == from);
    if let Some(idx) = world.resource::<RelIndex>() {
        // a complete index answers in O(1); a stale one defers to the
        // scan, since an unindexed bond could predate any indexed entry
        if idx.rev == world.mutation_rev::<Relationship>() {
            return match idx.by_pair.get(&pair_key(from, other)) {
                // the entry only counts if the entity still holds this
                // bond - a destroyed relationship leaves a stale key
                Some(&e) if world.get_component::<Relationship>(e)
                    .map(&matches).unwrap_or(false) => Some(e),
                Some(_) => lowest_matching(world, &matches),
                None => None,
            };
        }
    }
    lowest_matching(world, matches)
}

/// How a reply's responsiveness lands on the bond's parts (E6-3,
/// #1441): the reflect/validate/care/advise/deflect/pivot tags each
/// push their own components. A deflect does nothing - but enough of
/// them closes the nerve; a pivot costs and is remembered (`pivots`,
/// the reproach beat of E8-3 consumes it later).
pub fn push_respond(world: &mut World, from: Entity, other: Entity,
                    respond: crate::game::conversation::Respond) {
    use crate::game::conversation::Respond;
    let e = find_relationship(world, from, other).unwrap_or_else(|| {
        add_relationship(world, Relationship {
            from, other, kind: RelKind::Stranger, strength: 0.0, interactions: 0,
            ..Default::default()
        })
    });
    if let Some(r) = world.get_component_mut::<Relationship>(e) {
        r.recompose();
        match respond {
            Respond::Reflect => { r.intimacy += 0.06; r.trust.benevolence += 0.08; }
            Respond::Validate => { r.intimacy += 0.04; }
            Respond::Care => { r.trust.benevolence += 0.15; }
            Respond::Advise => { r.intimacy += 0.05; r.trust.ability += 0.10; }
            Respond::Deflect => {
                r.deflects += 1;
                if r.deflects % 3 == 0 { r.safety = r.safety.lower(); }
            }
            Respond::PivotAsk => {
                r.intimacy -= 0.05; r.trust.integrity -= 0.08; r.pivots += 1;
            }
        }
        r.derive();
        r.interactions += 1;
    }
}

/// Record an interaction - relationships evolve emergently. Finds the
/// relationship entity between `from` and the interaction's target (or
/// creates one), applies the interaction's effect on strength, and
/// re-infers the relationship kind from the new state.
pub fn record_interaction(world: &mut World, from: Entity, interaction: Interaction) -> Entity {
    // Gains are sized in hearts (strength*MAX_HEARTS is what's shown):
    // a talk warms a half heart, a shared table a whole one, a deed
    // done for them two - and unkindness takes them back the same way.
    // Each kind pushes the part(s) it truly moves (E6-1's map); the
    // parts sum to the old delta, so strength - the derived summary -
    // lands exactly where the pooled model put it
    let s = interaction.intensity.clamp(0.0, 1.0);
    let push = |r: &mut Relationship| match interaction.kind {
        InteractionKind::Conversation => r.intimacy += 0.10 * s,
        InteractionKind::SharedMeal => r.intimacy += 0.20 * s,
        InteractionKind::SharedWork => { r.intimacy += 0.15 * s; r.trust.ability += 0.10 * s; }
        InteractionKind::Conflict => { r.intimacy -= 0.10 * s; r.trust.integrity -= 0.10 * s; }
        InteractionKind::Aid => { r.trust.benevolence += 0.25 * s; r.trust.ability += 0.15 * s; }
        InteractionKind::Betrayal => { r.intimacy -= 0.10 * s; r.trust.benevolence -= 0.15 * s;
            r.trust.integrity -= 0.15 * s; r.safety = r.safety.lower(); }
        InteractionKind::Romance => { r.intimacy += 0.20 * s; r.trust.benevolence += 0.10 * s; }
    };
    let other = interaction.other;

    let rel_entity = find_relationship(world, from, other).unwrap_or_else(|| {
        add_relationship(world, Relationship {
            from, other, kind: RelKind::Stranger, strength: 0.0, interactions: 0,
            ..Default::default()
        })
    });

    if let Some(r) = world.get_component_mut::<Relationship>(rel_entity) {
        r.recompose();   // reconcile a seeded strength before the push lands
        push(r);
        r.derive();
        r.interactions += 1;
    }
    if let Some(r) = world.get_component::<Relationship>(rel_entity) {
        let kind = infer_kind(r);
        world.get_component_mut::<Relationship>(rel_entity).unwrap().kind = kind;
    }
    rel_entity
}

/// Infer relationship kind from history (emergent). Explicitly assigned
/// roles (Family, Lover, Mentor, Apprentice) are fixed and are never
/// overwritten by the strength/interaction-based inference below.
pub fn infer_kind(r: &Relationship) -> RelKind {
    if matches!(r.kind, RelKind::Family | RelKind::Lover | RelKind::Mentor | RelKind::Apprentice) {
        return r.kind;
    }
    if r.interactions == 0 {
        // Untouched pairs stay Stranger; neutral strength alone shouldn't age into Acquaintance.
        return RelKind::Stranger;
    }
    if r.strength > 0.6 {
        match r.interactions {
            0..=5 => RelKind::Acquaintance,
            _ => RelKind::Friend,
        }
    } else if r.strength < -0.3 {
        RelKind::Rival
    } else {
        RelKind::Acquaintance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::System;

    #[test]
    fn record_interaction_creates_a_relationship() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();

        let e = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Conversation, intensity: 1.0 });

        let r = world.get_component::<Relationship>(e).expect("relationship should exist");
        assert_eq!(r.from, a);
        assert_eq!(r.other, b);
        assert_eq!(r.interactions, 1);
        assert!((r.strength - 0.10).abs() < 1e-6);
    }

    #[test]
    fn repeated_interactions_accumulate_on_the_same_relationship() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();

        let e1 = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Conversation, intensity: 1.0 });
        let e2 = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Conversation, intensity: 1.0 });

        assert_eq!(e1, e2, "a second interaction with the same person should update the existing relationship, not create a new one");
        let r = world.get_component::<Relationship>(e1).unwrap();
        assert_eq!(r.interactions, 2);
        assert!((r.strength - 0.20).abs() < 1e-5);
    }

    #[test]
    fn relationship_lookup_is_bidirectional() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Aid, intensity: 1.0 });

        assert!(find_relationship(&world, a, b).is_some());
        assert!(find_relationship(&world, b, a).is_some(), "recording from A should also be visible when looking up from B's side");
    }

    #[test]
    fn strength_is_clamped_to_valid_range() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let mut last = Entity::default();
        for _ in 0..50 {
            last = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Romance, intensity: 1.0 });
        }
        let r = world.get_component::<Relationship>(last).unwrap();
        assert!(r.strength <= 1.0 && r.strength >= -1.0);
    }

    #[test]
    fn betrayals_can_turn_a_relationship_into_a_rivalry() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let mut e = Entity::default();
        for _ in 0..5 {
            e = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Betrayal, intensity: 1.0 });
        }
        let r = world.get_component::<Relationship>(e).unwrap();
        assert_eq!(r.kind, RelKind::Rival);
    }

    #[test]
    fn fixed_kinds_survive_interactions() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let e = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Conversation, intensity: 1.0 });
        world.get_component_mut::<Relationship>(e).unwrap().kind = RelKind::Family;

        record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Conversation, intensity: 1.0 });

        let r = world.get_component::<Relationship>(e).unwrap();
        assert_eq!(r.kind, RelKind::Family, "a Family bond should not be reclassified as Acquaintance by an ordinary interaction");
    }

    #[test]
    fn low_intensity_interactions_affect_bonds_less() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let weak = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Conversation, intensity: 0.1 });
        let r = world.get_component::<Relationship>(weak).unwrap();
        assert!((r.strength - 0.01).abs() < 1e-6, "a low-intensity interaction should change strength proportionally less than a full-intensity one");
    }

    #[test]
    fn rival_label_clears_once_strength_decays_back_to_neutral() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let mut e = Entity::default();
        for _ in 0..5 {
            e = record_interaction(&mut world, a, Interaction { other: b, kind: InteractionKind::Betrayal, intensity: 1.0 });
        }
        assert_eq!(world.get_component::<Relationship>(e).unwrap().kind, RelKind::Rival);

        let mut system = RelationshipSystem;
        for _ in 0..200 {
            system.run(&mut world);
        }

        let r = world.get_component::<Relationship>(e).unwrap();
        assert!(r.strength > -0.3, "strength should have decayed back above the rivalry threshold");
        assert_ne!(r.kind, RelKind::Rival, "kind should be re-inferred once strength decays out of rivalry range");
    }

    #[test]
    fn a_handmade_bond_keeps_its_place_when_a_bond_is_indexed_later() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        // a bond added bare - the index never sees it made
        let early = world.create_entity();
        world.add_component(early, Relationship {
            from: a, other: b, kind: RelKind::Family, strength: 0.8, interactions: 0, ..Default::default()
        });
        // then an indexed bond lands on the same pair - a duplicate, the
        // way a second Family or a stray record can be
        let later = add_relationship(&mut world, Relationship {
            from: b, other: a, kind: RelKind::Acquaintance, strength: 0.1, interactions: 1, ..Default::default()
        });
        assert_eq!(find_relationship(&world, a, b), Some(early),
            "the earliest record still wins the pair, indexed or not");
        assert_eq!(find_relationship(&world, b, a), Some(early));
        assert_ne!(later, early, "the new bond still exists, it just doesn't own the pair");
    }

    #[test]
    fn an_indexed_removal_and_a_bare_add_still_resolves() {
        // remove an indexed bond and add a bare one elsewhere: coverage
        // changed under the index - the bare bond can't go invisible
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let c = world.create_entity();
        let d = world.create_entity();
        let ab = add_relationship(&mut world, Relationship {
            from: a, other: b, kind: RelKind::Friend, strength: 0.5, interactions: 3, ..Default::default()
        });
        world.remove_component::<Relationship>(ab);
        let cd = world.create_entity();
        world.add_component(cd, Relationship {
            from: c, other: d, kind: RelKind::Friend, strength: 0.9, interactions: 9, ..Default::default()
        });
        assert_eq!(find_relationship(&world, c, d), Some(cd),
            "the bare bond resolves, so a second interaction lands on it");
        let e = record_interaction(&mut world, c, Interaction {
            other: d, kind: InteractionKind::Conversation, intensity: 1.0 });
        assert_eq!(e, cd, "no duplicate bond gets created for the pair");
        assert_eq!(world.get_component::<Relationship>(cd).unwrap().interactions, 10);
    }

    #[test]
    fn hearts_read_the_warm_half_of_strength_in_halves() {
        assert_eq!(hearts_of(0.0), 0.0);
        assert_eq!(hearts_of(0.1), 0.5, "a strength step of a tenth is half a heart");
        assert_eq!(hearts_of(0.5), 2.5);
        assert_eq!(hearts_of(1.0), MAX_HEARTS);
        assert_eq!(hearts_of(-0.4), 0.0, "bad blood shows no hearts at all");
        // a day's drift off the grid still reads as the half heart it was
        assert_eq!(hearts_of(0.099), 0.5, "decay doesn't make 0.495 hearts");
    }

    #[test]
    fn a_full_talk_is_half_a_heart_and_a_deed_is_two() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let talk = record_interaction(&mut world, a, Interaction {
            other: b, kind: InteractionKind::Conversation, intensity: 1.0 });
        assert_eq!(hearts_of(world.get_component::<Relationship>(talk).unwrap().strength), 0.5);
        let deed = record_interaction(&mut world, a, Interaction {
            other: b, kind: InteractionKind::Aid, intensity: 1.0 });
        // the talk's half heart plus the deed's two
        assert_eq!(hearts_of(world.get_component::<Relationship>(deed).unwrap().strength), 2.5);
    }

    #[test]
    fn untouched_stranger_stays_a_stranger_through_decay_ticks() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let e = world.create_entity();
        world.add_component(e, Relationship {
            from: a, other: b, kind: RelKind::Stranger, strength: 0.0, interactions: 0, ..Default::default()
        });

        let mut system = RelationshipSystem;
        for _ in 0..5 {
            system.run(&mut world);
        }

        let r = world.get_component::<Relationship>(e).unwrap();
        assert_eq!(r.interactions, 0);
        assert_eq!(r.kind, RelKind::Stranger, "a pair with zero interactions should not age into Acquaintance on its own");
    }
}

#[cfg(test)]
mod e6_1 {
    use super::*;
    use crate::engine::ecs::System;

    fn strength_after(kind: InteractionKind, intensity: f32) -> (f32, Relationship) {
        let mut w = World::new();
        let (a, b) = (w.create_entity(), w.create_entity());
        let e = record_interaction(&mut w, a, Interaction { other: b, kind, intensity });
        let r = w.get_component::<Relationship>(e).unwrap();
        (r.strength, Relationship {
            from: r.from, other: r.other, kind: r.kind,
            strength: r.strength, interactions: r.interactions,
            intimacy: r.intimacy, trust: r.trust, safety: r.safety,
            pivots: r.pivots, deflects: r.deflects,
        })
    }

    /// the parts move what the kind truly moves, and the derived
    /// summary lands the old pooled delta exactly
    #[test]
    fn interactions_move_the_parts_not_the_pool() {
        let (s, r) = strength_after(InteractionKind::Aid, 1.0);
        assert!((s - 0.40).abs() < 1e-6);
        assert!((r.trust.benevolence - 0.25).abs() < 1e-6);
        assert!((r.trust.ability - 0.15).abs() < 1e-6);
        assert_eq!(r.intimacy, 0.0);

        let (s, r) = strength_after(InteractionKind::Betrayal, 1.0);
        assert!((s + 0.40).abs() < 1e-6);
        assert!((r.intimacy + 0.10).abs() < 1e-6);
        assert!((r.trust.benevolence + 0.15).abs() < 1e-6);
        assert!((r.trust.integrity + 0.15).abs() < 1e-6);
        assert_eq!(r.safety, SafetyRung::Inclusion, "a betrayal's nerve was never earned");

        let (s, _r) = strength_after(InteractionKind::SharedWork, 1.0);
        assert!((s - 0.25).abs() < 1e-6);
        let (s, _r) = strength_after(InteractionKind::Romance, 1.0);
        assert!((s - 0.30).abs() < 1e-6);
        let (s, _) = strength_after(InteractionKind::Conversation, 1.0);
        assert!((s - 0.10).abs() < 1e-6);
        let (s, _) = strength_after(InteractionKind::SharedMeal, 1.0);
        assert!((s - 0.20).abs() < 1e-6);
        let (s, r) = strength_after(InteractionKind::Conflict, 1.0);
        assert!((s + 0.20).abs() < 1e-6);
        assert!((r.intimacy + 0.10).abs() < 1e-6);
        assert!((r.trust.integrity + 0.10).abs() < 1e-6);
    }

    /// gifts past the cap saturate the trust parts; decay must shave
    /// them alike, not invent coldness where none was given
    #[test]
    fn saturated_trust_does_not_drain_intimacy() {
        let mut w = World::new();
        let (a, b) = (w.create_entity(), w.create_entity());
        for _ in 0..3 {
            record_interaction(&mut w, a, Interaction {
                other: b, kind: InteractionKind::Aid, intensity: 1.0,
            });
        }
        let e = find_relationship(&w, a, b).unwrap();
        let before = w.get_component::<Relationship>(e).unwrap();
        assert!(before.trust.ability + before.trust.benevolence > 1.0);
        assert_eq!(before.strength, 1.0, "the summary caps at one");

        RelationshipSystem.run(&mut w);   // a day of decay

        let r = w.get_component::<Relationship>(e).unwrap();
        assert!(r.intimacy >= 0.0,
            "saturated trust must not drain intimacy: {}", r.intimacy);
        assert!(r.strength > 0.97 && r.strength < 1.0,
            "decay shaved the parts, not invented a grievance: {}", r.strength);
    }

    /// a betrayal costs the bond its nerve, whatever it had reached
    #[test]
    fn betrayal_knocks_the_rung_down() {
        let mut w = World::new();
        let (a, b) = (w.create_entity(), w.create_entity());
        let e = add_relationship(&mut w, Relationship {
            from: a, other: b, kind: RelKind::Friend, strength: 0.6, interactions: 4,
            safety: SafetyRung::Challenger, ..Default::default()
        });
        record_interaction(&mut w, a, Interaction {
            other: b, kind: InteractionKind::Betrayal, intensity: 1.0,
        });
        let r = w.get_component::<Relationship>(e).unwrap();
        assert_eq!(r.safety, SafetyRung::Contributor);
    }

    /// a bond seeded with bare strength back-fills intimacy on the
    /// first touch, so decay and pushes read it like recorded warmth
    #[test]
    fn a_seeded_bond_survives_recompose() {
        let mut w = World::new();
        let (a, b) = (w.create_entity(), w.create_entity());
        let e = w.create_entity();
        w.add_component(e, Relationship {
            from: a, other: b, kind: RelKind::Friend,
            strength: 0.8, interactions: 5, ..Default::default()
        });
        // one decay tick: 0.8 * 0.99, exactly the old trajectory
        RelationshipSystem.run(&mut w);
        let r = w.get_component::<Relationship>(e).unwrap();
        assert!((r.strength - 0.8 * 0.99).abs() < 1e-6);
        assert!(r.intimacy > 0.0, "seeded warmth parked under intimacy");
        // and a talk lands on top of it - the seeded warmth holds,
        // the push adds on
        record_interaction(&mut w, a, Interaction {
            other: b, kind: InteractionKind::Conversation, intensity: 1.0 });
        let r = w.get_component::<Relationship>(e).unwrap();
        assert!((r.strength - (0.8 * 0.99 + 0.10)).abs() < 1e-4,
            "strength {} kept the seed and the talk", r.strength);
    }
}
