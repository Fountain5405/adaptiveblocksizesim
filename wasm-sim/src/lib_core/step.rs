//! Steppable simulator for research harnesses.
//!
//! This is the run loop of `run_simulation_core`, split into `begin_block` (medians, limits, fee
//! levels) and `finish_block` (arrivals, block building, window updates) so that callers can add
//! demand models, attackers, rule switches and instrumentation *around* the consensus and fee
//! functions of `lib_core`. Nothing here changes those functions: medians, clamps, the penalty,
//! fee levels, `affordable_txs` and `choose_fee_level` are called exactly as the core calls them,
//! in the same order.
//!
//! Extensions over the core loop (all off by default, so the legacy path is unchanged):
//!  - several transaction classes (weights). The pool is kept per fee level and class as FIFO
//!    queues, so wait times can be measured. Within a fee level, blocks take transactions in
//!    arrival order across classes, as monerod's pool (sorted by fee per byte, then receive time)
//!    hands them to `fill_block_template`;
//!  - transactions owned by a spam attacker, tracked separately;
//!  - a block may be padded by its miner with zero-fee filler (malicious miner);
//!  - the rule set can be switched between blocks, optionally rescaling pre-fork weights;
//!  - the median windows can be initialised from a real chain history;
//!  - pool transactions can expire after `pool_lifetime` blocks (monerod: 3 days), and the pool
//!    can be capped at `pool_max_bytes` (monerod: 648 MB). Over the cap, the lowest fee level is
//!    pruned first and, within it, the newest transactions, as `tx_memory_pool::prune` does.
//!
//! With one transaction class and the legacy demand, `run_legacy` reproduces
//! `run_simulation_core` exactly (tests at the bottom).

use super::*;

pub const OWNER_USER: usize = 0;
pub const OWNER_SPAM: usize = 1;

/// monerod's pool defaults (cryptonote_config.h): `CRYPTONOTE_MEMPOOL_TX_LIVETIME` (3 days) in
/// blocks, and `DEFAULT_TXPOOL_MAX_WEIGHT` in bytes.
pub const MONEROD_POOL_LIFETIME_BLOCKS: u32 = 3 * 720;
pub const MONEROD_POOL_MAX_BYTES: i64 = 648_000_000;

#[derive(Clone, Copy, Debug)]
struct Entry {
    arrival: u32,
    owner: u8,
    count: i64,
}

#[derive(Default, Clone)]
struct Queue {
    total: i64,
    by_owner: [i64; 2],
    fifo: VecDeque<Entry>,
}

/// Transactions taken from the pool into a block: `count` transactions of class `class`, paying
/// fee level `level`, that arrived `wait` blocks before the block that included them.
#[derive(Clone, Copy, Debug)]
pub struct Inclusion {
    pub wait: u32,
    pub count: i64,
    pub owner: u8,
    pub class: u8,
    pub level: u8,
    pub fee_per_tx: f64,
}

/// Why pool transactions left the pool without being mined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropReason {
    /// Older than `pool_lifetime`.
    Expired,
    /// Pruned because the pool was over `pool_max_bytes`.
    Evicted,
}

/// Pool transactions dropped without being mined: `count` transactions of class `class` at fee
/// level `level`, broadcast at block `arrival`.
#[derive(Clone, Copy, Debug)]
pub struct Dropped {
    pub arrival: u32,
    pub count: i64,
    pub owner: u8,
    pub class: u8,
    pub level: u8,
    pub reason: DropReason,
}

/// How the 2026 spec's transitional rule treats the pre-fork long-term weights. The spec scales
/// pre-fork block weights by Z_M/Z_M_old in every median that spans the fork (M_L, M_S, M_N and
/// the wallet's M_LW) and says "the medians are then calculated normally". Short-term windows hold
/// block weights, so both modes scale them the same way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RescaleMode {
    /// Multiply the stored long-term weights, each already clamped under the old rules.
    Stored,
    /// Recompute them from the scaled block weights with the new rules' clamp, taken around the
    /// pre-fork long-term median scaled the same way (and floored at the new Z_M).
    Reclamped,
    /// Re-run the long-term computation over the scaled pre-fork block weights block by block
    /// under the new rules, as if the scaled weights had been the chain's history: each block's
    /// long-term weight is clamped around the median of the recomputed window before it, floored
    /// at the new Z_M. The blocks before the stored window are unknown; the `Stored` reading of
    /// the window stands in for them.
    Rolling,
}

impl Queue {
    fn push(&mut self, arrival: u32, owner: usize, count: i64) {
        if count <= 0 {
            return;
        }
        self.total += count;
        self.by_owner[owner] += count;
        if let Some(b) = self.fifo.back_mut() {
            if b.arrival == arrival && b.owner as usize == owner {
                b.count += count;
                return;
            }
        }
        self.fifo.push_back(Entry { arrival, owner: owner as u8, count });
    }

    /// Remove the `n` oldest transactions; returns the count taken per owner.
    fn take(&mut self, n: i64, now: u32, class: usize, level: usize, fee: f64, out: &mut Vec<Inclusion>) -> [i64; 2] {
        let mut left = n;
        let mut taken = [0i64; 2];
        while left > 0 {
            let e = self.fifo.front_mut().expect("queue total out of sync");
            let k = e.count.min(left);
            e.count -= k;
            left -= k;
            taken[e.owner as usize] += k;
            out.push(Inclusion {
                wait: now - e.arrival,
                count: k,
                owner: e.owner,
                class: class as u8,
                level: level as u8,
                fee_per_tx: fee,
            });
            if e.count == 0 {
                self.fifo.pop_front();
            }
        }
        self.total -= n;
        self.by_owner[0] -= taken[0];
        self.by_owner[1] -= taken[1];
        taken
    }

