//! C7: Contracts + reputation - sign, breach, consequences.
//! Legal relationships enforce cooperation; breach damages reputation/trust.

use game2_core::engine::ecs::World;
use game2_core::game::contracts::{Contract, Term, ContractStatus, Reputation, breach};

fn main() {
    println!("=== CONTRACTS + REPUTATION ===\n");
    let mut world = World::new();

    // Two parties: miller (obligated) and baker
    let miller = world.create_entity();
    let baker = world.create_entity();
    world.add_component(miller, Reputation { entity: miller, trust_score: 0.8, contracts_honored: 3, contracts_breached: 0 });
    world.add_component(baker, Reputation { entity: baker, trust_score: 0.7, contracts_honored: 5, contracts_breached: 0 });

    // They sign a supply contract
    let contract = world.create_entity();
    world.add_component(contract, Contract {
        parties: vec![miller, baker],
        terms: vec![
            Term { description: "deliver 20 flour by t50".into(), obligated: miller, fulfilled: false },
        ],
        status: ContractStatus::Active,
        breach_penalty: 0.3,
    });
    println!("Contract signed: miller -> baker, 20 flour by t50");

    // At t30, miller breaches (can't deliver)
    println!("\nt30: miller fails to deliver -> breach");
    breach(&mut world, contract);

    let status = world.get_component::<Contract>(contract).unwrap().status;
    println!("contract status: {:?}", status);

    // Miller's reputation takes the hit (breach applies the penalty)
    let m_rep = world.get_component::<Reputation>(miller).unwrap().trust_score;
    let b_rep = world.get_component::<Reputation>(baker).unwrap().trust_score;
    println!("\nmiller reputation: {:.2} (breached {} contracts)", m_rep,
        world.get_component::<Reputation>(miller).unwrap().contracts_breached);
    println!("baker reputation: {:.2} (unaffected)", b_rep);

    // Consequence: future trust damaged
    println!("\nMiller's low reputation means baker won't trust new contracts");
    println!("=== CONTRACTS COMPLETE ===");
}
