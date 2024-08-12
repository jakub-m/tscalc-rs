use super::core::*;

/// Match the matchers one by one. Succeeds if all the matchers succeed.
pub struct Sequence<'a> {
    matchers: Vec<&'a dyn Matcher>,
}

impl<'a> Matcher for Sequence<'a> {
    fn match_input<'b>(&self, pointer: InputPointer<'b>) -> Result<Match<'b>, String> {
        let mut current_pointer = Some(pointer);
        for i in 0..self.matchers.len() {
            let matcher = self.matchers[i];
            let p = current_pointer.take().unwrap();
            let result = matcher.match_input(p);
            if result.is_ok() {
                let p = result.unwrap();
                current_pointer = Some(InputPointer {
                    input: p.pointer.input,
                    pos: p.pointer.pos,
                });
            } else {
                return Err(result.unwrap_err()); // TODO refactor with match etc
            }
        }
        let p = current_pointer.take().unwrap();
        Ok(Match {
            pointer: p,
            matched: &pointer.input[pointer.pos..p.pos], // TODO refactor, add method.
        })
    }
}

mod tests {
    use super::{InputPointer, Matcher, Sequence};
    use crate::matcher::{Digit, LowerCaseLetter};

    #[test]
    fn test_sequence() {
        let digit = Digit;
        let letter = LowerCaseLetter;

        let sequence = Sequence {
            matchers: vec![&digit, &letter, &digit, &letter],
        };
        let s = String::from("1a2b3c");
        let p = InputPointer::from_string(&s);
        let result = sequence.match_input(p);
        assert!(result.is_ok(), "expected match, got {:?}", result);
        assert_eq!("1a2b", result.unwrap().matched);
    }
}
