use std::io::{self, BufRead};

/// The context passed around between the parsers.
#[derive(Debug)]
struct ParserPointer<'a> {
    /// The input string
    input: &'a String,
    /// Position in the input string
    pos: usize,
}

fn main() {
    let stdin = io::stdin();
    let parser = FirstOf(&NumberParser, &LetterParser);
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let context = &mut ParserPointer {
            input: &line,
            pos: 0,
        };
        loop {
            let new_context = match parser.parse(&context) {
                Ok(c) => c,
                Err(s) => {
                    println!("error! {}", s);
                    break;
                }
            };
            println!("{:?}", new_context);
            context.pos = new_context.pos;
            if context.pos == line.len() {
                // TODO add is_end() method
                break;
            }
        }
    }
}

trait Parser {
    fn parse<'a>(&self, context: &'a ParserPointer) -> Result<ParserPointer<'a>, String>;
}

struct NumberParser;

impl Parser for NumberParser {
    fn parse<'a>(&self, context: &'a ParserPointer) -> Result<ParserPointer<'a>, String> {
        let rest = &context.input[context.pos..];
        let mut offset = rest.len();
        let mut is_ok = false;
        for (i, c) in rest.char_indices() {
            if i == 0 {
                if c >= '0' && c <= '9' {
                    is_ok = true;
                } else {
                    break;
                }
            } else {
                // There is a next character, so use this character position as the offset.
                offset = i;
                break;
            }
        }
        if is_ok {
            Ok(ParserPointer {
                input: context.input,
                pos: context.pos + offset,
            })
        } else {
            Err(String::from("not a number"))
        }
    }
}

struct LetterParser;

impl Parser for LetterParser {
    fn parse<'a>(&self, context: &'a ParserPointer) -> Result<ParserPointer<'a>, String> {
        let rest = &context.input[context.pos..];
        let mut offset = rest.len();
        let mut is_ok = false;
        for (i, c) in rest.char_indices() {
            if i == 0 {
                if c >= 'a' && c <= 'z' {
                    is_ok = true;
                } else {
                    break;
                }
            } else {
                // There is a next character, so use this character position as the offset.
                offset = i;
                break;
            }
        }
        if is_ok {
            Ok(ParserPointer {
                input: context.input,
                pos: context.pos + offset,
            })
        } else {
            Err(String::from("not a letter"))
        }
    }
}

struct FirstOf<'a>(&'a dyn Parser, &'a dyn Parser);

impl<'a> Parser for FirstOf<'a> {
    fn parse<'b>(&self, context: &'b ParserPointer) -> Result<ParserPointer<'b>, String> {
        let a_result = self.0.parse(context);
        if a_result.is_ok() {
            return a_result;
        }
        let b_result = self.1.parse(context);
        if b_result.is_ok() {
            return b_result;
        }
        Err(format!(
            "No parser matched for: {}",
            &context.input[context.pos..] // TODO implement rest() method for ParserPointer.
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_parser_with_number_is_ok() {
        let parser = NumberParser;
        let input = String::from("1");
        let pp = ParserPointer {
            input: &input,
            pos: 0,
        };
        let result = parser.parse(&pp);
        assert!(result.is_ok(), "result was: {:?}", result)
    }

    #[test]
    fn number_parser_with_garbage_is_not_ok() {
        let parser = NumberParser;
        let input = String::from("x");
        let pp = ParserPointer {
            input: &input,
            pos: 0,
        };
        let result = parser.parse(&pp);
        assert!(!result.is_ok(), "result was: {:?}", result)
    }

    #[test]
    fn letter_parser_with_letter_is_ok() {
        let parser = LetterParser;
        let input = String::from("x");
        let pp = ParserPointer {
            input: &input,
            pos: 0,
        };
        let result = parser.parse(&pp);
        assert!(result.is_ok(), "result was: {:?}", result)
    }

    #[test]
    fn letter_parser_with_garbage_is_not_ok() {
        let parser = LetterParser;
        let input = String::from("1");
        let pp = ParserPointer {
            input: &input,
            pos: 0,
        };
        let result = parser.parse(&pp);
        assert!(!result.is_ok(), "result was: {:?}", result)
    }
}
