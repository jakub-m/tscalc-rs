use chrono::{DurationRound, TimeDelta};

use super::State;

pub fn full_day(arg1: &State) -> Result<State, String> {
    let datetime = if let State::DateTime(datetime) = arg1 {
        datetime
    } else {
        return Err(format!(
            "the first argument to full_day should be datetime, was: {:?}",
            arg1
        ));
    };
    let truncated = datetime.duration_trunc(TimeDelta::days(1)).unwrap();
    Ok(State::DateTime(truncated))
}

pub fn full_hour(arg1: &State) -> Result<State, String> {
    let datetime = if let State::DateTime(datetime) = arg1 {
        datetime
    } else {
        return Err(format!(
            "the first argument to full_hour should be datetime, was: {:?}",
            arg1
        ));
    };
    let truncated = datetime.duration_trunc(TimeDelta::hours(1)).unwrap();
    Ok(State::DateTime(truncated))
}
