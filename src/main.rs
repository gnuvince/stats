use getopts::Options;
use separator::{FixedPlaceSeparatable, Separatable};
use std::cmp::Ordering;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process::exit;

const PROGNAME: &str = "stats";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, Debug)]
struct Stats {
    len: usize,
    sum: f64,
    min: f64,
    max: f64,
    avg: f64,
    std: f64,
    mode: f64,
    mode_occ: usize,
    p50: f64,
    p75: f64,
    p90: f64,
    p95: f64,
    p99: f64,
}

/// Displays one stat (and its title) per line; good for humans.
fn fmt_full(filename: &str, stats: &Stats, thousands_separators: bool) {
    if thousands_separators {
        println!("{}", filename);
        println!("  len    {}", stats.len.separated_string());
        println!("  sum    {}", stats.sum.separated_string());
        println!("  min    {}", stats.min.separated_string());
        println!("  max    {}", stats.max.separated_string());
        println!(
            "  avg    {}",
            stats.avg.separated_string_with_fixed_place(5)
        );
        println!(
            "  std    {}",
            stats.std.separated_string_with_fixed_place(5)
        );
        println!("  mode   {}", stats.mode.separated_string());
        println!("  mode#  {}", stats.mode_occ.separated_string());
        println!("  p50    {}", stats.p50.separated_string());
        println!("  p75    {}", stats.p75.separated_string());
        println!("  p90    {}", stats.p90.separated_string());
        println!("  p95    {}", stats.p95.separated_string());
        println!("  p99    {}", stats.p99.separated_string());
    } else {
        println!("{}", filename);
        println!("  len    {}", stats.len);
        println!("  sum    {}", stats.sum);
        println!("  min    {}", stats.min);
        println!("  max    {}", stats.max);
        println!("  avg    {:.5}", stats.avg);
        println!("  std    {:.5}", stats.std);
        println!("  mode   {}", stats.mode);
        println!("  mode#  {}", stats.mode_occ);
        println!("  p50    {}", stats.p50);
        println!("  p75    {}", stats.p75);
        println!("  p90    {}", stats.p90);
        println!("  p95    {}", stats.p95);
        println!("  p99    {}", stats.p99);
    }
}

/// Displays all stats on a single line (no titles); good for pipelines.
fn fmt_compact(filename: &str, stats: &Stats, thousands_separators: bool) {
    if thousands_separators {
        println!(
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            filename,
            stats.len.separated_string(),
            stats.sum.separated_string(),
            stats.min.separated_string(),
            stats.max.separated_string(),
            stats.avg.separated_string_with_fixed_place(5),
            stats.std.separated_string_with_fixed_place(5),
            stats.mode.separated_string(),
            stats.mode_occ.separated_string(),
            stats.p50.separated_string(),
            stats.p75.separated_string(),
            stats.p90.separated_string(),
            stats.p95.separated_string(),
            stats.p99.separated_string()
        );
    } else {
        println!(
            "{} {} {} {} {} {:.05} {:.05} {} {} {} {} {} {} {}",
            filename,
            stats.len,
            stats.sum,
            stats.min,
            stats.max,
            stats.avg,
            stats.std,
            stats.mode,
            stats.mode_occ,
            stats.p50,
            stats.p75,
            stats.p90,
            stats.p95,
            stats.p99
        );
    }
}

fn percentile(v: &[f64], p: f64) -> f64 {
    assert!(
        p > 0.0 && p < 1.0,
        "the percentile must be in the range ]0,1["
    );
    match v.len() {
        0 => std::f64::NAN,
        n => {
            let i: usize = (n as f64 * p) as usize;
            v[i]
        }
    }
}

