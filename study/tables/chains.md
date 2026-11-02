### Mainnet, heights 3,697,558–3,797,557 (139 days)

- block weight: mean 95,639, median 63,214, 90th pct 268,422, max 456,144
- transactions per block: mean 38.4; bytes per transaction (incl. coinbase share) 2,493
- long-term weights: median 176,470 (= 300,000 × 10/17, the lower clamp), so M_L = Z_M = 300,000; blocks above 300,000: 7,450

### Stressnet (testnet with the FCMP++ fork, hard fork version 18 from height 3,103,520), heights 3,103,520–3,123,060

- every small post-fork block has long-term weight 8,333,333 = 10,000,000 × 5/6, i.e. the v17 lower clamp around M_L = 10,000,000: stressnet runs with a 10 MB penalty-free zone, so its absolute sizes are not the mainnet parameters
- most common weights of one-transaction blocks: 9,330, 9,421, 9,200, 9,420, 7,887; minus the coinbase (107, 237 or 328 bytes in empty blocks) these are exactly 9,093 (2-in/2-out) and 7,650 (1-in/2-out), as the simulator assumes
- 195 blocks exceeded the 10,000,000 penalty-free size, by up to 4.1% — the 'slightly exceed M_N when fees cover the penalty' mechanism, seen on a live network
- transactions per post-fork block: mean 26.2; bytes per transaction 8,573
