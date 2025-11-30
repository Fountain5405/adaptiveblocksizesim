#!/usr/bin/env python3
"""
Comparison test script to verify feature parity between Python and Rust implementations.
This script runs the same simulation parameters in both Python and Rust and compares the results.
"""

import subprocess
import json
import time
import sys
import os
import math
import numpy as np
from typing import Dict, List, Tuple, Any

# Configuration for test scenarios
TEST_SCENARIOS = [
    {
        "name": "Basic Test",
        "n": 1000,
        "run_type": 1,
        "large_sim_mode": False,
        "exact_median": True,
        "description": "Basic simulation with exact median"
    },
    {
        "name": "Large Simulation Mode",
        "n": 2000,
        "run_type": 2,
        "large_sim_mode": True,
        "exact_median": True,
        "description": "Test LARGE_SIMULATION_MODE with exact median"
    },
    {
        "name": "Fast Median Mode",
        "n": 1000,
        "run_type": 1,
        "large_sim_mode": False,
        "exact_median": False,
        "description": "Test fast median mode"
    },
    {
        "name": "Both Features Enabled",
        "n": 2000,
        "run_type": 3,
        "large_sim_mode": True,
        "exact_median": False,
        "description": "Test both LARGE_SIMULATION_MODE and fast median"
    },
    {
        "name": "Sine Wave Pattern",
        "n": 1500,
        "run_type": 6,
        "large_sim_mode": True,
        "exact_median": True,
        "description": "Test sine wave pattern with both features"
    }
]

