//! Diagnostic probe: what the village's innards actually do over a run.
//! Not committed to the suite - a workbench tool for tuning the tester.

use game2_core::session::{driver, Game};
use game2_core::game::villager::{self, Hunger, Pos, Villager, Activity, FoodSource};
use game2_core::game::livelihood::Purse;

fn main() {
    game2_core::infra::crash::install();
    let args: Vec<String> = std::env::args().collect();
    let days: u64 = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(300);
    let seed: u64 = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(42);
    let mut game = Game::new_seeded(seed);
    game.speed = 4;

    let budget = (days + 50) * game.ticks_per_frame() as u64 * 2;
    let mut last = game.tick;
    println!("day,alive,b0,b10,b30,b40,b70,b100,seeking,idle,other_act,food,gold_mean,gold_max,gold_min,org_members,org_power,esteem");
    for _ in 0..budget {
        if game.tick >= days { break; }
        game.update();
        if game.frame_n % driver::DRIVE_EVERY as u32 == 0 { driver::drive_player(&mut game, 0); }
        if game.tick == last { continue; }
        last = game.tick;
        if game.tick % 10 != 0 { continue; }
        let mut b = [0usize; 6]; // [0,1-10,11-30,31-40,41-70,71-100]
        let (mut seeking, mut idle, other) = (0, 0, 0);
        let mut golds = vec![];
        let mut alive = 0;
        for &v in &game.villagers {
            if villager::is_dead(&game.world, v) { continue; }
            alive += 1;
            let h = game.world.get_component::<Hunger>(v).map(|h| h.value).unwrap_or(100.0);
            b[match h { x if x <= 0.0 => 0, x if x <= 10.0 => 1, x if x <= 30.0 => 2,
                     x if x <= 40.0 => 3, x if x <= 70.0 => 4, _ => 5 }] += 1;
            match game.world.get_component::<Villager>(v).map(|v| v.activity) {
                Some(Activity::SeekingFood) | Some(Activity::Eating) => seeking += 1,
                _ => idle += 1,
            }
            let _ = other;
            if let Some(p) = game.world.get_component::<Purse>(v) { golds.push(p.gold); }
        }
        let food = game.world.get_component::<FoodSource>(game.market_food).map(|f| f.servings).unwrap_or(0);
        golds.sort_by(|a, c| a.partial_cmp(c).unwrap_or(std::cmp::Ordering::Equal));
        let gmean = if golds.is_empty() { 0.0 } else { golds.iter().sum::<f32>() / golds.len() as f32 };
        let t: Vec<(String, f64)> = game.secret_kpis().iter().map(|(k, v)| (k.to_string(), *v)).collect();
        let get = |k: &str| t.iter().find(|(n, _)| n == k).map(|(_, v)| *v).unwrap_or(0.0);
        println!("{},{},{},{},{},{},{},{},{},{},{},{},{:.1},{:.0},{:.0},{:.0},{:.1},{:.1}",
            game.tick, alive, b[0], b[1], b[2], b[3], b[4], b[5], seeking, idle, other, food,
            gmean, golds.last().copied().unwrap_or(0.0), golds.first().copied().unwrap_or(0.0),
            get("org.members"), get("org.power"), get("org.lord_esteem"));
    }

    // tail: the hungriest villagers' details at the end
    println!("\n-- hungriest at end (day {}) --", game.tick);
    let mut hs: Vec<(f32, usize, f32, f32, String)> = game.villagers.iter().enumerate()
        .filter(|(_, &v)| !villager::is_dead(&game.world, v))
        .map(|(i, &v)| {
            let h = game.world.get_component::<Hunger>(v).map(|h| h.value).unwrap_or(100.0);
            let p = game.world.get_component::<Pos>(v).copied().unwrap_or(Pos { x: 0.0, y: 0.0 });
            let g = game.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0);
            let act = game.world.get_component::<Villager>(v).map(|v| format!("{:?}", v.activity)).unwrap_or("?".into());
            (h, i, g, (p.x, p.y).0 + (p.x, p.y).1, act)
        }).collect();
    hs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mp = game.world.get_component::<Pos>(game.market_food).copied().unwrap_or(Pos { x: 0.0, y: 0.0 });
    for (h, i, g, _, act) in hs.iter().take(12) {
        let v = game.villagers[*i];
        let p = game.world.get_component::<Pos>(v).copied().unwrap_or(Pos { x: 0.0, y: 0.0 });
        let d = (p.x - mp.x).abs() + (p.y - mp.y).abs();
        println!("  {:<14} hunger={:<5.0} gold={:<6.1} dist_to_store={:<5.0} act={}", game.world.get_component::<Villager>(v).map(|_| format!("v{}", i)).unwrap_or_default(), h, g, d, act);
    }
    println!("store at ({:.0},{:.0}), servings {}", mp.x, mp.y,
        game.world.get_component::<FoodSource>(game.market_food).map(|f| f.servings).unwrap_or(0));
}
