# Study brief: Monero's block weight and fee rules, v16 vs FCMP++ (v17)

You are running a research study on this machine (16 threads, 32 GB RAM). Work autonomously; the
study should take as long as it needs to be thorough. Report what you find faithfully, including
results that are null, surprising, or contradict expectations.

## Setup

```
git clone https://github.com/Fountain5405/adaptiveblocksizesim && cd adaptiveblocksizesim
git checkout -b study-2026
```

Read `RULES.md` first, then `wasm-sim/src/lib_core.rs`. That file implements Monero's block weight,
penalty and fee rules exactly as the reference code does, for v16 (mainnet today, "2021 scaling")
and v17 (the FCMP++ hard fork, "2026 scaling", ArticMine's MoneroScaling2026-03 as implemented in
seraphis-migration/monero `fcmp++-stage`). It has been checked line by line against that code.

**The consensus and fee functions are frozen.** Don't change medians, clamps, penalty, fee levels,
`affordable_txs`, or the order of operations in a block. Anything you add (demand models, attackers,
rule switching, instrumentation) goes around them. Before and after your changes:
`cd wasm-sim && cargo test --release --lib` and `node tests/parity.mjs` must pass. If you refactor
the run loop into a steppable simulator, add a regression test showing it reproduces the current
`run_simulation_core` output exactly for the existing run types on both presets.

Units: one block every 2 minutes, 720 blocks/day, 262,800/year. Rewards and fees in XMR; base reward
is the 0.6 XMR tail emission.

## Build a study harness

Create a Rust crate in `study/`, next to this brief, that depends on `wasm-sim` by path and uses
`wasm_sim::lib_core` (the library already builds as an rlib). Keep extra dependencies such as rayon,
serde and csv out of the wasm crate. It should:

- read scenario definitions (JSON or TOML) and expand parameter grids;
- run scenarios in parallel across all cores, each deterministic from a recorded seed;
- write one summary row per run (CSV) and downsampled time series (e.g. every 100 blocks, plus
  exact values at events), keeping memory per run bounded so 16 runs of 1M+ blocks fit easily;
- record the git commit, scenario file, and seed with every output.

Scenario features to add (outside the frozen core):

1. **Demand models:** constant; step surge (magnitude × duration); repeated daily cycle; steady
   exponential growth (% per year); Poisson arrivals. Transaction weight per scenario, plus an
   optional mix (FCMP++ 1-in/2-out 7,650; 2-in/2-out 9,093; 4-in/2-out 11,851 bytes).
2. **Spam attacker:** submits transactions of a given weight at a chosen fee level and rate for a
   duration, or until a budget in XMR is spent. Track the attacker's spend separately.
3. **Malicious miner:** with hashrate share h, mines its own blocks padded to a target weight with
   zero-fee filler, paying the penalty itself. Track penalties paid.
4. **Rule switch at block K:** run v16, then switch to v17 at the fork, carrying the median windows
   across. Implement both the reference code's behaviour (no rescaling) and the spec's transitional
   rule (pre-fork block weights × 625000/300000 in medians that span the fork) as an option.
5. **Initial state from the real chain (optional):** if a monerod RPC is reachable (env var
   `MONERO_RPC`, e.g. http://127.0.0.1:18081), fetch the last 100,000 block weights and long-term
   weights (`get_block_headers_range`) to initialise the windows. If not, start at steady state and say so.

## Research questions

Answer each with v16 vs v17 side by side, with numbers, plots, and the scenario that produced them.

1. **Growth capacity.** With demand permanently above capacity, how fast can M_L, M_N and the
   maximum block weight grow? Time to reach 2, 5, 10, 50, 100 MB blocks. Compare: miners filling
   blocks (upper bound) vs rational miners with users at each fee level, and with `users_pay_more`.
   Check the analytic bound (M_L rises at most ~clamp ratio per 50,000 blocks) against the runs.
2. **Demand surges.** Spikes of 2×, 5×, 10×, 20×, 50× for 1 hour, 1 day, 1 week. Backlog size,
   how long it takes to clear, longest wait, fee paid per transaction and per byte, which fee level
   clears in time. Does v17's 8× surge cap (vs 50×) hurt legitimate surges, and by how much?
3. **Spam and bloat attacks.** Cost to (a) keep blocks at M_N for 1, 7, 30 days; (b) raise M_L to
   2× and 4×; (c) add N GB to the chain. For the fee-paying spammer and for a malicious miner at
   h = 10%, 30%, 51%. Report XMR spent per GB added and per day sustained. Under v17, how much more
   or less does an attack cost than under v16?
4. **Recovery.** After demand or an attack stops, how fast do M_L, M_N and fees fall back? What does
   the chain look like 1, 3, 6, 12 months later?
5. **Fee market.** Steady-state fee per transaction at demand of 0.5×, 1×, 2×, 5× Z_M. When does the
   wallets' automatic priority choice (lowest, or normal under backlog) fail to get transactions
   included in reasonable time? Is "normal" enough to make blocks grow when demand is genuine?
   Quantify the mechanism that lets a block slightly exceed M_N when fees cover the penalty.
6. **Parameter sensitivity** (Custom mode): Z_M ∈ {300k, 625k, 1M}; long-term growth ∈ {1.1, 1.2,
   1.4, 1.7}; surge factor ∈ {4, 8, 16, 50}; T_R ∈ {3,000, 10,000, 12,500, 20,000}; transaction
   weight ∈ {7,650, 9,093, 11,851}. Map how the answers to questions 1–3 change. Which parameter
   matters most for each?
7. **The fork transition.** Simulate the v16 → v17 switch with pre-fork M_L at 300,000 and at
   hypothetical higher values (e.g. 450k, 600k, 1M). Compare reference-code behaviour (no rescaling)
   with the spec's 625/300 rescaling. Does the missing transitional rule ever change block limits or
   fees in a way that matters? For how many blocks after the fork?

Run a small pilot of each question first, check results for plausibility against hand calculations,
then scale up. Prefer more seeds and finer grids over longer runs once results are stable.

## Deliverables (on branch `study-2026`)

- `study/` crate, scenario files, and a `study/run_all.sh` that reproduces everything.
- `study/REPORT.md`: summary of findings up front (one short paragraph per question), then methods,
  results with figures (PNG or SVG, generated by a script in the repo), limitations, and exact
  commands to reproduce. State assumptions plainly: equal-weight transactions within a class,
  fees evaluated at inclusion, the miner and wallet models in RULES.md.
- Data: summary CSVs committed; large time series compressed, or regenerated by `run_all.sh` if over
  ~50 MB.
- Commit as you go. Push the branch if this machine has credentials; otherwise leave it local and
  say so.

Don't present findings as positions of the Monero project, and flag anything that would be worth
raising with the people implementing the FCMP++ scaling (jeffro256, ArticMine, MRL), with the
evidence for it.
