# Monero Scaling 2025 - Specification Compliance Analysis

## Executive Summary

This document analyzes the WASM (Rust) and JavaScript implementations against the specifications defined in `MoneroScaling2025Final_printed20251219.pdf`.

---

## 1. Consensus Rules (New) - PDF Lines 81-156

### 1.1 Z_M (Minimum Penalty Free Zone)
**Specification (PDF line 85):** `Z_M = 625000 bytes`

**WASM Implementation (lib_core.rs:212):**
```rust
let m_l_weight = m_b.min(ml_upper).max(config.z_m).max(ml_lower);
```
- ✅ **CORRECT**: Uses configurable `config.z_m` which defaults to 625000 in new rules

**JavaScript Implementation (index.html:888, 996):**
```javascript
const Z_M = cfg.z_m;
let M_L_weight = Math.max(Math.min(M_B, ML_MULT * M_L_prev), Z_M, M_L_prev / ML_MULT);
```
- ✅ **CORRECT**: Uses configurable `cfg.z_m`

---

### 1.2 M_L (Long Term Median)
**Specification (PDF lines 90-109):**
```
M_L = median over last 100000 blocks of max(min(M_B, 1.2*M_L), Z_M, M_L/1.2)
```

**WASM Implementation (lib_core.rs:208-212):**
```rust
let ml_mult = if config.use_new_scaling_rules { 1.2 } else { config.ml_mult };
let ml_upper = (ml_mult * m_l_prev as f64) as i64;
let ml_lower = (m_l_prev as f64 / ml_mult) as i64;
let m_l_weight = m_b.min(ml_upper).max(config.z_m).max(ml_lower);
```
- ✅ **CORRECT**: Implements `max(min(M_B, 1.2*M_L), Z_M, M_L/1.2)` when `use_new_scaling_rules = true`

**JavaScript Implementation (index.html:896-899, 996):**
```javascript
const ML_MULT = cfg.mlMult;
let M_L_weight = Math.max(Math.min(M_B, ML_MULT * M_L_prev), Z_M, M_L_prev / ML_MULT);
```
- ✅ **CORRECT**: Implements the formula with configurable ML_MULT (1.2 for new rules)

---

### 1.3 M_S (Short Term Median)
**Specification (PDF lines 115-118):**
```
M_S = median over last 100 blocks of M_B
```

**WASM Implementation (lib_core.rs:217-223):**
```rust
let m_s_weight = if config.use_new_scaling_rules {
    // NEW: Just M_B (PDF line 116-117: "median over the last 100 blocks of M_B")
    m_b
} else {
    // OLD: max(M_B, M_L)
    m_b.max(m_l)
};
```
- ✅ **CORRECT**: Uses only `M_B` for new rules (not `max(M_B, M_L)`)

**JavaScript Implementation (index.html:998-999):**
```javascript
// M_S_weight = max(M_B, M_L) - uses previous M_B
let M_S_weight = Math.max(M_B, M_L);
```
- ❌ **INCORRECT**: JavaScript ALWAYS uses `max(M_B, M_L)` regardless of scaling rules
- **Impact**: This causes M_S to be artificially inflated in new rules mode

---

### 1.4 M_N (Median for Penalty Calculation)
**Specification (PDF lines 120-130):**
```
M_N = min(max(M_S, M_L), 8*M_L)
```

**WASM Implementation (lib_core.rs:227-235):**
```rust
let m_n = if config.use_new_scaling_rules {
    // NEW (PDF line 120-130): M_N = min(max(M_S, M_L), 8M_L)
    let ms_cap = (config.ms_cap_multiplier * m_l as f64) as i64;
    m_s.max(m_l).min(ms_cap)
} else {
    // OLD (PDF line 46-50): M_N = min(M_S, 50*M_L)
    let ms_cap = (config.mn_mult * m_l as f64) as i64;
    m_s.min(ms_cap)
};
```
- ✅ **CORRECT**: Implements `min(max(M_S, M_L), 8*M_L)` for new rules

**JavaScript Implementation (index.html:1001-1003):**
```javascript
// M_N calculation - NEW RULES: M_N = M_S (no cap)
// OLD: let M_N = Math.min(M_S, MN_MULT * M_L);
let M_N = M_S;
```
- ❌ **INCORRECT**: JavaScript uses `M_N = M_S` (no formula application)
- **Expected**: Should be `min(max(M_S, M_L), 8*M_L)` for new rules
- **Impact**: This is a CRITICAL ERROR - the penalty median calculation is wrong

