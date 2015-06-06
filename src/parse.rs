extern crate regex;

use self::regex::Regex;
use std::str::FromStr;
use instruction::{Instruction, Label};

#[derive(Debug)]
pub struct Line {
    label: Option<Label>,
    insn: Option<Instruction>
}

/* Matches an optional label followed by an optional instruction. Whitespace or empty string matches as well */
static LINE_RE: &'static str = r"\s*((?P<label>\S+):)?\s*((?P<insn>\S+.*))?";

pub fn parse_line(line: &str) -> Result<Line, &'static str> {
    let re = Regex::new(LINE_RE).unwrap(); // TODO optimize regex compilation

    match re.captures(line) {
        Some(caps) => {
            let label = caps.name("label").map(|s| s.to_string());

            /* No insn regex match is ok. Else return Err() from parse_insn() or Ok(Some(Insn)) */
            let insn: Result<Option<Instruction>, &'static str> = caps.name("insn")
                .map_or(Ok(None), |s| Instruction::from_str(s).map(|i| Some(i)));

            insn.map(|insn| Line { insn: insn, label: label })
        },
        None => Err("Unparsed line"),
    }
}