    /// Remove up to `n` of the newest transactions, from the newest batch only; returns
    /// (arrival, owner, removed).
    fn evict_back(&mut self, n: i64) -> (u32, usize, i64) {
        let e = self.fifo.back_mut().expect("evict from an empty queue");
        let k = e.count.min(n);
        e.count -= k;
        let (arrival, owner) = (e.arrival, e.owner as usize);
        if e.count == 0 {
            self.fifo.pop_back();
        }
        self.total -= k;
        self.by_owner[owner] -= k;
        (arrival, owner, k)
    }
}

/// Limits and fee levels for the block about to be built.
#[derive(Clone, Debug)]
pub struct BlockContext {
    pub height: u32,
    pub m_l: i64,
    pub m_s: i64,
    pub m_n: i64,
    pub m_b_max: i64,
    pub levels: Vec<f64>,
    pub f_min: f64,
}

/// Who mines the block.
#[derive(Clone, Copy, Debug)]
pub enum Miner {
    /// Rational (or filling, if `rational_miners` is off), as in the core.
    Honest,
    /// Builds the block like an honest miner, then pads it with zero-fee filler up to `target`
    /// bytes (capped at the maximum block weight), paying the extra penalty itself.
    Pad { target: i64 },
}

#[derive(Clone, Debug, Default)]
pub struct BlockOutcome {
    pub m_b: i64,
    pub long_term_weight: i64,
    pub fees: f64,
    pub fees_by_owner: [f64; 2],
    pub bytes_by_owner: [i64; 2],
    pub txs_by_owner: [i64; 2],
    pub padding: i64,
    pub penalty: f64,
    /// Penalty the block would have carried without the padding.
    pub penalty_unpadded: f64,
    pub base_level: usize,
    /// User transactions broadcast this block (accepted or not), and their bytes.
    pub broadcast_count: i64,
    pub broadcast_bytes: i64,
    pub rejected: [i64; 2],
    /// Transactions dropped from the pool this block because they were too old.
    pub expired: [i64; 2],
    /// Transactions pruned from the pool this block because it was over its size cap.
    pub evicted: [i64; 2],
    pub pool_bytes: [i64; 2],
}

pub struct Sim {
    pub c: SimulationConfig,
    classes: Vec<i64>,
    long_term: SlidingOrderStat,
    short_term: SlidingOrderStat,
    long_term_wallet: GraceMedian,
    short_term_wallet: GraceMedian,
    /// Raw block weights of the blocks in the long-term window, oldest first (for rescaling).
    raw_long: VecDeque<i64>,
    last10: VecDeque<i64>,
    pool: Vec<Vec<Queue>>, // [fee level][class]
    percent_response: f64,
    height: u32,
    ctx: Option<BlockContext>,
    expired_now: [i64; 2],
    /// Inclusions of the last finished block.
    pub inclusions: Vec<Inclusion>,
    /// Transactions dropped without being mined during the last block (expired, evicted).
    pub drops: Vec<Dropped>,
    /// Drop pool transactions once they are this many blocks old (None: never, as in the core).
    pub pool_lifetime: Option<u32>,
    /// Cap the pool at this many bytes (None: unbounded, as in the core).
    pub pool_max_bytes: Option<i64>,
}

impl Sim {
    /// Fresh chain: both median windows filled with `c.initial_weight`, as the core does.
    pub fn new(c: SimulationConfig, classes: &[i64]) -> Self {
        assert!(!classes.is_empty() && classes.len() < 256);
        let lw = c.long_window.max(2) as usize;
        let sw = c.short_window.max(2) as usize;
        let grace = c.wallet_grace_blocks as usize;
        let init = c.initial_weight;
        Sim {
            c,
            classes: classes.iter().map(|&w| w.max(1)).collect(),
            long_term: SlidingOrderStat::median_of(lw, init),
            short_term: SlidingOrderStat::median_of(sw, init),
            long_term_wallet: GraceMedian::new(lw, grace, init),
            short_term_wallet: GraceMedian::new(sw, grace.min(sw / 2 - 1), init),
            raw_long: std::iter::repeat(init).take(lw).collect(),
            last10: VecDeque::with_capacity(11),
            pool: Vec::new(),
            percent_response: 0.0,
            height: 0,
            ctx: None,
            expired_now: [0; 2],
            inclusions: Vec::new(),
            drops: Vec::new(),
            pool_lifetime: None,
            pool_max_bytes: None,
        }
    }

    /// Chain whose windows hold a real history: `long_term_weights` and `block_weights` of the same
    /// blocks, oldest first (only the latest window's worth of each is used; shorter histories are
    /// preceded by `c.initial_weight`).
    pub fn with_history(c: SimulationConfig, classes: &[i64], long_term_weights: &[i64], block_weights: &[i64]) -> Self {
        assert_eq!(long_term_weights.len(), block_weights.len(), "history columns differ in length");
        let mut s = Sim::new(c, classes);
        for (&lt, &w) in long_term_weights.iter().zip(block_weights) {
            s.long_term.push(lt);
            s.long_term_wallet.push(lt);
            s.push_raw_long(w);
            s.short_term.push(w);
            s.short_term_wallet.push(w);
            s.last10.push_back(w);
            if s.last10.len() > 10 {
                s.last10.pop_front();
            }
        }
        s
    }

