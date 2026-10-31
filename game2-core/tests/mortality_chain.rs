// Integration test: a death runs through every system that should notice it.
// Each piece has unit tests; this checks they still work together - the dead
// stop eating and worshipping, the bereaved grieve, the dog is rehomed, the
// guild drops them and passes on leadership, and they stop aging.

use game2_core::engine::ecs::{Entity, Scheduler, World};
use game2_core::game::lifecycle::{Age, AgingSystem, LifeStage, Vitality};
use game2_core::game::organizing::living_leader;
use game2_core::game::pets::{self, Pet, PetRole, Species};
use game2_core::game::relationships::{RelKind, Relationship};
use game2_core::game::villager::{self, Hunger, Pos};
use game2_core::sim::community::{Citizen, InfluenceSystem, Organization};
use game2_core::sim::illness::{Disease, DiseaseKind, Health, IllnessSystem};
use game2_core::sim::psychology::{AttachmentHistory, Bond, GriefSystem};
use game2_core::sim::religion::{self, Congregation, Faith, Service};

struct Village {
    world: World,
    sched: Scheduler,
    elder: Entity,
    child: Entity,
    friend: Entity,
    dog: Entity,
    guild: Entity,
    church: Entity,
}

fn village() -> Village {
    let mut world = World::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(villager::NourishmentSystem);
    sched.add_system(IllnessSystem);
    sched.add_system(GriefSystem);
    sched.add_system(pets::PetBondSystem);
    sched.add_system(InfluenceSystem);
    sched.add_system(AgingSystem);

    villager::spawn_food_source(&mut world, 0.0, 0.0, 10);
    let person = |w: &mut World, x: f32, influence: f32| {
        let v = villager::spawn_villager(w, x, 0.0);
        w.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        w.add_component(v, AttachmentHistory { anxiety_episodes: 0, warmth_episodes: 0, episodes: vec![] });
        w.add_component(v, Citizen { influence, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        w.add_component(v, Faith { religion: "Old Faith".into(), devotion: 0.5, traditions: vec![] });
        w.add_component(v, Age { years: 40.0, stage: LifeStage::Adult });
        w.add_component(v, Vitality { health: 1.0, max_age: 90.0, cause_of_death: None });
        v
    };
    let elder = person(&mut world, 0.5, 0.3);
    let child = person(&mut world, 1.0, 0.1);
    let friend = person(&mut world, 1.5, 0.4);
    world.add_component(child, Bond { target: elder, security: 0.8 });
    let r = world.create_entity();
    world.add_component(r, Relationship { from: friend, other: elder, kind: RelKind::Friend, strength: 0.7, interactions: 9 });

    let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
    pets::adopt(&mut world, dog, elder);
    let guild = world.create_entity();
    world.add_component(guild, Organization {
        name: "Harvesters".into(), members: vec![child, friend], leader: elder,
        influence_pool: 0.0, resources: 0.0, charter: "Mutual aid".into(),
    });
    let church = world.create_entity();
    world.add_component(church, Congregation {
        name: "Old Faith".into(), members: vec![elder, child, friend], leader: None, orthodoxy: 0.5, services: vec![],
    });
    Village { world, sched, elder, child, friend, dog, guild, church }
}

#[test]
fn a_death_ripples_through_the_village() {
    let mut v = village();
    // The elder is struck down by a fatal illness
    let d = v.world.create_entity();
    v.world.add_component(d, Disease::new(DiseaseKind::Consumption, v.elder, 1.0));
    v.world.get_component_mut::<Health>(v.elder).unwrap().condition = 0.005;
    v.world.get_component_mut::<Health>(v.elder).unwrap().immunity = 0.0;
    v.sched.tick(&mut v.world);
    assert!(villager::is_dead(&v.world, v.elder), "the elder should have died");

    // The bereaved grieve, once
    for _ in 0..3 { v.sched.tick(&mut v.world); }
    let losses = |w: &World, e: Entity| w.get_component::<AttachmentHistory>(e).unwrap().anxiety_episodes;
    assert_eq!(losses(&v.world, v.child), 1, "the child loses their attachment figure");
    assert_eq!(losses(&v.world, v.friend), 1, "the close friend grieves");

    // The dog goes to the elder's closest friend
    assert_eq!(v.world.get_component::<Pet>(v.dog).unwrap().owner, Some(v.friend));

    // The guild passes leadership to its most influential living member
    assert_eq!(living_leader(&mut v.world, v.guild), Some(v.friend));
    let members = &v.world.get_component::<Organization>(v.guild).unwrap().members;
    assert!(!members.contains(&v.elder));

    // The dead stop aging, don't eat, and don't come to services
    let age = v.world.get_component::<Age>(v.elder).unwrap();
    assert_eq!(age.stage, LifeStage::Dead);
    let years = age.years;
    // (starving and right beside the food, a living villager would walk over and eat)
    v.world.get_component_mut::<Hunger>(v.elder).unwrap().value = 0.0;
    let pos = *v.world.get_component::<Pos>(v.elder).unwrap();
    for _ in 0..20 { v.sched.tick(&mut v.world); }
    assert_eq!(v.world.get_component::<Age>(v.elder).unwrap().years, years);
    assert_eq!(*v.world.get_component::<Pos>(v.elder).unwrap(), pos, "the dead don't walk to the food");
    assert_eq!(v.world.get_component::<Hunger>(v.elder).unwrap().value, 0.0, "nor eat");
    let attended = religion::perform_service(&mut v.world, v.church, Service {
        name: "Rite".into(), frequency: "weekly".into(),
        participants: vec![v.elder, v.child], cohesion_boost: 0.1, charity_output: 0.0,
    });
    assert_eq!(attended, 1, "only the living child attends");
}

#[test]
fn old_age_death_is_the_same_death() {
    let mut v = village();
    // The elder reaches the end of their natural span
    v.world.get_component_mut::<Age>(v.elder).unwrap().years = 89.95;
    v.sched.tick(&mut v.world);
    assert!(villager::is_dead(&v.world, v.elder), "dying of old age should be a death the rest of the sim sees");
    v.sched.tick(&mut v.world);
    assert_eq!(v.world.get_component::<Pet>(v.dog).unwrap().owner, Some(v.friend));
    assert_eq!(v.world.get_component::<AttachmentHistory>(v.child).unwrap().anxiety_episodes, 1);
}
