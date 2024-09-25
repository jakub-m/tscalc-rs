## tscalc

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
% ./tscalc -- 'full_day(now) + (2000-01-01T00:00:00Z - 1234567890.000) + 1d - 2h - 3s'
2015-07-12T22:28:27+00:00
```

It is a rewrite of [a similar toy tool in Go][ref_go].

[ref_go]: https://github.com/jakub-m/toolbox/tree/main/tscalc

# Build

```bash
make release
```

Find the binary in:

```bash
./target/release/tscalc
```

# Recipes

Generate a sequence of times separated by minute in custom format:

```bash
seq 1440 | while read d; do tscalc -f %F-%H-%M -- "2024-06-01T00:00:00Z + ${d}m"; done 
```


# TODO. Known bugs, missing features

- Dates like `2024-01-01` are not recognised.
