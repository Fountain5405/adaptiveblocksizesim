//! Scenario files and the run specification they expand into.
//!
//! A scenario file is JSON:
//! ```json
//! { "name": "q2_surge", "seeds": 4, "seed_base": 1000, "series_every": 100,
//!   "base":  { ...RunSpec fields... },
//!   "cases": [ { "label": "v16", "rules": "v16" }, { "label": "v17", "rules": "v17" } ],
//!   "grid":  { "demand.surge_mult": [2, 5, 10], "demand.surge_blocks": [30, 720] } }
//! ```
//! Runs are `cases x grid x seeds`. Each case is deep-merged into `base`, then each grid
//! combination is set by dotted path. Grid keys and case labels become summary columns.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Deserialize)]
pub struct Scenario {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "one")]
    pub seeds: u32,
    #[serde(default = "seed_base")]
    pub seed_base: u64,
    #[serde(default = "series_every")]
    pub series_every: u32,
    /// Write series only for the first `series_seeds` seeds of each configuration (default: all).
    #[serde(default)]
    pub series_seeds: Option<u32>,
    #[serde(default)]
    pub base: Value,
    #[serde(default)]
    pub cases: Vec<Value>,
    #[serde(default)]
    pub grid: Map<String, Value>,
}

fn one() -> u32 {
    1
}
fn seed_base() -> u64 {
    1
}
fn series_every() -> u32 {
    100
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct RunSpec {
    pub label: String,
    /// "v16", "v17" or "custom" (2026 fee algorithm, v17 defaults, any parameters)
    pub rules: String,
    pub params: Params,
    pub blocks: u32,
    /// Transaction classes. Empty: one class of the preset's t_sim.
    pub tx: Vec<TxClass>,
    /// "auto" (wallet2 default priority) or a fixed level index.
    pub fee_level: Value,
    pub users_pay_more: bool,
    pub rational_miners: bool,
    pub demand: Demand,
    pub spam: Option<Spam>,
    pub miner_attack: Option<MinerAttack>,
    pub fork: Option<Fork>,
    /// "steady" (windows filled with initial_weight) or "mainnet" (study/data/mainnet_weights.csv.gz)
    pub init: String,
    /// Extra blocks at which exact values are recorded as events.
    pub snapshots: Vec<u32>,
    /// Write the downsampled series for this run.
    pub series: bool,
    /// Drop pool transactions after this many blocks (monerod: 3 days = 2160). Default: never.
    pub pool_lifetime: Option<u32>,
}

impl Default for RunSpec {
    fn default() -> Self {
        RunSpec {
            label: String::new(),
            rules: "v17".into(),
            params: Params::default(),
            blocks: 100_000,
            tx: Vec::new(),
            fee_level: Value::String("auto".into()),
            users_pay_more: false,
            rational_miners: true,
            demand: Demand::default(),
            spam: None,
            miner_attack: None,
            fork: None,
            init: "steady".into(),
            snapshots: Vec::new(),
            series: true,
            pool_lifetime: None,
        }
    }
}

/// Overrides of the preset's parameters.
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Params {
    pub z_m: Option<i64>,
    /// Long-term clamp ratio as a decimal (1.2 -> 120/100) ...
    pub lt_growth: Option<f64>,
    /// ... or as an exact fraction [num, den].
    pub lt_growth_frac: Option<[i64; 2]>,
    pub surge_factor: Option<i64>,
    pub t_r: Option<i64>,
    pub wallet_grace_blocks: Option<u32>,
    pub min_fee_fraction: Option<f64>,
    /// Fills the median windows at block 0 (default: Z_M).
    pub initial_weight: Option<i64>,
    pub long_window: Option<u32>,
    pub short_window: Option<u32>,
    pub r_base: Option<f64>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TxClass {
    pub weight: i64,
    #[serde(default = "one_f")]
    pub share: f64,
}
fn one_f() -> f64 {
    1.0
}

/// User demand. The base rate is `txs_per_block`, `bytes_per_block` or `zm_mult` x Z_M bytes;
/// modifiers multiply it. `fill_mult` > 0 replaces it with fill_mult x the maximum block weight
/// (demand permanently above capacity) until `fill_until`.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct Demand {
    pub txs_per_block: Option<f64>,
    pub bytes_per_block: Option<f64>,
    pub zm_mult: Option<f64>,
    pub poisson: bool,
    // step surge
    pub surge_start: u32,
    pub surge_blocks: u32,
    pub surge_mult: f64,
    // daily cycle: x (1 + amplitude sin(2 pi i / period))
    pub daily_amplitude: f64,
    pub daily_period: u32,
    // steady exponential growth, fraction per year (0.5 = +50%/yr)
    pub growth_per_year: f64,
    // demand above capacity
    pub fill_mult: f64,
    pub fill_from: u32,
    pub fill_until: Option<u32>,
    // after this block, demand is after_stop_mult x the base (0 = nothing)
    pub stop_at: Option<u32>,
    pub after_stop_mult: f64,
}

impl Default for Demand {
    fn default() -> Self {
        Demand {
            txs_per_block: None,
            bytes_per_block: None,
            zm_mult: None,
            poisson: false,
            surge_start: 0,
            surge_blocks: 0,
            surge_mult: 1.0,
            daily_amplitude: 0.0,
            daily_period: 720,
            growth_per_year: 0.0,
            fill_mult: 0.0,
            fill_from: 0,
            fill_until: None,
            stop_at: None,
            after_stop_mult: 0.0,
        }
    }
}

/// A fee-paying spammer.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct Spam {
    pub start: u32,
    /// 0: until the end of the run
    pub blocks: u32,
    /// "auto", "top" (highest wallet level) or a level index
    pub level: Value,
    /// Transaction weight; 0 = the weight of the first user transaction class.
    pub weight: i64,
    /// "topup": keep target_mn x M_N bytes of spam pending each block (builds a backlog).
    /// "fill_to": submit just enough that pending spam + pending and new user transactions reach
    /// max(target_mn x M_N, target_ml x M_L) bytes (no backlog beyond the target).
    /// "rate": bytes_per_block.
    pub mode: String,
    pub target_mn: f64,
    pub target_ml: f64,
    /// fill_to target is (at least) the long-term clamp ratio x M_L of the rules in force.
    pub target_clamp: bool,
    pub bytes_per_block: f64,
    pub budget_xmr: Option<f64>,
}

