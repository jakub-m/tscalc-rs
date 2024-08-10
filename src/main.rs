use std::io::{self, BufRead};

/// A context passed around between the parsers, pointing where in the input is the parser now.
#[derive(Debug)]
struct InputPointer<'a> {
    /// The input string.
    input: &'a String,
    /// Position in the input string.
    pos: usize,
}

impl<'a> InputPointer<'a> {
    fn is_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn rest(&self) -> &str {
        if self.is_end() {
            return &"";
        }
        &self.input[self.pos..]
    }

    fn advance(&self, n: usize) -> InputPointer<'a> {
        return InputPointer {
            input: self.input,
            pos: self.pos + n,
        };
    }
}

fn main() {
    let stdin = io::stdin();
    let parser = FirstOf(&NumberParser, &LetterParser);
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

trait Parser {
    fn parse<'a>(&self, pointer: &'a InputPointer) -> Result<Match<'a>, String>;
}

#[derive(Debug)]
struct Match<'a> {
    pointer: InputPointer<'a>,
    //chars: &'a str,
}

/// CharRangeParser checks if the input char is between the two chars specified in the constructor (inclusive).
struct CharRangeParser(char, char);

impl Parser for CharRangeParser {
    fn parse<'a>(&self, pointer: &'a InputPointer) -> Result<Match<'a>, String> {
        let mut offset = pointer.rest().len();
        let mut is_ok = false;
        for (i, c) in pointer.rest().char_indices() {
            if i == 0 {
                if c >= self.0 && c <= self.1 {
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
            Ok(Match {
                pointer: pointer.advance(offset),
            })
        } else {
            Err(format!("Character not in [{0}, {1}]", self.0, self.1))
        }
    }
}

#[allow(non_upper_case_globals)]
const LetterParser: CharRangeParser = CharRangeParser('a', 'z');
#[allow(non_upper_case_globals)]
const NumberParser: CharRangeParser = CharRangeParser('0', '9');

struct FirstOf<'a>(&'a dyn Parser, &'a dyn Parser);

impl<'a> Parser for FirstOf<'a> {
    fn parse<'b>(&self, pointer: &'b InputPointer) -> Result<Match<'b>, String> {
        let a_result = self.0.parse(pointer);
        if a_result.is_ok() {
            return a_result;
        }
        let b_result = self.1.parse(pointer);
        if b_result.is_ok() {
            return b_result;
        }
        Err(format!("No parser matched for: {}", pointer.rest()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letter_parser_with_letter_is_ok() {
        let parser = LetterParser;
        let input = String::from("x");
        let pp = InputPointer {
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
        let pp = InputPointer {
            input: &input,
            pos: 0,
        };
        let result = parser.parse(&pp);
        assert!(!result.is_ok(), "result was: {:?}", result)
    }
}
