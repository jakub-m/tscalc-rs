use super::{
    core::{InputPointer, Node, Oper, ParseErr, ParseOk, Parser},
    match_duration, DisplayParseResult, ShortFormat,
};
use crate::log::debug_nested_log;
use chrono::{self, TimeDelta};
use regex::{Captures, Regex};
use std::rc::Rc;

pub fn parse_expr<'a>(input: &'a String) -> Result<ParseOk<'a>, ParseErr<'a>> {
    let pointer = InputPointer::from_string(input);
    let result = ExprParser.parse(pointer, 0);
    result.map(|parse_ok| {
        if parse_ok.pointer.is_end() {
            Ok(parse_ok)
        } else {
            Err(ParseErr {
                pointer: parse_ok.pointer,
                message: "not all input matched".to_string(),
            })
        }
    })?
}

/// Expression grammar is:
///  (sighed_duration | date) (signed_duration | signed_date)*
/// Validity of the expression is figured during evaluation.
struct ExprParser;

impl Parser for ExprParser {
    fn parse<'a>(
        &self,
        pointer: InputPointer<'a>,
        nesting: usize,
    ) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_nested_log(nesting, format!("ExprParer input={}", pointer));
        let expr = ExprParser;
        let ws0 = Whitespace::new_optional();
        let ws1 = Whitespace::new_must_have();
        let now = LiteralNode::new("now", Node::Now);
        let datetime = DateTime;
        let timestamp = Timestamp;
        //let datetime_or_now = FirstOf::new(vec![&datetime, &timestamp, &now]);
        let signed_duration = SignedDuration;
        let sign = Literal::new_any(&["+", "-"]).set_skip();
        let left_bracket = Literal::new("(").set_skip();
        let right_bracket = Literal::new(")").set_skip();
        let bracket_expr =
            Sequence::new_as_expr(&vec![&left_bracket, &ws0, &expr, &ws0, &right_bracket]);
        // The function names are hardcoded in the parser.
        let func_ary1_literals = Literal::new_any(&["full_day", "full_hour"]);
        let func_ary1 = Sequence::new(
            &vec![&func_ary1_literals, &left_bracket, &expr, &right_bracket],
            |nodes| sequence_to_func_ary1(nodes),
        );
        // A "term" is datetime or now or duration or function call or expression in brackets.
        let term = FirstOf::new(vec![
            //&datetime_or_now,
            &datetime,
            &now,
            &signed_duration,
            &timestamp, // timestamp is after signed duration, otherwise 1s would be matched as "1" being timestamp and "s" possibly and causing error.
            &func_ary1,
            &bracket_expr,
        ]);
        let oper_term = Sequence::new(&vec![&ws1, &sign, &ws1, &term], |nodes| {
            nodes_to_oper_expr(nodes)
        });
        let repeated_terms = RepeatedAsExpr(&oper_term);

        // list of terms that are either added or subtracted
        let list_of_terms = Sequence::new_as_expr(&vec![&ws0, &term, &repeated_terms, &ws0]);
        list_of_terms.parse(pointer, nesting + 1)
    }
}

/// Convert a parsed sequence to function call. The order and set of the nodes is well-determined by the parser.
fn sequence_to_func_ary1(nodes: &[Node]) -> Node {
    let nodes = filter_insignificant_nodes(nodes);
    if nodes.len() != 2 {
        panic!("expected exactly two nodes got {:?}", nodes);
    }
    let name = if let Node::Literal { literal, skip: _ } = nodes.get(0).unwrap() {
        literal.to_owned()
    } else {
        panic!(
            "expected the first node to be literal with func name, got {:?}",
            nodes
        );
    };
    let arg1 = nodes.get(1).unwrap().to_owned();
    Node::FuncAry1 {
        name,
        arg1: Rc::new(arg1),
    }
}