impl Default for Spam {
    fn default() -> Self {
        Spam {
            start: 0,
            blocks: 0,
            level: Value::from(0),
            weight: 0,
            mode: "topup".into(),
            target_mn: 1.0,
            target_ml: 0.0,
            target_clamp: false,
            bytes_per_block: 0.0,
            budget_xmr: None,
        }
    }
}

/// A miner with hashrate share `share` that pads its blocks with zero-fee filler.
/// Target = the largest of the given targets, capped at the maximum block weight.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct MinerAttack {
    pub share: f64,
    pub start: u32,
    pub blocks: u32,
    pub target_mn: Option<f64>,
    pub target_ml: Option<f64>,
    pub target_bytes: Option<i64>,
    pub target_max: bool,
    /// Target (at least) the long-term clamp ratio x M_L of the rules in force.
    pub target_clamp: bool,
}

impl Default for MinerAttack {
    fn default() -> Self {
        MinerAttack { share: 0.0, start: 0, blocks: 0, target_mn: None, target_ml: None, target_bytes: None, target_max: false, target_clamp: false }
    }
}

/// Switch rules at block `at`.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct Fork {
    pub at: u32,
    pub to: String,
    pub params: Params,
    /// Rescale pre-fork weights by new Z_M / old Z_M (the spec's transitional rule).
    pub rescale: bool,
    /// Transaction mix after the fork (default: unchanged).
    pub tx_after: Vec<TxClass>,
}

