use super::core::*;

pub struct Collapse<'a> {
    pub parser: &'a dyn Parser,
    pub at_least: Option<u32>,
    pub at_most: Option<u32>,
}

impl<'a> Parser for Collapse<'a> {
    fn parse<'b>(&self, pointer: &'b InputPointer) -> Result<Match<'b>, String> {
        let mut current_pos: usize = pointer.pos;
        loop {
            let current_pointer = pointer.at_pos(current_pos);
            let m = self.parser.parse(&current_pointer);
            if m.is_ok() {
                current_pos = m.unwrap().pointer.pos;
            } else {
                if current_pos == pointer.pos {
                    // Return the error since no parser matched anything.
                    return Err(m.unwrap_err());
                } else {
                    // The parser advanced before the error, so we are good.
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
