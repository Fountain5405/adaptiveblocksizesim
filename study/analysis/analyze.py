#!/usr/bin/env python3
"""Figures and tables for study/REPORT.md, from study/results/.

Run from anywhere: python3 study/analysis/analyze.py
Writes study/figures/*.png, study/tables/*.md (one file per question) and
study/tables/key_numbers.md (the headline numbers quoted in the report, computed here).
"""
import os, sys, math
import numpy as np
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.ticker import FuncFormatter, LogLocator, NullLocator

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
RES = os.path.join(ROOT, "results")
FIG = os.path.join(ROOT, "figures")
TAB = os.path.join(ROOT, "tables")
os.makedirs(FIG, exist_ok=True)
os.makedirs(TAB, exist_ok=True)

DAY, MONTH, YEAR = 720, 21600, 262800
# Reference categorical palette (dataviz skill, light mode), fixed order per entity
C = {"v17": "#2a78d6", "v16": "#eb6834", "v17_sf50": "#1baf7a", "v16_9093": "#eda100", "v16_9093_lo": "#eda100", "v17_mix": "#6250d6"}
INK, INK2, GRID, SURF = "#0b0b0b", "#52514e", "#e4e3df", "#fcfcfb"
plt.rcParams.update({
    "figure.facecolor": SURF, "axes.facecolor": SURF, "savefig.facecolor": SURF,
    "axes.edgecolor": INK2, "axes.labelcolor": INK, "xtick.color": INK2, "ytick.color": INK2,
    "axes.grid": True, "grid.color": GRID, "grid.linewidth": 0.6, "axes.spines.top": False,
    "axes.spines.right": False, "lines.linewidth": 2, "font.size": 9, "axes.titlesize": 10,
    "legend.frameon": False, "figure.dpi": 110,
})

KEY = []  # (question, label, value) for tables/key_numbers.md


def key(q, label, value, fmt="{:,.3g}"):
    KEY.append((q, label, value if isinstance(value, str) else ("–" if pd.isna(value) else fmt.format(value))))


def blues(n):
    cm = plt.get_cmap("Blues")
    return [cm(0.35 + 0.6 * i / max(1, n - 1)) for i in range(n)]


def load(name):
    d = pd.read_csv(os.path.join(RES, name, "summary.csv"), low_memory=False).copy()
    parts = d["case"].fillna("").str.split("/", expand=True)
    for i in range(parts.shape[1]):
        d[f"c{i}"] = parts[i]
    return d


def series(name, run_id):
    return pd.read_csv(os.path.join(RES, name, "series", f"{run_id:05d}.csv.gz"))


def events(name):
    return pd.read_csv(os.path.join(RES, name, "events.csv"))


def nseeds(d):
    return int(d.seed_index.nunique())


def md(df, floatfmt=None):
    """DataFrame -> markdown table (no tabulate dependency)."""
    cols = list(df.columns)
    out = ["| " + " | ".join(str(c) for c in cols) + " |", "|" + "|".join("---" for _ in cols) + "|"]
    for _, r in df.iterrows():
        cells = []
        for c in cols:
            v = r[c]
            if isinstance(v, (float, np.floating)):
                if math.isnan(v):
                    cells.append("–")
                elif floatfmt and c in floatfmt:
                    cells.append(floatfmt[c].format(v))
                elif abs(v) >= 1000 or v == int(v):
                    cells.append(f"{v:,.0f}")
                elif abs(v) >= 1:
                    cells.append(f"{v:.2f}")
                else:
                    cells.append(f"{v:.3g}")
            else:
                cells.append(str(v))
        out.append("| " + " | ".join(cells) + " |")
    return "\n".join(out)


def days(blocks):
    return blocks / DAY


def dd(v):
    return days(v) if pd.notna(v) else float("nan")


def write(name, text):
    with open(os.path.join(TAB, name), "w") as f:
        f.write(text.strip() + "\n")
    print("wrote tables/" + name)


def savefig(fig, name):
    fig.savefig(os.path.join(FIG, name), bbox_inches="tight")
    plt.close(fig)
    print("wrote figures/" + name)


def log_x(ax, ticks):
    ax.set_xscale("log")
    ax.xaxis.set_minor_locator(NullLocator())
    ax.set_xticks(ticks)
    ax.set_xticklabels([f"{t:g}" for t in ticks])


# ----------------------------------------------------------------------------------------
# Q1 growth capacity
# ----------------------------------------------------------------------------------------
def q1():
    d = load("q1_growth")
    d["rules"], d["mode"] = d.c0, d.c1
    out = []
    rows = []
    for (r, m, fm), g in d.groupby(["rules", "mode", "demand.fill_mult"], sort=False):
        x = g.iloc[0]
        row = {"rules": r, "miners/fee": m, "demand": "1.0×max" if fm == 1.0 else "1.1×M_N"}
        for t in [2, 5, 10, 50, 100]:
            row[f"limit {t} MB"] = dd(x[f"t_maxw_{t}MB"])
        for t in [10, 50, 100]:
            row[f"block {t} MB"] = dd(x[f"t_block_{t}MB"])
        row["M_L 2×"] = dd(x["t_m_l_2x"])
        row["M_L 10×"] = dd(x["t_m_l_10x"])
        row["fee/tx (XMR)"] = x["user_fee_per_tx"]
        rows.append(row)
    t = pd.DataFrame(rows)
    fmt = {c: "{:,.1f}" for c in t.columns if "MB" in c or "M_L" in c}
    fmt["fee/tx (XMR)"] = "{:.2e}"
    out.append("### Days until the limit and the blocks reach a size, demand permanently above capacity\n")
    out.append("Days from a steady chain at Z_M. `limit N MB`: the maximum block weight 2·M_N reaches N MB. "
               "`block N MB`: the first block of at least N MB is mined. "
               "`fill` = miners fill every block to the maximum regardless of fees (upper bound; blocks sit at the limit). "
               "`L0`..`L4` = rational miners, every user pays that wallet level; `auto` = wallet2 default priority; "
               "`auto_upm` = auto with users-pay-more. With rational miners, blocks run at about M_N, half the limit, "
               "so the first block of a size comes one doubling of M_L later. v17's limit starts at 2·8·625,000 = 10 MB "
               "once M_S reaches the 8× cap. v16 uses 2,500-byte transactions (mainnet average), v16_9093 and v17 use 9,093.\n")
    out.append(md(t[t.demand == "1.0×max"].drop(columns="demand"), fmt))
    out.append("\nSame with demand only 10% above M_N (fill_mult 0.55 × the maximum weight):\n")
    out.append(md(t[t.demand == "1.1×M_N"].drop(columns="demand"), fmt))
    for r in ["v16", "v17"]:
        for m in ["fill", "auto", "L0"]:
            x = t[(t.rules == r) & (t["miners/fee"] == m) & (t.demand == "1.0×max")].iloc[0]
            for c in ["limit 10 MB", "limit 50 MB", "limit 100 MB", "block 10 MB", "block 50 MB", "block 100 MB", "M_L 2×"]:
                key("Q1", f"{r} {m}: {c} (days)", x[c], "{:,.1f}")

    # analytic bound check: M_L(t+50000)/M_L(t) <= clamp ratio
    chk = []
    for _, x in d[d["demand.fill_mult"] == 1.0].iterrows():
        s = series("q1_growth", int(x.run_id))
        ml = s.set_index("block").m_l
        b = ml.index.values
        a = ml.values.astype(float)
        idx = {v: i for i, v in enumerate(b)}
        ratios = [a[idx[v + 50000]] / a[i] for i, v in enumerate(b) if v + 50000 in idx]
        clamp = 1.7 if x.rules.startswith("v16") else 1.2
        chk.append({"rules": x.rules, "miners/fee": x["mode"], "max M_L ratio over 50k blocks": max(ratios), "clamp": clamp,
                    "violations": int(sum(r > clamp * (1 + 1e-9) + 1e-6 for r in ratios))})
    ck = pd.DataFrame(chk)
    out.append("\n### Analytic bound: M_L rises at most by the clamp ratio per 50,000 blocks\n")
    out.append(f"Largest ratio M_L(b+50,000)/M_L(b) over every sampled block b, {d.blocks.iloc[0]:,}-block runs:\n")
    out.append(md(ck, {"max M_L ratio over 50k blocks": "{:.4f}"}))
    key("Q1", "max M_L ratio over 50k blocks, v16 runs", ck[ck.rules == "v16"]["max M_L ratio over 50k blocks"].max(), "{:.4f}")
    key("Q1", "max M_L ratio over 50k blocks, v17 runs", ck[ck.rules == "v17"]["max M_L ratio over 50k blocks"].max(), "{:.4f}")
    key("Q1", "bound violations, all runs", int(ck.violations.sum()), "{:d}")
    pred = []
    for rules, z, c, cap in [("v16", 300000, 1.7, 50), ("v17", 625000, 1.2, 8)]:
        for t_ in [50, 100]:
            k = max(0, math.ceil(math.log(t_ * 1e6 / (2 * cap * z)) / math.log(c) - 1e-12))
            pred.append({"rules": rules, "limit": f"{t_} MB", "predicted (days)": k * 50000 / DAY,
                         "simulated, fill (days)": days(d[(d.rules == rules) & (d["mode"] == "fill") & (d["demand.fill_mult"] == 1.0)][f"t_maxw_{t_}MB"].iloc[0])})
    out.append("\nPredicted time for the limit 2·surge·M_L to reach a size if M_L steps up by the clamp ratio every 50,000 blocks, vs simulated (fill):\n")
    out.append(md(pd.DataFrame(pred), {"predicted (days)": "{:.1f}", "simulated, fill (days)": "{:.1f}"}))

    fig, axes = plt.subplots(1, 3, figsize=(12, 3.8), sharey=True)
    for ax, col, title in [(axes[0], "max_weight", "Limit: maximum block weight 2·M_N"), (axes[1], "mb_mean", "Actual mean block weight"),
                           (axes[2], "m_l", "Long-term median M_L")]:
        for rules in ["v16", "v17"]:
            for mode, ls in [("fill", "-"), ("auto", "--"), ("L0", ":")]:
                x = d[(d.rules == rules) & (d["mode"] == mode) & (d["demand.fill_mult"] == 1.0)].iloc[0]
                s = series("q1_growth", int(x.run_id))
                s = s[s.block <= 3 * YEAR]
                ax.plot(s.block / YEAR, s[col], ls=ls, color=C[rules], label=f"{rules} {mode}")
        ax.set_yscale("log")
        ax.set_title(title, loc="left")
        ax.set_xlabel("years")
        for y in [2e6, 1e7, 1e8]:
            ax.axhline(y, color=INK2, lw=0.6, ls="-", alpha=0.4)
    axes[0].set_ylabel("bytes")
    axes[0].legend(ncol=2, fontsize=8)
    fig.suptitle("Q1: growth with demand permanently above capacity (fill = miners ignore fees; auto/L0 = rational miners)", x=0.01, y=1.03, ha="left", fontsize=10)
    savefig(fig, "q1_growth.png")

    fig, ax = plt.subplots(figsize=(6.5, 3.6))
    cols = blues(5)
    for rules, ls in [("v17", "-"), ("v16", "--")]:
        for k, mode in enumerate(["L0", "L1", "L2", "L3", "L4"]):
            if rules == "v16" and mode == "L4":
                continue
            x = d[(d.rules == rules) & (d["mode"] == mode) & (d["demand.fill_mult"] == 1.0)].iloc[0]
            s = series("q1_growth", int(x.run_id))
            s = s[s.block <= 12 * DAY]
            ax.plot(s.block / DAY, s.max_weight, ls=ls, color=cols[k], label=f"{rules} {mode}")
    ax.set_yscale("log")
    ax.set_xlabel("days")
    ax.set_ylabel("maximum block weight, bytes")
    ax.set_title("Q1: short-term ramp by fee level (solid v17, dashed v16; darker = higher fee; v17 L3 and L4 coincide, as do v16 L2 and L3)", loc="left", fontsize=9)
    ax.legend(ncol=3, fontsize=7)
    savefig(fig, "q1_ramp_by_level.png")

    b = load("q1b_demand_growth")
    b["rules"], b["fee"] = b.c0, b.c1
    b["dropped"] = (b.evicted_user_txs + b.expired_user_txs + b.rejected_user_txs) / b.arrived_user_txs
    rows = []
    for (r, f, g_), g in b.groupby(["rules", "fee", "demand.growth_per_year"]):
        rows.append({"rules": r, "wallets": f, "growth/yr": f"{g_*100:.0f}%", "final demand (tx/block)": 38 * (1 + g_) ** 5,
                     "final M_L (MB)": g.final_m_l.mean() / 1e6, "final M_N (MB)": g.final_m_n.mean() / 1e6,
                     "wait mean (blocks)": g.wait_mean.astype(float).mean(), "wait p99": g.wait_p99.astype(float).mean(),
                     "wait max": g.wait_max.max(), "dropped share": g.dropped.mean(), "fee/tx (XMR)": g.user_fee_per_tx.astype(float).mean(),
                     "max pool (MB)": g.max_pool_user_bytes.mean() / 1e6})
    q1b = pd.DataFrame(rows)
    out.append(f"\n### Q1b: steady exponential demand growth from 38 tx/block for 5 years (Poisson, {nseeds(b)} seeds)\n")
    out.append("Waits are over mined transactions; `dropped share` = transactions evicted from the full pool, expired after 3 days, or below the relay minimum, as a share of all broadcast.\n")
    out.append(md(q1b, {"final M_L (MB)": "{:.3g}", "final M_N (MB)": "{:.3g}", "wait mean (blocks)": "{:.2f}", "fee/tx (XMR)": "{:.2e}", "max pool (MB)": "{:.3g}",
                        "final demand (tx/block)": "{:,.0f}", "wait p99": "{:.1f}", "dropped share": "{:.3f}"}))
    for r in ["v16", "v17"]:
        for gr in ["200%", "400%"]:
            x = q1b[(q1b.rules == r) & (q1b.wallets == "auto") & (q1b["growth/yr"] == gr)].iloc[0]
            key("Q1", f"Q1b {r} auto {gr}/yr: final M_L / M_N (MB)", f"{x['final M_L (MB)']:.3g} / {x['final M_N (MB)']:.3g}")
            key("Q1", f"Q1b {r} auto {gr}/yr: p99 wait, dropped share", f"{x['wait p99']:.1f} / {x['dropped share']:.3f}")
    write("q1.md", "\n".join(out))


