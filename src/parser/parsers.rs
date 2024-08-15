use super::core::{InputPointer, Node, ParseErr, ParseOk, Parser};
use chrono;
use regex::{Captures, Regex};

const SECOND: i64 = 1;
const MINUTE: i64 = SECOND * 60;
const HOUR: i64 = MINUTE * 60;
const DAY: i64 = HOUR * 24;

pub fn parse_expr<'a>(input: &'a String) -> Result<ParseOk<'a>, ParseErr<'a>> {
    let pointer = InputPointer::from_string(input);
    ExprParser.parse(pointer)
}

/// Expression grammar is:
///
/// - First of:
///   - date (signed_duration)*
///   - signed_duration (signed_duration)* "+" date (signed_duration)*
struct ExprParser;

impl Parser for ExprParser {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_log(format!("ExprParer {:?}", pointer.rest()));
        let ws = SkipWhitespace;
        let datetime = DateTime;
        let single_duration = SignedDuration;
        let many_durations = ZeroOrMoreDurations;
        let date_durations = Sequence::new(&vec![&ws, &datetime, &ws, &many_durations], |nodes| {
            let nodes = filter_insignificant_nodes(nodes);
            Node::Expr(nodes.to_vec())
        });
        let plus_sign = SkipLiteral::new("+");
        let durations_date_durations = Sequence::new(
            &vec![
                &ws,
                &single_duration,
                &ws,
                &many_durations,
                &ws,
                &plus_sign,
                &ws,
                &datetime,
                &ws,
                &many_durations,
            ],
            |nodes| Node::Expr(filter_insignificant_nodes(nodes).to_vec()),
        );
        let expr_parser = FirstOf::new(vec![&date_durations, &durations_date_durations]);
        expr_parser.parse(pointer)
    }
}

fn filter_insignificant_nodes(nodes: &Vec<Node>) -> Vec<Node> {
    let mut filtered_nodes: Vec<Node> = vec![];

    for node in nodes {
        match node {
            Node::Duration(_) => filtered_nodes.push(node.clone()),
            Node::DateTime(_) => filtered_nodes.push(node.clone()),
            Node::Durations(nodes) => {
                if !nodes.is_empty() {
                    filtered_nodes.push(node.clone())
                }
            }
            Node::Expr(nodes) => {
                if !nodes.is_empty() {
                    filtered_nodes.push(node.clone())
                }
            }
            Node::Skip(_) => (),
        }
    }
    return filtered_nodes;
}

struct SignedDuration;

impl Parser for SignedDuration {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_log(format!("SignedDuration {:?}", pointer.rest()));
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

fn captures_to_duration(caps: &Captures) -> Result<chrono::Duration, String> {
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

    Ok(chrono::Duration::seconds(sign * scale * unit))
}

struct DateTime;

impl Parser for DateTime {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_log(format!("DateTime {:?}", pointer.rest()));
        let pat = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|([+-]\d{2}:\d{2}))")
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

/// Sequence of parsers. All the parsers must match.
struct Sequence<'a> {
    parsers: Vec<&'a dyn Parser>,
    node_fn: fn(&Vec<Node>) -> Node,
}

impl<'a> Sequence<'a> {
    fn new(parsers: &Vec<&'a dyn Parser>, node_fn: fn(&Vec<Node>) -> Node) -> Sequence<'a> {
        Sequence {
            parsers: parsers.clone(),
            node_fn,
        }
    }
}

impl<'p> Parser for Sequence<'p> {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_log(format!("Sequence {:?}", pointer.rest()));
        let result = consume_sequence(&self.parsers, pointer);
        if let Ok(result) = result {
            let result_node = (self.node_fn)(&result.nodes);
            return Ok(ParseOk {
                pointer: result.pointer,
                node: result_node,
            });
        } else {
            return Err(result.unwrap_err());
        }
    }
}

#[derive(Debug)]
struct RepeatedOk<'a> {
    pointer: InputPointer<'a>,
    nodes: Vec<Node>,
}

struct ZeroOrMoreDurations;

impl Parser for ZeroOrMoreDurations {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_log(format!("ZeroOrModeDurations {:?}", pointer.rest()));
        let parser = LTrim(&SignedDuration);
        let result = consume_repeated(
            &parser,
            pointer,
            ConsumeRepeated::ZeroOrMore,
            "failed to match durations",
        );
        let mut nodes = Vec::new();
        if let Ok(result) = result {
            for node in result.nodes {
                if let Node::Duration(delta) = node {
                    nodes.push(Node::Duration(delta));
                } else {
                    panic!("Expected duration node but got: {:?}", node);
                }
            }
            return Ok(ParseOk {
                pointer: result.pointer,
                node: Node::Durations(nodes),
            });
        } else {
            return Err(result.unwrap_err());
        }
    }
}

