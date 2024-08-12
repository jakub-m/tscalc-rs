// Core interfaces and structures.

pub trait Parser {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<Match<'a>, String>;
}

/// The parser that returns an arbitrary parse node. The parse node is an interpreted match. For
/// example, if a match is a series of digits, the parse node can be a number.
pub trait WithParseNode<T> {
    fn parse_node<'a>(&self, m: &'a Match) -> Option<T>;
}

/// A context passed around between the parsers, pointing where in the input is the parser now.
#[derive(Copy, Clone, Debug)]
pub struct InputPointer<'a> {
    /// The input string.
    pub input: &'a String,
    /// Position in the input string.
    pub pos: usize,
}

impl<'a> InputPointer<'a> {
    pub fn from_string(s: &String) -> InputPointer {
        InputPointer { input: s, pos: 0 }
    }
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
    pub fn peek_n(&self, offset: usize) -> &'a str {
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
