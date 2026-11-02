#!/usr/bin/env python3
"""Fetch the latest block weights and long-term weights from a monerod RPC.

Usage: MONERO_RPC=http://host:port python3 fetch_chain.py OUT.csv.gz [COUNT]
Writes height, block_weight, long_term_weight, num_txes, timestamp, reward for the latest
COUNT blocks (default 100000), oldest first. Exits 2 if MONERO_RPC is unset or unreachable.
"""
import gzip, json, os, sys, urllib.request

def rpc(url, method, params=None):
    body = json.dumps({"jsonrpc": "2.0", "id": "0", "method": method, "params": params or {}}).encode()
    req = urllib.request.Request(url + "/json_rpc", body, {"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=60) as r:
        out = json.load(r)
    if "error" in out:
        raise RuntimeError(out["error"])
    return out["result"]

def main():
    url = os.environ.get("MONERO_RPC")
    if not url:
        print("MONERO_RPC not set", file=sys.stderr); sys.exit(2)
    out = sys.argv[1]
    count = int(sys.argv[2]) if len(sys.argv) > 2 else 100_000
    try:
        info = rpc(url, "get_info")
    except Exception as e:
        print(f"RPC unreachable: {e}", file=sys.stderr); sys.exit(2)
    top = info["height"] - 1
    start = top - count + 1
    rows = []
    h = start
    while h <= top:
        end = min(h + 999, top)
        res = rpc(url, "get_block_headers_range", {"start_height": h, "end_height": end})
        for b in res["headers"]:
            rows.append((b["height"], b["block_weight"], b["long_term_weight"], b["num_txes"], b["timestamp"], b["reward"]))
        h = end + 1
        print(f"\r{h - start}/{count}", end="", file=sys.stderr)
    rows.sort()
    with gzip.open(out, "wt") as f:
        f.write(f"# source={url} nettype={info.get('nettype')} top_height={top} version={info.get('version')}\n")
        f.write("height,block_weight,long_term_weight,num_txes,timestamp,reward\n")
        for r in rows:
            f.write(",".join(map(str, r)) + "\n")
    print(f"\nwrote {len(rows)} rows to {out}", file=sys.stderr)

if __name__ == "__main__":
    main()
