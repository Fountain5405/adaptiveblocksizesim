# Monero's block weight and fee rules, v16 vs FCMP++ (v17): a simulation study

*Study branch `study-2026`, November 2026. The rules simulated are the reference code's (RULES.md): v16 =
monero-project/monero `master` ("2021 scaling"), v17 = seraphis-migration/monero `fcmp++-stage` (ArticMine's
MoneroScaling2026-03). This is an independent simulation study. Nothing here is a position of the Monero project.
Every number comes from the scenario files in `scenarios/`. `run_all.sh` regenerates all of them in about two minutes.*

## Summary of findings

**Q1 Growth capacity.** If demand stays above capacity, the long-term median M_L grows at exactly the clamp ratio
per 50,000 blocks (×1.7 for v16, ×1.2 for v17). This happened in every run, and no run exceeded the bound. The
predicted times to reach 50 MB and 100 MB blocks matched the simulated times to the day. The fee users pay barely
matters for this. Once there is a backlog, even minimum-fee transactions push rational miners slightly past M_N.
The short-term median then ratchets up until it hits the surge cap, and M_L grows at the clamp's maximum rate
from then on. The fee level only changes how fast the first ramp happens:

* **v17:** the maximum block weight (2·M_N) reaches 10 MB in 0.3 days if miners fill blocks, 1.7 days with
  wallets on their default priority, and 6.2 days at the minimum fee. 50 MB takes 625 days and 100 MB 903 days,
  whatever the fee.
* **v16:** 10 MB takes 0.35, 3.5 and 13 days on the same three bases. 50 MB takes 69 days and 100 MB 208 days.
* **Sustained growth:** v17 sustains at most ×2.6 per year in M_L. Starting from today's demand (38 tx/block),
  the chain keeps up with demand growing 200% a year (×3 per year, with M_N carrying 2.3·M_L after 5 years). At
  400% a year it falls behind permanently.

**Q2 Demand surges.** Surges up to 2× today's demand are absorbed. Larger ones queue, because wallets on default
priority pay only "normal" (4× the minimum), and at that level M_N follows the backlog slowly.

* **One-day 5× surge:** mean wait 274 blocks (9 h) on v17 vs 131 on v16.
* **Why v17 waits longer:** it isn't the new rules. With v16 rules and the same 9,093-byte FCMP++ transactions
  the mean wait would be 716 blocks. v17 waits longer than today's chain because its 625 kB penalty-free zone
  holds 69 FCMP++ transactions, while today's 300 kB holds about 120 of today's.
* **What helps:** paying 64× the minimum, or wallets that raise their fee during a backlog ("users pay more"),
  brings waits down to a few blocks.
* **The 8× surge cap:** it doesn't affect surges below about 8·Z_M of demand (about 550 FCMP++ tx/block, 14×
  today's). For 1-hour surges it makes no difference at all. It hurts sustained large surges: a one-week 50×
  surge gives a mean wait of 4,988 blocks (6.9 days) under v17 vs 233 blocks with a 50× cap. For one day at 50×,
  the means are 1,461 vs 983 blocks.

**Q3 Spam and bloat.** Attacks on v17 cost the same or more than on v16, and much more at scale.

* **Keeping blocks full** at M_N without letting a backlog form costs 3.9 XMR/day under v17 vs 3.0 under v16
  (×1.33). Both pay 20 XMR per GB.
* **A spammer that keeps a backlog instead** triggers the ratchet of Q1. Blocks then grow to the surge cap. Over 30 days the attack
  costs 15–36× more per day at the lowest fee, and 67–220× more if the spam pays the wallets' auto level.
* **Doubling M_L** costs a minimum-fee spammer 1,440 XMR over 281 days on v17 vs 687 XMR over 142 days on v16.
  Quadrupling costs 2,358 vs 866 XMR.
* **Adding 1 TB** costs at least about 12,500 XMR under v17 vs about 1,070 XMR under v16. The cheapest strategy
  is to grow the limit first, because the minimum fee falls as 1/M_L².
* **A malicious miner below 50% hashrate** cannot move M_L at all, since both windows are medians. It can pad its
  own blocks up to M_N for free: 37 GB/yr at 51% on v17.
* **A miner at 51%** padding to the clamp multiplied M_L by 1.44 in a year on v17 (1,481 XMR) vs 2.89 on v16
  (17,857 XMR).

**Q4 Recovery.** M_N drops back within about 50 blocks, but M_L stays at its peak for 50,000 blocks and then falls
by at most the same clamp ratio per 50,000 blocks.

* **After a year of maximum growth,** both rule sets need about 350 days to return to the penalty-free zone.
  Six months after demand stops, M_L is still 1.7× its starting value on v17 and 4.9× on v16.
* **Fees stay low while M_L is high.** The minimum fee per byte is 0.16× its starting value at the end of a
  growth year on v17 and 0.005× on v16, which makes follow-up spam cheaper. After a 2-year clamp spam campaign,
  M_L is still 1.5 MB (v17) or 1.15 MB (v16) 12 months later.

