#!/usr/bin/env python3
"""Figures and tables for study/REPORT.md, from study/results/.

Run from anywhere: python3 study/analysis/analyze.py
Writes study/figures/*.png and study/tables/*.md (one file per question).
"""
import os, sys, math
import numpy as np
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.ticker import FuncFormatter, NullFormatter, NullLocator

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
RES = os.path.join(ROOT, "results")
FIG = os.path.join(ROOT, "figures")
TAB = os.path.join(ROOT, "tables")
os.makedirs(FIG, exist_ok=True)
os.makedirs(TAB, exist_ok=True)

DAY, MONTH, YEAR = 720, 21600, 262800
# Reference categorical palette (dataviz skill, light mode), fixed order per entity
C = {"v17": "#2a78d6", "v16": "#eb6834", "v17_sf50": "#1baf7a", "v16_9093": "#eda100", "v17_mix": "#6250d6"}
INK, INK2, GRID, SURF = "#0b0b0b", "#52514e", "#e4e3df", "#fcfcfb"
plt.rcParams.update({
    "figure.facecolor": SURF, "axes.facecolor": SURF, "savefig.facecolor": SURF,
    "axes.edgecolor": INK2, "axes.labelcolor": INK, "xtick.color": INK2, "ytick.color": INK2,
    "axes.grid": True, "grid.color": GRID, "grid.linewidth": 0.6, "axes.spines.top": False,
    "axes.spines.right": False, "lines.linewidth": 2, "font.size": 9, "axes.titlesize": 10,
    "legend.frameon": False, "figure.dpi": 110,
})

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

def md(df, floatfmt=None):
    """DataFrame -> markdown table (no tabulate dependency)."""
    cols = list(df.columns)
    out = ["| " + " | ".join(str(c) for c in cols) + " |", "|" + "|".join("---" for _ in cols) + "|"]
    for _, r in df.iterrows():
        cells = []
        for c in cols:
            v = r[c]
            if isinstance(v, float):
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

def write(name, text):
    with open(os.path.join(TAB, name), "w") as f:
        f.write(text.strip() + "\n")
    print("wrote tables/" + name)

def savefig(fig, name):
    fig.savefig(os.path.join(FIG, name), bbox_inches="tight")
    plt.close(fig)
    print("wrote figures/" + name)

mb_fmt = FuncFormatter(lambda x, _: f"{x/1e6:g}")

# ----------------------------------------------------------------------------------------
# Q1 growth capacity
# ----------------------------------------------------------------------------------------
def q1():
    d = load("q1_growth")
    d["rules"], d["mode"] = d.c0, d.c1
    out = []
    thr = [2, 5, 10, 50, 100]
    rows = []
    for (r, m, f), g in d.groupby(["rules", "mode", "demand.fill_mult"], sort=False):
        x = g.iloc[0]
        row = {"rules": r, "miners/fee": m, "demand": "1.0×max" if f == 1.0 else "1.1×M_N"}
        for t in thr:
            v = x[f"t_maxw_{t}MB"]
            row[f"{t} MB (days)"] = days(v) if pd.notna(v) else float("nan")
        row["M_L 2× (days)"] = days(x["t_m_l_2x"]) if pd.notna(x["t_m_l_2x"]) else float("nan")
        row["M_L 10× (days)"] = days(x["t_m_l_10x"]) if pd.notna(x["t_m_l_10x"]) else float("nan")
        row["fee/tx (XMR)"] = x["user_fee_per_tx"]
        rows.append(row)
    t = pd.DataFrame(rows)
    fmt = {c: "{:,.1f}" for c in t.columns if "days" in c}
    fmt["fee/tx (XMR)"] = "{:.2e}"
    out.append("### Time for the maximum block weight (2·M_N) to reach a size, demand permanently above capacity\n")
    out.append("Days from a steady chain at Z_M. `fill` = miners fill every block to the maximum regardless of fees (upper bound). "
               "`L0`..`L4` = rational miners, every user pays that wallet level; `auto` = wallet2 default priority; "
               "`auto_upm` = auto with users-pay-more. v16 uses 2,500-byte transactions (mainnet average), v16_9093 and v17 use 9,093.\n")
    out.append(md(t[t.demand == "1.0×max"].drop(columns="demand"), fmt))
    out.append("\nSame with demand only 10% above M_N (fill_mult 0.55 × max weight):\n")
    out.append(md(t[t.demand == "1.1×M_N"].drop(columns="demand"), fmt))

    # analytic bound check: M_L(t+50000)/M_L(t) <= clamp ratio
    chk = []
    for _, x in d[d["demand.fill_mult"] == 1.0].iterrows():
        s = series("q1_growth", int(x.run_id))
        ml = s.set_index("block").m_l
        # series every 1000 blocks: compare blocks b and b+50000
        b = ml.index.values
        a = ml.values.astype(float)
        idx = {v: i for i, v in enumerate(b)}
        ratios = [a[idx[v + 50000]] / a[i] for i, v in enumerate(b) if v + 50000 in idx]
        clamp = 1.7 if x.rules.startswith("v16") else 1.2
        chk.append({"rules": x.rules, "miners/fee": x["mode"], "max M_L ratio over 50k blocks": max(ratios), "clamp": clamp,
                    "violations": int(sum(r > clamp * (1 + 1e-9) + 1e-6 for r in ratios))})
    ck = pd.DataFrame(chk)
    out.append("\n### Analytic bound: M_L rises at most by the clamp ratio per 50,000 blocks\n")
    out.append("Largest ratio M_L(b+50,000)/M_L(b) over every block b sampled (every 1,000 blocks), 1.6M-block runs:\n")
    out.append(md(ck, {"max M_L ratio over 50k blocks": "{:.4f}"}))
    # predicted times for fill: M_L(k)=Z*c^k at k*50000, max weight = 2*cap*M_L
    pred = []
    for rules, z, c, cap in [("v16", 300000, 1.7, 50), ("v17", 625000, 1.2, 8)]:
        for t_ in [50, 100]:
            k = max(0, math.ceil(math.log(t_ * 1e6 / (2 * cap * z)) / math.log(c) - 1e-12))
            pred.append({"rules": rules, "size": f"{t_} MB", "predicted (days)": k * 50000 / DAY,
                         "simulated, fill (days)": days(d[(d.rules == rules) & (d["mode"] == "fill") & (d["demand.fill_mult"] == 1.0)][f"t_maxw_{t_}MB"].iloc[0])})
    out.append("\nPredicted time for the maximum weight 2·surge·M_L to reach a size if M_L steps up by the clamp ratio every 50,000 blocks, vs simulated (fill):\n")
    out.append(md(pd.DataFrame(pred), {"predicted (days)": "{:.1f}", "simulated, fill (days)": "{:.1f}"}))

    # figure: max weight and M_L over time
    fig, axes = plt.subplots(1, 2, figsize=(10, 3.8), sharey=False)
    for ax, col, title in [(axes[0], "max_weight", "Maximum block weight (2·M_N)"), (axes[1], "m_l", "Long-term median M_L")]:
        for rules in ["v16", "v17"]:
            for mode, ls in [("fill", "-"), ("auto", "--"), ("L0", ":")]:
                x = d[(d.rules == rules) & (d["mode"] == mode) & (d["demand.fill_mult"] == 1.0)].iloc[0]
                s = series("q1_growth", int(x.run_id))
                s = s[s.block <= 3 * YEAR]
                ax.plot(s.block / YEAR, s[col], ls=ls, color=C[rules], label=f"{rules} {mode}")
        ax.set_yscale("log")
        ax.set_title(title, loc="left")
        ax.set_xlabel("years")
        ax.set_ylabel("bytes")
        for y in [2e6, 1e7, 1e8]:
            ax.axhline(y, color=INK2, lw=0.6, ls="-", alpha=0.4)
    axes[0].legend(ncol=2, fontsize=8)
    fig.suptitle("Q1: growth with demand permanently above capacity (fill = miners ignore fees; auto/L0 = rational miners)", x=0.01, ha="left", fontsize=10)
    savefig(fig, "q1_growth.png")

    # zoom: first 10 days, max weight
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
    ax.set_title("Q1: short-term ramp by fee level (solid v17, dashed v16; darker = higher fee)", loc="left")
    ax.legend(ncol=3, fontsize=7)
    savefig(fig, "q1_ramp_by_level.png")

    # Q1b demand growth
    b = load("q1b_demand_growth")
    b["rules"], b["fee"] = b.c0, b.c1
    rows = []
    for (r, f, g_), g in b.groupby(["rules", "fee", "demand.growth_per_year"]):
        rows.append({"rules": r, "wallets": f, "growth/yr": f"{g_*100:.0f}%", "final demand (tx/block)": 38 * (1 + g_) ** 5,
                     "final M_L (MB)": g.final_m_l.mean() / 1e6, "final M_N (MB)": g.final_m_n.mean() / 1e6,
                     "wait mean (blocks)": g.wait_mean.astype(float).mean(), "wait p99": g.wait_p99.astype(float).mean(),
                     "wait max": g.wait_max.max(), "fee/tx (XMR)": g.user_fee_per_tx.astype(float).mean(),
                     "max pool (MB)": g.max_pool_user_bytes.mean() / 1e6})
    out.append("\n### Q1b: steady exponential demand growth from 38 tx/block for 5 years (Poisson, 8 seeds)\n")
    out.append(md(pd.DataFrame(rows), {"final M_L (MB)": "{:.3g}", "final M_N (MB)": "{:.3g}", "wait mean (blocks)": "{:.2f}", "fee/tx (XMR)": "{:.2e}", "max pool (MB)": "{:.3g}", "final demand (tx/block)": "{:,.0f}", "wait p99": "{:.1f}"}))
    write("q1.md", "\n".join(out))


