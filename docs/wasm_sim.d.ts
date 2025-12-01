/* tslint:disable */
/* eslint-disable */

export class SimulationConfig {
  free(): void;
  [Symbol.dispose](): void;
  constructor(n: number, steady_state: bigint, z_m: bigint, t_r: bigint, r_base: number, mid_100k: number, mid_100: number, t_sim: bigint, run_type: number, ramp_multiplier: number, ml_mult: number, mn_mult: number, add_noise: boolean, users_pay_more: boolean, simple_blocks: boolean, large_sim_mode: boolean, exact_median: boolean);
  n: number;
  steady_state: bigint;
  z_m: bigint;
  t_r: bigint;
  r_base: number;
  mid_100k: number;
  mid_100: number;
  t_sim: bigint;
  run_type: number;
  ramp_multiplier: number;
  ml_mult: number;
  mn_mult: number;
  add_noise: boolean;
  users_pay_more: boolean;
  simple_blocks: boolean;
  large_sim_mode: boolean;
  exact_median: boolean;
}

export class SimulationResults {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  max_mb: bigint;
  max_penalty: number;
  max_mempool: bigint;
  cumulative_fees: number;
  data_points: number;
}

export function get_block_fee_ptr(): number;

export function get_cumulative_fees(): number;

export function get_indices_ptr(): number;

export function get_input_vol_ptr(): number;

/**
 * Get pointer to result arrays for JS to read
 */
export function get_m_b_ptr(): number;

export function get_m_l_ptr(): number;

export function get_m_n_ptr(): number;

export function get_m_s_ptr(): number;

export function get_max_mb(): bigint;

export function get_max_mempool(): bigint;

export function get_max_penalty(): number;

export function get_mempool_ptr(): number;

export function get_penalty_ptr(): number;

export function get_result_len(): number;

/**
 * Initialize the WASM module
 */
export function main(): void;

/**
 * Main simulation function - NOTE: This returns immediately but data must be read from memory
 */
export function run_simulation(config: SimulationConfig): SimulationResults;

/**
 * Utility function to test WASM is working
 */
export function test_wasm(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_simulationconfig_free: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_n: (a: number) => number;
  readonly __wbg_set_simulationconfig_n: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_steady_state: (a: number) => bigint;
  readonly __wbg_set_simulationconfig_steady_state: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationconfig_z_m: (a: number) => bigint;
  readonly __wbg_set_simulationconfig_z_m: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationconfig_t_r: (a: number) => bigint;
  readonly __wbg_set_simulationconfig_t_r: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationconfig_r_base: (a: number) => number;
  readonly __wbg_set_simulationconfig_r_base: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_mid_100k: (a: number) => number;
  readonly __wbg_set_simulationconfig_mid_100k: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_mid_100: (a: number) => number;
  readonly __wbg_set_simulationconfig_mid_100: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_t_sim: (a: number) => bigint;
  readonly __wbg_set_simulationconfig_t_sim: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationconfig_run_type: (a: number) => number;
  readonly __wbg_set_simulationconfig_run_type: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_ramp_multiplier: (a: number) => number;
  readonly __wbg_set_simulationconfig_ramp_multiplier: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_ml_mult: (a: number) => number;
  readonly __wbg_set_simulationconfig_ml_mult: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_mn_mult: (a: number) => number;
  readonly __wbg_set_simulationconfig_mn_mult: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_add_noise: (a: number) => number;
  readonly __wbg_set_simulationconfig_add_noise: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_users_pay_more: (a: number) => number;
  readonly __wbg_set_simulationconfig_users_pay_more: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_simple_blocks: (a: number) => number;
  readonly __wbg_set_simulationconfig_simple_blocks: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_large_sim_mode: (a: number) => number;
  readonly __wbg_set_simulationconfig_large_sim_mode: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_exact_median: (a: number) => number;
  readonly __wbg_set_simulationconfig_exact_median: (a: number, b: number) => void;
  readonly simulationconfig_new: (a: number, b: bigint, c: bigint, d: bigint, e: number, f: number, g: number, h: bigint, i: number, j: number, k: number, l: number, m: number, n: number, o: number, p: number, q: number) => number;
  readonly __wbg_simulationresults_free: (a: number, b: number) => void;
  readonly __wbg_get_simulationresults_max_penalty: (a: number) => number;
  readonly __wbg_set_simulationresults_max_penalty: (a: number, b: number) => void;
  readonly __wbg_get_simulationresults_data_points: (a: number) => number;
  readonly __wbg_set_simulationresults_data_points: (a: number, b: number) => void;
  readonly get_m_b_ptr: () => number;
  readonly get_m_l_ptr: () => number;
  readonly get_m_s_ptr: () => number;
  readonly get_m_n_ptr: () => number;
  readonly get_input_vol_ptr: () => number;
  readonly get_block_fee_ptr: () => number;
  readonly get_penalty_ptr: () => number;
  readonly get_mempool_ptr: () => number;
  readonly get_indices_ptr: () => number;
  readonly get_result_len: () => number;
  readonly get_max_mb: () => bigint;
  readonly get_max_penalty: () => number;
  readonly get_max_mempool: () => bigint;
  readonly get_cumulative_fees: () => number;
  readonly run_simulation: (a: number) => number;
  readonly test_wasm: () => [number, number];
  readonly main: () => void;
  readonly __wbg_set_simulationresults_max_mb: (a: number, b: bigint) => void;
  readonly __wbg_set_simulationresults_max_mempool: (a: number, b: bigint) => void;
  readonly __wbg_set_simulationresults_cumulative_fees: (a: number, b: number) => void;
  readonly __wbg_get_simulationresults_max_mb: (a: number) => bigint;
  readonly __wbg_get_simulationresults_max_mempool: (a: number) => bigint;
  readonly __wbg_get_simulationresults_cumulative_fees: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