# ----------------------------------------------------------------------------------------
# Q2 surges
# ----------------------------------------------------------------------------------------
DUR = {30: "1 hour", 720: "1 day", 5040: "1 week"}


def surge_agg(d, keys):
    d = d.copy()
    d["dropped"] = (d.win_rejected + d.win_expired + d.win_evicted) / d.win_arrived
    return d.groupby(keys).agg(
        peak_pool=("win_peak_pool_user_bytes", "mean"), clear=("win_clear_after_end", "mean"),
        uncleared=("win_clear_after_end", lambda x: int(x.isna().sum())),
        mined=("win_mined_share", "mean"), dropped=("dropped", "mean"), evicted=("win_evicted", "mean"), expired=("win_expired", "mean"),
        pending=("win_pending_end", "mean"),
        wait_mean=("win_wait_mean", "mean"), wait_p90=("win_wait_p90", "mean"), wait_p99=("win_wait_p99", "mean"),
        wait_max=("win_wait_max", "max"), fee_tx=("win_fee_per_tx", "mean"), fee_b=("win_fee_per_byte", "mean"),
        served30=("win_served_within_30", "mean"), served720=("win_served_within_720", "mean"), served2160=("win_served_within_2160", "mean"),
        max_mn=("max_m_n", "mean"), n=("seed", "count")).reset_index()


