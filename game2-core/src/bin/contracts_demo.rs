//! C7: Contracts + reputation - sign, breach, consequences.
//! Legal relationships enforce cooperation; breach damages reputation/trust.

use game2_core::engine::ecs::{World, Scheduler};
use game2_core::game::contracts::{Contract, Term, ContractStatus, ContractSystem, Reputation, breach};

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
        aid_pact: false,
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

    // The engine itself: a second contract that DOES get fulfilled -
    // ContractSystem sees every term met and credits the obligated party
    let mut sched = Scheduler::new();
    sched.add_system(ContractSystem);
    let honest = world.create_entity();
    world.add_component(honest, Reputation { entity: honest, trust_score: 0.6, contracts_honored: 0, contracts_breached: 0 });
    let c2 = world.create_entity();
    world.add_component(c2, Contract {
        parties: vec![honest, baker],
        terms: vec![Term { description: "deliver 10 barley by t60".into(), obligated: honest, fulfilled: false }],
        status: ContractStatus::Active,
        breach_penalty: 0.2,
        aid_pact: false,
    });
    println!("\nSecond contract signed: honest -> baker, 10 barley");
    // the delivery lands on time
    for t in world.get_component_mut::<Contract>(c2).unwrap().terms.iter_mut() { t.fulfilled = true; }
    sched.tick(&mut world);
    let r = world.get_component::<Reputation>(honest).unwrap();
    println!("engine ticks: contract fulfilled -> trust {:.2} ({} honored)", r.trust_score, r.contracts_honored);

    println!("=== CONTRACTS COMPLETE ===");
}
