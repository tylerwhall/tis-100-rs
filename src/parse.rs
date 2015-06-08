extern crate regex;

use self::regex::Regex;
use std::str::FromStr;
use std::ascii::AsciiExt;
use std::collections::HashMap;
use instruction::{Instruction, Label};

#[derive(Debug, PartialEq)]
struct Line {
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

fn parse_program(p: &str) -> Result<Vec<Line>, &'static str> {
    let line_strs: Vec<&str> = p.lines().collect();
    let mut lines = Vec::with_capacity(line_strs.len());

    for line_str in line_strs {
        lines.push(try!(Line::from_str(line_str)));
    }
    Ok(lines)
}

#[derive(Debug, PartialEq)]
pub struct InstructionLine {
    insn: Instruction,
    srcline: u32,
}

#[derive(Debug, PartialEq)]
pub struct Executable {
    lines: Vec<InstructionLine>,
    labels: HashMap<Label, u32>,
}

pub fn parse(p: &str) -> Result<Executable, &'static str> {
    let mut lines = try!(parse_program(p));

    let validlines = lines.iter().filter(|l| l.insn != None).count();
    let numlabels = lines.iter().filter(|l| l.label != None).count();
    let mut executable = Executable { lines: Vec::with_capacity(validlines),
                                      labels: HashMap::with_capacity(numlabels) };

    /* Would like a better consuming iterator */
    for i in 0..lines.len() as u32 {
        let l = lines.remove(0);

        if let Some(insn) = l.insn {
            executable.lines.push(InstructionLine { insn: insn, srcline: i });
        }
        if let Some(label) = l.label {
            executable.labels.insert(label, i);
        }
    }
    assert_eq!(executable.lines.len(), validlines);
    assert_eq!(executable.labels.len(), numlabels);

    /* Resolve label pointers from src line to instruction # */
    for (_, lineno) in executable.labels.iter_mut() {
        let mut i = 0;
        for insnline in executable.lines.iter() {
            if insnline.srcline >= *lineno {
                *lineno = i;
                break;
            }
            i += 1;
        }
        assert!(i < executable.lines.len() as u32);
    }

    /* Make sure all JMP labels exist */
    for line in executable.lines.iter() {
        if let Instruction::J { cond: _, ref dst } = line.insn {
            if !executable.labels.contains_key(dst) {
                return Err("Jump to undefined label");
            }
        }
    }

    Ok(executable)
}

#[test]
fn test_parse() {
    let e1 = parse("TOP:\nNOP\nJMP TOP").unwrap();
    let e2 = parse("\nTOP:NOP\nJMP TOP").unwrap();
    assert_eq!(e1.lines.len(), e2.lines.len());
    for (l1, l2) in e1.lines.iter().zip(e2.lines.iter()) {
        assert_eq!(l1.insn, l2.insn);
    }
}
