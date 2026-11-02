//! Study harness: `study SCENARIO.json [--out DIR] [--threads N] [--filter SUBSTR]`
//!
//! Expands the scenario's cases x grid x seeds, runs them in parallel, and writes into
//! `DIR/<scenario name>/`:
//!   summary.csv   one row per run (scenario variables first, then metrics)
//!   events.csv    exact values at events (window start/end, thresholds, fork, snapshots)
//!   runs.jsonl    the full expanded RunSpec of each run
//!   series/NNNNN.csv.gz  downsampled series (every `series_every` blocks)
//!   meta.json     git commit, scenario file and a copy of it, seeds, timing

mod run;
mod spec;

use rayon::prelude::*;
use std::io::Write;
use std::time::Instant;

fn git_commit() -> String {
    let rev = std::process::Command::new("git").args(["rev-parse", "HEAD"]).output();
    let dirty = std::process::Command::new("git").args(["status", "--porcelain", "--untracked-files=no"]).output();
    match (rev, dirty) {
        (Ok(r), Ok(d)) => {
            let mut s = String::from_utf8_lossy(&r.stdout).trim().to_string();
            if !d.stdout.is_empty() {
                s.push_str("-dirty");
            }
            s
        }
        _ => "unknown".into(),
    }
}

fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut scen_path = None;
    let mut out_dir = "results".to_string();
    let mut threads = 0usize;
    let mut filter: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                out_dir = args[i + 1].clone();
                i += 1;
            }
            "--threads" => {
                threads = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--filter" => {
                filter = Some(args[i + 1].clone());
                i += 1;
            }
            a => scen_path = Some(a.to_string()),
        }
        i += 1;
    }
    let scen_path = scen_path.expect("usage: study SCENARIO.json [--out DIR] [--threads N] [--filter SUBSTR]");
    if threads > 0 {
        rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();
    }
    let text = std::fs::read_to_string(&scen_path).expect("read scenario");
    let scen: spec::Scenario = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{scen_path}: {e}"));
    let mut runs = spec::expand(&scen).unwrap_or_else(|e| panic!("{scen_path}: {e}"));
    if let Some(f) = &filter {
        runs.retain(|r| r.case_label.contains(f.as_str()));
    }
    let dir = format!("{out_dir}/{}", scen.name);
    std::fs::create_dir_all(format!("{dir}/series")).unwrap();

    let needs_history = runs.iter().any(|r| r.spec.init == "mainnet");
    let history = if needs_history {
        let p = std::env::var("STUDY_HISTORY").unwrap_or_else(|_| "data/mainnet_weights.csv.gz".into());
        match run::load_history(&p) {
            Ok(h) => {
                eprintln!("loaded {} blocks of history from {p}", h.0.len());
                Some(h)
            }
            Err(e) => {
                eprintln!("WARNING: init=mainnet requested but {e}; those runs will fail");
                None
            }
        }
    } else {
        None
    };

    let commit = git_commit();
    let started = Instant::now();
    eprintln!("{}: {} runs on {} threads", scen.name, runs.len(), rayon::current_num_threads());
    let done = std::sync::atomic::AtomicUsize::new(0);
    let results: Vec<(usize, Result<run::RunOutput, String>, f64)> = runs
        .par_iter()
        .map(|r| {
            let t0 = Instant::now();
            let sp = format!("{dir}/series/{:05}.csv.gz", r.id);
            let want = scen.series_seeds.map_or(true, |k| r.seed_index < k);
            let out = run::run(&r.spec, r.id, r.seed, scen.series_every, want.then_some(sp.as_str()), history.as_ref());
            let k = done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            eprint!("\r  {k}/{}", runs.len());
            (r.id, out, t0.elapsed().as_secs_f64())
        })
        .collect();
    eprintln!();

    // summary.csv
    let mut sum = std::io::BufWriter::new(std::fs::File::create(format!("{dir}/summary.csv")).unwrap());
    let mut evf = std::io::BufWriter::new(std::fs::File::create(format!("{dir}/events.csv")).unwrap());
    writeln!(evf, "{}", run::EVENT_HEADER).unwrap();
    let mut jl = std::io::BufWriter::new(std::fs::File::create(format!("{dir}/runs.jsonl")).unwrap());
    let mut header_done = false;
    let mut failures = 0;
    for (r, (id, res, secs)) in runs.iter().zip(&results) {
        assert_eq!(r.id, *id);
        writeln!(jl, "{}", serde_json::json!({"id": r.id, "seed": r.seed, "spec": r.json})).unwrap();
        let out = match res {
            Ok(o) => o,
            Err(e) => {
                eprintln!("run {id} failed: {e}");
                failures += 1;
                continue;
            }
        };
        let mut cols: Vec<(String, String)> = vec![
            ("run_id".into(), r.id.to_string()),
            ("case".into(), r.case_label.clone()),
            ("rules".into(), r.spec.rules.clone()),
            ("seed".into(), r.seed.to_string()),
            ("seed_index".into(), r.seed_index.to_string()),
            ("blocks".into(), r.spec.blocks.to_string()),
        ];
        for (k, v) in &r.vars {
            let s = match v {
                serde_json::Value::String(s) => s.clone(),
                o => o.to_string(),
            };
            cols.push((k.clone(), s));
        }
        cols.extend(out.summary.0.iter().cloned());
        cols.push(("secs".into(), format!("{secs:.2}")));
        if !header_done {
            writeln!(sum, "{}", cols.iter().map(|c| csv_field(&c.0)).collect::<Vec<_>>().join(",")).unwrap();
            header_done = true;
        }
        writeln!(sum, "{}", cols.iter().map(|c| csv_field(&c.1)).collect::<Vec<_>>().join(",")).unwrap();
        for e in &out.events {
            writeln!(evf, "{e}").unwrap();
        }
    }
    let meta = serde_json::json!({
        "scenario": scen.name,
        "description": scen.description,
        "scenario_file": scen_path,
        "scenario_text": text,
        "git_commit": commit,
        "runs": runs.len(),
        "failures": failures,
        "seeds": scen.seeds,
        "seed_base": scen.seed_base,
        "series_every": scen.series_every,
        "series_seeds": scen.series_seeds,
        "history": if needs_history { std::env::var("STUDY_HISTORY").unwrap_or_else(|_| "data/mainnet_weights.csv.gz".into()) } else { String::new() },
        "wall_seconds": started.elapsed().as_secs_f64(),
        "threads": rayon::current_num_threads(),
    });
    std::fs::write(format!("{dir}/meta.json"), serde_json::to_string_pretty(&meta).unwrap()).unwrap();
    eprintln!("{}: wrote {dir} in {:.1}s ({failures} failures)", scen.name, started.elapsed().as_secs_f64());
    if failures > 0 {
        std::process::exit(1);
    }
}