fn stats(mut v: Vec<f64>) -> Stats {
    v.sort_unstable_by(|a, b| match a.partial_cmp(b) {
        Some(ordering) => ordering,
        None => Ordering::Less,
    });
    let mut s = Stats::default();
    s.len = v.len();
    s.p50 = percentile(&v, 0.50);
    s.p75 = percentile(&v, 0.75);
    s.p90 = percentile(&v, 0.90);
    s.p95 = percentile(&v, 0.95);
    s.p99 = percentile(&v, 0.99);
    s.min = *v.first().unwrap_or(&std::f64::NAN);
    s.max = *v.last().unwrap_or(&std::f64::NAN);

    let n = s.len as f64;
    // Variables for computing average and standard deviation
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    // Variables for computing the mode
    let mut mode_val = std::f64::NAN;
    let mut mode_count = 0;
    let mut mode_candidate = std::f64::NAN;
    let mut mode_candidate_count = 0;
    for x in &v {
        sum += x;
        sum_sq += x * x;

        if *x == mode_candidate {
            mode_candidate_count += 1;
        } else {
            if mode_candidate_count > mode_count {
                mode_val = mode_candidate;
                mode_count = mode_candidate_count;
            }
            mode_candidate_count = 1;
        }

        mode_candidate = *x;
    }

    if mode_candidate_count > mode_count {
        mode_val = mode_candidate;
        mode_count = mode_candidate_count;
    }
    s.sum = sum;
    s.avg = s.sum / n;
    s.mode = mode_val;
    s.mode_occ = mode_count;
    // https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Na%C3%AFve_algorithm
    s.std = f64::sqrt((sum_sq - (sum * sum) / n) / n);
    return s;
}

fn bufreader_from_file(filename: &str) -> io::Result<Box<dyn BufRead>> {
    if filename == "-" {
        let stdin = io::stdin();
        Ok(Box::new(BufReader::new(stdin)))
    } else {
        let f = File::open(&filename)?;
        Ok(Box::new(BufReader::new(f)))
    }
}

fn main() {
    let mut opts = Options::new();
    opts.optflag("c", "compact", "display each file on one line");
    opts.optflag("h", "help", "display help");
    opts.optflag("q", "quiet", "suppress error messages");
    opts.optflag("s", "separators", "use thousand separators");
    opts.optflag("t", "title", "display column titles (compact mode)");
    opts.optflag("v", "version", "display version");

    let mut matches = match opts.parse(env::args_os().skip(1)) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}: {}", PROGNAME, e);
            exit(1);
        }
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [-chstv] [FILES]", PROGNAME);
        print!("{}", opts.usage(&brief));
        exit(0);
    }

    if matches.opt_present("v") {
        println!("{}", VERSION);
        exit(0);
    }

    let quiet = matches.opt_present("q");
    let thousands_separators = matches.opt_present("s");

    let out_fn: fn(&str, &Stats, bool) = if matches.opt_present("c") {
        fmt_compact
    } else {
        fmt_full
    };

    if matches.opt_present("c") && matches.opt_present("t") {
        println!("filename len sum min max avg std mode mode# p50 p75 p90 p95 p99");
    }

    let mut ret = 0;
    if matches.free.is_empty() {
        matches.free.push("-".to_owned());
    }

    for filename in &matches.free {
        let mut v: Vec<f64> = Vec::with_capacity(1024);

        let reader: Box<dyn BufRead> = match bufreader_from_file(filename) {
            Ok(r) => r,
            Err(e) => {
                ret = 1;
                if !quiet {
                    eprintln!("{}: {}: {}", PROGNAME, filename, e);
                }
                continue;
            }
        };

        for line in reader.lines() {
            let line = line.unwrap();
            match str::parse::<f64>(&line) {
                Err(e) => {
                    if !quiet {
                        eprintln!("{}: {:?} {}", PROGNAME, line, e);
                    }
                }
                Ok(x) => {
                    if x.is_finite() {
                        v.push(x);
                    } else if !quiet {
                        eprintln!("{}: skipping {}", PROGNAME, line);
                    }
                }
            }
        }

        let s = stats(v);
        out_fn(filename, &s, thousands_separators);
    }
    exit(ret);
}

#[test]
fn mode() {
    // Mode of an empty vector is NAN
    let s = stats(vec![]);
    assert!(s.mode.is_nan());

    // Mode of a singleton vector is its only value.
    let s = stats(vec![1.0]);
    assert_eq!(1.0, s.mode);

    // In stats, mode ties are broken by taking the smallest mode.
    let s = stats(vec![2.0, 1.0]);
    assert_eq!(1.0, s.mode);
    let s = stats(vec![2.0, 1.0, 3.0]);
    assert_eq!(1.0, s.mode);

    let s = stats(vec![2.0, 1.0, 2.0]);
    assert_eq!(2.0, s.mode);
}

#[test]
fn test_percentile() {
    let mut v: Vec<f64> = Vec::new();
    for i in 0..100 {
        v.push(i as f64);
    }
    assert_eq!(percentile(&v, 0.5), 50.0);
    assert_eq!(percentile(&v, 0.75), 75.0);
    assert_eq!(percentile(&v, 0.9), 90.0);
    assert_eq!(percentile(&v, 0.95), 95.0);
    assert_eq!(percentile(&v, 0.99), 99.0);
}