fn nodes_to_oper_expr(nodes: &Vec<Node>) -> Node {
    let oper = nodes.iter().find_map(|node| {
        if let Node::Literal { literal, skip: _ } = node {
            return match literal.as_str() {
                "+" => Some(Oper::Plus),
                "-" => Some(Oper::Minus),
                _ => None,
            };
        }
        return None;
    });
    let oper = oper.expect(
        format!(
            "BUG! Expected operator at input to nodes_to_oper_expr, got {:?}",
            nodes
        )
        .as_str(),
    );
    let nodes = filter_insignificant_nodes(nodes);
    if nodes.len() != 1 {
        panic!(
            "BUG! There must be exactly one node for nodes_to_oper_expr, was: {:?}",
            nodes
        )
    }
    Node::OperNode {
        oper,
        node: Rc::new(nodes.get(0).unwrap().clone()),
    }
}

fn filter_insignificant_nodes(nodes: &[Node]) -> Vec<Node> {
    let mut filtered_nodes: Vec<Node> = vec![];
    for node in nodes {
        match node {
            Node::Duration(_)
            | Node::DateTime(_)
            | Node::Now
            | Node::FuncAry1 { name: _, arg1: _ }
            | Node::OperNode { oper: _, node: _ } => filtered_nodes.push(node.clone()),
            Node::Expr(nodes) => {
                if !nodes.is_empty() {
                    filtered_nodes.push(node.clone())
                }
            }
            Node::Literal { literal: _, skip } => {
                if !skip {
                    filtered_nodes.push(node.clone())
                }
            }
        }
    }
    return filtered_nodes;
}

struct SignedDuration;

impl Parser for SignedDuration {
    fn parse<'a>(
        &self,
        pointer: InputPointer<'a>,
        nesting: usize,
    ) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_nested_log(nesting, format!("SignedDuration input={}", pointer));

        match match_duration(pointer.rest()) {
            Some(matched) => {
                let duration = TimeDelta::from_short_format(matched)
                    .expect("failed to parse previously matched timedelta");
                Ok(ParseOk {
                    pointer: pointer.advance(matched.len()),
                    node: Node::Duration(duration),
                })
            }
            None => Err(ParseErr {
                pointer,
                message: String::from("did not match any duration"),
            }),
        }
    }
}

/// Datetime as epoch-timestamp (seconds).
struct Timestamp;

impl Parser for Timestamp {
    fn parse<'a>(
        &self,
        pointer: InputPointer<'a>,
        nesting: usize,
    ) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_nested_log(nesting, format!("Timestamp input={}", pointer));
        let pat = Regex::new(r"^(-?\d+)(\.(\d+))?").unwrap();
        let (match_len, secs_str, nsecs_str) = if let Some(captures) = pat.captures(&pointer.rest())
        {
            (
                captures.get(0).unwrap().len(),
                captures.get(1).unwrap().as_str(),
                captures.get(3).map_or("0", |v| v.as_str()),
            )
        } else {
            return Err(ParseErr {
                pointer,
                message: "not a timestamp".to_string(),
            });
        };
        let unix_secs = secs_str.parse::<i64>().unwrap();
        let nsecs_str = format!("{:0<9}", nsecs_str);
        let unix_nsecs = nsecs_str.parse::<u32>().unwrap();
        debug_nested_log(
            nesting,
            format!("Timestamp parsed secs={} nsecs={}", unix_secs, unix_nsecs),
        );

        match chrono::DateTime::from_timestamp(unix_secs, unix_nsecs) {
            Some(d) => Ok(ParseOk {
                pointer: pointer.advance(match_len),
                node: Node::DateTime(d.into()),
            }),
            None => Err(ParseErr {
                pointer,
                message: format!("bad datetime for {:?} {:?}", unix_secs, unix_nsecs),
            }),
        }
    }
}

struct DateTime;

