//! One run: build the configuration, drive `lib_core::step::Sim` block by block with the demand,
//! attackers and fork of a `RunSpec`, and collect metrics, events and a downsampled series.

use crate::spec::*;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::Value;
use std::io::Write;
use wasm_sim::lib_core::step::{BlockContext, Miner, Sim, StudyRng, OWNER_SPAM, OWNER_USER};
use wasm_sim::lib_core::{SimulationConfig, FEE_LEVEL_AUTO, RULES_CUSTOM, RULES_V16, RULES_V17};

pub const DAY: u32 = 720;
pub const MONTH: u32 = 30 * DAY;
const MB: i64 = 1_000_000;
const SIZE_THRESHOLDS: [i64; 5] = [2, 5, 10, 50, 100];

pub fn rules_id(s: &str) -> Result<u32, String> {
    match s {
        "v16" => Ok(RULES_V16),
        "v17" => Ok(RULES_V17),
        "custom" => Ok(RULES_CUSTOM),
        _ => Err(format!("unknown rules {s}")),
    }
}

fn apply_params(c: &mut SimulationConfig, p: &Params) {
    if let Some(z) = p.z_m {
        c.z_m = z;
        c.initial_weight = z;
    }
    if let Some(g) = p.lt_growth {
        c.lt_growth_num = (g * 100.0).round() as i64;
        c.lt_growth_den = 100;
    }
    if let Some([n, d]) = p.lt_growth_frac {
        c.lt_growth_num = n;
        c.lt_growth_den = d;
    }
    if let Some(v) = p.surge_factor {
        c.surge_factor = v;
    }
    if let Some(v) = p.t_r {
        c.t_r = v;
    }
    if let Some(v) = p.wallet_grace_blocks {
        c.wallet_grace_blocks = v;
    }
    if let Some(v) = p.min_fee_fraction {
        c.min_fee_fraction = v;
    }
    if let Some(v) = p.initial_weight {
        c.initial_weight = v;
    }
    if let Some(v) = p.long_window {
        c.long_window = v;
    }
    if let Some(v) = p.short_window {
        c.short_window = v;
    }
    if let Some(v) = p.r_base {
        c.r_base = v;
    }
}

fn fee_level_of(v: &Value) -> Result<i32, String> {
    match v {
        Value::String(s) if s == "auto" => Ok(FEE_LEVEL_AUTO),
        Value::Number(n) => Ok(n.as_i64().ok_or("bad fee level")? as i32),
        _ => Err(format!("bad fee_level {v}")),
    }
}

pub fn build_config(spec: &RunSpec, rules: &str, params: &Params) -> Result<SimulationConfig, String> {
    let mut c = SimulationConfig::preset(rules_id(rules)?);
    apply_params(&mut c, params);
    c.n = spec.blocks;
    c.fee_level = fee_level_of(&spec.fee_level)?;
    c.users_pay_more = spec.users_pay_more;
    c.rational_miners = spec.rational_miners;
    Ok(c)
}

/// Exact wait-time histogram (blocks), with an overflow bin past 60 days.
#[derive(Clone)]
pub struct Hist {
    bins: Vec<u64>,
    over: u64,
    pub max: u32,
    pub n: u64,
    sum: f64,
}

impl Hist {
    fn new() -> Self {
        Hist { bins: vec![0; 60 * DAY as usize + 1], over: 0, max: 0, n: 0, sum: 0.0 }
    }
    fn add(&mut self, wait: u32, count: i64) {
        let c = count as u64;
        if (wait as usize) < self.bins.len() {
            self.bins[wait as usize] += c;
        } else {
            self.over += c;
        }
        self.max = self.max.max(wait);
        self.n += c;
        self.sum += wait as f64 * count as f64;
    }
    fn mean(&self) -> Option<f64> {
        (self.n > 0).then(|| self.sum / self.n as f64)
    }
    fn pct(&self, p: f64) -> Option<u32> {
        if self.n == 0 {
            return None;
        }
        let target = (p * self.n as f64).ceil().max(1.0) as u64;
        let mut acc = 0;
        for (i, &b) in self.bins.iter().enumerate() {
            acc += b;
            if acc >= target {
                return Some(i as u32);
            }
        }
        Some(self.max)
    }
    fn share_within(&self, blocks: u32) -> Option<f64> {
        if self.n == 0 {
            return None;
        }
        let s: u64 = self.bins.iter().take(blocks as usize + 1).sum();
        Some(s as f64 / self.n as f64)
    }
}

/// Ordered key/value row.
#[derive(Default)]
pub struct Row(pub Vec<(String, String)>);