**Q5 Fee market.**

* **Below about 0.8·Z_M** wallets pay the minimum (v17: 0.00018 XMR per 2-in/2-out transaction) and nothing
  waits.
* **From Z_M upward** wallets switch to "normal" and pay 0.0005–0.0007 XMR per transaction under v17
  (0.00007–0.0002 under v16).
* **"Normal" is enough to make blocks grow when demand is real.** In about half of all blocks, rational miners go
  2–5% past M_N, and M_N follows demand.
* **Where auto priority fails** is a sudden step in demand:
  * Going to 2·Z_M gives a p99 wait of 18 blocks on v17 (291 on v16).
  * Going to 5·Z_M gives 333 blocks (1,012 on v16).
  * A daily demand cycle around Z_M gives mean waits of 20–80 blocks and some low-fee transactions waiting
    300–770 blocks. M_N shrinks in each daily trough, and transactions sent at the lowest level just before a
    backlog are overtaken by every later "normal" one.
* **Escalating the fee during a backlog** (users-pay-more) removes these waits for 5–8× the fee.
* **The over-M_N mechanism,** quantified: with a deep pool, a block can be as large as 1.017·M_N at the minimum
  fee, 1.047 at normal, 1.17 / 1.65 at 16× / 64×, and 2·M_N at f_X (v17, M_N = M_L = Z_M). It reaches 2·M_N at
  lower levels as M_N/M_L grows. Stressnet shows it live: 195 blocks were 0.1–4.1% over its 10 MB penalty-free
  size.

**Q6 Parameter sensitivity** (576 rule sets; share of variance of the log metric):

| Parameter | What it dominates |
|---|---|
| Clamp ratio | time to double M_L (100%), the cost of a 51% miner doing it (75%) |
| Surge factor | time to reach 10 MB (57%), clearing of a 50× surge (47%) |
| T_R | cost of keeping blocks full (79%), waits in a 10× surge (56%), cost of doubling M_L by spam (32%) |
| Z_M | second-order everywhere (≤ 28%) |
| Transaction weight | small, except where it exceeds T_R |

When the transaction weight is above T_R, a minimum-fee transaction can't pay to cross M_N at all. Blocks then
don't grow at the minimum fee, and a minimum-fee spammer can't raise M_L. With T_R = 3,000 the spammer never did.

**Q7 The fork transition.** The reference code does not implement the spec's transitional rule. Whether that
matters depends on M_L at the fork:

* **At today's mainnet state** (M_L at the 300,000 floor, long-term weights 176k–456k), it makes no difference at
  ordinary demand: the runs are bit-identical.
* **With high post-fork demand it diverges, even from the real windows.** Scaling the short-term window changes
  M_N from 39 blocks after the fork. Scaling the upper tail of the long-term weights gives up to 7.5% higher M_L
  once post-fork blocks fill half the window, about 46,000 blocks after the fork.
* **If pre-fork M_L is above 300,000, it matters a lot.**
  * Without rescaling, M_L at the fork is 625k instead of 938k for X = 450k, and 1.0M instead of 2.08M for
    X = 1M.
  * The minimum fee per byte jumps 2.3× (450k) to 4.2–4.5× (≥ 600k), where with rescaling it would stay nearly
    continuous.
  * These differences last the whole 200,000 blocks simulated when demand stays high, and 146,000–196,000 blocks
    when demand falls back.

## Worth raising with the FCMP++ scaling implementers (jeffro256, ArticMine, MRL)

1. **The transitional rule is missing from `fcmp++-stage`, and "it only matters above 300k" isn't quite right
   (Q7).** From the real mainnet windows, post-fork demand above about 2·Z_M produces different limits within 39
   blocks of the fork, and M_L up to 7.5% higher, once the rescaled pre-fork tail or the short-term window takes
   part. For pre-fork M_L above 300k the differences are 2–4.5× in fees and up to 2.08× in limits, for over
   100,000 blocks. Either implement the rule, or amend the spec to match the code. Evidence: `tables/q7.md`,
   `figures/q7_fork.png`. Note that this study applied the rescale to every median window (long-term and
   short-term, consensus and wallet). If the spec means only the long-term window, the early differences in the
   x3 case go away and the long-term ones remain.
2. **Fee level does not throttle growth, only the clamp does (Q1, Q3).** With a persistent backlog, even at the
   minimum fee, M_N reaches the 8× cap within about 6 days, because v17 fee levels depend only on M_L. M_L then
   grows at the full clamp rate. That is probably intended. It does mean that the cost of growing M_L is simply
   the minimum fee × bytes. Evidence: `tables/q1.md` (identical M_L trajectories across L0–L4) and the analytic
   table in `tables/q5.md`.
3. **The penalty-free zone in transactions (Q2).** 625 kB holds 69 2-in/2-out FCMP++ transactions, against about
   120 of today's transactions in 300 kB. After the fork, moderate surges therefore queue about twice as long as
   they would today, although at equal transaction weight v17 does better than v16 (274 vs 716 blocks for a
   one-day 5× surge).
