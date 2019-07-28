stats
=====

A simple command-line utility that reads a stream of floating-point
numbers and outputs descriptive statistics such as the minimum, the
maximum, the mean, the standard deviation, some percentiles, etc.

The primary usage for `stats` is to give you a general sense of the data.
**It is not meant to compute statistics precisely!**
We take shortcuts in `stats` to make the implementation simple and fast,
but which make it unsuitable for serious statistical work:

1. We show only one mode, not all of them
2. We do not take the average of the two middle elements when computing `p50`, we just take the middle element as determined by integer division.
   (The same applies for all percentiles.)
3. We make no effort to protect against numeric errors
4. NaN and infinities are filtered out, so you can't tell if your original data
   had them

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
