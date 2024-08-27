use std::{
    env,
    error::Error,
    io::{self, BufRead},
    process,
};

mod log;

mod parser;
use chrono::SubsecRound;
use parser::{evaluate, parse_expr, ShortFormat};
use std::fmt::Write;

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_cli_args()?;
    if args.print_help {
        print_help();
        process::exit(0);
    }
    let stdin = io::stdin();
    // Intentionally truncate to seconds to make the calculator more practical (although less precise).
    let now = chrono::Utc::now().trunc_subsecs(0);

    if let Some(input) = args.expression {
        match parse_and_eval(&input, args.output_format, now.into()) {
            Ok(output) => println!("{}", output),
            Err(message) => {
                println!("{}", message);
                process::exit(1);
            }
        }
    } else {
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            match parse_and_eval(&line, args.output_format, now.into()) {
                Ok(output) => println!("{}", output),
                Err(message) => {
                    println!("{}", message);
                    process::exit(1);
                }
            }
        }
    };
    Ok(())
}

#[derive(Debug)]
struct Args {
    output_format: OutputFormat,
    print_help: bool,
    expression: Option<String>,
}

fn parse_cli_args() -> Result<Args, String> {
    let mut output = Args {
        output_format: OutputFormat::ISO,
        print_help: false,
        expression: None,
    };
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    let mut found_sentinel = false;
    loop {
        if i >= args.len() {
            break;
        }
        let param = args.get(i).unwrap();
        if found_sentinel {
            output = Args {
                expression: Some(
                    output
                        .expression
                        .map_or(param.to_owned(), |s| s + " " + param),
                ),
                ..output
            }
        } else if param == "-h" {
            output = Args {
                print_help: true,
                ..output
            };
        } else if param == "-s" {
            output = Args {
                output_format: OutputFormat::EPOCH_SECONDS,
                ..output
            }
        } else if param == "--" {
            found_sentinel = true;
        } else {
            return Err(format!("unknown param {:?}", param));
        }
        i = i + 1;
    }
    Ok(output)
}

fn print_help() {
    let help = "
Simple calculator for date-time and durations.

Built-in functions:
- full_day\tReturn full day of the date-time.
- full_hour\tReturn full hour of the date-time.

-s\tOutput time as epoch seconds.
-h\tPrint this help.
--\tAfter this sentinel, concatenate all the arguments into a single expression.
";
    println!("{}", help.trim());
}

#[derive(Clone, Copy, Debug)]
enum OutputFormat {
    ISO,
    EPOCH_SECONDS,
}

fn parse_and_eval(
    input: &String,
    output_format: OutputFormat,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> Result<String, String> {
    let parse_result = parse_expr(input);
    if let Err(parse_err) = parse_result {
        let mut m = String::from("");
        write!(m, "{}", parse_err.pointer.input).unwrap();
        write!(m, "\n{}^", "_".repeat(parse_err.pointer.pos)).unwrap();
        write!(m, "\n{}", parse_err.message).unwrap();
        return Err(m);
    }
    let parse_ok = parse_result.unwrap();
    let eval_result = evaluate(parse_ok.node, now)?;
    return Ok(match eval_result {
        parser::EvaluationResult::DateTime(datetime) => match output_format {
            OutputFormat::ISO => datetime.to_rfc3339(),
            OutputFormat::EPOCH_SECONDS => {
                format!("{:.3}", (datetime.timestamp_millis() as f64) / 1000.0)
            }
        },
        parser::EvaluationResult::TimeDelta(delta) => match output_format {
            OutputFormat::ISO => delta.as_short_format(),
            OutputFormat::EPOCH_SECONDS => todo!(),
        },
    });
}

#[cfg(test)]
mod tests {
    use crate::parse_and_eval;

    #[test]
    fn test_eval_garbage_on_right() {
        check_parse_and_eval("1h + 2h + 2000-01-01T00:00:00Z garbage", None);
    }

    #[test]
    fn test_eval_with_now() {
        check_parse_and_eval("1s + now", Some("2001-01-01T01:01:02+00:00"));
    }

    #[test]
    fn test_eval_brackets_1() {
        check_parse_and_eval("now - (1s - 1s)", Some("2001-01-01T01:01:01+00:00"));
    }

    #[test]
    fn test_eval_brackets_2() {
        check_parse_and_eval(
            "(1s - (2s - 1s)) + now - (1s - (2s - 1s))",
            Some("2001-01-01T01:01:01+00:00"),
        );
    }

    #[test]
    fn test_eval_brackets_3() {
        check_parse_and_eval(
            "(now - (now - 1d)) + now - (now - (now - 1d))",
            Some("2001-01-01T01:01:01+00:00"),
        );
    }

    #[test]
    fn test_eval_func_full_day_1() {
        check_parse_and_eval("full_day(now)", Some("2001-01-01T00:00:00+00:00"));
    }

    #[test]
    fn test_eval_timestamp_1() {
        check_parse_and_eval("1234567890.000", Some("2009-02-13T23:31:30+00:00"));
    }
    #[test]
    fn test_eval_timestamp_2() {
        check_parse_and_eval("0.0 + (0.0 - 1.0)", Some("1969-12-31T23:59:59+00:00"));
    }

    #[test]
    fn test_eval_timestamp_3() {
        check_parse_and_eval("0", Some("1970-01-01T00:00:00+00:00"));
    }

    #[test]
    fn test_eval_timestamp_4() {
        check_parse_and_eval("0.0", Some("1970-01-01T00:00:00+00:00"));
    }

    #[test]
    fn test_eval_timestamp_5() {
        check_parse_and_eval("0.1", Some("1970-01-01T00:00:00.100+00:00"));
    }

    #[test]
    fn test_eval_timestamp_6() {
        check_parse_and_eval("0.12345", Some("1970-01-01T00:00:00.123450+00:00"));
    }

    #[test]
    fn test_eval_missing_bracket_1() {
        check_parse_and_eval("0.0 + (0.0 - 1.0", None);
    }

    fn check_parse_and_eval(input: &str, expected: Option<&str>) {
        let result = parse_and_eval(&input.to_string(), crate::OutputFormat::ISO, now());
        let result_str = format!("{:?}", result);
        if let Some(expected) = expected {
            let actual = result.expect(&format!("expected ok result, got: {}", result_str));
            assert_eq!(actual, expected);
        } else {
            result.expect_err("expected err result");
        }
    }

    fn now() -> chrono::DateTime<chrono::FixedOffset> {
        chrono::DateTime::parse_from_rfc3339("2001-01-01T01:01:01Z").unwrap()
    }
}
