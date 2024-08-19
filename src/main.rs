use std::{
    env,
    io::{self, BufRead},
    process,
};

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

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        match parse_and_eval(
            &line,
            if args.output_epoch_seconds {
                OutputFormat::EPOCH_SECONDS
            } else {
                OutputFormat::ISO
            },
        ) {
            Ok(output) => println!("{}", output),
            Err(message) => {
                println!("{}", message);
                process::exit(1);
            }
        }
    }
}

struct Args {
    output_epoch_seconds: bool,
    print_help: bool,
}

fn parse_cli_args() -> Args {
    let mut output = Args {
        output_epoch_seconds: false,
        print_help: false,
    };
    let args: Vec<String> = env::args().collect();
    let mut i = 0;
    loop {
        if i >= args.len() {
            break;
        }
        let param = args.get(i).unwrap();
        if param == "-h" {
            output = Args {
                print_help: true,
                ..output
            };
        } else if param == "-s" {
            output = Args {
                output_epoch_seconds: true,
                ..output
            }
        }
        i = i + 1;
    }
    output
}

fn print_help() {
    println!("-s\tOutput time as epoch seconds.");
    println!("-h\tPrint this help.");
}

enum OutputFormat {
    ISO,
    EPOCH_SECONDS,
}

fn parse_and_eval(input: &String, output_format: OutputFormat) -> Result<String, String> {
    let parse_result = parse_expr(input);
    if let Err(parse_err) = parse_result {
        let mut m = String::from("");
        write!(m, "{}", parse_err.pointer.input).unwrap();
        write!(m, "\n{}^", "_".repeat(parse_err.pointer.pos)).unwrap();
        write!(m, "\n{}", parse_err.message).unwrap();
        return Err(m);
    }
    let parse_ok = parse_result.unwrap();
    let eval_result = eval_to_datetime(parse_ok.node);
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
        let result = parse_and_eval(&input, crate::OutputFormat::ISO);
        assert!(
            result.is_err(),
            "expected err for input {:?}, got {:?}",
            input,
            result
        );
    }
}
