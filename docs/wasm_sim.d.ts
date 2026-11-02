/* tslint:disable */
/* eslint-disable */

export class SimulationConfig {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * rules: 0 = v16 (mainnet, 2021 scaling), 1 = v17 (FCMP++, 2026 scaling), 2 = custom
   */
  constructor(rules: number);
  n: number;
  rules: number;
  z_m: bigint;
  lt_growth_num: bigint;
  lt_growth_den: bigint;
  surge_factor: bigint;
  long_window: number;
  short_window: number;
  initial_weight: bigint;
  sanity_cap: boolean;
  sanity_start_weight: bigint;
  sanity_start_block: number;
  t_r: bigint;
  min_fee_fraction: number;
  wallet_grace_blocks: number;
  fee_rounding_digits: number;
  fee_level: number;
  r_base: number;
  t_sim: bigint;
  run_type: number;
  ramp_multiplier: number;
  add_noise: boolean;
  users_pay_more: boolean;
  rational_miners: boolean;
}

export class SimulationResults {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  max_mb: bigint;
  max_penalty: number;
  max_mempool: bigint;
  cumulative_fees: number;
  rejected_txs: bigint;
  min_fee_per_byte: number;
  data_points: number;
}

export function get_block_fee(): Float64Array;

export function get_fee_levels(): Float64Array;

export function get_fee_low(): Float64Array;

export function get_input_volume(): BigInt64Array;

export function get_m_b(): BigInt64Array;

export function get_m_l(): BigInt64Array;

export function get_m_n(): BigInt64Array;

export function get_m_s(): BigInt64Array;

export function get_mempool(): BigInt64Array;

export function get_penalty(): Float64Array;

/**
 * Initialize the WASM module
 */
export function main(): void;

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
  readonly __wbg_get_simulationconfig_rules: (a: number) => number;
  readonly __wbg_set_simulationconfig_rules: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_z_m: (a: number) => bigint;
  readonly __wbg_set_simulationconfig_z_m: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationconfig_lt_growth_num: (a: number) => bigint;
  readonly __wbg_set_simulationconfig_lt_growth_num: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationconfig_lt_growth_den: (a: number) => bigint;
  readonly __wbg_set_simulationconfig_lt_growth_den: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationconfig_surge_factor: (a: number) => bigint;
  readonly __wbg_set_simulationconfig_surge_factor: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationconfig_long_window: (a: number) => number;
  readonly __wbg_set_simulationconfig_long_window: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_short_window: (a: number) => number;
  readonly __wbg_set_simulationconfig_short_window: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_initial_weight: (a: number) => bigint;
  readonly __wbg_set_simulationconfig_initial_weight: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationconfig_sanity_cap: (a: number) => number;
  readonly __wbg_set_simulationconfig_sanity_cap: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_sanity_start_weight: (a: number) => bigint;
  readonly __wbg_set_simulationconfig_sanity_start_weight: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationconfig_sanity_start_block: (a: number) => number;
  readonly __wbg_set_simulationconfig_sanity_start_block: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_t_r: (a: number) => bigint;
  readonly __wbg_set_simulationconfig_t_r: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationconfig_min_fee_fraction: (a: number) => number;
  readonly __wbg_set_simulationconfig_min_fee_fraction: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_wallet_grace_blocks: (a: number) => number;
  readonly __wbg_set_simulationconfig_wallet_grace_blocks: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_fee_rounding_digits: (a: number) => number;
  readonly __wbg_set_simulationconfig_fee_rounding_digits: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_fee_level: (a: number) => number;
  readonly __wbg_set_simulationconfig_fee_level: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_r_base: (a: number) => number;
  readonly __wbg_set_simulationconfig_r_base: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_t_sim: (a: number) => bigint;
  readonly __wbg_set_simulationconfig_t_sim: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationconfig_run_type: (a: number) => number;
  readonly __wbg_set_simulationconfig_run_type: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_ramp_multiplier: (a: number) => number;
  readonly __wbg_set_simulationconfig_ramp_multiplier: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_add_noise: (a: number) => number;
  readonly __wbg_set_simulationconfig_add_noise: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_users_pay_more: (a: number) => number;
  readonly __wbg_set_simulationconfig_users_pay_more: (a: number, b: number) => void;
  readonly __wbg_get_simulationconfig_rational_miners: (a: number) => number;
  readonly __wbg_set_simulationconfig_rational_miners: (a: number, b: number) => void;
  readonly simulationconfig_new: (a: number) => number;
  readonly __wbg_simulationresults_free: (a: number, b: number) => void;
  readonly __wbg_get_simulationresults_max_penalty: (a: number) => number;
  readonly __wbg_set_simulationresults_max_penalty: (a: number, b: number) => void;
  readonly __wbg_get_simulationresults_cumulative_fees: (a: number) => number;
  readonly __wbg_set_simulationresults_cumulative_fees: (a: number, b: number) => void;
  readonly __wbg_get_simulationresults_min_fee_per_byte: (a: number) => number;
  readonly __wbg_set_simulationresults_min_fee_per_byte: (a: number, b: number) => void;
  readonly __wbg_get_simulationresults_data_points: (a: number) => number;
  readonly __wbg_set_simulationresults_data_points: (a: number, b: number) => void;
  readonly run_simulation: (a: number) => number;
  readonly get_m_b: () => [number, number];
  readonly get_m_l: () => [number, number];
  readonly get_m_s: () => [number, number];
  readonly get_m_n: () => [number, number];
  readonly get_input_volume: () => [number, number];
  readonly get_block_fee: () => [number, number];
  readonly get_penalty: () => [number, number];
  readonly get_mempool: () => [number, number];
  readonly get_fee_low: () => [number, number];
  readonly get_fee_levels: () => [number, number];
  readonly test_wasm: () => [number, number];
  readonly main: () => void;
  readonly __wbg_set_simulationresults_max_mb: (a: number, b: bigint) => void;
  readonly __wbg_set_simulationresults_max_mempool: (a: number, b: bigint) => void;
  readonly __wbg_set_simulationresults_rejected_txs: (a: number, b: bigint) => void;
  readonly __wbg_get_simulationresults_max_mb: (a: number) => bigint;
  readonly __wbg_get_simulationresults_max_mempool: (a: number) => bigint;
  readonly __wbg_get_simulationresults_rejected_txs: (a: number) => bigint;
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
