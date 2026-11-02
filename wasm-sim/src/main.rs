//! Standalone CLI version of blockchain simulator for testing
//!
//!   blockchain-sim [--rules v16|v17|custom] [--n BLOCKS] [--run-type N] [--t-sim BYTES]
//!                  [--ramp X] [--noise] [--users-pay-more] [--fill-blocks] [--fee-level N]
//!                  [--sanity-cap]

mod lib_core;

use lib_core::{run_simulation_core, SimulationConfig, RULES_CUSTOM, RULES_V16, RULES_V17};
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    let value = |i: usize| -> &str { args.get(i + 1).map(String::as_str).unwrap_or_else(|| panic!("{} needs a value", args[i])) };

    let rules = match args.iter().position(|a| a == "--rules").map(|i| value(i)) {
        Some("v16") => RULES_V16,
        Some("custom") => RULES_CUSTOM,
        _ => RULES_V17,
    };
    let mut c = SimulationConfig::preset(rules);

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--rules" => i += 1,
            "--n" => { c.n = value(i).parse().expect("--n"); i += 1; }
            "--run-type" => { c.run_type = value(i).parse().expect("--run-type"); i += 1; }
            "--t-sim" => { c.t_sim = value(i).parse().expect("--t-sim"); i += 1; }
            "--ramp" => { c.ramp_multiplier = value(i).parse().expect("--ramp"); i += 1; }
            "--fee-level" => { c.fee_level = value(i).parse().expect("--fee-level"); i += 1; }
            "--noise" => c.add_noise = true,
            "--users-pay-more" => c.users_pay_more = true,
            "--fill-blocks" => c.rational_miners = false,
            "--sanity-cap" => c.sanity_cap = true,
            other => panic!("unknown argument {other}"),
        }
        i += 1;
    }

    println!("Rules: {}  blocks: {}  run type: {}  tx weight: {}", ["v16", "v17", "custom"][c.rules as usize], c.n, c.run_type, c.t_sim);
    println!("Z_M {}  long-term clamp {}/{}  surge {}x  T_R {}  grace {}", c.z_m, c.lt_growth_num, c.lt_growth_den, c.surge_factor, c.t_r, c.wallet_grace_blocks);

    let start = Instant::now();
    let r = run_simulation_core(c);
    let elapsed = start.elapsed();

    let last = r.M_B.len() - 1;
    println!("Done in {:.2?}", elapsed);
    println!("Max block weight:   {}", r.max_mb);
    println!("Final M_L / M_N:    {} / {}", r.M_L[last], r.M_N[last]);
    println!("Max penalty:        {:.6} XMR", r.max_penalty);
    println!("Max mempool:        {} bytes", r.max_mempool);
    println!("Cumulative fees:    {:.6} XMR", r.cumulative_fees);
    println!("Rejected txs:       {}", r.rejected_txs);
    println!("Relay min fee/byte: {:.3e} XMR", r.min_fee_per_byte);
    println!("Wallet fee levels:  {:?}", r.fee_levels.iter().map(|f| format!("{f:.2e}")).collect::<Vec<_>>());
}