---

### 1.5 Block Size Validation
**Specification (PDF lines 135-138):**
```
0 < M_B ≤ 2*M_N (requirement for valid block)
```

**WASM Implementation (lib_core.rs:252-259):**
```rust
let m_b_max = if config.use_new_scaling_rules {
    // New: M_B_max = min(2*M_N, 16*M_L, A_C)
    (2 * m_n).min(16 * m_l).min(sanity_cap)
} else {
    // Old: M_B_max = 2*M_N
    2 * m_n
};
```
- ✅ **CORRECT**: New rules add 16*M_L and sanity cap constraints

**JavaScript Implementation (index.html:1015-1018):**
```javascript
// M_B_max calculation - NEW RULES: min(2*M_N, 16*M_L, A_C)
// OLD: M_B_max = 2 * M_N
// NEW: M_B_max = min(2*M_N, 16*M_L, A_C)
let M_B_max = Math.min(2 * M_N, 16 * M_L, sanity_cap);
```
- ✅ **CORRECT**: Implements new rules formula

---

### 1.6 Penalty Calculation
**Specification (PDF lines 140-156):**
```
B = M_B/M_N - 1 where -1 < B ≤ 1
P_B = R_Base * B^2 for B > 0
P_B = 0 for B ≤ 0
```

**WASM Implementation (lib_core.rs:466-467):**
```rust
let b_final = (m_b as f64 / m_n as f64) - 1.0;
let p_b = if b_final > 0.0 { config.r_base * b_final * b_final } else { 0.0 };
```
- ✅ **CORRECT**: Matches specification exactly

**JavaScript Implementation (index.html:1249-1251):**
```javascript
let B_final = (M_B / M_N) - 1;
let P_B = R_Base * (B_final * B_final);
if (B_final <= 0) P_B = 0;
```
- ✅ **CORRECT**: Matches specification exactly

---

## 2. Minimum Fee For Node Relay (New) - PDF Lines 309-435

### 2.1 T_R (Reference Transaction Weight)
**Specification (PDF lines 318-319):**
```
T_R = 10000 bytes
```

**WASM Implementation (lib_core.rs:311):**
```rust
let t_r = config.t_r;  // T_R: old=3000, new=10000 bytes (PDF lines 179, 305)
```
- ✅ **CORRECT**: Configurable, defaults to 10000 for new rules

**JavaScript Implementation (index.html:889):**
```javascript
const T_R = cfg.t_r;
```
- ✅ **CORRECT**: Configurable

---

### 2.2 Minimum Fee Percentage
**Specification (PDF lines 403-404):**
```
100% of the fee required to pay the penalty incurred is the minimum fee
```

**WASM Implementation (lib_core.rs:315-321):**
```rust
let min_fee_percentage = if config.use_new_scaling_rules {
    config.min_fee_percentage  // New: 1.0 (100% - no reduction, PDF line 390)
} else {
    0.95  // Old: 95% reduction (PDF line 263-264)
};
let f_i_raw = min_fee_percentage * f_r;
let f_i = round_to_significant_digits(f_i_raw, config.fee_rounding_digits);
```
- ✅ **CORRECT**: Uses 100% (1.0) for new rules, 95% (0.95) for old rules
- ✅ **CORRECT**: Applies fee rounding

**JavaScript Implementation (index.html:1020-1023):**
```javascript
let f_R = R_Base * T_R / (M_L * M_L);
let f_I = 0.95 * f_R;
if (f_I < 1e-12) f_I = 1e-12;
```
- ❌ **INCORRECT**: ALWAYS uses 95% reduction, doesn't check scaling rules
- ❌ **MISSING**: No fee rounding to 2 significant digits

---

## 3. Transitional Considerations - PDF Lines 729-775

### 3.1 Pre-Fork Block Scaling
**Specification (PDF lines 765-774):**
```
For blocks before the hard fork:
M_B = M_B_old * (Z_M / Z_M_old)
```

