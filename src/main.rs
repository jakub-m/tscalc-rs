use std::io::{self, BufRead};

/// The context passed around between the parsers.
#[derive(Debug)]
struct ParserPointer<'a> {
    /// The input string
    input: &'a String,
    /// Position in the input string
    pos: usize,
}

//struct NumberNode<'a> {
//    number: i32,
//    pointer: &'a ParserPointer<'a>,
//}

// TODO implement "advance"
// TODO implement returning arbitrary node

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let context = &mut ParserPointer {
            input: &line,
            pos: 0,
        };
        loop {
            let new_context = match parse_number(&context) {
                Ok(c) => c,
                Err(s) => {
                    println!("error! {}", s);
                    break;
                }
            };
            println!("{:?}", new_context);
            context.pos = new_context.pos; // how to reassign whole context?
        }
    }
}

fn parse_number<'a>(context: &'a ParserPointer) -> Result<ParserPointer<'a>, &'static str> {
    let rest = &context.input[context.pos..];
    for (i, c) in rest.char_indices() {
        if i == 0 {
            continue;
        }
        if c >= '0' && c <= '9' {
            return Ok(ParserPointer {
                input: context.input,
                pos: context.pos + i,
            });
        } else {
            return Err("not a number");
        }
    }
    return Err("end of parser");
}
