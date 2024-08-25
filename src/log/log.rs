// TODO use debug! macro
pub fn debug_nested_log(nesting: usize, _s: String) {
    let space = String::from("| ").repeat(nesting / 2) + (if nesting % 2 == 1 { "|" } else { "" });
    debug_log(format!("{:02}{}{}", nesting, space, _s));
}

pub fn debug_log(_s: String) {
    // println!("{}", _s);
}
