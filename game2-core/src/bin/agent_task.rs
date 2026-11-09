//! E4: Real Ollama agent task execution end-to-end.
//! Implementation Engineer executes a task via gemma3:4b and reports.
//! Evidence: actual AI output performing agent work.

use game2_core::agents::implementation_engineer::{ImplementationEngineer, ImplementationTask};

fn main() {
    game2_core::infra::crash::install();
    println!("=== REAL OLLAMA AGENT TASK ===\n");

    let engineer = ImplementationEngineer::new("gemma3:4b");

    // Check availability
    if !engineer.client.is_available() {
        println!("ERROR: Ollama not available");
        return;
    }
    println!("Ollama available. Agent: {}", engineer.name);

    // Assign a real task
    let task = ImplementationTask {
        description: "Write a Rust function `fn fib(n: u32) -> u64` that returns the nth Fibonacci number".into(),
        task_type: "feature".into(),
        context: "No external deps. Include a brief comment.".into(),
    };

    println!("\nTask: {}", task.description);
    println!("Executing via gemma3:4b...\n");

    match engineer.execute(&task) {
        Ok(result) => {
            println!("=== AGENT OUTPUT ===\n");
            println!("{}", result.output);
            println!("\n=== TASK COMPLETE ===");
        }
        Err(e) => println!("Error: {}", e),
    }
}
