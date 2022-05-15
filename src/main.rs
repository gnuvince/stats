use getopts::Options;
use separator::Separatable;
use std::cmp::Ordering;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process::exit;

const PROGNAME: &str = "stats";
const VERSION: &str = env!("CARGO_PKG_VERSION");

// NB(vfoley): Usually I would use an enum, but it's annoying to
// convert enum items to/from usize integers in Rust. Since this is
// just a small, simple program, I make my life easier by defining a
// bunch of constants.
mod stat {
    pub const LEN: usize = 0;
    pub const SUM: usize = 1;
    pub const MIN: usize = 2;
    pub const MAX: usize = 3;
    pub const AVG: usize = 4;
    pub const STD: usize = 5;
    pub const MODE: usize = 6;
    pub const MODE_OCC: usize = 7;
    pub const P50: usize = 8;
    pub const P75: usize = 9;
    pub const P90: usize = 10;
    pub const P95: usize = 11;
    pub const P99: usize = 12;
    pub const COUNT: usize = 13;

    pub const NAMES: [&str; COUNT] = [
        "len", "sum", "min", "max", "avg", "std", "mode", "mode#", "p50", "p75", "p90", "p95",
        "p99",
    ];
}

// NB(vfoley): There are two stats, len and mode#, that really ought to be u64s.
type Stats = [f64; stat::COUNT];

struct DisplayOpts {
    short: bool,
    thousands: bool,
}

// Display one stat per line, with the name of the stat
fn print_long(filename: &str, stats: &Stats, thousands: bool) {
    println!("{}", filename);
    for i in 0..stat::COUNT {
        if thousands {
            println!("  {:6}  {}", stat::NAMES[i], stats[i].separated_string());
        } else {
            println!("  {:6}  {}", stat::NAMES[i], stats[i]);
        }
    }
}

// Display all stats on a single line
fn print_short(filename: &str, stats: &Stats, thousands: bool) {
    print!("{}", filename);
    for s in stats {
        if thousands {
            print!(" {}", s.separated_string());
        } else {
            print!(" {}", s);
        }
    }
    println!("");
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

fn stats(v: &mut Vec<f64>) -> Stats {
    v.sort_unstable_by(|a, b| match a.partial_cmp(b) {
        Some(ordering) => ordering,
        None => Ordering::Less,
    });
    let mut s: Stats = [0.0; stat::COUNT];
    s[stat::LEN] = v.len() as f64;
    s[stat::P50] = percentile(&v, 0.50);
    s[stat::P75] = percentile(&v, 0.75);
    s[stat::P90] = percentile(&v, 0.90);
    s[stat::P95] = percentile(&v, 0.95);
    s[stat::P99] = percentile(&v, 0.99);
    s[stat::MIN] = *v.first().unwrap_or(&std::f64::NAN);
    s[stat::MAX] = *v.last().unwrap_or(&std::f64::NAN);

    let n = s[stat::LEN];
    // Variables for computing average and standard deviation
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    // Variables for computing the mode
    let mut mode_val = std::f64::NAN;
    let mut mode_count = 0;
    let mut mode_candidate = std::f64::NAN;
    let mut mode_candidate_count = 0;
    for x in v {
        sum += *x;
        sum_sq += *x * *x;

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
    s[stat::SUM] = sum;
    s[stat::AVG] = sum / n;
    s[stat::MODE] = mode_val;
    s[stat::MODE_OCC] = mode_count as f64;
    // https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Na%C3%AFve_algorithm
    s[stat::STD] = f64::sqrt((sum_sq - (sum * sum) / n) / n);
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

    let display_opts = DisplayOpts {
        short: matches.opt_present("c"),
        thousands: matches.opt_present("s"),
    };

    let quiet = matches.opt_present("q");

    // Display titles for compact output format
    if display_opts.short && matches.opt_present("t") {
        println!("filename len sum min max avg std mode mode# p50 p75 p90 p95 p99");
    }

    let print_stats = if display_opts.short {
        print_short
    } else {
        print_long
    };

    let mut ret = 0;
    if matches.free.is_empty() {
        matches.free.push("-".to_owned());
    }

    let mut buf = String::with_capacity(4096);
    let mut v: Vec<f64> = Vec::with_capacity(4096);

    for filename in &matches.free {
        v.clear();
        let mut reader: Box<dyn BufRead> = match bufreader_from_file(filename) {
            Ok(r) => r,
            Err(e) => {
                ret = 1;
                if !quiet {
                    eprintln!("{}: {}: {}", PROGNAME, filename, e);
                }
                continue;
            }
        };

        loop {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => break,
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}: cannot read line: {}", PROGNAME, e);
                    break;
                }
            }
            let trimmed = buf.trim();
            match fast_float::parse::<f64, _>(trimmed) {
                Err(_) => {
                    if !quiet {
                        eprintln!("{}: cannot convert {:?} to a number", PROGNAME, trimmed);
                    }
                }
                Ok(x) => {
                    if x.is_finite() {
                        v.push(x);
                    } else if !quiet {
                        eprintln!("{}: skipping {}", PROGNAME, trimmed);
                    }
                }
            }
        }

        let s = stats(&mut v);
        print_stats(filename, &s, display_opts.thousands);
    }
    exit(ret);
}

#[test]
fn mode() {
    // Mode of an empty vector is NAN
    let s = stats(&mut vec![]);
    assert!(s[stat::MODE].is_nan());

    // Mode of a singleton vector is its only value.
    let s = stats(&mut vec![1.0]);
    assert_eq!(1.0, s[stat::MODE]);

    // In stats, mode ties are broken by taking the smallest mode.
    let s = stats(&mut vec![2.0, 1.0]);
    assert_eq!(1.0, s[stat::MODE]);
    let s = stats(&mut vec![2.0, 1.0, 3.0]);
    assert_eq!(1.0, s[stat::MODE]);

    let s = stats(&mut vec![2.0, 1.0, 2.0]);
    assert_eq!(2.0, s[stat::MODE]);
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
