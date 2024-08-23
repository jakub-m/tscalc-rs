use std::{
    env,
    io::{self, BufRead},
    process,
};

mod log;

mod parser;
use parser::{eval_to_datetime, parse_expr};
use std::fmt::Write;

fn main() {
    let args = parse_cli_args();
    if args.print_help {
        print_help();
        process::exit(0);
    }
    let stdin = io::stdin();
    let now = chrono::Utc::now();

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
    }
}

#[derive(Debug)]
struct Args {
    output_format: OutputFormat,
    print_help: bool,
    expression: Option<String>,
}

fn parse_cli_args() -> Args {
    let mut output = Args {
        output_format: OutputFormat::ISO,
        print_help: false,
        expression: None,
    };
    let args: Vec<String> = env::args().collect();
    let mut i = 0;
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
        }
        i = i + 1;
    }
    output
}

fn print_help() {
    println!("-s\tOutput time as epoch seconds.");
    println!("-h\tPrint this help.");
    println!("--\tAfter this sentinel, concatenate all the arguments into a single expression.");
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
    let eval_result = eval_to_datetime(parse_ok.node, now);
    if let Err(message) = eval_result {
        return Err(message);
    }
    return Ok(match output_format {
        OutputFormat::ISO => eval_result.unwrap().to_rfc3339(),
        OutputFormat::EPOCH_SECONDS => {
            format!(
                "{:.3}",
                (eval_result.unwrap().timestamp_millis() as f64) / 1000.0
            )
        }
    });
}

mod tests {
    use crate::parse_and_eval;

    #[test]
    fn test_eval_garbage_on_right() {
        let input = "1h + 2h + 2000-01-01T00:00:00Z garbage".to_string();
        let result = parse_and_eval(&input, crate::OutputFormat::ISO, now());
        assert!(
            result.is_err(),
            "expected err for input {:?}, got {:?}",
            input,
            result
        );
    }

    // TODO add macro assert_ok!
    #[test]
    fn test_eval_with_now() {
        let input = "1s + now".to_string();
        let result = parse_and_eval(&input, crate::OutputFormat::ISO, now());
        assert!(result.is_ok(), "expected ok was {:?}", result);
        assert_eq!(result.unwrap(), "2001-01-01T01:01:02+00:00");
    }

    fn now() -> chrono::DateTime<chrono::FixedOffset> {
        chrono::DateTime::parse_from_rfc3339("2001-01-01T01:01:01Z").unwrap()
    }
}
