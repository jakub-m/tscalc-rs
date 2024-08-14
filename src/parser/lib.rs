use crate::matcher::InputPointer;

use chrono;
use chrono::{Duration, FixedOffset};

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
    DateTime(chrono::DateTime<FixedOffset>),
    Sequence(Vec<Node>),
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
        match pat.captures(pointer.rest().as_ref()) {
            Some(caps) => match captures_to_duration(&caps) {
                Ok(dur) => Ok(ParseOk {
                    pointer: pointer.advance(caps.get(0).unwrap().len()),
                    node: Node::Duration(dur),
                }),
                Err(s) => Err(ParseErr {
                    pointer,
                    message: String::from(s),
                }),
            },
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

pub struct DateTime;

impl Parser for DateTime {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        let pat = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|([+-]\d{2}:\d{2}))$")
            .unwrap();
        let match_ = if let Some(match_) = pat.find(&pointer.rest()) {
            match_.as_str()
        } else {
            return Err(ParseErr {
                pointer,
                message: "not a datetime".to_string(),
            });
        };
        if let Ok(d) = chrono::DateTime::parse_from_rfc3339(match_) {
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

#[derive(Debug)]
struct RepeatedOk<'a> {
    pointer: InputPointer<'a>,
    nodes: Vec<Node>,
}

fn consume_repeated<'a>(
    parser: &'a dyn Parser,
    pointer: InputPointer<'a>,
    error_message: &str,
) -> Result<RepeatedOk<'a>, ParseErr<'a>> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut current_pointer = Some(pointer.clone());
    loop {
        let result = parser.parse(current_pointer.take().unwrap());
        if let Ok(result_ok) = result {
            nodes.push(result_ok.node);
            current_pointer = Some(result_ok.pointer);
        } else {
            current_pointer = Some(result.unwrap_err().pointer);
            break;
        }
    }
    if nodes.is_empty() {
        assert_eq!(
            current_pointer.unwrap(),
            pointer,
            "BUG, nodes are empty but the pointers are different"
        );
        return Err(ParseErr {
            pointer: current_pointer.unwrap(),
            message: String::from(error_message),
        });
    } else {
        assert_ne!(
            current_pointer.unwrap(),
            pointer,
            "BUG, nodes not empty but the pointers are equal"
        );
        return Ok(RepeatedOk {
            nodes,
            pointer: current_pointer.unwrap(),
        });
    }
}

pub struct FirstOf<'a> {
    pub parsers: Vec<&'a dyn Parser>,
}

impl<'p> FirstOf<'p> {
    pub fn new<'a>(parsers: &Vec<&'a dyn Parser>) -> FirstOf<'a> {
        FirstOf {
            parsers: parsers.clone(),
        }
    }
}

impl<'p> Parser for FirstOf<'p> {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        return consume_first(&self.parsers, pointer);
    }
}

fn consume_first<'a, 'p>(
    parsers: &Vec<&'p dyn Parser>,
    pointer: InputPointer<'a>,
) -> Result<ParseOk<'a>, ParseErr<'a>> {
    for i in 0..parsers.len() {
        let parser = parsers.get(i).unwrap();
        if let Ok(result) = parser.parse(pointer) {
            return Ok(result);
        }
    }
    return Err(ParseErr {
        pointer,
        message: "none of the parsers matched".to_string(),
    });
}

mod tests {
    use super::{consume_repeated, DateTime, FirstOf, Node, Parser, SignedDuration, DAY, HOUR};
    use crate::matcher::InputPointer;
    use chrono;
    use chrono::{Duration, FixedOffset, TimeDelta};

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
        let parser = DateTime;
        let s = String::from(input);
        let p = InputPointer::from_string(&s);
        let result = parser.parse(p);
        if let Some(expected) = expected {
            assert!(result.is_ok(), "result not ok: {:?}", result);
            let actual_node = result.unwrap().node;
            let expected = chrono::DateTime::parse_from_rfc3339(expected).unwrap();
            assert_eq!(actual_node, Node::DateTime(expected),);
        } else {
            assert!(result.is_err(), "result not err: {:?}", result);
        }
    }

    #[test]
    fn test_consume_repeated() {
        let input = "1s+2s-3s".to_string();
        let result = consume_repeated(&SignedDuration, InputPointer::from_string(&input), "bla");
        assert!(result.is_ok(), "expected ok, was: {:?}", result);
        let result = result.unwrap();
        let expected_nodes = vec![
            Node::Duration(TimeDelta::seconds(1)),
            Node::Duration(TimeDelta::seconds(2)),
            Node::Duration(TimeDelta::seconds(-3)),
        ];
        assert_eq!(result.nodes, expected_nodes);
    }

    #[test]
    fn test_parse_first_of() {
        let parser = FirstOf::new(&vec![&DateTime, &SignedDuration]);
        let input = String::from("1h");
        let p = InputPointer::from_string(&input);
        let result = parser.parse(p);
        assert!(result.is_ok(), "expected ok, was {:?}", result);
    }
}
