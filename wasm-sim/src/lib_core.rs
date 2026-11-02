//! Core simulation logic (no WASM dependencies)
//!
//! Consensus rules follow the reference implementation:
//!  - v16 ("2021 scaling", mainnet today): monero-project/monero, src/cryptonote_core/blockchain.cpp
//!  - v17 ("2026 scaling", the FCMP++ hard fork): seraphis-migration/monero branch fcmp++-stage,
//!    which implements ArticMine's MoneroScaling2026-03
//!
//! Block weights, medians and the long-term weight clamp are integers, computed the way
//! blockchain.cpp computes them. Penalties and fees are in XMR as f64.

use std::collections::{BTreeMap, VecDeque};

/// Steppable simulator for research harnesses (built around the functions below, which it does not change).
pub mod step;

/// Rule presets. `RULES_CUSTOM` uses the 2026 fee algorithm with whatever parameters are set.
pub const RULES_V16: u32 = 0;
pub const RULES_V17: u32 = 1;
pub const RULES_CUSTOM: u32 = 2;

/// Fee level used by simulated wallets: AUTO picks it the way wallet2 does by default.
pub const FEE_LEVEL_AUTO: i32 = -1;

#[derive(Clone, Copy, Debug)]
pub struct SimulationConfig {
    pub n: u32,
    pub rules: u32,
    // --- consensus ---
    pub z_m: i64,                     // minimum penalty-free zone (300000 v16, 625000 v17)
    pub lt_growth_num: i64,           // long-term weight clamp: [M_L*den/num, M_L*num/den]
    pub lt_growth_den: i64,           //   (17/10 v16, 6/5 v17)
    pub surge_factor: i64,            // M_N <= surge_factor * M_L (50 v16, 8 v17)
    pub long_window: u32,             // 100000
    pub short_window: u32,            // 100
    pub initial_weight: i64,          // fills both median windows at block 0
    pub sanity_cap: bool,             // Dec 2025 draft only, not adopted
    pub sanity_start_weight: i64,
    pub sanity_start_block: u32,
    // --- fees ---
    pub t_r: i64,                     // reference tx weight (3000 v16, 12500 v17)
    pub min_fee_fraction: f64,        // node relay minimum as a fraction of the penalty (0.95 v16, 1.0 v17)
    pub wallet_grace_blocks: u32,     // 10 v16, 1000 v17
    pub fee_rounding_digits: u32,     // wallet fees rounded up to this many significant digits
    pub fee_level: i32,               // FEE_LEVEL_AUTO or a fixed index into the wallet fee levels
    pub r_base: f64,                  // base block reward (tail emission: 0.6)
    // --- demand and miners ---
    pub t_sim: i64,                   // weight of one simulated transaction
    pub run_type: u32,
    pub ramp_multiplier: f64,
    pub add_noise: bool,
    pub users_pay_more: bool,
    pub rational_miners: bool,        // false: miners fill blocks to the maximum regardless of fees
}

#[derive(Clone, Debug)]
pub struct SimulationResults {
    pub max_mb: i64,
    pub max_penalty: f64,
    pub max_mempool: i64,
    pub cumulative_fees: f64,
    pub rejected_txs: i64,
    pub data_points: u32,
    pub fee_levels: Vec<f64>,         // wallet fee levels per byte at the last block
    pub min_fee_per_byte: f64,        // node relay minimum per byte at the last block
    pub M_B: Vec<i64>,
    pub M_L: Vec<i64>,
    pub M_S: Vec<i64>,
    pub M_N: Vec<i64>,
    pub input_volume: Vec<i64>,
    pub block_fee: Vec<f64>,
    pub penalty: Vec<f64>,
    pub mempool_size: Vec<i64>,
    pub fee_low: Vec<f64>,            // lowest wallet fee level per byte
    pub indices: Vec<u32>,
}

