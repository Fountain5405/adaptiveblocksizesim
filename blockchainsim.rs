//! Blockchain Dynamic Block Size Simulator
//! Compiled to WebAssembly for high-performance browser execution

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

/// Get pointer to result arrays for JS to read
#[no_mangle]
pub extern "C" fn get_m_b_ptr() -> *const i64 {
    unsafe { RESULT_M_B.as_ptr() }
}

#[no_mangle]
pub extern "C" fn get_m_l_ptr() -> *const i64 {
    unsafe { RESULT_M_L.as_ptr() }
}

#[no_mangle]
pub extern "C" fn get_m_s_ptr() -> *const i64 {
    unsafe { RESULT_M_S.as_ptr() }
}

#[no_mangle]
pub extern "C" fn get_m_n_ptr() -> *const i64 {
    unsafe { RESULT_M_N.as_ptr() }
}

#[no_mangle]
pub extern "C" fn get_input_vol_ptr() -> *const i64 {
    unsafe { RESULT_INPUT_VOL.as_ptr() }
}

#[no_mangle]
pub extern "C" fn get_block_fee_ptr() -> *const f64 {
    unsafe { RESULT_BLOCK_FEE.as_ptr() }
}

#[no_mangle]
pub extern "C" fn get_penalty_ptr() -> *const f64 {
    unsafe { RESULT_PENALTY.as_ptr() }
}

#[no_mangle]
pub extern "C" fn get_mempool_ptr() -> *const i64 {
    unsafe { RESULT_MEMPOOL.as_ptr() }
}

#[no_mangle]
pub extern "C" fn get_indices_ptr() -> *const u32 {
    unsafe { RESULT_INDICES.as_ptr() }
}

#[no_mangle]
pub extern "C" fn get_result_len() -> u32 {
    unsafe { RESULT_M_B.len() as u32 }
}

#[no_mangle]
pub extern "C" fn get_max_mb() -> i64 {
    unsafe { STATS_MAX_MB }
}

#[no_mangle]
pub extern "C" fn get_max_penalty() -> f64 {
    unsafe { STATS_MAX_PENALTY }
}

#[no_mangle]
pub extern "C" fn get_max_mempool() -> i64 {
    unsafe { STATS_MAX_MEMPOOL }
}

#[no_mangle]
pub extern "C" fn get_cumulative_fees() -> f64 {
    unsafe { STATS_CUMULATIVE_FEES }
}

/// Simple pseudo-random number generator (xorshift)
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng { state: seed }
    }
    
    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
    
    fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }
    
    // Box-Muller transform for normal distribution
    fn normal(&mut self, mean: f64, std_dev: f64) -> f64 {
        let u1 = self.next_f64().max(1e-10);
        let u2 = self.next_f64();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        mean + std_dev * z
    }
}

