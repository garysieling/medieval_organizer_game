//! Stratification: economic hierarchy emerges from work + surplus extraction.
//! From lit review: village hierarchy is economic, peasants vs lords over surplus.


fn main() {
    println!("=== VILLAGE STRATIFICATION (economic) ===\n");

    // Villagers with different productive capacity
    let workers = vec![
        ("Aldric", 0.8), ("Eliza", 0.6), ("Bram", 0.9), ("Corvin", 0.5), ("Dagny", 0.7),
    ];

    let mut wealth: Vec<f32> = vec![0.0; 5];
    let lord_share = 0.3; // lord extracts 30% of surplus

    println!("year,aldric,eliza,bram,corvin,dagny,lord_take\n");
    for year in 0..20 {
        for (i, (_, prod)) in workers.iter().enumerate() {
            let production = prod * 100.0;
            let surplus: f32 = if production > 40.0 { production - 40.0 } else { 0.0 };
            let lord_take = surplus * lord_share;
            wealth[i] += surplus - lord_take; // peasants keep remainder
        }
        if year % 5 == 0 {
            println!("{},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0}",
                year, wealth[0], wealth[1], wealth[2], wealth[3], wealth[4],
                (0..5).map(|i| workers[i].1 * 100.0 * 0.3 * 0.6).sum::<f32>());
        }
    }

    // Hierarchy emerges: Bram (productive) rich, Corvin (low) poor
    println!("\n=== Final Hierarchy ===");
    let mut ranked: Vec<_> = workers.iter().enumerate().map(|(i, (n, _))| (*n, wealth[i])).collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (rank, (name, w)) in ranked.iter().enumerate() {
        let status = if *w > 500.0 { "rich peasant" } else if *w > 200.0 { "middling" } else { "poor" };
        println!("{}. {} ({:.0} wealth) - {}", rank + 1, name, w, status);
    }
    println!("\n=== STRATIFICATION COMPLETE ===");
}