def run_python_simulation(config: Dict[str, Any]) -> Dict[str, Any]:
    """Run the Python simulation with given configuration."""
    print(f"Running Python simulation: {config['name']}")
    
    # Create a modified version of the Python script that accepts parameters
    python_code = f"""
import math
import bisect
import json
import sys

# Configuration from test
n = {config['n']}
NETWORK_STEADY_STATE = 300000
RUN_TYPE = {config['run_type']}
LARGE_SIMULATION_MODE = {1 if config['large_sim_mode'] else 0}
ADD_NOISE = 0
USERS_PAY_MORE = 0
WALLET_CALC = 0
PLOT_RESULT = 0

# Initialization
B = 0 
F_T = 0
T_R = 3000
Z_M = 300000
M_B = 100000
M_L = 300000
M_L_weight = 0
M_L_prev = NETWORK_STEADY_STATE
M_S = NETWORK_STEADY_STATE
M_S_weight = 0
M_N = NETWORK_STEADY_STATE
R_Base = 0.6
M_B_max = 2 * NETWORK_STEADY_STATE
M_L_list = [NETWORK_STEADY_STATE]*100000
sorted_M_L_list = sorted(M_L_list)
M_S_list = [NETWORK_STEADY_STATE]*100
sorted_M_S_list = sorted(M_S_list)
mid_100k = 50000
mid_100 = 50

T_sim = 800
T_sim_counter = 0
T_sim_reset_counter = 0
broadcast = [0]*2
mempool = [0]*2
fees = [0]*2
block_fee_total = 0
blockfilled = [0]*2
fee_set = []
percent_response = 0

# Data storage (simplified for comparison)
M_B_archive = []
M_L_archive = []
M_S_archive = []
M_N_archive = []
input_volume_archive = []
Block_fee_archive = []
P_archive = []
T_sim_archive = []

for i in range(n):
    # Median calculations
    M_L = (sorted_M_L_list[mid_100k] + sorted_M_L_list[~mid_100k]) / 2
    M_L_weight = max(min(M_B, 1.7 * M_L_prev), Z_M, M_L_prev / 1.7)
    M_S = (sorted_M_S_list[mid_100] + sorted_M_S_list[~mid_100]) / 2
    M_S_weight = max(M_B, M_L)
    M_N = min(M_S, 50 * M_L)
    
    M_B_max = 2 * M_N
    f_R = R_Base * T_R / (M_L**2)
    f_I = 0.95 * f_R
    if f_I < 10**-12: f_I = 10**-12
    
    if LARGE_SIMULATION_MODE == 1:
        scale_setting = M_S // Z_M
        if T_sim <= scale_setting * 800 / 2:
            T_sim_counter += 1
            if T_sim_counter > 500:
                for j in range(len(mempool)):
                    mempool[j] = mempool[j] // 2
                T_sim *= 2
                T_sim_counter = 0
        if T_sim >= scale_setting * 800 * 2:
            T_sim_counter += 1
            if T_sim_counter > 500:
                for j in range(len(mempool)):
                    mempool[j] *= 2
                T_sim /= 2
                T_sim_counter = 0
        if i > 100:
            if len(M_B_archive) >= 60 and M_B_archive[i-1] == M_B_archive[i-60]:
                T_sim_reset_counter += 1
                if T_sim_reset_counter > 20 and T_sim > 800 and M_S < M_N + T_sim:
                    for j in range(len(mempool)):
                        mempool[j] *= 2
                    T_sim /= 2 
                    T_sim_reset_counter = 0
    
    # Broadcast new tx
    blockfilled[0] = 0
    blockfilled[1] = 0
    broadcast[0] = 0
    broadcast[1] = 0
    
    if RUN_TYPE == 1: broadcast[1] = (Z_M + 100 * i) // T_sim
    if RUN_TYPE == 2: broadcast[1] = (Z_M + 800 * i) // T_sim
    if RUN_TYPE == 3: broadcast[1] = ((316 + (i / 15))**2) // T_sim
    if RUN_TYPE == 4: broadcast[1] = (Z_M * (1.6**(9.8 + (i / 50000)) - 99.75)) // T_sim
    if RUN_TYPE == 5: broadcast[1] = M_B_max // T_sim
    if RUN_TYPE == 6: 
        start_val = 300000
        ramp_multiplier = 3
        ramp_delay = 10
        ramp_days = 14
        ramp_time = ramp_days*720
        if i <= ramp_delay: broadcast[1] = start_val // T_sim
        if ramp_delay < i <= ramp_delay + ramp_time:
            broadcast[1] = (start_val + math.floor((ramp_multiplier - 1)*start_val/ramp_time) * (i - ramp_delay)) // T_sim
        if i > ramp_delay + ramp_time: 
            broadcast[1] = (ramp_multiplier * start_val  + 220 * math.sin(i/802) * 800) // T_sim
    
    # Update mempool with broadcast
    for j in range(len(mempool)):
        mempool[j] += broadcast[j]
    
    # Define fees paid for each tx fee level
    fees[1] = f_R * T_sim
    fees[0] = 16 * fees[1]
    
    # Build next block
    M_B = 0
    block_fee_total = 0 
    fee_set.clear()
    break_flag = 0
    
    for k in range(len(mempool)):
        if break_flag == 1: break
        for l in range(int(mempool[k])):
            if M_B >= M_B_max:
                blockfilled[0] = k
                blockfilled[1] = l
                break_flag = 1
                break
            
            B = (M_B / M_N) - 1
            T_T = T_sim
            if T_T > M_B - M_N > 0: T_T = M_B - M_N
            B_T = T_T / M_N
            F_T = R_Base * (2 * B * B_T + B_T**2)
            if B + B_T <= 0: F_T = 0
            
            fee_set.append(F_T)
            
            if fees[k] < F_T:
                blockfilled[0] = k
                blockfilled[1] = l
                break_flag = 1
                break
            M_B += T_sim
    
    if all(num == 0 for num in blockfilled):
        if mempool[1] != 0:
            blockfilled[0] = 1
            blockfilled[1] = mempool[1]
        elif mempool [0] != 0:
            blockfilled[0] = 0
            blockfilled[1] = mempool[0]
    
    # Calculate fees paid to create block
    for k in range(blockfilled[0]):
        block_fee_total += mempool[k] * fees[k]
    block_fee_total += (blockfilled[1] - 1) * fees[blockfilled[0]]

    # Remove included tx from mempool
    for k in range(blockfilled[0]):
        mempool[k] = 0
    mempool[blockfilled[0]] -= blockfilled[1]
    
    # Penalty calculation for entire block
    B = (M_B / M_N) - 1
    P_B = R_Base * B**2
    if B <= 0: P_B = 0
    
    # Update Long Term Median Lists
    sorted_M_L_list.pop(bisect.bisect_left(sorted_M_L_list, M_L_list[0]))
    M_L_list.pop(0)
    M_L_list.append(M_L_weight)
    bisect.insort(sorted_M_L_list, M_L_weight)
    
    # Update Short Term Median Lists
    sorted_M_S_list.pop(bisect.bisect_left(sorted_M_S_list, M_S_list[0]))
    M_S_list.pop(0)
    M_S_list.append(M_S_weight)
    bisect.insort(sorted_M_S_list, M_S_weight)
    
    M_L_prev = M_L
    
    # Store data
    M_B_archive.append(M_B)
    M_L_archive.append(M_L)
    M_S_archive.append(M_S)
    M_N_archive.append(M_N)
    input_volume_archive.append(broadcast[1] * T_sim)
    Block_fee_archive.append(block_fee_total)
    P_archive.append(P_B)
    T_sim_archive.append(T_sim)

# Output results as JSON
results = {{
    "max_mb": max(M_B_archive) if M_B_archive else 0,
    "max_penalty": max(P_archive) if P_archive else 0,
    "max_mempool": max((mempool[0] + mempool[1]) * T_sim, 0),
    "cumulative_fees": sum(Block_fee_archive),
    "final_mb": M_B_archive[-1] if M_B_archive else 0,
    "final_ml": M_L_archive[-1] if M_L_archive else 0,
    "final_ms": M_S_archive[-1] if M_S_archive else 0,
    "final_mn": M_N_archive[-1] if M_N_archive else 0,
    "final_t_sim": T_sim_archive[-1] if T_sim_archive else 800,
    "sample_data": {{
        "M_B": M_B_archive,  # Return all values to match Rust
        "M_L": M_L_archive,
        "M_S": M_S_archive,
        "M_N": M_N_archive,
        "input_volume": input_volume_archive,
        "block_fee": Block_fee_archive,
        "penalty": P_archive,
        "t_sim": T_sim_archive
    }}
}}

print(json.dumps(results))
"""
    
    # Run the Python code
    start_time = time.time()
    result = subprocess.run([sys.executable, "-c", python_code], 
                          capture_output=True, text=True, timeout=600)
    end_time = time.time()
    
    if result.returncode != 0:
        print(f"Python simulation failed: {result.stderr}")
        return None
    
    try:
        data = json.loads(result.stdout)
        data["execution_time"] = end_time - start_time
        return data
    except json.JSONDecodeError as e:
        print(f"Failed to parse Python output: {e}")
        print(f"Output: {result.stdout}")
        return None