4. **The 8× surge cap (Q2)** is invisible for surges up to about 8·Z_M. A sustained 20–50× surge costs 6–21×
   longer mean waits than a 50× cap would. This is a trade-off against Q3: under 50× a spammer's ratchet reaches
   much larger blocks.
5. **T_R and transaction weights (Q6).** Minimum-fee growth needs T_R ≥ transaction weight. 12,500 clears the
   standard FCMP++ weights (7,650 / 9,093 / 11,851), but only narrowly for 4-in/2-out. Larger transactions (up to
   the 178,063 limit) can't push blocks past M_N at the minimum fee.
6. **Wallet behaviour (Q2, Q5)** is wallet-side, but it decides user experience. Auto priority tops out at
   "normal", and there is no fee bumping, so low-fee transactions starve behind later normal-fee ones for hundreds
   of blocks. A users-pay-more style escalation removed almost all waiting in every scenario, at 5–8× the fee.
   The 1,000-block wallet grace period did not cause trouble in any run.
7. **Stressnet's 10 MB penalty-free zone.** Its long-term weights are all 8,333,333 = 10 MB × 5/6, so
   block-size dynamics observed on stressnet are not those of mainnet parameters. Its transaction weights do
   match `get_fcmp_pp_transaction_weight_v1` exactly.

## Methods

### Simulator and what was frozen

`wasm-sim/src/lib_core.rs` implements the consensus and fee rules. None of its functions were changed. All the
study's work happens around them:

* **Stepper.** `wasm-sim/src/lib_core/step.rs`, a child module that can call the private functions unchanged,
  splits the run loop into `begin_block` (medians, limits, fee levels) and `finish_block` (arrivals, block
  building, window updates). The order of operations is the core's.
* **Additions,** all off by default:
  * several transaction classes;
  * FIFO pool queues per fee level and class, which give exact wait times;
  * attacker-owned transactions;
  * miner padding;
  * a rule switch, with or without rescaling;
  * initialisation from a real chain;
  * pool expiry.
* **Regression.** `stepper_reproduces_core_exactly` checks that the stepper reproduces `run_simulation_core` bit
  for bit, including floating-point fees, in 90 configurations: run types 1–6 × v16 / v17 / Custom × rational
  miners, noise, users-pay-more and fixed levels.
* **Checks.** `cargo test --release --lib` (12 tests) and `node tests/parity.mjs` (all PASS) pass before and
  after the changes.

### Harness

`study/` is a Rust crate that depends on `wasm-sim` by path.

* **Scenarios** are JSON files, expanded as cases (optionally crossed) × parameter grid × seeds.
* **Runs** go in parallel on all 12 cores (rayon). Each run is deterministic from its recorded seed.
* **Outputs:**
  * `summary.csv`: one row per run;
  * `events.csv`: exact values at window starts and ends, thresholds, the fork, and +1/3/6/12 months;
  * `runs.jsonl`: the full spec of each run;
  * `meta.json`: git commit, scenario text, seeds, timing;
  * gzipped series sampled every N blocks.
* **Memory** per run is bounded: series are streamed, and wait times go into a fixed-size histogram.
  The longest scenario (1.6 M blocks × 48 runs) takes 2 s, and the whole study takes about 90 s.

| Model | How it works |
|---|---|
| Demand | constant (tx/block, bytes, or × Z_M); step surge (× multiplier for a duration); daily sine cycle; exponential growth (%/yr); "fill" (k × maximum block weight, i.e. permanently above capacity, optionally between two blocks); Poisson arrivals (exact sampler) or deterministic with fractional carry; a weighted transaction mix |
| Spam attacker | a fixed fee level, `auto` or `top`. Modes: `fill_to` (each block, top the pool up to max(a·M_N, b·M_L) or the clamp ratio × M_L, so no backlog forms), `topup` (keep k·M_N of spam waiting), `rate`. Optional duration or XMR budget. Spend is counted when a transaction is included, at the fee then in force. |
| Malicious miner | each block is the attacker's with probability h (separate random stream). It builds the block like an honest miner, then pads with zero-fee filler to M_N, the clamp ratio × M_L, or 2·M_N. Its cost is the penalty above what the unpadded block would have paid. |
| Rule switch | v16 → v17 at block K, windows carried across. Reference behaviour: no rescaling. Spec option: every weight in every window (long- and short-term, consensus and wallet) × 625,000/300,000 once at the fork, which equals scaling pre-fork weights in any median spanning the fork. Wallet grace (10 → 1,000) and the number of fee levels change at the fork. |
| Initial state | steady (windows filled with Z_M), or the last 100,000 mainnet blocks: heights 3,697,558–3,797,557 from the mainnet node at 192.168.1.76 (`scripts/fetch_chain.py`, committed as `data/mainnet_weights.csv.gz`). Only Q7b uses the mainnet start. The other questions start at steady state, which equals today's mainnet state: M_L = Z_M = 300,000. |
| Pool lifetime | off except in Q4, where pool transactions expire after 2,160 blocks (3 days, like monerod). Without expiry, the enormous backlog left by "demand above capacity" keeps blocks full for months after demand stops. |

