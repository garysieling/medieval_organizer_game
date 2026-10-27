//! Integrated village simulation - all sim domains wired to shared entities.
//! Villagers have hunger, health, attachment, and trade at a market.
//! Evidence: emergent interplay between systems on real entities.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::villager::{self, Hunger, Villager, Activity, Pos};
use game2_core::sim::psychology::{AttachmentHistory, infer_style};
use game2_core::sim::illness::{Health, Disease, Stage, IllnessSystem};
use game2_core::sim::climate::{self, SeasonSystem, climate_multiplier};
use game2_core::sim::economics::{self, Market, ProductionSystem};
use game2_core::infra::telemetry::Telemetry;

fn main() {
    println!("=== INTEGRATED VILLAGE SIM ===\n");
    println!("Villagers have hunger + health + attachment; market + climate interact\n");

    let mut world = World::new();
    let mut telem = Telemetry::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(ProductionSystem);
    sched.add_system(IllnessSystem);

    // Spawn 4 villagers with full component sets
    let mut villagers = vec![];
    for i in 0..4 {
        let v = villager::spawn_villager(&mut world, (i * 3) as f32, 0.0);
        world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        world.add_component(v, AttachmentHistory {
            anxiety_episodes: i as u32,
            warmth_episodes: (3 - i) as u32,
            episodes: vec![],
        });
        villagers.push(v);
    }
    villager::spawn_food_source(&mut world, 6.0, 6.0, 200);
    let climate = climate::spawn_climate(&mut world);
    let market = economics::spawn_market(&mut world);
    world.get_component_mut::<Market>(market).unwrap().add_supply("food", 20);
    economics::spawn_producer(&mut world, "food", 10.0);

    println!("tick,hunger_avg,health_avg,seeking,idle,sick,infected,season,food_price,food_stock\n");
    for tick in 0..200 {
        sched.tick(&mut world);

        // Villagers getting hungry shop at the market; their demand moves the price
        let demand = villagers.iter()
            .filter(|v| world.get_component::<Hunger>(**v)
                .map(|h| h.value < 60.0).unwrap_or(false))
            .count() as u32;
        if let Some(mkt) = world.get_component_mut::<Market>(market) {
            mkt.update_price("food", demand);
            let stock = mkt.supply.get("food").copied().unwrap_or(0);
            mkt.buy("food", demand.min(stock));
        }

        // Random villager gets sick at tick 50 (flu outbreak); it's contagious,
        // so whoever spends time near them may catch it too
        if tick == 50 {
            let d = world.create_entity();
            world.add_component(d, Disease {
                name: "flu".into(), patient: villagers[0], stage: Stage::Mild,
                severity: 0.4, contagious: true,
            });
            if let Some(h) = world.get_component_mut::<Health>(villagers[0]) {
                h.condition -= 0.2;
            }
        }

        if tick % 20 == 0 {
            let mut hunger_sum = 0.0;
            let mut health_sum = 0.0;
            let (mut seeking, mut idle, mut sick) = (0, 0, 0);
            for v in &villagers {
                if let Some(h) = world.get_component::<Hunger>(*v) { hunger_sum += h.value; }
                if let Some(h) = world.get_component::<Health>(*v) {
                    health_sum += h.condition;
                    if h.condition < 0.8 { sick += 1; }
                }
                if let Some(vil) = world.get_component::<Villager>(*v) {
                    match vil.activity {
                        Activity::SeekingFood => seeking += 1,
                        _ => idle += 1,
                    }
                }
            }
            // everyone carrying the flu, however mild
            let infected = world.query::<Disease>().len();
            let cl = world.get_component::<climate::Climate>(climate).unwrap();
            let mkt = world.get_component::<Market>(market).unwrap();
            let (price, stock) = (mkt.prices["food"], mkt.supply["food"]);
            println!("{},{:.1},{:.2},{},{},{},{},{:?},{:.2},{}",
                tick, hunger_sum / 4.0, health_sum / 4.0,
                seeking, idle, sick, infected, cl.season, price, stock);
            telem.set_tick(tick as u64);
            telem.gauge("hunger", (hunger_sum / 4.0) as f64);
            telem.gauge("health", (health_sum / 4.0) as f64);
        }
    }

    println!("\n=== ATTACHMENT STYLES (emerged from histories) ===");
    for (i, v) in villagers.iter().enumerate() {
        if let Some(h) = world.get_component::<AttachmentHistory>(*v) {
            println!("villager{}: anx={} warm={} -> {:?}", i, h.anxiety_episodes, h.warmth_episodes, infer_style(h));
        }
    }
    println!("\n=== SIM COMPLETE ===");
}
