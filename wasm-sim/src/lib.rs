//! Blockchain Dynamic Block Size Simulator
//! Compiled to WebAssembly for high-performance browser execution

pub mod lib_core;

use wasm_bindgen::prelude::*;
use lib_core::{run_simulation_core, SimulationConfig as CoreConfig, SimulationResults as CoreResults};

// Global buffers for results (WASM memory)
static mut RESULT_M_B: Vec<i64> = Vec::new();
static mut RESULT_M_L: Vec<i64> = Vec::new();
static mut RESULT_M_S: Vec<i64> = Vec::new();
static mut RESULT_M_N: Vec<i64> = Vec::new();
static mut RESULT_INPUT_VOL: Vec<i64> = Vec::new();
static mut RESULT_BLOCK_FEE: Vec<f64> = Vec::new();
static mut RESULT_PENALTY: Vec<f64> = Vec::new();
static mut RESULT_MEMPOOL: Vec<i64> = Vec::new();
static mut RESULT_INDICES: Vec<u32> = Vec::new();

// Stats
static mut STATS_MAX_MB: i64 = 0;
static mut STATS_MAX_PENALTY: f64 = 0.0;
static mut STATS_MAX_MEMPOOL: i64 = 0;
static mut STATS_CUMULATIVE_FEES: f64 = 0.0;

/// Simulation configuration structure (WASM wrapper)
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct SimulationConfig {
    pub n: u32,
    pub steady_state: i64,
    pub z_m: i64,
    pub t_r: i64,
    pub r_base: f64,
    pub mid_100k: u32,
    pub mid_100: u32,
    pub t_sim: i64,
    pub run_type: u32,
    pub ramp_multiplier: f64,
    pub ml_mult: f64,
    pub mn_mult: f64,
    pub add_noise: bool,
    pub users_pay_more: bool,
    pub simple_blocks: bool,
    pub large_sim_mode: bool,
    pub exact_median: bool,
    pub max_blocksize: i64,
    pub max_blocksize_growth_rate: f64,
    pub use_long_term_median_cap: bool,
    pub sanity_start_weight: i64,
    pub sanity_start_block: u32,
}

#[wasm_bindgen]
impl SimulationConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(
        n: u32,
        steady_state: i64,
        z_m: i64,
        t_r: i64,
        r_base: f64,
        mid_100k: u32,
        mid_100: u32,
        t_sim: i64,
        run_type: u32,
        ramp_multiplier: f64,
        ml_mult: f64,
        mn_mult: f64,
        add_noise: bool,
        users_pay_more: bool,
        simple_blocks: bool,
        large_sim_mode: bool,
        exact_median: bool,
        max_blocksize: i64,
        max_blocksize_growth_rate: f64,
        use_long_term_median_cap: bool,
        sanity_start_weight: i64,
        sanity_start_block: u32,
    ) -> SimulationConfig {
        SimulationConfig {
            n,
            steady_state,
            z_m,
            t_r,
            r_base,
            mid_100k,
            mid_100,
            t_sim,
            run_type,
            ramp_multiplier,
            ml_mult,
            mn_mult,
            add_noise,
            users_pay_more,
            simple_blocks,
            large_sim_mode,
            exact_median,
            max_blocksize,
            max_blocksize_growth_rate,
            use_long_term_median_cap,
            sanity_start_weight,
            sanity_start_block,
        }
    }
}

impl From<SimulationConfig> for CoreConfig {
    fn from(config: SimulationConfig) -> Self {
        CoreConfig {
            n: config.n,
            steady_state: config.steady_state,
            z_m: config.z_m,
            t_r: config.t_r,
            r_base: config.r_base,
            mid_100k: config.mid_100k,
            mid_100: config.mid_100,
            t_sim: config.t_sim,
            run_type: config.run_type,
            ramp_multiplier: config.ramp_multiplier,
            ml_mult: config.ml_mult,
            mn_mult: config.mn_mult,
            add_noise: config.add_noise,
            users_pay_more: config.users_pay_more,
            simple_blocks: config.simple_blocks,
            large_sim_mode: config.large_sim_mode,
            exact_median: config.exact_median,
            max_blocksize: config.max_blocksize,
            max_blocksize_growth_rate: config.max_blocksize_growth_rate,
            use_long_term_median_cap: config.use_long_term_median_cap,
            sanity_start_weight: config.sanity_start_weight,
            sanity_start_block: config.sanity_start_block,
        }
    }
}

