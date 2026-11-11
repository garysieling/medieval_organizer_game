//! First game mechanic: villager hunger cycle
//!
//! Villagers have Hunger that rises each tick. When hungry they seek
//! a FoodSource, move toward it, and eat when they arrive.
//! Demonstrates emergent behavior from simple rules on the ECS.

use crate::engine::ecs::{Entity, System, World};

/// Position on the 2.5D map
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

/// Villager hunger - 0 = starving, 100 = full
#[derive(Debug, Clone, Copy)]
pub struct Hunger {
    pub value: f32,
}

/// Marks an entity as a food source with limited servings
#[derive(Debug)]
pub struct FoodSource {
    pub servings: u32,
}

/// Current activity state of a villager
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Activity {
    Idle,
    SeekingFood,
    Eating,
}

/// Villager marker + state
#[derive(Debug)]
/// Marching in a formation: the day's other drivers - routines, drift,
/// food-seeking - leave them be while the column moves
pub struct InFormation;

/// Days spent at the schoolhouse desk - the marker a scholar carries,
/// whether or not the letters take
#[derive(Debug)]
pub struct Scholared {
    pub days: u32,
}

pub struct Villager {
    pub activity: Activity,
    pub target_food: Option<Entity>,
}

/// Movement is steered elsewhere - the windowed game's daily routines walk
/// them on real paths - so the idle drift here leaves them be (it knows
/// nothing of walls and houses, and hopped folk into them)
pub struct Steered;

/// Out on a protected long ride (the collector's remit): food-seeking and
/// the homeward pull leave him to it - the road's his errand now
pub struct OnTheRoad;

/// Riding a horse: a knight on a country journey (levy, tribute,
/// escort). Set when the long ride starts, cleared when it ends - the
/// mount is for the road, not the doorstep
pub struct Mounted;

/// Made a Christian of the parish or sworn to the old stones: the rite
/// of coming of age is once-only, marked here
#[derive(Clone, Copy)]
pub struct Initiated;

/// The coming-of-age rite is done; the household party is still owed
#[derive(Clone, Copy)]
pub struct DueParty;

/// Walking to the coming-of-age rite: `site` is the gather tile (the
/// church door, the stones, the grove), `by` who officiates
#[derive(Clone, Copy)]
pub struct PendingRite {
    pub site: (i32, i32),
    pub by: Entity,
    /// the old faith's rite at the stones or grove rather than the church
    pub old: bool,
    /// the day it was called - a rite that can't gather gives way
    pub since: u64,
}

/// Betrothed and walking to the wedding: `partner` the other half,
/// `site` where the ceremony gathers, `by` who officiates
#[derive(Clone, Copy)]
pub struct Betrothed {
    pub partner: Entity,
    pub site: (i32, i32),
    pub by: Entity,
    pub old: bool,
    pub since: u64,
}

/// Marks an entity as permanently dead. The windowed game removes `Health`
/// on death (for its grave/alive checks) rather than leaving it at 0, so
/// this marker is the durable record `is_dead` can still see afterward.
pub struct Dead;

/// The funeral has come: the dead have been laid in the churchyard and
/// their `Pos` is their grave slot - it never moves again. Until this
/// lands they lie in at the place they died (`Dead` alone).
pub struct Buried;

/// A funeral procession's cargo: the dead villager the column is
/// carrying to their churchyard plot. Carried by the pallbearers'
/// shoulders - their `Pos` is drawn along with the file each frame.
#[derive(Debug)]
pub struct Funeral {
    /// The grave slot the column is bound for
    pub slot: (i32, i32),
    /// The pallbearers whose shoulders the body rides
    pub carriers: Vec<Entity>,
    /// The one who says the words at the graveside, when the village
    /// could supply one
    pub officiant: Option<Entity>,
    /// The pallbearers have taken the body up - until then it lies
    /// where it fell while the column walks to it
    pub carried: bool,
    /// The day the procession formed
    pub since: u64,
}

/// What a villager smells of: the fold, the smoke, the cut. Recomputed
/// daily from vocation; the rain washes it off; it shows in talk and
/// damps the heart-helper's rolls. Texture, not a sim.
#[derive(Debug, Clone, Copy)]
pub struct Smell {
    pub intensity: f32,
    pub kind: &'static str,
}

