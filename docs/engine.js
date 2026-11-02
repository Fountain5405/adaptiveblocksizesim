// JavaScript port of wasm-sim/src/lib_core.rs (run_simulation_core). Keep the two in sync.
// All i64 quantities are plain JS integers (safe below 2^53); i64 division uses Math.trunc.

export const RULES_V16 = 0, RULES_V17 = 1, RULES_CUSTOM = 2;
export const FEE_LEVEL_AUTO = -1;

export function presetConfig(rules) {
  const v16 = rules === RULES_V16;
  return {
    n: 200000,
    rules,
    z_m: v16 ? 300000 : 625000,
    lt_growth_num: v16 ? 17 : 6,
    lt_growth_den: v16 ? 10 : 5,
    surge_factor: v16 ? 50 : 8,
    long_window: 100000,
    short_window: 100,
    initial_weight: v16 ? 300000 : 625000,
    sanity_cap: false,
    sanity_start_weight: 10000000,
    sanity_start_block: 0,
    t_r: v16 ? 3000 : 12500,
    min_fee_fraction: v16 ? 0.95 : 1.0,
    wallet_grace_blocks: v16 ? 10 : 1000,
    fee_rounding_digits: 2,
    fee_level: FEE_LEVEL_AUTO,
    r_base: 0.6,
    t_sim: v16 ? 2000 : 9093,
    run_type: 6,
    ramp_multiplier: 3.0,
    add_noise: false,
    users_pay_more: false,
    rational_miners: true,
  };
}

// Multiset of non-negative integers: heap with lazy deletion plus per-value counts.
// isMax: top() is the maximum (else the minimum). `len` is the exact element count.
class Multiset {
  constructor(isMax) {
    this.s = isMax ? -1 : 1; // heap is a min-heap of s*x
    this.heap = [];
    this.counts = new Map();
    this.len = 0;
  }
  insert(x) {
    this.counts.set(x, (this.counts.get(x) || 0) + 1);
    this.len++;
    const h = this.heap;
    let i = h.length;
    const k = this.s * x;
    h.push(k);
    while (i > 0) {
      const p = (i - 1) >> 1;
      if (h[p] <= k) break;
      h[i] = h[p];
      i = p;
    }
    h[i] = k;
  }
  remove(x) {
    const c = this.counts.get(x);
    if (!c) return false;
    if (c === 1) this.counts.delete(x); else this.counts.set(x, c - 1);
    this.len--;
    return true;
  }
  top() {
    const h = this.heap;
    while (h.length) {
      const k = h[0];
      const x = this.s === 1 ? k : 0 - k;
      if (this.counts.has(x)) return x;
      const last = h.pop();
      if (h.length) {
        let i = 0;
        const n = h.length;
        for (;;) {
          let c = 2 * i + 1;
          if (c >= n) break;
          if (c + 1 < n && h[c + 1] < h[c]) c++;
          if (h[c] >= last) break;
          h[i] = h[c];
          i = c;
        }
        h[i] = last;
      }
    }
    return undefined;
  }
}

// Ring buffer of the latest `size` values, with the `low_target` smallest kept in `low`.
class SlidingOrderStat {
  constructor(size, low_target, fill) {
    this.size = size;
    this.low_target = low_target;
    this.values = new Float64Array(size + 1);
    this.head = 0;
    this.count = 0;
    this.low = new Multiset(true);
    this.high = new Multiset(false);
    for (let i = 0; i < size; i++) this.push(fill);
  }
  static medianOf(size, fill) {
    const low_target = size % 2 === 0 ? size / 2 : (size + 1) / 2;
    return new SlidingOrderStat(size, low_target, fill);
  }
  push(x) {
    const cap = this.size + 1;
    this.values[(this.head + this.count) % cap] = x;
    this.count++;
    const m = this.low.top();
    if (m === undefined || x <= m) this.low.insert(x); else this.high.insert(x);
    if (this.count > this.size) {
      const old = this.values[this.head];
      this.head = (this.head + 1) % cap;
      this.count--;
      const m2 = this.low.top();
      const inLow = m2 !== undefined && old <= m2;
      if (!(inLow && this.low.remove(old))) this.high.remove(old);
    }
    while (this.low.len > this.low_target) {
      const m3 = this.low.top();
      this.low.remove(m3);
      this.high.insert(m3);
    }
    while (this.low.len < this.low_target && this.high.len > 0) {
      const m4 = this.high.top();
      this.high.remove(m4);
      this.low.insert(m4);
    }
  }
  kth() { return this.low.top(); }
  median() {
    if (this.size % 2 === 0) return Math.trunc((this.kth() + this.high.top()) / 2);
    return this.kth();
  }
}

