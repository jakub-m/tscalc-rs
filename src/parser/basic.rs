// Basic parsers.

use super::core::*;

/// CharRangeParser checks if the input char is between the two chars specified in the constructor (inclusive).
pub struct CharRangeParser(pub char, pub char);

impl Parser for CharRangeParser {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<Match<'a>, String> {
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
            let advanced = pointer.advance(offset);
            let matched = pointer.peek_n(offset);
            Ok(Match {
                pointer: advanced,
                matched,
            })
        } else {
            Err(format!("Character not in [{0}, {1}]", self.0, self.1))
        }
    }
}

#[allow(non_upper_case_globals)]
pub const LowerCaseLetter: CharRangeParser = CharRangeParser('a', 'z');
#[allow(non_upper_case_globals)]
pub const Digit: CharRangeParser = CharRangeParser('0', '9');

pub struct FirstOf<'a>(pub &'a dyn Parser, pub &'a dyn Parser);

impl<'a> Parser for FirstOf<'a> {
    fn parse<'b>(&self, pointer: InputPointer<'b>) -> Result<Match<'b>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letter_parser_with_letter_is_ok() {
        let parser = LowerCaseLetter;
        let input = String::from("x");
        let pp = InputPointer {
            input: &input,
            pos: 0,
        };
        let result = parser.parse(pp);
        assert!(result.is_ok(), "result was: {:?}", result)
    }

    #[test]
    fn letter_parser_with_garbage_is_not_ok() {
        let parser = LowerCaseLetter;
        let input = String::from("1");
        let pp = InputPointer {
            input: &input,
            pos: 0,
        };
        let result = parser.parse(pp);
        assert!(!result.is_ok(), "result was: {:?}", result)
    }
}