/// A difference a villager carries: a lame leg, failing sight, a mind
/// that needs quiet. It's texture, not punchline - the village walks a
/// little slower for them, and theirs check on them.
#[derive(Debug, Clone, Copy)]
pub struct Condition {
    pub kind: &'static str,
    /// The day it lifts - u64::MAX for the ones old age doesn't give back
    pub until: u64,
}

/// Last night's cup still has them: they sleep it off instead of
/// working the day. The short lift was worth the lost morning.
#[derive(Debug, Clone, Copy)]
pub struct Drunk {
    /// The day they're sober again
    pub until: u64,
}

/// A day of care: someone checked on a villager carrying a Condition,
/// and the checker gave the morning to the visit instead of the bench.
/// Set by care_tick, spent by the day's work loop.
#[derive(Debug, Clone, Copy)]
pub struct CareVisit {
    pub day: u64,
}

/// A thing made to outlast its maker: a lute, a window, a fine toolset -
/// named, owned, and passed down. It gathers stories the way a person does.
#[derive(Debug)]
pub struct Heirloom {
    pub maker: Entity,
    pub owner: Entity,
    pub name: &'static str,
    pub born_tick: u64,
    /// What it's been through: "played at the feast of the Harvest Home"
    pub stories: Vec<String>,
}

/// Whether an entity has died: either its `Health` has hit 0, or it was
/// marked dead (e.g. by the windowed game after removing `Health`).
/// Entities with neither are never counted as dead.
pub fn is_dead(world: &World, e: Entity) -> bool {
    if world.get_component::<Dead>(e).is_some() { return true; }
    world.get_component::<crate::sim::illness::Health>(e).map(|h| h.condition <= 0.0).unwrap_or(false)
}

/// Hunger rises each tick (the dead no longer hunger, and the held are
/// fed at the castle's charge - they can't seek, so letting them hunger
/// would starve them where they stand)
pub struct HungerSystem;
impl System for HungerSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Hunger>() {
            if is_dead(world, e)
                || world.get_component::<crate::game::dungeon::Imprisoned>(e).is_some() { continue; }
            if let Some(h) = world.get_component_mut::<Hunger>(e) {
                h.value = (h.value - 1.0).max(0.0);
            }
        }
    }
}

/// Hungry villagers seek and eat food; the dead lie still and eat nothing
/// The tile edge a sim runs on: idle drifting keeps villagers inside it
/// (a fixed edge pinches anyone homed on a bigger map)
pub struct WorldBounds {
    pub w: f32,
    pub h: f32,
}

