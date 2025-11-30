//! Standalone CLI version of blockchain simulator for testing

mod lib_core;

use lib_core::{SimulationConfig, run_simulation_core};
use std::time::Instant;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Default configuration
    let mut config = SimulationConfig {
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
        simple_blocks: false,  // Use detailed mode for comparison
        large_sim_mode: false,
        exact_median: false,
    };
    
    let mut json_output = false;
    
    // Parse command line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--n" => {
                if i + 1 < args.len() {
                    config.n = args[i + 1].parse().unwrap_or(20000);
                    i += 1;
                }
            }
            "--run-type" => {
                if i + 1 < args.len() {
                    config.run_type = args[i + 1].parse().unwrap_or(6);
                    i += 1;
                }
            }
            "--large-sim-mode" => {
                if i + 1 < args.len() {
                    config.large_sim_mode = args[i + 1].parse().unwrap_or(0) != 0;
                    i += 1;
                }
            }
            "--exact-median" => {
                if i + 1 < args.len() {
                    config.exact_median = args[i + 1].parse().unwrap_or(0) != 0;
                    i += 1;
                }
            }
            "--json" => {
                json_output = true;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }
    
    // Run simulation
    let start = Instant::now();
    let results = run_simulation_core(config);
    let duration = start.elapsed();
    
    if json_output {
        // Output JSON for comparison script
        println!("{{");
        println!("  \"max_mb\": {},", results.max_mb);
        println!("  \"max_penalty\": {},", results.max_penalty);
        println!("  \"max_mempool\": {},", results.max_mempool);
        println!("  \"cumulative_fees\": {},", results.cumulative_fees);
        println!("  \"final_mb\": {},", results.M_B.last().unwrap_or(&0));
        println!("  \"final_ml\": {},", results.M_L.last().unwrap_or(&0));
        println!("  \"final_ms\": {},", results.M_S.last().unwrap_or(&0));
        println!("  \"final_mn\": {},", results.M_N.last().unwrap_or(&0));
        println!("  \"final_t_sim\": 800,");
        println!("  \"sample_data\": {{");
        
        // Helper function to format array
        fn format_array<T: std::fmt::Display>(name: &str, data: &[T]) {
            print!("    \"{}\": [", name);
            for (i, item) in data.iter().enumerate() {
                if i > 0 { print!(", "); }
                print!("{}", item);
            }
            println!("]");
        }
        
        format_array("M_B", &results.M_B);
        print!(",");
        format_array("M_L", &results.M_L);
        print!(",");
        format_array("M_S", &results.M_S);
        print!(",");
        format_array("M_N", &results.M_N);
        print!(",");
        format_array("input_volume", &results.input_volume);
        print!(",");
        format_array("block_fee", &results.block_fee);
        print!(",");
        format_array("penalty", &results.penalty);
        print!(",");
        print!("    \"t_sim\": [");
        for i in 0..results.indices.len() {
            if i > 0 { print!(", "); }
            print!("800");
        }
        println!("]");
        
        println!("  }}");
        println!("}}");
    } else {
        // Human-readable output
        println!("Blockchain Dynamic Block Size Simulator - CLI Test");
        println!("==================================================\n");
        
        println!("Configuration:");
        println!("  Blocks: {}", config.n);
        println!("  Run Type: {}", config.run_type);
        println!("  Simple Blocks: {}", config.simple_blocks);
        println!("  Add Noise: {}", config.add_noise);
        println!("  Users Pay More: {}", config.users_pay_more);
        println!("  Large Sim Mode: {}", config.large_sim_mode);
        println!("  Exact Median: {}", config.exact_median);
        println!();
        
        println!("Running simulation...");
        println!("\n✅ Simulation Complete!");
        println!("Time: {:.2}ms", duration.as_secs_f64() * 1000.0);
        println!("\nResults:");
        println!("  Max Block Size: {} bytes", results.max_mb);
        println!("  Max Mempool: {} bytes", results.max_mempool);
        println!("  Max Penalty: {:.6}", results.max_penalty);
        println!("  Cumulative Fees: {:.6} XMR", results.cumulative_fees);
        println!("  Data Points: {}", results.data_points);
    }
}