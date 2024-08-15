use std::{
    io::{self, BufRead},
    process,
};

mod parser;
use parser::{eval_to_datetime, parse_expr};

fn main() {
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let parse_result = parse_expr(&line);
        if let Err(parse_err) = parse_result {
            println!("{}", parse_err.message); // TODO better error on parse
            process::exit(1);
        }
        let parse_ok = parse_result.unwrap();
        let eval_result = eval_to_datetime(parse_ok.node);
        if let Err(message) = eval_result {
            println!("{}", message);
            process::exit(1);
        }
        println!("{}", eval_result.unwrap().to_rfc3339());
    }
}