pub struct SeekFoodSystem;
impl System for SeekFoodSystem {
    fn run(&mut self, world: &mut World) {
        let hunger_threshold = 30.0;
        let eat_distance = 1.0;
        let bound = world.query::<WorldBounds>().first()
            .and_then(|&e| world.get_component::<WorldBounds>(e))
            .map(|b| (b.w - 2.0, b.h - 2.0))
            .unwrap_or((31.0, 31.0));

        // Find villagers and their state - the marching are already walking
        let villagers: Vec<Entity> = world.query::<Villager>().into_iter()
            .filter(|&v| world.get_component::<InFormation>(v).is_none()).collect();
        let snapshot: Vec<(Entity, Pos)> = villagers.iter()
            .filter_map(|&v| world.get_component::<Pos>(v).map(|p| (v, *p)))
            .collect();

        for v in villagers {
            // the held can't seek: they take what the jailer gives
            if is_dead(world, v)
                || world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some()
                || world.get_component::<OnTheRoad>(v).is_some() { continue; }
            let hunger = match world.get_component::<Hunger>(v) {
                Some(h) => h.value,
                None => continue,
            };
            let pos = *world.get_component::<Pos>(v).unwrap_or(&Pos { x: 0.0, y: 0.0 });

            // Decide activity
            let mut activity = world.get_component::<Villager>(v).unwrap().activity;
            let mut target = world.get_component::<Villager>(v).unwrap().target_food;

            if hunger < hunger_threshold {
                // Find nearest food source
                if target.is_none() {
                    let mut best: Option<(Entity, f32)> = None;
                    for f in world.query::<FoodSource>() {
                        let has_food = world.get_component::<FoodSource>(f).map(|fs| fs.servings > 0).unwrap_or(false);
                        if !has_food { continue; } // skip depleted sources, even if nearest
                        if let Some(fp) = world.get_component::<Pos>(f) {
                            let d = (fp.x - pos.x).abs() + (fp.y - pos.y).abs();
                            if best.map(|(_, bd)| d < bd).unwrap_or(true) {
                                best = Some((f, d));
                            }
                        }
                    }
                    target = best.map(|(f, _)| f);
                    activity = Activity::SeekingFood;
                }

                // Move toward or eat at target
                if let Some(f) = target {
                    if let Some(fp) = world.get_component::<Pos>(f) {
                        let dx = fp.x - pos.x;
                        let dy = fp.y - pos.y;
                        let dist = dx.abs() + dy.abs();

                        if dist <= eat_distance {
                            let served = world.get_component_mut::<FoodSource>(f)
                                .map(|fs| { if fs.servings > 0 { fs.servings -= 1; true } else { false } })
                                .unwrap_or(false);
                            if served {
                                if let Some(h) = world.get_component_mut::<Hunger>(v) {
                                    h.value = 100.0;
                                }
                                activity = Activity::Idle;
                                target = None;
                            } else {
                                // this source is empty - give up on it and
                                // look for another next tick, instead of
                                // getting stuck reporting Eating forever
                                activity = Activity::SeekingFood;
                                target = None;
                            }
                        } else {
                            // Step toward food (speed 1 per tick)
                            let step = 1.0f32.min(dist);
                            let nx = pos.x + dx.signum() * step.min(dx.abs());
                            let ny = pos.y + dy.signum() * (step - step.min(dx.abs()));
                            if let Some(p) = world.get_component_mut::<Pos>(v) {
                                p.x = nx;
                                p.y = ny;
                            }
                            activity = Activity::SeekingFood;
                        }
                    } else {
                        target = None; // food source gone
                    }
                }
            } else {
                activity = Activity::Idle;
                target = None;
                if world.get_component::<Steered>(v).is_some() {
                    if let Some(vill) = world.get_component_mut::<Villager>(v) {
                        vill.activity = activity;
                        vill.target_food = target;
                    }
                    continue;
                }
                // wander when idle: drift a little so they don't pile up
                let seed = (v as u64).wrapping_add((pos.x * 7.0 + pos.y * 13.0) as u64);
                let mut dx = ((seed % 7) as f32 - 3.0) * 0.06;
                let mut dy = (((seed / 7) % 7) as f32 - 3.0) * 0.06;
                // separation: idle villagers step away from crowded neighbours (e.g. after eating at the same food source)
                let personal_space = 1.5;
                for (o, op) in &snapshot {
                    if *o == v { continue; }
                    let (ox, oy) = (pos.x - op.x, pos.y - op.y);
                    let d = (ox * ox + oy * oy).sqrt();
                    if d < personal_space {
                        let (ux, uy) = if d > 1e-3 { (ox / d, oy / d) } else {
                            let a = (v as f32) * 2.399;
                            (a.cos(), a.sin())
                        };
                        dx += ux * 0.2;
                        dy += uy * 0.2;
                    }
                }
                if let Some(p) = world.get_component_mut::<Pos>(v) {
                    p.x = (p.x + dx).clamp(0.0, bound.0);
                    p.y = (p.y + dy).clamp(0.0, bound.1);
                }
            }

            if let Some(vill) = world.get_component_mut::<Villager>(v) {
                vill.activity = activity;
                vill.target_food = target;
            }
        }
    }
}

/// Health lost per tick while starving (hunger at 0)
pub const STARVATION_DAMAGE: f32 = 0.005;
/// Health regained per tick while well fed and not sick
pub const NOURISHED_HEALING: f32 = 0.002;
/// Hunger at or above which a villager counts as well fed
pub const WELL_FED: f32 = 50.0;
/// Health a well-fed body mends back up to on its own
pub const NOURISHED_HEALTH: f32 = 0.9;