impl Parser for DateTime {
    fn parse<'a>(
        &self,
        pointer: InputPointer<'a>,
        nesting: usize,
    ) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_nested_log(nesting, format!("DateTime input={}", pointer));
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
            return Ok(ParseOk {
                pointer: pointer.advance(match_.len()),
                node: Node::DateTime(d),
            });
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
    /// Return sequence as Expr node.
    fn new_as_expr(parsers: &Vec<&'a dyn Parser>) -> Sequence<'a> {
        Sequence::new(parsers, |nodes| {
            Node::Expr(filter_insignificant_nodes(nodes).to_vec())
        })
    }

    fn new(parsers: &Vec<&'a dyn Parser>, node_fn: fn(&Vec<Node>) -> Node) -> Sequence<'a> {
        Sequence {
            parsers: parsers.clone(),
            node_fn,
        }
    }
}

impl<'p> Parser for Sequence<'p> {
    fn parse<'a>(
        &self,
        pointer: InputPointer<'a>,
        nesting: usize,
    ) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_nested_log(nesting, format!("Sequence input={}", pointer));
        let result = consume_sequence(&self.parsers, pointer, nesting + 1);
        result.map(|result| {
            let result_node = (self.node_fn)(&result.nodes);
            Ok(ParseOk {
                pointer: result.pointer,
                node: result_node,
            })
        })?
    }
}

#[derive(Debug)]
struct RepeatedOk<'a> {
    pointer: InputPointer<'a>,
    nodes: Vec<Node>,
}

struct RepeatedAsExpr<'p>(&'p dyn Parser);

impl<'p> Parser for RepeatedAsExpr<'p> {
    fn parse<'a>(
        &self,
        pointer: InputPointer<'a>,
        nesting: usize,
    ) -> Result<ParseOk<'a>, ParseErr<'a>> {
        consume_repeated(
            self.0,
            pointer,
            ConsumeRepeated::ZeroOrMore,
            nesting + 1,
            "failed to match repeated",
        )
        .map(|repeated_ok| {
            Ok(ParseOk {
                pointer: repeated_ok.pointer,
                node: Node::Expr(repeated_ok.nodes),
            })
        })?
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
    nesting: usize,
    error_message: &str,
) -> Result<RepeatedOk<'a>, ParseErr<'a>> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut current_pointer = Some(pointer);
    loop {
        let result = parser.parse(current_pointer.take().unwrap(), nesting + 1);
        debug_nested_log(
            nesting,
            format!("consume_repeated result {}", result.to_string()),
        );
        if let Ok(result_ok) = result {
            nodes.push(result_ok.node);
            current_pointer = Some(result_ok.pointer);
        } else {
            current_pointer = Some(result.unwrap_err().pointer);
            break;
        }
    }
    if nodes.is_empty() {
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
    fn parse<'a>(
        &self,
        pointer: InputPointer<'a>,
        nesting: usize,
    ) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_nested_log(nesting, format!("FirstOf input={}", pointer));
        return consume_first(&self.parsers, pointer, nesting + 1);
    }
}

