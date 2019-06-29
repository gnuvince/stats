stats
=====

A simple command-line utility that reads a stream of floating-point
numbers and outputs descriptive statistics such as the minimum, the
maximum, the mean, the standard deviation, some percentiles, etc.

Installing
----------

You can download a pre-built binary in the [releases page](https://github.com/gnuvince/stats/releases).

Building
--------

To build and install stats, clone this repository and run the
following command:

```
~/src/stats$ cargo install --path=.
```

Usage
-----

```
Usage: stats [-chv] [FILES]

Options:
    -c, --compact       display each file on one line
    -h, --help          display help
    -v, --version       display version
```

Examples
--------

```
$ seq 1 10 | stats
-
  len  10
  sum  55.00000
  min  1.00000
  max  10.00000
  mean 5.50000
  std  2.87228
  p50  6.00000
  p75  8.00000
  p90  10.00000
  p95  10.00000
  p99  10.00000


$ seq 1 10 | stats -c
- 10 55.00000 1.00000 10.00000 5.50000 2.87228 6.00000 8.00000 10.00000 10.00000 10.00000
```

License
-------

stats is licensed under the terms of the MIT license.
See LICENSE for more information.
