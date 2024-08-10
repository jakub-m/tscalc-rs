// Basic parsers.

use super::core::*;

/// CharRangeParser checks if the input char is between the two chars specified in the constructor (inclusive).
pub struct CharRangeParser(pub char, pub char);

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
                matched: pointer.peek_n(offset),
            })
        } else {
            Err(format!("Character not in [{0}, {1}]", self.0, self.1))
        }
    }
}

#[allow(non_upper_case_globals)]
pub const LetterParser: CharRangeParser = CharRangeParser('a', 'z');
#[allow(non_upper_case_globals)]
pub const NumberParser: CharRangeParser = CharRangeParser('0', '9');

pub struct FirstOf<'a>(pub &'a dyn Parser, pub &'a dyn Parser);

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

/// A parser that collapses many matches of the underlying parser into a single match.
pub struct CollapseParser<'a>(pub &'a dyn Parser);

impl<'a> Parser for CollapseParser<'a> {
    fn parse<'b>(&self, pointer: &'b InputPointer) -> Result<Match<'b>, String> {
        let mut current_pos: usize = pointer.pos;
        loop {
            let current_pointer = pointer.at_pos(current_pos);
            let m = self.0.parse(&current_pointer);
            if m.is_ok() {
                current_pos = m.unwrap().pointer.pos;
            } else {
                if current_pos == pointer.pos {
                    // Return the error since no parser matched anything.
                    return Err(m.unwrap_err());
                } else {
                    // The parser advanced before the error, so we are good.
                    let final_match = Match {
                        pointer: pointer.at_pos(current_pos),
                        matched: &pointer.input[pointer.pos..current_pos],
                    };
                    return Ok(final_match);
                }
            }
        }
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