# ----------------------------------------------------------------------------------------
# Q2 surges
# ----------------------------------------------------------------------------------------
def q2():
    d = load("q2_surge")
    d["rules"], d["fee"] = d.c0, d.c1
    dur_name = {30: "1 hour", 720: "1 day", 5040: "1 week"}
    agg = d.groupby(["rules", "fee", "demand.surge_blocks", "demand.surge_mult"]).agg(
        peak_pool=("win_peak_pool_user_bytes", "mean"), clear=("win_clear_after_end", "mean"),
        uncleared=("win_clear_after_end", lambda x: int(x.isna().sum())),
        wait_mean=("win_wait_mean", "mean"), wait_p90=("win_wait_p90", "mean"), wait_p99=("win_wait_p99", "mean"),
        wait_max=("win_wait_max", "max"), fee_tx=("win_fee_per_tx", "mean"), fee_b=("win_fee_per_byte", "mean"),
        within10=("win_share_within_10", "mean"), within720=("win_share_within_720", "mean"),
        within2160=("win_share_within_2160", "mean"), max_mn=("max_m_n", "mean"), n=("seed", "count")).reset_index()
    agg.to_csv(os.path.join(TAB, "q2_surge_all.csv"), index=False)
    out = []
    for dur in [30, 720, 5040]:
        a = agg[(agg.fee == "auto") & (agg["demand.surge_blocks"] == dur)]
        t = a[["rules", "demand.surge_mult", "peak_pool", "clear", "wait_mean", "wait_p99", "wait_max", "within720", "fee_tx", "fee_b", "max_mn"]].copy()
        t.columns = ["rules", "surge ×", "peak backlog (MB)", "clears after end (blocks)", "mean wait", "p99 wait", "max wait", "share in ≤1 day",
                     "fee/tx (XMR)", "fee/byte (XMR)", "peak M_N (MB)"]
        t["peak backlog (MB)"] /= 1e6
        t["peak M_N (MB)"] /= 1e6
        out.append(f"\n### Surge lasting {dur_name[dur]}, wallets on auto priority (means over 16 seeds; waits in blocks of 2 min)\n")
        out.append(md(t, {"peak backlog (MB)": "{:.3g}", "clears after end (blocks)": "{:,.0f}", "mean wait": "{:,.1f}", "p99 wait": "{:,.0f}",
                          "share in ≤1 day": "{:.3f}", "fee/tx (XMR)": "{:.2e}", "fee/byte (XMR)": "{:.2e}", "peak M_N (MB)": "{:.3g}"}))
    # fee level table for 1 day surges
    a = agg[agg["demand.surge_blocks"] == 720]
    piv = a.pivot_table(index=["rules", "demand.surge_mult"], columns="fee", values="wait_mean").reset_index()
    piv = piv[["rules", "demand.surge_mult", "L0", "L1", "L2", "L3", "L4", "auto", "auto_upm"]]
    out.append("\n### Mean wait (blocks) by fee level, 1-day surges\n")
    out.append(md(piv, {k: "{:,.1f}" for k in ["L0", "L1", "L2", "L3", "L4", "auto", "auto_upm"]}))
    piv = a.pivot_table(index=["rules", "demand.surge_mult"], columns="fee", values="fee_tx").reset_index()
    piv = piv[["rules", "demand.surge_mult", "L0", "L1", "L2", "L3", "L4", "auto", "auto_upm"]]
    out.append("\n### Fee per surge transaction (XMR) by fee level, 1-day surges\n")
    out.append(md(piv, {k: "{:.2e}" for k in ["L0", "L1", "L2", "L3", "L4", "auto", "auto_upm"]}))
    # which level clears in time: lowest level whose p99 wait <= 30 blocks (1h) and <= 720
    rows = []
    for (r, dur, m), g in a.groupby(["rules", "demand.surge_blocks", "demand.surge_mult"]) if False else agg.groupby(["rules", "demand.surge_blocks", "demand.surge_mult"]):
        row = {"rules": r, "duration": dur_name[dur], "surge ×": m}
        for lim, nm in [(30, "p99 ≤ 1 h"), (720, "p99 ≤ 1 day")]:
            ok = g[g.fee.isin(["L0", "L1", "L2", "L3", "L4"]) & (g.wait_p99 <= lim)]
            row[nm] = ok.fee.min() if len(ok) else "none"
        rows.append(row)
    lv = pd.DataFrame(rows)
    out.append("\n### Lowest fixed fee level whose 99th-percentile wait stays within a limit\n")
    out.append(md(lv))
    # 8x cap: v17 vs v17_sf50
    rows = []
    for (dur, m), g in agg[agg.fee == "auto"].groupby(["demand.surge_blocks", "demand.surge_mult"]):
        a8 = g[g.rules == "v17"].iloc[0]
        a50 = g[g.rules == "v17_sf50"].iloc[0]
        rows.append({"duration": dur_name[dur], "surge ×": m, "v17 (8×) mean wait": a8.wait_mean, "surge 50× mean wait": a50.wait_mean,
                     "v17 max wait": a8.wait_max, "surge 50× max wait": a50.wait_max,
                     "v17 clears after end": a8.clear, "surge 50× clears after end": a50.clear,
                     "v17 peak M_N (MB)": a8.max_mn / 1e6, "surge 50× peak M_N (MB)": a50.max_mn / 1e6})
    cap = pd.DataFrame(rows)
    out.append("\n### Does the 8× surge cap hurt? v17 vs the same rules with a 50× cap (Custom), auto priority\n")
    out.append(md(cap, {c: "{:,.1f}" for c in cap.columns if "wait" in c or "clears" in c} | {"v17 peak M_N (MB)": "{:.3g}", "surge 50× peak M_N (MB)": "{:.3g}"}))
    write("q2.md", "\n".join(out))

    # figure: mean wait vs multiplier by duration, auto
    fig, axes = plt.subplots(1, 3, figsize=(11, 3.4), sharey=True)
    for ax, dur in zip(axes, [30, 720, 5040]):
        for r in ["v16", "v16_9093", "v17", "v17_sf50", "v17_mix"]:
            g = agg[(agg.fee == "auto") & (agg.rules == r) & (agg["demand.surge_blocks"] == dur)]
            ax.plot(g["demand.surge_mult"], g.wait_mean.clip(lower=0.1), marker="o", ms=4, color=C[r], label=r)
        ax.set_xscale("log")
        ax.set_yscale("log")
        ax.xaxis.set_minor_locator(NullLocator())
        ax.set_xticks([2, 5, 10, 20, 50])
        ax.set_xticklabels(["2", "5", "10", "20", "50"])
        ax.set_title(f"surge lasting {dur_name[dur]}", loc="left")
        ax.set_xlabel("surge multiplier (× 38 tx/block)")
    axes[0].set_ylabel("mean wait of surge txs, blocks")
    axes[0].legend(fontsize=8)
    fig.suptitle("Q2: waits during demand surges, wallets on auto priority (16 seeds)", x=0.01, ha="left")
    savefig(fig, "q2_surge_wait.png")

    # figure: by level, 1 day
    fig, axes = plt.subplots(1, 2, figsize=(10, 3.4), sharey=True)
    cols = blues(5)
    for ax, r in zip(axes, ["v16", "v17"]):
        for k, f in enumerate(["L0", "L1", "L2", "L3", "L4"]):
            if r == "v16" and f == "L4":
                continue
            g = agg[(agg.fee == f) & (agg.rules == r) & (agg["demand.surge_blocks"] == 720)]
            ax.plot(g["demand.surge_mult"], g.wait_mean.clip(lower=0.1), marker="o", ms=4, color=cols[k], label=f)
        g = agg[(agg.fee == "auto_upm") & (agg.rules == r) & (agg["demand.surge_blocks"] == 720)]
        ax.plot(g["demand.surge_mult"], g.wait_mean.clip(lower=0.1), marker="s", ms=4, color=C["v16"] if r == "v17" else C["v17_sf50"], ls="--", label="auto + pay more")
        ax.set_xscale("log"); ax.set_yscale("log")
        ax.xaxis.set_minor_locator(NullLocator())
        ax.set_xticks([2, 5, 10, 20, 50]); ax.set_xticklabels(["2", "5", "10", "20", "50"])
        ax.set_title(f"{r}: 1-day surge, by fee level", loc="left")
        ax.set_xlabel("surge multiplier")
        ax.legend(fontsize=7, ncol=2)
    axes[0].set_ylabel("mean wait, blocks (0 shown as 0.1)")
    savefig(fig, "q2_surge_levels.png")

    # example time series 10x 1 day, seed index 0, auto
    fig, axes = plt.subplots(1, 3, figsize=(11, 3.2))
    for r in ["v16", "v17", "v17_sf50"]:
        x = d[(d.rules == r) & (d.fee == "auto") & (d["demand.surge_blocks"] == 720) & (d["demand.surge_mult"] == 20) & (d.seed_index == 0)].iloc[0]
        s = series("q2_surge", int(x.run_id))
        s = s[(s.block >= 4000) & (s.block <= 9000)]
        t = (s.block - 5000) / DAY
        axes[0].plot(t, s.pool_user_bytes / 1e6, color=C[r], label=r)
        axes[1].plot(t, s.m_n / 1e6, color=C[r], label=r)
        axes[2].plot(t, s.mb_mean / 1e6, color=C[r], label=r)
    for ax, ttl in zip(axes, ["backlog (MB)", "M_N (MB)", "mean block weight (MB)"]):
        ax.axvspan(0, 1, color=GRID, alpha=0.6, lw=0)
        ax.set_title(ttl, loc="left")
        ax.set_xlabel("days since surge start")
    axes[0].legend(fontsize=8)
    fig.suptitle("Q2: a 20× surge lasting one day (shaded), auto priority, seed 1", x=0.01, ha="left")
    savefig(fig, "q2_surge_example.png")