def q2():
    d = load("q2_surge")
    d["rules"], d["fee"] = d.c0, d.c1
    ns = nseeds(d)
    agg = surge_agg(d, ["rules", "fee", "demand.surge_blocks", "demand.surge_mult"])
    agg.to_csv(os.path.join(TAB, "q2_surge_all.csv"), index=False)
    out = ["Pool: monerod's defaults (648 MB cap, lowest fee level and newest pruned first; 3-day expiry). "
           "`mined` = share of the transactions broadcast during the surge that were ever mined (the rest were pruned from the full pool or expired). "
           "Waits are over mined transactions; `mined ≤ 1 day` is over all of them.\n"]
    for dur in [30, 720, 5040]:
        a = agg[(agg.fee == "auto") & (agg["demand.surge_blocks"] == dur)]
        t = a[["rules", "demand.surge_mult", "peak_pool", "clear", "mined", "served720", "wait_mean", "wait_p99", "wait_max", "fee_tx", "max_mn"]].copy()
        t.columns = ["rules", "surge ×", "peak backlog (MB)", "clears after end (blocks)", "mined", "mined ≤ 1 day", "mean wait", "p99 wait", "max wait",
                     "fee/tx (XMR)", "peak M_N (MB)"]
        t["peak backlog (MB)"] /= 1e6
        t["peak M_N (MB)"] /= 1e6
        out.append(f"\n### Surge lasting {DUR[dur]}, wallets on auto priority (means over {ns} seeds; waits in blocks of 2 min)\n")
        out.append(md(t, {"peak backlog (MB)": "{:.3g}", "clears after end (blocks)": "{:,.0f}", "mined": "{:.3f}", "mined ≤ 1 day": "{:.3f}",
                          "mean wait": "{:,.1f}", "p99 wait": "{:,.0f}", "fee/tx (XMR)": "{:.2e}", "peak M_N (MB)": "{:.3g}"}))
    a = agg[agg["demand.surge_blocks"] == 720]
    lv = ["L0", "L1", "L2", "L3", "L4", "auto", "auto_upm"]
    for val, title, f in [("wait_mean", "Mean wait of mined surge transactions (blocks) by fee level, 1-day surges", "{:,.1f}"),
                          ("mined", "Share of surge transactions mined at all, by fee level, 1-day surges", "{:.3f}"),
                          ("fee_tx", "Fee per mined surge transaction (XMR) by fee level, 1-day surges", "{:.2e}")]:
        piv = a.pivot_table(index=["rules", "demand.surge_mult"], columns="fee", values=val).reset_index()
        piv = piv[["rules", "demand.surge_mult"] + lv]
        out.append(f"\n### {title}\n")
        out.append(md(piv, {k: f for k in lv}))
    rows = []
    for (r, dur, m), g in agg.groupby(["rules", "demand.surge_blocks", "demand.surge_mult"]):
        row = {"rules": r, "duration": DUR[dur], "surge ×": m}
        for col, nm in [("served30", "99% mined ≤ 1 h"), ("served720", "99% mined ≤ 1 day")]:
            ok = g[g.fee.isin(["L0", "L1", "L2", "L3", "L4"]) & (g[col] >= 0.99)]
            row[nm] = ok.fee.min() if len(ok) else "none"
        rows.append(row)
    out.append("\n### Lowest fixed fee level at which 99% of the surge's transactions are mined within a limit\n")
    out.append(md(pd.DataFrame(rows)))
    rows = []
    for (dur, m), g in agg[agg.fee == "auto"].groupby(["demand.surge_blocks", "demand.surge_mult"]):
        a8 = g[g.rules == "v17"].iloc[0]
        a50 = g[g.rules == "v17_sf50"].iloc[0]
        rows.append({"duration": DUR[dur], "surge ×": m, "mined, 8×": a8.mined, "mined, 50×": a50.mined,
                     "mined ≤ 1 day, 8×": a8.served720, "mined ≤ 1 day, 50×": a50.served720,
                     "mean wait, 8×": a8.wait_mean, "mean wait, 50×": a50.wait_mean,
                     "peak M_N 8× (MB)": a8.max_mn / 1e6, "peak M_N 50× (MB)": a50.max_mn / 1e6})
    cap = pd.DataFrame(rows)
    out.append("\n### Does the 8× surge cap hurt? v17 vs the same rules with a 50× cap (Custom), auto priority\n")
    out.append(md(cap, {c: "{:.3f}" for c in cap.columns if c.startswith("mined")} | {c: "{:,.1f}" for c in cap.columns if "wait" in c}
                  | {"peak M_N 8× (MB)": "{:.3g}", "peak M_N 50× (MB)": "{:.3g}"}))

    # pool sensitivity (q2b)
    p = load("q2b_pool")
    p["rules"], p["pool"] = p.c0, p.c1
    pa = surge_agg(p, ["rules", "pool", "demand.surge_blocks", "demand.surge_mult"])
    rows = []
    for (r, dur, m), g in pa.groupby(["rules", "demand.surge_blocks", "demand.surge_mult"]):
        mo = g[g.pool == "monerod_pool"].iloc[0]
        un = g[g.pool == "unbounded_pool"].iloc[0]
        rows.append({"rules": r, "duration": DUR[dur], "surge ×": m, "mean wait, unbounded": un.wait_mean, "mean wait, monerod": mo.wait_mean,
                     "mined, monerod": mo.mined, "mined ≤ 1 day, unbounded": un.served720, "mined ≤ 1 day, monerod": mo.served720,
                     "peak backlog unbounded (MB)": un.peak_pool / 1e6})
    pt = pd.DataFrame(rows)
    out.append(f"\n### Pool limits: the same surges with an unbounded pool (first version of this study) and with monerod's limits, auto priority ({nseeds(p)} seeds)\n")
    out.append(md(pt, {c: "{:,.1f}" for c in pt.columns if "wait" in c} | {c: "{:.3f}" for c in pt.columns if c.startswith("mined")}
                  | {"peak backlog unbounded (MB)": "{:,.0f}"}))
    write("q2.md", "\n".join(out))

    A = agg[agg.fee == "auto"].set_index(["rules", "demand.surge_blocks", "demand.surge_mult"])
    for r in ["v16", "v16_9093", "v16_9093_lo", "v17", "v17_sf50", "v17_mix"]:
        x = A.loc[(r, 720, 5)]
        key("Q2", f"{r} 1-day 5×: mean wait / mined / mined ≤1 day", f"{x.wait_mean:,.0f} / {x.mined:.3f} / {x.served720:.3f}")
    for r in ["v16", "v17", "v17_sf50"]:
        for dur, m in [(720, 10), (720, 20), (720, 50), (5040, 10), (5040, 20), (5040, 50)]:
            x = A.loc[(r, dur, m)]
            key("Q2", f"{r} {DUR[dur]} {m}×: mean wait / mined / mined ≤1 day / peak backlog MB",
                f"{x.wait_mean:,.0f} / {x.mined:.3f} / {x.served720:.3f} / {x.peak_pool/1e6:,.0f}")
    x = A.loc[("v17", 30, 50)]
    key("Q2", "v17 1-hour 50×: mean wait / mined", f"{x.wait_mean:,.0f} / {x.mined:.3f}")
    for (r, dur, m) in [("v17", 5040, 50), ("v17_sf50", 5040, 50), ("v16", 5040, 50), ("v17", 720, 20)]:
        g = pa[(pa.rules == r) & (pa["demand.surge_blocks"] == dur) & (pa["demand.surge_mult"] == m)].set_index("pool")
        key("Q2", f"pool sensitivity {r} {DUR[dur]} {m}×: mean wait unbounded→monerod, mined monerod",
            f"{g.loc['unbounded_pool'].wait_mean:,.0f} → {g.loc['monerod_pool'].wait_mean:,.0f}, {g.loc['monerod_pool'].mined:.3f}")
    L = agg[agg["demand.surge_blocks"] == 720].set_index(["rules", "demand.surge_mult", "fee"])
    for f in ["L0", "L1", "L2", "L3", "L4", "auto_upm"]:
        x = L.loc[("v17", 5, f)]
        key("Q2", f"v17 1-day 5× at {f}: mean wait / mined / fee per tx", f"{x.wait_mean:,.1f} / {x.mined:.3f} / {x.fee_tx:.2e}")

    fig, axes = plt.subplots(2, 3, figsize=(11, 6.2), sharex=True, sharey="row")
    for k, dur in enumerate([30, 720, 5040]):
        for r in ["v16", "v16_9093", "v16_9093_lo", "v17", "v17_sf50", "v17_mix"]:
            g = agg[(agg.fee == "auto") & (agg.rules == r) & (agg["demand.surge_blocks"] == dur)]
            axes[0, k].plot(g["demand.surge_mult"], g.wait_mean.clip(lower=0.1), marker="o", ms=4, color=C[r], label=r)
            axes[1, k].plot(g["demand.surge_mult"], g.served720, marker="o", ms=4, color=C[r], label=r)
        axes[0, k].set_yscale("log")
        axes[0, k].set_title(f"surge lasting {DUR[dur]}", loc="left")
        log_x(axes[1, k], [2, 5, 10, 20, 50])
        axes[1, k].set_xlabel("surge multiplier (× 38 tx/block)")
        axes[1, k].set_ylim(0, 1.05)
    axes[0, 0].set_ylabel("mean wait of mined\nsurge txs, blocks")
    axes[1, 0].set_ylabel("share of surge txs\nmined within 1 day")
    axes[0, 0].legend(fontsize=8)
    fig.suptitle(f"Q2: demand surges, wallets on auto priority, monerod pool limits ({ns} seeds)", x=0.01, y=1.03, ha="left")
    savefig(fig, "q2_surge_wait.png")

    fig, axes = plt.subplots(1, 2, figsize=(10, 3.4), sharey=True)
    cols = blues(5)
    for ax, r in zip(axes, ["v16", "v17"]):
        for k, f in enumerate(["L0", "L1", "L2", "L3", "L4"]):
            if r == "v16" and f == "L4":
                continue
            g = agg[(agg.fee == f) & (agg.rules == r) & (agg["demand.surge_blocks"] == 720)]
            ax.plot(g["demand.surge_mult"], g.served720, marker="o", ms=4, color=cols[k], label=f)
        g = agg[(agg.fee == "auto_upm") & (agg.rules == r) & (agg["demand.surge_blocks"] == 720)]
        ax.plot(g["demand.surge_mult"], g.served720, marker="s", ms=4, color=C["v16"] if r == "v17" else C["v17_sf50"], ls="--", label="auto + pay more")
        log_x(ax, [2, 5, 10, 20, 50])
        ax.set_ylim(0, 1.05)
        ax.set_title(f"{r}: 1-day surge, by fee level", loc="left")
        ax.set_xlabel("surge multiplier")
        ax.legend(fontsize=7, ncol=2)
    axes[0].set_ylabel("share of surge txs mined within 1 day")
    savefig(fig, "q2_surge_levels.png")

    fig, axes = plt.subplots(1, 3, figsize=(11, 3.2))
    # The 8x and 50x caps give identical runs for a one-day surge, so v17_sf50 is drawn dashed on top of v17.
    for r, ls, lab in [("v16", "-", "v16"), ("v17", "-", "v17"), ("v17_sf50", "--", "v17_sf50 (= v17)")]:
        x = d[(d.rules == r) & (d.fee == "auto") & (d["demand.surge_blocks"] == 720) & (d["demand.surge_mult"] == 20) & (d.seed_index == 0)].iloc[0]
        s = series("q2_surge", int(x.run_id))
        s = s[(s.block >= 4000) & (s.block <= 9000)]
        t = (s.block - 5000) / DAY
        lw = 1.2 if r == "v17_sf50" else 2
        axes[0].plot(t, s.pool_user_bytes / 1e6, color=C[r], ls=ls, lw=lw, label=lab)
        axes[1].plot(t, s.m_n / 1e6, color=C[r], ls=ls, lw=lw, label=lab)
        axes[2].plot(t, s.mb_mean / 1e6, color=C[r], ls=ls, lw=lw, label=lab)
    for ax, ttl in zip(axes, ["backlog (MB; pool cap 648 MB)", "M_N (MB)", "mean block weight (MB)"]):
        ax.axvspan(0, 1, color=GRID, alpha=0.6, lw=0)
        ax.set_title(ttl, loc="left")
        ax.set_xlabel("days since surge start")
    axes[2].legend(fontsize=8, loc="upper right")  # the only panel with empty space top-right
    fig.suptitle("Q2: a 20× surge lasting one day (shaded), auto priority, first seed", x=0.01, y=1.03, ha="left")
    savefig(fig, "q2_surge_example.png")