### Baseline numbers taken from real chains (`tables/chains.md`)

* **Mainnet** averaged 38.4 transactions per block and about 2,490 bytes per transaction over the last 100,000
  blocks. The study's baseline demand is therefore 38 tx/block, with 2,500-byte transactions for v16.
* **v17 runs** use 9,093-byte transactions (FCMP++ 2-in/2-out), plus one mix of 25% 7,650 / 50% 9,093 / 25%
  11,851.
* **Stressnet** (testnet with the FCMP++ fork, version 18 from height 3,103,520, node at 192.168.1.22:28089):
  one-transaction blocks minus the coinbase are exactly 9,093 or 7,650 bytes.

### Assumptions (as in RULES.md)

* Transactions within a class have equal weight.
* Fees are the wallet levels in force when the transaction is *included*.
* Miners are rational: they take the highest level first, and add a transaction only if its fee covers the
  penalty it adds (closed form `affordable_txs`), unless `fill`.
* Wallets follow `wallet2::adjust_priority`: the lowest level, or "normal" under a backlog. Users-pay-more moves a
  growing share two levels up.
* The reward is the 0.6 XMR tail emission.
* Not modelled: the coinbase reserve, the pool size limit, per-transaction weight limits, and network or
  verification limits.

### Pilots and hand checks

Every question was first run as a small pilot (`scenarios/pilot/`).

* **Fill pilot:** v16 M_L went 300k → 510k → 867k at blocks 50k and 100k, and v17 went 625k → 750k → 900k,
  exactly ×1.7 and ×1.2. M_N sat at 50·M_L and 8·M_L. Blocks at 2·M_N paid the full 0.6 XMR penalty.
* **Spam pilot:** 7 days of blocks filled to M_N under v17 cost 27.5 XMR. By hand: 2.0e-8 XMR/byte (the minimum
  level, rounded up to 2 significant digits) × (625,000 − 345,500 user bytes) × 5,040 blocks ≈ 27.6.
* **Analytic M_L bound and growth times:** checked in Q1.
* **Over-M_N table:** computed from the same formula `affordable_txs` uses. Its overshoots match the simulated
  mean overshoots in Q5.

## Results

### Q1 Growth capacity (`scenarios/q1_growth.json`, `scenarios/q1b_demand_growth.json`, `tables/q1.md`)

![Q1 growth](figures/q1_growth.png)

![Q1 ramp](figures/q1_ramp_by_level.png)

Days for the maximum block weight to reach a size, with demand = the maximum block weight every block
(selected rows, full table in `tables/q1.md`):

| rules | miners / fee | 2 MB | 5 MB | 10 MB | 50 MB | 100 MB | M_L ×2 | M_L ×10 |
|---|---|---|---|---|---|---|---|---|
| v16 | fill (upper bound) | 0.14 | 0.28 | 0.35 | 69.4 | 208 | 139 | 347 |
| v16 | auto (normal under backlog) | 2.6 | 3.2 | 3.5 | 70.9 | 209 | 139 | 348 |
| v16 | lowest fee (L0) | 9.5 | 12.1 | 13.0 | 74.6 | 210 | 140 | 349 |
| v16 | auto + users pay more | 0.16 | 0.30 | 0.37 | 69.5 | 208 | 139 | 347 |
| v17 | fill (upper bound) | 0.07 | 0.21 | 0.28 | 625 | 903 | 278 | 903 |
| v17 | auto | 0.71 | 1.47 | 1.75 | 625 | 903 | 278 | 903 |
| v17 | lowest fee (L0) | 2.6 | 5.3 | 6.2 | 626 | 904 | 279 | 903 |
| v17 | auto + users pay more | 0.09 | 0.23 | 0.30 | 625 | 903 | 278 | 903 |

* **Analytic bound.** Over all 1.6 M-block runs, the largest M_L(b + 50,000) / M_L(b) was exactly 1.7000 (v16)
  and 1.2000 (v17), never more.
* **Predicted times.** With M_L stepping ×clamp every 50,000 blocks and the maximum weight at 2·surge·M_L, the
  predicted times are 69.4 / 208.3 days (v16, 50 / 100 MB) and 625.0 / 902.8 days (v17). The simulations hit
  them exactly.
* **Demand only 10% above M_N** instead of at the maximum gives the same long-term times (`tables/q1.md`).
* **Exception: transactions larger than T_R.** v16 with 9,093-byte transactions at the minimum fee does not grow
  at all (M_L 300,069 after 6 years). A minimum-fee transaction larger than T_R (3,000) can't pay the penalty for
  crossing M_N. This is the T_R-vs-weight effect of Q6.
