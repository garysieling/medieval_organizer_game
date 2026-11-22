//! Hooks (#1243): the old wounds and hungers a villager carries from
//! their past. A hook colors their talk long before they'll speak of it;
//! a deep bond - yours, or a friend's in the village - finally lays it
//! to rest, and that's remembered. Some hooks leave the next layer behind.

use crate::engine::ecs::{Entity, World};

/// One authored old wound
#[derive(Debug)]
pub struct Hook {
    /// The talk status it lends while open (`self:hook:<slug>`)
    pub status: &'static str,
    /// What it is, in a phrase ("the brother who went to soldier")
    pub subject: &'static str,
    /// What they say when it's finally asked after
    pub laid: &'static str,
    /// The weight in the village it shifts once laid to rest
    pub standing: f32,
    /// The next layer it leaves behind, if any
    pub sequel: Option<usize>,
}

pub const HOOKS: &[Hook] = &[
    Hook { status: "hook:brother_soldier", subject: "the brother who went to soldier",
        laid: "He went with the lord's levy eleven years back and never wrote. There - I've said it aloud at last.",
        standing: 0.0, sequel: Some(1) },
    Hook { status: "hook:brother_fate", subject: "whether the brother deserted or died",
        laid: "A carter swore he saw him at Lincoln fair, alive and shamed. Deserted, then. Better that than the ditch.",
        standing: 0.03, sequel: None },
    Hook { status: "hook:miller_debt", subject: "the debt owed the miller",
        laid: "I've carried that debt like a stone. Paid now, every penny - I can look the man in the eye.",
        standing: 0.10, sequel: None },
    Hook { status: "hook:old_faith", subject: "the old faith they swore off",
        laid: "My mother rang the stones, and I spat on them to please the priest. I'll walk out there come spring.",
        standing: 0.0, sequel: None },
    Hook { status: "hook:child_buried", subject: "the child buried young",
        laid: "She'd be twelve this Lammas. I can say her name now without the floor going.",
        standing: 0.0, sequel: None },
    Hook { status: "hook:burnt_barn", subject: "the barn that burned on their watch",
        laid: "I'd been at the ale when the barn went. There - the whole village can know it now.",
        standing: 0.05, sequel: Some(6) },
    Hook { status: "hook:blamed_neighbor", subject: "the neighbor blamed for the fire",
        laid: "I let them blame Hob for that barn. I'll stand up in church and say otherwise.",
        standing: 0.08, sequel: None },
    Hook { status: "hook:broken_troth", subject: "the troth they broke",
        laid: "I promised myself to a girl in the next valley and never went back. I've sent word, late as it is.",
        standing: 0.0, sequel: None },
];

/// A hook a villager carries, and how it was laid to rest
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Carried {
    pub id: usize,
    /// Who laid it to rest, and when
    pub laid: Option<(Entity, u64)>,
    /// Whether their Talk's spoken of it since
    pub told: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Hooks {
    pub carried: Vec<Carried>,
}

fn mix(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x
}

/// Hooks someone can carry from the start: none that's another's sequel
pub fn starters() -> Vec<usize> {
    (0..HOOKS.len()).filter(|&i| !HOOKS.iter().any(|h| h.sequel == Some(i))).collect()
}

/// The 0-2 hooks a villager draws at setup, hashed off their index so no
/// shared RNG stream moves
pub fn seed(index: u64) -> Hooks {
    let s = starters();
    let h = mix(index ^ 0x4007_5EED);
    let n = (h % 3) as usize;
    let mut carried: Vec<Carried> = vec![];
    for k in 0..n {
        let id = s[(mix(h.wrapping_add(k as u64 + 1)) % s.len() as u64) as usize];
        if !carried.iter().any(|c| c.id == id) {
            carried.push(Carried { id, laid: None, told: false });
        }
    }
    Hooks { carried }
}

/// The first hook they've not laid to rest
pub fn open(world: &World, v: Entity) -> Option<usize> {
    world.get_component::<Hooks>(v)?.carried.iter().find(|c| c.laid.is_none()).map(|c| c.id)
}

/// The statuses their open hooks lend their talk
pub fn statuses(world: &World, v: Entity) -> Vec<&'static str> {
    world.get_component::<Hooks>(v)
        .map(|h| h.carried.iter().filter(|c| c.laid.is_none()).map(|c| HOOKS[c.id].status).collect())
        .unwrap_or_default()
}

/// Lay their first open hook to rest, `by` someone, surfacing its sequel.
/// Returns the hook laid.
pub fn lay(world: &mut World, v: Entity, by: Entity, day: u64) -> Option<usize> {
    let hooks = world.get_component_mut::<Hooks>(v)?;
    let c = hooks.carried.iter_mut().find(|c| c.laid.is_none())?;
    c.laid = Some((by, day));
    let id = c.id;
    if let Some(next) = HOOKS[id].sequel {
        if !hooks.carried.iter().any(|c| c.id == next) {
            hooks.carried.push(Carried { id: next, laid: None, told: false });
        }
    }
    Some(id)
}

/// A laid hook their Talk hasn't spoken of yet, and who laid it
pub fn untold(world: &World, v: Entity) -> Option<(usize, Entity)> {
    world.get_component::<Hooks>(v)?.carried.iter()
        .find_map(|c| c.laid.filter(|_| !c.told).map(|(by, _)| (c.id, by)))
}

pub fn tell(world: &mut World, v: Entity, id: usize) {
    if let Some(h) = world.get_component_mut::<Hooks>(v) {
        if let Some(c) = h.carried.iter_mut().find(|c| c.id == id) { c.told = true; }
    }
}

/// What they say of it after: to the one who asked, or of the friend who did
pub fn remembered(id: usize, by_you: bool, friend: &str) -> String {
    let subject = HOOKS[id].subject;
    if by_you { format!("You're the one who finally asked after {subject}.") }
    else { format!("{friend} finally got me to speak of {subject}.") }
}