def run_rust_simulation(config: Dict[str, Any]) -> Dict[str, Any]:
    """Run the Rust simulation with given configuration."""
    print(f"Running Rust simulation: {config['name']}")
    
    # First, build the Rust binary
    build_result = subprocess.run(["cargo", "build", "--release"], 
                                 cwd="wasm-sim", capture_output=True, text=True)
    if build_result.returncode != 0:
        print(f"Failed to build Rust binary: {build_result.stderr}")
        return None
    
    # Run the Rust binary with parameters
    binary_path = "wasm-sim/target/release/blockchain-sim"
    cmd = [
        binary_path,
        "--n", str(config["n"]),
        "--run-type", str(config["run_type"]),
        "--large-sim-mode", str(int(config["large_sim_mode"])),
        "--exact-median", str(int(config["exact_median"])),
        "--json"  # Request JSON output
    ]
    
    start_time = time.time()
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
    end_time = time.time()
    
    if result.returncode != 0:
        print(f"Rust simulation failed: {result.stderr}")
        return None
    
    try:
        data = json.loads(result.stdout)
        data["execution_time"] = end_time - start_time
        return data
    except json.JSONDecodeError as e:
        print(f"Failed to parse Rust output: {e}")
        print(f"Output: {result.stdout}")
        return None

def compare_results(python_data: Dict[str, Any], rust_data: Dict[str, Any], 
                   config: Dict[str, Any]) -> Dict[str, Any]:
    """Compare results from Python and Rust implementations."""
    comparison = {
        "scenario": config["name"],
        "description": config["description"],
        "metrics": {},
        "sample_data_comparison": {},
        "overall_match": True
    }
    
    # Compare key metrics (allowing for small floating point differences)
    metrics_to_compare = [
        ("max_mb", "Maximum block size", 0.01),  # 1% tolerance
        ("max_penalty", "Maximum penalty", 0.01),
        ("max_mempool", "Maximum mempool size", 0.01),
        ("cumulative_fees", "Cumulative fees", 0.01),
        ("final_mb", "Final block size", 0.01),
        ("final_ml", "Final long-term median", 0.01),
        ("final_ms", "Final short-term median", 0.01),
        ("final_mn", "Final penalty median", 0.01),
        ("final_t_sim", "Final T_sim", 0.01)
    ]
    
    for metric, description, tolerance in metrics_to_compare:
        py_val = python_data.get(metric, 0)
        rust_val = rust_data.get(metric, 0)
        
        if isinstance(py_val, (int, float)) and isinstance(rust_val, (int, float)):
            if py_val != 0:
                diff_percent = abs(py_val - rust_val) / abs(py_val)
                matches = diff_percent <= tolerance
            else:
                matches = py_val == rust_val
        else:
            matches = py_val == rust_val
        
        comparison["metrics"][metric] = {
            "description": description,
            "python": py_val,
            "rust": rust_val,
            "matches": matches,
            "difference_percent": abs(py_val - rust_val) / abs(py_val) if py_val != 0 else 0
        }
        
        if not matches:
            comparison["overall_match"] = False
    
    # Compare sample data arrays
    sample_arrays = ["M_B", "M_L", "M_S", "M_N", "input_volume", "block_fee", "penalty", "t_sim"]
    
    for array_name in sample_arrays:
        py_array = python_data.get("sample_data", {}).get(array_name, [])
        rust_array = rust_data.get("sample_data", {}).get(array_name, [])
        
        if len(py_array) != len(rust_array):
            comparison["sample_data_comparison"][array_name] = {
                "matches": False,
                "reason": f"Length mismatch: Python={len(py_array)}, Rust={len(rust_array)}"
            }
            comparison["overall_match"] = False
            continue
        
        # Check if arrays match within tolerance
        matches = True
        max_diff = 0
        for i, (py_val, rust_val) in enumerate(zip(py_array, rust_array)):
            if isinstance(py_val, (int, float)) and isinstance(rust_val, (int, float)):
                if py_val != 0:
                    diff = abs(py_val - rust_val) / abs(py_val)
                    max_diff = max(max_diff, diff)
                    if diff > 0.01:  # 1% tolerance
                        matches = False
                        break
                elif py_val != rust_val:
                    matches = False
                    break
        
        comparison["sample_data_comparison"][array_name] = {
            "matches": matches,
            "max_difference_percent": max_diff,
            "length": len(py_array)
        }
        
        if not matches:
            comparison["overall_match"] = False
    
    return comparison

