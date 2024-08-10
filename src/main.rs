use std::io::{self, BufRead};

mod parser;
use parser::basic::*;
use parser::core::*;

fn main() {
    let stdin = io::stdin();
    //let parser = FirstOf(&NumberParser, &LetterParser);
    let parser = &FirstOf(
        &CollapseParser(&NumberParser),
        &CollapseParser(&LetterParser),
    );

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let pointer = &mut InputPointer {
            input: &line,
            pos: 0,
        };
        loop {
            let new_match = match parser.parse(&pointer) {
                Ok(m) => m,
                Err(s) => {
                    println!("error! {}", s);
                    break;
                }
            };
            println!("{:?}", new_match);
            pointer.pos = new_match.pointer.pos;
            if pointer.is_end() {
                break;
            }
        }
    }
}