impl SimulationConfig {
    /// Defaults matching the reference code for a rule preset.
    pub fn preset(rules: u32) -> Self {
        let v16 = rules == RULES_V16;
        SimulationConfig {
            n: 200_000,
            rules,
            z_m: if v16 { 300_000 } else { 625_000 },
            lt_growth_num: if v16 { 17 } else { 6 },
            lt_growth_den: if v16 { 10 } else { 5 },
            surge_factor: if v16 { 50 } else { 8 },
            long_window: 100_000,
            short_window: 100,
            initial_weight: if v16 { 300_000 } else { 625_000 },
            sanity_cap: false,
            sanity_start_weight: 10_000_000,
            sanity_start_block: 0,
            t_r: if v16 { 3_000 } else { 12_500 },
            min_fee_fraction: if v16 { 0.95 } else { 1.0 },
            wallet_grace_blocks: if v16 { 10 } else { 1000 },
            fee_rounding_digits: 2,
            fee_level: FEE_LEVEL_AUTO,
            r_base: 0.6,
            t_sim: if v16 { 2_000 } else { 9_093 },
            run_type: 6,
            ramp_multiplier: 3.0,
            add_noise: false,
            users_pay_more: false,
            rational_miners: true,
        }
    }
}

// ---------------------------------------------------------------------------------------
// Sliding-window order statistics
// ---------------------------------------------------------------------------------------

/// Multiset of i64 with O(log n) insert/remove and min/max.
#[derive(Default)]
struct Multiset {
    map: BTreeMap<i64, u32>,
    len: usize,
}

impl Multiset {
    fn insert(&mut self, x: i64) {
        *self.map.entry(x).or_insert(0) += 1;
        self.len += 1;
    }
    fn remove(&mut self, x: i64) -> bool {
        match self.map.get_mut(&x) {
            Some(c) => {
                *c -= 1;
                if *c == 0 {
                    self.map.remove(&x);
                }
                self.len -= 1;
                true
            }
            None => false,
        }
    }
    fn max(&self) -> Option<i64> {
        self.map.keys().next_back().copied()
    }
    fn min(&self) -> Option<i64> {
        self.map.keys().next().copied()
    }
}

/// The latest `size` values of a stream, with the `low_target` smallest kept in `low`.
/// That makes the k-th smallest value (k = low_target - 1) and the next one O(1) to read
/// and O(log n) to maintain, which is what an exact rolling median needs.
struct SlidingOrderStat {
    size: usize,
    low_target: usize,
    values: VecDeque<i64>,
    low: Multiset,
    high: Multiset,
}

impl SlidingOrderStat {
    fn new(size: usize, low_target: usize, fill: i64) -> Self {
        assert!(size >= 1 && low_target >= 1 && low_target <= size);
        let mut s = SlidingOrderStat {
            size,
            low_target,
            values: VecDeque::with_capacity(size + 1),
            low: Multiset::default(),
            high: Multiset::default(),
        };
        for _ in 0..size {
            s.push(fill);
        }
        s
    }

    /// Order statistics for the median of a window of `size`, the way epee's median() takes it.
    fn median_of(size: usize, fill: i64) -> Self {
        // even size: average of elements size/2-1 and size/2; odd: element (size-1)/2
        let low_target = if size % 2 == 0 { size / 2 } else { (size + 1) / 2 };
        Self::new(size, low_target, fill)
    }

    fn push(&mut self, x: i64) {
        self.values.push_back(x);
        match self.low.max() {
            Some(m) if x <= m => self.low.insert(x),
            None => self.low.insert(x),
            _ => self.high.insert(x),
        }
        if self.values.len() > self.size {
            let old = self.values.pop_front().unwrap();
            let in_low = matches!(self.low.max(), Some(m) if old <= m);
            if !(in_low && self.low.remove(old)) {
                let removed = self.high.remove(old);
                debug_assert!(removed);
            }
        }
        self.rebalance();
    }