**WASM Implementation (lib_core.rs:100-103):**
```rust
fn apply_transitional_scaling(m_b_old: i64, z_m_old: i64, z_m_new: i64) -> i64 {
    (m_b_old as f64 * (z_m_new as f64 / z_m_old as f64)) as i64
}
```
- ✅ **CORRECT**: Function exists but appears unused
- ⚠️ **WARNING**: Function is defined but not called in main simulation loop

**JavaScript Implementation:**
- ❌ **MISSING**: No transitional scaling implementation

---

## 4. Fee Rounding - PDF Lines 777-812

**Specification (PDF lines 804-812):**
```
Wallet fees rounded to 2 significant digits
Examples:
27810 → 28000
37.94 → 38
0.5555 → 0.56
0.002342 → 0.0023
```

**WASM Implementation (lib_core.rs:88-98):**
```rust
fn round_to_significant_digits(value: f64, digits: u32) -> f64 {
    if value == 0.0 || !value.is_finite() {
        return value;
    }
    
    let digits_f64 = digits as f64;
    let exponent = (value.abs().log10().floor() - digits_f64 + 1.0).floor();
    let factor = 10.0_f64.powf(-exponent);
    (value * factor).round() / factor
}
```
- ✅ **CORRECT**: Implements rounding to significant digits
- ✅ **CORRECT**: Applied at line 321

**JavaScript Implementation:**
- ❌ **MISSING**: No fee rounding implementation

---

## 5. Summary of Issues

### Critical Issues (Breaking Specification Compliance)

1. **JavaScript M_S_weight calculation (index.html:999)**
   - Uses `max(M_B, M_L)` instead of just `M_B` for new rules
   - **Fix Required**: Add conditional based on scaling rules

2. **JavaScript M_N calculation (index.html:1003)**
   - Uses `M_N = M_S` instead of `min(max(M_S, M_L), 8*M_L)`
   - **Fix Required**: Implement correct formula for new rules

3. **JavaScript minimum fee percentage (index.html:1022)**
   - Always uses 95% instead of 100% for new rules
   - **Fix Required**: Check scaling rules and use appropriate percentage

### Important Missing Features

4. **JavaScript fee rounding**
   - Missing 2 significant digit rounding
   - **Fix Required**: Implement `roundToSignificantDigits()` function

5. **Transitional scaling**
   - WASM has function but doesn't use it
   - JavaScript completely missing
   - **Fix Required**: Apply transitional scaling to pre-fork blocks

---

## 6. Correct Implementation Reference

### WASM (Rust) - lib_core.rs
- ✅ Z_M configuration
- ✅ M_L calculation with 1.2x growth rate
- ✅ M_S calculation (M_B only for new rules)
- ✅ M_N calculation with min(max(M_S, M_L), 8*M_L)
- ✅ Block size validation with 16*M_L cap
- ✅ Penalty calculation
- ✅ Fee percentage (100% for new, 95% for old)
- ✅ Fee rounding to 2 significant digits
- ⚠️ Transitional scaling defined but unused

### JavaScript - index.html
- ✅ Z_M configuration
- ✅ M_L calculation
- ❌ M_S calculation (incorrect for new rules)
- ❌ M_N calculation (incorrect for new rules)
- ✅ Block size validation
- ✅ Penalty calculation
- ❌ Fee percentage (always 95%)
- ❌ Fee rounding (missing)
- ❌ Transitional scaling (missing)

---

## 7. Recommendations

1. **Immediate Priority**: Fix JavaScript M_N calculation - this is critical for penalty calculations
2. **High Priority**: Fix JavaScript M_S_weight to use M_B only in new rules
3. **High Priority**: Add fee percentage checking in JavaScript
4. **Medium Priority**: Implement fee rounding in JavaScript
5. **Low Priority**: Implement and activate transitional scaling in both engines

---

## 8. Test Cases Needed

Based on PDF specifications, the following test scenarios should be validated:

1. Verify M_N = min(max(M_S, M_L), 8*M_L) with various M_S and M_L values
2. Verify M_S uses only M_B (not max(M_B, M_L)) in new rules
3. Verify minimum fee is 100% of penalty (not 95%) in new rules
4. Verify fee rounding: 27810 → 28000, 0.5555 → 0.56, etc.
5. Verify block size cap includes 16*M_L constraint
6. Verify transitional scaling: M_B_scaled = M_B_old * (625000/300000)