// Median of [0; grace] ++ window, window being the latest size-grace values.
class GraceMedian {
  constructor(size, grace, fill) {
    grace = Math.min(grace, Math.floor(size / 2) - 1);
    const kept = size - grace;
    const low_target = size % 2 === 0 ? size / 2 - grace : (size + 1) / 2 - grace;
    this.stat = new SlidingOrderStat(kept, low_target, fill);
    this.size = size;
  }
  push(x) { this.stat.push(x); }
  median() {
    if (this.size % 2 === 0) return Math.trunc((this.stat.kth() + this.stat.high.top()) / 2);
    return this.stat.kth();
  }
}

// Same algorithm as compiler-rt __powidf2 (what Rust's f64::powi lowers to).
function powi(a, b) {
  const recip = b < 0;
  let r = 1;
  for (;;) {
    if (b & 1) r *= a;
    b = Math.trunc(b / 2);
    if (b === 0) break;
    a *= a;
  }
  return recip ? 1 / r : r;
}

function roundUpSignificant(value, digits) {
  if (value <= 0 || !Number.isFinite(value) || digits === 0) return value;
  const exponent = Math.trunc(Math.floor(Math.log10(value))) - digits + 1;
  const factor = powi(10, exponent);
  const scaled = value / factor;
  const rounded = Math.abs(scaled - Math.round(scaled)) < 1e-9 ? Math.round(scaled) : Math.ceil(scaled);
  return rounded * factor;
}

function walletFeeLevels(c, mlw, msw_raw) {
  const r = c.r_base, tr = c.t_r, digits = c.fee_rounding_digits;
  let f;
  if (c.rules === RULES_V16) {
    const mlwf = Math.max(mlw, c.z_m);
    const msw = Math.max(msw_raw, mlwf);
    const mnw = Math.min(msw, c.surge_factor * mlwf);
    const mfw = Math.min(mnw, mlwf);
    const zm = c.z_m;
    const fl = r * tr / (mfw * mfw);
    const fn = 4.0 * fl;
    const fm = 16.0 * r * tr / (zm * mfw);
    const fh = Math.max(4.0 * fm, 4.0 * fm * mfw / (32.0 * tr * mnw / zm));
    f = [fl, fn, fm, fh];
  } else {
    const mfw = Math.max(mlw, c.z_m);
    const fl = r * tr / (mfw * mfw);
    const fx = 2.0 * r / mfw;
    f = [fl, 4.0 * fl, 16.0 * fl, 64.0 * fl, fx];
  }
  return f.map((v) => roundUpSignificant(v, digits));
}

function minFeePerByte(c, m_l) {
  const ml = Math.max(m_l, c.z_m);
  return c.min_fee_fraction * c.r_base * c.t_r / (ml * ml);
}

function penaltyOf(r, w, m_n) {
  const b = w / m_n - 1.0;
  return b > 0 ? r * b * b : 0.0;
}

