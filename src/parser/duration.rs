use chrono::TimeDelta;

pub const NS: i64 = 1;
pub const US_NS: i64 = 1000 * NS;
pub const MS_NS: i64 = 1000 * US_NS;
pub const SECOND_NS: i64 = 1000 * MS_NS;
pub const MINUTE_NS: i64 = 60 * SECOND_NS;
pub const HOUR_NS: i64 = 60 * MINUTE_NS;
pub const DAY_NS: i64 = 24 * HOUR_NS;

const RE_DURATION: &str = r"^(?<neg>-)?((?<days>\d+)d)?((?<hours>\d+)h)?((?<minutes>\d+)m)?((?<secs>\d+)s)?((?<msecs>\d+)ms)?((?<usecs>\d+)us)?((?<nsecs>\d+)ns)?";

pub trait ShortFormat {
    fn from_short_format(s: &str) -> Result<TimeDelta, String>;
    fn as_short_format(&self) -> String;
}

pub fn match_duration(s: &str) -> Option<&str> {
    let re = regex::Regex::new(RE_DURATION).unwrap();
    let m = re.find(s)?;
    if m.as_str().is_empty() {
        None
    } else {
        Some(m.as_str())
    }
}

impl ShortFormat for TimeDelta {
    fn from_short_format(s: &str) -> Result<TimeDelta, String> {
        let pat = regex::Regex::new(RE_DURATION).unwrap();
        let caps = if let Some(caps) = pat.captures(s) {
            if caps.get(0).unwrap().len() != s.len() {
                return Err(format!("did not match entire input {:?}", s));
            }
            caps
        } else {
            return Err(format!("could not match {:?}", s));
        };

        let mut total_nanos: i64 = 0;
        let mut consume_group = |name, multiplier: i64| {
            let value = caps
                .name(name)
                .map(|m| m.as_str())
                .unwrap_or("0")
                .parse::<i64>()
                .map_err(|e| e.to_string())
                .expect("failed to parse int");
            total_nanos = total_nanos + (value * multiplier);
        };
        consume_group("days", DAY_NS);
        consume_group("hours", HOUR_NS);
        consume_group("minutes", MINUTE_NS);
        consume_group("secs", SECOND_NS);
        consume_group("msecs", MS_NS);
        consume_group("usecs", US_NS);
        consume_group("nsecs", NS);
        if caps.name("neg").is_some() {
            total_nanos = total_nanos * -1;
        }
        Ok(chrono::TimeDelta::nanoseconds(total_nanos))
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
        let days = consume(DAY_NS);
        display(days, "d");
        let hours = consume(HOUR_NS);
        display(hours, "h");
        let minutes = consume(MINUTE_NS);
        display(minutes, "m");
        let seconds = consume(SECOND_NS);
        display(seconds, "s");
        let millis = consume(MS_NS);
        display(millis, "ms");
        let micros = consume(US_NS);
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
    use std::error::Error;

    use super::ShortFormat;
    use crate::parser::duration::*;

    #[test]
    fn format_large() {
        let d = chrono::TimeDelta::nanoseconds(
            1 * DAY_NS
                + 2 * HOUR_NS
                + 3 * MINUTE_NS
                + 4 * SECOND_NS
                + 5 * MS_NS
                + 6 * US_NS
                + 7 * NS,
        );
        assert_eq!("1d2h3m4s5ms6us7ns", d.as_short_format());
    }

    #[test]
    fn parse_large() {
        let actual = chrono::TimeDelta::from_short_format("1d2h3m4s5ms6us7ns").unwrap();
        let d = chrono::TimeDelta::nanoseconds(
            1 * DAY_NS
                + 2 * HOUR_NS
                + 3 * MINUTE_NS
                + 4 * SECOND_NS
                + 5 * MS_NS
                + 6 * US_NS
                + 7 * NS,
        );
        assert_eq!(actual, d);
    }

    #[test]
    fn format_3() {
        let d = chrono::TimeDelta::nanoseconds(DAY_NS + MINUTE_NS + MS_NS);
        assert_eq!("1d1m1ms", d.as_short_format());
    }

    #[test]
    fn parse_3() {
        let d = chrono::TimeDelta::nanoseconds(DAY_NS + MINUTE_NS + MS_NS);
        let actual = TimeDelta::from_short_format("1d1m1ms").unwrap();
        assert_eq!(actual, d);
    }

    #[test]
    fn format_zero() {
        let d = chrono::TimeDelta::nanoseconds(0);
        assert_eq!("0s", d.as_short_format());
    }

    #[test]
    fn parse_zero() {
        let d = chrono::TimeDelta::nanoseconds(0);
        let actual = TimeDelta::from_short_format("0s").unwrap();
        assert_eq!(actual, d);
    }

    #[test]
    fn format_neg_small() {
        let d = chrono::TimeDelta::nanoseconds(-3 * HOUR_NS);
        assert_eq!("-3h", d.as_short_format());
    }

    #[test]
    fn parse_neg_small() {
        let d = chrono::TimeDelta::nanoseconds(-3 * HOUR_NS);
        let actual = TimeDelta::from_short_format("-3h").unwrap();
        assert_eq!(actual, d);
    }

    #[test]
    fn format_neg_large() {
        let d = chrono::TimeDelta::nanoseconds(
            -(DAY_NS + HOUR_NS + MINUTE_NS + SECOND_NS + MS_NS + US_NS + NS),
        );
        assert_eq!("-1d1h1m1s1ms1us1ns", d.as_short_format());
    }

    #[test]
    fn parse_neg_large() {
        let d = chrono::TimeDelta::nanoseconds(
            -(DAY_NS + HOUR_NS + MINUTE_NS + SECOND_NS + MS_NS + US_NS + NS),
        );
        let actual = TimeDelta::from_short_format("-1d1h1m1s1ms1us1ns").unwrap();
        assert_eq!(actual, d);
    }

    #[test]
    fn fail_on_not_full_match() {
        assert!(TimeDelta::from_short_format("1dxxx").is_err());
    }
}