* **v16 at very large sizes.** At L0/L1, v16 eventually falls below the clamp bound (left panel, dotted). The
  overshoot allowed at a fixed fee shrinks relative to M_N as M_L grows, so after a step the short-term median
  can no longer climb back above 1.7·M_L.

Exponential demand growth from 38 tx/block over 5 years (8 seeds, Poisson):

| growth/yr | demand at year 5 (tx/block) | v17 M_L / M_N at 5 yr (MB) | v17 p99 wait | v16 M_L / M_N (MB) | v16 p99 wait |
|---|---|---|---|---|---|
| 50% | 289 | 2.4 / 2.7 | 1 | 0.67 / 0.74 | 1 |
| 100% | 1,216 | 9.7 / 11.2 | 1 | 2.7 / 3.1 | 1 |
| 200% | 9,234 | 36.6 / 84.2 | 0.2 | 18.7 / 23.1 | 1 |
| 400% | 118,750 | 45.8 / 366 | 105,125 (falls behind) | 85.6 / 93 | 85,226 (falls behind; keeps up with users-pay-more) |

The v17 clamp allows ×1.2^5.26 ≈ ×2.6 per year in M_L. Faster growth is carried by the 8× surge headroom for a
while: M_N/M_L reached 2.3 after 5 years at +200% a year.

### Q2 Demand surges (`scenarios/q2_surge.json`, 16 seeds per point, `tables/q2.md`, `tables/q2_surge_all.csv`)

![Q2 waits](figures/q2_surge_wait.png)

![Q2 levels](figures/q2_surge_levels.png)

![Q2 example](figures/q2_surge_example.png)

All surges start from 38 tx/block (Poisson). "Wait" is counted in blocks (2 min each) for the transactions that
arrive during the surge.

Mean wait (blocks), one-day surge, auto priority:

| surge | v16 (2,500 B) | v16 with 9,093 B | v17 | v17 mix | v17 with 50× cap |
|---|---|---|---|---|---|
| 2× | 0 | 196 | 1.1 | 1.8 | 1.1 |
| 5× | 131 | 716 | 274 | 255 | 274 |
| 10× | 500 | 1,162 | 549 | 511 | 549 |
| 20× | 1,002 | 1,513 | 836 | 796 | 778 |
| 50× | 1,659 | 1,834 | 1,461 | 1,412 | 983 |

More detail for the one-day surges under v17 (auto):

| surge | peak backlog | clears after the surge ends | max wait | M_N reached | fee per transaction (XMR) |
|---|---|---|---|---|---|
| 5× | 584 MB | 348 blocks | 1,070 | 2.8 MB | 7.0e-4 (normal) |
| 10× | 1.8 GB | 607 blocks | 1,328 | 5 MB (cap) | 7.0e-4 |
| 50× | 11.8 GB | 2,198 blocks | 2,921 | 5 MB (cap) | 7.0e-4 |

* **Which fee level clears in time** (lowest fixed level with p99 wait ≤ 1 day):
  * v17: L0 for 1-hour surges; L1 (4×) for 1-day or 1-week surges up to 10×; L2 (16×) up to 20×; L3 (64×) for a
    1-day 50× surge.
  * Nothing does within a day for a 1-week 50× surge.
  * A p99 within 1 hour needs L3 at 5×, and nothing works above 5×.
  * v16 needs L2 for 10–50× one-day surges.
* **What paying more buys** in a one-day 5× surge on v17, mean wait per level:

  | L0 (1×) | L1 (4×) | L2 (16×) | L3 (64×) | L4 (f_X) |
  |---|---|---|---|---|
  | 501 | 273 | 26 | 1.8 | 0.8 |

  Fees per transaction run from 0.00018 to 0.018 XMR across those levels. Users-pay-more gives 2.7 blocks at an
  average 0.0026 XMR.
* **The 8× cap vs 50×,** auto priority (v17 / with 50× cap):

  | surge | mean wait | clears after end (blocks) |
  |---|---|---|
  | 1 week, 20× | 1,022 / 166 | 1,218 / 0 |
  | 1 week, 50× | 4,988 / 233 | 9,570 / 0 |
  | 1 day, 50× | 1,461 / 983 | 2,198 / 825 |
  | 1 day, 20× | 836 / 778 | 1,004 / 701 |

  At 10× and below, and for every 1-hour surge, the two are identical or within 3%. The cap binds when surge
  demand exceeds 8·M_L = 5 MB per block.

### Q3 Spam and bloat attacks (`scenarios/q3a_spam_full.json`, `q3b_spam_growth.json`, `q3d_miner.json`, `tables/q3.md`)

![Q3 spam](figures/q3_spam.png)

![Q3 miner](figures/q3_miner.png)

Fee-paying spammer, with users at 38 tx/block (8 seeds for (a), 4 for (b, c)):

