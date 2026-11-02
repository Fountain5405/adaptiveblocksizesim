// Parity test: JS engine (docs/engine.js) vs Rust/WASM (docs/wasm_sim.js). Run: node tests/parity.mjs
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { runSimulationJS, presetConfig } from '../docs/engine.js';
import * as W from '../docs/wasm_sim.js';

const docs = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'docs');
W.initSync({ module: fs.readFileSync(path.join(docs, 'wasm_sim_bg.wasm')) });

const BIG = ['z_m', 'lt_growth_num', 'lt_growth_den', 'surge_factor', 'initial_weight',
  'sanity_start_weight', 't_r', 't_sim'];
const N = 120000;

function runWasm(cfg) {
  const c = new W.SimulationConfig(cfg.rules);
  for (const [k, v] of Object.entries(cfg)) c[k] = BIG.includes(k) ? BigInt(v) : v;
  const s = W.run_simulation(c);
  const r = {
    max_mb: Number(s.max_mb), max_penalty: s.max_penalty, max_mempool: Number(s.max_mempool),
    cumulative_fees: s.cumulative_fees, rejected_txs: Number(s.rejected_txs),
    min_fee_per_byte: s.min_fee_per_byte, data_points: s.data_points,
    M_B: Array.from(W.get_m_b(), Number), M_L: Array.from(W.get_m_l(), Number),
    M_S: Array.from(W.get_m_s(), Number), M_N: Array.from(W.get_m_n(), Number),
    input_volume: Array.from(W.get_input_volume(), Number), block_fee: Array.from(W.get_block_fee()),
    penalty: Array.from(W.get_penalty()), mempool_size: Array.from(W.get_mempool(), Number),
    fee_low: Array.from(W.get_fee_low()), fee_levels: Array.from(W.get_fee_levels()),
  };
  s.free(); // run_simulation consumes the config
  return r;
}

const INT_SERIES = ['M_B', 'M_L', 'M_S', 'M_N', 'input_volume', 'mempool_size'];
const FLOAT_SERIES = ['block_fee', 'penalty', 'fee_low'];
const INT_SUMMARY = ['max_mb', 'max_mempool', 'rejected_txs', 'data_points'];
const FLOAT_SUMMARY = ['max_penalty', 'cumulative_fees', 'min_fee_per_byte'];
const rel = (a, b) => (a === b ? 0 : Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)));

function compare(a, b) {
  const errs = [];
  for (const k of INT_SERIES) {
    if (a[k].length !== b[k].length) { errs.push(`${k}: length`); continue; }
    for (let i = 0; i < a[k].length; i++) if (a[k][i] !== b[k][i]) { errs.push(`${k}[${i}] js=${a[k][i]} wasm=${b[k][i]}`); break; }
  }
  for (const k of FLOAT_SERIES) {
    for (let i = 0; i < a[k].length; i++) if (rel(a[k][i], b[k][i]) >= 1e-12) { errs.push(`${k}[${i}] js=${a[k][i]} wasm=${b[k][i]}`); break; }
  }
  for (const k of INT_SUMMARY) if (a[k] !== b[k]) errs.push(`${k} js=${a[k]} wasm=${b[k]}`);
  for (const k of FLOAT_SUMMARY) if (rel(a[k], b[k]) >= 1e-12) errs.push(`${k} js=${a[k]} wasm=${b[k]}`);
  if (a.fee_levels.length !== b.fee_levels.length || a.fee_levels.some((v, i) => rel(v, b.fee_levels[i]) >= 1e-12)) errs.push('fee_levels');
  return errs;
}

const cases = [];
for (const rules of [0, 1]) for (const run_type of [1, 5, 6]) for (const rational_miners of [true, false])
  cases.push({ name: `rules=${rules} rt=${run_type} rational=${rational_miners}`, cfg: { ...presetConfig(rules), n: N, run_type, rational_miners } });
cases.push({ name: 'rules=1 rt=6 noise+pay_more', cfg: { ...presetConfig(1), n: N, run_type: 6, add_noise: true, users_pay_more: true } });
cases.push({ name: 'custom g=1.5 surge=10 sanity', cfg: { ...presetConfig(2), n: N, run_type: 6, lt_growth_num: 1500, lt_growth_den: 1000, surge_factor: 10, sanity_cap: true, sanity_start_weight: 1000000 } });

let fail = 0;
console.log('case'.padEnd(36), 'result', ' js ms', 'wasm ms');
for (const { name, cfg } of cases) {
  let t0 = performance.now(); const w = runWasm(cfg); const tw = performance.now() - t0;
  t0 = performance.now(); const j = runSimulationJS(cfg); const tj = performance.now() - t0;
  const errs = compare(j, w);
  if (errs.length) fail++;
  console.log(name.padEnd(36), (errs.length ? 'FAIL' : 'PASS').padEnd(6), tj.toFixed(0).padStart(6), tw.toFixed(0).padStart(7));
  for (const e of errs) console.log('    ' + e);
}
console.log(fail ? `${fail} case(s) FAILED` : 'all PASS');
process.exit(fail ? 1 : 0);
