use crate::log::debug_log;

use super::Node;
use chrono::{DateTime, FixedOffset};

pub fn eval_to_datetime(
    node: Node,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<DateTime<FixedOffset>, String> {
    debug_log(format!("eval_to_date node {:?}", node));
    match eval(State::None, node, now) {
        Ok(state) => match state {
            State::DateTime(datetime) => Ok(datetime),
            State::TimeDelta(_) => Err("the result of evaluation was duration".to_string()),
            State::None => Err("BUG: the result of evaluation was State::None".to_string()),
        },
        Err(m) => Err(m),
    }
}

#[derive(Debug)]
enum State {
    TimeDelta(chrono::TimeDelta),
    DateTime(chrono::DateTime<chrono::FixedOffset>),
    None,
}

fn eval(
    state: State,
    node: Node,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<State, String> {
    debug_log(format!("eval input: {:?} {:?}", state, node));
    let eval_result = match node {
        Node::Duration(delta) => match state {
            State::TimeDelta(prev_delta) => {
                Ok(State::TimeDelta(delta.checked_add(&prev_delta).unwrap()))
            }
            State::DateTime(datetime) => Ok(State::DateTime(datetime + delta)),
            State::None => Ok(State::TimeDelta(delta)),
        },
        Node::DateTime(datetime) => match state {
            State::TimeDelta(delta) => Ok(State::DateTime(datetime + delta)),
            State::DateTime(_) => Err("tried to add two datetimes".to_string()),
            State::None => Ok(State::DateTime(datetime)),
        },
        Node::Expr(nodes) => eval_list(state, nodes, now),
        Node::Skip(_) => Ok(state),
        Node::Now => match state {
            State::DateTime(_) => Err("cannot add now and datetime".to_string()),
            State::TimeDelta(delta) => Ok(State::DateTime(now + delta)),
            State::None => Ok(State::DateTime(now)),
        },
        Node::Plus => todo!(),
        Node::Minus => todo!(),
    };
    debug_log(format!("eval output: {:?}", eval_result));
    eval_result
}

fn eval_list(
    state: State,
    nodes: Vec<Node>,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<State, String> {
    let mut current_state = Some(state);
    for node in nodes {
        let result = eval(current_state.take().unwrap(), node, now);
        if let Ok(result_state) = result {
            current_state = Some(result_state);
        } else {
            return result;
        }
    }
    if let Some(state) = current_state {
        Ok(state)
    } else {
        Err("BUG: eval_list resulted in Option::None state".to_string())
    }
}

mod tests {
    use super::super::parse_expr;
    use super::eval_to_datetime;

    #[test]
    fn parse_and_eval_sums() {
        let input = "1d + 2h + 2000-01-01T00:00:00Z + 3m + 4s".to_string();
        let result_node = parse_expr(&input).unwrap().node;
        let result = eval_to_datetime(result_node, now());
        assert!(result.is_ok(), "result not ok");
        assert_eq!(result.unwrap(), parse_from_rfc3339("2000-01-02T02:03:04Z"))
    }

    #[test]
    fn parse_and_eval_diff_duration() {
        let input = "1d + 2h + 2000-01-01T00:00:00Z - 1d - 2h".to_string();
        let result_node = parse_expr(&input).unwrap().node;
        let result = eval_to_datetime(result_node, now());
        assert!(result.is_ok(), "result not ok");
        assert_eq!(result.unwrap(), parse_from_rfc3339("2000-01-01T00:00:00Z"))
    }

    #[test]
    fn parse_and_eval_diff_datetimes() {
        let input =
            "1999-01-01T01:00:00Z - 1999-01-01T00:00:00Z + 2000-01-01T01:00:00Z".to_string();
        let result_node = parse_expr(&input).unwrap().node;
        let result = eval_to_datetime(result_node, now());
        assert!(result.is_ok(), "result not ok");
        assert_eq!(result.unwrap(), parse_from_rfc3339("2000-01-01T01:00:00Z"))
    }

    #[test]
    fn parse_and_eval_diff_datetimes_and_deltas() {
        let input =
            "1s + 1999-01-01T01:00:00Z + 1m - 1999-01-01T00:00:00Z -1d + 2000-01-02T01:00:00Z"
                .to_string();
        let result_node = parse_expr(&input).unwrap().node;
        let result = eval_to_datetime(result_node, now());
        assert!(result.is_ok(), "result not ok");
        assert_eq!(result.unwrap(), parse_from_rfc3339("2000-01-01T01:01:01Z"))
    }

    fn parse_from_rfc3339(s: &str) -> chrono::DateTime<chrono::FixedOffset> {
        chrono::DateTime::parse_from_rfc3339(s).unwrap()
    }

    fn now() -> chrono::DateTime<chrono::FixedOffset> {
        chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap()
    }
}
