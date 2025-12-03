let wasm;

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    }
}

let WASM_VECTOR_LEN = 0;

const SimulationConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_simulationconfig_free(ptr >>> 0, 1));

const SimulationResultsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_simulationresults_free(ptr >>> 0, 1));

/**
 * Simulation configuration structure (WASM wrapper)
 */
export class SimulationConfig {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SimulationConfigFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_simulationconfig_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get n() {
        const ret = wasm.__wbg_get_simulationconfig_n(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set n(arg0) {
        wasm.__wbg_set_simulationconfig_n(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get steady_state() {
        const ret = wasm.__wbg_get_simulationconfig_steady_state(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set steady_state(arg0) {
        wasm.__wbg_set_simulationconfig_steady_state(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get z_m() {
        const ret = wasm.__wbg_get_simulationconfig_z_m(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set z_m(arg0) {
        wasm.__wbg_set_simulationconfig_z_m(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get t_r() {
        const ret = wasm.__wbg_get_simulationconfig_t_r(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set t_r(arg0) {
        wasm.__wbg_set_simulationconfig_t_r(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get r_base() {
        const ret = wasm.__wbg_get_simulationconfig_r_base(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set r_base(arg0) {
        wasm.__wbg_set_simulationconfig_r_base(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get mid_100k() {
        const ret = wasm.__wbg_get_simulationconfig_mid_100k(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set mid_100k(arg0) {
        wasm.__wbg_set_simulationconfig_mid_100k(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get mid_100() {
        const ret = wasm.__wbg_get_simulationconfig_mid_100(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set mid_100(arg0) {
        wasm.__wbg_set_simulationconfig_mid_100(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get t_sim() {
        const ret = wasm.__wbg_get_simulationconfig_t_sim(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set t_sim(arg0) {
        wasm.__wbg_set_simulationconfig_t_sim(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get run_type() {
        const ret = wasm.__wbg_get_simulationconfig_run_type(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set run_type(arg0) {
        wasm.__wbg_set_simulationconfig_run_type(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get ramp_multiplier() {
        const ret = wasm.__wbg_get_simulationconfig_ramp_multiplier(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set ramp_multiplier(arg0) {
        wasm.__wbg_set_simulationconfig_ramp_multiplier(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get ml_mult() {
        const ret = wasm.__wbg_get_simulationconfig_ml_mult(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set ml_mult(arg0) {
        wasm.__wbg_set_simulationconfig_ml_mult(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get mn_mult() {
        const ret = wasm.__wbg_get_simulationconfig_mn_mult(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set mn_mult(arg0) {
        wasm.__wbg_set_simulationconfig_mn_mult(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get add_noise() {
        const ret = wasm.__wbg_get_simulationconfig_add_noise(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set add_noise(arg0) {
        wasm.__wbg_set_simulationconfig_add_noise(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get users_pay_more() {
        const ret = wasm.__wbg_get_simulationconfig_users_pay_more(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set users_pay_more(arg0) {
        wasm.__wbg_set_simulationconfig_users_pay_more(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get simple_blocks() {
        const ret = wasm.__wbg_get_simulationconfig_simple_blocks(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set simple_blocks(arg0) {
        wasm.__wbg_set_simulationconfig_simple_blocks(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get large_sim_mode() {
        const ret = wasm.__wbg_get_simulationconfig_large_sim_mode(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set large_sim_mode(arg0) {
        wasm.__wbg_set_simulationconfig_large_sim_mode(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get exact_median() {
        const ret = wasm.__wbg_get_simulationconfig_exact_median(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set exact_median(arg0) {
        wasm.__wbg_set_simulationconfig_exact_median(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get max_blocksize() {
        const ret = wasm.__wbg_get_simulationconfig_max_blocksize(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set max_blocksize(arg0) {
        wasm.__wbg_set_simulationconfig_max_blocksize(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get max_blocksize_growth_rate() {
        const ret = wasm.__wbg_get_simulationconfig_max_blocksize_growth_rate(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set max_blocksize_growth_rate(arg0) {
        wasm.__wbg_set_simulationconfig_max_blocksize_growth_rate(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get use_long_term_median_cap() {
        const ret = wasm.__wbg_get_simulationconfig_use_long_term_median_cap(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set use_long_term_median_cap(arg0) {
        wasm.__wbg_set_simulationconfig_use_long_term_median_cap(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get sanity_start_weight() {
        const ret = wasm.__wbg_get_simulationconfig_sanity_start_weight(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set sanity_start_weight(arg0) {
        wasm.__wbg_set_simulationconfig_sanity_start_weight(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get sanity_start_block() {
        const ret = wasm.__wbg_get_simulationconfig_sanity_start_block(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set sanity_start_block(arg0) {
        wasm.__wbg_set_simulationconfig_sanity_start_block(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} n
     * @param {bigint} steady_state
     * @param {bigint} z_m
     * @param {bigint} t_r
     * @param {number} r_base
     * @param {number} mid_100k
     * @param {number} mid_100
     * @param {bigint} t_sim
     * @param {number} run_type
     * @param {number} ramp_multiplier
     * @param {number} ml_mult
     * @param {number} mn_mult
     * @param {boolean} add_noise
     * @param {boolean} users_pay_more
     * @param {boolean} simple_blocks
     * @param {boolean} large_sim_mode
     * @param {boolean} exact_median
     * @param {bigint} max_blocksize
     * @param {number} max_blocksize_growth_rate
     * @param {boolean} use_long_term_median_cap
     * @param {bigint} sanity_start_weight
     * @param {number} sanity_start_block
     */
    constructor(n, steady_state, z_m, t_r, r_base, mid_100k, mid_100, t_sim, run_type, ramp_multiplier, ml_mult, mn_mult, add_noise, users_pay_more, simple_blocks, large_sim_mode, exact_median, max_blocksize, max_blocksize_growth_rate, use_long_term_median_cap, sanity_start_weight, sanity_start_block) {
        const ret = wasm.simulationconfig_new(n, steady_state, z_m, t_r, r_base, mid_100k, mid_100, t_sim, run_type, ramp_multiplier, ml_mult, mn_mult, add_noise, users_pay_more, simple_blocks, large_sim_mode, exact_median, max_blocksize, max_blocksize_growth_rate, use_long_term_median_cap, sanity_start_weight, sanity_start_block);
        this.__wbg_ptr = ret >>> 0;
        SimulationConfigFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) SimulationConfig.prototype[Symbol.dispose] = SimulationConfig.prototype.free;

/**
 * Simulation results structure (WASM wrapper)
 */
export class SimulationResults {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SimulationResults.prototype);
        obj.__wbg_ptr = ptr;
        SimulationResultsFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SimulationResultsFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_simulationresults_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    get max_mb() {
        const ret = wasm.__wbg_get_simulationconfig_steady_state(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set max_mb(arg0) {
        wasm.__wbg_set_simulationconfig_steady_state(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get max_penalty() {
        const ret = wasm.__wbg_get_simulationresults_max_penalty(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set max_penalty(arg0) {
        wasm.__wbg_set_simulationresults_max_penalty(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get max_mempool() {
        const ret = wasm.__wbg_get_simulationconfig_t_r(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set max_mempool(arg0) {
        wasm.__wbg_set_simulationconfig_t_r(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get cumulative_fees() {
        const ret = wasm.__wbg_get_simulationconfig_r_base(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set cumulative_fees(arg0) {
        wasm.__wbg_set_simulationconfig_r_base(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get data_points() {
        const ret = wasm.__wbg_get_simulationresults_data_points(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set data_points(arg0) {
        wasm.__wbg_set_simulationresults_data_points(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) SimulationResults.prototype[Symbol.dispose] = SimulationResults.prototype.free;

/**
 * @returns {number}
 */
export function get_block_fee_ptr() {
    const ret = wasm.get_block_fee_ptr();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function get_cumulative_fees() {
    const ret = wasm.get_cumulative_fees();
    return ret;
}

/**
 * @returns {number}
 */
export function get_indices_ptr() {
    const ret = wasm.get_indices_ptr();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function get_input_vol_ptr() {
    const ret = wasm.get_input_vol_ptr();
    return ret >>> 0;
}

/**
 * Get pointer to result arrays for JS to read
 * @returns {number}
 */
export function get_m_b_ptr() {
    const ret = wasm.get_m_b_ptr();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function get_m_l_ptr() {
    const ret = wasm.get_m_l_ptr();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function get_m_n_ptr() {
    const ret = wasm.get_m_n_ptr();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function get_m_s_ptr() {
    const ret = wasm.get_m_s_ptr();
    return ret >>> 0;
}

/**
 * @returns {bigint}
 */
export function get_max_mb() {
    const ret = wasm.get_max_mb();
    return ret;
}

/**
 * @returns {bigint}
 */
export function get_max_mempool() {
    const ret = wasm.get_max_mempool();
    return ret;
}

/**
 * @returns {number}
 */
export function get_max_penalty() {
    const ret = wasm.get_max_penalty();
    return ret;
}

/**
 * @returns {number}
 */
export function get_mempool_ptr() {
    const ret = wasm.get_mempool_ptr();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function get_penalty_ptr() {
    const ret = wasm.get_penalty_ptr();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function get_result_len() {
    const ret = wasm.get_result_len();
    return ret >>> 0;
}

/**
 * Initialize the WASM module
 */
export function main() {
    wasm.main();
}

/**
 * Main simulation function - NOTE: This returns immediately but data must be read from memory
 * @param {SimulationConfig} config
 * @returns {SimulationResults}
 */
export function run_simulation(config) {
    _assertClass(config, SimulationConfig);
    var ptr0 = config.__destroy_into_raw();
    const ret = wasm.run_simulation(ptr0);
    return SimulationResults.__wrap(ret);
}

/**
 * Utility function to test WASM is working
 * @returns {string}
 */
export function test_wasm() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.test_wasm();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg___wbindgen_throw_dd24417ed36fc46e = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
        const ret = new Error();
        return ret;
    };
    imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
        const ret = arg1.stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_externrefs;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
    };

    return imports;
}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('wasm_sim_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