/// Trim whitespace on the left input of the parser.
struct LTrim<'a>(&'a dyn Parser);

impl<'p> Parser for LTrim<'p> {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_log(format!("LTrim {:?}", pointer.rest()));
        let current_pointer = match pointer.rest().find(|c: char| !c.is_whitespace()) {
            Some(offset) => pointer.advance(offset),
            None => pointer,
        };
        self.0.parse(current_pointer)
    }
}

enum ConsumeRepeated {
    ZeroOrMore,
    OneOrMore,
}

fn consume_repeated<'a, 'p>(
    parser: &'p dyn Parser,
    pointer: InputPointer<'a>,
    zero_config: ConsumeRepeated,
    error_message: &str,
) -> Result<RepeatedOk<'a>, ParseErr<'a>> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut current_pointer = Some(pointer);
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
        // Not true if LTrim is used.
        //assert_eq!(
        //    current_pointer.unwrap(),
        //    pointer,
        //    "BUG, nodes are empty but the pointers are different"
        //);
        return match zero_config {
            ConsumeRepeated::ZeroOrMore => Ok(RepeatedOk {
                pointer: current_pointer.unwrap(),
                nodes: vec![],
            }),
            ConsumeRepeated::OneOrMore => Err(ParseErr {
                pointer: current_pointer.unwrap(),
                message: String::from(error_message),
            }),
        };
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

struct FirstOf<'a> {
    parsers: Vec<&'a dyn Parser>,
}

impl<'p> FirstOf<'p> {
    fn new<'a>(parsers: Vec<&'a dyn Parser>) -> FirstOf<'a> {
        FirstOf { parsers: parsers }
    }
}

impl<'p> Parser for FirstOf<'p> {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_log(format!("FirstOf {:?}", pointer.rest()));
        return consume_first(&self.parsers, pointer);
    }
}

/// Try the parsers one after one and return the result of the first one matching.
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

#[derive(Debug)]
struct SequenceOk<'a> {
    nodes: Vec<Node>,
    pointer: InputPointer<'a>,
}

/// Succeed only if all the parses succeed one after another.
fn consume_sequence<'a, 'p>(
    parsers: &Vec<&'p dyn Parser>,
    pointer: InputPointer<'a>,
) -> Result<SequenceOk<'a>, ParseErr<'a>> {
    let mut nodes: Vec<Node> = vec![];
    let mut current_pointer = Some(pointer);
    for i in 0..parsers.len() {
        let parser = parsers.get(i).unwrap();
        let result = parser.parse(current_pointer.take().unwrap());
        if let Ok(result_ok) = result {
            nodes.push(result_ok.node);
            current_pointer = Some(result_ok.pointer);
        } else {
            return Err(result.unwrap_err());
        }
    }
    Ok(SequenceOk {
        nodes,
        pointer: current_pointer.take().unwrap(),
    })
}

struct SkipLiteral(String);

impl SkipLiteral {
    fn new(literal: &str) -> SkipLiteral {
        SkipLiteral(literal.to_string())
    }
}

impl Parser for SkipLiteral {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_log(format!("SkipLiteral {:?}", pointer.rest()));
        if pointer.rest().starts_with(&self.0) {
            let pointer = pointer.advance(self.0.len());
            return Ok(ParseOk {
                pointer,
                node: Node::Skip(self.0.to_string()),
            });
        } else {
            return Err(ParseErr {
                pointer,
                message: format!("expected {}", self.0),
            });
        }
    }
}

struct SkipWhitespace;

impl Parser for SkipWhitespace {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_log(format!("SkipWhitespace {:?}", pointer.rest()));
        let mut offset = pointer.rest().len();
        for (char_pos, c) in pointer.rest().char_indices() {
            if c != ' ' {
                offset = char_pos;
                break;
            }
        }
        Ok(ParseOk {
            pointer: pointer.advance(offset),
            node: Node::Skip(" ".to_string()),
        })
    }
}

fn debug_log(_s: String) {
    // println!("{}", s);
}

mod tests {
    use core::hash;
    use std::os::unix::fs::chroot;

