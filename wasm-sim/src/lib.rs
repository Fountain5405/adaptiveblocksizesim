//! Blockchain Dynamic Block Size Simulator
//! Compiled to WebAssembly for high-performance browser execution

pub mod lib_core;

use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use lib_core::{run_simulation_core, SimulationConfig as CoreConfig, SimulationResults as CoreResults};

thread_local! {
    static LAST: RefCell<Option<CoreResults>> = RefCell::new(None);
}

/// Simulation configuration (WASM wrapper).
/// Create one from a rule preset, then set any fields to change from the preset:
///   const c = new SimulationConfig(1); c.n = 300000; c.t_sim = 9093;
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct SimulationConfig {
    pub n: u32,
    pub rules: u32,
    pub z_m: i64,
    pub lt_growth_num: i64,
    pub lt_growth_den: i64,
    pub surge_factor: i64,
    pub long_window: u32,
    pub short_window: u32,
    pub initial_weight: i64,
    pub sanity_cap: bool,
    pub sanity_start_weight: i64,
    pub sanity_start_block: u32,
    pub t_r: i64,
    pub min_fee_fraction: f64,
    pub wallet_grace_blocks: u32,
    pub fee_rounding_digits: u32,
    pub fee_level: i32,
    pub r_base: f64,
    pub t_sim: i64,
    pub run_type: u32,
    pub ramp_multiplier: f64,
    pub add_noise: bool,
    pub users_pay_more: bool,
    pub rational_miners: bool,
}

#[wasm_bindgen]
impl SimulationConfig {
    /// rules: 0 = v16 (mainnet, 2021 scaling), 1 = v17 (FCMP++, 2026 scaling), 2 = custom
    #[wasm_bindgen(constructor)]
    pub fn new(rules: u32) -> SimulationConfig {
        CoreConfig::preset(rules).into()
    }
}

impl From<CoreConfig> for SimulationConfig {
    fn from(c: CoreConfig) -> Self {
        SimulationConfig {
            n: c.n,
            rules: c.rules,
            z_m: c.z_m,
            lt_growth_num: c.lt_growth_num,
            lt_growth_den: c.lt_growth_den,
            surge_factor: c.surge_factor,
            long_window: c.long_window,
            short_window: c.short_window,
            initial_weight: c.initial_weight,
            sanity_cap: c.sanity_cap,
            sanity_start_weight: c.sanity_start_weight,
            sanity_start_block: c.sanity_start_block,
            t_r: c.t_r,
            min_fee_fraction: c.min_fee_fraction,
            wallet_grace_blocks: c.wallet_grace_blocks,
            fee_rounding_digits: c.fee_rounding_digits,
            fee_level: c.fee_level,
            r_base: c.r_base,
            t_sim: c.t_sim,
            run_type: c.run_type,
            ramp_multiplier: c.ramp_multiplier,
            add_noise: c.add_noise,
            users_pay_more: c.users_pay_more,
            rational_miners: c.rational_miners,
        }
    }
}

impl From<SimulationConfig> for CoreConfig {
    fn from(c: SimulationConfig) -> Self {
        CoreConfig {
            n: c.n,
            rules: c.rules,
            z_m: c.z_m,
            lt_growth_num: c.lt_growth_num,
            lt_growth_den: c.lt_growth_den,
            surge_factor: c.surge_factor,
            long_window: c.long_window,
            short_window: c.short_window,
            initial_weight: c.initial_weight,
            sanity_cap: c.sanity_cap,
            sanity_start_weight: c.sanity_start_weight,
            sanity_start_block: c.sanity_start_block,
            t_r: c.t_r,
            min_fee_fraction: c.min_fee_fraction,
            wallet_grace_blocks: c.wallet_grace_blocks,
            fee_rounding_digits: c.fee_rounding_digits,
            fee_level: c.fee_level,
            r_base: c.r_base,
            t_sim: c.t_sim,
            run_type: c.run_type,
            ramp_multiplier: c.ramp_multiplier,
            add_noise: c.add_noise,
            users_pay_more: c.users_pay_more,
            rational_miners: c.rational_miners,
        }
    }
}

/// Summary of the last run (WASM wrapper). Per-block series are read with the get_* functions.
#[wasm_bindgen]
pub struct SimulationResults {
    pub max_mb: i64,
    pub max_penalty: f64,
    pub max_mempool: i64,
    pub cumulative_fees: f64,
    pub rejected_txs: i64,
    pub min_fee_per_byte: f64,
    pub data_points: u32,
}

#[wasm_bindgen]
pub fn run_simulation(config: SimulationConfig) -> SimulationResults {
    let r = run_simulation_core(config.into());
    let summary = SimulationResults {
        max_mb: r.max_mb,
        max_penalty: r.max_penalty,
        max_mempool: r.max_mempool,
        cumulative_fees: r.cumulative_fees,
        rejected_txs: r.rejected_txs,
        min_fee_per_byte: r.min_fee_per_byte,
        data_points: r.data_points,
    };
    LAST.with(|l| *l.borrow_mut() = Some(r));
    summary
}

fn with_last<T: Default>(f: impl FnOnce(&CoreResults) -> T) -> T {
    LAST.with(|l| l.borrow().as_ref().map(f).unwrap_or_default())
}

// Per-block series of the last run, copied out as typed arrays.
#[wasm_bindgen] pub fn get_m_b() -> Vec<i64> { with_last(|r| r.M_B.clone()) }
#[wasm_bindgen] pub fn get_m_l() -> Vec<i64> { with_last(|r| r.M_L.clone()) }
#[wasm_bindgen] pub fn get_m_s() -> Vec<i64> { with_last(|r| r.M_S.clone()) }
#[wasm_bindgen] pub fn get_m_n() -> Vec<i64> { with_last(|r| r.M_N.clone()) }
#[wasm_bindgen] pub fn get_input_volume() -> Vec<i64> { with_last(|r| r.input_volume.clone()) }
#[wasm_bindgen] pub fn get_block_fee() -> Vec<f64> { with_last(|r| r.block_fee.clone()) }
#[wasm_bindgen] pub fn get_penalty() -> Vec<f64> { with_last(|r| r.penalty.clone()) }
#[wasm_bindgen] pub fn get_mempool() -> Vec<i64> { with_last(|r| r.mempool_size.clone()) }
#[wasm_bindgen] pub fn get_fee_low() -> Vec<f64> { with_last(|r| r.fee_low.clone()) }
#[wasm_bindgen] pub fn get_fee_levels() -> Vec<f64> { with_last(|r| r.fee_levels.clone()) }

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

/// Utility function to test WASM is working
#[wasm_bindgen]
pub fn test_wasm() -> String {
    "WASM is working!".to_string()
}