    fn push_raw_long(&mut self, w: i64) {
        self.raw_long.push_back(w);
        if self.raw_long.len() > self.long_term.size {
            self.raw_long.pop_front();
        }
    }

    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn classes(&self) -> &[i64] {
        &self.classes
    }

    /// Pool contents in bytes, per owner.
    pub fn pool_bytes(&self) -> [i64; 2] {
        let mut b = [0i64; 2];
        for lvl in &self.pool {
            for (j, q) in lvl.iter().enumerate() {
                b[0] += q.by_owner[0] * self.classes[j];
                b[1] += q.by_owner[1] * self.classes[j];
            }
        }
        b
    }

    /// Pool transactions per owner.
    pub fn pool_count(&self) -> [i64; 2] {
        let mut b = [0i64; 2];
        for lvl in &self.pool {
            for q in lvl {
                b[0] += q.by_owner[0];
                b[1] += q.by_owner[1];
            }
        }
        b
    }

    /// Every pending batch in the pool: (arrival, owner, count).
    pub fn pending(&self) -> impl Iterator<Item = (u32, usize, i64)> + '_ {
        self.pool
            .iter()
            .flat_map(|lvl| lvl.iter())
            .flat_map(|q| q.fifo.iter().map(|e| (e.arrival, e.owner as usize, e.count)))
    }

    /// Arrival height of the oldest pending transaction of an owner.
    pub fn oldest_pending(&self, owner: usize) -> Option<u32> {
        let mut best: Option<u32> = None;
        for lvl in &self.pool {
            for q in lvl {
                if q.by_owner[owner] == 0 {
                    continue;
                }
                if let Some(e) = q.fifo.iter().find(|e| e.owner as usize == owner) {
                    best = Some(best.map_or(e.arrival, |b| b.min(e.arrival)));
                }
            }
        }
        best
    }

    /// Long-term weights currently in the consensus window, oldest first.
    pub fn long_term_window(&self) -> impl Iterator<Item = i64> + '_ {
        self.long_term.values.iter().copied()
    }

    /// Step 1-2 of the core loop: medians, limits and fee levels for the next block. Pool
    /// transactions past `pool_lifetime` are dropped first, so callers see the pool as it is.
    pub fn begin_block(&mut self) -> BlockContext {
        self.inclusions.clear();
        self.drops.clear();
        self.expired_now = [0; 2];
        if let Some(life) = self.pool_lifetime {
            let now = self.height;
            for (k, lvl) in self.pool.iter_mut().enumerate() {
                for (j, q) in lvl.iter_mut().enumerate() {
                    while let Some(e) = q.fifo.front() {
                        if e.arrival + life > now {
                            break;
                        }
                        let e = q.fifo.pop_front().unwrap();
                        q.total -= e.count;
                        q.by_owner[e.owner as usize] -= e.count;
                        self.expired_now[e.owner as usize] += e.count;
                        self.drops.push(Dropped {
                            arrival: e.arrival,
                            count: e.count,
                            owner: e.owner,
                            class: j as u8,
                            level: k as u8,
                            reason: DropReason::Expired,
                        });
                    }
                }
            }
        }

        let c = &self.c;
        let i = self.height as usize;
        // 1. Medians and limits for the next block (update_next_cumulative_weight_limit)
        let m_l = self.long_term.median().max(c.z_m);
        let m_s = self.short_term.median();
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
        let levels = wallet_fee_levels(c, self.long_term_wallet.median(), self.short_term_wallet.median());
        let f_min = min_fee_per_byte(c, m_l);
        if self.pool.len() != levels.len() {
            let nc = self.classes.len();
            self.pool.resize_with(levels.len(), || vec![Queue::default(); nc]);
        }
        let ctx = BlockContext { height: self.height, m_l, m_s, m_n, m_b_max, levels, f_min };
        self.ctx = Some(ctx.clone());
        ctx
    }

    /// The level a wallet using the default priority would pick now (call between
    /// `begin_block` and `finish_block`).
    pub fn auto_level(&self, ctx: &BlockContext) -> usize {
        let pool_bytes: i64 = self.pool_bytes().iter().sum();
        let mut cc = self.c;
        cc.fee_level = FEE_LEVEL_AUTO;
        choose_fee_level(&cc, &ctx.levels, &[pool_bytes], 1, ctx.m_n, &self.last10)
    }

    /// Steps 3-5 of the core loop. `user` holds the user transactions broadcast this block per
    /// class; `spam` holds (class, fee level, count) for the attacker's transactions.
    pub fn finish_block(&mut self, user: &[i64], spam: &[(usize, usize, i64)], miner: Miner) -> BlockOutcome {
        let ctx = self.ctx.take().expect("finish_block without begin_block");
        let c = self.c;
        let levels = &ctx.levels;
        let nl = levels.len();
        let nc = self.classes.len();
        let mut out = BlockOutcome { expired: self.expired_now, ..Default::default() };
        let now = self.height;

        // 3. New transactions
        let pool_bytes_all: i64 = self.pool_bytes().iter().sum();
        let base_level = choose_fee_level(&c, levels, &[pool_bytes_all], 1, ctx.m_n, &self.last10);
        let high_level = (base_level + 2).min(nl - 1);
        let count: i64 = user.iter().sum();
        // broadcast[class][level]
        let mut broadcast = vec![vec![0i64; nl]; nc];
        for j in 0..nc {
            broadcast[j][base_level] = user.get(j).copied().unwrap_or(0);
        }
        if c.users_pay_more && count > 0 && high_level != base_level {
            let at_base: i64 = self.pool[base_level].iter().map(|q| q.total).sum();
            let calc = (at_base as f64 / (3.0 * count as f64) * 100.0).floor();
            self.percent_response =
                (self.percent_response + 0.1 * (calc - self.percent_response)).floor().clamp(0.0, 100.0);
            for j in 0..nc {
                let moved = (broadcast[j][base_level] as f64 * self.percent_response / 100.0) as i64;
                broadcast[j][high_level] += moved;
                broadcast[j][base_level] -= moved;
            }
        }
        for j in 0..nc {
            let t = self.classes[j];
            for k in 0..nl {
                let b = broadcast[j][k];
                out.broadcast_count += b;
                out.broadcast_bytes += b * t;
                // nodes accept fees down to 98% of the minimum (Blockchain::check_fee)
                if levels[k] * t as f64 >= 0.98 * ctx.f_min * t as f64 {
                    self.pool[k][j].push(now, OWNER_USER, b);
                } else {
                    out.rejected[OWNER_USER] += b;
                }
            }
        }
        for &(j, k, b) in spam {
            let k = k.min(nl - 1);
            let t = self.classes[j];
            if levels[k] * t as f64 >= 0.98 * ctx.f_min * t as f64 {
                self.pool[k][j].push(now, OWNER_SPAM, b);
            } else {
                out.rejected[OWNER_SPAM] += b;
            }
        }

        // 3b. Pool size cap (tx_memory_pool::prune after each add): drop from the lowest fee level
        //     upwards and, within a level, the newest transactions first (larger ones first among
        //     equally new ones), until the pool fits.
        if let Some(cap) = self.pool_max_bytes {
            let mut bytes: i64 = self.pool_bytes().iter().sum();
            let mut k = 0;
            while bytes > cap && k < nl {
                let mut pick: Option<usize> = None;
                for j in 0..nc {
                    if let Some(e) = self.pool[k][j].fifo.back() {
                        pick = match pick {
                            Some(p) if {
                                let b = self.pool[k][p].fifo.back().unwrap();
                                b.arrival > e.arrival || (b.arrival == e.arrival && self.classes[p] > self.classes[j])
                            } => Some(p),
                            _ => Some(j),
                        };
                    }
                }
                let Some(j) = pick else {
                    k += 1;
                    continue;
                };
                let t = self.classes[j];
                let need = (bytes - cap + t - 1) / t;
                let (arrival, owner, removed) = self.pool[k][j].evict_back(need);
                bytes -= removed * t;
                out.evicted[owner] += removed;
                self.drops.push(Dropped {
                    arrival,
                    count: removed,
                    owner: owner as u8,
                    class: j as u8,
                    level: k as u8,
                    reason: DropReason::Evicted,
                });
            }
        }

        // 4. Build the block, highest fee level first. Within a level, transactions go in arrival
        //    order across classes (equally new ones: smaller first). A class whose next transaction
        //    doesn't pay for its penalty, or doesn't fit, is passed over for the rest of the level,
        //    as fill_block_template passes over it; smaller ones may still go in.
        let mut m_b: i64 = 0;
        let mut fees: f64 = 0.0;
        let mut passed = vec![false; nc];
        for k in (0..nl).rev() {
            passed.iter_mut().for_each(|p| *p = false);
            loop {
                let mut pick: Option<usize> = None;
                let mut candidates = 0;
                for j in 0..nc {
                    if passed[j] || self.pool[k][j].total == 0 {
                        continue;
                    }
                    candidates += 1;
                    let a = self.pool[k][j].fifo.front().unwrap().arrival;
                    pick = match pick {
                        Some(p) if {
                            let ap = self.pool[k][p].fifo.front().unwrap().arrival;
                            ap < a || (ap == a && self.classes[p] < self.classes[j])
                        } => Some(p),
                        _ => Some(j),
                    };
                }
                let Some(j) = pick else { break };
                let t = self.classes[j];
                let q = &mut self.pool[k][j];
                let fee = levels[k] * t as f64;
                // With one class left, offer its whole queue in one go: with a single class this is
                // exactly the core's step. Otherwise one arrival batch at a time.
                let avail = if candidates == 1 { q.total } else { q.fifo.front().unwrap().count };
                let take = if c.rational_miners {
                    affordable_txs(c.r_base, m_b, ctx.m_n, t, fee, ctx.m_b_max, avail)
                } else {
                    ((ctx.m_b_max - m_b) / t).min(avail).max(0)
                };
                if take > 0 {
                    let by = q.take(take, now, j, k, fee, &mut self.inclusions);
                    for o in 0..2 {
                        out.fees_by_owner[o] += by[o] as f64 * fee;
                        out.bytes_by_owner[o] += by[o] * t;
                        out.txs_by_owner[o] += by[o];
                    }
                }
                m_b += take * t;
                fees += take as f64 * fee;
                if take < avail {
                    passed[j] = true;
                }
            }
        }
        out.penalty_unpadded = penalty(c.r_base, m_b, ctx.m_n);
        if let Miner::Pad { target } = miner {
            let target = target.min(ctx.m_b_max);
            if target > m_b {
                out.padding = target - m_b;
                m_b = target;
            }
        }
        let p_b = penalty(c.r_base, m_b, ctx.m_n);

        // 5. Add the block to the median windows (get_next_long_term_block_weight)
        let lower = ctx.m_l * c.lt_growth_den / c.lt_growth_num;
        let upper = ctx.m_l * c.lt_growth_num / c.lt_growth_den;
        let long_term_weight = m_b.max(lower).min(upper);
        self.long_term.push(long_term_weight);
        self.long_term_wallet.push(long_term_weight);
        self.push_raw_long(m_b);
        self.short_term.push(m_b);
        self.short_term_wallet.push(m_b);
        self.last10.push_back(m_b);
        if self.last10.len() > 10 {
            self.last10.pop_front();
        }

        out.m_b = m_b;
        out.long_term_weight = long_term_weight;
        out.fees = fees;
        out.penalty = p_b;
        out.base_level = base_level;
        out.pool_bytes = self.pool_bytes();
        self.height += 1;
        out
    }

    /// Switch to another rule set between blocks (a hard fork), keeping the median windows.
    /// With `rescale = Some((num, den))` the 2026 spec's transitional rule is applied: pre-fork
    /// block weights count as weight × num/den in every median that spans the fork (consensus and
    /// wallet, long- and short-term). Applying it once to every weight in the windows at the fork
    /// is the same thing, since only pre-fork weights are in them. `mode` says how the long-term
    /// weights are derived (see `RescaleMode`). `None` is the reference code's behaviour: the
    /// windows carry across unchanged.
    pub fn switch_rules(&mut self, new_c: SimulationConfig, rescale: Option<(i64, i64)>, mode: RescaleMode) {
        let scale = |w: i64| match rescale {
            Some((num, den)) => w * num / den,
            None => w,
        };
        let lt: Vec<i64> = match rescale {
            Some((num, den)) if mode == RescaleMode::Reclamped => {
                let ml_old = self.long_term.median().max(self.c.z_m);
                let ml = (ml_old * num / den).max(new_c.z_m);
                let lower = ml * new_c.lt_growth_den / new_c.lt_growth_num;
                let upper = ml * new_c.lt_growth_num / new_c.lt_growth_den;
                self.raw_long.iter().map(|&w| (w * num / den).max(lower).min(upper)).collect()
            }
            Some((num, den)) if mode == RescaleMode::Rolling => {
                let seed: Vec<i64> = self.long_term.values.iter().map(|&w| scale(w)).collect();
                debug_assert_eq!(seed.len(), self.raw_long.len());
                // The window before block i: the seed values for the blocks not yet recomputed
                // (standing in for the unknown history before the stored window), then the
                // recomputed weights of the blocks before i.
                let mut med = SlidingOrderStat::median_of(seed.len().max(2), seed[0]);
                for &w in &seed {
                    med.push(w);
                }
                let mut out = Vec::with_capacity(seed.len());
                for &raw in &self.raw_long {
                    let ml = med.median().max(new_c.z_m);
                    let lower = ml * new_c.lt_growth_den / new_c.lt_growth_num;
                    let upper = ml * new_c.lt_growth_num / new_c.lt_growth_den;
                    let w = (raw * num / den).max(lower).min(upper);
                    out.push(w);
                    med.push(w);
                }
                out
            }
            _ => self.long_term.values.iter().map(|&w| scale(w)).collect(),
        };
        let st: Vec<i64> = self.short_term.values.iter().map(|&w| scale(w)).collect();
        let lw = new_c.long_window.max(2) as usize;
        let sw = new_c.short_window.max(2) as usize;
        let grace = new_c.wallet_grace_blocks as usize;
        let lt_fill = lt[0];
        let st_fill = st[0];
        self.long_term = SlidingOrderStat::median_of(lw, lt_fill);
        self.long_term_wallet = GraceMedian::new(lw, grace, lt_fill);
        for &w in &lt {
            self.long_term.push(w);
            self.long_term_wallet.push(w);
        }
        self.short_term = SlidingOrderStat::median_of(sw, st_fill);
        self.short_term_wallet = GraceMedian::new(sw, grace.min(sw / 2 - 1), st_fill);
        for &w in &st {
            self.short_term.push(w);
            self.short_term_wallet.push(w);
        }
        while self.raw_long.len() > lw {
            self.raw_long.pop_front();
        }
        self.c = new_c;
    }
}

