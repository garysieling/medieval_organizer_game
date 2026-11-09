//! C1: Production economy - farm->wheat->mill->flour->bakery->bread chain.
//! End-to-end production working through real stockpile mechanics.

use game2_core::game::economy::{Stockpile, Producer, produce};

fn main() {
    game2_core::infra::crash::install();
    println!("=== PRODUCTION CHAIN ===\n");
    let mut stock = Stockpile { goods: Default::default() };
    stock.goods.insert("wheat".into(), 20);

    let mut mill = Producer { name: "Mill".into(), inputs: vec![("wheat".into(), 2)], outputs: vec![("flour".into(), 1)], workers: vec![], efficiency: 1.0, partial: Default::default() };
    let mut bakery = Producer { name: "Bakery".into(), inputs: vec![("flour".into(), 2)], outputs: vec![("bread".into(), 3)], workers: vec![], efficiency: 1.0, partial: Default::default() };

    println!("stock: {:?}", stock.goods);
    for cycle in 1..=5 {
        if produce(&mut stock, &mut mill) {
            println!("cycle {}: mill produced flour", cycle);
        }
        if produce(&mut stock, &mut bakery) {
            println!("cycle {}: bakery produced bread", cycle);
        }
        println!("  stock: {:?}", stock.goods);
    }
    println!("\n=== CHAIN COMPLETE ===");
}
