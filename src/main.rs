use std::cmp::Ordering;
use std::fmt;
use std::io::{self, BufRead};

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

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "len  {}", self.len)?;
        writeln!(f, "sum  {:.05}", self.sum)?;
        writeln!(f, "min  {:.05}", self.min)?;
        writeln!(f, "max  {:.05}", self.max)?;
        writeln!(f, "mean {:.05}", self.mean)?;
        writeln!(f, "std  {:.05}", self.std)?;
        writeln!(f, "p50  {:.05}", self.p50)?;
        writeln!(f, "p75  {:.05}", self.p75)?;
        writeln!(f, "p90  {:.05}", self.p90)?;
        writeln!(f, "p95  {:.05}", self.p95)?;
        writeln!(f, "p99  {:.05}", self.p99)?;
        return Ok(());
    }
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

fn main() {
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let mut v: Vec<f64> = Vec::with_capacity(1024);

    for line in stdin.lines() {
        let line = line.unwrap();
        match str::parse::<f64>(&line) {
            Ok(x) => v.push(x),
            Err(e) => eprintln!("stats: {}", e),
        }
    }

    let s = stats(v);
    println!("{}", s);
}