/// Main simulation function
#[no_mangle]
pub extern "C" fn run_simulation(
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
    add_noise: u32,
    users_pay_more: u32,
    simple_blocks: u32,
) {
    let n = n as usize;
    let mid_100k = mid_100k as usize;
    let mid_100 = mid_100 as usize;
    let len_l = mid_100k * 2;
    let len_s = mid_100 * 2;
    let add_noise = add_noise != 0;
    let users_pay_more = users_pay_more != 0;
    let simple_blocks = simple_blocks != 0;
    
    // Initialize
    let mut m_b: i64 = 0;
    let mut m_l_prev = steady_state;
    let mut rng = Rng::new(12345);
    
    // Circular buffers for median calculation
    let mut m_l_buffer: Vec<i64> = vec![steady_state; len_l];
    let mut m_l_head: usize = 0;
    let mut m_s_buffer: Vec<i64> = vec![steady_state; len_s];
    let mut m_s_head: usize = 0;
    
    // For median calculation, we'll sort periodically
    let mut m_l_sorted: Vec<i64> = vec![steady_state; len_l];
    let mut m_s_sorted: Vec<i64> = vec![steady_state; len_s];
    let sort_interval_l = (len_l / 100).max(100);
    let sort_interval_s = (len_s / 10).max(10);
    let mut updates_since_sort_l: usize = 0;
    let mut updates_since_sort_s: usize = 0;
    
    // Mempool
    let mut mempool: [i64; 2] = [0, 0];
    let mut percent_response: f64 = 0.0;
    
    // Results tracking
    let mut max_mb: i64 = 0;
    let mut max_penalty: f64 = 0.0;
    let mut max_mempool: i64 = 0;
    let mut cumulative_fees: f64 = 0.0;
    
    // Calculate sample rate
    let sample_rate = (n / 1000).max(1);
    
    // Clear result buffers
    unsafe {
        RESULT_M_B.clear();
        RESULT_M_L.clear();
        RESULT_M_S.clear();
        RESULT_M_N.clear();
        RESULT_INPUT_VOL.clear();
        RESULT_BLOCK_FEE.clear();
        RESULT_PENALTY.clear();
        RESULT_MEMPOOL.clear();
        RESULT_INDICES.clear();
        
        // Reserve capacity
        let cap = n / sample_rate + 1;
        RESULT_M_B.reserve(cap);
        RESULT_M_L.reserve(cap);
        RESULT_M_S.reserve(cap);
        RESULT_M_N.reserve(cap);
        RESULT_INPUT_VOL.reserve(cap);
        RESULT_BLOCK_FEE.reserve(cap);
        RESULT_PENALTY.reserve(cap);
        RESULT_MEMPOOL.reserve(cap);
        RESULT_INDICES.reserve(cap);
    }
    
    for i in 0..n {
        // ============================================
        // 1. MEDIAN CALCULATIONS
        // ============================================
        
        // Periodic sort for M_L
        if updates_since_sort_l >= sort_interval_l || i == 0 {
            m_l_sorted.copy_from_slice(&m_l_buffer);
            m_l_sorted.sort_unstable();
            updates_since_sort_l = 0;
        }
        
        // Periodic sort for M_S
        if updates_since_sort_s >= sort_interval_s || i == 0 {
            m_s_sorted.copy_from_slice(&m_s_buffer);
            m_s_sorted.sort_unstable();
            updates_since_sort_s = 0;
        }
        
        // Calculate medians
        let m_l = (m_l_sorted[mid_100k] + m_l_sorted[len_l - 1 - mid_100k]) / 2;
        let m_s = (m_s_sorted[mid_100] + m_s_sorted[len_s - 1 - mid_100]) / 2;
        
        // M_L_weight calculation
        let ml_upper = (ml_mult * m_l_prev as f64) as i64;
        let ml_lower = (m_l_prev as f64 / ml_mult) as i64;
        let m_l_weight = m_b.min(ml_upper).max(z_m).max(ml_lower);
        
        // M_S_weight calculation
        let m_s_weight = m_b.max(m_l);
        
        // M_N calculation (capped)
        let mn_cap = (mn_mult * m_l as f64) as i64;
        let m_n = m_s.min(mn_cap);
        
        // M_B_max
        let m_b_max = 2 * m_n;
        
        // Fee calculations
        let f_r = r_base * (t_r as f64) / ((m_l as f64) * (m_l as f64));
        
        // ============================================
        // 2. BROADCAST TRANSACTIONS
        // ============================================
        let mut broadcast: [i64; 2] = [0, 0];
        
        let vol: i64 = match run_type {
            1 => z_m + 100 * (i as i64),
            2 => z_m + 800 * (i as i64),
            3 => ((316.0 + (i as f64 / 15.0)).powi(2)) as i64,
            4 => (z_m as f64 * (1.6_f64.powf(9.8 + (i as f64 / 50000.0)) - 99.75)) as i64,
            5 => m_b_max,
            6 | _ => {
                let start_val: i64 = 300000;
                let ramp_delay: usize = 10;
                let ramp_days: usize = 14;
                let ramp_time = ramp_days * 720;
                
                if i <= ramp_delay {
                    start_val
                } else if i <= ramp_delay + ramp_time {
                    start_val + (((ramp_multiplier - 1.0) * start_val as f64 / ramp_time as f64) as i64) * ((i - ramp_delay) as i64)
                } else {
                    (ramp_multiplier * start_val as f64 + 220.0 * (i as f64 / 802.0).sin() * 800.0) as i64
                }
            }
        };
        
        broadcast[1] = vol / t_sim;
        let input_volume = broadcast[1] * t_sim;
        
        // Add noise if enabled
        if add_noise && broadcast[1] > 0 {
            let noise = 0.2 * rng.normal(0.0, broadcast[1] as f64);
            broadcast[1] = (broadcast[1] + noise as i64).max(1);
        }
        
        // Users pay more if enabled
        if users_pay_more && broadcast[1] > 0 {
            let prev_resp = percent_response;
            let calc = (mempool[1] as f64 / (3.0 * broadcast[1] as f64) * 100.0).floor();
            percent_response = (prev_resp + 0.1 * (calc - prev_resp)).floor();
            percent_response = percent_response.clamp(0.0, 100.0);
            
            if percent_response > 0.0 {
                broadcast[0] = ((broadcast[1] as f64 * percent_response) / 100.0) as i64;
                broadcast[1] = ((broadcast[1] as f64 * (100.0 - percent_response)) / 100.0) as i64;
            }
        }
        
        // Update mempool
        mempool[0] += broadcast[0];
        mempool[1] += broadcast[1];
        
        // Fee levels
        let fees: [f64; 2] = [16.0 * f_r * t_sim as f64, f_r * t_sim as f64];
        
        // ============================================
        // 3. BUILD BLOCK
        // ============================================
        let mut block_fee_total: f64 = 0.0;
        
        if simple_blocks {
            // Simple mode: just fill block from mempool
            let mempool_total_bytes = (mempool[0] + mempool[1]) * t_sim;
            m_b = m_b_max.min(mempool_total_bytes);
            
            // Approximate fees
            let b_final_approx = (m_b as f64 / m_n as f64) - 1.0;
            if b_final_approx > 0.0 {
                block_fee_total = r_base * b_final_approx * b_final_approx;
            }
            
            // Remove from mempool
            let mut tx_to_remove = (m_b + t_sim - 1) / t_sim; // ceil division
            let remove_from_high = mempool[0].min(tx_to_remove);
            mempool[0] -= remove_from_high;
            tx_to_remove -= remove_from_high;
            mempool[1] = (mempool[1] - tx_to_remove).max(0);
        } else {
            // Detailed mode: per-tx fee calculation
            m_b = 0;
            let mut blockfilled: [i64; 2] = [0, 0];
            let mut break_flag = false;
            
            for k in 0..2 {
                if break_flag { break; }
                
                for l in 0..mempool[k] {
                    if m_b >= m_b_max {
                        blockfilled[0] = k as i64;
                        blockfilled[1] = l;
                        break_flag = true;
                        break;
                    }
                    
                    let b = (m_b as f64 / m_n as f64) - 1.0;
                    let mut t_t = t_sim as f64;
                    if t_t > (m_b - m_n) as f64 && m_b > m_n {
                        t_t = (m_b - m_n) as f64;
                    }
                    let b_t = t_t / m_n as f64;
                    let mut f_t = r_base * (2.0 * b * b_t + b_t * b_t);
                    if b + b_t <= 0.0 { f_t = 0.0; }
                    
                    if fees[k] < f_t {
                        blockfilled[0] = k as i64;
                        blockfilled[1] = l;
                        break_flag = true;
                        break;
                    }
                    
                    m_b += t_sim;
                }
            }
            
            // Handle case where all tx were processed
            if blockfilled[0] == 0 && blockfilled[1] == 0 {
                if mempool[1] != 0 {
                    blockfilled[0] = 1;
                    blockfilled[1] = mempool[1];
                } else if mempool[0] != 0 {
                    blockfilled[0] = 0;
                    blockfilled[1] = mempool[0];
                }
            }
            
            // Calculate fees
            for k in 0..(blockfilled[0] as usize) {
                block_fee_total += mempool[k] as f64 * fees[k];
            }
            block_fee_total += (blockfilled[1] - 1).max(0) as f64 * fees[blockfilled[0] as usize];
            
            // Remove from mempool
            for k in 0..(blockfilled[0] as usize) {
                mempool[k] = 0;
            }
            mempool[blockfilled[0] as usize] = (mempool[blockfilled[0] as usize] - blockfilled[1]).max(0);
        }
        
        cumulative_fees += block_fee_total;
        
        // ============================================
        // 4. PENALTY CALCULATION
        // ============================================
        let b_final = (m_b as f64 / m_n as f64) - 1.0;
        let p_b = if b_final > 0.0 { r_base * b_final * b_final } else { 0.0 };
        
        // ============================================
        // 5. UPDATE MEDIAN BUFFERS
        // ============================================
        m_l_buffer[m_l_head] = m_l_weight;
        m_l_head = (m_l_head + 1) % len_l;
        updates_since_sort_l += 1;
        
        m_s_buffer[m_s_head] = m_s_weight;
        m_s_head = (m_s_head + 1) % len_s;
        updates_since_sort_s += 1;
        
        m_l_prev = m_l;
        
        // ============================================
        // 6. TRACK STATS
        // ============================================
        let mempool_size_bytes = (mempool[0] + mempool[1]) * t_sim;
        if mempool_size_bytes > max_mempool { max_mempool = mempool_size_bytes; }
        if m_b > max_mb { max_mb = m_b; }
        if p_b > max_penalty { max_penalty = p_b; }
        
        // Sample for output
        if i % sample_rate == 0 {
            unsafe {
                RESULT_INDICES.push(i as u32);
                RESULT_M_B.push(m_b);
                RESULT_M_L.push(m_l);
                RESULT_M_S.push(m_s);
                RESULT_M_N.push(m_n);
                RESULT_INPUT_VOL.push(input_volume);
                RESULT_BLOCK_FEE.push(block_fee_total);
                RESULT_PENALTY.push(p_b);
                RESULT_MEMPOOL.push(mempool_size_bytes);
            }
        }
    }
    
    // Store final stats
    unsafe {
        STATS_MAX_MB = max_mb;
        STATS_MAX_PENALTY = max_penalty;
        STATS_MAX_MEMPOOL = max_mempool;
        STATS_CUMULATIVE_FEES = cumulative_fees;
    }
}

/// Allocate memory for WASM
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Free memory
#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, 0, size);
    }
}