impl Row {
    fn put<T: ToString>(&mut self, k: &str, v: T) {
        self.0.push((k.to_string(), v.to_string()));
    }
    fn opt<T: ToString>(&mut self, k: &str, v: Option<T>) {
        self.0.push((k.to_string(), v.map(|x| x.to_string()).unwrap_or_default()));
    }
}

pub struct RunOutput {
    pub summary: Row,
    pub events: Vec<String>,
}

pub const EVENT_HEADER: &str = "run_id,block,event,m_l,m_s,m_n,max_weight,m_b,fee_low,f_min,pool_user_bytes,pool_spam_bytes,cum_spam_xmr,cum_miner_penalty";

struct Window {
    start: u32,
    end: u32,
}

/// Mainnet history: (long-term weights, block weights), oldest first.
pub fn load_history(path: &str) -> Result<(Vec<i64>, Vec<i64>), String> {
    use std::io::{BufRead, BufReader};
    let f = std::fs::File::open(path).map_err(|e| format!("{path}: {e}"))?;
    let r = BufReader::new(flate2::read::GzDecoder::new(f));
    let (mut lt, mut bw) = (Vec::new(), Vec::new());
    for line in r.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.starts_with('#') || line.starts_with("height") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        bw.push(f[1].parse::<i64>().map_err(|e| e.to_string())?);
        lt.push(f[2].parse::<i64>().map_err(|e| e.to_string())?);
    }
    Ok((lt, bw))
}

