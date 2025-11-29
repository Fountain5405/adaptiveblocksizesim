//! Standalone CLI version of the blockchain simulator for testing

mod lib_core;

use lib_core::{SimulationConfig, run_simulation_core};
use std::time::Instant;

fn main() {
    println!("Blockchain Dynamic Block Size Simulator - CLI Test");
    println!("==================================================\n");
    
    // Default configuration matching web interface defaults
    let config = SimulationConfig {
        n: 20000,
        steady_state: 300000,
        z_m: 300000,
        t_r: 3000,
        r_base: 0.6,
        mid_100k: 50000,
        mid_100: 50,
        t_sim: 800,
        run_type: 6,
        ramp_multiplier: 3.0,
        ml_mult: 2.0,
        mn_mult: 50.0,
        add_noise: false,
        users_pay_more: false,
        simple_blocks: true,
    };
    
    println!("Configuration:");
    println!("  Blocks: {}", config.n);
    println!("  Run Type: {}", config.run_type);
    println!("  Simple Blocks: {}", config.simple_blocks);
    println!("  Add Noise: {}", config.add_noise);
    println!("  Users Pay More: {}", config.users_pay_more);
    println!();
    
    println!("Running simulation...");
    let start = Instant::now();
    let results = run_simulation_core(config);
    let duration = start.elapsed();
    
    println!("\n✅ Simulation Complete!");
    println!("Time: {:.2}ms", duration.as_secs_f64() * 1000.0);
    println!("\nResults:");
    println!("  Max Block Size: {} bytes", results.max_mb);
    println!("  Max Mempool: {} bytes", results.max_mempool);
    println!("  Max Penalty: {:.6}", results.max_penalty);
    println!("  Cumulative Fees: {:.6} XMR", results.cumulative_fees);
    println!("  Data Points: {}", results.data_points);
}