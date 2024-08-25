use std::fmt;
use std::rc::Rc;

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

impl<'a> fmt::Display for InputPointer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //let rest = self.rest();
        //let len_to_take = self
        //    .rest()
        //    .char_indices()
        //    .take_while(|(i, _)| *i < 64)
        //    .map(|(i, _)| i)
        //    .max()
        //    .unwrap_or(rest.len());
        //write!(f, "{}", &rest[..len_to_take])
        write!(f, "'{}'", &self.rest())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Duration(chrono::Duration),
    DateTime(chrono::DateTime<chrono::FixedOffset>),
    /// A sequence of nodes that form an expression and can be evaluated.
    Expr(Vec<Node>),
    OperNode {
        oper: Oper,
        node: Rc<Node>,
    },
    /// "now" literal that evaluates to current time.
    Now,
    /// A literal string, e.g. whitespace to skip or function name.
    Literal {
        literal: String,
        skip: bool,
    },
    /// Function with arity of 1
    FuncAry1 {
        /// Name of the function
        name: String,
        arg1: Rc<Node>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Oper {
    Plus,
    Minus,
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

pub trait DisplayParseResult {
    fn to_string(&self) -> String;
}

impl DisplayParseResult for Result<ParseOk<'_>, ParseErr<'_>> {
    fn to_string(&self) -> String {
        match self {
            Ok(parse_ok) => format!("ParseOk({:?}, {})", parse_ok.node, parse_ok.pointer),
            Err(parse_err) => format!("ParseErr({}, {})", parse_err.message, parse_err.pointer),
        }
    }
}

pub trait Parser {
    fn parse<'a>(&self, pointer: InputPointer<'a>) -> Result<ParseOk<'a>, ParseErr<'a>>;
}