pub fn run(spec_in: &RunSpec, run_id: usize, seed: u64, series_every: u32, series_path: Option<&str>, history: Option<&(Vec<i64>, Vec<i64>)>) -> Result<RunOutput, String> {
    let mut spec_owned = spec_in.clone();
    if let Some(s) = spec_owned.spam.as_mut() {
        if s.weight == 0 {
            s.weight = spec_in.tx.first().map(|t| t.weight).unwrap_or(build_config(spec_in, &spec_in.rules, &spec_in.params)?.t_sim);
        }
    }
    let spec = &spec_owned;
    let c0 = build_config(spec, &spec.rules, &spec.params)?;
    let n = spec.blocks;

    // Transaction classes: union of the user mix, the post-fork mix and the spammer's weight.
    let mut tx_before = spec.tx.clone();
    if tx_before.is_empty() {
        tx_before.push(TxClass { weight: c0.t_sim, share: 1.0 });
    }
    let tx_after = match &spec.fork {
        Some(f) if !f.tx_after.is_empty() => f.tx_after.clone(),
        _ => tx_before.clone(),
    };
    let mut classes: Vec<i64> = Vec::new();
    for t in tx_before.iter().chain(tx_after.iter()) {
        if !classes.contains(&t.weight) {
            classes.push(t.weight);
        }
    }
    if let Some(s) = &spec.spam {
        if !classes.contains(&s.weight) {
            classes.push(s.weight);
        }
    }
    let shares_of = |mix: &[TxClass]| -> (Vec<f64>, f64) {
        let tot: f64 = mix.iter().map(|t| t.share).sum();
        let mut sh = vec![0.0; classes.len()];
        for t in mix {
            let j = classes.iter().position(|&w| w == t.weight).unwrap();
            sh[j] += t.share / tot;
        }
        let avg: f64 = sh.iter().zip(&classes).map(|(s, &w)| s * w as f64).sum();
        (sh, avg)
    };
    let (mut shares, mut avg_w) = shares_of(&tx_before);

    let mut sim = match (spec.init.as_str(), history) {
        ("steady", _) => Sim::new(c0, &classes),
        ("mainnet", Some((lt, bw))) => Sim::with_history(c0, &classes, lt, bw),
        ("mainnet", None) => return Err("init=mainnet but no history file".into()),
        (o, _) => return Err(format!("unknown init {o}")),
    };

    sim.pool_lifetime = spec.pool_lifetime;
    let mut expired = [0i64; 2];
    let mut rng_demand = StudyRng::new(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x1234_5678);
    let mut rng_miner = StudyRng::new(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ 0xABCD_EF01);
    let mut carry = vec![0.0f64; classes.len()];
    let mut spam_carry = 0.0f64;

    // windows for event metrics
    let d = &spec.demand;
    let event_window: Option<Window> = if d.surge_blocks > 0 && d.surge_mult != 1.0 {
        Some(Window { start: d.surge_start, end: d.surge_start + d.surge_blocks })
    } else if let Some(s) = &spec.spam {
        Some(Window { start: s.start, end: if s.blocks == 0 { n } else { s.start + s.blocks } })
    } else if let Some(m) = &spec.miner_attack {
        Some(Window { start: m.start, end: if m.blocks == 0 { n } else { m.start + m.blocks } })
    } else if d.fill_mult > 0.0 && d.fill_until.is_some() {
        Some(Window { start: d.fill_from, end: d.fill_until.unwrap() })
    } else if let Some(s) = d.stop_at {
        Some(Window { start: 0, end: s })
    } else {
        None
    };

    // series output
    let mut series = match (series_path, spec.series && series_every > 0) {
        (Some(p), true) => {
            let f = std::fs::File::create(p).map_err(|e| format!("{p}: {e}"))?;
            let mut g = GzEncoder::new(std::io::BufWriter::new(f), Compression::default());
            writeln!(g, "block,m_l,m_s,m_n,max_weight,mb_mean,mb_max,penalty_sum,fees_sum,user_fees_sum,spam_fees_sum,pool_user_bytes,pool_spam_bytes,fee_low,fee_l1,fee_l2,fee_l3,fee_l4,f_min,base_level,cum_spam_xmr,cum_miner_penalty,cum_chain_bytes,user_incl_txs,user_wait_mean,blocks_over_mn,cum_spam_bytes,cum_padding_bytes,cum_user_bytes").unwrap();
            Some(g)
        }
        _ => None,
    };

    let mut events: Vec<String> = Vec::new();
    let mut hist_all = Hist::new();
    let mut hist_win = Hist::new();
    let mut win_fees = 0.0f64;
    let mut win_bytes: i64 = 0;
    let mut win_txs: i64 = 0;
    let mut win_level_counts = vec![0i64; 5];

    // accumulators
    let mut chain_bytes: i64 = 0;
    let (mut user_bytes, mut spam_bytes, mut pad_bytes) = (0i64, 0i64, 0i64);
    let (mut user_fees, mut spam_spent, mut spam_spent_in_win) = (0.0f64, 0.0f64, 0.0f64);
    let (mut user_txs, mut spam_txs) = (0i64, 0i64);
    let (mut miner_blocks, mut miner_pen_total, mut miner_pen_extra) = (0u32, 0.0f64, 0.0f64);
    let mut total_penalty = 0.0f64;
    let mut honest_penalty = 0.0f64;
    let mut total_fees = 0.0f64;
    let mut rejected = [0i64; 2];
    let (mut max_ml, mut max_mn, mut max_block, mut max_maxw) = (0i64, 0i64, 0i64, 0i64);
    let mut max_pool_user: i64 = 0;
    let mut t_maxw = [None::<u32>; 5];
    let mut t_block = [None::<u32>; 5];
    let mut t_ml_mult = [None::<u32>; 3]; // 2x, 4x, 10x the initial M_L
    let ml_mults = [2i64, 4, 10];
    let mut ml0: i64 = 0;
    let mut cost_at_ml = [(f64::NAN, f64::NAN, 0i64); 3];
    // blocks over M_N (honest only): count, max ratio, sum ratio, fees in them
    let (mut over_n, mut over_max_ratio, mut over_sum_ratio, mut over_fees, mut over_pen) = (0u64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let mut spam_blocks_at_mn: u32 = 0;
    let mut win_blocks: u32 = 0;
    let mut win_full_blocks: u32 = 0;
    // event-window state
    let mut base_ml = 0i64;
    let mut base_mn = 0i64;
    let mut base_fee_low = 0.0f64;
    let mut base_pool = 0i64;
    let mut pool_hist: std::collections::VecDeque<i64> = std::collections::VecDeque::new();
    let mut end_vals: Option<(i64, i64, f64, i64)> = None; // ml, mn, fee_low, max weight at window end
    let mut peak_pool_win: i64 = 0;
    let mut clear_block: Option<u32> = None;
    let (mut rec_ml, mut rec_mn, mut rec_fee) = (None::<u32>, None::<u32>, None::<u32>);
    let mut after_snap: Vec<(u32, i64, i64, f64, i64)> = Vec::new(); // months after end
    let snap_months = [1u32, 3, 6, 12];
    let mut ml_at_end = 0i64;
    let mut t_win_ml = [None::<u32>; 2]; // 2x and 4x the M_L at window start, blocks after start
    let mut fork_done = false;
    let mut spam_budget_hit: Option<u32> = None;
    let mut last_ctx: Option<BlockContext> = None;

    // series interval accumulators
    let mut s_mb_sum = 0f64;
    let mut s_mb_max = 0i64;
    let mut s_pen = 0f64;
    let mut s_fees = 0f64;
    let mut s_ufees = 0f64;
    let mut s_sfees = 0f64;
    let mut s_incl = 0i64;
    let mut s_wait = 0f64;
    let mut s_over = 0u32;
    let mut s_k = 0u32;

    let ev = |events: &mut Vec<String>, i: u32, name: &str, ctx: &BlockContext, m_b: i64, pool: [i64; 2], cs: f64, cm: f64| {
        events.push(format!(
            "{run_id},{i},{name},{},{},{},{},{m_b},{:e},{:e},{},{},{cs},{cm}",
            ctx.m_l, ctx.m_s, ctx.m_n, ctx.m_b_max, ctx.levels[0], ctx.f_min, pool[0], pool[1]
        ));
    };

    for i in 0..n {
        // rule switch
        if let Some(f) = &spec.fork {
            if !fork_done && i == f.at {
                let nc = build_config(spec, &f.to, &f.params)?;
                let old_zm = sim.c.z_m;
                let rescale = f.rescale.then_some((nc.z_m, old_zm));
                sim.switch_rules(nc, rescale);
                let (s2, a2) = shares_of(&tx_after);
                shares = s2;
                avg_w = a2;
                fork_done = true;
            }
        }
        let ctx = sim.begin_block();
        if i == 0 {
            ml0 = ctx.m_l;
        }

        // ---- user demand ----
        let mut bytes = if let Some(t) = d.txs_per_block {
            t * avg_w
        } else if let Some(b) = d.bytes_per_block {
            b
        } else if let Some(z) = d.zm_mult {
            z * sim.c.z_m as f64
        } else {
            0.0
        };
        if d.surge_blocks > 0 && i >= d.surge_start && i < d.surge_start + d.surge_blocks {
            bytes *= d.surge_mult;
        }
        if d.daily_amplitude != 0.0 {
            bytes *= 1.0 + d.daily_amplitude * (2.0 * std::f64::consts::PI * i as f64 / d.daily_period.max(1) as f64).sin();
        }
        if d.growth_per_year != 0.0 {
            bytes *= (1.0 + d.growth_per_year).powf(i as f64 / 262_800.0);
        }
        if d.fill_mult > 0.0 && i >= d.fill_from && d.fill_until.map_or(true, |u| i < u) {
            bytes = d.fill_mult * ctx.m_b_max as f64;
        }
        if let Some(s) = d.stop_at {
            if i >= s {
                bytes = d.after_stop_mult
                    * if let Some(t) = d.txs_per_block {
                        t * avg_w
                    } else if let Some(b) = d.bytes_per_block {
                        b
                    } else if let Some(z) = d.zm_mult {
                        z * sim.c.z_m as f64
                    } else {
                        0.0
                    };
            }
        }
        let expected = bytes.max(0.0) / avg_w;
        let user: Vec<i64> = (0..classes.len())
            .map(|j| {
                let e = expected * shares[j];
                if d.poisson {
                    rng_demand.poisson(e)
                } else {
                    carry[j] += e;
                    let k = carry[j].floor();
                    carry[j] -= k;
                    k as i64
                }
            })
            .collect();

        // ---- spammer ----
        let mut spam_sub: Vec<(usize, usize, i64)> = Vec::new();
        if let Some(s) = &spec.spam {
            let active = i >= s.start && (s.blocks == 0 || i < s.start + s.blocks);
            let budget_ok = s.budget_xmr.map_or(true, |b| spam_spent < b);
            if active && !budget_ok && spam_budget_hit.is_none() {
                spam_budget_hit = Some(i);
            }
            if active && budget_ok {
                let j = classes.iter().position(|&w| w == s.weight).unwrap();
                let level = match &s.level {
                    Value::String(x) if x == "auto" => sim.auto_level(&ctx),
                    Value::String(x) if x == "top" => ctx.levels.len() - 1,
                    Value::Number(k) => (k.as_u64().unwrap_or(0) as usize).min(ctx.levels.len() - 1),
                    o => return Err(format!("bad spam level {o}")),
                };
                let cnt = if s.mode == "topup" {
                    let pending = sim.pool_bytes()[OWNER_SPAM];
                    let target = (s.target_mn * ctx.m_n as f64) as i64;
                    if target > pending { (target - pending + s.weight - 1) / s.weight } else { 0 }
                } else if s.mode == "fill_to" {
                    let pending: i64 = sim.pool_bytes().iter().sum();
                    let new_user: i64 = user.iter().zip(sim.classes()).map(|(c, w)| c * w).sum();
                    let mut target = (s.target_mn * ctx.m_n as f64).max(s.target_ml * ctx.m_l as f64) as i64;
                    if s.target_clamp {
                        target = target.max(ctx.m_l * sim.c.lt_growth_num / sim.c.lt_growth_den);
                    }
                    if target > pending + new_user { (target - pending - new_user) / s.weight } else { 0 }
                } else if s.mode == "rate" {
                    spam_carry += s.bytes_per_block / s.weight as f64;
                    let k = spam_carry.floor();
                    spam_carry -= k;
                    k as i64
                } else {
                    return Err(format!("bad spam mode {}", s.mode));
                };
                if cnt > 0 {
                    spam_sub.push((j, level, cnt));
                }
            }
        }

        // ---- miner ----
        let mut miner = Miner::Honest;
        let mut is_attacker = false;
        if let Some(m) = &spec.miner_attack {
            let active = i >= m.start && (m.blocks == 0 || i < m.start + m.blocks);
            let draw = rng_miner.next_f64();
            if active && draw < m.share {
                let mut target: i64 = 0;
                if let Some(x) = m.target_mn {
                    target = target.max((x * ctx.m_n as f64) as i64);
                }
                if let Some(x) = m.target_ml {
                    target = target.max((x * ctx.m_l as f64) as i64);
                }
                if let Some(x) = m.target_bytes {
                    target = target.max(x);
                }
                if m.target_clamp {
                    target = target.max(ctx.m_l * sim.c.lt_growth_num / sim.c.lt_growth_den);
                }
                if m.target_max {
                    target = ctx.m_b_max;
                }
                miner = Miner::Pad { target };
                is_attacker = true;
            }
        }

        let o = sim.finish_block(&user, &spam_sub, miner);

        // ---- metrics ----
        chain_bytes += o.m_b;
        user_bytes += o.bytes_by_owner[OWNER_USER];
        spam_bytes += o.bytes_by_owner[OWNER_SPAM];
        pad_bytes += o.padding;
        user_fees += o.fees_by_owner[OWNER_USER];
        spam_spent += o.fees_by_owner[OWNER_SPAM];
        user_txs += o.txs_by_owner[OWNER_USER];
        spam_txs += o.txs_by_owner[OWNER_SPAM];
        total_fees += o.fees;
        total_penalty += o.penalty;
        rejected[0] += o.rejected[0];
        rejected[1] += o.rejected[1];
        expired[0] += o.expired[0];
        expired[1] += o.expired[1];
        if is_attacker {
            miner_blocks += 1;
            miner_pen_total += o.penalty;
            miner_pen_extra += o.penalty - o.penalty_unpadded;
        } else {
            honest_penalty += o.penalty;
            if o.m_b > ctx.m_n {
                over_n += 1;
                let r = o.m_b as f64 / ctx.m_n as f64;
                over_max_ratio = over_max_ratio.max(r);
                over_sum_ratio += r;
                over_fees += o.fees;
                over_pen += o.penalty;
            }
        }
        max_ml = max_ml.max(ctx.m_l);
        max_mn = max_mn.max(ctx.m_n);
        max_block = max_block.max(o.m_b);
        max_maxw = max_maxw.max(ctx.m_b_max);
        max_pool_user = max_pool_user.max(o.pool_bytes[OWNER_USER]);
        for (k, &mb) in SIZE_THRESHOLDS.iter().enumerate() {
            if t_maxw[k].is_none() && ctx.m_b_max >= mb * MB {
                t_maxw[k] = Some(i);
                ev(&mut events, i, &format!("max_weight_{mb}MB"), &ctx, o.m_b, o.pool_bytes, spam_spent, miner_pen_extra);
            }
            if t_block[k].is_none() && o.m_b >= mb * MB {
                t_block[k] = Some(i);
            }
        }
        for (k, &m) in ml_mults.iter().enumerate() {
            if t_ml_mult[k].is_none() && ctx.m_l >= m * ml0 {
                t_ml_mult[k] = Some(i);
                cost_at_ml[k] = (spam_spent, miner_pen_extra, chain_bytes);
                ev(&mut events, i, &format!("m_l_{m}x"), &ctx, o.m_b, o.pool_bytes, spam_spent, miner_pen_extra);
            }
        }
        for inc in &sim.inclusions {
            if inc.owner as usize == OWNER_USER {
                hist_all.add(inc.wait, inc.count);
                s_incl += inc.count;
                s_wait += inc.wait as f64 * inc.count as f64;
                let arrival = i - inc.wait;
                if let Some(w) = &event_window {
                    if arrival >= w.start && arrival < w.end {
                        hist_win.add(inc.wait, inc.count);
                        win_fees += inc.fee_per_tx * inc.count as f64;
                        win_bytes += sim.classes()[inc.class as usize] * inc.count;
                        win_txs += inc.count;
                        let l = (inc.level as usize).min(4);
                        win_level_counts[l] += inc.count;
                    }
                }
            }
        }
        if o.m_b > ctx.m_n {
            s_over += 1;
        }

        // event window bookkeeping
        pool_hist.push_back(o.pool_bytes[OWNER_USER]);
        if pool_hist.len() > 100 {
            pool_hist.pop_front();
        }
        if let Some(w) = &event_window {
            if i + 1 == w.start || (w.start == 0 && i == 0) {
                base_ml = ctx.m_l;
                base_mn = ctx.m_n;
                base_fee_low = ctx.levels[0];
                base_pool = if w.start == 0 { 0 } else { pool_hist.iter().copied().max().unwrap_or(0) };
            }
            if i == w.start {
                ev(&mut events, i, "window_start", &ctx, o.m_b, o.pool_bytes, spam_spent, miner_pen_extra);
            }
            if i >= w.start && i < w.end {
                win_blocks += 1;
                if o.m_b + sim.classes().iter().max().unwrap() > ctx.m_n {
                    win_full_blocks += 1;
                }
                if o.bytes_by_owner[OWNER_SPAM] > 0 || o.padding > 0 {
                    spam_spent_in_win = spam_spent;
                }
                if o.m_b + sim.classes().iter().max().unwrap() > ctx.m_n && spec.spam.is_some() {
                    spam_blocks_at_mn += 1;
                }
                for (k, &m) in [2i64, 4].iter().enumerate() {
                    if t_win_ml[k].is_none() && ctx.m_l >= m * base_ml.max(1) {
                        t_win_ml[k] = Some(i - w.start);
                    }
                }
            }
            if i >= w.start {
                peak_pool_win = peak_pool_win.max(o.pool_bytes[OWNER_USER]);
            }
            if i + 1 == w.end {
                end_vals = Some((ctx.m_l, ctx.m_n, ctx.levels[0], ctx.m_b_max));
                ml_at_end = ctx.m_l;
                ev(&mut events, i, "window_end", &ctx, o.m_b, o.pool_bytes, spam_spent, miner_pen_extra);
            }
            if i >= w.end {
                let tol = 2 * sim.classes().iter().max().unwrap();
                if clear_block.is_none() && o.pool_bytes[OWNER_USER] <= base_pool + tol {
                    clear_block = Some(i);
                    ev(&mut events, i, "backlog_cleared", &ctx, o.m_b, o.pool_bytes, spam_spent, miner_pen_extra);
                }
                if rec_ml.is_none() && (ctx.m_l as f64) <= 1.1 * base_ml as f64 {
                    rec_ml = Some(i - w.end);
                }
                if rec_mn.is_none() && (ctx.m_n as f64) <= 1.1 * base_mn as f64 {
                    rec_mn = Some(i - w.end);
                }
                if rec_fee.is_none() && ctx.levels[0] >= 0.9 * base_fee_low {
                    rec_fee = Some(i - w.end);
                }
                for &mo in &snap_months {
                    if i == w.end + mo * MONTH {
                        after_snap.push((mo, ctx.m_l, ctx.m_n, ctx.levels[0], ctx.m_b_max));
                        ev(&mut events, i, &format!("end_plus_{mo}mo"), &ctx, o.m_b, o.pool_bytes, spam_spent, miner_pen_extra);
                    }
                }
            }
        }
        if let Some(f) = &spec.fork {
            if i == f.at || i + 1 == f.at {
                ev(&mut events, i, if i == f.at { "fork_first_block" } else { "fork_last_pre" }, &ctx, o.m_b, o.pool_bytes, spam_spent, miner_pen_extra);
            }
            for off in [100u32, 1000, 10_000, 50_000, 100_000] {
                if i == f.at + off {
                    ev(&mut events, i, &format!("fork_plus_{off}"), &ctx, o.m_b, o.pool_bytes, spam_spent, miner_pen_extra);
                }
            }
        }
        if spec.snapshots.contains(&i) {
            ev(&mut events, i, "snapshot", &ctx, o.m_b, o.pool_bytes, spam_spent, miner_pen_extra);
        }

        // series
        s_mb_sum += o.m_b as f64;
        s_mb_max = s_mb_max.max(o.m_b);
        s_pen += o.penalty;
        s_fees += o.fees;
        s_ufees += o.fees_by_owner[OWNER_USER];
        s_sfees += o.fees_by_owner[OWNER_SPAM];
        s_k += 1;
        if let Some(g) = series.as_mut() {
            if (i + 1) % series_every == 0 || i + 1 == n {
                let lv = |k: usize| ctx.levels.get(k).map(|x| format!("{x:e}")).unwrap_or_default();
                writeln!(
                    g,
                    "{},{},{},{},{},{:.0},{},{:.6e},{:.6e},{:.6e},{:.6e},{},{},{},{},{},{},{},{:e},{},{:.9},{:.9},{},{},{},{},{},{},{}",
                    i, ctx.m_l, ctx.m_s, ctx.m_n, ctx.m_b_max, s_mb_sum / s_k as f64, s_mb_max, s_pen, s_fees, s_ufees, s_sfees,
                    o.pool_bytes[0], o.pool_bytes[1], lv(0), lv(1), lv(2), lv(3), lv(4), ctx.f_min, o.base_level,
                    spam_spent, miner_pen_extra, chain_bytes, s_incl,
                    if s_incl > 0 { format!("{:.2}", s_wait / s_incl as f64) } else { String::new() }, s_over,
                    spam_bytes, pad_bytes, user_bytes
                )
                .map_err(|e| e.to_string())?;
                s_mb_sum = 0.0;
                s_mb_max = 0;
                s_pen = 0.0;
                s_fees = 0.0;
                s_ufees = 0.0;
                s_sfees = 0.0;
                s_incl = 0;
                s_wait = 0.0;
                s_over = 0;
                s_k = 0;
            }
        }
        if i + 1 == n {
            ev(&mut events, i, "end", &ctx, o.m_b, o.pool_bytes, spam_spent, miner_pen_extra);
            last_ctx = Some(ctx);
        }
    }
    if let Some(g) = series {
        g.finish().map_err(|e| e.to_string())?.flush().map_err(|e| e.to_string())?;
    }

    // ---- summary ----
    let ctx = last_ctx.unwrap();
    let mut r = Row::default();
    r.put("final_m_l", ctx.m_l);
    r.put("final_m_s", ctx.m_s);
    r.put("final_m_n", ctx.m_n);
    r.put("final_max_weight", ctx.m_b_max);
    r.put("final_fee_low", format!("{:e}", ctx.levels[0]));
    r.put("final_fee_levels", ctx.levels.iter().map(|x| format!("{x:e}")).collect::<Vec<_>>().join(" "));
    r.put("initial_m_l", ml0);
    r.put("max_m_l", max_ml);
    r.put("max_m_n", max_mn);
    r.put("max_max_weight", max_maxw);
    r.put("max_block", max_block);
    for (k, &mb) in SIZE_THRESHOLDS.iter().enumerate() {
        r.opt(&format!("t_maxw_{mb}MB"), t_maxw[k]);
    }
    for (k, &mb) in SIZE_THRESHOLDS.iter().enumerate() {
        r.opt(&format!("t_block_{mb}MB"), t_block[k]);
    }
    for (k, &m) in ml_mults.iter().enumerate() {
        r.opt(&format!("t_m_l_{m}x"), t_ml_mult[k]);
    }
    for (k, &m) in ml_mults.iter().enumerate() {
        r.opt(&format!("spam_xmr_at_m_l_{m}x"), t_ml_mult[k].map(|_| format!("{:.6}", cost_at_ml[k].0)));
        r.opt(&format!("miner_xmr_at_m_l_{m}x"), t_ml_mult[k].map(|_| format!("{:.6}", cost_at_ml[k].1)));
        r.opt(&format!("chain_bytes_at_m_l_{m}x"), t_ml_mult[k].map(|_| cost_at_ml[k].2));
    }
    r.put("chain_bytes", chain_bytes);
    r.put("user_bytes", user_bytes);
    r.put("spam_bytes", spam_bytes);
    r.put("padding_bytes", pad_bytes);
    r.put("user_txs", user_txs);
    r.put("user_fees_xmr", format!("{user_fees:.9}"));
    r.opt("user_fee_per_tx", (user_txs > 0).then(|| format!("{:e}", user_fees / user_txs as f64)));
    r.opt("user_fee_per_byte", (user_bytes > 0).then(|| format!("{:e}", user_fees / user_bytes as f64)));
    r.opt("wait_mean", hist_all.mean().map(|x| format!("{x:.3}")));
    r.opt("wait_p50", hist_all.pct(0.5));
    r.opt("wait_p90", hist_all.pct(0.9));
    r.opt("wait_p99", hist_all.pct(0.99));
    r.put("wait_max", hist_all.max);
    let pc = sim.pool_count();
    r.put("pending_user_txs_end", pc[OWNER_USER]);
    r.put("pending_user_bytes_end", sim.pool_bytes()[OWNER_USER]);
    r.opt("oldest_pending_age_end", sim.oldest_pending(OWNER_USER).map(|a| n - a));
    r.put("max_pool_user_bytes", max_pool_user);
    r.put("rejected_user_txs", rejected[0]);
    r.put("rejected_spam_txs", rejected[1]);
    r.put("expired_user_txs", expired[0]);
    r.put("expired_spam_txs", expired[1]);
    r.put("total_fees_xmr", format!("{total_fees:.9}"));
    r.put("total_penalty_xmr", format!("{total_penalty:.9}"));
    r.put("honest_penalty_xmr", format!("{honest_penalty:.9}"));
    r.put("blocks_over_mn", over_n);
    r.opt("over_mn_mean_ratio", (over_n > 0).then(|| format!("{:.6}", over_sum_ratio / over_n as f64)));
    r.put("over_mn_max_ratio", format!("{over_max_ratio:.6}"));
    r.put("over_mn_fees_xmr", format!("{over_fees:.9}"));
    r.put("over_mn_penalty_xmr", format!("{over_pen:.9}"));
    // attackers
    r.put("spam_spent_xmr", format!("{spam_spent:.9}"));
    r.put("spam_spent_in_window_xmr", format!("{spam_spent_in_win:.9}"));
    r.put("spam_bytes", spam_bytes);
    r.put("spam_txs", spam_txs);
    r.opt("spam_xmr_per_gb", (spam_bytes > 0).then(|| format!("{:.6}", spam_spent / spam_bytes as f64 * 1e9)));
    r.put("spam_blocks_full", spam_blocks_at_mn);
    r.opt("spam_budget_hit_block", spam_budget_hit);
    r.put("spam_pending_bytes_end", sim.pool_bytes()[OWNER_SPAM]);
    r.put("miner_blocks", miner_blocks);
    r.put("miner_penalty_total_xmr", format!("{miner_pen_total:.9}"));
    r.put("miner_penalty_extra_xmr", format!("{miner_pen_extra:.9}"));
    r.opt("miner_xmr_per_gb_padding", (pad_bytes > 0).then(|| format!("{:.6}", miner_pen_extra / pad_bytes as f64 * 1e9)));
    // event window
    let w = event_window.as_ref();
    r.opt("win_start", w.map(|w| w.start));
    r.opt("win_end", w.map(|w| w.end));
    r.put("win_blocks", win_blocks);
    r.put("win_full_blocks", win_full_blocks);
    r.put("win_base_m_l", base_ml);
    r.put("win_base_m_n", base_mn);
    r.put("win_base_fee_low", format!("{base_fee_low:e}"));
    r.put("win_base_pool", base_pool);
    r.opt("win_end_m_l", end_vals.map(|v| v.0));
    r.opt("win_end_m_n", end_vals.map(|v| v.1));
    r.opt("win_end_fee_low", end_vals.map(|v| format!("{:e}", v.2)));
    r.opt("win_end_max_weight", end_vals.map(|v| v.3));
    r.opt("win_m_l_ratio", (base_ml > 0 && ml_at_end > 0).then(|| format!("{:.6}", ml_at_end as f64 / base_ml as f64)));
    r.opt("win_t_m_l_2x", t_win_ml[0]);
    r.opt("win_t_m_l_4x", t_win_ml[1]);
    r.put("win_peak_pool_user_bytes", peak_pool_win);
    r.opt("win_clear_block", clear_block);
    r.opt("win_clear_after_end", clear_block.zip(w).map(|(c, w)| c.saturating_sub(w.end)));
    r.opt("win_rec_m_l", rec_ml);
    r.opt("win_rec_m_n", rec_mn);
    r.opt("win_rec_fee_low", rec_fee);
    for &mo in &snap_months {
        let s = after_snap.iter().find(|s| s.0 == mo);
        r.opt(&format!("win_plus{mo}mo_m_l"), s.map(|s| s.1));
        r.opt(&format!("win_plus{mo}mo_m_n"), s.map(|s| s.2));
        r.opt(&format!("win_plus{mo}mo_fee_low"), s.map(|s| format!("{:e}", s.3)));
    }
    r.put("win_txs_included", win_txs);
    r.opt("win_wait_mean", hist_win.mean().map(|x| format!("{x:.3}")));
    r.opt("win_wait_p50", hist_win.pct(0.5));
    r.opt("win_wait_p90", hist_win.pct(0.9));
    r.opt("win_wait_p99", hist_win.pct(0.99));
    r.put("win_wait_max", hist_win.max);
    r.opt("win_share_within_10", hist_win.share_within(10).map(|x| format!("{x:.6}")));
    r.opt("win_share_within_720", hist_win.share_within(720).map(|x| format!("{x:.6}")));
    r.opt("win_share_within_2160", hist_win.share_within(2160).map(|x| format!("{x:.6}")));
    r.opt("win_fee_per_tx", (win_txs > 0).then(|| format!("{:e}", win_fees / win_txs as f64)));
    r.opt("win_fee_per_byte", (win_bytes > 0).then(|| format!("{:e}", win_fees / win_bytes as f64)));
    r.put("win_level_counts", win_level_counts.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "));
    Ok(RunOutput { summary: r, events })
}