function affordableTxs(r, m_b, m_n, t, fee, cap, available) {
  if (available <= 0 || m_b + t > cap) return 0;
  const mn = m_n, tf = t;
  const root = mn * Math.sqrt(fee / r);
  const x_max = root >= tf ? mn + (fee * mn * mn / r - tf * tf) / (2.0 * tf) : mn - tf + root;
  if (x_max < m_b) return 0;
  const by_fee = Math.trunc(Math.floor((x_max - m_b) / tf)) + 1;
  const by_cap = Math.trunc((cap - m_b) / t);
  return Math.max(Math.min(by_fee, by_cap, available), 0);
}

const U64 = (1n << 64n) - 1n;
class Rng {
  constructor(seed) { this.state = BigInt(seed); }
  next() {
    let x = this.state;
    x ^= (x << 13n) & U64;
    x ^= x >> 7n;
    x ^= (x << 17n) & U64;
    this.state = x;
    return x;
  }
  nextF64() { return Number(this.next()) / 18446744073709551616; } // u64::MAX as f64 == 2^64
  normal(mean, sd) {
    const u1 = Math.max(this.nextF64(), 1e-10);
    const u2 = this.nextF64();
    return mean + sd * Math.sqrt(-2.0 * Math.log(u1)) * Math.cos(2.0 * Math.PI * u2);
  }
}

function demandBytes(c, i, m_b_max) {
  switch (c.run_type) {
    case 1: return c.z_m + 100 * i;
    case 2: return c.z_m + 800 * i;
    case 3: return Math.trunc((316.0 + i / 15.0) * (316.0 + i / 15.0));
    case 4: return Math.trunc(c.z_m * (Math.pow(1.6, 9.8 + i / 50000.0) - 99.75));
    case 5: return m_b_max;
    default: {
      const start = 300000, delay = 10, ramp = 14 * 720;
      if (i <= delay) return start;
      if (i <= delay + ramp) {
        return start + Math.trunc((c.ramp_multiplier - 1.0) * start / ramp) * (i - delay);
      }
      return Math.trunc(c.ramp_multiplier * start + 220.0 * Math.sin(i / 802.0) * 800.0);
    }
  }
}

function chooseFeeLevel(c, levels, mempool, t, m_n, last10) {
  if (c.fee_level >= 0) return Math.min(c.fee_level, levels.length - 1);
  let pool = 0;
  for (const m of mempool) pool += m;
  pool *= t;
  let recent = 0;
  for (const v of last10) recent += v;
  const backlog = pool >= m_n;
  const busy = last10.length === 10 && 100 * recent > 80 * 10 * m_n;
  return backlog || busy ? 1 : 0;
}

