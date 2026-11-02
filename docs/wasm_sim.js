let wasm;

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function getArrayF64FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat64ArrayMemory0().subarray(ptr / 8, ptr / 8 + len);
}

function getArrayI64FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getBigInt64ArrayMemory0().subarray(ptr / 8, ptr / 8 + len);
}

let cachedBigInt64ArrayMemory0 = null;
function getBigInt64ArrayMemory0() {
    if (cachedBigInt64ArrayMemory0 === null || cachedBigInt64ArrayMemory0.byteLength === 0) {
        cachedBigInt64ArrayMemory0 = new BigInt64Array(wasm.memory.buffer);
    }
    return cachedBigInt64ArrayMemory0;
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat64ArrayMemory0 = null;
function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
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
 * Simulation configuration (WASM wrapper).
 * Create one from a rule preset, then set any fields to change from the preset:
 *   const c = new SimulationConfig(1); c.n = 300000; c.t_sim = 9093;
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
     * @returns {number}
     */
    get rules() {
        const ret = wasm.__wbg_get_simulationconfig_rules(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set rules(arg0) {
        wasm.__wbg_set_simulationconfig_rules(this.__wbg_ptr, arg0);
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
    get lt_growth_num() {
        const ret = wasm.__wbg_get_simulationconfig_lt_growth_num(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set lt_growth_num(arg0) {
        wasm.__wbg_set_simulationconfig_lt_growth_num(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get lt_growth_den() {
        const ret = wasm.__wbg_get_simulationconfig_lt_growth_den(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set lt_growth_den(arg0) {
        wasm.__wbg_set_simulationconfig_lt_growth_den(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get surge_factor() {
        const ret = wasm.__wbg_get_simulationconfig_surge_factor(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set surge_factor(arg0) {
        wasm.__wbg_set_simulationconfig_surge_factor(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get long_window() {
        const ret = wasm.__wbg_get_simulationconfig_long_window(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set long_window(arg0) {
        wasm.__wbg_set_simulationconfig_long_window(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get short_window() {
        const ret = wasm.__wbg_get_simulationconfig_short_window(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set short_window(arg0) {
        wasm.__wbg_set_simulationconfig_short_window(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get initial_weight() {
        const ret = wasm.__wbg_get_simulationconfig_initial_weight(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set initial_weight(arg0) {
        wasm.__wbg_set_simulationconfig_initial_weight(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get sanity_cap() {
        const ret = wasm.__wbg_get_simulationconfig_sanity_cap(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set sanity_cap(arg0) {
        wasm.__wbg_set_simulationconfig_sanity_cap(this.__wbg_ptr, arg0);
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
    get min_fee_fraction() {
        const ret = wasm.__wbg_get_simulationconfig_min_fee_fraction(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set min_fee_fraction(arg0) {
        wasm.__wbg_set_simulationconfig_min_fee_fraction(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get wallet_grace_blocks() {
        const ret = wasm.__wbg_get_simulationconfig_wallet_grace_blocks(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set wallet_grace_blocks(arg0) {
        wasm.__wbg_set_simulationconfig_wallet_grace_blocks(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get fee_rounding_digits() {
        const ret = wasm.__wbg_get_simulationconfig_fee_rounding_digits(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set fee_rounding_digits(arg0) {
        wasm.__wbg_set_simulationconfig_fee_rounding_digits(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get fee_level() {
        const ret = wasm.__wbg_get_simulationconfig_fee_level(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set fee_level(arg0) {
        wasm.__wbg_set_simulationconfig_fee_level(this.__wbg_ptr, arg0);
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
    get rational_miners() {
        const ret = wasm.__wbg_get_simulationconfig_rational_miners(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set rational_miners(arg0) {
        wasm.__wbg_set_simulationconfig_rational_miners(this.__wbg_ptr, arg0);
    }
    /**
     * rules: 0 = v16 (mainnet, 2021 scaling), 1 = v17 (FCMP++, 2026 scaling), 2 = custom
     * @param {number} rules
     */
    constructor(rules) {
        const ret = wasm.simulationconfig_new(rules);
        this.__wbg_ptr = ret >>> 0;
        SimulationConfigFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) SimulationConfig.prototype[Symbol.dispose] = SimulationConfig.prototype.free;

/**
 * Summary of the last run (WASM wrapper). Per-block series are read with the get_* functions.
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
        const ret = wasm.__wbg_get_simulationconfig_z_m(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set max_mb(arg0) {
        wasm.__wbg_set_simulationconfig_z_m(this.__wbg_ptr, arg0);
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
        const ret = wasm.__wbg_get_simulationconfig_lt_growth_den(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set max_mempool(arg0) {
        wasm.__wbg_set_simulationconfig_lt_growth_den(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get cumulative_fees() {
        const ret = wasm.__wbg_get_simulationresults_cumulative_fees(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set cumulative_fees(arg0) {
        wasm.__wbg_set_simulationresults_cumulative_fees(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get rejected_txs() {
        const ret = wasm.__wbg_get_simulationconfig_initial_weight(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set rejected_txs(arg0) {
        wasm.__wbg_set_simulationconfig_initial_weight(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get min_fee_per_byte() {
        const ret = wasm.__wbg_get_simulationresults_min_fee_per_byte(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set min_fee_per_byte(arg0) {
        wasm.__wbg_set_simulationresults_min_fee_per_byte(this.__wbg_ptr, arg0);
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
 * @returns {Float64Array}
 */
export function get_block_fee() {
    const ret = wasm.get_block_fee();
    var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * @returns {Float64Array}
 */
export function get_fee_levels() {
    const ret = wasm.get_fee_levels();
    var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * @returns {Float64Array}
 */
export function get_fee_low() {
    const ret = wasm.get_fee_low();
    var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * @returns {BigInt64Array}
 */
export function get_input_volume() {
    const ret = wasm.get_input_volume();
    var v1 = getArrayI64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * @returns {BigInt64Array}
 */
export function get_m_b() {
    const ret = wasm.get_m_b();
    var v1 = getArrayI64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * @returns {BigInt64Array}
 */
export function get_m_l() {
    const ret = wasm.get_m_l();
    var v1 = getArrayI64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * @returns {BigInt64Array}
 */
export function get_m_n() {
    const ret = wasm.get_m_n();
    var v1 = getArrayI64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * @returns {BigInt64Array}
 */
export function get_m_s() {
    const ret = wasm.get_m_s();
    var v1 = getArrayI64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * @returns {BigInt64Array}
 */
export function get_mempool() {
    const ret = wasm.get_mempool();
    var v1 = getArrayI64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * @returns {Float64Array}
 */
export function get_penalty() {
    const ret = wasm.get_penalty();
    var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * Initialize the WASM module
 */
export function main() {
    wasm.main();
}

/**
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
    cachedBigInt64ArrayMemory0 = null;
    cachedDataViewMemory0 = null;
    cachedFloat64ArrayMemory0 = null;
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