# ----------------------------------------------------------------------------------------
# Q3 attacks
# ----------------------------------------------------------------------------------------
def q3():
    out = []
    a = load("q3a_spam_full")
    a["rules"], a["strategy"] = a.c0, a.c1
    a["days"] = a["spam.blocks"] / DAY
    a["user_extra_day"] = (a.win_fee_per_tx - a.pre_fee_per_tx) * a.win_txs_included / a.days
    g = a.groupby(["rules", "strategy", "spam.blocks"]).agg(spent=("spam_spent_in_window_xmr", "mean"), gb=("spam_bytes", "mean"),
            per_gb=("spam_xmr_per_gb", "mean"), full=("win_full_blocks", "mean"), wb=("win_blocks", "mean"), max_mn=("max_m_n", "mean"),
            uwait=("win_wait_mean", "mean"), ufee_pre=("pre_fee_per_tx", "mean"), ufee=("win_fee_per_tx", "mean"),
            uextra=("user_extra_day", "mean")).reset_index()
    g["days"] = g["spam.blocks"] / DAY
    g["XMR/day"] = g.spent / g.days
    g["full share"] = g.full / g.wb
    g["gb"] /= 1e9
    g["max_mn"] /= 1e6
    t = g[["rules", "strategy", "days", "spent", "XMR/day", "gb", "per_gb", "full share", "max_mn", "uwait", "ufee_pre", "ufee", "uextra"]]
    t.columns = ["rules", "strategy", "days", "XMR spent", "XMR/day", "GB added", "XMR/GB", "share of blocks full", "peak M_N (MB)", "user mean wait",
                 "user fee/tx before", "user fee/tx during", "users' extra XMR/day"]
    out.append(f"### Q3a: keeping blocks full (at M_N) with fee-paying spam, users at 38 tx/block on auto ({nseeds(a)} seeds)\n")
    out.append("`fill_to_L0`: each block the spammer tops the pool up to exactly M_N at the lowest fee (no backlog). "
               "`topup_L0`: keeps a full block of spam waiting at the lowest fee, which builds a backlog. "
               "`topup_auto`: a naive spammer paying the wallet's auto level; its own backlog makes that 'normal' (4×). "
               "`users' extra XMR/day`: what users paid above their pre-attack fee per transaction, for the transactions they sent during the attack, per day.\n")
    out.append(md(t, {"XMR spent": "{:,.1f}", "XMR/day": "{:,.2f}", "GB added": "{:.3g}", "XMR/GB": "{:.1f}", "share of blocks full": "{:.3f}", "peak M_N (MB)": "{:.3g}",
                      "user mean wait": "{:.2f}", "user fee/tx before": "{:.2e}", "user fee/tx during": "{:.2e}", "users' extra XMR/day": "{:,.2f}"}))
    base = g[g.strategy == "fill_to_L0"].set_index(["rules", "days"])
    rat = []
    for dd_ in [1, 7, 30]:
        rat.append({"days": dd_, "v16 (XMR)": base.spent[("v16", dd_)], "v17 (XMR)": base.spent[("v17", dd_)],
                    "v17 / v16": base.spent[("v17", dd_)] / base.spent[("v16", dd_)],
                    "users' extra XMR/day v16": base.uextra[("v16", dd_)], "users' extra XMR/day v17": base.uextra[("v17", dd_)]})
    out.append("\nCheapest way to keep blocks at M_N (fill_to_L0): v17 vs v16, and what it costs users\n")
    out.append(md(pd.DataFrame(rat), {"v17 / v16": "{:.2f}", "v16 (XMR)": "{:,.1f}", "v17 (XMR)": "{:,.1f}", "users' extra XMR/day v16": "{:,.2f}",
                                      "users' extra XMR/day v17": "{:,.2f}"}))
    for r in ["v16", "v17"]:
        x = g[(g.rules == r) & (g.strategy == "fill_to_L0") & (g["spam.blocks"] == 5040)].iloc[0]
        key("Q3", f"{r} keep blocks at M_N (fill_to_L0, 7 d): XMR/day; users' extra XMR/day; user fee/tx before→during",
            f"{x['XMR/day']:.2f}; {x.uextra:.2f}; {x.ufee_pre:.2e}→{x.ufee:.2e}")
        for s_ in ["topup_L0", "topup_auto"]:
            x = g[(g.rules == r) & (g.strategy == s_) & (g["spam.blocks"] == 21600)].iloc[0]
            key("Q3", f"{r} {s_} 30 d: XMR total, XMR/day, peak M_N MB", f"{x.spent:,.0f}, {x['XMR/day']:.1f}, {x.max_mn:.3g}")

    b = load("q3b_spam_growth")
    b["rules"], b["strategy"] = b.c0, b.c1
    rows = []
    for (r, s), gg in b.groupby(["rules", "strategy"]):
        # Days are counted from the start of the campaign (win_t_*), not from block 0.
        rows.append({"rules": r, "strategy": s, "M_L 2× (days)": days(gg.win_t_m_l_2x.mean()), "XMR to 2×": gg.spam_xmr_at_m_l_2x.mean(),
                     "M_L 4× (days)": days(gg.win_t_m_l_4x.mean()), "XMR to 4×": gg.spam_xmr_at_m_l_4x.mean(),
                     "seeds reaching 2×": int(gg.t_m_l_2x.notna().sum()), "M_L after 2 yr (MB)": gg.win_end_m_l.mean() / 1e6,
                     "spent 2 yr (XMR)": gg.spam_spent_in_window_xmr.mean(), "spam added 2 yr (GB)": gg.spam_bytes.mean() / 1e9,
                     "XMR/GB": gg.spam_xmr_per_gb.mean(), "M_L 12 mo after (MB)": gg.win_plus12mo_m_l.mean() / 1e6})
    tb = pd.DataFrame(rows)
    out.append(f"\n### Q3b/c: spam campaigns over 2 years (users at 38 tx/block, {nseeds(b)} seeds)\n")
    out.append("`clamp_L0`: fill each block to the clamp ratio × M_L (1.7× v16, 1.2× v17) at the lowest fee — the long-term weight can't count more. "
               "`mn_L0`: fill to M_N only. `topup_L0`: keep a block of backlog. `clamp_top`: as clamp_L0 at the highest wallet level. "
               "`surgecap_L0`: fill to 8 × M_L.\n")
    out.append(md(tb, {"M_L 2× (days)": "{:,.0f}", "XMR to 2×": "{:,.0f}", "M_L 4× (days)": "{:,.0f}", "XMR to 4×": "{:,.0f}", "M_L after 2 yr (MB)": "{:.3g}",
                       "spent 2 yr (XMR)": "{:,.0f}", "spam added 2 yr (GB)": "{:,.0f}", "XMR/GB": "{:.3g}", "M_L 12 mo after (MB)": "{:.3g}"}))
    for r in ["v16", "v17"]:
        x = tb[(tb.rules == r) & (tb.strategy == "clamp_L0")].iloc[0]
        key("Q3", f"{r} clamp_L0: XMR to 2× M_L (days); XMR to 4×", f"{x['XMR to 2×']:,.0f} ({x['M_L 2× (days)']:,.0f} d); {x['XMR to 4×']:,.0f}")
        x = tb[(tb.rules == r) & (tb.strategy == "clamp_top")].iloc[0]
        key("Q3", f"{r} clamp_top: XMR to 2× M_L", x["XMR to 2×"], "{:,.0f}")
    rows = []
    curves = {}
    for r in ["v16", "v17"]:
        for s in ["mn_L0", "clamp_L0", "topup_L0", "surgecap_L0"]:
            x = b[(b.rules == r) & (b.strategy == s) & (b.seed_index == 0)].iloc[0]
            curves[(r, s)] = series("q3b_spam_growth", int(x.run_id))
        for N in [1, 10, 100, 1000, 10000]:
            best = None
            for s in ["mn_L0", "clamp_L0", "topup_L0", "surgecap_L0"]:
                ss = curves[(r, s)]
                hit = ss[ss.cum_spam_bytes >= N * 1e9]
                if len(hit):
                    h = hit.iloc[0]
                    cand = (h.cum_spam_xmr, s, h.block / DAY)
                    if best is None or cand[0] < best[0]:
                        best = cand
            rows.append({"rules": r, "GB added": N, "cheapest XMR": best[0] if best else float("nan"), "strategy": best[1] if best else "not reached in 2 yr",
                         "days": best[2] if best else float("nan")})
    tc = pd.DataFrame(rows)
    out.append("\n### Cost to add N GB of spam to the chain: cheapest of the four lowest-fee strategies above, within 2 years (first seed)\n")
    out.append(md(tc, {"cheapest XMR": "{:,.0f}", "days": "{:,.0f}", "GB added": "{:,.0f}"}))
    for r in ["v16", "v17"]:
        for N in [10, 100, 1000]:
            x = tc[(tc.rules == r) & (tc["GB added"] == N)].iloc[0]
            key("Q3", f"{r} add {N} GB: cheapest XMR (strategy, days)", f"{x['cheapest XMR']:,.0f} ({x.strategy}, {x.days:,.0f} d)")

    fig, axes = plt.subplots(1, 2, figsize=(10, 3.6))
    for (r, s), ss in curves.items():
        ls = {"mn_L0": ":", "clamp_L0": "-", "topup_L0": "--", "surgecap_L0": "-."}[s]
        axes[0].plot(ss.cum_spam_xmr, ss.m_l / ss.m_l.iloc[0], ls=ls, color=C[r], label=f"{r} {s}")
        axes[1].plot(ss.cum_spam_bytes / 1e9, ss.cum_spam_xmr, ls=ls, color=C[r], label=f"{r} {s}")
    axes[0].set_xscale("symlog", linthresh=10); axes[0].set_xlabel("XMR spent by the spammer"); axes[0].set_ylabel("M_L / initial M_L")
    axes[0].set_yscale("log"); axes[0].set_title("M_L raised vs XMR spent", loc="left")
    axes[1].set_xscale("log"); axes[1].set_yscale("log"); axes[1].set_xlabel("GB of spam in the chain"); axes[1].set_ylabel("cumulative XMR spent")
    # Headroom for the legend: after the scales are set, so the log axes keep positive limits.
    for ax in axes:
        lo, hi = ax.get_ylim()
        ax.set_ylim(lo, hi * 1.6)
    axes[1].set_title("Bloat: XMR spent vs GB added", loc="left")
    axes[0].legend(fontsize=7, ncol=1)
    fig.suptitle("Q3: fee-paying spammer, 2-year campaigns (first seed)", x=0.01, y=1.03, ha="left")
    savefig(fig, "q3_spam.png")

    m = load("q3d_miner")
    m["rules"], m["target"] = m.c0, m.c1
    gm = m.groupby(["rules", "target", "miner_attack.share"]).agg(blocks=("miner_blocks", "mean"), pen=("miner_penalty_extra_xmr", "mean"),
          pad=("padding_bytes", "mean"), ml_end=("win_end_m_l", "mean"), ml_ratio=("win_m_l_ratio", "mean"), ml_ratio_min=("win_m_l_ratio", "min"),
          ml_ratio_max=("win_m_l_ratio", "max"), max_mn=("max_m_n", "mean"),
          reached2=("t_m_l_2x", lambda x: int(x.notna().sum())), n=("seed", "count")).reset_index()
    gm["pad"] /= 1e9
    gm["xmr_per_gb"] = gm.pen / gm.pad
    t = gm[["rules", "target", "miner_attack.share", "pen", "pad", "xmr_per_gb", "ml_ratio", "ml_ratio_min", "ml_ratio_max", "max_mn", "reached2", "n"]].copy()
    t["max_mn"] /= 1e6
    t.columns = ["rules", "padding target", "hashrate h", "penalty paid (XMR)", "padding (GB)", "XMR/GB", "M_L after 1 yr ÷ before (mean)", "min", "max", "peak M_N (MB)", "seeds reaching 2×", "seeds"]
    out.append(f"\n### Q3d: malicious miner padding its own blocks with zero-fee filler for 1 year, honest demand 38 tx/block ({nseeds(m)} seeds)\n")
    out.append("Honest blocks carry 38 tx/block: 95 kB under v16 (0.32·Z_M) and 345 kB under v17 (0.55·Z_M), well below M_L. "
               "`pad_mn`: pad to M_N (free). `pad_clamp`: pad to the clamp ratio × M_L. `pad_max`: pad to 2·M_N (forfeits the whole reward). "
               "Penalty paid = the penalty above what its block would have paid without padding.\n")
    out.append(md(t, {"penalty paid (XMR)": "{:,.0f}", "padding (GB)": "{:,.1f}", "XMR/GB": "{:,.1f}", "M_L after 1 yr ÷ before (mean)": "{:.3f}", "min": "{:.3f}", "max": "{:.3f}", "peak M_N (MB)": "{:.3g}"}))
    for r in ["v16", "v17"]:
        for h in [0.51, 0.6]:
            x = gm[(gm.rules == r) & (gm.target == "pad_clamp") & (gm["miner_attack.share"] == h)].iloc[0]
            key("Q3", f"{r} miner h={h} pad_clamp: M_L ×, penalty XMR", f"{x.ml_ratio:.3f}, {x.pen:,.0f}")
        x = gm[(gm.rules == r) & (gm.target == "pad_mn") & (gm["miner_attack.share"] == 0.51)].iloc[0]
        key("Q3", f"{r} miner h=0.51 pad_mn: free padding GB/yr", x.pad, "{:.1f}")

    e = load("q3e_miner_busy")
    e["rules"] = e.c0
    ge = e.groupby(["rules", "demand.zm_mult", "miner_attack.share"]).agg(ml=("win_m_l_ratio", "mean"), ml_min=("win_m_l_ratio", "min"),
          ml_max=("win_m_l_ratio", "max"), pen=("miner_penalty_extra_xmr", "mean")).reset_index()
    rows = []
    for (r, z), g in ge.groupby(["rules", "demand.zm_mult"]):
        row = {"rules": r, "honest demand ×Z_M": z}
        for _, x in g.iterrows():
            h = x["miner_attack.share"]
            row[f"M_L ×, h={h:g}"] = x.ml
            if h > 0:
                row[f"penalty h={h:g} (XMR)"] = x.pen
        rows.append(row)
    te = pd.DataFrame(rows)
    out.append(f"\n### Q3e: minority miner padding to the clamp for 1 year when honest demand is close to Z_M ({nseeds(e)} seeds)\n")
    out.append("M_L at the end of the attack ÷ M_L at its start (mean over seeds); h = 0 is the same chain without an attacker.\n")
    out.append(md(te, {c: "{:.3f}" for c in te.columns if c.startswith("M_L")} | {c: "{:,.0f}" for c in te.columns if c.startswith("penalty")}))
    for r in ["v16", "v17"]:
        for z in [0.8, 0.9, 0.95]:
            x = te[(te.rules == r) & (te["honest demand ×Z_M"] == z)].iloc[0]
            key("Q3", f"{r} honest {z}·Z_M: M_L × with h=0 / 0.3 / 0.45", f"{x['M_L ×, h=0']:.3f} / {x['M_L ×, h=0.3']:.3f} / {x['M_L ×, h=0.45']:.3f}")
    write("q3.md", "\n".join(out))

    fig, axes = plt.subplots(1, 2, figsize=(10, 3.4))
    for r in ["v16", "v17"]:
        for tg, ls in [("pad_clamp", "-"), ("pad_max", "--")]:
            g = gm[(gm.rules == r) & (gm.target == tg)]
            axes[0].plot(g["miner_attack.share"], g.ml_ratio, ls=ls, marker="o", ms=4, color=C[r], label=f"{r} {tg}")
            axes[1].plot(g["miner_attack.share"], g.pen, ls=ls, marker="o", ms=4, color=C[r], label=f"{r} {tg}")
    axes[0].set_xlabel("attacker hashrate share h"); axes[0].set_ylabel("M_L after 1 year ÷ M_L before"); axes[0].set_title("M_L growth (honest demand 38 tx/block)", loc="left")
    axes[1].set_xlabel("attacker hashrate share h"); axes[1].set_ylabel("extra penalty paid in 1 year, XMR"); axes[1].set_yscale("log"); axes[1].set_title("cost", loc="left")
    axes[0].axvline(0.5, color=INK2, lw=0.8, alpha=0.5)
    axes[1].axvline(0.5, color=INK2, lw=0.8, alpha=0.5)
    axes[0].legend(fontsize=8)
    fig.text(0.01, -0.04, "Left: v16 pad_clamp and pad_max coincide (both exceed the clamp). Right: pad_max overlaps for v16 and v17 (each padded block forfeits the 0.6 XMR reward).", fontsize=7, color=INK2)
    fig.suptitle(f"Q3d: malicious miner padding its blocks for one year ({nseeds(m)} seeds)", x=0.01, y=1.03, ha="left")
    savefig(fig, "q3_miner.png")


