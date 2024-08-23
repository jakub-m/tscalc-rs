use crate::log::debug_log;

use super::{Node, Oper};
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

/// Evaluation works by modifying state with node.
fn eval(
    state: State,
    node: Node,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<State, String> {
    debug_log(format!("eval input: {:?} {:?}", state, node));
    let eval_result = match state {
        State::DateTime(datetime) => match node {
            Node::Expr(nodes) => Ok(eval_oper_expr(&state, &nodes)?),
            Node::OperNode { oper, node: expr } => {
                todo!()
            }
            Node::Skip(_) => Ok(state),
            Node::Now | Node::Duration(_) | Node::DateTime(_) => {
                Err(format!("cannot evaluate {:?} with {:?}", node, state))
            }
        },
        State::TimeDelta(_) => todo!(),
        State::None => todo!(),
    };
    todo!();
    //let eval_result = match node {
    //    Node::DateTime(datetime) => match state {
    //        State::TimeDelta(_) => Err("Cannot apply datetime to timedelta".to_string()),
    //        State::DateTime(_) => Err("Cannot apply datetime to datetime".to_string()),
    //        State::None => Ok(State::DateTime(datetime)),
    //    },
    //    Node::Duration(duration) => match state {
    //        State::TimeDelta(_) => Err("Cannot apply duration to timedelta".to_string()),
    //        State::DateTime(_) => Err("Cannot apply duration to datetime".to_string()),
    //        State::None => Ok(State::TimeDelta(duration)),
    //    },
    //    Node::Now => match state {
    //        State::TimeDelta(_) => Err("Cannot apply now to timedelta".to_string()),
    //        State::DateTime(_) => Err("Cannot apply now to datetime".to_string()),
    //        State::None => Ok(State::DateTime(now)),
    //    },
    //    Node::Expr(nodes) => eval_expr(state, nodes, now),
    //    Node::OperExpr { oper, expr } => eval_expr(state, expr, now),
    //    Node::Skip(_) => Ok(state),
    //    //Node::Expr(nodes) => eval_list(state, nodes, now),
    //};
    //debug_log(format!("eval output: {:?}", eval_result));
    //eval_result
}

/// Apply expression on state assuming that each node in the expression is OperExpr.
fn eval_oper_expr(state: &State, nodes: &Vec<Node>) -> Result<State, String> {
    todo!()
}

//fn eval_oper(state: State, oper: Oper, node: Node) -> Result<State, String> {}
//
//fn eval_expr(
//    state: State,
//    nodes: Vec<Node>,
//    now: chrono::DateTime<chrono::FixedOffset>,
//) -> Result<State, String> {
//    let mut nodes_iter = nodes.iter();
//    let node = nodes_iter.next();
//    if node.is_none() {
//        return Ok(state);
//    }
//    let node = node.unwrap();
//    let mut state = eval(state, node.clone(), now)?;
//    for node in nodes_iter {
//        if let Node::OperExpr {
//            oper,
//            expr: sub_expr,
//        } = node
//        {
//            let sub_state = eval(State::None, Node::Expr(sub_expr.clone()), now)?;
//            if let (State::DateTime(left), Oper::Minus, State::DateTime(right)) =
//                (&state, oper, &sub_state)
//            {
//                state = State::TimeDelta(*left - *right);
//            } else if let (State::DateTime(left), Oper::Minus, State::TimeDelta(right)) =
//                (&state, oper, &sub_state)
//            {
//                state = State::DateTime(*left - *right);
//            } else if let (State::DateTime(left), Oper::Plus, State::TimeDelta(right)) =
//                (&state, oper, &sub_state)
//            {
//                state = State::DateTime(*left + *right);
//            } else if let (State::TimeDelta(left), Oper::Plus, State::DateTime(right)) =
//                (&state, oper, &sub_state)
//            {
//                state = State::DateTime(*right + *left);
//            } else if let (State::TimeDelta(left), Oper::Minus, State::TimeDelta(right)) =
//                (&state, oper, &sub_state)
//            {
//                state = State::TimeDelta(*left - *right);
//            } else if let (State::TimeDelta(left), Oper::Plus, State::TimeDelta(right)) =
//                (&state, oper, &sub_state)
//            {
//                state = State::TimeDelta(*left + *right);
//            } else {
//                return Err(format!(
//                    "Cannot evaluate operation {:?} {:?} {:?}",
//                    state, oper, sub_state
//                ));
//            }
//        } else {
//            return Err(format!(
//                "BUG! expected OperExpr got {:?} in {:?}",
//                node, nodes
//            ));
//        }
//    }
//    return Ok(state);
//}

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