/// Demo-only ECS system: going without food wears down health; a well-fed villager who
/// isn't sick slowly mends back to normal health. The dead (health 0) are
/// left alone.
pub struct NourishmentSystem;
impl System for NourishmentSystem {
    fn run(&mut self, world: &mut World) {
        use crate::sim::illness::{actively_sick, Disease, Health};
        // only a showing illness bars the mend: the brewing feel nothing
        // yet, and a full belly helps the mending along
        let sick: Vec<Entity> = world.query::<Disease>().iter()
            .filter_map(|d| world.get_component::<Disease>(*d)
                .filter(|d| actively_sick(d)).map(|d| d.patient))
            .collect();
        // a heat wave or a frost bite everyone a little each tick, same
        // as the crops take the drought - the weather touches health too
        use crate::sim::weather::{health_effect, CurrentWeather};
        let weather: f32 = world.query::<CurrentWeather>().iter()
            .filter_map(|w| world.get_component::<CurrentWeather>(*w))
            .map(health_effect)
            .sum();
        if weather != 0.0 {
            for e in world.query::<Hunger>() {
                let Some(h) = world.get_component_mut::<Health>(e) else { continue };
                if h.condition <= 0.0 { continue; }
                h.condition = (h.condition + weather).clamp(0.0, 1.0);
            }
        }
        nourishment_step(world, &sick);
    }
}

/// The nourishment beat itself: starving hurts, eating well mends the
/// unsick. Split out so a session that already drains health by its own
/// weather can take just this part (its own weather tick floors the
/// drain; this block would hit the sick twice otherwise)
pub fn nourishment_step(world: &mut World, sick: &[Entity]) {
    use crate::sim::illness::Health;
    for e in world.query::<Hunger>() {
        // the dead are past eating
        if is_dead(world, e) { continue; }
        let hunger = world.get_component::<Hunger>(e).unwrap().value;
        // the jail feeds its prisoners though their hunger stays put
        let held = world.get_component::<crate::game::dungeon::Imprisoned>(e).is_some();
        let Some(h) = world.get_component_mut::<Health>(e) else { continue };
        if h.condition <= 0.0 { continue; }
        if hunger <= 0.0 {
            if held { continue; }
            h.condition = (h.condition - STARVATION_DAMAGE).max(0.0);
        } else if hunger >= WELL_FED && !sick.contains(&e) && h.condition < NOURISHED_HEALTH {
            h.condition = (h.condition + NOURISHED_HEALING).min(NOURISHED_HEALTH);
        }
    }
}

/// Spawn a villager at a position
/// What the village calls the body it gave them - the suffrage question
/// turns on it, so it has to exist before the campaign can fight it
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Sex { Female, Male }

/// A newcomer to the village - stamped when the road brings them, so
/// the village can count who's new enough to be called a stranger
pub struct Migrant { pub arrived: u64 }

/// A country hand in for the season's work - drinks harder than the
/// village's own, and the night takes it out of him
pub struct Laborer { pub until: u64 }

/// In the stocks by the market: a body shamed is a body pinned - the
/// village feeds its theater, and the sentence's days run out
pub struct InStocks { pub until: u64 }

impl Sex {
    /// The body the spawn gives them, by the entity's own number -
    /// deterministic, and half the world either way
    pub fn from_id(e: Entity) -> Sex { if e % 2 == 0 { Sex::Female } else { Sex::Male } }
}

pub fn spawn_villager(world: &mut World, x: f32, y: f32) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Pos { x, y });
    world.add_component(e, Hunger { value: 100.0 });
    world.add_component(e, Sex::from_id(e));
    world.add_component(e, Villager {
        activity: Activity::Idle,
        target_food: None,
    });
    e
}

/// Spawn a food source at a position
pub fn spawn_food_source(world: &mut World, x: f32, y: f32, servings: u32) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Pos { x, y });
    world.add_component(e, FoodSource { servings });
    e
}