impl Default for Fork {
    fn default() -> Self {
        Fork { at: 0, to: "v17".into(), params: Params::default(), rescale: false, tx_after: Vec::new() }
    }
}

/// Deep-merge `patch` into `base`.
pub fn merge(base: &mut Value, patch: &Value) {
    match (base, patch) {
        (Value::Object(b), Value::Object(p)) => {
            for (k, v) in p {
                match b.get_mut(k) {
                    Some(bv) if bv.is_object() && v.is_object() => merge(bv, v),
                    _ => {
                        b.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (b, p) => *b = p.clone(),
    }
}

/// Set a dotted path in a JSON object, creating objects on the way.
pub fn set_path(root: &mut Value, path: &str, v: Value) {
    let mut cur = root;
    let parts: Vec<&str> = path.split('.').collect();
    for (i, p) in parts.iter().enumerate() {
        if !cur.is_object() {
            *cur = Value::Object(Map::new());
        }
        let obj = cur.as_object_mut().unwrap();
        if i + 1 == parts.len() {
            obj.insert(p.to_string(), v);
            return;
        }
        cur = obj.entry(p.to_string()).or_insert(Value::Object(Map::new()));
    }
}

pub struct Expanded {
    pub id: usize,
    pub seed: u64,
    pub seed_index: u32,
    pub case_label: String,
    pub vars: Vec<(String, Value)>,
    pub spec: RunSpec,
    pub json: Value,
}

pub fn expand(s: &Scenario) -> Result<Vec<Expanded>, String> {
    // `cases` is a list of overrides, or a list of such lists that are crossed (labels joined by "/").
    let cases: Vec<Value> = if s.cases.is_empty() {
        vec![Value::Object(Map::new())]
    } else if s.cases[0].is_array() {
        let mut acc: Vec<Value> = vec![Value::Object(Map::new())];
        for dim in &s.cases {
            let dim = dim.as_array().ok_or("cases: mix of lists and objects")?;
            let mut next = Vec::new();
            for a in &acc {
                for d in dim {
                    let mut m = a.clone();
                    let la = a.get("label").and_then(|l| l.as_str()).unwrap_or("").to_string();
                    let ld = d.get("label").and_then(|l| l.as_str()).unwrap_or("").to_string();
                    merge(&mut m, d);
                    let joined = if la.is_empty() { ld } else { format!("{la}/{ld}") };
                    set_path(&mut m, "label", Value::String(joined));
                    next.push(m);
                }
            }
            acc = next;
        }
        acc
    } else {
        s.cases.clone()
    };
    // cartesian product of the grid
    let mut combos: Vec<Vec<(String, Value)>> = vec![vec![]];
    for (k, vals) in &s.grid {
        let vals = vals.as_array().ok_or_else(|| format!("grid.{k} must be an array"))?;
        let mut next = Vec::new();
        for c in &combos {
            for v in vals {
                let mut c2 = c.clone();
                c2.push((k.clone(), v.clone()));
                next.push(c2);
            }
        }
        combos = next;
    }
    let mut out = Vec::new();
    let mut id = 0;
    for case in &cases {
        for combo in &combos {
            for si in 0..s.seeds {
                let mut v = if s.base.is_null() { Value::Object(Map::new()) } else { s.base.clone() };
                merge(&mut v, case);
                for (k, val) in combo {
                    set_path(&mut v, k, val.clone());
                }
                let spec: RunSpec = serde_json::from_value(v.clone()).map_err(|e| format!("run {id}: {e}"))?;
                out.push(Expanded {
                    id,
                    seed: s.seed_base + si as u64,
                    seed_index: si,
                    case_label: case.get("label").and_then(|l| l.as_str()).unwrap_or("").to_string(),
                    vars: combo.clone(),
                    spec,
                    json: v,
                });
                id += 1;
            }
        }
    }
    Ok(out)
}
