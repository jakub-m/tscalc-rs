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
    let number_parser = NumberParser;
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let context = &mut ParserPointer {
            input: &line,
            pos: 0,
        };
        loop {
            let new_context = match number_parser.parse(&context) {
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

trait Parser {
    fn parse<'a>(&self, context: &'a ParserPointer) -> Result<ParserPointer<'a>, &'static str>;
}

struct NumberParser;

impl Parser for NumberParser {
    fn parse<'a>(&self, context: &'a ParserPointer) -> Result<ParserPointer<'a>, &'static str> {
        let rest = &context.input[context.pos..];
        for (i, c) in rest.char_indices() {
            if i == 0 {
                continue;
            }
            if c >= '0' && c <= '9' {
                return Ok(ParserPointer {
                    pos: context.pos + i,
                    ..*context
                });
            } else {
                return Err("not a number");
            }
        }
        return Err("end of parser");
    }
}

struct LetterParser;

impl Parser for LetterParser {
    fn parse<'a>(&self, context: &'a ParserPointer) -> Result<ParserPointer<'a>, &'static str> {
        let rest = &context.input[context.pos..];
        for (i, c) in rest.char_indices() {
            if i == 0 {
                continue;
            }
            if c >= 'a' && c <= 'z' {
                return Ok(ParserPointer {
                    pos: context.pos + i,
                    ..*context
                });
            } else {
                return Err("not a number");
            }
        }
        return Err("end of parser");
    }
}

struct FirstOf<'a> {
    a: &'a dyn Parser,
    b: &'a dyn Parser,
}

impl<'a> Parser for FirstOf<'a> {
    fn parse<'b>(&self, context: &'b ParserPointer) -> Result<ParserPointer<'b>, &'static str> {
        let a_result = self.a.parse(context);
        if a_result.is_ok() {
            return a_result;
        }
        let b_result = self.b.parse(context);
        if b_result.is_ok() {
            return b_result;
        }
        return Err("No parser matched");
    }
}
