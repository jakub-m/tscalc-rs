use std::io::{self, BufRead};

/// The context passed around between the parsers.
#[derive(Debug)]
struct ParserContext<'a> {
    /// The input string
    input: &'a String,
    /// Position in the input string
    pos: usize,
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let context = &mut ParserContext {
            input: &line,
            pos: 0,
        };
        loop {
            let new_context = match parse_number(&context) {
                Ok(c) => c,
                Err(s) => {
                    println!("{}", s);
                    break;
                }
            };
            println!("{:?}", new_context);
            context.pos = new_context.pos; // how to reassign whole context?
        }
    }
}

fn parse_number<'a>(context: &'a ParserContext) -> Result<ParserContext<'a>, &'static str> {
    let rest = &context.input[context.pos..];
    for (i, c) in rest.char_indices() {
        if i == 0 {
            continue;
        }
        if c >= '0' && c <= '9' {
            return Ok(ParserContext {
                input: context.input,
                pos: context.pos + i,
            });
        } else {
            return Err("not a number");
        }
    }
    return Err("end of parser");
}
