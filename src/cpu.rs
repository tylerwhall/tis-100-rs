use parse::Executable;
use instruction::{Instruction, Condition};

struct Cpu {
    pc: i32,
    pub acc: i32,
    pub bak: i32,
    executable: Executable
}

impl Cpu {
    pub fn new(executable: Executable) -> Cpu {
        Cpu { pc: 0,
              acc: 0,
              bak: 0,
              executable: executable,
        }
    }

    pub fn execute(&mut self) -> bool {
        if self.executable.len() == 0 {
            return false;
        }

        let advance_pc = match *self.executable.insn_at(self.pc()) {
            Instruction::NOP => true,
            Instruction::MOV { ref src, ref dst } => {
                panic!("Unimplemented");
                false
            },
            Instruction::SWP => {
                let tmp = self.acc;
                self.acc = self.bak;
                self.bak = tmp;
                true
            },
            Instruction::SAV => {
                self.bak = self.acc;
                true
            },
            Instruction::ADD { ref addend } => {
                panic!("Unimplemented");
                false
            },
            Instruction::SUB { ref subtrahend } => {
                panic!("Unimplemented");
                false
            },
            Instruction::NEG => {
                self.acc = -self.acc;
                true
            },
            Instruction::J { ref cond, ref dst } => {
                if match *cond {
                    Condition::Unconditional => true,
                    Condition::Ez => self.acc == 0,
                    Condition::Nz => self.acc != 0,
                    Condition::Gz => self.acc > 0,
                    Condition::Lz => self.acc < 0,
                } {
                    self.pc = self.executable.label_line(dst) as i32;
                    false
                } else {
                    true
                }
            },
            Instruction::JRO { ref dst } => {
                panic!("Unimplemented");
                false
            },
        };
        if advance_pc {
            self.pc += 1;
        }
        /* Handle wrapping at the end and via JRO */
        self.pc %= self.executable.len() as i32;
        if self.pc < 0 {
            self.pc = self.executable.len() as i32 + self.pc;
        }
        true
    }

    pub fn current_line(&self) -> u32 {
        self.executable.srcline_at(self.pc())
    }

    fn read_port(&self) -> Option<i32> {
        Some(0)
    }

    fn pc(&self) -> usize {
        self.pc as usize
    }
}

#[cfg(test)]
mod tests {
    use super::Cpu;
    use parse;

    #[test]
    fn test_cpu_wrapping() {
        let e = parse::parse("TOP: NOP\nNOP").unwrap();
        let mut cpu = Cpu::new(e);
        assert_eq!(cpu.current_line(), 0);
        cpu.execute();
        assert_eq!(cpu.current_line(), 1);
        cpu.execute();
        assert_eq!(cpu.current_line(), 0);
        cpu.execute();
        assert_eq!(cpu.current_line(), 1);
        cpu.execute();
    }
}