/// A food source that regrows: it ripens `growth` servings per tick in a
/// normal season, scaled by the climate (lean winters, climate drift), up
/// to `capacity`
#[derive(Debug)]
pub struct Crop {
    pub capacity: u32,
    pub growth: f32,   // servings per tick at a climate multiplier of 1.0
    pub progress: f32, // fraction of the next serving ripened so far
    pub grown: u32,    // total servings this crop has produced
}

/// Spawn a crop field: a food source that starts full and regrows
pub fn spawn_crop_field(world: &mut World, x: f32, y: f32, capacity: u32, growth: f32) -> Entity {
    let e = spawn_food_source(world, x, y, capacity);
    world.add_component(e, Crop { capacity, growth, progress: 0.0, grown: 0 });
    e
}

/// System: crops ripen at a rate set by the season and the climate; with no
/// climate in the world they grow at their base rate
pub struct CropGrowthSystem;
impl System for CropGrowthSystem {
    fn run(&mut self, world: &mut World) {
        use crate::sim::climate::{climate_multiplier, Climate};
        use crate::sim::weather::{food_effect, CurrentWeather};
        let climate = world.query::<Climate>().first()
            .and_then(|c| world.get_component::<Climate>(*c))
            .map(|c| climate_multiplier(c, "food"))
            .unwrap_or(1.0);
        // Any weather event under way (a drought, a frost) stunts the crops too
        let weather: f32 = world.query::<CurrentWeather>().iter()
            .filter_map(|w| world.get_component::<CurrentWeather>(*w))
            .map(food_effect)
            .product();
        let mult = climate * weather.max(0.0);
        for e in world.query::<Crop>() {
            let servings = world.get_component::<FoodSource>(e).map(|f| f.servings).unwrap_or(0);
            let Some(crop) = world.get_component_mut::<Crop>(e) else { continue };
            if servings >= crop.capacity {
                crop.progress = 0.0; // a full field doesn't stockpile
                continue;
            }
            crop.progress += crop.growth * mult;
            let ripe = crop.progress.floor() as u32;
            if ripe == 0 { continue; }
            let ripe = ripe.min(crop.capacity - servings);
            crop.progress -= ripe as f32;
            crop.grown += ripe;
            if let Some(f) = world.get_component_mut::<FoodSource>(e) {
                f.servings += ripe;
            }
        }
    }
}

/// Total servings every crop has produced so far
pub fn servings_grown(world: &World) -> u32 {
    world.query::<Crop>().iter()
        .filter_map(|c| world.get_component::<Crop>(*c).map(|c| c.grown))
        .sum()
}

/// A market stall: a food source stocked from a market. Each tick it takes
/// every unit of `good` the market holds and puts it out as servings, so
/// what workers make (e.g. bakers' bread) becomes meals villagers can eat
#[derive(Debug)]
pub struct Stall {
    pub market: Entity,
    pub good: String,
}

/// Spawn an empty stall selling `good` from `market`
pub fn spawn_stall(world: &mut World, x: f32, y: f32, market: Entity, good: &str) -> Entity {
    let e = spawn_food_source(world, x, y, 0);
    world.add_component(e, Stall { market, good: good.to_string() });
    e
}

/// Demo-only system: stalls take their good off the market and serve it
pub struct StallSystem;
impl System for StallSystem {
    fn run(&mut self, world: &mut World) {
        use crate::sim::economics::Market;
        for e in world.query::<Stall>() {
            let (market, good) = match world.get_component::<Stall>(e) {
                Some(s) => (s.market, s.good.clone()),
                None => continue,
            };
            let stock = world.get_component_mut::<Market>(market)
                .and_then(|m| m.supply.get_mut(&good))
                .map(|q| std::mem::take(q))
                .unwrap_or(0);
            if stock == 0 { continue; }
            if let Some(f) = world.get_component_mut::<FoodSource>(e) {
                f.servings += stock;
            }
        }
    }
}

