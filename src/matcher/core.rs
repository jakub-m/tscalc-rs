// Core interfaces and structures.

pub trait Matcher {
    fn match_input<'a>(&self, pointer: InputPointer<'a>) -> Result<Match<'a>, String>;
}

/// A context passed around between the matchers, pointing where in the input is the matched now.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InputPointer<'a> {
    /// The input string.
    pub input: &'a String,
    /// Position in the input string.
    pub pos: usize,
}

impl<'a> InputPointer<'a> {
    pub fn from_string(s: &String) -> InputPointer {
        // TODO deprecate
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
    /// After a successful match, the next unmatched part of the input. The next matcher can continue from this pointer.
    pub pointer: InputPointer<'a>,
    /// The characters matched by the matcher.
    pub matched: &'a str,
}