# ----------------------------------------------------------------------------------------
# Q4 recovery
# ----------------------------------------------------------------------------------------
def q4():
    d = load("q4_recovery")
    d["rules"], d["miners"] = d.c0, d.c1
    rows = []
    for (r, mn, fu), g in d.groupby(["rules", "miners", "demand.fill_until"]):
        row = {"rules": r, "miners": mn, "growth phase (days)": fu / DAY, "M_L at end (MB)": g.win_end_m_l.mean() / 1e6,
               "fee_low at end ÷ base": g.win_end_fee_low.astype(float).mean() / g.win_base_fee_low.astype(float).mean(),
               "M_N back ≤1.1×base (days)": days(g.win_rec_m_n.mean()), "M_L back ≤1.1×base (days)": days(g.win_rec_m_l.mean()),
               "fee_low back ≥0.9×base (days)": days(g.win_rec_fee_low.mean())}
        for mo in [1, 3, 6, 12]:
            row[f"M_L +{mo} mo (MB)"] = g[f"win_plus{mo}mo_m_l"].mean() / 1e6
        rows.append(row)
    t = pd.DataFrame(rows)
    out = [f"### Q4: recovery after demand above capacity stops (back to 38 tx/block; monerod pool limits; {nseeds(d)} seeds)\n",
           md(t, {c: "{:.3g}" for c in t.columns if "MB" in c} | {c: "{:,.1f}" for c in t.columns if "days" in c} | {"fee_low at end ÷ base": "{:.3g}"})]
    for r in ["v16", "v17"]:
        x = t[(t.rules == r) & (t.miners == "auto") & (t["growth phase (days)"] == 365)].iloc[0]
        key("Q4", f"{r} after 1-yr growth: M_L at end MB; +6 mo MB; M_N back (days); fee at end ÷ base",
            f"{x['M_L at end (MB)']:.3g}; {x['M_L +6 mo (MB)']:.3g}; {x['M_N back ≤1.1×base (days)']:.1f}; {x['fee_low at end ÷ base']:.3g}")
        x = t[(t.rules == r) & (t.miners == "auto") & (t["growth phase (days)"] == 30)].iloc[0]
        key("Q4", f"{r} after 30-d growth: M_N back (days)", x["M_N back ≤1.1×base (days)"], "{:.1f}")
    b = load("q3b_spam_growth")
    b["rules"], b["strategy"] = b.c0, b.c1
    rows = []
    for (r, s), g in b.groupby(["rules", "strategy"]):
        row = {"rules": r, "after 2-yr spam": s, "M_L at end (MB)": g.win_end_m_l.mean() / 1e6}
        for mo in [1, 3, 6, 12]:
            row[f"M_L +{mo} mo (MB)"] = g[f"win_plus{mo}mo_m_l"].mean() / 1e6
            row[f"fee_low +{mo} mo"] = g[f"win_plus{mo}mo_fee_low"].astype(float).mean()
        rows.append(row)
    t = pd.DataFrame(rows)
    out.append("\n### After the 2-year spam campaigns of Q3b\n")
    out.append(md(t, {c: "{:.3g}" for c in t.columns if "MB" in c} | {c: "{:.2e}" for c in t.columns if "fee" in c}))
    write("q4.md", "\n".join(out))

    fig, axes = plt.subplots(1, 3, figsize=(11, 3.3))
    for r in ["v16", "v17"]:
        for fu, ls in [(30 * DAY, ":"), (90 * DAY, "--"), (YEAR, "-")]:
            x = d[(d.rules == r) & (d.miners == "auto") & (d["demand.fill_until"] == fu) & (d.seed_index == 0)].iloc[0]
            s = series("q4_recovery", int(x.run_id))
            t = (s.block - fu) / MONTH
            k = s.block >= fu - 10 * DAY
            lab = f"{r}, {fu//DAY} d growth"
            axes[0].plot(t[k], s.m_l[k] / s.m_l.iloc[0], ls=ls, color=C[r], label=lab)
            axes[1].plot(t[k], s.m_n[k] / s.m_n.iloc[0], ls=ls, color=C[r], label=lab)
            axes[2].plot(t[k], s.fee_low[k] / s.fee_low.iloc[0], ls=ls, color=C[r], label=lab)
    for ax, ttl in zip(axes, ["M_L ÷ before", "M_N ÷ before", "lowest fee level ÷ before"]):
        ax.set_yscale("log"); ax.set_title(ttl, loc="left"); ax.set_xlabel("months after demand returns to 38 tx/block")
        lo, hi = ax.get_ylim()
        ax.set_ylim(lo / 1.3, hi * 1.6)
    axes[0].legend(fontsize=7)
    fig.suptitle("Q4: recovery after periods of demand above capacity (rational miners, auto wallets)", x=0.01, y=1.03, ha="left")
    savefig(fig, "q4_recovery.png")


# ----------------------------------------------------------------------------------------
# Q5 fee market
# ----------------------------------------------------------------------------------------
def q5_rows(d, horizon):
    rows = []
    for (r, f, z), g in d.groupby(["rules", "fee", "demand.zm_mult"]):
        moving = (g.ss_m_l_end / g.ss_m_l_start - 1).abs().max()
        rows.append({"rules": r, "wallets": f, "demand ×Z_M": z,
                     "M_L (MB)": g.ss_m_l_end.mean() / 1e6, "M_L still moving": "yes" if moving > 0.01 else "no",
                     "fee/tx (XMR)": g.ss_fee_per_tx.astype(float).mean(), "mean wait": g.ss_wait_mean.astype(float).mean(),
                     "p99 wait": g.ss_wait_p99.astype(float).mean(), "max wait": g.ss_wait_max.max(), "mined": g.ss_mined_share.astype(float).mean(),
                     "blocks > M_N": g.ss_over_mn_share.astype(float).mean(), "M_B/M_N when over": g.ss_over_mn_mean_ratio.astype(float).mean(),
                     "mean M_N (MB)": g.ss_m_n_mean.astype(float).mean() / 1e6})
    t = pd.DataFrame(rows)
    if horizon == "transient":
        t = t.drop(columns=["M_L still moving"])
    return t


Q5FMT = {"M_L (MB)": "{:.3g}", "fee/tx (XMR)": "{:.2e}", "mean wait": "{:.3f}", "p99 wait": "{:.1f}", "mined": "{:.3f}", "blocks > M_N": "{:.3f}",
         "M_B/M_N when over": "{:.4f}", "mean M_N (MB)": "{:.3g}"}


