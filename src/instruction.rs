use std::str::FromStr;

#[derive(Debug)]
pub enum Instruction {
    NOP,
    MOV { src: Operand, dst: Operand },
    SWP,
    SAV,
    ADD { addend: Operand },
    SUB { subtrahend: Operand} ,
    NEG,
    J { cond: Condition, dst: Label},
    JRO { dst: Operand },
}

impl FromStr for Instruction {
    type Err = &'static str;

    fn from_str(insn: &str) -> Result<Instruction, Self::Err> {
        let words: Vec<&str> = insn.split(' ').collect();

        match words.len() {
            1 => match words[0] {
                "NOP" => Ok(Instruction::NOP),
                "SWP" => Ok(Instruction::SWP),
                "SAV" => Ok(Instruction::SAV),
                "NEG" => Ok(Instruction::NEG),
                _ => Err("Bad opcode for # of arguments"),
            },

            2 => match words[0] {
                "ADD" => Operand::from_str(words[1]).map(|o| Instruction::ADD { addend: o }),
                "SUB" => Operand::from_str(words[1]).map(|o| Instruction::SUB { subtrahend: o }),
                "JMP" => Ok(Instruction::J { cond: Condition::Unconditional, dst: words[1].to_string() }),
                "JEZ" => Ok(Instruction::J { cond: Condition::Ez,            dst: words[1].to_string() }),
                "JNZ" => Ok(Instruction::J { cond: Condition::Nz,            dst: words[1].to_string() }),
                "JGZ" => Ok(Instruction::J { cond: Condition::Gz,            dst: words[1].to_string() }),
                "JLZ" => Ok(Instruction::J { cond: Condition::Lz,            dst: words[1].to_string() }),
                "JRO" => Operand::from_str(words[1]).map(|o| Instruction::JRO { dst: o }),
                _ => Err("Bad opcode for # of arguments"),
            },

            3 => match words[0] {
                "MOV" => Operand::from_str(words[1]).and_then(|s| Operand::from_str(words[2]).map(|d| (s, d)))
                            .map(|(s, d)| Instruction::MOV { src: s, dst: d }),
                _ => Err("Bad opcode for # of arguments"),
            },

            _ => Err("number of args")
        }
    }
}

#[derive(Debug)]
pub enum Port {
    Up,
    Down,
    Left,
    Right,
}

impl FromStr for Port {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UP" => Ok(Port::Up),
            "DOWN" => Ok(Port::Down),
            "LEFT" => Ok(Port::Left),
            "RIGHT" => Ok(Port::Right),
            _ => Err("bad port"),
        }
    }
}

#[derive(Debug)]
pub enum Operand {
    Lit(i32),
    Port(Port),
    ACC,
}

impl FromStr for Operand {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "ACC" {
            return Ok(Operand::ACC);
        }

        let as_int = i32::from_str(s);
        if as_int.is_ok() {
            return Ok(Operand::Lit(as_int.unwrap()));
        }

        let as_port = Port::from_str(s);
        if as_port.is_ok() {
            return Ok(Operand::Port(as_port.unwrap()));
        }

        Err("Invalid operand")
    }
}

#[derive(Debug)]
pub enum Condition {
    Unconditional,
    Ez,
    Nz,
    Gz,
    Lz,
}

pub type Label = String;
