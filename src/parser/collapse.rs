use super::core::*;

/// Concatenate subsequent results of applying a parser into a single match.
pub struct Concat<'a> {
    pub parser: &'a dyn Parser,
    /// The minimum number of expected matches. If there are less than those matches, the parser returns an error.
    pub at_least: Option<u32>,
    /// The maximum number of matches. The parser returns after this number of matches, even if there are more possible matches.
    pub at_most: Option<u32>,
}

impl<'a> Parser for Concat<'a> {
    fn parse<'b>(&self, pointer: &'b InputPointer) -> Result<Match<'b>, String> {
        let mut current_pos: usize = pointer.pos;
        let mut match_count: u32 = 0;
        loop {
            let current_pointer = pointer.at_pos(current_pos);
            let m = self.parser.parse(&current_pointer);
            if m.is_ok() {
                match_count += 1;
                if self.at_most.is_some_and(|upper| match_count >= upper) {
                    // TODO Here a Copy trait could be used.
                    let input = pointer.input;
                    let final_pointer = InputPointer {
                        input,
                        pos: m.as_ref().unwrap().pointer.pos,
                    };
                    let final_match = Match {
                        pointer: final_pointer,
                        matched: &input[pointer.pos..m.unwrap().pointer.pos],
                    };
                    return Ok(final_match);
                }
                current_pos = m.unwrap().pointer.pos;
            } else {
                if current_pos == pointer.pos {
                    // Return the error since no parser matched anything.
                    return Err(m.unwrap_err());
                } else {
                    // The parser advanced before the error, so we are good.
                    if self.at_least.is_some_and(|lower| match_count < lower) {
                        return Err(format!("expected at least {:?} matches", self.at_least));
                    }
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

mod tests {
    use super::super::basic::*;
    use super::*;

    #[test]
    fn test_concat_variants() {
        test_concat("1234", Some(2), Some(3), Some("123"));
        test_concat("1234", Some(2), Some(5), Some("1234"));
        test_concat("1234", Some(6), Some(11), None);
        test_concat("1234", Some(4), Some(4), Some("1234"));
        test_concat("1234", None, Some(4), Some("1234"));
        test_concat("1234", Some(1), None, Some("1234"));
        test_concat("1234", None, None, Some("1234"));
        test_concat("123x", None, None, Some("123"));
        test_concat("12xx", Some(3), None, None);
    }

    fn test_concat(
        input: &str,
        at_least: Option<u32>,
        at_most: Option<u32>,
        expected_match: Option<&str>,
    ) {
        let parser = Concat {
            parser: &Digit,
            at_least,
            at_most,
        };
        let pointer = InputPointer {
            input: &String::from(input),
            pos: 0,
        };
        let result = parser.parse(&pointer);
        match expected_match {
            Some(expected_match) => {
                assert!(
                    result.is_ok(),
                    "expected match for input {}, but didn't get one",
                    input
                );
                let actual_match = result.as_ref().unwrap().matched;
                assert_eq!(
                    expected_match, actual_match,
                    "expected match {} but got {}",
                    expected_match, actual_match
                );
            }
            None => {
                assert!(
                    !result.is_ok(),
                    "expected to fail for input {} but got match {}",
                    input,
                    result.unwrap().matched
                );
            }
        }
    }
}
