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
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Duration(chrono::Duration),
    DateTime(chrono::DateTime<chrono::FixedOffset>),
    SignedDateTime(SignedDateTime),
    /// The nodes are guaranteed to be variants Node::Duration.
    Durations(Vec<Node>), // TODO remove, not needed anymore.
    /// A string (e.g. a literal) that was matched and is defacto skipped.
    Skip(String),
    /// A sequence of nodes that form an expression.
    Expr(Vec<Node>),
    /// "now" literal that evaluates to current time.
    Now,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SignedDateTime {
    pub sign: i32,
    pub datetime: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug)]
pub struct ParseOk<'a> {
    pub pointer: InputPointer<'a>,
    pub node: Node,
}

#[derive(Debug)]
pub struct ParseErr<'a> {
    pub pointer: InputPointer<'a>,
    pub message: String,
}

pub trait Parser {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>>;
}
