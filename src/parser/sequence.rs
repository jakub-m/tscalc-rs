use super::core::*;

/// Match the parsers one by one.
pub struct Sequence<'a> {
    parsers: Vec<&'a dyn Parser>,
}

impl<'a> Parser for Sequence<'a> {
    fn parse<'b>(&self, pointer: InputPointer<'b>) -> Result<Match<'b>, String> {
        let mut current_pointer = Some(pointer);
        for i in 0..self.parsers.len() {
            let parser = self.parsers[i];
            let p = current_pointer.take().unwrap();
            let result = parser.parse(p);
            if result.is_ok() {
                let p = result.unwrap();
                current_pointer = Some(InputPointer {
                    // error, current_pointer already borrowed.
                    input: p.pointer.input,
                    pos: p.pointer.pos,
                });
            }
        }
        todo!()
    }
}

mod tests {
    use super::{InputPointer, Parser, Sequence};
    use crate::parser::{Digit, LowerCaseLetter};

    #[test]
    fn test_sequence() {
        let digit = Digit;
        let letter = LowerCaseLetter;

        let sequence = Sequence {
            parsers: vec![&digit, &letter, &digit, &letter],
        };
        let s = String::from("1a2b3c");
        let p = InputPointer::from_string(&s);
        let result = sequence.parse(p);
        assert!(result.is_ok(), "expected match, got {:?}", result);
        assert_eq!("1a2b", result.unwrap().matched);
    }
}
