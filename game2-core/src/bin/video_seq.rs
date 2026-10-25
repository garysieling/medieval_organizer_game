//! E2: Frame sequence - render sim frames as BMPs that could be a video.
//! The "video" is a numbered sequence of renders showing the world evolve.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger, Villager, Pos};
use game2_core::ui::isometric::{render_terrain, render_entity};
use game2_core::sim::climate::{self, SeasonSystem};

fn main() {
    println!("=== FRAME SEQUENCE (video) ===\n");

    let mut world = World::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(SeasonSystem);

    let terrain = Terrain::generate(25, 0.7, 7);
    let climate = climate::spawn_climate(&mut world);
    villager::spawn_food_source(&mut world, 8.0, 8.0, 200);

    // Villagers that move around
    let mut villagers = vec![];
    for i in 0..4 {
        villagers.push(villager::spawn_villager(&mut world, (i*5) as f32, (i%3) as f32));
    }

    std::fs::create_dir_all("evidence/frames").unwrap();

    // Render a frame every 20 ticks
    for tick in 0..200 {
        sched.tick(&mut world);

        if tick % 20 == 0 {
            let frame_num = tick / 20;
            let mut buf = render_terrain(&terrain, 300, 200);
            // Draw villagers at their positions
            let colors = [(255,80,80),(80,255,80),(80,80,255),(255,255,80)];
            for (i, v) in villagers.iter().enumerate() {
                if let Some(p) = world.get_component::<Pos>(*v) {
                    let c = colors[i];
                    render_entity(&mut buf, p.x, p.y, c.0, c.1, c.2);
                }
            }
            let path = format!("evidence/frames/frame_{:03}.bmp", frame_num);
            buf.save_bmp(&path).unwrap();
            println!("frame {}: {}x{} (tick {}, season {:?})",
                frame_num, buf.width, buf.height, tick,
                world.get_component::<climate::Climate>(climate).unwrap().season);
        }
    }
    println!("\n=== {} frames rendered ===", 10);
}