| goal | v16 | v17 | v17 ÷ v16 |
|---|---|---|---|
| (a) keep blocks at M_N for 1 day (no backlog) | 3.0 XMR | 3.9 XMR | 1.33 |
| (a) … 7 days | 20.7 | 27.6 | 1.33 |
| (a) … 30 days | 88.6 | 117.9 | 1.33 |
| (a) 30 days, keeping a block of backlog at the lowest fee | 3,145 (M_N → 15 MB) | 1,823 (M_N → 5 MB) | 0.58 |
| (b) raise M_L to 2× (lowest fee, fill to clamp × M_L) | 687 XMR, 142 days | 1,440 XMR, 281 days | 2.1 |
| (b) raise M_L to 4× | 866 XMR, 217 days | 2,358 XMR, 559 days | 2.7 |
| (b) same at the top wallet level | 104,590 to 2× | 192,753 to 2× | 1.8 |
| (c) add 10 GB (cheapest strategy) | 201 | 202 | 1.0 |
| (c) add 100 GB | 793 (185 days) | 1,238 (235 days) | 1.6 |
| (c) add 1,000 GB | 1,073 (463 days) | 12,542 (237 days) | 11.7 |
| (c) add 10,000 GB | 5,393 (548 days) | not reachable in 2 years | – |
| 2 years of clamp spam | 1,114 XMR, 3.7 TB, 0.30 XMR/GB | 2,695 XMR, 0.88 TB, 3.05 XMR/GB | – |

* **Why growth makes bloat cheap.** At steady size bloat costs 20 XMR/GB under both rule sets. A spammer who
  first raises M_L pays less per byte, because the minimum fee per byte is R·T_R/M_L².
* **Malicious miner** (16 seeds), padding its own blocks for 1 year:

  | h | target | v16: penalty paid, M_L × | v17: penalty paid, M_L × |
  |---|---|---|---|
  | 10%, 30%, 45% | any | M_L unchanged (median) | M_L unchanged |
  | any | pad to M_N | 0 XMR (free: 5–32 GB/yr) | 0 XMR (7–44 GB/yr) |
  | 51% | clamp × M_L | 17,857 XMR, ×2.89 | 1,481 XMR, ×1.44 |
  | 60% | clamp × M_L | 1,225 XMR, ×4.91 | 103 XMR, ×1.73 |
  | 51% | 2·M_N (whole reward) | 80,428 XMR, ×2.89 | 80,428 XMR, ×1.44 |

  At 60% the attacker's blocks also hold up M_S, so its padding becomes almost penalty-free. In M_L terms, a
  majority miner achieves far less under v17's 1.2 clamp than under 1.7. A fee-paying spammer at the lowest fee
  doubles M_L faster (281 days) than a 51% miner does under v17. The miner strategies here are simple fixed
  targets, not optimised.

### Q4 Recovery (`scenarios/q4_recovery.json`, 8 seeds, `tables/q4.md`)

![Q4 recovery](figures/q4_recovery.png)

| growth phase | rules | M_L at end | +1 month | +3 months | +6 months | +12 months | min fee at end ÷ before | back to ≤ 1.1× base |
|---|---|---|---|---|---|---|---|---|
| 30 days | v16 | 0.30 MB | 0.30 | 0.30 | 0.30 | 0.30 | 1 | M_N in hours |
| 30 days | v17 | 0.625 | 0.625 | 0.625 | 0.625 | 0.625 | 1 | M_N in 3 days (backlog expiry) |
| 90 days | v16 | 0.51 | 0.51 | 0.30 | 0.30 | 0.30 | 0.35 | 72 days |
| 90 days | v17 | 0.75 | 0.75 | 0.625 | 0.625 | 0.625 | 0.70 | 73 days |
| 1 year | v16 | 4.26 | 4.26 | 2.51 | 1.47 | 0.30 | 0.005 | 350 days |
| 1 year | v17 | 1.56 | 1.56 | 1.30 | 1.08 | 0.625 | 0.16 | 350 days |

* **M_S** falls within about 50 blocks, but M_N = max(M_S, M_L) stays at M_L until M_L moves.
* **M_L** stays flat for 50,000 blocks, then steps down by the clamp ratio per 50,000 blocks: 5 steps from 1.56 MB
  for v17, 5 from 4.26 MB for v16. Decay is as fast as growth was, never faster.
* **After 2 years of clamp spam** (Q3b), M_L 12 months later is still 1.52 MB (v17) or 1.15 MB (v16), and the
  minimum fee is 6× (v17) and 14× (v16) below its pre-attack value.

### Q5 Fee market (`scenarios/q5_fee_market.json`, `q5b_daily.json`, 16 / 8 seeds, `tables/q5.md`)

![Q5 fees](figures/q5_fee_market.png)

Steady Poisson demand at multiples of Z_M. The fee per transaction is taken over the second half of each
100,000-block run; the waits include the initial ramp from Z_M.

