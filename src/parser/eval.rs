use crate::log::debug_log;

use super::{full_day, full_hour, Node, Oper};
use chrono_tz::Tz;

#[derive(Debug, PartialEq)]
pub enum EvaluationResult {
    TimeDelta(chrono::TimeDelta),
    DateTime(chrono::DateTime<Tz>),
}

pub fn evaluate(node: Node, now: chrono::DateTime<Tz>) -> Result<EvaluationResult, String> {
    debug_log(format!("eval_to_date node {:?}", node));
    match eval(&State::None, &node, now) {
        Ok(state) => match state {
            State::DateTime(datetime) => Ok(EvaluationResult::DateTime(datetime)),
            State::TimeDelta(delta) => Ok(EvaluationResult::TimeDelta(delta)),
            State::None => Err("BUG: the result of evaluation was State::None".to_string()),
        },
        Err(m) => Err(m),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum State {
    TimeDelta(chrono::TimeDelta),
    DateTime(chrono::DateTime<Tz>),
    None,
}

/// Evaluation works by modifying state with node.
fn eval(state: &State, node: &Node, now: chrono::DateTime<Tz>) -> Result<State, String> {
    debug_log(format!("eval input: {:?} {:?}", state, node));
    let eval_result = match node {
        Node::Expr(nodes) => eval_expr(&state, &nodes, now),
        Node::OperNode { oper, node: expr } => apply_oper_node(state, oper, expr.as_ref(), now),
        Node::Literal {
            literal: _,
            skip: _,
        } => Ok((*state).clone()),
        Node::Duration(duration) => {
            if let State::None = state {
                Ok(State::TimeDelta(duration.clone()))
            } else {
                Err(format!("cannot evaluate {:?} with {:?}", node, state))
            }
        }
        Node::DateTime(datetime) => {
            if let State::None = state {
                Ok(State::DateTime(datetime.clone()))
            } else {
                Err(format!("cannot evaluate {:?} with {:?}", node, state))
            }
        }
        Node::Now => {
            if let State::None = state {
                Ok(State::DateTime(now.clone()))
            } else {
                Err(format!("cannot evaluate {:?} with {:?}", node, state))
            }
        }
        Node::FuncAry1 { name, arg1 } => {
            let arg_evaluated = eval(&State::None, arg1, now)?;
            eval_func_ary1(name, &arg_evaluated)
        }
    };
    debug_log(format!("eval output: {:?}", eval_result));
    eval_result
}

fn eval_expr(state: &State, nodes: &Vec<Node>, now: chrono::DateTime<Tz>) -> Result<State, String> {
    let mut state = (*state).clone();
    for node in nodes {
        state = eval(&state, node, now)?;
    }
    Ok(state)
}

/// Apply state, oper, node.
fn apply_oper_node(
    state: &State,
    oper: &Oper,
    node: &Node,
    now: chrono::DateTime<Tz>,
) -> Result<State, String> {
    let sub_state = eval(&State::None, node, now)?;
    if let (State::DateTime(left), Oper::Minus, State::DateTime(right)) = (&state, oper, &sub_state)
    {
        return Ok(State::TimeDelta(*left - *right));
    } else if let (State::DateTime(left), Oper::Minus, State::TimeDelta(right)) =
        (&state, oper, &sub_state)
    {
        return Ok(State::DateTime(*left - *right));
    } else if let (State::DateTime(left), Oper::Plus, State::TimeDelta(right)) =
        (&state, oper, &sub_state)
    {
        return Ok(State::DateTime(*left + *right));
    } else if let (State::TimeDelta(left), Oper::Plus, State::DateTime(right)) =
        (&state, oper, &sub_state)
    {
        return Ok(State::DateTime(*right + *left));
    } else if let (State::TimeDelta(left), Oper::Minus, State::TimeDelta(right)) =
        (&state, oper, &sub_state)
    {
        return Ok(State::TimeDelta(*left - *right));
    } else if let (State::TimeDelta(left), Oper::Plus, State::TimeDelta(right)) =
        (&state, oper, &sub_state)
    {
        return Ok(State::TimeDelta(*left + *right));
    } else {
        return Err(format!(
            "Cannot evaluate operation {:?} {:?} {:?}",
            state, oper, sub_state
        ));
    }
}

fn eval_func_ary1(name: &String, arg1: &State) -> Result<State, String> {
    match name.as_str() {
        "full_day" => full_day(arg1),
        "full_hour" => full_hour(arg1),
        _ => Err(format!("no such function {:?}", name)),
    }
}

#[cfg(test)]
mod tests {
    use chrono_tz::{Tz, UTC};

    use super::super::parse_expr;
    use super::{evaluate, EvaluationResult};

    #[test]
    fn parse_and_eval_sums() {
        let input = "1d + 2h + 2000-01-01T00:00:00Z + 3m + 4s".to_string();
        let result_node = parse_expr(&input).unwrap().node;
        let result = evaluate(result_node, now());
        assert!(result.is_ok(), "result not ok: {:?}", result);
        assert_eq!(result.unwrap(), parse_from_rfc3339("2000-01-02T02:03:04Z"))
    }

    #[test]
    fn parse_and_eval_diff_duration() {
        let input = "1d + 2h + 2000-01-01T00:00:00Z - 1d - 2h".to_string();
        let result_node = parse_expr(&input).unwrap().node;
        let result = evaluate(result_node, now());
        assert!(result.is_ok(), "result not ok");
        assert_eq!(result.unwrap(), parse_from_rfc3339("2000-01-01T00:00:00Z"))
    }

    #[test]
    fn parse_and_eval_diff_datetimes_1() {
        let input =
            "1999-01-01T01:00:00Z - 1999-01-01T00:00:00Z + 2000-01-01T00:00:00Z".to_string();
        let result_node = parse_expr(&input).unwrap().node;
        let result = evaluate(result_node, now());
        assert!(result.is_ok(), "result not ok");
        assert_eq!(result.unwrap(), parse_from_rfc3339("2000-01-01T01:00:00Z"))
    }

    #[test]
    fn parse_and_eval_diff_datetimes_2() {
        let input =
            "1s + 1999-01-01T01:00:00Z - 1m - 1999-01-01T00:00:00Z + -2s + 2000-01-01T00:00:00Z"
                .to_string();
        let result_node = parse_expr(&input).unwrap().node;
        let result = evaluate(result_node, now());
        assert!(result.is_ok(), "result not ok");
        assert_eq!(result.unwrap(), parse_from_rfc3339("2000-01-01T00:58:59Z"))
    }

    fn parse_from_rfc3339(s: &str) -> EvaluationResult {
        EvaluationResult::DateTime(
            chrono::DateTime::parse_from_rfc3339(s)
                .unwrap()
                .with_timezone(&UTC),
        )
    }

    fn now() -> chrono::DateTime<Tz> {
        chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono_tz::UTC)
    }
}