/// Deterministic random numbers for harnesses (the core's xorshift generator).
pub struct StudyRng(Rng);

impl StudyRng {
    pub fn new(seed: u64) -> Self {
        StudyRng(Rng::new(seed.max(1)))
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0.next()
    }
    pub fn next_f64(&mut self) -> f64 {
        self.0.next_f64()
    }
    pub fn normal(&mut self, mean: f64, std_dev: f64) -> f64 {
        self.0.normal(mean, std_dev)
    }
    /// Poisson-distributed count: Knuth's method below 30, Hörmann's PTRS above (exact).
    pub fn poisson(&mut self, lam: f64) -> i64 {
        if !(lam > 0.0) {
            return 0;
        }
        if lam < 30.0 {
            let l = (-lam).exp();
            let mut k = 0i64;
            let mut p = 1.0;
            loop {
                p *= self.next_f64();
                if p <= l {
                    return k;
                }
                k += 1;
            }
        }
        let slam = lam.sqrt();
        let loglam = lam.ln();
        let b = 0.931 + 2.53 * slam;
        let a = -0.059 + 0.02483 * b;
        let invalpha = 1.1239 + 1.1328 / (b - 3.4);
        let vr = 0.9277 - 3.6224 / (b - 2.0);
        loop {
            let u = self.next_f64() - 0.5;
            let v = self.next_f64();
            let us = 0.5 - u.abs();
            let k = ((2.0 * a / us + b) * u + lam + 0.43).floor();
            if us >= 0.07 && v <= vr {
                return k as i64;
            }
            if k < 0.0 || (us < 0.013 && v > us) {
                continue;
            }
            if v.ln() + invalpha.ln() - (a / (us * us) + b).ln() <= -lam + k * loglam - ln_factorial(k) {
                return k as i64;
            }
        }
    }
}

