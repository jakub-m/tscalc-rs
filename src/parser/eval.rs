use super::Node;
use chrono::{DateTime, FixedOffset};

pub fn eval_to_datetime(
    node: Node,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<DateTime<FixedOffset>, String> {
    // Pass some dummy default state.
    match eval(State::None, node, now) {
        Ok(state) => match state {
            State::DateTime(datetime) => Ok(datetime),
            State::TimeDelta(_) => Err("the result of evaluation was duration".to_string()),
            State::None => Err("BUG: the result of evaluation was State::None".to_string()),
        },
        Err(m) => Err(m),
    }
}

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
    match node {
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
        Node::Durations(nodes) => eval_list(state, nodes, now),
        Node::Expr(nodes) => eval_list(state, nodes, now),
        Node::Skip(_) => Ok(state),
        Node::Now => Ok(State::DateTime(now)),
    }
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
        assert_eq!(
            result.unwrap(),
            chrono::DateTime::parse_from_rfc3339("2000-01-02T02:03:04Z").unwrap()
        )
    }

    #[test]
    fn parse_and_eval_diffs() {
        let input = "1d + 2h + 2000-01-01T00:00:00Z - 1d - 2h".to_string();
        let result_node = parse_expr(&input).unwrap().node;
        let result = eval_to_datetime(result_node, now());
        assert!(result.is_ok(), "result not ok");
        assert_eq!(
            result.unwrap(),
            chrono::DateTime::parse_from_rfc3339("2000-01-01T00:00:00Z").unwrap()
        )
    }

    fn now() -> chrono::DateTime<chrono::FixedOffset> {
        chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap()
    }
}