/// Simulation results structure (WASM wrapper)
#[wasm_bindgen]
pub struct SimulationResults {
    pub max_mb: i64,
    pub max_penalty: f64,
    pub max_mempool: i64,
    pub cumulative_fees: f64,
    pub data_points: u32,
}

/// Get pointer to result arrays for JS to read
#[wasm_bindgen]
pub fn get_m_b_ptr() -> *const i64 {
    unsafe { RESULT_M_B.as_ptr() }
}

#[wasm_bindgen]
pub fn get_m_l_ptr() -> *const i64 {
    unsafe { RESULT_M_L.as_ptr() }
}

#[wasm_bindgen]
pub fn get_m_s_ptr() -> *const i64 {
    unsafe { RESULT_M_S.as_ptr() }
}

#[wasm_bindgen]
pub fn get_m_n_ptr() -> *const i64 {
    unsafe { RESULT_M_N.as_ptr() }
}

#[wasm_bindgen]
pub fn get_input_vol_ptr() -> *const i64 {
    unsafe { RESULT_INPUT_VOL.as_ptr() }
}

#[wasm_bindgen]
pub fn get_block_fee_ptr() -> *const f64 {
    unsafe { RESULT_BLOCK_FEE.as_ptr() }
}

#[wasm_bindgen]
pub fn get_penalty_ptr() -> *const f64 {
    unsafe { RESULT_PENALTY.as_ptr() }
}

#[wasm_bindgen]
pub fn get_mempool_ptr() -> *const i64 {
    unsafe { RESULT_MEMPOOL.as_ptr() }
}

#[wasm_bindgen]
pub fn get_indices_ptr() -> *const u32 {
    unsafe { RESULT_INDICES.as_ptr() }
}

#[wasm_bindgen]
pub fn get_result_len() -> u32 {
    unsafe { RESULT_M_B.len() as u32 }
}

#[wasm_bindgen]
pub fn get_max_mb() -> i64 {
    unsafe { STATS_MAX_MB }
}

#[wasm_bindgen]
pub fn get_max_penalty() -> f64 {
    unsafe { STATS_MAX_PENALTY }
}

#[wasm_bindgen]
pub fn get_max_mempool() -> i64 {
    unsafe { STATS_MAX_MEMPOOL }
}

#[wasm_bindgen]
pub fn get_cumulative_fees() -> f64 {
    unsafe { STATS_CUMULATIVE_FEES }
}

/// Main simulation function - NOTE: This returns immediately but data must be read from memory
#[wasm_bindgen]
pub fn run_simulation(config: SimulationConfig) -> SimulationResults {
    // Run the optimized core simulation
    let core_config: CoreConfig = config.into();
    let core_results = run_simulation_core(core_config);
    
    // Store results in global statics
    unsafe {
        STATS_MAX_MB = core_results.max_mb;
        STATS_MAX_PENALTY = core_results.max_penalty;
        STATS_MAX_MEMPOOL = core_results.max_mempool;
        STATS_CUMULATIVE_FEES = core_results.cumulative_fees;
        
        // Store detailed data for charting
        RESULT_M_B = core_results.M_B;
        RESULT_M_L = core_results.M_L;
        RESULT_M_S = core_results.M_S;
        RESULT_M_N = core_results.M_N;
        RESULT_INPUT_VOL = core_results.input_volume;
        RESULT_BLOCK_FEE = core_results.block_fee;
        RESULT_PENALTY = core_results.penalty;
        RESULT_MEMPOOL = core_results.mempool_size;
        RESULT_INDICES = core_results.indices;
    }
    
    // NOTE: For now, we're not storing per-block data to avoid the performance overhead
    // The web UI will need to be updated to only show summary stats
    // Or we need to run the simulation again to collect detailed data
    
    SimulationResults {
        max_mb: core_results.max_mb,
        max_penalty: core_results.max_penalty,
        max_mempool: core_results.max_mempool,
        cumulative_fees: core_results.cumulative_fees,
        data_points: core_results.data_points,
    }
}

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