def q5():
    d = load("q5_fee_market")
    d["rules"], d["fee"] = d.c0, d.c1
    eq = q5_rows(d, "equilibrium")
    eq.to_csv(os.path.join(TAB, "q5_fee_market_all.csv"), index=False)
    s = load("q5c_step")
    s["rules"], s["fee"] = s.c0, s.c1
    tr = q5_rows(s, "transient")
    out = []
    for r in ["v16", "v17"]:
        out.append(f"\n### Q5 equilibrium, {r}: constant Poisson demand at multiples of Z_M, measured over blocks "
                   f"{int(d.ss_from.iloc[0]):,}–{int(d.blocks.iloc[0]):,} ({nseeds(d)} seeds)\n")
        out.append("`M_L still moving`: M_L changed by more than 1% over the measurement window in some seed (not yet at equilibrium).\n")
        out.append(md(eq[eq.rules == r].drop(columns="rules"), Q5FMT))
    for r in ["v16", "v17"]:
        out.append(f"\n### Q5 transient, {r}: the first {int(s.blocks.iloc[0]):,} blocks after demand jumps to a multiple of Z_M ({nseeds(s)} seeds)\n")
        out.append("Same demand, starting from a chain at steady state. Fees and waits over the whole period, including the ramp.\n")
        out.append(md(tr[tr.rules == r].drop(columns="rules"), Q5FMT))
    for r in ["v16", "v17"]:
        for z in [0.5, 1.0, 2.0, 5.0]:
            x = eq[(eq.rules == r) & (eq.wallets == "auto") & (eq["demand ×Z_M"] == z)].iloc[0]
            y = tr[(tr.rules == r) & (tr.wallets == "auto") & (tr["demand ×Z_M"] == z)].iloc[0]
            key("Q5", f"{r} auto {z}·Z_M: fee/tx equilibrium (M_L MB, moving?) | first 139 d (p99 wait)",
                f"{x['fee/tx (XMR)']:.2e} ({x['M_L (MB)']:.3g}, {x['M_L still moving']}) | {y['fee/tx (XMR)']:.2e} ({y['p99 wait']:.1f})")
        x = eq[(eq.rules == r) & (eq.wallets == "auto") & (eq["demand ×Z_M"] >= 1.0)]
        key("Q5", f"{r} auto ≥ Z_M at equilibrium: share of blocks > M_N; mean M_B/M_N when over",
            f"{x['blocks > M_N'].min():.3f}–{x['blocks > M_N'].max():.3f}; {x['M_B/M_N when over'].min():.4f}–{x['M_B/M_N when over'].max():.4f}")

    R = 0.6
    rows = []
    for rules, zm, tr_, t_ in [("v16", 300000, 3000, 2500), ("v17", 625000, 12500, 9093)]:
        fl = R * tr_ / zm**2
        levels = {"v16": [fl, 4 * fl, 16 * R * tr_ / (zm * zm), 4 * 16 * R * tr_ / (zm * zm)], "v17": [fl, 4 * fl, 16 * fl, 64 * fl, 2 * R / zm]}[rules]
        names = ["L0", "L1", "L2", "L3", "L4"]
        for k, f in enumerate(levels):
            for ratio in [1, 2, 4, 8]:
                mn = ratio * zm
                fee = f * t_
                root = mn * math.sqrt(fee / R)
                xmax = mn + (fee * mn * mn / R - t_ * t_) / (2 * t_) if root >= t_ else mn - t_ + root
                end = min(xmax + t_, 2 * mn) if root >= t_ else mn + root
                rows.append({"rules": rules, "level": names[k], "fee/byte": f, "M_N ÷ M_L": ratio,
                             "largest block (÷ M_N)": min(2.0, max(end, mn) / mn)})
    an = pd.DataFrame(rows).pivot_table(index=["rules", "level", "fee/byte"], columns="M_N ÷ M_L", values="largest block (÷ M_N)").reset_index()
    an.columns = [str(c) if not isinstance(c, (int, float)) else f"M_N = {c}·M_L" for c in an.columns]
    out.append("\n### How far past M_N a rational miner goes when the pool is deep (analytic, from affordable_txs)\n")
    out.append("Largest block weight ÷ M_N when every waiting transaction pays the given level, with M_L = Z_M (fees depend on M_L only in v17; "
               "in v16 the levels above L1 also depend on the wallet's M_N, which this table holds at M_L). "
               "Values above 1 are the 'slightly exceed M_N' mechanism; when they are above 1 at M_N = M_L, the short-term median ratchets up while the backlog lasts.\n")
    out.append(md(an, {"fee/byte": "{:.2e}"} | {c: "{:.4f}" for c in an.columns if c.startswith("M_N =")}))

    q = load("q5b_daily")
    q["rules"], q["fee"] = q.c0, q.c1
    q["dropped"] = (q.evicted_user_txs + q.expired_user_txs + q.rejected_user_txs) / q.arrived_user_txs
    g = q.groupby(["rules", "fee", "demand.zm_mult", "demand.daily_amplitude"]).agg(fee_tx=("user_fee_per_tx", "mean"), wait=("wait_mean", "mean"),
         p99=("wait_p99", "mean"), wmax=("wait_max", "max"), dropped=("dropped", "mean"), mn=("final_m_n", "mean")).reset_index()
    g["mn"] /= 1e6
    g.columns = ["rules", "wallets", "mean demand ×Z_M", "daily amplitude", "fee/tx (XMR)", "mean wait", "p99 wait", "max wait", "dropped share", "M_N at end (MB)"]
    out.append(f"\n### Q5b: daily demand cycle (720-block period), {nseeds(q)} seeds\n")
    out.append(md(g, {"fee/tx (XMR)": "{:.2e}", "mean wait": "{:.3f}", "p99 wait": "{:.1f}", "dropped share": "{:.4f}", "M_N at end (MB)": "{:.3g}"}))
    for z in [1.0, 1.5]:
        x = g[(g.rules == "v17") & (g.wallets == "auto") & (g["mean demand ×Z_M"] == z) & (g["daily amplitude"] >= 0.5)]
        key("Q5", f"v17 auto daily cycle at {z}·Z_M (amplitude 0.5–0.9): mean wait; p99; max",
            f"{x['mean wait'].min():.1f}–{x['mean wait'].max():.1f}; {x['p99 wait'].min():.0f}–{x['p99 wait'].max():.0f}; {x['max wait'].min():,.0f}–{x['max wait'].max():,.0f}")
        y = g[(g.rules == "v17") & (g.wallets == "auto_upm") & (g["mean demand ×Z_M"] == z) & (g["daily amplitude"] >= 0.5)]
        ratio = y["fee/tx (XMR)"].values / x["fee/tx (XMR)"].values
        key("Q5", f"v17 users-pay-more at {z}·Z_M (amplitude 0.5–0.9): mean wait; fee ÷ auto fee",
            f"{y['mean wait'].min():.1f}–{y['mean wait'].max():.1f}; {ratio.min():.1f}–{ratio.max():.1f}")
    write("q5.md", "\n".join(out))

    fig, axes = plt.subplots(1, 2, figsize=(10, 3.4))
    cols = blues(5)
    for r, ls in [("v16", "--"), ("v17", "-")]:
        for k, f in enumerate(["L0", "L1", "L2", "L3", "L4"]):
            if r == "v16" and f == "L4":
                continue
            g = eq[(eq.rules == r) & (eq.wallets == f)]
            axes[0].plot(g["demand ×Z_M"], g["fee/tx (XMR)"], ls=ls, marker="o", ms=3, color=cols[k], label=f"{r} {f}")
            g = tr[(tr.rules == r) & (tr.wallets == f)]
            axes[1].plot(g["demand ×Z_M"], g["p99 wait"].clip(lower=0.1), ls=ls, marker="o", ms=3, color=cols[k], label=f"{r} {f}")
        g = eq[(eq.rules == r) & (eq.wallets == "auto")]
        axes[0].plot(g["demand ×Z_M"], g["fee/tx (XMR)"], ls=ls, marker="s", ms=4, color=C["v16"], label=f"{r} auto")
        g = tr[(tr.rules == r) & (tr.wallets == "auto")]
        axes[1].plot(g["demand ×Z_M"], g["p99 wait"].clip(lower=0.1), ls=ls, marker="s", ms=4, color=C["v16"], label=f"{r} auto")
    axes[0].set_yscale("log"); axes[0].set_xlabel("demand ÷ Z_M"); axes[0].set_ylabel("fee per tx, XMR"); axes[0].set_title("fee per transaction at equilibrium", loc="left")
    axes[1].set_yscale("log"); axes[1].set_xlabel("demand ÷ Z_M"); axes[1].set_ylabel("p99 wait, blocks (0 → 0.1)"); axes[1].set_title("p99 wait, first 139 days after a demand step", loc="left")
    for ax in axes:
        log_x(ax, [0.5, 0.8, 1, 1.2, 2, 5])
    axes[1].legend(fontsize=6, ncol=1, loc="upper left", bbox_to_anchor=(1.02, 1.0))
    fig.suptitle("Q5: fee market (solid v17, dashed v16; darker blue = higher level; orange = auto)", x=0.01, y=1.03, ha="left")
    savefig(fig, "q5_fee_market.png")