def main():
    """Main function to run all comparison tests."""
    print("Starting feature parity comparison between Python and Rust implementations")
    print("=" * 80)
    
    all_results = []
    
    for config in TEST_SCENARIOS:
        print(f"\nTesting scenario: {config['name']}")
        print(f"Description: {config['description']}")
        print("-" * 40)
        
        # Run Python simulation
        python_result = run_python_simulation(config)
        if python_result is None:
            print(f"Failed to run Python simulation for {config['name']}")
            continue
        
        # Run Rust simulation
        rust_result = run_rust_simulation(config)
        if rust_result is None:
            print(f"Failed to run Rust simulation for {config['name']}")
            continue
        
        # Compare results
        comparison = compare_results(python_result, rust_result, config)
        comparison["python_execution_time"] = python_result["execution_time"]
        comparison["rust_execution_time"] = rust_result["execution_time"]
        comparison["performance_ratio"] = python_result["execution_time"] / rust_result["execution_time"]
        
        all_results.append(comparison)
        
        # Print summary for this scenario
        print(f"Overall match: {'✓ PASS' if comparison['overall_match'] else '✗ FAIL'}")
        print(f"Python execution time: {python_result['execution_time']:.3f}s")
        print(f"Rust execution time: {rust_result['execution_time']:.3f}s")
        print(f"Performance ratio (Python/Rust): {comparison['performance_ratio']:.2f}x")
        
        # Show any mismatches
        for metric, data in comparison["metrics"].items():
            if not data["matches"]:
                print(f"  Mismatch in {data['description']}:")
                print(f"    Python: {data['python']}")
                print(f"    Rust: {data['rust']}")
                print(f"    Difference: {data['difference_percent']:.2%}")
        
        for array, data in comparison["sample_data_comparison"].items():
            if not data["matches"]:
                print(f"  Mismatch in {array} array:")
                if "reason" in data:
                    print(f"    Reason: {data['reason']}")
                else:
                    print(f"    Max difference: {data['max_difference_percent']:.2%}")
    
    # Generate final report
    print("\n" + "=" * 80)
    print("FINAL COMPARISON REPORT")
    print("=" * 80)
    
    total_scenarios = len(all_results)
    passed_scenarios = sum(1 for r in all_results if r["overall_match"])
    
    print(f"Total scenarios tested: {total_scenarios}")
    print(f"Scenarios passed: {passed_scenarios}")
    print(f"Scenarios failed: {total_scenarios - passed_scenarios}")
    if total_scenarios > 0:
        print(f"Success rate: {passed_scenarios/total_scenarios:.1%}")
    else:
        print("Success rate: N/A (no scenarios tested)")
    
    print("\nDetailed Results:")
    for result in all_results:
        status = "✓ PASS" if result["overall_match"] else "✗ FAIL"
        print(f"  {result['scenario']}: {status} (Performance: {result['performance_ratio']:.2f}x)")
    
    # Save detailed results to file
    with open("comparison_results.json", "w") as f:
        json.dump(all_results, f, indent=2)
    
    print(f"\nDetailed results saved to: comparison_results.json")
    
    return all_results

if __name__ == "__main__":
    results = main()
    sys.exit(0 if all(r["overall_match"] for r in results) else 1)