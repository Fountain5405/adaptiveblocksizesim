#!/usr/bin/env bash
# Reproduce the whole study: tests, every scenario, figures and tables.
# Usage: study/run_all.sh            (from anywhere; uses all cores)
#   MONERO_RPC=http://host:18081 study/run_all.sh   also refreshes data/mainnet_weights.csv.gz
#   PYTHON=/path/to/python (needs matplotlib, numpy, pandas)
set -euo pipefail
cd "$(dirname "$0")"
PY=${PYTHON:-python3}

echo "== frozen-core checks"
(cd ../wasm-sim && cargo test --release --lib)
(cd .. && node tests/parity.mjs)

if [[ -n "${MONERO_RPC:-}" ]]; then
  echo "== fetching the last 100,000 block weights from $MONERO_RPC"
  python3 scripts/fetch_chain.py data/mainnet_weights.csv.gz 100000 || echo "fetch failed; keeping the committed copy"
else
  echo "== MONERO_RPC not set: using the committed data/mainnet_weights.csv.gz (mainnet, heights 3697558-3797557)"
fi

echo "== building"
cargo build --release

echo "== scenarios"
for s in q1_growth q1b_demand_growth q2_surge q3a_spam_full q3b_spam_growth q3d_miner \
         q4_recovery q5_fee_market q5b_daily q6_sensitivity q7_fork q7b_fork_mainnet; do
  ./target/release/study "scenarios/$s.json" --out results
done

echo "== figures and tables"
$PY analysis/analyze.py