# ----------------------------------------------------------------------------------------
# Q3 attacks
# ----------------------------------------------------------------------------------------
def q3():
    out = []
    a = load("q3a_spam_full")
    a["rules"], a["strategy"] = a.c0, a.c1
    g = a.groupby(["rules", "strategy", "spam.blocks"]).agg(spent=("spam_spent_in_window_xmr", "mean"), gb=("spam_bytes", "mean"),
            per_gb=("spam_xmr_per_gb", "mean"), full=("win_full_blocks", "mean"), wb=("win_blocks", "mean"), max_mn=("max_m_n", "mean"),
            uwait=("win_wait_mean", "mean"), ufee=("win_fee_per_tx", "mean")).reset_index()
    g["days"] = g["spam.blocks"] / DAY
    g["XMR/day"] = g.spent / g.days
    g["full share"] = g.full / g.wb
    g["gb"] /= 1e9
    g["max_mn"] /= 1e6
    t = g[["rules", "strategy", "days", "spent", "XMR/day", "gb", "per_gb", "full share", "max_mn", "uwait", "ufee"]]
    t.columns = ["rules", "strategy", "days", "XMR spent", "XMR/day", "GB added", "XMR/GB", "share of blocks full", "peak M_N (MB)", "user mean wait", "user fee/tx"]
    out.append("### Q3a: keeping blocks full (at M_N) with fee-paying spam, users at 38 tx/block on auto (8 seeds)\n")
    out.append("`fill_to_L0`: each block the spammer tops the pool up to exactly M_N at the lowest fee (no backlog). "
               "`topup_L0`: keeps a full block of spam waiting at the lowest fee. `topup_auto`: same at the wallet's auto level.\n")
    out.append(md(t, {"XMR spent": "{:,.1f}", "XMR/day": "{:,.2f}", "GB added": "{:.3g}", "XMR/GB": "{:.1f}", "share of blocks full": "{:.3f}", "peak M_N (MB)": "{:.3g}", "user mean wait": "{:.2f}", "user fee/tx": "{:.2e}"}))
    base = g[g.strategy == "fill_to_L0"].set_index(["rules", "days"]).spent
    rat = []
    for dd in [1, 7, 30]:
        rat.append({"days": dd, "v16 (XMR)": base[("v16", dd)], "v17 (XMR)": base[("v17", dd)], "v17 / v16": base[("v17", dd)] / base[("v16", dd)]})
    out.append("\nCheapest way to keep blocks at M_N (fill_to_L0): v17 vs v16\n")
    out.append(md(pd.DataFrame(rat), {"v17 / v16": "{:.2f}", "v16 (XMR)": "{:,.1f}", "v17 (XMR)": "{:,.1f}"}))

    b = load("q3b_spam_growth")
    b["rules"], b["strategy"] = b.c0, b.c1
    rows = []
    for (r, s), gg in b.groupby(["rules", "strategy"]):
        rows.append({"rules": r, "strategy": s, "M_L 2× (days)": days(gg.t_m_l_2x.mean()), "XMR to 2×": gg.spam_xmr_at_m_l_2x.mean(),
                     "M_L 4× (days)": days(gg.t_m_l_4x.mean()), "XMR to 4×": gg.spam_xmr_at_m_l_4x.mean(),
                     "M_L after 2 yr (MB)": gg.win_end_m_l.mean() / 1e6, "spent 2 yr (XMR)": gg.spam_spent_in_window_xmr.mean(),
                     "spam added 2 yr (GB)": gg.spam_bytes.mean() / 1e9, "XMR/GB": gg.spam_xmr_per_gb.mean(),
                     "M_L 12 mo after (MB)": gg.win_plus12mo_m_l.mean() / 1e6})
    t = pd.DataFrame(rows)
    out.append("\n### Q3b/c: spam campaigns over 2 years (users at 38 tx/block, 4 seeds)\n")
    out.append("`clamp_L0`: fill each block to the clamp ratio × M_L (1.7× v16, 1.2× v17) at the lowest fee — the long-term weight can't count more. "
               "`mn_L0`: fill to M_N only. `topup_L0`: keep a block of backlog. `clamp_top`: as clamp_L0 at the highest wallet level. "
               "`surgecap_L0`: fill to 8 × M_L.\n")
    out.append(md(t, {"M_L 2× (days)": "{:,.0f}", "XMR to 2×": "{:,.0f}", "M_L 4× (days)": "{:,.0f}", "XMR to 4×": "{:,.0f}", "M_L after 2 yr (MB)": "{:.3g}",
                      "spent 2 yr (XMR)": "{:,.0f}", "spam added 2 yr (GB)": "{:,.0f}", "XMR/GB": "{:.3g}", "M_L 12 mo after (MB)": "{:.3g}"}))
    # cost of N GB: min over strategies (seed 0 series)
    rows = []
    curves = {}
    for r in ["v16", "v17"]:
        for s in ["mn_L0", "clamp_L0", "topup_L0", "surgecap_L0"]:
            x = b[(b.rules == r) & (b.strategy == s) & (b.seed_index == 0)].iloc[0]
            ss = series("q3b_spam_growth", int(x.run_id))
            curves[(r, s)] = ss
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
    t = pd.DataFrame(rows)
    out.append("\n### Cost to add N GB of spam to the chain: cheapest of the strategies above (within 2 years)\n")
    out.append(md(t, {"cheapest XMR": "{:,.0f}", "days": "{:,.0f}", "GB added": "{:,.0f}"}))

    fig, axes = plt.subplots(1, 2, figsize=(10, 3.6))
    for (r, s), ss in curves.items():
        ls = {"mn_L0": ":", "clamp_L0": "-", "topup_L0": "--", "surgecap_L0": "-."}[s]
        axes[0].plot(ss.cum_spam_xmr, ss.m_l / ss.m_l.iloc[0], ls=ls, color=C[r], label=f"{r} {s}")
        axes[1].plot(ss.cum_spam_bytes / 1e9, ss.cum_spam_xmr, ls=ls, color=C[r], label=f"{r} {s}")
    axes[0].set_xscale("symlog", linthresh=10); axes[0].set_xlabel("XMR spent by the spammer"); axes[0].set_ylabel("M_L / initial M_L")
    axes[0].set_yscale("log"); axes[0].set_title("M_L raised vs XMR spent", loc="left")
    axes[1].set_xscale("log"); axes[1].set_yscale("log"); axes[1].set_xlabel("GB of spam in the chain"); axes[1].set_ylabel("cumulative XMR spent")
    axes[1].set_title("Bloat: XMR spent vs GB added", loc="left")
    axes[0].legend(fontsize=7, ncol=1)
    fig.suptitle("Q3: fee-paying spammer, 2-year campaigns (seed 1)", x=0.01, ha="left")
    savefig(fig, "q3_spam.png")

    m = load("q3d_miner")
    m["rules"], m["target"] = m.c0, m.c1
    gm = m.groupby(["rules", "target", "miner_attack.share"]).agg(blocks=("miner_blocks", "mean"), pen=("miner_penalty_extra_xmr", "mean"),
          pad=("padding_bytes", "mean"), ml_end=("win_end_m_l", "mean"), ml_ratio=("win_m_l_ratio", "mean"), ml_ratio_min=("win_m_l_ratio", "min"),
          ml_ratio_max=("win_m_l_ratio", "max"), max_mn=("max_m_n", "mean"), t2=("t_m_l_2x", "mean"), c2=("miner_xmr_at_m_l_2x", "mean"),
          reached2=("t_m_l_2x", lambda x: int(x.notna().sum())), n=("seed", "count")).reset_index()
    gm["pad"] /= 1e9
    gm["xmr_per_gb"] = gm.pen / gm.pad
    t = gm[["rules", "target", "miner_attack.share", "pen", "pad", "xmr_per_gb", "ml_ratio", "ml_ratio_min", "ml_ratio_max", "max_mn", "reached2", "n"]].copy()
    t["max_mn"] /= 1e6
    t.columns = ["rules", "padding target", "hashrate h", "penalty paid (XMR)", "padding (GB)", "XMR/GB", "M_L after 1 yr ÷ before (mean)", "min", "max", "peak M_N (MB)", "seeds reaching 2×", "seeds"]
    out.append("\n### Q3d: malicious miner padding its own blocks with zero-fee filler for 1 year (16 seeds)\n")
    out.append("`pad_mn`: pad to M_N (free). `pad_clamp`: pad to the clamp ratio × M_L. `pad_max`: pad to 2·M_N (forfeits the whole reward). "
               "Penalty paid = the penalty above what its block would have paid without padding.\n")
    out.append(md(t, {"penalty paid (XMR)": "{:,.0f}", "padding (GB)": "{:,.1f}", "XMR/GB": "{:,.1f}", "M_L after 1 yr ÷ before (mean)": "{:.3f}", "min": "{:.3f}", "max": "{:.3f}", "peak M_N (MB)": "{:.3g}"}))
    write("q3.md", "\n".join(out))

    fig, axes = plt.subplots(1, 2, figsize=(10, 3.4))
    for r in ["v16", "v17"]:
        for tg, ls in [("pad_clamp", "-"), ("pad_max", "--")]:
            g = gm[(gm.rules == r) & (gm.target == tg)]
            axes[0].plot(g["miner_attack.share"], g.ml_ratio, ls=ls, marker="o", ms=4, color=C[r], label=f"{r} {tg}")
            axes[1].plot(g["miner_attack.share"], g.pen, ls=ls, marker="o", ms=4, color=C[r], label=f"{r} {tg}")
    axes[0].set_xlabel("attacker hashrate share h"); axes[0].set_ylabel("M_L after 1 year ÷ M_L before"); axes[0].set_title("M_L growth", loc="left")
    axes[1].set_xlabel("attacker hashrate share h"); axes[1].set_ylabel("extra penalty paid in 1 year, XMR"); axes[1].set_yscale("log"); axes[1].set_title("cost", loc="left")
    axes[0].axvline(0.5, color=INK2, lw=0.8, alpha=0.5)
    axes[1].axvline(0.5, color=INK2, lw=0.8, alpha=0.5)
    axes[0].legend(fontsize=8)
    axes[1].text(0.02, 0.02, "pad_max: v16 and v17 overlap (each padded block forfeits the 0.6 XMR reward)", transform=axes[1].transAxes, fontsize=7, color=INK2)
    fig.suptitle("Q3d: malicious miner padding its blocks for one year (16 seeds)", x=0.01, ha="left")
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
    out = ["### Q4: recovery after demand above capacity stops (back to 38 tx/block; pool transactions expire after 3 days)\n",
           md(t, {c: "{:.3g}" for c in t.columns if "MB" in c} | {c: "{:,.1f}" for c in t.columns if "days" in c} | {"fee_low at end ÷ base": "{:.3g}"})]
    # spam recovery from q3b
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
    axes[0].legend(fontsize=7)
    fig.suptitle("Q4: recovery after periods of demand above capacity (rational miners, auto wallets)", x=0.01, ha="left")
    savefig(fig, "q4_recovery.png")


