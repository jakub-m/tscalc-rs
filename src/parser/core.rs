// Core interfaces and structures.

pub trait Parser {
    fn parse<'a>(&self, pointer: &'a InputPointer) -> Result<Match<'a>, String>;
}

/// A context passed around between the parsers, pointing where in the input is the parser now.
#[derive(Debug)]
pub struct InputPointer<'a> {
    /// The input string.
    pub input: &'a String,
    /// Position in the input string.
    pub pos: usize,
}

impl<'a> InputPointer<'a> {
    /// Check if the pointer is at the end of the input.
    pub fn is_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Get the remainder of the input (at pos).
    pub fn rest(&self) -> &str {
        if self.is_end() {
            return &"";
        }
        &self.input[self.pos..]
    }

    /// Advance the pointer by n bytes.
    pub fn advance(&self, n: usize) -> InputPointer<'a> {
        return InputPointer {
            input: self.input,
            pos: self.pos + n,
        };
    }

    /// Peek next N characters.
    pub fn peek_n(&self, offset: usize) -> &str {
        // TODO Add right bound.
        return &self.input[self.pos..self.pos + offset];
    }

    /// Return the pointer with pos set to specific value
    pub fn at_pos(&self, pos: usize) -> InputPointer<'a> {
        let pos = if pos > self.input.len() {
            self.input.len()
        } else {
            pos
        };
        InputPointer {
            input: self.input,
            pos,
        }
    }
}

#[derive(Debug)]
pub struct Match<'a> {
    /// After a successful match, the next unparsed part of the input. The next parser can continue from this pointer.
    pub pointer: InputPointer<'a>,
    /// The characters matched by the parser.
    pub matched: &'a str,
}
