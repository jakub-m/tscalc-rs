use std::{
    io::{self, BufRead},
    process,
};

mod parser;
use parser::{eval_to_datetime, parse_expr};
use std::fmt::Write;

fn main() {
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        match parse_and_eval(&line) {
            Ok(output) => println!("{}", output),
            Err(message) => {
                println!("{}", message);
                process::exit(1);
            }
        }
    }
}

fn parse_and_eval(input: &String) -> Result<String, String> {
    let parse_result = parse_expr(input);
    if let Err(parse_err) = parse_result {
        let mut m = String::from("");
        write!(m, "{}\n{}", parse_err.message, parse_err.pointer.input).unwrap();
        write!(m, "\n{}^", "_".repeat(parse_err.pointer.pos)).unwrap();
        return Err(m);
    }
    let parse_ok = parse_result.unwrap();
    let eval_result = eval_to_datetime(parse_ok.node);
    if let Err(message) = eval_result {
        return Err(message);
    }
    return Ok(eval_result.unwrap().to_rfc3339());
}