/// Try the parsers one after one and return the result of the first one matching.
fn consume_first<'a, 'p>(
    parsers: &Vec<&'p dyn Parser>,
    pointer: InputPointer<'a>,
    nesting: usize,
) -> Result<ParseOk<'a>, ParseErr<'a>> {
    let mut furthest_err_pointer = None;
    for i in 0..parsers.len() {
        let parser = parsers.get(i).unwrap();
        let result = parser.parse(pointer, nesting + 1);
        debug_nested_log(
            nesting,
            format!("consume_first result {}", result.to_string()),
        );
        match result {
            Ok(parse_ok) => return Ok(parse_ok),
            Err(parse_err) => {
                if furthest_err_pointer.is_none() {
                    furthest_err_pointer = Some(parse_err.pointer)
                } else {
                    // If all the parsers fail, as an error reason return the error that advanced the most in the parsing.
                    let curr_err_pointer = furthest_err_pointer.take().unwrap();
                    if parse_err.pointer.pos > curr_err_pointer.pos {
                        furthest_err_pointer = Some(parse_err.pointer)
                    } else {
                        furthest_err_pointer = Some(curr_err_pointer)
                    }
                }
            }
        }
    }
    return Err(ParseErr {
        pointer: furthest_err_pointer.unwrap(),
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
    nesting: usize,
) -> Result<SequenceOk<'a>, ParseErr<'a>> {
    debug_nested_log(nesting, format!("consume_sequence input {}", pointer));
    let mut nodes: Vec<Node> = vec![];
    let mut current_pointer = Some(pointer);
    for i in 0..parsers.len() {
        let parser = parsers.get(i).unwrap();
        let result = parser.parse(current_pointer.take().unwrap(), nesting + 1);
        debug_nested_log(
            nesting,
            format!(
                "consume_sequence result [{}/{}] {}",
                i + 1,
                parsers.len(),
                result.to_string()
            ),
        );
        match result {
            Ok(parse_ok) => {
                nodes.push(parse_ok.node);
                current_pointer = Some(parse_ok.pointer);
            }
            Err(parse_err) => {
                return Err(ParseErr {
                    pointer, // Pass the original pointer so when the sequence fails, pointer does not move.
                    message: parse_err.message,
                });
            }
        }
    }
    let pointer = current_pointer.take().unwrap();
    debug_nested_log(
        nesting,
        format!("consume_sequence ok, nodes={:?}, output={}", nodes, pointer,),
    );
    Ok(SequenceOk { nodes, pointer })
}

/// Match any of the literal strings.
struct Literal {
    literals: Vec<String>,
    skip: bool,
}

impl Literal {
    fn new(literal: &str) -> Literal {
        Literal {
            literals: vec![literal.to_string()],
            skip: false,
        }
    }

    fn new_any(literals: &[&str]) -> Literal {
        let literals: Vec<String> = literals.iter().map(|s| s.to_string()).collect();
        Literal {
            literals,
            skip: false,
        }
    }

    fn set_skip(self) -> Literal {
        Literal { skip: true, ..self }
    }

    fn skip(&self) -> bool {
        self.skip
    }
}

impl Parser for Literal {
    fn parse<'a>(
        &self,
        pointer: InputPointer<'a>,
        nesting: usize,
    ) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_nested_log(
            nesting,
            format!("Literal {:?} input={}", self.literals, pointer),
        );
        for literal in &self.literals {
            if pointer.rest().starts_with(literal) {
                let pointer = pointer.advance(literal.len());
                return Ok(ParseOk {
                    pointer,
                    node: Node::Literal {
                        literal: literal.to_owned(),
                        skip: self.skip,
                    },
                });
            }
        }
        return Err(ParseErr {
            pointer,
            message: format!("expected {:?}", self.literals),
        });
    }
}

struct Whitespace {
    optional: bool,
}

impl Whitespace {
    pub fn new_must_have() -> Whitespace {
        Whitespace { optional: false }
    }

    pub fn new_optional() -> Whitespace {
        Whitespace { optional: true }
    }
}

impl Parser for Whitespace {
    fn parse<'a>(
        &self,
        pointer: InputPointer<'a>,
        nesting: usize,
    ) -> Result<ParseOk<'a>, ParseErr<'a>> {
        debug_nested_log(nesting, format!("Whitespace input={}", pointer));
        // Set offset to len() at start in case all the remainder of the input is whitespace.
        let mut offset = pointer.rest().len();
        let mut matched = false;
        for (char_pos, c) in pointer.rest().char_indices() {
            if c != ' ' {
                offset = char_pos;
                break;
            }
            matched = true;
        }
        if self.optional || matched {
            Ok(ParseOk {
                pointer: pointer.advance(offset),
                node: Node::Literal {
                    literal: " ".to_string(),
                    skip: true,
                },
            })
        } else {
            Err(ParseErr {
                pointer,
                message: "whitespace not matched".to_string(),
            })
        }
    }
}

struct LiteralNode {
    /// Literal to match.
    literal: String,
    /// Node to return.
    node: Node,
}
impl LiteralNode {
    fn new(literal: &str, node: Node) -> LiteralNode {
        LiteralNode {
            literal: literal.to_string(),
            node: node.clone(),
        }
    }
}