# ----------------------------------------------------------------------------------------
# Q5 fee market
# ----------------------------------------------------------------------------------------
def q5():
    d = load("q5_fee_market")
    d["rules"], d["fee"] = d.c0, d.c1
    out = []
    # steady state: second half of each run from series (seeds 0-1); whole-run waits from summaries
    rows = []
    for (r, f, z), g in d.groupby(["rules", "fee", "demand.zm_mult"]):
        ss = []
        for _, x in g[g.seed_index < 2].iterrows():
            s = series("q5_fee_market", int(x.run_id))
            s = s[s.block >= 50000]
            ss.append((s.user_fees_sum.sum() / max(1, s.user_incl_txs.sum()), (s.user_wait_mean * s.user_incl_txs).sum() / max(1, s.user_incl_txs.sum()),
                       s.m_n.iloc[-1], (s.blocks_over_mn.sum() / len(s) / 100)))
        ss = np.array(ss)
        rows.append({"rules": r, "wallets": f, "demand ×Z_M": z, "fee/tx, 2nd half (XMR)": ss[:, 0].mean(), "mean wait, 2nd half": ss[:, 1].mean(),
                     "wait p99, whole run": g.wait_p99.astype(float).mean(), "wait max, whole run": g.wait_max.max(),
                     "share of blocks > M_N": g.blocks_over_mn.mean() / g.blocks.iloc[0], "mean M_B/M_N when over": g.over_mn_mean_ratio.astype(float).mean(),
                     "M_N at end (MB)": ss[:, 2].mean() / 1e6})
    t = pd.DataFrame(rows)
    t.to_csv(os.path.join(TAB, "q5_fee_market_all.csv"), index=False)
    for r in ["v16", "v17"]:
        out.append(f"\n### Q5: {r}, steady Poisson demand at multiples of Z_M (100,000 blocks, 16 seeds; demand starts at block 0)\n")
        out.append(md(t[t.rules == r].drop(columns="rules"), {"fee/tx, 2nd half (XMR)": "{:.2e}", "mean wait, 2nd half": "{:.3f}", "wait p99, whole run": "{:.1f}",
                                                             "share of blocks > M_N": "{:.3f}", "mean M_B/M_N when over": "{:.4f}", "M_N at end (MB)": "{:.3g}"}))
    # analytic: how far beyond M_N a block can go, by level, at M_N = M_L = Z_M and as M_N/M_L grows
    R = 0.6
    rows = []
    for rules, zm, tr, t_ in [("v16", 300000, 3000, 2500), ("v17", 625000, 12500, 9093)]:
        fl = R * tr / zm**2
        levels = {"v16": [fl, 4 * fl, 16 * R * tr / (zm * zm), 4 * 16 * R * tr / (zm * zm)], "v17": [fl, 4 * fl, 16 * fl, 64 * fl, 2 * R / zm]}[rules]
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

    # daily cycle
    q = load("q5b_daily")
    q["rules"], q["fee"] = q.c0, q.c1
    g = q.groupby(["rules", "fee", "demand.zm_mult", "demand.daily_amplitude"]).agg(fee_tx=("user_fee_per_tx", "mean"), wait=("wait_mean", "mean"),
         p99=("wait_p99", "mean"), wmax=("wait_max", "max"), mn=("final_m_n", "mean")).reset_index()
    g["mn"] /= 1e6
    g.columns = ["rules", "wallets", "mean demand ×Z_M", "daily amplitude", "fee/tx (XMR)", "mean wait", "p99 wait", "max wait", "M_N at end (MB)"]
    out.append("\n### Q5b: daily demand cycle (720-block period), 8 seeds\n")
    out.append(md(g, {"fee/tx (XMR)": "{:.2e}", "mean wait": "{:.3f}", "p99 wait": "{:.1f}", "M_N at end (MB)": "{:.3g}"}))
    write("q5.md", "\n".join(out))

    fig, axes = plt.subplots(1, 2, figsize=(10, 3.4))
    cols = blues(5)
    for r, ls in [("v16", "--"), ("v17", "-")]:
        for k, f in enumerate(["L0", "L1", "L2", "L3", "L4"]):
            if r == "v16" and f == "L4":
                continue
            g = t[(t.rules == r) & (t.wallets == f)]
            axes[0].plot(g["demand ×Z_M"], g["fee/tx, 2nd half (XMR)"], ls=ls, marker="o", ms=3, color=cols[k], label=f"{r} {f}")
            axes[1].plot(g["demand ×Z_M"], g["wait p99, whole run"].clip(lower=0.1), ls=ls, marker="o", ms=3, color=cols[k], label=f"{r} {f}")
        g = t[(t.rules == r) & (t.wallets == "auto")]
        axes[0].plot(g["demand ×Z_M"], g["fee/tx, 2nd half (XMR)"], ls=ls, marker="s", ms=4, color=C["v16"], label=f"{r} auto")
        axes[1].plot(g["demand ×Z_M"], g["wait p99, whole run"].clip(lower=0.1), ls=ls, marker="s", ms=4, color=C["v16"], label=f"{r} auto")
    axes[0].set_yscale("log"); axes[0].set_xscale("log"); axes[0].set_xlabel("demand ÷ Z_M"); axes[0].set_ylabel("fee per tx, XMR"); axes[0].set_title("fee per transaction (steady state)", loc="left")
    axes[1].set_yscale("log"); axes[1].set_xscale("log"); axes[1].set_xlabel("demand ÷ Z_M"); axes[1].set_ylabel("p99 wait, blocks (0 → 0.1)"); axes[1].set_title("p99 wait including the ramp from Z_M", loc="left")
    for ax in axes:
        ax.xaxis.set_minor_locator(NullLocator())
        ax.set_xticks([0.5, 0.8, 1, 1.2, 2, 5]); ax.set_xticklabels(["0.5", "0.8", "1", "1.2", "2", "5"])
    axes[1].legend(fontsize=6, ncol=2)
    fig.suptitle("Q5: fee market at steady demand (solid v17, dashed v16; darker blue = higher level; orange = auto)", x=0.01, ha="left")
    savefig(fig, "q5_fee_market.png")