fn ln_factorial(k: f64) -> f64 {
    if k < 10.0 {
        let mut f = 1.0f64;
        let mut i = 2.0;
        while i <= k {
            f *= i;
            i += 1.0;
        }
        f.ln()
    } else {
        // Stirling series for ln Gamma(k + 1)
        let x = k + 1.0;
        (x - 0.5) * x.ln() - x + 0.5 * (2.0 * std::f64::consts::PI).ln() + 1.0 / (12.0 * x) - 1.0 / (360.0 * x * x * x)
            + 1.0 / (1260.0 * x.powi(5))
    }
}

/// `run_simulation_core` rebuilt on `Sim`, with the core's demand models and noise.
pub fn run_legacy(c: SimulationConfig) -> SimulationResults {
    let n = c.n as usize;
    let t = c.t_sim.max(1);
    let mut sim = Sim::new(c, &[t]);
    let mut rng = Rng::new(12345);
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
        let ctx = sim.begin_block();
        let vol = demand_bytes(&c, i, ctx.m_b_max);
        let mut count = vol / t;
        if c.add_noise && count > 0 {
            let noise = 0.2 * rng.normal(0.0, count as f64);
            count = (count + noise as i64).max(1);
        }
        let o = sim.finish_block(&[count], &[], Miner::Honest);
        let mempool_bytes = o.pool_bytes[0] + o.pool_bytes[1];
        res.rejected_txs += o.rejected[0];
        res.max_mempool = res.max_mempool.max(mempool_bytes);
        res.max_mb = res.max_mb.max(o.m_b);
        res.max_penalty = res.max_penalty.max(o.penalty);
        res.cumulative_fees += o.fees;
        res.M_B.push(o.m_b);
        res.M_L.push(ctx.m_l);
        res.M_S.push(ctx.m_s);
        res.M_N.push(ctx.m_n);
        res.input_volume.push(o.broadcast_bytes);
        res.block_fee.push(o.fees);
        res.penalty.push(o.penalty);
        res.mempool_size.push(mempool_bytes);
        res.fee_low.push(ctx.levels[0]);
        res.indices.push(i as u32);
        if i + 1 == n {
            res.fee_levels = ctx.levels.clone();
            res.min_fee_per_byte = ctx.f_min;
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_same(a: &SimulationResults, b: &SimulationResults, label: &str) {
        assert_eq!(a.M_B, b.M_B, "{label}: M_B");
        assert_eq!(a.M_L, b.M_L, "{label}: M_L");
        assert_eq!(a.M_S, b.M_S, "{label}: M_S");
        assert_eq!(a.M_N, b.M_N, "{label}: M_N");
        assert_eq!(a.input_volume, b.input_volume, "{label}: input_volume");
        assert_eq!(a.mempool_size, b.mempool_size, "{label}: mempool");
        let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        assert_eq!(bits(&a.block_fee), bits(&b.block_fee), "{label}: block_fee");
        assert_eq!(bits(&a.penalty), bits(&b.penalty), "{label}: penalty");
        assert_eq!(bits(&a.fee_low), bits(&b.fee_low), "{label}: fee_low");
        assert_eq!(bits(&a.fee_levels), bits(&b.fee_levels), "{label}: fee_levels");
        assert_eq!(a.max_mb, b.max_mb);
        assert_eq!(a.max_mempool, b.max_mempool);
        assert_eq!(a.rejected_txs, b.rejected_txs);
        assert_eq!(a.max_penalty.to_bits(), b.max_penalty.to_bits());
        assert_eq!(a.cumulative_fees.to_bits(), b.cumulative_fees.to_bits(), "{label}: cumulative_fees");
        assert_eq!(a.min_fee_per_byte.to_bits(), b.min_fee_per_byte.to_bits());
    }

    /// The steppable simulator reproduces run_simulation_core bit for bit for every run type,
    /// on both presets and Custom, with and without rational miners, noise, users-pay-more and
    /// fixed fee levels.
    #[test]
    fn stepper_reproduces_core_exactly() {
        let mut cases = 0;
        for rules in [RULES_V16, RULES_V17, RULES_CUSTOM] {
            for run_type in 1..=6u32 {
                for &(rational, noise, pay_more, level) in &[
                    (true, false, false, FEE_LEVEL_AUTO),
                    (false, false, false, FEE_LEVEL_AUTO),
                    (true, true, true, FEE_LEVEL_AUTO),
                    (true, false, true, 0),
                    (true, false, false, 3),
                ] {
                    let mut c = SimulationConfig::preset(rules);
                    c.n = 60_000;
                    c.run_type = run_type;
                    c.rational_miners = rational;
                    c.add_noise = noise;
                    c.users_pay_more = pay_more;
                    c.fee_level = level;
                    if rules == RULES_CUSTOM {
                        c.lt_growth_num = 3;
                        c.lt_growth_den = 2;
                        c.surge_factor = 10;
                        c.sanity_cap = true;
                        c.sanity_start_weight = 2_000_000;
                    }
                    let label = format!("rules={rules} rt={run_type} rational={rational} noise={noise} upm={pay_more} lvl={level}");
                    assert_same(&run_simulation_core(c), &run_legacy(c), &label);
                    cases += 1;
                }
            }
        }
        assert_eq!(cases, 90);
    }

    #[test]
    fn switch_without_rescale_keeps_medians() {
        let mut c = SimulationConfig::preset(RULES_V16);
        c.long_window = 1000;
        let mut s = Sim::new(c, &[2000]);
        for _ in 0..500 {
            let _ = s.begin_block();
            s.finish_block(&[200], &[], Miner::Honest);
        }
        let before = s.long_term.median();
        let mut c17 = SimulationConfig::preset(RULES_V17);
        c17.long_window = 1000;
        s.switch_rules(c17, None, RescaleMode::Stored);
        assert_eq!(s.long_term.median(), before);
        s.switch_rules(c17, Some((625_000, 300_000)), RescaleMode::Stored);
        assert_eq!(s.long_term.median(), before * 625_000 / 300_000);
    }

    /// Mainnet-like windows: long-term weights at the old lower clamp, a tail of larger blocks.
    #[test]
    fn rescale_modes_treat_the_long_term_tail_differently() {
        let mut c = SimulationConfig::preset(RULES_V16);
        c.long_window = 1000;
        let lt: Vec<i64> = (0..1000).map(|i| if i % 10 == 0 { 456_000 } else { 176_470 }).collect();
        let bw: Vec<i64> = (0..1000).map(|i| if i % 10 == 0 { 456_000 } else { 100_000 }).collect();
        let mut c17 = SimulationConfig::preset(RULES_V17);
        c17.long_window = 1000;

        let mut stored = Sim::with_history(c, &[2500], &lt, &bw);
        stored.switch_rules(c17, Some((625_000, 300_000)), RescaleMode::Stored);
        let w: Vec<i64> = stored.long_term_window().collect();
        assert_eq!(*w.iter().max().unwrap(), 950_000);
        assert_eq!(*w.iter().min().unwrap(), 367_645);

        // M_L before = 300k (floor), scaled 625k: new bounds [625k·5/6, 625k·6/5]
        let mut reclamped = Sim::with_history(c, &[2500], &lt, &bw);
        reclamped.switch_rules(c17, Some((625_000, 300_000)), RescaleMode::Reclamped);
        let w: Vec<i64> = reclamped.long_term_window().collect();
        assert_eq!(*w.iter().max().unwrap(), 750_000);
        assert_eq!(*w.iter().min().unwrap(), 520_833);
        // the short-term window holds block weights, scaled the same way in both modes
        assert_eq!(stored.short_term.median(), reclamped.short_term.median());

        // With M_L at the floor throughout, the rolling reading recomputes every weight around the
        // same floor, so it coincides with the re-clamped one.
        let mut rolling = Sim::with_history(c, &[2500], &lt, &bw);
        rolling.switch_rules(c17, Some((625_000, 300_000)), RescaleMode::Rolling);
        let wr: Vec<i64> = rolling.long_term_window().collect();
        assert_eq!(wr, w);
        assert_eq!(stored.short_term.median(), rolling.short_term.median());
    }

    /// Above the floor the readings differ: the rolling one lets the recomputed weights move the
    /// median they are clamped around, the re-clamped one uses the at-fork median for all of them.
    #[test]
    fn rolling_reading_differs_from_reclamped_above_the_floor() {
        let mut c = SimulationConfig::preset(RULES_V16);
        c.long_window = 1000;
        let lt: Vec<i64> = vec![1_000_000; 1000];
        let bw: Vec<i64> = (0..1000).map(|i| if i % 2 == 0 { 1_700_000 } else { 600_000 }).collect();
        let mut c17 = SimulationConfig::preset(RULES_V17);
        c17.long_window = 1000;

        // a steady chain stays steady under every reading
        let flat: Vec<i64> = vec![1_000_000; 1000];
        for mode in [RescaleMode::Stored, RescaleMode::Reclamped, RescaleMode::Rolling] {
            let mut s = Sim::with_history(c, &[2500], &lt, &flat);
            s.switch_rules(c17, Some((625_000, 300_000)), mode);
            assert!(s.long_term_window().all(|w| w == 1_000_000 * 625_000 / 300_000), "{mode:?}");
        }

        let mut reclamped = Sim::with_history(c, &[2500], &lt, &bw);
        reclamped.switch_rules(c17, Some((625_000, 300_000)), RescaleMode::Reclamped);
        let wc: Vec<i64> = reclamped.long_term_window().collect();
        // at-fork M_L 1M × 625/300 = 2,083,333: bounds [1,736,110, 2,499,999]
        assert_eq!(wc[0], 2_083_333 * 6 / 5);
        assert_eq!(wc[1], 2_083_333 * 5 / 6);

        let mut rolling = Sim::with_history(c, &[2500], &lt, &bw);
        rolling.switch_rules(c17, Some((625_000, 300_000)), RescaleMode::Rolling);
        let wr: Vec<i64> = rolling.long_term_window().collect();
        assert_eq!(wr[0], wc[0]); // first block: same median as the re-clamped reading
        assert_ne!(wr, wc); // later ones are clamped around a moved median
        assert_ne!(rolling.long_term.median(), reclamped.long_term.median());
    }

    /// Two classes in a growing backlog at one fee level (demand above the 8× cap, so it can't
    /// clear): neither may starve, and the outcome must not depend on the order the classes are
    /// listed in.
    #[test]
    fn mixed_classes_are_mined_in_arrival_order() {
        let mut c = SimulationConfig::preset(RULES_V17);
        c.fee_level = 1;
        let run = |classes: [i64; 2]| {
            let mut s = Sim::new(c, &classes);
            let mut max_wait = [0u32; 2];
            let mut weights = Vec::new();
            for _ in 0..3000 {
                let _ = s.begin_block();
                let o = s.finish_block(&[800, 800], &[], Miner::Honest);
                weights.push(o.m_b);
                for inc in &s.inclusions {
                    let k = if classes[inc.class as usize] == 7650 { 0 } else { 1 };
                    max_wait[k] = max_wait[k].max(inc.wait);
                }
            }
            (weights, max_wait)
        };
        let (wa, ma) = run([7650, 9093]);
        let (wb, mb) = run([9093, 7650]);
        assert_eq!(wa, wb);
        assert_eq!(ma, mb);
        assert!(ma[1] > 100, "expected a backlog, waits {ma:?}");
        assert!(ma[0].abs_diff(ma[1]) <= 2, "max waits by class: {ma:?}");
    }

    #[test]
    fn pool_cap_prunes_lowest_level_newest_first() {
        let c = SimulationConfig::preset(RULES_V17);
        let mut s = Sim::new(c, &[9093]);
        s.pool_max_bytes = Some(100 * 9093);
        let _ = s.begin_block();
        // 300 txs at the lowest level and 60 at level 2 arrive together; the pool keeps 100
        let o = s.finish_block(&[], &[(0, 0, 300), (0, 2, 60)], Miner::Honest);
        assert_eq!(o.evicted[OWNER_SPAM], 260);
        assert!(s.drops.iter().all(|d| d.reason == DropReason::Evicted && d.level == 0));
        assert!(s.inclusions.iter().any(|i| i.level == 2 && i.count == 60));
        // next block: the new lowest-level arrivals are the newest, so they are pruned first
        let _ = s.begin_block();
        let o = s.finish_block(&[], &[(0, 0, 200)], Miner::Honest);
        assert!(o.evicted[OWNER_SPAM] > 0);
        assert!(s.drops.iter().all(|d| d.arrival == 1));
        assert!(s.pool_bytes()[OWNER_SPAM] <= 100 * 9093);
    }

    #[test]
    fn spam_and_padding_are_tracked() {
        let c = SimulationConfig::preset(RULES_V17);
        let mut s = Sim::new(c, &[9093]);
        let ctx = s.begin_block();
        let o = s.finish_block(&[10], &[(0, 0, 20)], Miner::Pad { target: ctx.m_n + 100_000 });
        assert_eq!(o.txs_by_owner, [10, 20]);
        assert_eq!(o.m_b, ctx.m_n + 100_000);
        assert_eq!(o.padding, ctx.m_n + 100_000 - 30 * 9093);
        assert!(o.penalty > 0.0 && o.penalty_unpadded == 0.0);
        assert!(o.fees_by_owner[1] > 0.0);
    }

    #[test]
    fn pool_lifetime_drops_old_transactions() {
        let c = SimulationConfig::preset(RULES_V17);
        let mut s = Sim::new(c, &[9093]);
        s.pool_lifetime = Some(5);
        let mut expired = 0;
        for i in 0..20 {
            let _ = s.begin_block();
            // 200 txs per block: far more than fits, so a backlog builds and ages out
            let o = s.finish_block(&[200], &[], Miner::Honest);
            expired += o.expired[0];
            assert!(s.drops.iter().all(|d| d.reason == DropReason::Expired && i - d.arrival >= 5));
            assert!(s.oldest_pending(OWNER_USER).map_or(true, |a| i - a < 5));
        }
        assert!(expired > 0);
    }

    #[test]
    fn poisson_mean_and_variance() {
        let mut r = StudyRng::new(42);
        for &lam in &[3.0, 50.0, 2000.0] {
            let n = 100_000;
            let xs: Vec<f64> = (0..n).map(|_| r.poisson(lam) as f64).collect();
            let m = xs.iter().sum::<f64>() / n as f64;
            let v = xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / n as f64;
            assert!((m - lam).abs() < 0.02 * lam + 0.05, "lam {lam} mean {m}");
            assert!((v - lam).abs() < 0.05 * lam + 0.1, "lam {lam} var {v}");
        }
    }
}