    use super::{
        consume_repeated, consume_sequence, ConsumeRepeated, DateTime, ExprParser, FirstOf,
        InputPointer, Node, Parser, SignedDuration, DAY, HOUR,
    };
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
        let result = consume_repeated(
            &SignedDuration,
            InputPointer::from_string(&input),
            ConsumeRepeated::OneOrMore,
            "bla",
        );
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
        let parser = FirstOf::new(vec![&SignedDuration, &DateTime]);
        let input = String::from("1s + bla");
        let p = InputPointer::from_string(&input);
        let result = parser.parse(p);
        assert!(result.is_ok(), "expected ok, was {:?}", result);
        assert_eq!(result.unwrap().node, Node::Duration(Duration::seconds(1)));
    }

    #[test]
    fn test_consume_sequence() {
        let input = "1s+2s+3s".to_string();
        let p = InputPointer::from_string(&input);
        let parsers: Vec<&dyn Parser> = vec![&SignedDuration, &SignedDuration];
        let result = consume_sequence(&parsers, p);
        assert!(result.is_ok(), "expected ok, got {:?}", result);
        assert_eq!(
            result.unwrap().nodes,
            vec![
                Node::Duration(Duration::seconds(1)),
                Node::Duration(Duration::seconds(2)),
            ]
        );
    }

    #[test]
    fn test_expr_parser() {
        let datetime_node =
            Node::DateTime(chrono::DateTime::parse_from_rfc3339("2000-01-01T00:00:00Z").unwrap());
        let duration_1s_node = Node::Duration(chrono::TimeDelta::seconds(1));
        let duration_2s_node = Node::Duration(chrono::TimeDelta::seconds(2));
        let duration_3s_node = Node::Duration(chrono::TimeDelta::seconds(3));
        check_expr_parser(
            "2000-01-01T00:00:00Z",
            Some(Node::Expr(vec![datetime_node.clone()])),
        );
        check_expr_parser(
            " 2000-01-01T00:00:00Z",
            Some(Node::Expr(vec![datetime_node.clone()])),
        );
        check_expr_parser(
            "2000-01-01T00:00:00Z+1s",
            Some(Node::Expr(vec![
                datetime_node.clone(),
                Node::Durations(vec![duration_1s_node.clone()]),
            ])),
        );
        check_expr_parser(
            "2000-01-01T00:00:00Z+1s+2s",
            Some(Node::Expr(vec![
                datetime_node.clone(),
                Node::Durations(vec![duration_1s_node.clone(), duration_2s_node.clone()]),
            ])),
        );
        check_expr_parser(
            "1s+2000-01-01T00:00:00Z",
            Some(Node::Expr(vec![
                duration_1s_node.clone(),
                datetime_node.clone(),
            ])),
        );
        check_expr_parser(
            " 1s + 2000-01-01T00:00:00Z ",
            Some(Node::Expr(vec![
                duration_1s_node.clone(),
                datetime_node.clone(),
            ])),
        );
        check_expr_parser(
            "1s+2s+3s+2000-01-01T00:00:00Z+1s+2s+3s",
            Some(Node::Expr(vec![
                duration_1s_node.clone(),
                Node::Durations(vec![duration_2s_node.clone(), duration_3s_node.clone()]),
                datetime_node.clone(),
                Node::Durations(vec![
                    duration_1s_node.clone(),
                    duration_2s_node.clone(),
                    duration_3s_node.clone(),
                ]),
            ])),
        );
        check_expr_parser(
            //" 1s + 2s +  3s +  2000-01-01T00:00:00Z +  1s +  2s + 3s ",
            "1s+2s+3s+2000-01-01T00:00:00Z + 1s + 2s + 3s",
            Some(Node::Expr(vec![
                duration_1s_node.clone(),
                Node::Durations(vec![duration_2s_node.clone(), duration_3s_node.clone()]),
                datetime_node.clone(),
                Node::Durations(vec![
                    duration_1s_node.clone(),
                    duration_2s_node.clone(),
                    duration_3s_node.clone(),
                ]),
            ])),
        );
    }

    fn check_expr_parser(input: &str, expected: Option<Node>) {
        let parser = ExprParser;
        let input = input.to_string();
        let pointer = InputPointer::from_string(&input);
        let result = parser.parse(pointer);
        if let Some(expected) = expected {
            assert!(result.is_ok(), "expected ok got {:?}", result);
            assert_eq!(result.unwrap().node, expected, "input: {}", input)
        } else {
            assert!(!result.is_ok(), "expected not ok got {:?}", result);
        }
    }
}
