use std::{
    env,
    error::Error,
    io::{self, BufRead},
    process,
    str::FromStr,
};

mod log;

mod parser;
use chrono::SubsecRound;
use chrono_tz;
use chrono_tz::{Tz, UTC};
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
    let now = chrono::Utc::now()
        .trunc_subsecs(0)
        .with_timezone(&args.timezone.unwrap_or(UTC));

    let print_result_or_exit = |eval_result: Result<String, String>| {
        match eval_result {
            Ok(output) => println!("{}", output),
            Err(message) => {
                println!("{}", message);
                process::exit(1);
            }
        };
    };

    let output_tz = args.timezone.unwrap_or(UTC);

    if let Some(input) = args.expression {
        let eval_result = parse_and_eval(&input, args.output_format, &output_tz, now);
        print_result_or_exit(eval_result);
    } else if args.read_from_stdin {
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            let eval_result = parse_and_eval(&line, args.output_format, &output_tz, now.into());
            print_result_or_exit(eval_result);
        }
    } else {
        let input = "now".to_string();
        let eval_result = parse_and_eval(&input, args.output_format, &output_tz, now.into());
        print_result_or_exit(eval_result);
    };
    Ok(())
}

// #[derive(Debug)]
struct Args {
    output_format: OutputFormat,
    print_help: bool,
    expression: Option<String>,
    read_from_stdin: bool,
    //timezone: chrono::FixedOffset,
    timezone: Option<Tz>,
}

fn parse_cli_args() -> Result<Args, String> {
    let mut output = Args {
        output_format: OutputFormat::ISO,
        print_help: false,
        expression: None,
        read_from_stdin: false,
        timezone: None,
    };
    let args: Vec<String> = env::args().collect();
    let mut found_sentinel = false;
    let mut iter_args = args.iter();
    iter_args.next();
    while let Some(arg) = iter_args.next() {
        if found_sentinel {
            output = Args {
                expression: Some(output.expression.map_or(arg.to_owned(), |s| s + " " + arg)),
                ..output
            }
        } else if arg == "-i" {
            output = Args {
                read_from_stdin: true,
                ..output
            };
        } else if arg == "-h" {
            output = Args {
                print_help: true,
                ..output
            };
        } else if arg == "-s" {
            output = Args {
                output_format: OutputFormat::EPOCH_SECONDS,
                ..output
            }
        } else if arg == "-S" {
            output = Args {
                output_format: OutputFormat::FULL_EPOCH_SECONDS,
                ..output
            }
        } else if arg == "-tz" {
            let tz_str = iter_args.next().ok_or("expected timezone".to_string())?;
            let tz = Tz::from_str(&tz_str).map_err(|err: chrono_tz::ParseError| {
                format!("failed to parse {:?}: {}", tz_str, err)
            })?;
            output = Args {
                timezone: Some(tz),
                ..output
            }
        } else if arg == "--" {
            found_sentinel = true;
        } else {
            return Err(format!("unknown param {:?}", arg));
        }
    }
    Ok(output)
}

fn print_help() {
    let help = "
Simple calculator for date-time and durations.

Built-in functions:
- full_day\tReturn full day of the date-time.
- full_hour\tReturn full hour of the date-time.

-i\tRead input from stdin and process line by line.
-s\tOutput time as epoch seconds.
-S\tOutput time as epoch seconds, without the decimal part.
-tz\tTimezone like US/Eastern or Europe/Warsaw , as in https://docs.rs/chrono-tz/latest/chrono_tz/enum.Tz.html
-h\tPrint this help.
--\tAfter this sentinel, concatenate all the arguments into a single expression.
";
    println!("{}", help.trim());
}

#[derive(Clone, Copy, Debug)]
enum OutputFormat {
    ISO,
    EPOCH_SECONDS,
    FULL_EPOCH_SECONDS,
}

fn parse_and_eval(
    input: &String,
    output_format: OutputFormat,
    output_tz: &chrono_tz::Tz,
    now: chrono::DateTime<Tz>,
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
            OutputFormat::ISO => datetime.with_timezone(output_tz).to_rfc3339(),
            OutputFormat::EPOCH_SECONDS => {
                format!("{:.3}", (datetime.timestamp_millis() as f64) / 1000.0)
            }
            OutputFormat::FULL_EPOCH_SECONDS => format!("{}", (datetime.timestamp_millis() / 1000)),
        },
        parser::EvaluationResult::TimeDelta(delta) => match output_format {
            OutputFormat::ISO => delta.as_short_format(),
            OutputFormat::EPOCH_SECONDS => todo!("display delta as seconds"),
            OutputFormat::FULL_EPOCH_SECONDS => todo!("display delta as full seconds"),
        },
    });
}

#[cfg(test)]
mod tests {
    use crate::parse_and_eval;
    use chrono_tz::{Tz, UTC};

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

    #[test]
    fn test_eval_duration_6() {
        check_parse_and_eval(
            "2000-01-01T00:00:00Z + 1m2s3ms",
            Some("2000-01-01T00:01:02.003+00:00"),
        );
    }

    #[test]
    fn test_eval_different_tz_1() {
        check_parse_and_eval_tz(
            "2000-01-02T00:00:00Z",
            Some("2000-01-01T19:00:00-05:00"),
            &chrono_tz::US::Eastern,
        );
    }

    fn check_parse_and_eval(input: &str, expected: Option<&str>) {
        check_parse_and_eval_tz(input, expected, &UTC)
    }

    fn check_parse_and_eval_tz(input: &str, expected: Option<&str>, tz: &chrono_tz::Tz) {
        let result = parse_and_eval(&input.to_string(), crate::OutputFormat::ISO, tz, now());
        let result_str = format!("{:?}", result);
        if let Some(expected) = expected {
            let actual = result.expect(&format!("expected ok result, got: {}", result_str));
            assert_eq!(actual, expected);
        } else {
            result.expect_err("expected err result");
        }
    }

    fn now() -> chrono::DateTime<Tz> {
        chrono::DateTime::parse_from_rfc3339("2001-01-01T01:01:01Z")
            .unwrap()
            .with_timezone(&UTC)
    }
}