| demand | v17 auto: fee/tx, p99 wait, max | v17 L0 | v16 auto | v16 L0 |
|---|---|---|---|---|
| 0.5·Z_M | 0.00018 XMR, 0, 0 | same | 0.00005, 0, 0 | same |
| 0.8·Z_M | 0.00044, 0, 5 | 0.00018, 0, 1 | 0.00012, 0, 3 | 0.00005, 0, 1 |
| 1.0·Z_M | 0.00069, 1, 51 | 0.00018, 1, 3 | 0.00020, 1, 62 | 0.00005, 1, 2 |
| 1.2·Z_M | 0.00054, 1, 413 | 0.00014, 31, 67 | 0.00015, 1, 789 | 0.00004, 82, 104 |
| 2·Z_M | 0.00049, 18, 1,055 | 0.00013, 694, 749 | 0.00007, 291, 2,048 | 0.00002, 1,352, 1,381 |
| 5·Z_M | 0.00049, 333, 1,746 | 0.00013, 2,247, 2,371 | 0.00007, 1,012, 2,651 | 0.00002, 4,552, 4,616 |

* **Fees per transaction fall** at high demand because M_L grows and the levels scale as 1/M_L².
* **Auto's max wait** exceeds L0's at 1.2·Z_M. Transactions sent at L0 just before a backlog forms are overtaken
  by every later "normal" one: the starvation effect.
* **Users-pay-more** cuts the p99 to ≤ 1 block and the max to under 190 at every demand level.
* **Daily cycle** around Z_M (amplitude 0.5–0.9), v17 auto: mean waits of 20–57 blocks and p99 48–108 at mean
  demand Z_M; 57–82 and 160–250 at 1.5·Z_M. With users-pay-more: about 1 block, at 4–8× the fee.
* **Blocks over M_N.** Once demand is at or above Z_M, about half of all blocks exceed M_N, by 2–5% on average
  at normal fees (Q5 table) and by up to 2× at high levels. How far a block can go depends on the level and on
  M_N/M_L (v17; full table with v16 in `tables/q5.md`):

  | level | M_N = M_L | M_N = 2·M_L | M_N = 8·M_L |
  |---|---|---|---|
  | L0 1× | 1.017 | 1.024 | 1.081 |
  | L1 4× | 1.047 | 1.084 | 1.32 |
  | L2 16× | 1.17 | 1.32 | 2 |
  | L3 64× | 1.65 | 2 | 2 |
  | L4 f_X | 2 | 2 | 2 |

  Any value above 1 at M_N = M_L means that a backlog at that level makes the short-term median climb. The
  increase compounds, because fees are tied to M_L while the overshoot scales with M_N²/M_L².

### Q6 Parameter sensitivity (`scenarios/q6_sensitivity.json`, `tables/q6.md`, `tables/q6_main_effects.csv`)

![Q6 sensitivity](figures/q6_sensitivity.png)

| metric (log) | Z_M | clamp | surge | T_R | tx weight |
|---|---|---|---|---|---|
| blocks to 10 MB max weight | 0.18 | 0.01 | **0.57** | 0.03 | 0.00 |
| blocks to 2× M_L | 0.00 | **1.00** | 0.00 | 0.00 | 0.00 |
| 10× 1-day surge, mean wait | 0.10 | 0.00 | 0.05 | **0.56** | 0.02 |
| 50× 1-day surge, clear time | 0.00 | 0.00 | **0.47** | 0.29 | 0.01 |
| spam at M_N for 7 days, XMR | 0.02 | 0.00 | 0.00 | **0.79** | 0.00 |
| spam to 2× M_L, XMR | 0.07 | 0.25 | 0.00 | **0.32** | 0.10 |
| 51% miner to 2× M_L, XMR | 0.28 | **0.75** | 0.00 | 0.00 | 0.09 |

How each parameter moves the metrics (geometric means over the rest of the grid):

* **Clamp ratio** 1.1 → 1.7: time to double M_L 400k → 100k blocks. A 51% miner's cost to do it rises from 131
  to 9,300 XMR, since a larger clamp means more penalty per padded block.
* **Surge factor** 4 → 50: time to reach 10 MB 168k → 1.5k blocks; 50× one-day surge clear time 5,960 → 935
  blocks.
* **T_R** 3,000 → 20,000: cost of 7 days of full blocks 6.1 → 33 XMR; mean wait in a 10× surge 1,030 → 266
  blocks.
* **T_R below the transaction weight.** With T_R = 3,000 the minimum-fee spammer never doubled M_L, and with T_R
  = 10,000 and 11,851-byte transactions it did so in only 17% of rule sets.

### Q7 The fork transition (`scenarios/q7_fork.json`, `q7b_fork_mainnet.json`, `tables/q7.md`)

![Q7 fork](figures/q7_fork.png)

Pre-fork windows were filled with M_L = X. Demand after the fork is 38 tx/block; "ref" is the reference code,
"spec" the transitional rule. At the first post-fork block:

| X | M_L ref / spec | max weight ref / spec | f_L ref / spec (XMR/byte) | f_L just before the fork |
|---|---|---|---|---|
| 300k | 625k / 625k | 1.25 / 1.25 MB | 2.0e-8 / 2.0e-8 | 2.0e-8 |
| 450k | 625k / 938k | 1.25 / 1.88 MB | 2.0e-8 / 8.6e-9 | 8.9e-9 |
| 600k | 625k / 1,250k | 1.25 / 2.5 MB | 2.0e-8 / 4.8e-9 | 5.0e-9 |
| 1M | 1,000k / 2,083k | 2.0 / 4.17 MB | 7.5e-9 / 1.8e-9 | 1.8e-9 |
| 2M | 2,000k / 4,167k | 4.0 / 8.33 MB | 1.9e-9 / 4.4e-10 | 4.5e-10 |

* **Fees.** The rescale keeps the per-byte minimum fee nearly continuous across the fork, because (625/300)² ≈
  T_R ratio 12,500/3,000. Without it, fees per byte jump 2.3–4.5× on top of the 3.6× larger transactions.
* **Duration.** With demand at 38 tx/block, the differences fade only as pre-fork weights leave the window: up
  to 146,000 blocks for X = 450k and 196,000 for 600k. With demand above capacity after the fork, they persist
  for all 200,000 blocks, with 2× in limits and 2–4.5× in fees.
* **From the real mainnet windows** (Q7b, 8 seeds):

  | demand after fork | effect of the rule |
  |---|---|
  | 38 tx/block | none: the series are identical |
  | 3× today's transactions | M_N differs from 39 blocks after the fork (short-term window) |
  | above capacity | M_L 7.5% higher from about 46,000 blocks after the fork (end-of-run M_L 1.158 MB vs 1.080 MB) |

  These come from the rescaled pre-fork long-term weights (up to 456,144 × 625/300 = 950k) carrying the median
  once post-fork weights make up half the window.

## Limitations

* **Miners and wallets are idealised.** Miners are perfectly rational and fully informed. Wallets use the
  documented levels without fee bumping. Users don't drop out or react to waits, except under users-pay-more.
* **Attacker strategies are simple fixed policies.** Optimised or mixed strategies (for example a <50% miner
  combined with spam, or a miner tuning its target to M_S) may be cheaper than the numbers here.
* **No network, verification, propagation or orphaning costs, and no pool size limit.** Pool expiry is used only
  in Q4. Without it, surge waits beyond 3 days (only in 1-week 50× surges) would in reality become dropped
  transactions.
* **Equal transaction weight within a class; fees evaluated at inclusion.** Fees in pending transactions do not
  follow the levels that were in force when they were broadcast.
* **Reward fixed at 0.6 XMR.** Penalties and fees in XMR scale with it.
* **Simple starting state.** Except for Q7b, every run starts from windows filled with Z_M, which matches
  mainnet's current M_L but not its short-term window. Demand starts instantly at its level, so the Q5 "whole run"
  waits include one ramp from Z_M.
* **Q6 seeds.** Q6 uses one seed per rule set (576 sets). The surge metrics are averaged over the grid rather than
  over seeds.
* **Scope of the transitional rule.** It was applied to all windows. Whether the spec intends the short-term
  window too should be confirmed (see the first point for implementers).
* **Stressnet's sizes aren't mainnet's.** It was used only to confirm transaction weights and the clamp, not as a
  source of block-size dynamics, because of its 10 MB zone.

## Reproduce

```
git clone https://github.com/Fountain5405/adaptiveblocksizesim && cd adaptiveblocksizesim
git checkout study-2026
python3 -m venv ~/studyvenv && ~/studyvenv/bin/pip install matplotlib numpy pandas
PYTHON=~/studyvenv/bin/python study/run_all.sh                      # tests, all scenarios, figures, tables (~2 min on 12 cores)
MONERO_RPC=http://127.0.0.1:18081 PYTHON=... study/run_all.sh        # also refreshes data/mainnet_weights.csv.gz
cd study && ./target/release/study scenarios/q2_surge.json --out results   # one scenario
MONERO_RPC=http://192.168.1.22:28089 python3 scripts/fetch_chain.py data/stressnet_weights.csv.gz 120000
```

Committed: `results/*/summary.csv`, `events.csv`, `runs.jsonl` and `meta.json` (git commit, scenario text,
seeds). The per-run series (about 70 MB) are regenerated by `run_all.sh`, because they are over the ~50 MB
guideline. One caveat: refreshing the mainnet data moves the chain window, so Q7b will then differ slightly from
the committed results.

## Files

| Path | Contents |
|---|---|
| `wasm-sim/src/lib_core/step.rs` | steppable simulator and its regression tests |
| `study/src/{main,spec,run}.rs` | harness: scenario expansion, parallel runs, metrics, output |
| `study/scenarios/*.json` | the scenarios behind every number (`pilot/` holds the pilots) |
| `study/analysis/analyze.py` | figures and tables |
| `study/figures/`, `study/tables/` | outputs referenced above |
| `study/data/` | mainnet and stressnet weight histories |