# ----------------------------------------------------------------------------------------
# Q6 sensitivity
# ----------------------------------------------------------------------------------------
def q6():
    d = load("q6_sensitivity")
    d["txw"] = d.tx.str.extract(r"(\d+)").astype(int)
    P = {"params.z_m": "Z_M", "params.lt_growth": "clamp", "params.surge_factor": "surge", "params.t_r": "T_R", "txw": "tx weight"}
    M = [("growth", "t_maxw_10MB", "blocks to 10 MB max weight"), ("growth", "t_m_l_2x", "blocks to 2× M_L"),
         ("surge10x1d", "win_wait_mean", "10× 1-day surge: mean wait"), ("surge10x1d", "win_clear_after_end", "10× 1-day surge: clear time"),
         ("surge50x1d", "win_wait_mean", "50× 1-day surge: mean wait"), ("surge50x1d", "win_clear_after_end", "50× 1-day surge: clear time"),
         ("spam_mn_7d", "spam_spent_in_window_xmr", "spam at M_N 7 days: XMR"), ("spam_clamp", "spam_xmr_at_m_l_2x", "spam to 2× M_L: XMR"),
         ("spam_clamp", "t_m_l_2x", "spam to 2× M_L: blocks"), ("miner51_clamp", "miner_xmr_at_m_l_2x", "51% miner to 2× M_L: XMR"),
         ("miner51_clamp", "t_m_l_2x", "51% miner to 2× M_L: blocks")]
    rows, levels = [], []
    for case, m, label in M:
        s = d[d.case == case].copy()
        y = np.log10(s[m].astype(float) + 1)
        ok = y.notna()
        tot = y[ok].var(ddof=0)
        row = {"metric (log scale)": label, "runs with a value": f"{ok.sum()}/{len(s)}"}
        for p, pn in P.items():
            sub = s[ok].assign(y=y[ok])
            gm = sub.groupby(p).y.mean()
            gn = sub.groupby(p).y.size()
            between = float((gn * (gm - sub.y.mean()) ** 2).sum() / gn.sum())  # size-weighted, <= total
            row[pn] = between / tot if tot > 0 else 0.0
            for lv, v in gm.items():
                levels.append({"metric": label, "param": pn, "value": lv, "mean of metric": 10 ** v - 1, "n": int((s[ok][p] == lv).sum())})
        rows.append(row)
    t = pd.DataFrame(rows)
    out = ["### Q6: first-order sensitivity (share of the variance of log10(metric) explained by each parameter's main effect)\n",
           "Full factorial: Z_M ∈ {300k, 625k, 1M} × clamp ∈ {1.1, 1.2, 1.4, 1.7} × surge factor ∈ {4, 8, 16, 50} × T_R ∈ {3k, 10k, 12.5k, 20k} × tx weight ∈ {7,650, 9,093, 11,851} = 576 rule sets, "
           "Custom mode (2026 fee algorithm). Runs where the metric never happened (e.g. M_L never doubled) are left out of that metric; the count is shown.\n",
           md(t, {pn: "{:.2f}" for pn in P.values()})]
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
    # censoring detail for spam_clamp
    s = d[d.case == "spam_clamp"]
    cens = s.assign(reached=s.t_m_l_2x.notna()).groupby(["params.t_r", "txw"]).reached.mean().unstack()
    out.append("\nShare of rule sets in which the minimum-fee spammer doubled M_L within 600,000 blocks, by T_R (rows) and tx weight (columns):\n")
    cens = cens.reset_index()
    cens.columns = [str(c) for c in cens.columns]
    out.append(md(cens, {c: "{:.2f}" for c in cens.columns[1:]}))
    write("q6.md", "\n".join(out))

    fig, ax = plt.subplots(figsize=(7.5, 4.2))
    mat = t[list(P.values())].values
    im = ax.imshow(mat, cmap="Blues", vmin=0, vmax=1, aspect="auto")
    ax.set_xticks(range(len(P))); ax.set_xticklabels(list(P.values()))
    ax.set_yticks(range(len(t))); ax.set_yticklabels(t["metric (log scale)"])
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
def q7():
    d = load("q7_fork")
    d["pre"], d["wallets"] = d.c0, d.c1
    e = events("q7_fork")
    out = []
    piv = e[e.event.isin(["fork_last_pre", "fork_first_block", "fork_plus_1000", "fork_plus_10000", "fork_plus_50000", "fork_plus_100000"])].merge(
        d[["run_id", "pre", "wallets", "fork.rescale"]], on="run_id")
    rows = []
    for (pre, w), g in piv.groupby(["pre", "wallets"], sort=False):
        for ev in ["fork_last_pre", "fork_first_block", "fork_plus_10000", "fork_plus_50000", "fork_plus_100000"]:
            a = g[(g.event == ev) & (g["fork.rescale"] == False)]
            b = g[(g.event == ev) & (g["fork.rescale"] == True)]
            if not len(a) or not len(b):
                continue
            a, b = a.iloc[0], b.iloc[0]
            rows.append({"pre-fork / demand": pre, "miners": w, "when": ev.replace("fork_", ""), "M_L ref (kB)": a.m_l / 1e3, "M_L spec (kB)": b.m_l / 1e3,
                         "max weight ref (MB)": a.max_weight / 1e6, "max weight spec (MB)": b.max_weight / 1e6,
                         "f_L ref": a.fee_low, "f_L spec": b.fee_low, "f_L ref ÷ spec": a.fee_low / b.fee_low})
    t = pd.DataFrame(rows)
    out.append("### Q7: v16 → v17 at block 2,000; reference code (no rescaling, `ref`) vs the spec's transitional rule (pre-fork weights × 625/300, `spec`)\n")
    out.append("Pre-fork windows filled with M_L = X (v16, 2,500-byte transactions). After the fork transactions weigh 9,093 bytes. "
               "`same_txs` keeps the transaction count (X/2,500 per block, so bytes ×3.6); `base38` drops to 38 tx/block; `fill_after` = rational miners with demand above capacity after the fork.\n")
    out.append(md(t[t.miners == "auto"].drop(columns="miners"), {"M_L ref (kB)": "{:,.0f}", "M_L spec (kB)": "{:,.0f}", "max weight ref (MB)": "{:.3g}", "max weight spec (MB)": "{:.3g}",
                                                               "f_L ref": "{:.2e}", "f_L spec": "{:.2e}", "f_L ref ÷ spec": "{:.2f}"}))
    out.append("\nWith demand above capacity after the fork:\n")
    out.append(md(t[(t.miners == "fill_after") & t["pre-fork / demand"].str.endswith("base38")].drop(columns="miners"),
                  {"M_L ref (kB)": "{:,.0f}", "M_L spec (kB)": "{:,.0f}", "max weight ref (MB)": "{:.3g}", "max weight spec (MB)": "{:.3g}",
                   "f_L ref": "{:.2e}", "f_L spec": "{:.2e}", "f_L ref ÷ spec": "{:.2f}"}))
    # divergence duration
    rows = []
    for (pre, w), g in d.groupby(["pre", "wallets"], sort=False):
        a = g[g["fork.rescale"] == False].iloc[0]
        b = g[g["fork.rescale"] == True].iloc[0]
        sa, sb = series("q7_fork", int(a.run_id)), series("q7_fork", int(b.run_id))
        sa, sb = sa[sa.block >= 2000], sb[sb.block >= 2000]
        diff_lim = (sa.max_weight.values != sb.max_weight.values)
        diff_fee = (sa.fee_low.values != sb.fee_low.values)
        diff_ml = (sa.m_l.values != sb.m_l.values)
        last = lambda m: (sa.block.values[m].max() - 2000) if m.any() else 0
        rows.append({"pre-fork / demand": pre, "miners": w, "limit differs (share of blocks)": diff_lim.mean(), "last block limit differs": last(diff_lim),
                     "fee differs (share)": diff_fee.mean(), "last block fee differs": last(diff_fee), "M_L differs (share)": diff_ml.mean(),
                     "max |log2 limit ratio|": np.abs(np.log2(sa.max_weight.values / sb.max_weight.values)).max(),
                     "max fee ratio ref÷spec": (sa.fee_low.values / sb.fee_low.values).max(), "min fee ratio": (sa.fee_low.values / sb.fee_low.values).min()})
    dv = pd.DataFrame(rows)
    out.append("\n### How long and how much the two behaviours differ (200,000 blocks after the fork, sampled every 10 blocks)\n")
    out.append(md(dv, {"limit differs (share of blocks)": "{:.3f}", "fee differs (share)": "{:.3f}", "M_L differs (share)": "{:.3f}", "max |log2 limit ratio|": "{:.3f}",
                       "max fee ratio ref÷spec": "{:.2f}", "min fee ratio": "{:.2f}", "last block limit differs": "{:,.0f}", "last block fee differs": "{:,.0f}"}))
    # mainnet
    b = load("q7b_fork_mainnet")
    rows = []
    for (c, rs), g in b.groupby(["case", "fork.rescale"]):
        rows.append({"post-fork demand": c, "rescale": rs, "M_L end (kB)": g.final_m_l.mean() / 1e3, "max M_N (MB)": g.max_m_n.mean() / 1e6,
                     "fee/tx (XMR)": g.user_fee_per_tx.astype(float).mean(), "chain GB": g.chain_bytes.mean() / 1e9, "t M_L 2× (days)": days(g.t_m_l_2x.mean())})
    out.append("\n### Fork from the real mainnet windows (last 100,000 blocks to height 3,797,557), 8 seeds\n")
    out.append(md(pd.DataFrame(rows), {"M_L end (kB)": "{:,.0f}", "max M_N (MB)": "{:.3g}", "fee/tx (XMR)": "{:.3e}", "chain GB": "{:,.1f}", "t M_L 2× (days)": "{:,.1f}"}))
    rows = []
    for c in b.case.unique():
        for si in [0]:
            a = b[(b.case == c) & (b["fork.rescale"] == False) & (b.seed_index == si)].iloc[0]
            bb = b[(b.case == c) & (b["fork.rescale"] == True) & (b.seed_index == si)].iloc[0]
            sa, sb = series("q7b_fork_mainnet", int(a.run_id)), series("q7b_fork_mainnet", int(bb.run_id))
            dif = (sa.m_l.values != sb.m_l.values) | (sa.max_weight.values != sb.max_weight.values)
            first = sa.block.values[dif].min() - 2000 if dif.any() else None
            rows.append({"post-fork demand": c, "series differ (share of samples)": dif.mean(), "first difference (blocks after fork)": first,
                         "max M_L ratio spec÷ref": (sb.m_l.values / sa.m_l.values).max()})
    out.append("\nMainnet start: where the two behaviours first differ (seed 1)\n")
    out.append(md(pd.DataFrame(rows), {"series differ (share of samples)": "{:.3f}", "max M_L ratio spec÷ref": "{:.3f}"}))
    write("q7.md", "\n".join(out))

    fig, axes = plt.subplots(2, 3, figsize=(11, 5.6), sharex=True)
    for row, wal in enumerate(["auto", "fill_after"]):
        for X, col in [("300k", "#52514e"), ("450k", C["v16"]), ("600k", C["v17_sf50"]), ("1000k", C["v17"])]:
            for rs, ls in [(False, "-"), (True, "--")]:
                x = d[(d.pre == f"ML{X}_base38") & (d.wallets == wal) & (d["fork.rescale"] == rs)].iloc[0]
                s = series("q7_fork", int(x.run_id))
                t = (s.block - 2000) / 1000
                lab = f"pre-fork M_L {X}, {'spec rescale' if rs else 'reference'}"
                axes[row, 0].plot(t, s.m_l / 1e6, ls=ls, color=col, label=lab)
                axes[row, 1].plot(t, s.max_weight / 1e6, ls=ls, color=col, label=lab)
                axes[row, 2].plot(t, s.fee_low, ls=ls, color=col, label=lab)
        axes[row, 0].set_ylabel("after fork: " + ("38 tx/block" if wal == "auto" else "demand > capacity"))
        axes[row, 2].set_yscale("log")
        axes[row, 1].set_yscale("log")
    axes[0, 0].set_title("M_L (MB)", loc="left"); axes[0, 1].set_title("max block weight (MB)", loc="left"); axes[0, 2].set_title("lowest fee level (XMR/byte)", loc="left")
    for ax in axes[1]:
        ax.set_xlabel("thousand blocks after the fork")
    h, l = axes[0, 0].get_legend_handles_labels()
    fig.legend(h, l, loc="lower center", ncol=4, fontsize=7, bbox_to_anchor=(0.5, -0.06))
    fig.suptitle("Q7: v16→v17 fork; solid = reference code (no rescale), dashed = spec transitional rule", x=0.01, ha="left")
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
    out.append(f"- long-term weights: median {m.long_term_weight.median():,.0f} (= 300,000 × 10/17, the lower clamp), so M_L = Z_M = 300,000; blocks above 300,000: {(m.block_weight>300000).sum():,}")
    p = os.path.join(data, "stressnet_weights.csv.gz")
    if os.path.exists(p):
        s = pd.read_csv(p, comment="#")
        f = s[s.height >= 3103520]
        one = f[f.num_txes == 1].block_weight
        out.append(f"\n### Stressnet (testnet with the FCMP++ fork, hard fork version 18 from height 3,103,520), heights {f.height.min():,}–{f.height.max():,}\n")
        out.append(f"- every small post-fork block has long-term weight {f.long_term_weight.mode().iloc[0]:,} = 10,000,000 × 5/6, i.e. the v17 lower clamp around M_L = 10,000,000: "
                   "stressnet runs with a 10 MB penalty-free zone, so its absolute sizes are not the mainnet parameters")
        out.append(f"- most common weights of one-transaction blocks: {', '.join(f'{k:,}' for k in one.value_counts().head(5).index)}; "
                   "minus the coinbase (107, 237 or 328 bytes in empty blocks) these are exactly 9,093 (2-in/2-out) and 7,650 (1-in/2-out), as the simulator assumes")
        over = f[f.block_weight > 10_000_000]
        out.append(f"- {len(over)} blocks exceeded the 10,000,000 penalty-free size, by up to {(f.block_weight.max()/1e7-1)*100:.1f}% — the 'slightly exceed M_N when fees cover the penalty' mechanism, seen on a live network")
        out.append(f"- transactions per post-fork block: mean {f.num_txes.mean():.1f}; bytes per transaction {f.block_weight.sum()/max(1,f.num_txes.sum()):,.0f}")
    write("chains.md", "\n".join(out))


if __name__ == "__main__":
    which = sys.argv[1:] or ["chains", "q1", "q2", "q3", "q4", "q5", "q6", "q7"]
    for w in which:
        globals()[w]()