# ----------------------------------------------------------------------------------------
# Q6 sensitivity
# ----------------------------------------------------------------------------------------
def q6():
    d = load("q6_sensitivity")
    d["txw"] = d.tx.str.extract(r"(\d+)").astype(int)
    P = {"params.z_m": "Z_M", "params.lt_growth": "clamp", "params.surge_factor": "surge", "params.t_r": "T_R", "txw": "tx weight"}
    M = [("growth", "t_maxw_10MB", "blocks to 10 MB max weight"), ("growth", "t_m_l_2x", "blocks to 2× M_L"),
         ("surge10x1d", "win_wait_mean", "10× 1-day surge: mean wait"), ("surge10x1d", "win_mined_share", "10× 1-day surge: share mined"),
         ("surge50x1d", "win_wait_mean", "50× 1-day surge: mean wait"), ("surge50x1d", "win_mined_share", "50× 1-day surge: share mined"),
         ("spam_mn_7d", "spam_spent_in_window_xmr", "spam at M_N 7 days: XMR"), ("spam_clamp", "spam_xmr_at_m_l_2x", "spam to 2× M_L: XMR"),
         ("spam_clamp", "t_m_l_2x", "spam to 2× M_L: blocks"), ("miner51_clamp", "miner_xmr_at_m_l_2x", "51% miner to 2× M_L: XMR"),
         ("miner51_clamp", "t_m_l_2x", "51% miner to 2× M_L: blocks")]
    rows, levels = [], []
    for case, m, label in M:
        s = d[d.case == case].copy()
        y = np.log10(s[m].astype(float) + 1)
        ok = y.notna()
        tot = y[ok].var(ddof=0)
        row = {"metric (log10(x+1))": label, "runs with a value": f"{ok.sum()}/{len(s)}"}
        for p, pn in P.items():
            sub = s[ok].assign(y=y[ok])
            gm = sub.groupby(p).y.mean()
            gn = sub.groupby(p).y.size()
            between = float((gn * (gm - sub.y.mean()) ** 2).sum() / gn.sum())  # size-weighted, <= total
            row[pn] = between / tot if tot > 0 else 0.0
            for lv, v in gm.items():
                levels.append({"metric": label, "param": pn, "value": lv, "mean of metric": 10 ** v - 1, "n": int((s[ok][p] == lv).sum())})
        row["sum"] = sum(row[pn] for pn in P.values())
        rows.append(row)
    t = pd.DataFrame(rows)
    out = ["### Q6: first-order sensitivity (share of the variance of log10(metric + 1) explained by each parameter's main effect)\n",
           "Full factorial: Z_M ∈ {300k, 625k, 1M} × clamp ∈ {1.1, 1.2, 1.4, 1.7} × surge factor ∈ {4, 8, 16, 50} × T_R ∈ {3k, 10k, 12.5k, 20k} × tx weight ∈ {7,650, 9,093, 11,851} = 576 rule sets, "
           "Custom mode (2026 fee algorithm), one seed per rule set, monerod pool limits. Each share is computed on its own. "
           "Runs where the metric never happened (e.g. M_L never doubled) are left out of that metric; the count is shown. "
           "`sum` below 1 is variance the main effects don't explain (interactions, such as T_R × transaction weight); "
           "above 1 means leaving runs out has unbalanced the design, so the shares overlap. "
           "Two rows hold by construction: the clamp sets the doubling time, and the minimum fee, hence spam cost, is proportional to T_R.\n",
           md(t, {pn: "{:.2f}" for pn in list(P.values()) + ["sum"]})]
    lv = pd.DataFrame(levels)
    lv.to_csv(os.path.join(TAB, "q6_main_effects.csv"), index=False)
    out.append("\n### Main effects (geometric mean of the metric at each parameter value, over the other parameters)\n")
    for label in [m[2] for m in M]:
        x = lv[lv.metric == label]
        parts = []
        for pn in P.values():
            y = x[x.param == pn]
            parts.append(f"**{pn}**: " + ", ".join(f"{v:g}→{mm:,.3g}" for v, mm in zip(y.value, y["mean of metric"])))
        out.append(f"- *{label}* — " + "; ".join(parts))
    s = d[d.case == "spam_clamp"]
    cens = s.assign(reached=s.t_m_l_2x.notna()).groupby(["params.t_r", "txw"]).reached.mean().unstack()
    out.append(f"\nShare of rule sets in which the minimum-fee spammer doubled M_L within {int(s.blocks.iloc[0]):,} blocks, by T_R (rows) and tx weight (columns):\n")
    cens = cens.reset_index()
    cens.columns = [str(c) for c in cens.columns]
    out.append(md(cens, {c: "{:.2f}" for c in cens.columns[1:]}))
    write("q6.md", "\n".join(out))
    for _, x in t.iterrows():
        top = max(P.values(), key=lambda pn: x[pn])
        key("Q6", f"{x['metric (log10(x+1))']}: largest share; sum", f"{top} {x[top]:.2f}; {x['sum']:.2f}")
    key("Q6", "share of rule sets where the min-fee spammer doubled M_L, T_R = 3,000", cens[cens["params.t_r"] == "3000"].iloc[0, 1:].astype(float).mean() if "3000" in set(cens["params.t_r"]) else cens.iloc[0, 1:].astype(float).mean(), "{:.2f}")

    fig, ax = plt.subplots(figsize=(7.5, 4.4))
    mat = t[list(P.values())].values
    im = ax.imshow(mat, cmap="Blues", vmin=0, vmax=1, aspect="auto")
    ax.set_xticks(range(len(P))); ax.set_xticklabels(list(P.values()))
    ax.set_yticks(range(len(t))); ax.set_yticklabels(t["metric (log10(x+1))"])
    ax.grid(False)
    for i in range(mat.shape[0]):
        for j in range(mat.shape[1]):
            ax.text(j, i, f"{mat[i, j]:.2f}", ha="center", va="center", fontsize=7, color=INK if mat[i, j] < 0.6 else SURF)
    fig.colorbar(im, ax=ax, fraction=0.04, label="share of variance (main effect)")
    ax.set_title("Q6: which parameter matters most (576 rule sets)", loc="left")
    savefig(fig, "q6_sensitivity.png")


# ----------------------------------------------------------------------------------------
# Q7 fork transition
# ----------------------------------------------------------------------------------------
VARIANTS = ["ref", "spec", "spec_reclamped"]