    fn rebalance(&mut self) {
        while self.low.len > self.low_target {
            let m = self.low.max().unwrap();
            self.low.remove(m);
            self.high.insert(m);
        }
        while self.low.len < self.low_target && self.high.len > 0 {
            let m = self.high.min().unwrap();
            self.high.remove(m);
            self.low.insert(m);
        }
    }

    /// k-th smallest value (k = low_target - 1).
    fn kth(&self) -> i64 {
        self.low.max().unwrap()
    }

    /// Median as epee computes it (integer average for even windows).
    fn median(&self) -> i64 {
        if self.size % 2 == 0 {
            (self.kth() + self.high.min().unwrap()) / 2
        } else {
            self.kth()
        }
    }
}

/// Median of `[0; grace] ++ window`, where `window` holds the latest `size - grace` values.
/// This is how wallets estimate fees: the node's rolling median with `grace` empty blocks added.
struct GraceMedian {
    stat: SlidingOrderStat,
    size: usize,
    grace: usize,
}

impl GraceMedian {
    fn new(size: usize, grace: usize, fill: i64) -> Self {
        let grace = grace.min(size / 2 - 1);
        let kept = size - grace;
        // The g zeros sort first, so the median's rank in the kept values drops by g.
        let low_target = if size % 2 == 0 { size / 2 - grace } else { (size + 1) / 2 - grace };
        GraceMedian { stat: SlidingOrderStat::new(kept, low_target, fill), size, grace }
    }
    fn push(&mut self, x: i64) {
        self.stat.push(x);
    }
    fn median(&self) -> i64 {
        let _ = self.grace;
        if self.size % 2 == 0 {
            (self.stat.kth() + self.stat.high.min().unwrap()) / 2
        } else {
            self.stat.kth()
        }
    }
}

// ---------------------------------------------------------------------------------------
// Fees
// ---------------------------------------------------------------------------------------

/// Round up to `digits` significant digits (cryptonote::round_money_up).
fn round_up_significant(value: f64, digits: u32) -> f64 {
    if value <= 0.0 || !value.is_finite() || digits == 0 {
        return value;
    }
    let exponent = value.log10().floor() as i32 - digits as i32 + 1;
    let factor = 10f64.powi(exponent);
    let scaled = value / factor;
    // guard against 0.12 / 0.01 = 12.000000000000002 rounding up to 13
    let rounded = if (scaled - scaled.round()).abs() < 1e-9 { scaled.round() } else { scaled.ceil() };
    rounded * factor
}

/// Wallet fee levels per byte, lowest first.
/// 2026: get_dynamic_base_fee_estimate_2026_scaling. 2021: get_dynamic_base_fee_estimate_2021_scaling.
fn wallet_fee_levels(c: &SimulationConfig, mlw: i64, msw_raw: i64) -> Vec<f64> {
    let r = c.r_base;
    let tr = c.t_r as f64;
    let digits = c.fee_rounding_digits;
    if c.rules == RULES_V16 {
        let mlw = mlw.max(c.z_m) as f64;                    // Mlw_penalty_free_zone_for_wallet
        let msw = (msw_raw as f64).max(mlw);                 // Msw_effective_short_term_median
        let mnw = msw.min(c.surge_factor as f64 * mlw);
        let mfw = mnw.min(mlw);
        let zm = c.z_m as f64;
        let fl = r * tr / (mfw * mfw);
        let fn_ = 4.0 * fl;
        let fm = 16.0 * r * tr / (zm * mfw);
        let fh = (4.0 * fm).max(4.0 * fm * mfw / (32.0 * tr * mnw / zm));
        [fl, fn_, fm, fh].iter().map(|&f| round_up_significant(f, digits)).collect()
    } else {
        let mfw = mlw.max(c.z_m) as f64;
        let fl = r * tr / (mfw * mfw);
        let fx = 2.0 * r / mfw;                              // max marginal penalty rate
        [fl, 4.0 * fl, 16.0 * fl, 64.0 * fl, fx]
            .iter()
            .map(|&f| round_up_significant(f, digits))
            .collect()
    }
}