export function runSimulationJS(config) {
  const c = { ...presetConfig(config.rules ?? RULES_V17), ...config };
  const n = c.n;
  const lw = Math.max(c.long_window, 2);
  const sw = Math.max(c.short_window, 2);
  const grace = c.wallet_grace_blocks;
  const init = c.initial_weight;
  const t = Math.max(c.t_sim, 1);

  const longTerm = SlidingOrderStat.medianOf(lw, init);
  const shortTerm = SlidingOrderStat.medianOf(sw, init);
  const longTermWallet = new GraceMedian(lw, grace, init);
  const shortTermWallet = new GraceMedian(sw, Math.min(grace, Math.floor(sw / 2) - 1), init);

  const rng = new Rng(12345);
  let mempool = [];
  let percent = 0.0;
  const last10 = [];

  const res = {
    max_mb: 0, max_penalty: 0.0, max_mempool: 0, cumulative_fees: 0.0, rejected_txs: 0,
    data_points: n, fee_levels: [], min_fee_per_byte: 0.0,
    M_B: new Array(n), M_L: new Array(n), M_S: new Array(n), M_N: new Array(n),
    input_volume: new Array(n), block_fee: new Array(n), penalty: new Array(n),
    mempool_size: new Array(n), fee_low: new Array(n), indices: new Array(n),
  };

  for (let i = 0; i < n; i++) {
    const m_l = Math.max(longTerm.median(), c.z_m);
    const m_s = shortTerm.median();
    let m_n = Math.min(Math.max(m_s, m_l), c.surge_factor * m_l);
    if (m_n <= c.z_m) m_n = c.z_m;
    let m_b_max = 2 * m_n;
    if (c.sanity_cap && i >= c.sanity_start_block) {
      const k = i - c.sanity_start_block;
      const a_c = Math.trunc(c.sanity_start_weight * Math.pow(1.0 + 5.0 / 4.0e6, k));
      m_b_max = Math.min(m_b_max, a_c);
    }

    const levels = walletFeeLevels(c, longTermWallet.median(), shortTermWallet.median());
    const f_min = minFeePerByte(c, m_l);
    if (mempool.length !== levels.length) {
      const old = mempool;
      mempool = new Array(levels.length).fill(0);
      for (let k = 0; k < Math.min(old.length, levels.length); k++) mempool[k] = old[k];
    }

    const vol = demandBytes(c, i, m_b_max);
    let count = Math.trunc(vol / t);
    if (c.add_noise && count > 0) {
      const noise = 0.2 * rng.normal(0.0, count);
      count = Math.max(count + Math.trunc(noise), 1);
    }

    const base = chooseFeeLevel(c, levels, mempool, t, m_n, last10);
    const high = Math.min(base + 2, levels.length - 1);
    const broadcast = new Array(levels.length).fill(0);
    broadcast[base] = count;
    if (c.users_pay_more && count > 0 && high !== base) {
      const calc = Math.floor(mempool[base] / (3.0 * count) * 100.0);
      percent = Math.min(Math.max(Math.floor(percent + 0.1 * (calc - percent)), 0), 100);
      const moved = Math.trunc(count * percent / 100.0);
      broadcast[high] += moved;
      broadcast[base] -= moved;
    }
    let bsum = 0;
    for (let k = 0; k < broadcast.length; k++) {
      bsum += broadcast[k];
      if (levels[k] * t >= 0.98 * f_min * t) mempool[k] += broadcast[k];
      else res.rejected_txs += broadcast[k];
    }

    let m_b = 0;
    let fees = 0.0;
    for (let k = levels.length - 1; k >= 0; k--) {
      const fee = levels[k] * t;
      const take = c.rational_miners
        ? affordableTxs(c.r_base, m_b, m_n, t, fee, m_b_max, mempool[k])
        : Math.max(Math.min(Math.trunc((m_b_max - m_b) / t), mempool[k]), 0);
      mempool[k] -= take;
      m_b += take * t;
      fees += take * fee;
    }
    const p_b = penaltyOf(c.r_base, m_b, m_n);

    const lower = Math.trunc(m_l * c.lt_growth_den / c.lt_growth_num);
    const upper = Math.trunc(m_l * c.lt_growth_num / c.lt_growth_den);
    const ltw = Math.min(Math.max(m_b, lower), upper);
    longTerm.push(ltw);
    longTermWallet.push(ltw);
    shortTerm.push(m_b);
    shortTermWallet.push(m_b);
    last10.push(m_b);
    if (last10.length > 10) last10.shift();

    let pool = 0;
    for (const m of mempool) pool += m;
    const mempool_bytes = pool * t;
    if (mempool_bytes > res.max_mempool) res.max_mempool = mempool_bytes;
    if (m_b > res.max_mb) res.max_mb = m_b;
    if (p_b > res.max_penalty) res.max_penalty = p_b;
    res.cumulative_fees += fees;
    res.M_B[i] = m_b;
    res.M_L[i] = m_l;
    res.M_S[i] = m_s;
    res.M_N[i] = m_n;
    res.input_volume[i] = bsum * t;
    res.block_fee[i] = fees;
    res.penalty[i] = p_b;
    res.mempool_size[i] = mempool_bytes;
    res.fee_low[i] = levels[0];
    res.indices[i] = i;
    if (i + 1 === n) {
      res.fee_levels = levels;
      res.min_fee_per_byte = f_min;
    }
  }
  return res;
}
