// Basic matchers.

use super::core::*;

/// Check if the input char is between the two chars specified in the constructor (inclusive).
pub struct CharRange(pub char, pub char);

impl Matcher for CharRange {
    fn match_input<'a>(&self, pointer: InputPointer<'a>) -> Result<Match<'a>, String> {
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
pub const LowerCaseLetter: CharRange = CharRange('a', 'z');
#[allow(non_upper_case_globals)]
pub const Digit: CharRange = CharRange('0', '9');

pub struct FirstOf<'a>(pub &'a dyn Matcher, pub &'a dyn Matcher);

impl<'a> Matcher for FirstOf<'a> {
    fn match_input<'b>(&self, pointer: InputPointer<'b>) -> Result<Match<'b>, String> {
        let a_result = self.0.match_input(pointer);
        if a_result.is_ok() {
            return a_result;
        }
        let b_result = self.1.match_input(pointer);
        if b_result.is_ok() {
            return b_result;
        }
        Err(format!("Nothing matched for: {}", pointer.rest()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letter_matcher_with_letter_is_ok() {
        let matcher = LowerCaseLetter;
        let input = String::from("x");
        let pp = InputPointer {
            input: &input,
            pos: 0,
        };
        let result = matcher.match_input(pp);
        assert!(result.is_ok(), "result was: {:?}", result)
    }

    #[test]
    fn letter_matcher_with_garbage_is_not_ok() {
        let matcher = LowerCaseLetter;
        let input = String::from("1");
        let pp = InputPointer {
            input: &input,
            pos: 0,
        };
        let result = matcher.match_input(pp);
        assert!(!result.is_ok(), "result was: {:?}", result)
    }
}
