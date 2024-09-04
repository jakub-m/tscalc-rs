tscalc-rs
---------

A simple date-time calculator.

Features:

- Datetime in ISO format at input like `2024-08-25T16:48:25+00:00`.
- Timestamps like `1724606867.000`.
- `now` keyword. For sake of simplicity, the "now" time is rounded to seconds.
- Arithmetic on time deltas, like `now + 1d - 2m - 1s`.
- Brackets: `now - (1d + 2m)`.
- Arithmetic on times and sub-expressions: `now + (2000-01-01T01:00:00Z - 2000-01-01T00:00:00Z)`.
- Built-in functions: `full_day` and `full_hour`, like `full_day(now)`.

Usage:

```bash
% ./tscalc-rs -- 'full_day(now) + (2000-01-01T00:00:00Z - 1234567890.000) + 1d - 2h - 3s'
2015-07-12T22:28:27+00:00
```

It is a rewrite of [a similar toy tool in Go][ref_go].

[ref_go]:https://github.com/jakub-m/toolbox/tree/main/tscalc

# Bugs

- The output should be in tz timezone, not in UTC:

```
% ,tscalc -tz US/Eastern -- '2024-09-04T06:00:00+00:00'
2024-09-04T06:00:00+00:00
```
