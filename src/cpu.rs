use parse::Executable;
use instruction;
use instruction::{Instruction, Condition, Operand};
use port;
use port::{CpuWritePorts, GenericPort};

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
    last:   instruction::Port,
}

impl CpuPorts {
    fn new() -> Self {
        CpuPorts {
            up:     GenericPort::create(port::CpuPort::new()),
            down:   GenericPort::create(port::CpuPort::new()),
            left:   GenericPort::create(port::CpuPort::new()),
            right:  GenericPort::create(port::CpuPort::new()),
            last:   instruction::Port::Up,
        }
    }

    /// Index ports structure by instruction port enum
    fn match_port(&mut self, port: instruction::Port) -> &mut GenericPort {
        match port {
            instruction::Port::Up =>    &mut self.up,
            instruction::Port::Down =>  &mut self.down,
            instruction::Port::Left =>  &mut self.left,
            instruction::Port::Right => &mut self.right,
            _ => panic!("Unimplemented port")
        }
    }

    fn read_port(&mut self, port: instruction::Port) -> Option<i32> {
        let last = self.last.to_owned();

        match port {
            instruction::Port::Any => {
                let mut ret = None;
                for port in [instruction::Port::Up,
                             instruction::Port::Down,
                             instruction::Port::Left,
                             instruction::Port::Right].iter() {
                    ret = self.match_port(port.to_owned()).read();
                    if let Some(_) = ret {
                        self.last = port.to_owned();
                        break;
                    }
                }
                ret
            },
            instruction::Port::Last => self.match_port(last).read(),
            _ => {
                self.last = port.to_owned();
                self.match_port(port).read()
            }
        }
    }
}

pub struct Cpu<'a> {
    state:      CpuState,
    outports:   &'a CpuWritePorts,
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

impl<'a> Cpu<'a> {
    pub fn new(executable: Executable, ports: &'a CpuWritePorts) -> Cpu<'a>{
        let cpu = Cpu {
            state: Default::default(),
            outports: ports,
            inports: CpuPorts::new(),
            executable: executable,
        };
        cpu
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
    use super::Cpu;
    use instruction;
    use port::{CpuWritePorts, Port, ReadPort};
    use parse;

    #[test]
    fn test_cpu_wrapping() {
        let e = parse::parse("TOP: NOP\nNOP").unwrap();
        let ports: CpuWritePorts = Default::default();
        let mut cpu = Cpu::new(e, &ports);
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
        let ports: CpuWritePorts = Default::default();
        let mut cpu = Cpu::new(e, &ports);
        assert_eq!(cpu.current_line(), 0);
        assert_eq!(cpu.state.acc, 0);
        cpu.execute();
        assert_eq!(cpu.current_line(), 1);
        assert_eq!(cpu.state.acc, 10);
    }

    #[test]
    fn test_add_sub() {
        let e = parse::parse("ADD 10\nADD -20\nSUB 10\nSUB -30").unwrap();
        let ports: CpuWritePorts = Default::default();
        let mut cpu = Cpu::new(e, &ports);
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
        let ports: CpuWritePorts = Default::default();
        let mut cpu = Cpu::new(e, &ports);
        cpu.execute();
        assert_eq!(cpu.outports.get_read_port(instruction::Port::Down).read().unwrap(), 10);
    }

    #[test]
    fn port_borrow() {
        let e = parse::parse("MOV 10 DOWN").unwrap();
        let ports = CpuWritePorts::new();
        let down = ports.get_read_port(instruction::Port::Down);
        let mut cpu = Cpu::new(e, &ports);
        cpu.execute();
        assert_eq!(down.read().unwrap(), 10);
    }
}