def q7():
    d = load("q7_fork")
    d["pre"], d["wallets"], d["variant"] = d.c0, d.c1, d.c2
    e = events("q7_fork").merge(d[["run_id", "pre", "wallets", "variant"]], on="run_id")
    out = []
    rows = []
    for (pre, w), g in e.groupby(["pre", "wallets"], sort=False):
        for ev in ["fork_last_pre", "fork_first_block", "fork_plus_10000", "fork_plus_50000", "fork_plus_100000"]:
            v = {k: g[(g.event == ev) & (g.variant == k)] for k in VARIANTS}
            if any(len(x) == 0 for x in v.values()):
                continue
            a, b, c = (v[k].iloc[0] for k in VARIANTS)
            rows.append({"pre-fork / demand": pre, "miners": w, "when": ev.replace("fork_", ""), "M_L ref (kB)": a.m_l / 1e3, "M_L spec (kB)": b.m_l / 1e3,
                         "M_L spec re-clamped (kB)": c.m_l / 1e3, "max weight ref (MB)": a.max_weight / 1e6, "max weight spec (MB)": b.max_weight / 1e6,
                         "f_L ref": a.fee_low, "f_L spec": b.fee_low, "f_L ref ÷ spec": a.fee_low / b.fee_low})
    t = pd.DataFrame(rows)
    fmt = {"M_L ref (kB)": "{:,.0f}", "M_L spec (kB)": "{:,.0f}", "M_L spec re-clamped (kB)": "{:,.0f}", "max weight ref (MB)": "{:.3g}", "max weight spec (MB)": "{:.3g}",
           "f_L ref": "{:.2e}", "f_L spec": "{:.2e}", "f_L ref ÷ spec": "{:.2f}"}
    out.append("### Q7: v16 → v17 at block 2,000; reference code (no rescaling, `ref`) vs the spec's transitional rule (pre-fork block weights × 625/300 in every median spanning the fork)\n")
    out.append("`spec` multiplies the stored pre-fork long-term weights; `spec re-clamped` recomputes them from the scaled block weights with v17's clamp around the scaled pre-fork M_L. "
               "Both scale the short-term window's block weights. Pre-fork windows filled with M_L = X (v16, 2,500-byte transactions). After the fork transactions weigh 9,093 bytes. "
               "`same_txs` keeps the transaction count (X/2,500 per block, so bytes ×3.6); `base38` drops to 38 tx/block; `fill_after` = rational miners with demand above capacity after the fork.\n")
    out.append(md(t[t.miners == "auto"].drop(columns="miners"), fmt))
    out.append("\nWith demand above capacity after the fork:\n")
    out.append(md(t[(t.miners == "fill_after") & t["pre-fork / demand"].str.endswith("base38")].drop(columns="miners"), fmt))
    for X in ["300k", "450k", "600k", "1000k", "2000k"]:
        x = t[(t["pre-fork / demand"] == f"ML{X}_base38") & (t.miners == "auto") & (t.when == "first_block")].iloc[0]
        key("Q7", f"pre-fork M_L {X}: M_L ref / spec / re-clamped (kB); f_L ref ÷ spec at the first post-fork block",
            f"{x['M_L ref (kB)']:,.0f} / {x['M_L spec (kB)']:,.0f} / {x['M_L spec re-clamped (kB)']:,.0f}; {x['f_L ref ÷ spec']:.2f}")

    rows = []
    for (pre, w), g in d.groupby(["pre", "wallets"], sort=False):
        a = g[g.variant == "ref"].iloc[0]
        sa = series("q7_fork", int(a.run_id))
        sa = sa[sa.block >= 2000]
        for var in ["spec", "spec_reclamped"]:
            b = g[g.variant == var].iloc[0]
            sb = series("q7_fork", int(b.run_id))
            sb = sb[sb.block >= 2000]
            assert (sa.block.values == sb.block.values).all()
            diff_lim = (sa.max_weight.values != sb.max_weight.values)
            diff_fee = (sa.fee_low.values != sb.fee_low.values)
            diff_ml = (sa.m_l.values != sb.m_l.values)
            last = lambda m: (sa.block.values[m].max() - 2000) if m.any() else 0
            rows.append({"pre-fork / demand": pre, "miners": w, "variant": var, "limit differs (share of samples)": diff_lim.mean(), "last block limit differs": last(diff_lim),
                         "fee differs (share)": diff_fee.mean(), "last block fee differs": last(diff_fee), "M_L differs (share)": diff_ml.mean(),
                         "max |log2 limit ratio|": np.abs(np.log2(sa.max_weight.values / sb.max_weight.values)).max(),
                         "max fee ratio ref÷spec": (sa.fee_low.values / sb.fee_low.values).max(), "min fee ratio": (sa.fee_low.values / sb.fee_low.values).min()})
    dv = pd.DataFrame(rows)
    out.append(f"\n### How long and how much the behaviours differ ({int(d.blocks.iloc[0]) - 2000:,} blocks after the fork, series sampled every 10 blocks)\n")
    out.append(md(dv, {"limit differs (share of samples)": "{:.3f}", "fee differs (share)": "{:.3f}", "M_L differs (share)": "{:.3f}", "max |log2 limit ratio|": "{:.3f}",
                       "max fee ratio ref÷spec": "{:.2f}", "min fee ratio": "{:.2f}", "last block limit differs": "{:,.0f}", "last block fee differs": "{:,.0f}"}))

    b = load("q7b_fork_mainnet")
    b["demand"], b["variant"] = b.c0, b.c1
    rows = []
    for (c, v), g in b.groupby(["demand", "variant"], sort=False):
        rows.append({"post-fork demand": c, "variant": v, "M_L end (kB)": g.final_m_l.mean() / 1e3, "max M_N (MB)": g.max_m_n.mean() / 1e6,
                     "fee/tx (XMR)": g.user_fee_per_tx.astype(float).mean(), "chain GB": g.chain_bytes.mean() / 1e9, "t M_L 2× (days)": days(g.t_m_l_2x.mean())})
    out.append(f"\n### Fork from the real mainnet windows (last 100,000 blocks to height 3,797,557), {nseeds(b)} seeds\n")
    out.append(md(pd.DataFrame(rows), {"M_L end (kB)": "{:,.0f}", "max M_N (MB)": "{:.3g}", "fee/tx (XMR)": "{:.3e}", "chain GB": "{:,.1f}", "t M_L 2× (days)": "{:,.1f}"}))
    eb = events("q7b_fork_mainnet").merge(b[["run_id", "demand", "variant", "seed"]], on="run_id")
    eb = eb[eb.event.str.startswith("fork_plus")]
    piv = eb.pivot_table(index=["demand", "seed", "event"], columns="variant", values=["m_l", "m_n", "fee_low"], aggfunc="first")
    rows = []
    for (c, ev), g in piv.groupby(level=["demand", "event"]):
        row = {"post-fork demand": c, "when": ev.replace("fork_", "")}
        for var in ["spec", "spec_reclamped"]:
            for col, nm in [("m_l", "M_L"), ("m_n", "M_N"), ("fee_low", "f_L")]:
                ratio = g[(col, var)] / g[(col, "ref")]
                row[f"{nm} {var} ÷ ref (min–max)"] = f"{ratio.min():.4f}–{ratio.max():.4f}"
                if c == "fill_after" and var == "spec" and ev in ("fork_plus_50000", "fork_plus_100000"):
                    key("Q7", f"mainnet fill_after, spec ÷ ref at {ev.replace('fork_', '')}: {nm}", f"{ratio.min():.4f}–{ratio.max():.4f}")
        rows.append(row)
    order = {f"plus_{k}": i for i, k in enumerate([100, 1000, 10000, 50000, 100000])}
    rt = pd.DataFrame(rows)
    rt = rt.sort_values(["post-fork demand", "when"], key=lambda s: s.map(order) if s.name == "when" else s)
    out.append(f"\nMainnet start: ratios to the reference code at fixed times after the fork, over {nseeds(b)} seeds\n")
    out.append(md(rt))
    rows = []
    for c in b.demand.unique():
        for si in sorted(b.seed_index.unique())[:2]:
            a = b[(b.demand == c) & (b.variant == "ref") & (b.seed_index == si)].iloc[0]
            sa = series("q7b_fork_mainnet", int(a.run_id))
            for var in ["spec", "spec_reclamped"]:
                bb = b[(b.demand == c) & (b.variant == var) & (b.seed_index == si)].iloc[0]
                sb = series("q7b_fork_mainnet", int(bb.run_id))
                dif = (sa.m_l.values != sb.m_l.values) | (sa.max_weight.values != sb.max_weight.values)
                first = int(sa.block.values[dif].min() - 2000) if dif.any() else "none"
                rows.append({"post-fork demand": c, "seed": int(a.seed), "variant": var, "series differ (share of samples)": dif.mean(),
                             "first difference (blocks after fork)": first, "max M_L ratio ÷ ref": (sb.m_l.values / sa.m_l.values).max()})
    fd = pd.DataFrame(rows)
    out.append("\nMainnet start: where the behaviours first differ (the two seeds with series)\n")
    out.append(md(fd, {"series differ (share of samples)": "{:.3f}", "max M_L ratio ÷ ref": "{:.4f}"}))
    write("q7.md", "\n".join(out))
    for c in ["base38", "x3", "fill_after"]:
        for var in ["spec", "spec_reclamped"]:
            x = fd[(fd["post-fork demand"] == c) & (fd.variant == var)]
            key("Q7", f"mainnet {c}, {var}: first difference (blocks after fork, per seed); max M_L ratio",
                f"{', '.join(str(v) for v in x['first difference (blocks after fork)'])}; {x['max M_L ratio ÷ ref'].max():.4f}")

    fig, axes = plt.subplots(2, 3, figsize=(11, 5.6), sharex=True)
    for row, wal in enumerate(["auto", "fill_after"]):
        for X, col in [("300k", "#52514e"), ("450k", C["v16"]), ("600k", C["v17_sf50"]), ("1000k", C["v17"])]:
            for var, ls in [("ref", "-"), ("spec", "--")]:
                x = d[(d.pre == f"ML{X}_base38") & (d.wallets == wal) & (d.variant == var)].iloc[0]
                s = series("q7_fork", int(x.run_id))
                t = (s.block - 2000) / 1000
                lab = f"pre-fork M_L {X}, {'spec rescale' if var == 'spec' else 'reference'}"
                axes[row, 0].plot(t, s.m_l / 1e6, ls=ls, color=col, label=lab)
                axes[row, 1].plot(t, s.max_weight / 1e6, ls=ls, color=col, label=lab)
                axes[row, 2].plot(t, s.fee_low, ls=ls, color=col, label=lab)
        axes[row, 0].set_ylabel("after fork: " + ("38 tx/block" if wal == "auto" else "demand > capacity"))
        for ax in (axes[row, 1], axes[row, 2]):
            ax.set_yscale("log")
            ax.yaxis.set_minor_locator(LogLocator(base=10, subs=(2, 5)))
            ax.yaxis.set_major_formatter(FuncFormatter(lambda v, _: f"{v:g}"))
            ax.yaxis.set_minor_formatter(FuncFormatter(lambda v, _: f"{v:g}"))
    fig.subplots_adjust(wspace=0.38)
    axes[0, 0].set_title("M_L (MB)", loc="left"); axes[0, 1].set_title("max block weight (MB)", loc="left"); axes[0, 2].set_title("lowest fee level (XMR/byte)", loc="left")
    for ax in axes[1]:
        ax.set_xlabel("thousand blocks after the fork")
    h, l = axes[0, 0].get_legend_handles_labels()
    fig.legend(h, l, loc="lower center", ncol=4, fontsize=7, bbox_to_anchor=(0.5, -0.06))
    fig.suptitle("Q7: v16→v17 fork; solid = reference code (no rescale), dashed = spec transitional rule", x=0.01, y=1.03, ha="left")
    savefig(fig, "q7_fork.png")


# ----------------------------------------------------------------------------------------
# Real chains: mainnet and stressnet
# ----------------------------------------------------------------------------------------
def chains():
    out = []
    data = os.path.join(ROOT, "data")
    m = pd.read_csv(os.path.join(data, "mainnet_weights.csv.gz"), comment="#")
    out.append(f"### Mainnet, heights {m.height.min():,}–{m.height.max():,} ({(m.timestamp.max()-m.timestamp.min())/86400:.0f} days)\n")
    out.append(f"- block weight: mean {m.block_weight.mean():,.0f}, median {m.block_weight.median():,.0f}, 90th pct {m.block_weight.quantile(.9):,.0f}, max {m.block_weight.max():,}")
    out.append(f"- transactions per block: mean {m.num_txes.mean():.1f}; bytes per transaction (incl. coinbase share) {m.block_weight.sum()/m.num_txes.sum():,.0f}")
    lt_med = m.long_term_weight.median()
    clamp_lo = 300000 * 10 // 17
    out.append(f"- long-term weights: median {lt_med:,.0f} ({'equal to' if lt_med == clamp_lo else 'compare'} 300,000 × 10/17 = {clamp_lo:,}, the lower clamp), "
               f"max {m.long_term_weight.max():,}; blocks above 300,000: {(m.block_weight>300000).sum():,}")
    key("chains", "mainnet tx/block; bytes per tx", f"{m.num_txes.mean():.1f}; {m.block_weight.sum()/m.num_txes.sum():,.0f}")
    key("chains", "mainnet long-term weights: median; max", f"{lt_med:,.0f}; {m.long_term_weight.max():,}")
    p = os.path.join(data, "stressnet_weights.csv.gz")
    if os.path.exists(p):
        s = pd.read_csv(p, comment="#")
        f = s[s.height >= 3103520]
        one = f[f.num_txes == 1].block_weight
        top = list(one.value_counts().head(5).index)
        coinbase = [107, 237, 328]
        fcmp = {7650: "1-in/2-out", 9093: "2-in/2-out", 11851: "4-in/2-out"}
        matches = sorted({(w - c, fcmp[w - c]) for w in top for c in coinbase if (w - c) in fcmp})
        out.append(f"\n### Stressnet (testnet with the FCMP++ fork, hard fork version 18 from height 3,103,520), heights {f.height.min():,}–{f.height.max():,}\n")
        out.append(f"- most common long-term weight of post-fork blocks: {f.long_term_weight.mode().iloc[0]:,} (10,000,000 × 5/6 = {10_000_000*5//6:,}): "
                   "stressnet runs with a 10 MB penalty-free zone, so its absolute sizes are not the mainnet parameters, and its long-term window is 10,080 blocks rather than 100,000, so its medians move about 10× faster")
        out.append(f"- most common weights of one-transaction blocks: {', '.join(f'{k:,}' for k in top)}; minus a coinbase of 107, 237 or 328 bytes, "
                   f"these give {', '.join(f'{w:,} ({n})' for w, n in matches) or 'no FCMP++ reference weight'}")
        over = f[f.block_weight > 10_000_000]
        out.append(f"- {len(over)} blocks exceeded the 10,000,000 penalty-free size, by {(over.block_weight.min()/1e7-1)*100:.1f}% to {(f.block_weight.max()/1e7-1)*100:.1f}% — "
                   "the 'slightly exceed M_N when fees cover the penalty' mechanism, seen on a live network")
        out.append(f"- transactions per post-fork block: mean {f.num_txes.mean():.1f}; bytes per transaction {f.block_weight.sum()/max(1,f.num_txes.sum()):,.0f}")
        key("chains", "stressnet blocks over 10 MB; overshoot range", f"{len(over)}; {(over.block_weight.min()/1e7-1)*100:.1f}%–{(f.block_weight.max()/1e7-1)*100:.1f}%")
    write("chains.md", "\n".join(out))


def keys():
    out = ["### Headline numbers quoted in REPORT.md, computed by analyze.py\n", "| question | quantity | value |", "|---|---|---|"]
    out += [f"| {q} | {label} | {v} |" for q, label, v in KEY]
    write("key_numbers.md", "\n".join(out))
    print("\n".join(out))


if __name__ == "__main__":
    which = sys.argv[1:] or ["chains", "q1", "q2", "q3", "q4", "q5", "q6", "q7"]
    for w in which:
        globals()[w]()
    if not sys.argv[1:]:  # key_numbers.md covers every question, so only a full run writes it
        keys()
