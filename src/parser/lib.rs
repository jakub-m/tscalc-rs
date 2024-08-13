use crate::matcher::InputPointer;

use chrono::Duration;

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
    Sign(Sign),
    Duration(Duration),
}

#[derive(Debug, PartialEq)]
pub enum Sign {
    Plus,
    Minus,
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

pub fn parse_signed_duration(pointer: InputPointer) -> Result<ParseOk, ParseErr> {
    let pat = Regex::new("(-+)\\s*(\\d+)([dhms])").unwrap();
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

fn captures_to_duration(caps: &Captures) -> Result<Duration, String> {
    let sign = caps.get(1).map(|s| {
        if s.as_str() == "+" {
            Ok(1)
        } else if s.as_str() == "-" {
            Ok(-1)
        } else {
            Err("unknown sign")
        }
    });
    let scale = caps.get(2).map(|s| s.as_str().parse::<u32>());
    let unit = caps.get(3).map(|s| match s.as_str() {
        "s" => Ok(SECOND),
        "m" => Ok(MINUTE),
        "h" => Ok(HOUR),
        "d" => Ok(DAY),
        other => Err(format!("bad unit {}", other)),
    });
    match sign {
        None => Err("no sign".to_string()),
        Some(sign_result) => match sign_result {
            Err(e) => Err(e.to_string()),
            Ok(sign) => match scale {
                None => Err("no scale".to_string()),
                Some(scale_result) => match scale_result {
                    Err(e) => Err(e.to_string()),
                    Ok(scale) => match unit {
                        None => Err("no unit".to_string()),
                        Some(unit) => match unit {
                            Err(e) => Err(e),
                            Ok(unit) => Ok(Duration::seconds(
                                i64::from(sign) * i64::from(scale) * i64::from(unit),
                            )),
                        },
                    },
                },
            },
        },
    }
}

fn parse_optional(p: InputPointer) {
    todo!()
}

fn parse_add_sub(p: InputPointer) {
    todo!()
}

fn parse_date(p: InputPointer) {
    todo!()
}

fn parse_zero_or_many(p: InputPointer) {
    todo!()
}

mod tests {
    use chrono::Duration;

    use crate::{lib::HOUR, matcher::InputPointer};

    use super::{parse_signed_duration, Node};

    #[test]
    fn test_parse_signed_duration() {
        let s = String::from("-123h");
        let p = InputPointer::from_string(&s);
        let result = parse_signed_duration(p);
        assert!(result.is_ok(), "result not ok: {:?}", result);
        assert_eq!(
            result.unwrap().node,
            Node::Duration(Duration::seconds(-123 * HOUR))
        );
    }
}
