# Rules the simulator implements

The simulator follows the reference code, not a summary of it. Where the spec and the code
disagree, the code wins and the difference is listed below.

- **v16** (mainnet today, "2021 scaling"): [monero-project/monero](https://github.com/monero-project/monero) `master`
- **v17** (the FCMP++ hard fork, "2026 scaling"): [seraphis-migration/monero](https://github.com/seraphis-migration/monero) branch `fcmp++-stage`,
  which implements ArticMine's [MoneroScaling2026-03](https://github.com/ArticMine/Monero-Documents/blob/master/MoneroScaling2026-03.pdf)
  (merged in [#282](https://github.com/seraphis-migration/monero/pull/282), reference transaction raised to 12,500 bytes afterwards)

Checked against `fcmp++-stage` as of commit `f25388d156` (21 October 2026).

## Consensus

| | v16 | v17 | Code |
|---|---|---|---|
| Penalty-free zone Z_M | 300,000 | 625,000 | `get_min_block_weight` (cryptonote_basic_impl.cpp) |
| Long-term weight of a block | clamp(M_B, M_L·10/17, M_L·17/10) | clamp(M_B, M_L·5/6, M_L·6/5) | `get_next_long_term_block_weight` |
| M_L | max(Z_M, median of the last 100,000 long-term weights) | same | `update_next_cumulative_weight_limit` |
| M_S | median of the last 100 block weights | same | same |
| M_N (penalty median) | min(max(M_S, M_L), 50·M_L), at least Z_M | min(max(M_S, M_L), 8·M_L), at least Z_M | same |
| Maximum block weight | 2·M_N | 2·M_N | same |
| Penalty | R·(M_B/M_N − 1)² | same | `get_block_reward` |

All of these are integer operations in the code and in the simulator, including the median of an
even-sized window (`(a + b) / 2`, rounded down).

Because the long-term weight is clamped against the *current* M_L and M_L is a median, M_L can step
up by at most the clamp ratio for every half window: about 1.2× per 50,000 blocks under v17 and
1.7× under v16.

## Fees

| | v16 | v17 | Code |
|---|---|---|---|
| Reference transaction T_R | 3,000 | 12,500 | `get_reference_tx_weight` |
| Node relay minimum per byte | 0.95 · R·T_R / M_L² | R·T_R / M_L² | `get_dynamic_base_fee`, `check_fee` (accepts 98%) |
| Wallet fee levels | 4: f_L, 4f_L, f_M, f_H | 5: f_L, 4f_L, 16f_L, 64f_L, f_X = 2R/M_FW | `get_dynamic_base_fee_estimate_2021_scaling` / `_2026_scaling` |
| Wallet grace blocks | 10 | 1,000 | `FEE_ESTIMATE_GRACE_BLOCKS_2021/2026` (wallet2.cpp) |
| Rounding | up to 2 significant digits | same | `round_money_up` |

Wallets estimate M_L as if the next `grace` blocks were empty: the latest 100,000 − grace
long-term weights plus `grace` zeros. The simulator keeps that median exactly, block by block.

**Which level wallets pay.** By default (`fee_level = auto`) simulated wallets do what
`wallet2::adjust_priority` does: pay the lowest level, unless the pool holds at least one block's
worth of transactions or the last 10 blocks averaged more than 80% of M_N, in which case they pay
"normal". A fixed level can be chosen instead. *Users pay more* moves a growing share of
transactions two levels up while a backlog persists.

## What the simulator models on top

- **Miners.** By default miners are rational: they add the highest-fee transactions first and stop
  when a transaction's fee no longer covers the penalty it adds. Turning this off fills every block
  to the maximum weight regardless of fees.
- **Transactions** all have the same weight, `t_sim`. Defaults: 2,000 bytes for v16 (roughly a
  typical current transaction) and 9,093 bytes for v17 (an FCMP++ 2-input/2-output transaction per
  `get_fcmp_pp_transaction_weight_v1`; 1-in/2-out is 7,650 and 4-in/2-out is 11,851).
- **Fees** are evaluated when a transaction is included, at the levels current then.
- **Block reward** is fixed at the tail emission, 0.6 XMR.

## Not modelled

- The 2026 spec's transitional rule, which rescales pre-fork block weights by 625/300 when a median
  window spans the fork. The reference code doesn't implement it either. Per the spec it only
  matters if M_L is above 300,000 at the fork.
- The 600 bytes reserved for the coinbase, the transaction pool size limit, and the per-transaction
  weight limit (`FCMP_PLUS_PLUS_MAX_TX_WEIGHT`, 178,063).

## Earlier drafts

The December 2025 drafts of the proposal had a 1,000,000-byte Z_M, a 10 MB "sanity cap" growing
about 40% a year, a 16·M_L cap, and a 10,000-byte T_R. None of these made it into the final spec or
the code. The sanity cap is still available in **Custom** mode for comparison.