/// Node relay minimum per byte (Blockchain::get_dynamic_base_fee with M_F = M_L).
fn min_fee_per_byte(c: &SimulationConfig, m_l: i64) -> f64 {
    let ml = m_l.max(c.z_m) as f64;
    c.min_fee_fraction * c.r_base * c.t_r as f64 / (ml * ml)
}

/// Block reward penalty for a block of weight `w` given penalty median `m_n`.
fn penalty(r: f64, w: i64, m_n: i64) -> f64 {
    let b = w as f64 / m_n as f64 - 1.0;
    if b > 0.0 { r * b * b } else { 0.0 }
}

/// How many transactions of weight `t`, each paying `fee`, a rational miner adds to a block
/// that currently weighs `m_b`: each one is included only if its fee covers the penalty it adds.
/// Closed form, so the cost doesn't depend on how many transactions there are.
fn affordable_txs(r: f64, m_b: i64, m_n: i64, t: i64, fee: f64, cap: i64, available: i64) -> i64 {
    if available <= 0 || m_b + t > cap {
        return 0;
    }
    let (mn, tf) = (m_n as f64, t as f64);
    // The largest weight x a block can have *before* adding a tx, such that the tx is still worth it.
    // x + t <= M_N: free. x < M_N < x + t: (x+t-M_N)^2 R/M_N^2 <= fee. x >= M_N: R(2(x-M_N)t + t^2)/M_N^2 <= fee.
    let root = mn * (fee / r).sqrt();
    let x_max = if root >= tf {
        mn + (fee * mn * mn / r - tf * tf) / (2.0 * tf)
    } else {
        mn - tf + root
    };
    if x_max < m_b as f64 {
        return 0;
    }
    let by_fee = ((x_max - m_b as f64) / tf).floor() as i64 + 1;
    let by_cap = (cap - m_b) / t;
    by_fee.min(by_cap).min(available).max(0)
}

