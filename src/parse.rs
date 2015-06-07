extern crate regex;

use self::regex::Regex;
use std::str::FromStr;
use std::ascii::AsciiExt;
use instruction::{Instruction, Label};

#[derive(Debug, PartialEq)]
pub struct Line {
    label: Option<Label>,
    insn: Option<Instruction>
}

/* Matches an optional label followed by an optional instruction. Whitespace or empty string matches as well */
static LINE_RE: &'static str = r"\s*((?P<label>\S+):)?\s*((?P<insn>\S+.*))?";

impl FromStr for Line {
    type Err = &'static str;

    fn from_str(line: &str) -> Result<Line, Self::Err> {
        let re = Regex::new(LINE_RE).unwrap(); // TODO optimize regex compilation

        match re.captures(&line.to_ascii_uppercase()) {
            Some(caps) => {
                let label = caps.name("label").map(|s| s.to_string());

                /* No insn regex match is ok. Else return Err() from parse_insn() or Ok(Some(Insn)) */
                let insn: Result<Option<Instruction>, Self::Err> = caps.name("insn")
                    .map_or(Ok(None), |s| Instruction::from_str(s).map(|i| Some(i)));

                insn.map(|insn| Line { insn: insn, label: label })
            },
            None => Err("Unparsed line"),
        }
    }
}

#[test]
fn test_parse_line() {
    use instruction;

    fn l(s: &str) -> Result<Line, &'static str> {
        println!("{}", s);
        Line::from_str(s)
    }
    assert_eq!(l("foo: NOP").unwrap(), Line { label: Some("FOO".to_string()), insn: Some(Instruction::NOP) });
    /* Label with : has questionable utility */
    assert_eq!(l("foo:: NOP").unwrap(), Line { label: Some("FOO:".to_string()), insn: Some(Instruction::NOP) });
    assert_eq!(l(" NOP ").unwrap(), Line { label: None, insn: Some(Instruction::NOP) });
    assert_eq!(l("").unwrap(), Line { label: None, insn: None });
    assert_eq!(l("SUB b c").unwrap_err(), instruction::BAD_OPCODE_ERR);
    assert_eq!(l("a b c d").unwrap_err(), instruction::NUM_ARGS_ERR);
}

pub fn parse_program(p: &str) -> Result<Vec<Line>, &'static str> {
    let line_strs: Vec<&str> = p.lines().collect();
    let mut lines = Vec::with_capacity(line_strs.len());

    for line_str in line_strs {
        lines.push(try!(Line::from_str(line_str)));
    }
    Ok(lines)
}
