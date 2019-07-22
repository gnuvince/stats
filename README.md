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
Usage: stats [-chstv] [FILES]

Options:
    -c, --compact       display each file on one line
    -h, --help          display help
    -s, --silent        suppress error messages
    -t, --title         display column titles (compact mode)
    -v, --version       display version
```

Examples
--------

```
$ ( seq 1 10 ; yes 4.2 | head -4 ) | stats
-
  len    14
  sum    71.8
  min    1
  max    10
  avg    5.12857
  std    2.49755
  mode   4.2
  mode#  4
  p50    4.2
  p75    7
  p90    9
  p95    10
  p99    10


$ ( seq 1 10 ; yes 4.2 | head -4 ) | stats -ct | column -t
filename  len  sum   min  max  avg      std      mode  mode#  p50  p75  p90  p95  p99
-         14   71.8  1    10   5.12857  2.49755  4.2   4      4.2  7    9    10   10
```

License
-------

stats is licensed under the terms of the MIT license.
See LICENSE for more information.
