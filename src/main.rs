use std::io::{self, BufRead};

mod parser;
use parser::{InputPointer, Parser};

fn main() {
    let stdin = io::stdin();
    let parser = &parser::FirstOf(
        &parser::Concat {
            parser: &parser::Digit,
            at_least: None,
            at_most: None,
        },
        &parser::Concat {
            parser: &parser::LowerCaseLetter,
            at_least: None,
            at_most: None,
        },
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
