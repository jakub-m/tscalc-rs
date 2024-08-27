use chrono::TimeDelta;

const NS: i64 = 1;
const US: i64 = 1000 * NS;
const MS: i64 = 1000 * US;
const S: i64 = 1000 * MS;
const M: i64 = 60 * S;
const H: i64 = 60 * M;
const D: i64 = 24 * H;

pub trait ShortFormat {
    fn from_short_format() -> Result<TimeDelta, String>;
    fn as_short_format(&self) -> String;
}

impl ShortFormat for TimeDelta {
    fn from_short_format() -> Result<TimeDelta, String> {
        todo!()
    }

    fn as_short_format(&self) -> String {
        let mut ns = self.num_nanoseconds().unwrap();
        let mut neg = false;
        if ns < 0 {
            ns = -ns;
            neg = true;
        }

        let mut consume = |part_in_ns: i64| {
            let c = ns / part_in_ns;
            ns = ns - c * part_in_ns;
            c
        };
        let mut s = String::from(if neg { "-" } else { "" });
        let mut display = |val, symbol| {
            if val != 0 {
                s += format!("{}{}", val, symbol).as_str();
            }
        };
        let days = consume(D);
        display(days, "d");
        let hours = consume(H);
        display(hours, "h");
        let minutes = consume(M);
        display(minutes, "m");
        let seconds = consume(S);
        display(seconds, "s");
        let millis = consume(MS);
        display(millis, "ms");
        let micros = consume(US);
        display(micros, "us");
        let nanos = consume(NS);
        display(nanos, "ns");
        if s == "" {
            s = "0s".to_string();
        }
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::ShortFormat;
    use crate::parser::duration::*;

    #[test]
    fn format_large() {
        let d = chrono::TimeDelta::nanoseconds(D + H + M + S + MS + US + NS);
        assert_eq!("1d1h1m1s1ms1us1ns", d.as_short_format());
    }
    #[test]
    fn format_3() {
        let d = chrono::TimeDelta::nanoseconds(D + M + MS);
        assert_eq!("1d1m1ms", d.as_short_format());
    }
    #[test]
    fn format_zero() {
        let d = chrono::TimeDelta::nanoseconds(0);
        assert_eq!("0s", d.as_short_format());
    }
    #[test]
    fn format_neg_small() {
        let d = chrono::TimeDelta::nanoseconds(-3 * H);
        assert_eq!("-3h", d.as_short_format());
    }
    #[test]
    fn format_neg_large() {
        let d = chrono::TimeDelta::nanoseconds(-(D + H + M + S + MS + US + NS));
        assert_eq!("-1d1h1m1s1ms1us1ns", d.as_short_format());
    }
}