// ---------------------------------------------------------------------------------------
// Simulation
// ---------------------------------------------------------------------------------------

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
    fn normal(&mut self, mean: f64, std_dev: f64) -> f64 {
        let u1 = self.next_f64().max(1e-10);
        let u2 = self.next_f64();
        mean + std_dev * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

fn demand_bytes(c: &SimulationConfig, i: usize, m_b_max: i64) -> i64 {
    match c.run_type {
        1 => c.z_m + 100 * i as i64,
        2 => c.z_m + 800 * i as i64,
        3 => ((316.0 + i as f64 / 15.0).powi(2)) as i64,
        4 => (c.z_m as f64 * (1.6_f64.powf(9.8 + i as f64 / 50000.0) - 99.75)) as i64,
        5 => m_b_max,
        _ => {
            let start_val: i64 = 300_000;
            let ramp_delay: usize = 10;
            let ramp_time: usize = 14 * 720;
            if i <= ramp_delay {
                start_val
            } else if i <= ramp_delay + ramp_time {
                start_val
                    + (((c.ramp_multiplier - 1.0) * start_val as f64 / ramp_time as f64) as i64)
                        * (i - ramp_delay) as i64
            } else {
                (c.ramp_multiplier * start_val as f64 + 220.0 * (i as f64 / 802.0).sin() * 800.0) as i64
            }
        }
    }
}

pub fn run_simulation_core(c: SimulationConfig) -> SimulationResults {
    let n = c.n as usize;
    let lw = c.long_window.max(2) as usize;
    let sw = c.short_window.max(2) as usize;
    let grace = c.wallet_grace_blocks as usize;
    let init = c.initial_weight;
    let t = c.t_sim.max(1);

    // Long-term weights (consensus M_L), block weights (consensus M_S), and the wallet's
    // grace-period versions of both.
    let mut long_term = SlidingOrderStat::median_of(lw, init);
    let mut short_term = SlidingOrderStat::median_of(sw, init);
    let mut long_term_wallet = GraceMedian::new(lw, grace, init);
    let mut short_term_wallet = GraceMedian::new(sw, grace.min(sw / 2 - 1), init);

    let mut rng = Rng::new(12345);
    let mut mempool: Vec<i64> = Vec::new();
    let mut percent_response: f64 = 0.0;
    let mut last10: VecDeque<i64> = VecDeque::with_capacity(11);

    let mut res = SimulationResults {
        max_mb: 0,
        max_penalty: 0.0,
        max_mempool: 0,
        cumulative_fees: 0.0,
        rejected_txs: 0,
        data_points: n as u32,
        fee_levels: Vec::new(),
        min_fee_per_byte: 0.0,
        M_B: Vec::with_capacity(n),
        M_L: Vec::with_capacity(n),
        M_S: Vec::with_capacity(n),
        M_N: Vec::with_capacity(n),
        input_volume: Vec::with_capacity(n),
        block_fee: Vec::with_capacity(n),
        penalty: Vec::with_capacity(n),
        mempool_size: Vec::with_capacity(n),
        fee_low: Vec::with_capacity(n),
        indices: Vec::with_capacity(n),
    };

    for i in 0..n {
        // 1. Medians and limits for the next block (update_next_cumulative_weight_limit)
        let m_l = long_term.median().max(c.z_m);             // long_term_effective_median_block_weight
        let m_s = short_term.median();
        let mut m_n = m_s.max(m_l).min(c.surge_factor * m_l);
        if m_n <= c.z_m {
            m_n = c.z_m;
        }
        let mut m_b_max = 2 * m_n;
        if c.sanity_cap && i >= c.sanity_start_block as usize {
            let k = (i - c.sanity_start_block as usize) as f64;
            let a_c = (c.sanity_start_weight as f64 * (1.0 + 5.0 / 4.0e6_f64).powf(k)) as i64;
            m_b_max = m_b_max.min(a_c);
        }

        // 2. Fees wallets would pay right now, and the node's relay minimum
        let levels = wallet_fee_levels(&c, long_term_wallet.median(), short_term_wallet.median());
        let f_min = min_fee_per_byte(&c, m_l);
        if mempool.len() != levels.len() {
            mempool.resize(levels.len(), 0);
        }

        // 3. New transactions
        let vol = demand_bytes(&c, i, m_b_max);
        let mut count = vol / t;
        if c.add_noise && count > 0 {
            let noise = 0.2 * rng.normal(0.0, count as f64);
            count = (count + noise as i64).max(1);
        }

        let base_level = choose_fee_level(&c, &levels, &mempool, t, m_n, &last10);
        let high_level = (base_level + 2).min(levels.len() - 1);
        let mut broadcast = vec![0i64; levels.len()];
        broadcast[base_level] = count;
        if c.users_pay_more && count > 0 && high_level != base_level {
            let calc = (mempool[base_level] as f64 / (3.0 * count as f64) * 100.0).floor();
            percent_response = (percent_response + 0.1 * (calc - percent_response)).floor().clamp(0.0, 100.0);
            let moved = (count as f64 * percent_response / 100.0) as i64;
            broadcast[high_level] += moved;
            broadcast[base_level] -= moved;
        }
        for (k, b) in broadcast.iter().enumerate() {
            // nodes accept fees down to 98% of the minimum (Blockchain::check_fee)
            if levels[k] * t as f64 >= 0.98 * f_min * t as f64 {
                mempool[k] += b;
            } else {
                res.rejected_txs += b;
            }
        }

        // 4. Build the block, highest fee level first
        let mut m_b: i64 = 0;
        let mut fees: f64 = 0.0;
        for k in (0..levels.len()).rev() {
            let fee = levels[k] * t as f64;
            let take = if c.rational_miners {
                affordable_txs(c.r_base, m_b, m_n, t, fee, m_b_max, mempool[k])
            } else {
                ((m_b_max - m_b) / t).min(mempool[k]).max(0)
            };
            mempool[k] -= take;
            m_b += take * t;
            fees += take as f64 * fee;
        }
        let p_b = penalty(c.r_base, m_b, m_n);

        // 5. Add the block to the median windows (get_next_long_term_block_weight)
        let lower = m_l * c.lt_growth_den / c.lt_growth_num;
        let upper = m_l * c.lt_growth_num / c.lt_growth_den;
        let long_term_weight = m_b.max(lower).min(upper);
        long_term.push(long_term_weight);
        long_term_wallet.push(long_term_weight);
        short_term.push(m_b);
        short_term_wallet.push(m_b);
        last10.push_back(m_b);
        if last10.len() > 10 {
            last10.pop_front();
        }

        // 6. Stats
        let mempool_bytes: i64 = mempool.iter().sum::<i64>() * t;
        res.max_mempool = res.max_mempool.max(mempool_bytes);
        res.max_mb = res.max_mb.max(m_b);
        res.max_penalty = res.max_penalty.max(p_b);
        res.cumulative_fees += fees;
        res.M_B.push(m_b);
        res.M_L.push(m_l);
        res.M_S.push(m_s);
        res.M_N.push(m_n);
        res.input_volume.push(broadcast.iter().sum::<i64>() * t);
        res.block_fee.push(fees);
        res.penalty.push(p_b);
        res.mempool_size.push(mempool_bytes);
        res.fee_low.push(levels[0]);
        res.indices.push(i as u32);
        if i + 1 == n {
            res.fee_levels = levels;
            res.min_fee_per_byte = f_min;
        }
    }
    res
}

/// Fee level a wallet picks. With the default priority, wallet2::adjust_priority pays the lowest
/// level unless the pool holds at least a block's worth of transactions (estimate_backlog) or the
/// last 10 blocks averaged more than 80% of the penalty-free size; then it pays "normal".
fn choose_fee_level(c: &SimulationConfig, levels: &[f64], mempool: &[i64], t: i64, m_n: i64, last10: &VecDeque<i64>) -> usize {
    if c.fee_level >= 0 {
        return (c.fee_level as usize).min(levels.len() - 1);
    }
    let pool_bytes: i64 = mempool.iter().sum::<i64>() * t;
    let recent: i64 = last10.iter().sum();
    let backlog = pool_bytes >= m_n;
    let busy = last10.len() == 10 && 100 * recent > 80 * 10 * m_n;
    if backlog || busy { 1 } else { 0 }
}

// ---------------------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn brute_median(v: &[i64]) -> i64 {
        let mut s = v.to_vec();
        s.sort();
        let n = s.len();
        if n % 2 == 0 { (s[n / 2 - 1] + s[n / 2]) / 2 } else { s[n / 2] }
    }

    #[test]
    fn rolling_median_matches_sorting() {
        for &size in &[1usize, 2, 3, 10, 101, 1000] {
            let mut rng = Rng::new(7 + size as u64);
            let mut m = SlidingOrderStat::median_of(size, 500);
            let mut window: VecDeque<i64> = std::iter::repeat(500).take(size).collect();
            for _ in 0..5000 {
                let x = (rng.next() % 1000) as i64; // lots of duplicates
                m.push(x);
                window.push_back(x);
                window.pop_front();
                assert_eq!(m.median(), brute_median(window.make_contiguous()));
            }
        }
    }

    #[test]
    fn grace_median_matches_appending_zeros() {
        let (size, grace) = (200usize, 20usize);
        let mut rng = Rng::new(99);
        let mut g = GraceMedian::new(size, grace, 300);
        let mut window: VecDeque<i64> = std::iter::repeat(300).take(size - grace).collect();
        for _ in 0..3000 {
            let x = 1 + (rng.next() % 900) as i64;
            g.push(x);
            window.push_back(x);
            window.pop_front();
            let mut v: Vec<i64> = window.iter().copied().collect();
            v.extend(std::iter::repeat(0).take(grace));
            assert_eq!(g.median(), brute_median(&v));
        }
    }

    #[test]
    fn round_up_significant_examples() {
        // MoneroScaling2026-03, section 4
        assert_eq!(round_up_significant(27810.0, 2), 28000.0);
        assert!((round_up_significant(37.94, 2) - 38.0).abs() < 1e-9);
        assert!((round_up_significant(0.5555, 2) - 0.56).abs() < 1e-12);
        assert!((round_up_significant(0.002342, 2) - 0.0024).abs() < 1e-15);
        assert!((round_up_significant(0.12, 2) - 0.12).abs() < 1e-15);
    }

    #[test]
    fn affordable_txs_matches_greedy_loop() {
        let r = 0.6;
        for &(m_b, m_n, t, fee) in &[
            (0i64, 625_000i64, 9_093i64, 1e-6),
            (600_000, 625_000, 9_093, 5e-5),
            (700_000, 625_000, 9_093, 2e-3),
            (0, 1_000_000, 2_000, 1e-9),
            (1_500_000, 1_000_000, 2_000, 1e-2),
        ] {
            let cap = 2 * m_n;
            let mut w = m_b;
            let mut count = 0;
            while w + t <= cap && penalty(r, w + t, m_n) - penalty(r, w, m_n) <= fee && count < 1_000_000 {
                w += t;
                count += 1;
            }
            let got = affordable_txs(r, m_b, m_n, t, fee, cap, 1_000_000);
            assert!((got - count).abs() <= 1, "m_b={m_b} fee={fee}: closed form {got}, loop {count}");
        }
    }

    #[test]
    fn v17_fee_levels_follow_spec_ratios() {
        let c = SimulationConfig::preset(RULES_V17);
        let levels = wallet_fee_levels(&c, 0, 0); // M_FW floors at Z_M = 625000
        assert_eq!(levels.len(), 5);
        let fl = 0.6 * 12_500.0 / (625_000.0f64 * 625_000.0);
        assert!((levels[0] - round_up_significant(fl, 2)).abs() < 1e-18);
        // f_X = 2 R / M_FW = (M_FW / T_R) * 2 f_L = 100 f_L at Z_M with T_R = 12500
        assert!((levels[4] - round_up_significant(100.0 * fl, 2)).abs() < 1e-15);
    }

    #[test]
    fn steady_small_blocks_keep_medians_at_zone() {
        for rules in [RULES_V16, RULES_V17] {
            let mut c = SimulationConfig::preset(rules);
            c.n = 3000;
            c.run_type = 1; // demand just above Z_M, growing slowly
            let r = run_simulation_core(c);
            assert!(r.M_L.iter().all(|&m| m == c.z_m));
            assert!(r.M_N.iter().all(|&m| m >= c.z_m && m <= c.surge_factor * c.z_m));
            assert!(r.M_B.iter().zip(&r.M_N).all(|(&b, &n)| b <= 2 * n));
        }
    }

    #[test]
    fn long_term_median_growth_is_bounded() {
        // Push demand far above the limit. Each long-term weight is clamped to 1.2x the current
        // M_L, and M_L is a median, so it can step up by at most 1.2x per half window.
        let mut c = SimulationConfig::preset(RULES_V17);
        c.n = 150_000;
        c.run_type = 5; // demand = max block weight
        c.rational_miners = false;
        let r = run_simulation_core(c);
        let ml = |i: usize| r.M_L[i] as f64;
        for i in (0..100_000).step_by(1000) {
            assert!(ml(i + 50_000) <= ml(i) * 1.2 + 1.0, "M_L grew from {} to {} in 50000 blocks", ml(i), ml(i + 50_000));
        }
        assert!(ml(100_000) > ml(0) * 1.2, "sustained demand should lift M_L past one clamp step");
        assert!(r.M_N.iter().zip(&r.M_L).all(|(&n, &l)| n <= 8 * l));
    }
}
