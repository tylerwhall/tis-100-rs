use std::str::FromStr;

#[derive(Debug, PartialEq)]
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

pub static BAD_OPCODE_ERR: &'static str = "Bad opcode for # of arguments";
pub static NUM_ARGS_ERR: &'static str = "Wrong number of arguments";
pub static LIT_DST_ERR: &'static str = "Literal not allowed as dst operand";

impl FromStr for Instruction {
    type Err = &'static str;

    fn from_str(insn: &str) -> Result<Instruction, Self::Err> {
        //Nightly: let words: Vec<&str> = insn.split_whitespace().collect();
        let words: Vec<&str> = insn.split(' ').filter(|s| *s != "").collect();

        match words.len() {
            1 => match words[0] {
                "NOP" => Ok(Instruction::NOP),
                "SWP" => Ok(Instruction::SWP),
                "SAV" => Ok(Instruction::SAV),
                "NEG" => Ok(Instruction::NEG),
                _ => Err(BAD_OPCODE_ERR),
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
                _ => Err(BAD_OPCODE_ERR),
            },

            3 => match words[0] {
                "MOV" => Operand::from_str(words[1]).and_then(|s| Operand::from_str(words[2])
                    .and_then(|d| match d {
                        Operand::Lit(_) => Err(LIT_DST_ERR),
                        _ => Ok(Instruction::MOV { src: s, dst: d })
                        })),
                _ => Err(BAD_OPCODE_ERR),
            },

            _ => Err(NUM_ARGS_ERR)
        }
    }
}

#[test]
fn instruction_from_str() {
    fn i(s: &str) -> Instruction {
        Instruction::from_str(s).unwrap()
    }
    assert_eq!(i("NOP"), Instruction::NOP);
    assert_eq!(i("SWP"), Instruction::SWP);
    assert_eq!(i("SAV"), Instruction::SAV);
    assert_eq!(i("NEG"), Instruction::NEG);
    assert_eq!(Instruction::from_str("BOGUS").unwrap_err(), BAD_OPCODE_ERR);

    assert_eq!(i("ADD 10"), Instruction::ADD { addend: Operand::Lit(10) });
    assert_eq!(i("SUB 10"), Instruction::SUB { subtrahend: Operand::Lit(10) });
    assert_eq!(i("JMP LOC"), Instruction::J { cond: Condition::Unconditional,   dst: "LOC".to_string() });
    assert_eq!(i("JEZ LOC"), Instruction::J { cond: Condition::Ez,              dst: "LOC".to_string() });
    assert_eq!(i("JNZ LOC"), Instruction::J { cond: Condition::Nz,              dst: "LOC".to_string() });
    assert_eq!(i("JGZ LOC"), Instruction::J { cond: Condition::Gz,              dst: "LOC".to_string() });
    assert_eq!(i("JLZ LOC"), Instruction::J { cond: Condition::Lz,              dst: "LOC".to_string() });
    assert_eq!(i("JRO 1"), Instruction::JRO { dst: Operand::Lit(1) });
    assert_eq!(Instruction::from_str("JEQ LOC").unwrap_err(), BAD_OPCODE_ERR);

    assert_eq!(i("MOV UP DOWN"),    Instruction::MOV { src: Operand::Port(Port::Up), dst: Operand::Port(Port::Down) });
    assert_eq!(i("MOV  UP  DOWN"),  Instruction::MOV { src: Operand::Port(Port::Up), dst: Operand::Port(Port::Down) });
    assert_eq!(i("MOV UP ACC"),     Instruction::MOV { src: Operand::Port(Port::Up), dst: Operand::ACC });
    assert_eq!(i("MOV ACC ACC"),    Instruction::MOV { src: Operand::ACC, dst: Operand::ACC });
    assert_eq!(Instruction::from_str("MV UP ACC").unwrap_err(), BAD_OPCODE_ERR);
    assert_eq!(Instruction::from_str("MOV UP 10").unwrap_err(), LIT_DST_ERR);

    assert_eq!(Instruction::from_str("1 2 3 4").unwrap_err(), NUM_ARGS_ERR);
}

#[derive(Clone, Debug, PartialEq)]
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

#[test]
fn port_from_str() {
    assert_eq!(Port::from_str("UP").unwrap(),   Port::Up);
    assert_eq!(Port::from_str("DOWN").unwrap(), Port::Down);
    assert_eq!(Port::from_str("LEFT").unwrap(), Port::Left);
    assert_eq!(Port::from_str("RIGHT").unwrap(), Port::Right);
    assert_eq!(Port::from_str("OTHER").unwrap_err(), "bad port");
}

#[derive(Debug, PartialEq)]
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

#[test]
fn operand_from_str() {
    assert_eq!(Operand::from_str("ACC").unwrap(), Operand::ACC);
    assert_eq!(Operand::from_str("32").unwrap(), Operand::Lit(32));
    assert_eq!(Operand::from_str("UP").unwrap(), Operand::Port(Port::Up));
    assert_eq!(Operand::from_str("FOO").unwrap_err(), "Invalid operand");
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    Unconditional,
    Ez,
    Nz,
    Gz,
    Lz,
}

pub type Label = String;
