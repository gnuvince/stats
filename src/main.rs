use getopts::Options;
use std::cmp::Ordering;
use std::env;
use std::io::{self, BufReader, BufRead};
use std::fs::File;
use std::process::exit;

#[derive(Default, Debug)]
struct Stats {
    len: usize,
    sum: f64,
    min: f64,
    max: f64,
    mean: f64,
    std: f64,
    p50: f64,
    p75: f64,
    p90: f64,
    p95: f64,
    p99: f64,
}

fn fmt_full(filename: &str, stats: &Stats) {
    println!("{}", filename);
    println!("  len  {}", stats.len);
    println!("  sum  {:.05}", stats.sum);
    println!("  min  {:.05}", stats.min);
    println!("  max  {:.05}", stats.max);
    println!("  mean {:.05}", stats.mean);
    println!("  std  {:.05}", stats.std);
    println!("  p50  {:.05}", stats.p50);
    println!("  p75  {:.05}", stats.p75);
    println!("  p90  {:.05}", stats.p90);
    println!("  p95  {:.05}", stats.p95);
    println!("  p99  {:.05}", stats.p99);
}

fn fmt_compact(filename: &str, stats: &Stats) {
    println!(
        "{} {} {:.05} {:.05} {:.05} {:.05} {:.05} {:.05} {:.05} {:.05} {:.05} {:.05}",
        filename, stats.len, stats.sum, stats.min, stats.max, stats.mean, stats.std,
        stats.p50, stats.p75, stats.p90, stats.p95, stats.p99);
}

fn percentile(v: &[f64], num: usize, denom: usize) -> f64 {
    assert!(num < denom, "the percentile needs to be smaller than 1");
    match v.len() {
        0 => std::f64::NAN,
        n => v[n * num / denom],
    }
}

fn stats(mut v: Vec<f64>) -> Stats {
    v.sort_unstable_by(|a, b| match a.partial_cmp(b) {
        Some(ordering) => ordering,
        None => Ordering::Less,
    });
    let mut s = Stats::default();
    s.len = v.len();
    s.p50 = percentile(&v, 1, 2);
    s.p75 = percentile(&v, 3, 4);
    s.p90 = percentile(&v, 9, 10);
    s.p95 = percentile(&v, 19, 20);
    s.p99 = percentile(&v, 99, 100);

    s.min = std::f64::INFINITY;
    s.max = std::f64::NEG_INFINITY;
    for x in &v {
        s.sum += *x;
        s.min = f64::min(s.min, *x);
        s.max = f64::max(s.max, *x);
    }
    s.mean = s.sum / (s.len as f64);
    for x in &v {
        let d = x - s.mean;
        s.std += d*d;
    }
    s.std = f64::sqrt(s.std / (s.len as f64));
    return s;
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [-ch] [FILES]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("c", "compact", "display each file on one line");
    opts.optflag("h", "help", "display help");

    let mut matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(e) => {
            eprintln!("{}: {}", program, e);
            exit(1);
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        exit(0);
    }

    let mut out_fn: fn(&str, &Stats) = fmt_full;
    if matches.opt_present("c") {
        out_fn = fmt_compact;
    }

    let mut ret = 0;
    if matches.free.is_empty() {
        matches.free.push("-".to_owned());
    }

    for filename in matches.free {
        let mut v: Vec<f64> = Vec::with_capacity(1024);

        let reader: Box<dyn BufRead> =
            if filename == "-" {
                let stdin = io::stdin();
                Box::new(BufReader::new(stdin))
            } else {
                match File::open(&filename) {
                    Ok(f) => Box::new(BufReader::new(f)),
                    Err(e) => {
                        eprintln!("{}: {}: {}", program, filename, e);
                        ret = 1;
                        continue;
                    }
                }
            };

        for line in reader.lines() {
            let line = line.unwrap();
            match str::parse::<f64>(&line) {
                Ok(x) => v.push(x),
                Err(e) => eprintln!("stats: {:?} {}", line, e),
            }
        }

        let s = stats(v);
        out_fn(&filename, &s);
    }
    exit(ret);
}