/// Servings left across every food source; the drop between two readings
/// (plus anything grown in between) is how many meals were eaten
pub fn servings_left(world: &World) -> u32 {
    world.query::<FoodSource>().iter()
        .filter_map(|f| world.get_component::<FoodSource>(*f).map(|fs| fs.servings))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::Scheduler;

    #[test]
    fn test_stall_serves_market_bread() {
        use crate::sim::economics::Market;
        let mut w = World::new();
        let m = w.create_entity();
        w.add_component(m, Market::default());
        let stall = spawn_stall(&mut w, 0.0, 0.0, m, "bread");
        StallSystem.run(&mut w);
        assert_eq!(w.get_component::<FoodSource>(stall).unwrap().servings, 0);
        w.get_component_mut::<Market>(m).unwrap().add_supply("bread", 3);
        w.get_component_mut::<Market>(m).unwrap().add_supply("wheat", 5);
        StallSystem.run(&mut w);
        assert_eq!(w.get_component::<FoodSource>(stall).unwrap().servings, 3);
        let mkt = w.get_component::<Market>(m).unwrap();
        assert_eq!(mkt.supply["bread"], 0, "bread moves off the market onto the stall");
        assert_eq!(mkt.supply["wheat"], 5, "only the stall's own good is taken");

        // A hungry villager eats there
        let v = spawn_villager(&mut w, 0.0, 0.0);
        w.get_component_mut::<Hunger>(v).unwrap().value = 10.0;
        SeekFoodSystem.run(&mut w);
        SeekFoodSystem.run(&mut w);
        assert_eq!(w.get_component::<Hunger>(v).unwrap().value, 100.0);
        assert_eq!(w.get_component::<FoodSource>(stall).unwrap().servings, 2);
    }

    #[test]
    fn test_starving_hurts_and_eating_well_mends() {
        use crate::sim::illness::Health;
        let mut w = World::new();
        let v = spawn_villager(&mut w, 0.0, 0.0);
        w.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        w.get_component_mut::<Hunger>(v).unwrap().value = 0.0;
        for _ in 0..40 { NourishmentSystem.run(&mut w); }
        let starved = w.get_component::<Health>(v).unwrap().condition;
        assert!((starved - 0.7).abs() < 1e-4, "40 ticks starving should cost 0.2 health, got {}", starved);
        w.get_component_mut::<Hunger>(v).unwrap().value = 80.0;
        for _ in 0..200 { NourishmentSystem.run(&mut w); }
        assert_eq!(w.get_component::<Health>(v).unwrap().condition, NOURISHED_HEALTH);
    }

    #[test]
    fn test_the_sick_do_not_mend_from_food_alone() {
        use crate::sim::illness::{Disease, DiseaseKind, Health, Stage};
        let mut w = World::new();
        let v = spawn_villager(&mut w, 0.0, 0.0);
        w.add_component(v, Health { condition: 0.5, immunity: 0.5 });
        let d = w.create_entity();
        w.add_component(d, Disease { kind: DiseaseKind::Influenza, name: "flu".into(), patient: v, stage: Stage::Mild, severity: 0.2, contagious: false, incubation: 0, recovering: 0 });
        for _ in 0..50 { NourishmentSystem.run(&mut w); }
        assert_eq!(w.get_component::<Health>(v).unwrap().condition, 0.5);
    }

    #[test]
    fn test_the_dead_do_not_hunger_move_or_eat() {
        use crate::sim::illness::Health;
        let mut w = World::new();
        let food = spawn_food_source(&mut w, 0.0, 0.0, 3);
        let v = spawn_villager(&mut w, 5.0, 0.0);
        w.get_component_mut::<Hunger>(v).unwrap().value = 0.0;
        w.add_component(v, Health { condition: 0.0, immunity: 0.5 });
        for _ in 0..20 {
            HungerSystem.run(&mut w);
            SeekFoodSystem.run(&mut w);
        }
        assert_eq!(w.get_component::<FoodSource>(food).unwrap().servings, 3, "a corpse ate the village's food");
        assert_eq!(*w.get_component::<Pos>(v).unwrap(), Pos { x: 5.0, y: 0.0 }, "the dead don't walk");
        // and a living villager without Health is unaffected
        let alive = spawn_villager(&mut w, 1.0, 0.0);
        w.get_component_mut::<Hunger>(alive).unwrap().value = 10.0;
        for _ in 0..3 { SeekFoodSystem.run(&mut w); }
        assert_eq!(w.get_component::<FoodSource>(food).unwrap().servings, 2);
    }

    #[test]
    fn test_marked_dead_without_health_does_not_hunger_move_or_eat() {
        // Mirrors the windowed game: it removes Health on death and relies
        // on a separate Dead marker, since no-Health alone means "never
        // tracked death" (e.g. sims that don't model Health at all).
        let mut w = World::new();
        let food = spawn_food_source(&mut w, 0.0, 0.0, 3);
        let v = spawn_villager(&mut w, 5.0, 0.0);
        w.get_component_mut::<Hunger>(v).unwrap().value = 0.0;
        w.add_component(v, Dead);
        assert!(is_dead(&w, v));
        for _ in 0..20 {
            HungerSystem.run(&mut w);
            SeekFoodSystem.run(&mut w);
        }
        assert_eq!(w.get_component::<FoodSource>(food).unwrap().servings, 3, "a corpse ate the village's food");
        assert_eq!(*w.get_component::<Pos>(v).unwrap(), Pos { x: 5.0, y: 0.0 }, "the dead don't walk");
    }

    #[test]
    fn test_a_drought_stunts_the_crops_until_it_passes() {
        use crate::sim::weather::{spawn_weather, Weather, WeatherSystem};
        let grown_in = |drought: bool| {
            let mut w = World::new();
            let field = spawn_crop_field(&mut w, 0.0, 0.0, 100, 0.5);
            w.get_component_mut::<FoodSource>(field).unwrap().servings = 0;
            if drought { spawn_weather(&mut w, Weather::Drought, 1.0, 10); }
            for _ in 0..10 { CropGrowthSystem.run(&mut w); WeatherSystem.run(&mut w); }
            let during = w.get_component::<FoodSource>(field).unwrap().servings;
            for _ in 0..10 { CropGrowthSystem.run(&mut w); WeatherSystem.run(&mut w); }
            (during, w.get_component::<FoodSource>(field).unwrap().servings - during)
        };
        let (dry, after_dry) = grown_in(true);
        let (normal, after_normal) = grown_in(false);
        assert!(dry < normal, "a full drought should stunt growth: {} vs {}", dry, normal);
        assert_eq!(after_dry, after_normal, "once it passes, growth recovers");
    }

    fn harvest_field(world: &mut World, field: Entity) {
        world.get_component_mut::<FoodSource>(field).unwrap().servings = 0;
    }

    #[test]
    fn test_crop_regrows_up_to_capacity() {
        let mut world = World::new();
        let field = spawn_crop_field(&mut world, 0.0, 0.0, 5, 0.5);
        harvest_field(&mut world, field);
        for _ in 0..4 { CropGrowthSystem.run(&mut world); }
        assert_eq!(world.get_component::<FoodSource>(field).unwrap().servings, 2);
        for _ in 0..100 { CropGrowthSystem.run(&mut world); }
        assert_eq!(world.get_component::<FoodSource>(field).unwrap().servings, 5, "a field tops out at capacity");
        assert_eq!(servings_grown(&world), 5);
    }

    #[test]
    fn test_winter_crops_grow_slower_than_spring() {
        use crate::sim::climate::{self, Climate, Season};
        let grown_in = |season: Season| {
            let mut world = World::new();
            let c = climate::spawn_climate(&mut world);
            world.get_component_mut::<Climate>(c).unwrap().season = season;
            let field = spawn_crop_field(&mut world, 0.0, 0.0, 100, 0.5);
            harvest_field(&mut world, field);
            for _ in 0..40 { CropGrowthSystem.run(&mut world); }
            servings_grown(&world)
        };
        assert!(grown_in(Season::Winter) < grown_in(Season::Spring), "winter should be leaner");
    }

    #[test]
    fn test_plain_food_source_does_not_regrow() {
        let mut world = World::new();
        let store = spawn_food_source(&mut world, 0.0, 0.0, 3);
        harvest_field(&mut world, store);
        for _ in 0..50 { CropGrowthSystem.run(&mut world); }
        assert_eq!(world.get_component::<FoodSource>(store).unwrap().servings, 0);
    }

    #[test]
    fn test_servings_left_drops_by_meals_eaten() {
        let mut world = World::new();
        spawn_food_source(&mut world, 0.0, 0.0, 5);
        spawn_food_source(&mut world, 9.0, 9.0, 3);
        let v = spawn_villager(&mut world, 0.0, 0.0);
        world.get_component_mut::<Hunger>(v).unwrap().value = 10.0;
        let before = servings_left(&world);
        assert_eq!(before, 8);
        let mut sched = Scheduler::new();
        sched.add_system(SeekFoodSystem);
        sched.tick(&mut world);
        assert_eq!(before - servings_left(&world), 1, "one hungry villager eats one meal");
    }

    #[test]
    fn test_villager_eats_when_hungry() {
        let mut world = World::new();
        let v = spawn_villager(&mut world, 0.0, 0.0);
        spawn_food_source(&mut world, 3.0, 0.0, 5);

        // Make villager hungry
        world.get_component_mut::<Hunger>(v).unwrap().value = 10.0;

        let mut sched = Scheduler::new();
        sched.add_system(SeekFoodSystem);

        // Run ticks until villager reaches food
        for _ in 0..10 {
            sched.tick(&mut world);
            if world.get_component::<Hunger>(v).unwrap().value > 50.0 {
                break;
            }
        }

        // Villager should have eaten
        let hunger = world.get_component::<Hunger>(v).unwrap().value;
        assert!(hunger > 50.0, "villager should have eaten, hunger={}", hunger);
    }

    #[test]
    fn test_villager_does_not_get_stuck_eating_a_depleted_source() {
        let mut world = World::new();
        let v = spawn_villager(&mut world, 0.0, 0.0);
        spawn_food_source(&mut world, 0.5, 0.0, 0); // already empty, right next to the villager
        world.get_component_mut::<Hunger>(v).unwrap().value = 10.0;

        let mut sched = Scheduler::new();
        sched.add_system(SeekFoodSystem);
        for _ in 0..5 {
            sched.tick(&mut world);
        }

        let activity = world.get_component::<Villager>(v).unwrap().activity;
        assert_ne!(activity, Activity::Eating, "a villager at an empty source never actually ate, so it shouldn't report Eating");
        let hunger = world.get_component::<Hunger>(v).unwrap().value;
        assert!(hunger <= 10.0, "hunger should not be falsely reset to 100 by an empty source, got {}", hunger);
    }

    #[test]
    fn test_villager_finds_another_source_after_first_is_depleted() {
        let mut world = World::new();
        let v = spawn_villager(&mut world, 0.0, 0.0);
        spawn_food_source(&mut world, 0.5, 0.0, 0);  // empty, nearest
        spawn_food_source(&mut world, 3.0, 0.0, 5);  // has food, farther away
        world.get_component_mut::<Hunger>(v).unwrap().value = 10.0;

        let mut sched = Scheduler::new();
        sched.add_system(SeekFoodSystem);
        for _ in 0..10 {
            sched.tick(&mut world);
        }

        let hunger = world.get_component::<Hunger>(v).unwrap().value;
        assert!(hunger > 50.0, "villager should give up on the empty source and eat from the other one, hunger={}", hunger);
    }

    #[test]
    fn test_hunger_rises() {
        let mut world = World::new();
        let v = spawn_villager(&mut world, 0.0, 0.0);

        let mut sched = Scheduler::new();
        sched.add_system(HungerSystem);
        sched.tick(&mut world);

        assert_eq!(world.get_component::<Hunger>(v).unwrap().value, 99.0);
    }

    #[test]
    fn test_a_frost_bites_everyone_a_little() {
        use crate::sim::weather::{spawn_weather, Weather};
        use crate::sim::illness::Health;
        let mut world = World::new();
        let v = spawn_villager(&mut world, 0.0, 0.0);
        world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        spawn_weather(&mut world, Weather::Frost, 0.6, 15);

        let mut sched = Scheduler::new();
        sched.add_system(NourishmentSystem);
        sched.tick(&mut world);

        let c = world.get_component::<Health>(v).unwrap().condition;
        assert!(c < 0.9, "a frost bites everyone a little: {}", c);
    }
}
