use crate::matcher::InputPointer;

use chrono::{DateTime, Duration, FixedOffset};

/// Example of an expression:
///  2000-01-01T00:00:00Z + 1h
///  now + 1h
///  1h - 1h + now -2h
///
/// Grammar would be:
/// (delta (+- delta)* +-)? date (+- delta)*
use regex::{Captures, Regex};

const SECOND: i64 = 1;
const MINUTE: i64 = SECOND * 60;
const HOUR: i64 = MINUTE * 60;
const DAY: i64 = HOUR * 24;

#[derive(Debug, PartialEq)]
pub enum Node {
    Duration(Duration),
    DateTime(DateTime<FixedOffset>),
}

#[derive(Debug)]
pub struct ParseOk<'a> {
    pub pointer: InputPointer<'a>,
    pub node: Node,
}

#[derive(Debug)]
pub struct ParseErr<'a> {
    pointer: InputPointer<'a>,
    message: String,
}

pub trait Parser {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>>;
}

pub struct SignedDuration;

impl Parser for SignedDuration {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        let pat = Regex::new(r"^([-+])?\s*(\d+)([dhms])").unwrap();
        match pat.captures(pointer.input.as_ref()) {
            Some(caps) => {
                match captures_to_duration(&caps) {
                    Ok(dur) => Ok(ParseOk {
                        pointer, // TODO advance pointer here
                        node: Node::Duration(dur),
                    }),
                    Err(s) => Err(ParseErr {
                        pointer,
                        message: String::from(s),
                    }),
                }
            }
            None => Err(ParseErr {
                pointer,
                message: String::from("did not match any duration"),
            }),
        }
    }
}

fn captures_to_duration(caps: &Captures) -> Result<Duration, String> {
    let sign: i64 = if let Some(sign) = caps.get(1) {
        let sign = sign.as_str();
        if sign == "+" {
            1
        } else if sign == "-" {
            -1
        } else {
            return Err("unknown sign".to_string());
        }
    } else {
        1 // If neither + nor - then assume +.
    };

    let scale: i64 = if let Some(scale) = caps.get(2) {
        if let Ok(scale) = scale.as_str().parse::<u32>() {
            i64::from(scale)
        } else {
            return Err("bad scale".to_string());
        }
    } else {
        return Err("unknown scale".to_string());
    };

    let unit: i64 = if let Some(unit) = caps.get(3) {
        let unit = match unit.as_str() {
            "s" => Ok(SECOND),
            "m" => Ok(MINUTE),
            "h" => Ok(HOUR),
            "d" => Ok(DAY),
            other => Err(format!("bad unit {}", other)),
        };
        if let Ok(unit) = unit {
            unit
        } else {
            return Err(unit.unwrap_err());
        }
    } else {
        return Err("unknown unit".to_string());
    };

    Ok(Duration::seconds(sign * scale * unit))
}

pub struct DateTimeParser;

impl Parser for DateTimeParser {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        let pat = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|([+-]\d{2}:\d{2}))$")
            .unwrap();
        let match_ = if let Some(match_) = pat.find(&pointer.input) {
            match_.as_str()
        } else {
            return Err(ParseErr {
                pointer,
                message: "not a datetime".to_string(),
            });
        };
        if let Ok(d) = DateTime::parse_from_rfc3339(match_) {
            Ok(ParseOk {
                pointer: pointer.advance(match_.len()),
                node: Node::DateTime(d),
            })
        } else {
            return Err(ParseErr {
                pointer,
                message: "bad datetime".to_string(),
            });
        }
    }
}

mod tests {
    use super::{DateTimeParser, Node, Parser, SignedDuration, DAY, HOUR};
    use crate::matcher::InputPointer;
    use chrono::{DateTime, Duration, FixedOffset};

    #[test]
    fn test_parse_signed_duration() {
        check_parse_duration("-123h", Some(-123 * HOUR));
        check_parse_duration("+123h", Some(123 * HOUR));
        check_parse_duration("123h", Some(123 * HOUR));
        check_parse_duration("123d", Some(123 * DAY));
        check_parse_duration("123x", None);
        check_parse_duration("x123d", None);
        check_parse_duration("123", None);
    }

    fn check_parse_duration(input: &str, expected: Option<i64>) {
        let parser = SignedDuration;
        let s = String::from(input);
        let p = InputPointer::from_string(&s);
        let result = parser.parse(p);
        if let Some(seconds) = expected {
            assert!(result.is_ok(), "result not ok: {:?}", result);
            assert_eq!(
                result.unwrap().node,
                Node::Duration(Duration::seconds(seconds))
            );
        } else {
            assert!(result.is_err(), "result not err: {:?}", result);
        }
    }

    #[test]
    fn test_parse_datetime() {
        check_parse_datetime("2000-01-01T00:00:00Z", Some("2000-01-01T00:00:00Z"));
        check_parse_datetime(
            "2000-01-01T00:00:00+00:00",
            Some("2000-01-01T00:00:00+00:00"),
        );
        check_parse_datetime("2000-01-01T00:00:ZZZ", None);
    }

    fn check_parse_datetime(input: &str, expected: Option<&str>) {
        let parser = DateTimeParser;
        let s = String::from(input);
        let p = InputPointer::from_string(&s);
        let result = parser.parse(p);
        if let Some(expected) = expected {
            assert!(result.is_ok(), "result not ok: {:?}", result);
            let actual_node = result.unwrap().node;
            let expected = DateTime::parse_from_rfc3339(expected).unwrap();
            assert_eq!(actual_node, Node::DateTime(expected),);
        } else {
            assert!(result.is_err(), "result not err: {:?}", result);
        }
    }
}
