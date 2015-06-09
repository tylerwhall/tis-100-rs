use parse::Executable;
use instruction;
use instruction::{Instruction, Condition, Operand};
use port;
use port::GenericPort;

#[derive(Default)]
pub struct CpuState {
    pub acc: i32,
    pub bak: i32,
    pc: i32,
}

pub struct CpuPorts {
    up:     GenericPort,
    down:   GenericPort,
    left:   GenericPort,
    right:  GenericPort,
}

impl CpuPorts {
    fn new() -> Self {
        CpuPorts {
            up:     GenericPort::create(port::CpuPort::new()),
            down:   GenericPort::create(port::CpuPort::new()),
            left:   GenericPort::create(port::CpuPort::new()),
            right:  GenericPort::create(port::CpuPort::new()),
        }
    }

    fn write_port(&mut self, port: instruction::Port, val: i32) -> bool {
        match port {
            instruction::Port::Up =>    &self.up,
            instruction::Port::Down =>  &self.down,
            instruction::Port::Left =>  &self.left,
            instruction::Port::Right => &self.right,
        }.write(val)
    }

    fn read_port(&mut self, port: instruction::Port) -> Option<i32> {
        panic!("Unimplemented port read");
    }
}

pub struct Cpu {
    state:      CpuState,
    outports:   CpuPorts,
    inports:    CpuPorts,
    executable: Executable,
}

fn get_operand(state: &CpuState, ports: &mut CpuPorts, op: &Operand) -> Option<i32> {
    match op {
        &Operand::Lit(i) => Some(i),
        &Operand::ACC => Some(state.acc),
        &Operand::Port(ref p) => ports.read_port(p.to_owned()),
    }
}

impl Cpu {
    pub fn new(executable: Executable) -> Cpu {
        Self::with_ports(CpuPorts::new(), executable)
    }

    pub fn with_ports(out: CpuPorts, executable: Executable) -> Cpu {
        Cpu { state: Default::default(),
              outports: out,
              inports: CpuPorts::new(),
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
                match get_operand(&self.state, &mut self.inports, src) {
                    Some(i) => match dst {
                            &Operand::Lit(_) => panic!("Cannot store to a literal"),
                            &Operand::ACC => { self.state.acc = i; true },
                            &Operand::Port(ref p) => self.outports.write_port(p.to_owned(), i),
                    },
                    None => false
                }
            },
            Instruction::SWP => {
                let tmp = self.state.acc;
                self.state.acc = self.state.bak;
                self.state.bak = tmp;
                true
            },
            Instruction::SAV => {
                self.state.bak = self.state.acc;
                true
            },
            Instruction::ADD { ref addend } => {
                match get_operand(&self.state, &mut self.inports, addend) {
                    Some(i) => { self.state.acc += i; true },
                    None => false
                }
            },
            Instruction::SUB { ref subtrahend } => {
                match get_operand(&self.state, &mut self.inports, subtrahend) {
                    Some(i) => { self.state.acc -= i; true },
                    None => false
                }
            },
            Instruction::NEG => {
                self.state.acc = -self.state.acc;
                true
            },
            Instruction::J { ref cond, ref dst } => {
                if match *cond {
                    Condition::Unconditional => true,
                    Condition::Ez => self.state.acc == 0,
                    Condition::Nz => self.state.acc != 0,
                    Condition::Gz => self.state.acc > 0,
                    Condition::Lz => self.state.acc < 0,
                } {
                    self.state.pc = self.executable.label_line(dst) as i32;
                    false
                } else {
                    true
                }
            },
            Instruction::JRO { ref dst } => {
                match get_operand(&self.state, &mut self.inports, dst) {
                    Some(i) => { self.state.pc += i; true },
                    None => false
                }
            },
        };
        if advance_pc {
            self.state.pc += 1;
        }
        /* Handle wrapping at the end and via JRO */
        self.state.pc %= self.executable.len() as i32;
        if self.state.pc < 0 {
            self.state.pc = self.executable.len() as i32 + self.state.pc;
        }
        true
    }

    pub fn current_line(&self) -> u32 {
        self.executable.srcline_at(self.pc())
    }

    fn pc(&self) -> usize {
        self.state.pc as usize
    }
}

#[cfg(test)]
mod tests {
    use super::{Cpu, CpuPorts};
    use port::Port;
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

    #[test]
    fn test_mov() {
        let e = parse::parse("MOV 10 ACC\nNOP").unwrap();
        let mut cpu = Cpu::new(e);
        assert_eq!(cpu.current_line(), 0);
        assert_eq!(cpu.state.acc, 0);
        cpu.execute();
        assert_eq!(cpu.current_line(), 1);
        assert_eq!(cpu.state.acc, 10);
    }

    #[test]
    fn test_add_sub() {
        let e = parse::parse("ADD 10\nADD -20\nSUB 10\nSUB -30").unwrap();
        let mut cpu = Cpu::new(e);
        assert_eq!(cpu.current_line(), 0);
        assert_eq!(cpu.state.acc, 0);
        cpu.execute();
        assert_eq!(cpu.current_line(), 1);
        assert_eq!(cpu.state.acc, 10);
        cpu.execute();
        assert_eq!(cpu.current_line(), 2);
        assert_eq!(cpu.state.acc, -10);
        cpu.execute();
        assert_eq!(cpu.current_line(), 3);
        assert_eq!(cpu.state.acc, -20);
        cpu.execute();
        assert_eq!(cpu.current_line(), 0);
        assert_eq!(cpu.state.acc, 10);
    }

    #[test]
    fn test_ports() {
        let e = parse::parse("MOV 10 DOWN").unwrap();
        let mut cpu = Cpu::new(e);
        cpu.execute();
        assert_eq!(cpu.outports.down.read().unwrap(), 10);
    }

    #[test]
    fn port_borrow() {
        let e = parse::parse("MOV 10 DOWN").unwrap();
        let ports = CpuPorts::new();
        let down = ports.down.clone();
        let mut cpu = Cpu::with_ports(ports, e);
        cpu.execute();
        assert_eq!(down.read().unwrap(), 10);
    }
}
