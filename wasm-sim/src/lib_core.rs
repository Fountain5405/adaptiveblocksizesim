//! Core simulation logic (no WASM dependencies)

#[derive(Clone, Copy, Debug)]
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
}

#[derive(Clone, Debug)]
pub struct SimulationResults {
    pub max_mb: i64,
    pub max_penalty: f64,
    pub max_mempool: i64,
    pub cumulative_fees: f64,
    pub data_points: u32,
    // Detailed data for charting
    pub M_B: Vec<i64>,
    pub M_L: Vec<i64>,
    pub M_S: Vec<i64>,
    pub M_N: Vec<i64>,
    pub input_volume: Vec<i64>,
    pub block_fee: Vec<f64>,
    pub penalty: Vec<f64>,
    pub mempool_size: Vec<i64>,
    pub indices: Vec<u32>,
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

/// Main simulation function - core logic without WASM dependencies
pub fn run_simulation_core(config: SimulationConfig) -> SimulationResults {
    let n = config.n as usize;
    let mid_100k = config.mid_100k as usize;
    let mid_100 = config.mid_100 as usize;
    let len_l = mid_100k * 2;
    let len_s = mid_100 * 2;
    
    // Initialize
    let mut m_b: i64 = 0;
    let mut m_l_prev = config.steady_state;
    let mut rng = Rng::new(12345);
    
    // Circular buffers for median calculation
    let mut m_l_buffer: Vec<i64> = vec![config.steady_state; len_l];
    let mut m_l_head: usize = 0;
    let mut m_s_buffer: Vec<i64> = vec![config.steady_state; len_s];
    let mut m_s_head: usize = 0;
    
    // For median calculation, we'll sort periodically
    let mut m_l_sorted: Vec<i64> = vec![config.steady_state; len_l];
    let mut m_s_sorted: Vec<i64> = vec![config.steady_state; len_s];
    
    // PERFORMANCE FIX: Sort interval depends on exact_median flag
    let sort_interval_l = if config.exact_median {
        1  // Sort every update (matches Python bisect behavior)
    } else {
        (len_l / 2).max(1000)  // Fast mode: sort every 50k updates
    };

    let sort_interval_s = if config.exact_median {
        1  // Sort every update (matches Python bisect behavior)
    } else {
        (len_s / 2).max(10)  // Fast mode: sort every 50 updates
    };

    let mut updates_since_sort_l: usize = 0;
    let mut updates_since_sort_s: usize = 0;
    
    // Make t_sim mutable for LARGE_SIMULATION_MODE
    let mut t_sim = config.t_sim;

    // LARGE_SIMULATION_MODE tracking variables
    let mut t_sim_counter: u32 = 0;
    let mut t_sim_reset_counter: u32 = 0;
    let mut m_b_archive: Vec<i64> = if config.large_sim_mode {
        Vec::with_capacity(n)
    } else {
        Vec::new()
    };
    
    // Mempool
    let mut mempool: [i64; 2] = [0, 0];
    let mut percent_response: f64 = 0.0;
    
    // Results tracking
    let mut max_mb: i64 = 0;
    let mut max_penalty: f64 = 0.0;
    let mut max_mempool: i64 = 0;
    let mut cumulative_fees: f64 = 0.0;
    
    // Calculate sample rate - output all values to match Python
    let sample_rate = 1;  // Output all values
    let data_points = n;
    
    // Initialize data storage vectors
    let mut m_b_data: Vec<i64> = Vec::with_capacity(data_points);
    let mut m_l_data: Vec<i64> = Vec::with_capacity(data_points);
    let mut m_s_data: Vec<i64> = Vec::with_capacity(data_points);
    let mut m_n_data: Vec<i64> = Vec::with_capacity(data_points);
    let mut input_volume_data: Vec<i64> = Vec::with_capacity(data_points);
    let mut block_fee_data: Vec<f64> = Vec::with_capacity(data_points);
    let mut penalty_data: Vec<f64> = Vec::with_capacity(data_points);
    let mut mempool_size_data: Vec<i64> = Vec::with_capacity(data_points);
    let mut indices_data: Vec<u32> = Vec::with_capacity(data_points);
    
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
        let ml_upper = (config.ml_mult * m_l_prev as f64) as i64;
        let ml_lower = (m_l_prev as f64 / config.ml_mult) as i64;
        let m_l_weight = m_b.min(ml_upper).max(config.z_m).max(ml_lower);
        
        // M_S_weight calculation
        let m_s_weight = m_b.max(m_l);
        
        // M_N calculation (capped)
        let mn_cap = (config.mn_mult * m_l as f64) as i64;
        let m_n = m_s.min(mn_cap);
        
        // M_B_max
        let m_b_max = 2 * m_n;
        
        // ============================================
        // LARGE_SIMULATION_MODE: Dynamic T_sim Scaling
        // ============================================
        if config.large_sim_mode {
            // Improve simulation speed by scaling T_sim off M_S
            let scale_setting = m_s / config.z_m;
            
            // Increase T_sim if blocks are getting large
            if t_sim <= scale_setting * 800 / 2 {
                t_sim_counter += 1;
                if t_sim_counter > 500 {
                    // Halve mempool transaction counts and double T_sim
                    mempool[0] /= 2;
                    mempool[1] /= 2;
                    t_sim *= 2;
                    t_sim_counter = 0;
                }
            }
            
            // Decrease T_sim if blocks are getting small
            if t_sim >= scale_setting * 800 * 2 {
                t_sim_counter += 1;
                if t_sim_counter > 500 {
                    // Double mempool transaction counts and halve T_sim
                    mempool[0] *= 2;
                    mempool[1] *= 2;
                    t_sim /= 2;
                    t_sim_counter = 0;
                }
            }
            
            // Reset mechanism: if blocks are stuck at same size
            if i > 100 {
                if m_b_archive[i - 1] == m_b_archive[i - 60] {
                    t_sim_reset_counter += 1;
                    if t_sim_reset_counter > 20 && t_sim > 800 && m_s < m_n + t_sim {
                        // Decrease T_sim to unstick the simulation
                        mempool[0] *= 2;
                        mempool[1] *= 2;
                        t_sim /= 2;
                        t_sim_reset_counter = 0;
                    }
                }
            }
        }
        
        // Fee calculations
        let f_r = config.r_base * (config.t_r as f64) / ((m_l as f64) * (m_l as f64));
        
        // ============================================
        // 2. BROADCAST TRANSACTIONS
        // ============================================
        let mut broadcast: [i64; 2] = [0, 0];
        
        let vol: i64 = match config.run_type {
            1 => config.z_m + 100 * (i as i64),
            2 => config.z_m + 800 * (i as i64),
            3 => ((316.0 + (i as f64 / 15.0)).powi(2)) as i64,
            4 => (config.z_m as f64 * (1.6_f64.powf(9.8 + (i as f64 / 50000.0)) - 99.75)) as i64,
            5 => m_b_max,
            6 | _ => {
                let start_val: i64 = 300000;
                let ramp_delay: usize = 10;
                let ramp_days: usize = 14;
                let ramp_time = ramp_days * 720;
                
                if i <= ramp_delay {
                    start_val
                } else if i <= ramp_delay + ramp_time {
                    start_val + (((config.ramp_multiplier - 1.0) * start_val as f64 / ramp_time as f64) as i64) * ((i - ramp_delay) as i64)
                } else {
                    (config.ramp_multiplier * start_val as f64 + 220.0 * (i as f64 / 802.0).sin() * 800.0) as i64
                }
            }
        };
        
        broadcast[1] = vol / t_sim;
        
        // Add noise if enabled
        if config.add_noise && broadcast[1] > 0 {
            let noise = 0.2 * rng.normal(0.0, broadcast[1] as f64);
            broadcast[1] = (broadcast[1] + noise as i64).max(1);
        }
        
        // Users pay more if enabled
        if config.users_pay_more && broadcast[1] > 0 {
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
        
        if config.simple_blocks {
            // Simple mode: just fill block from mempool
            let mempool_total_bytes = (mempool[0] + mempool[1]) * t_sim;
            m_b = m_b_max.min(mempool_total_bytes);
            
            // Approximate fees
            let b_final_approx = (m_b as f64 / m_n as f64) - 1.0;
            if b_final_approx > 0.0 {
                block_fee_total = config.r_base * b_final_approx * b_final_approx;
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
                    let mut f_t = config.r_base * (2.0 * b * b_t + b_t * b_t);
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
        let p_b = if b_final > 0.0 { config.r_base * b_final * b_final } else { 0.0 };
        
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
        
        // Store M_B for LARGE_SIMULATION_MODE reset detection
        if config.large_sim_mode {
            m_b_archive.push(m_b);
        }
        
        // ============================================
        // 6. TRACK STATS & STORE DATA
        // ============================================
        let mempool_size_bytes = (mempool[0] + mempool[1]) * t_sim;
        if mempool_size_bytes > max_mempool { max_mempool = mempool_size_bytes; }
        if m_b > max_mb { max_mb = m_b; }
        if p_b > max_penalty { max_penalty = p_b; }
        
        // Store data points for charting (sampled)
        if i % sample_rate == 0 {
            m_b_data.push(m_b);
            m_l_data.push(m_l);
            m_s_data.push(m_s);
            m_n_data.push(m_n);
            input_volume_data.push((broadcast[0] + broadcast[1]) * t_sim);
            block_fee_data.push(block_fee_total);
            penalty_data.push(p_b);
            mempool_size_data.push(mempool_size_bytes);
            indices_data.push(i as u32);
        }
    }
    
    SimulationResults {
        max_mb,
        max_penalty,
        max_mempool,
        cumulative_fees,
        data_points: data_points as u32,
        M_B: m_b_data,
        M_L: m_l_data,
        M_S: m_s_data,
        M_N: m_n_data,
        input_volume: input_volume_data,
        block_fee: block_fee_data,
        penalty: penalty_data,
        mempool_size: mempool_size_data,
        indices: indices_data,
    }
}