impl Parser for LiteralNode {
    fn parse<'a>(
        &self,
        pointer: InputPointer<'a>,
        nesting: usize,
    ) -> Result<ParseOk<'a>, ParseErr<'a>> {
        if pointer.rest().starts_with(&self.literal) {
            Ok(ParseOk {
                pointer: pointer.advance(self.literal.len()),
                node: self.node.clone(),
            })
        } else {
            Err(ParseErr {
                pointer,
                message: format!("expected literal {:?}", self.literal),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        consume_repeated, consume_sequence, ConsumeRepeated, DateTime, ExprParser, FirstOf,
        InputPointer, Node, Oper, Parser, SignedDuration,
    };
    use crate::parser::parsers::Literal;
    use crate::parser::{DAY_NS, HOUR_NS, SECOND_NS};
    use chrono;
    use chrono::{Duration, TimeDelta};
    use std::rc::Rc;

    #[test]
    fn test_parse_signed_duration() {
        check_parse_duration("-123h", Some(-123 * HOUR_NS));
        check_parse_duration("123h", Some(123 * HOUR_NS));
        check_parse_duration("123d", Some(123 * DAY_NS));
        check_parse_duration("123x", None);
        check_parse_duration("x123d", None);
        check_parse_duration("123", None);
        check_parse_duration("-1d2s3ns", Some(-(DAY_NS + 2 * SECOND_NS + 3)));
    }

    fn check_parse_duration(input: &str, expected_ns: Option<i64>) {
        let parser = SignedDuration;
        let s = String::from(input);
        let p = InputPointer::from_string(&s);
        let result = parser.parse(p, 0);
        if let Some(ns) = expected_ns {
            assert!(result.is_ok(), "result not ok: {:?}", result);
            assert_eq!(
                result.unwrap().node,
                Node::Duration(Duration::nanoseconds(ns))
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
        let result = parser.parse(p, 0);
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
    fn test_consume_repeated_1() {
        let input = "1s2s3s".to_string();
        let result = consume_repeated(
            &SignedDuration,
            InputPointer::from_string(&input),
            ConsumeRepeated::OneOrMore,
            0,
            "bla",
        );
        assert!(result.is_ok(), "expected ok, was: {:?}", result);
        let result = result.unwrap();
        let expected_nodes = vec![
            Node::Duration(TimeDelta::seconds(1)),
            Node::Duration(TimeDelta::seconds(2)),
            Node::Duration(TimeDelta::seconds(3)),
        ];
        assert_eq!(result.nodes, expected_nodes);
        assert_eq!(result.pointer.rest(), "");
    }

    #[test]
    fn test_consume_repeated_2() {
        let input = "1s2sxx".to_string();
        let result = consume_repeated(
            &SignedDuration,
            InputPointer::from_string(&input),
            ConsumeRepeated::OneOrMore,
            0,
            "bla",
        );
        assert!(result.is_ok(), "expected ok, was: {:?}", result);
        let result = result.unwrap();
        let expected_nodes = vec![
            Node::Duration(TimeDelta::seconds(1)),
            Node::Duration(TimeDelta::seconds(2)),
        ];
        assert_eq!(result.nodes, expected_nodes);
        assert_eq!(result.pointer.rest(), "xx");
    }

    #[test]
    fn test_parse_first_of() {
        let parser = FirstOf::new(vec![&SignedDuration, &DateTime]);
        let input = String::from("1s + bla");
        let p = InputPointer::from_string(&input);
        let result = parser.parse(p, 0);
        assert!(result.is_ok(), "expected ok, was {:?}", result);
        assert_eq!(result.unwrap().node, Node::Duration(Duration::seconds(1)));
    }

    #[test]
    fn test_consume_sequence_1() {
        let input = "1s+2s+3s".to_string();
        let p = InputPointer::from_string(&input);
        let plus = Literal::new("+");
        let parsers: Vec<&dyn Parser> = vec![&SignedDuration, &plus, &SignedDuration];
        let result = consume_sequence(&parsers, p, 0);
        let result = result.expect("expected ok");
        assert_eq!(
            result.nodes,
            vec![
                Node::Duration(Duration::seconds(1)),
                Node::Literal {
                    literal: "+".to_string(),
                    skip: false
                },
                Node::Duration(Duration::seconds(2)),
            ]
        );
        assert_eq!(result.pointer.rest(), "+3s");
    }

    #[test]
    fn test_consume_sequence_2() {
        let input = "1s-2s-3s".to_string();
        let p = InputPointer::from_string(&input);
        let plus = Literal::new("+");
        let parsers: Vec<&dyn Parser> = vec![&SignedDuration, &plus, &SignedDuration];
        let result = consume_sequence(&parsers, p, 0);
        let result = result.expect_err("expected err");
        assert_eq!(result.pointer.rest(), "1s-2s-3s");
    }

    #[test]
    fn test_expr_parser_1() {
        check_expr_parser(
            "2000-01-01T00:00:00Z",
            Some(Node::Expr(vec![datetime_node()])),
        );
    }

    #[test]
    fn test_expr_parser_2() {
        check_expr_parser(
            " 2000-01-01T00:00:00Z",
            Some(Node::Expr(vec![datetime_node()])),
        );
    }

    #[test]
    fn test_expr_parser_3a() {
        check_expr_parser("2000-01-01T00:00:00Z+1s", None);
    }
    #[test]
    fn test_expr_parser_3b() {
        check_expr_parser(
            "2000-01-01T00:00:00Z + 1s",
            Some(Node::Expr(vec![
                datetime_node(),
                Node::Expr(vec![Node::OperNode {
                    oper: Oper::Plus,
                    node: Rc::new(duration_1s_node()),
                }]),
            ])),
        );
    }

    #[test]
    fn test_expr_parser_4() {
        check_expr_parser(
            "2000-01-01T00:00:00Z + 1s + 2s",
            Some(Node::Expr(vec![
                datetime_node(),
                Node::Expr(vec![
                    Node::OperNode {
                        oper: Oper::Plus,
                        node: Rc::new(duration_1s_node()),
                    },
                    Node::OperNode {
                        oper: Oper::Plus,
                        node: Rc::new(duration_2s_node()),
                    },
                ]),
            ])),
        );
    }

    #[test]
    fn test_expr_parser_5a() {
        check_expr_parser("1s+2000-01-01T00:00:00Z", None);
    }

    #[test]
    fn test_expr_parser_5b() {
        check_expr_parser(
            "1s + 2000-01-01T00:00:00Z",
            Some(Node::Expr(vec![
                duration_1s_node(),
                Node::Expr(vec![Node::OperNode {
                    oper: Oper::Plus,
                    node: Rc::new(datetime_node()),
                }]),
            ])),
        );
    }

    #[test]
    fn test_expr_parser_6() {
        check_expr_parser(
            " 1s + 2000-01-01T00:00:00Z ",
            Some(Node::Expr(vec![
                duration_1s_node(),
                Node::Expr(vec![Node::OperNode {
                    oper: Oper::Plus,
                    node: Rc::new(datetime_node()),
                }]),
            ])),
        );
    }

    #[test]
    fn test_expr_parser_7() {
        check_expr_parser(
            "1s + 2s + 3s - 2000-01-01T00:00:00Z - 1s + 2s + 3s",
            Some(Node::Expr(vec![
                duration_1s_node(),
                Node::Expr(vec![
                    Node::OperNode {
                        oper: Oper::Plus,
                        node: Rc::new(duration_2s_node()),
                    },
                    Node::OperNode {
                        oper: Oper::Plus,
                        node: Rc::new(duration_3s_node()),
                    },
                    Node::OperNode {
                        oper: Oper::Minus,
                        node: Rc::new(datetime_node()),
                    },
                    Node::OperNode {
                        oper: Oper::Minus,
                        node: Rc::new(duration_1s_node()),
                    },
                    Node::OperNode {
                        oper: Oper::Plus,
                        node: Rc::new(duration_2s_node()),
                    },
                    Node::OperNode {
                        oper: Oper::Plus,
                        node: Rc::new(duration_3s_node()),
                    },
                ]),
            ])),
        );
    }

    #[test]
    fn test_subtract_date_1() {
        check_expr_parser(
            "2000-01-01T00:00:00Z - 2000-01-01T00:00:00Z",
            Some(Node::Expr(vec![
                datetime_node(),
                Node::Expr(vec![Node::OperNode {
                    oper: Oper::Minus,
                    node: Rc::new(datetime_node()),
                }]),
            ])),
        )
    }

    #[test]
    fn test_subtract_date_2() {
        check_expr_parser(
            "2000-01-01T00:00:00Z + 1s - 2000-01-01T00:00:00Z + 2000-01-01T00:00:00Z",
            Some(Node::Expr(vec![
                datetime_node(),
                Node::Expr(vec![
                    Node::OperNode {
                        oper: Oper::Plus,
                        node: Rc::new(duration_1s_node()),
                    },
                    Node::OperNode {
                        oper: Oper::Minus,
                        node: Rc::new(datetime_node()),
                    },
                    Node::OperNode {
                        oper: Oper::Plus,
                        node: Rc::new(datetime_node()),
                    },
                ]),
            ])),
        )
    }

    #[test]
    fn test_parse_brackets_1() {
        check_expr_parser(
            "1s - (2s + 3s) - (1s + 2s)",
            Some(Node::Expr(vec![
                duration_1s_node(),
                Node::Expr(vec![
                    Node::OperNode {
                        oper: Oper::Minus,
                        node: Rc::new(Node::Expr(vec![Node::Expr(vec![
                            duration_2s_node(),
                            Node::Expr(vec![Node::OperNode {
                                oper: Oper::Plus,
                                node: Rc::new(duration_3s_node()),
                            }]),
                        ])])),
                    },
                    Node::OperNode {
                        oper: Oper::Minus,
                        node: Rc::new(Node::Expr(vec![Node::Expr(vec![
                            duration_1s_node(),
                            Node::Expr(vec![Node::OperNode {
                                oper: Oper::Plus,
                                node: Rc::new(duration_2s_node()),
                            }]),
                        ])])),
                    },
                ]),
            ])),
        );
    }

    #[test]
    fn test_func_call_1() {
        check_expr_parser(
            "full_day(now)",
            Some(Node::Expr(vec![Node::FuncAry1 {
                name: "full_day".to_string(),
                arg1: Rc::new(Node::Expr(vec![Node::Now])),
            }])),
        );
    }

    #[test]
    fn test_timestamp_1() {
        check_expr_parser("946684800.000", Some(Node::Expr(vec![datetime_node()])));
    }

    #[test]
    fn test_parse_missing_bracket_1() {
        check_expr_parser("0.0 + (0.0 - 1.0", None);
    }

    fn check_expr_parser(input: &str, expected: Option<Node>) {
        let parser = ExprParser;
        let input = input.to_string();
        let pointer = InputPointer::from_string(&input);
        let result = parser.parse(pointer, 0);
        if let Some(expected) = expected {
            let parse_ok = if let Ok(parse_ok) = result {
                parse_ok
            } else {
                panic!("parser failed: {:?}", result)
            };

            assert!(
                parse_ok.pointer.is_end(),
                "expected parser to parse to the end, rest={}",
                parse_ok.pointer
            );
            assert_eq!(parse_ok.node, expected, "input: {}", input)
        } else {
            assert!(
                !(pointer.is_end() && result.is_ok()),
                "expected not a full match or not ok result={:?}",
                result
            );
        }
    }

    fn duration_1s_node() -> Node {
        Node::Duration(chrono::TimeDelta::seconds(1))
    }

    fn duration_2s_node() -> Node {
        Node::Duration(chrono::TimeDelta::seconds(2))
    }

    fn duration_3s_node() -> Node {
        Node::Duration(chrono::TimeDelta::seconds(3))
    }

    fn datetime_node() -> Node {
        Node::DateTime(chrono::DateTime::parse_from_rfc3339("2000-01-01T00:00:00Z").unwrap())
    }
}
