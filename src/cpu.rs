use parse::Executable;
use instruction;
use instruction::{Instruction, Condition, Operand};
use port::{CpuWritePorts, ReadPort};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExecState {
    EXEC,
    READ(instruction::Port),
    WRITE(instruction::Port),
}

impl Default for ExecState {
    fn default() -> Self {
        ExecState::EXEC
    }
}

#[derive(Default)]
pub struct CpuState {
    pub acc:        i32,
    pub bak:        i32,
    pc:             i32,
    pending_write:  Option<(instruction::Port, i32)>,
    exec_state:     ExecState,
}

pub struct CpuReadPorts<'a> {
    up:     &'a ReadPort,
    down:   &'a ReadPort,
    left:   &'a ReadPort,
    right:  &'a ReadPort,
}

impl<'a> CpuReadPorts<'a> {
    /// Index ports structure by instruction port enum
    fn get_port(&self, p: instruction::Port) -> &ReadPort {
        match p {
            instruction::Port::Up =>    self.up,
            instruction::Port::Down =>  self.down,
            instruction::Port::Left =>  self.left,
            instruction::Port::Right => self.right,
            _ => panic!("Invalid port")
        }
    }

    fn read_port(&mut self, port: instruction::Port, last: &mut instruction::Port) -> Option<i32> {
        match port {
            instruction::Port::Any => {
                let mut ret = None;
                for port in [instruction::Port::Up,
                             instruction::Port::Down,
                             instruction::Port::Left,
                             instruction::Port::Right].iter() {
                    ret = self.get_port(*port).read();
                    if let Some(_) = ret {
                        *last = *port;
                        break;
                    }
                }
                ret
            },
            instruction::Port::Last => self.get_port(*last).read(),
            _ => {
                self.get_port(port).read()
            }
        }
    }
}

struct CpuPorts<'a> {
    outports:   &'a CpuWritePorts,
    inports:    CpuReadPorts<'a>,
    last:       instruction::Port,
}

impl<'a> CpuPorts<'a> {
    fn read_port(&mut self, port: instruction::Port) -> Option<i32> {
        self.inports.read_port(port, &mut self.last)
    }

    fn write_port(&mut self, port: instruction::Port, val: i32) {
        self.outports.write_port(match port {
            instruction::Port::Last => self.last,
            _ => port,
        }, val)
    }

    fn write_finished(&mut self, port: instruction::Port) -> bool {
        let finished = self.outports.write_finished();
        if finished && port == instruction::Port::Any {
            self.last = self.outports.get_last();
        }
        finished
    }
}

pub struct Cpu<'a> {
    state:      CpuState,
    ports:      CpuPorts<'a>,
    executable: Executable,
}

fn get_operand(state: &mut CpuState, ports: &mut CpuPorts, op: &Operand) -> Option<i32> {
    match op {
        &Operand::Lit(i) => Some(i),
        &Operand::ACC => Some(state.acc),
        &Operand::Port(p) => {
            let val = ports.read_port(p);
            if val == None {
                state.exec_state = ExecState::READ(p)
            } else {
                state.exec_state = ExecState::EXEC
            }
            val
        }
    }
}

impl<'a> Cpu<'a> {
    pub fn new(executable: Executable, write_ports: &'a CpuWritePorts, read_ports: CpuReadPorts<'a>) -> Cpu<'a>{
        let ports = CpuPorts {
            outports:   write_ports,
            inports:    read_ports,
            last:       instruction::Port::Up,
        };
        Cpu {
            state: Default::default(),
            ports: ports,
            executable: executable,
        }
    }

    pub fn execute(&mut self) -> bool {
        if self.executable.len() == 0 {
            return false;
        }

        // write_cycle() must be called between invocations of execute()
        assert_eq!(self.state.pending_write, None);

        if let ExecState::WRITE(_) = self.exec_state() {
            return false;
        }

        let advance_pc = match *self.executable.insn_at(self.pc()) {
            Instruction::NOP => true,
            Instruction::MOV { ref src, ref dst } => {
                match get_operand(&mut self.state, &mut self.ports, src) {
                    Some(i) => match dst {
                            &Operand::Lit(_) => panic!("Cannot store to a literal"),
                            &Operand::ACC => { self.state.acc = i; true },
                            &Operand::Port(p) => { self.state.pending_write = Some((p, i)); false },
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
                match get_operand(&mut self.state, &mut self.ports, addend) {
                    Some(i) => { self.state.acc += i; true },
                    None => false
                }
            },
            Instruction::SUB { ref subtrahend } => {
                match get_operand(&mut self.state, &mut self.ports, subtrahend) {
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
                match get_operand(&mut self.state, &mut self.ports, dst) {
                    Some(i) => { self.state.pc += i; true },
                    None => false
                }
            },
        };
        self.update_pc(advance_pc);
        true
    }

    /// Processes writes from the last instruction executed
    ///
    /// This must be called after execute for each call to execute. The
    /// write phase is separate from the execute phase to prevent reads and
    /// writes between multiple CPUs from being dependent on the order in which
    /// the CPUs are processed.
    pub fn write_cycle(&mut self) {
        if let Some((port, val)) = self.state.pending_write {
            // This must succeed. Failure means trying to write while a write
            // is already pending. CPU execution state should prevent that.
            self.state.pending_write = None;
            self.ports.write_port(port, val);
            self.state.exec_state = ExecState::WRITE(port);
        } else if let ExecState::WRITE(port) = self.state.exec_state {
            // Check for write completion to advance pc
            if self.ports.write_finished(port) {
                self.state.exec_state = ExecState::EXEC;
                self.update_pc(true);
            }
        }
    }

    pub fn current_line(&self) -> u32 {
        self.executable.srcline_at(self.pc())
    }

    pub fn exec_state(&self) -> ExecState {
        self.state.exec_state
    }

    fn update_pc(&mut self, advance: bool) {
        if advance {
            self.state.pc += 1;
        }
        /* Handle wrapping at the end and via JRO */
        self.state.pc %= self.executable.len() as i32;
        if self.state.pc < 0 {
            self.state.pc = self.executable.len() as i32 + self.state.pc;
        }
    }

    fn pc(&self) -> usize {
        self.state.pc as usize
    }
}

#[cfg(test)]
mod tests {
    use super::{Cpu, CpuReadPorts, ExecState};
    use instruction;
    use port::{CpuWritePorts, CpuWritePortsReaders, Port, ReadPort};
    use parse;

    #[derive(Default)]
    struct DummyReadPort;

    impl ReadPort for DummyReadPort {
        fn read(&self) -> Option<i32> {
            None
        }
    }

    #[derive(Default)]
    struct DummyReadPorts {
        port:    DummyReadPort,
    }

    impl DummyReadPorts {
        fn new() -> Self {
            Default::default()
        }

        fn cpuports(&self) -> CpuReadPorts {
            CpuReadPorts {
                up      : &self.port as &ReadPort,
                down    : &self.port as &ReadPort,
                left    : &self.port as &ReadPort,
                right   : &self.port as &ReadPort,
            }
        }
    }

    #[test]
    fn test_cpu_wrapping() {
        let e = parse::parse("TOP: NOP\nNOP").unwrap();
        let ports = CpuWritePorts::new();
        let rports = DummyReadPorts::new();
        let mut cpu = Cpu::new(e, &ports, rports.cpuports());
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
        let ports = CpuWritePorts::new();
        let rports = DummyReadPorts::new();
        let mut cpu = Cpu::new(e, &ports, rports.cpuports());
        assert_eq!(cpu.current_line(), 0);
        assert_eq!(cpu.state.acc, 0);
        cpu.execute();
        assert_eq!(cpu.current_line(), 1);
        assert_eq!(cpu.state.acc, 10);
    }

    #[test]
    fn test_add_sub() {
        let e = parse::parse("ADD 10\nADD -20\nSUB 10\nSUB -30").unwrap();
        let ports = CpuWritePorts::new();
        let rports = DummyReadPorts::new();
        let mut cpu = Cpu::new(e, &ports, rports.cpuports());
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
    fn test_port_write() {
        let e = parse::parse("MOV 10 DOWN\nNOP").unwrap();
        let ports = CpuWritePorts::new();
        let rports = DummyReadPorts::new();
        let mut cpu = Cpu::new(e, &ports, rports.cpuports());

        // First interation. Make sure writes appear immediately after the first write_cycle()
        cpu.execute();
        assert_eq!(cpu.ports.outports.get_read_port(instruction::Port::Down).read(), None);
        cpu.write_cycle();
        assert_eq!(cpu.current_line(), 0);
        assert_eq!(cpu.exec_state(), ExecState::WRITE(instruction::Port::Down));
        assert_eq!(cpu.ports.outports.get_read_port(instruction::Port::Down).read().unwrap(), 10);
        assert_eq!(cpu.exec_state(), ExecState::WRITE(instruction::Port::Down));

        cpu.execute();
        assert_eq!(cpu.current_line(), 0);
        assert_eq!(cpu.exec_state(), ExecState::WRITE(instruction::Port::Down));
        cpu.write_cycle();
        assert_eq!(cpu.exec_state(), ExecState::EXEC);
        assert_eq!(cpu.current_line(), 1);

        // NOP
        cpu.execute();
        cpu.write_cycle();
        assert_eq!(cpu.current_line(), 0);

        // Second interation. Make sure writes block
        cpu.execute();
        cpu.write_cycle();
        assert_eq!(cpu.current_line(), 0);
        assert_eq!(cpu.exec_state(), ExecState::WRITE(instruction::Port::Down));
        cpu.execute();
        cpu.write_cycle();
        assert_eq!(cpu.current_line(), 0);
        assert_eq!(cpu.exec_state(), ExecState::WRITE(instruction::Port::Down));
        cpu.execute();
        cpu.write_cycle();
        assert_eq!(cpu.ports.outports.get_read_port(instruction::Port::Down).read().unwrap(), 10);
        cpu.execute();
        cpu.write_cycle();
        assert_eq!(cpu.exec_state(), ExecState::EXEC);
        assert_eq!(cpu.current_line(), 1);
    }

    /// Ensures write ANY can be read from any output, but only one output
    #[test]
    fn test_write_any_last() {
        let e = parse::parse("MOV 10 ANY\n
                              MOV 20 ANY\n
                              MOV 30 ANY\n
                              MOV 40 ANY\n
                              MOV 50 LAST").unwrap();
        let ports = CpuWritePorts::new();
        let rports = DummyReadPorts::new();
        let mut cpu = Cpu::new(e, &ports, rports.cpuports());

        cpu.execute();
        cpu.write_cycle();
        assert_eq!(ports.get_read_port(instruction::Port::Up).read().unwrap(), 10);
        assert_eq!(ports.get_read_port(instruction::Port::Up).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Down).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Left).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Right).read(), None);
        cpu.execute();
        cpu.write_cycle();

        cpu.execute();
        cpu.write_cycle();
        assert_eq!(ports.get_read_port(instruction::Port::Down).read().unwrap(), 20);
        assert_eq!(ports.get_read_port(instruction::Port::Up).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Down).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Left).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Right).read(), None);
        cpu.execute();
        cpu.write_cycle();

        cpu.execute();
        cpu.write_cycle();
        assert_eq!(ports.get_read_port(instruction::Port::Left).read().unwrap(), 30);
        assert_eq!(ports.get_read_port(instruction::Port::Up).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Down).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Left).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Right).read(), None);
        cpu.execute();
        cpu.write_cycle();

        cpu.execute();
        cpu.write_cycle();
        assert_eq!(ports.get_read_port(instruction::Port::Right).read().unwrap(), 40);
        assert_eq!(ports.get_read_port(instruction::Port::Up).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Down).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Left).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Right).read(), None);
        cpu.execute();
        cpu.write_cycle();

        // Last
        cpu.execute();
        cpu.write_cycle();
        assert_eq!(ports.get_read_port(instruction::Port::Up).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Down).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Left).read(), None);
        assert_eq!(ports.get_read_port(instruction::Port::Right).read().unwrap(), 50);
        assert_eq!(ports.get_read_port(instruction::Port::Right).read(), None);
    }

    #[test]
    fn port_borrow() {
        let e = parse::parse("MOV 10 DOWN").unwrap();
        let ports = CpuWritePorts::new();
        let rports = DummyReadPorts::new();
        let down = ports.get_read_port(instruction::Port::Down);
        let mut cpu = Cpu::new(e, &ports, rports.cpuports());
        cpu.execute();
        cpu.write_cycle();
        assert_eq!(down.read().unwrap(), 10);
    }

    #[test]
    fn blocking_read() {
        let e = parse::parse("MOV UP DOWN\nMOV DOWN ACC").unwrap();
        let ports = CpuWritePorts::new();
        let inports = CpuWritePorts::new();
        let rports = CpuWritePortsReaders::from(&inports);
        let rports = CpuReadPorts {
            up:     &rports.up,
            down:   &rports.down,
            left:   &rports.left,
            right:  &rports.right,
        };

        let mut cpu = Cpu::new(e, &ports, rports);

        // READ -> WRITE -> EXEC state
        assert_eq!(cpu.exec_state(), ExecState::EXEC);
        cpu.execute();
        cpu.write_cycle();
        assert_eq!(cpu.exec_state(), ExecState::READ(instruction::Port::Up));
        inports.write_port(instruction::Port::Up, 10);
        assert_eq!(ports.get_read_port(instruction::Port::Down).read(), None);

        cpu.execute();
        cpu.write_cycle();
        assert_eq!(ports.get_read_port(instruction::Port::Down).read().unwrap(), 10);
        assert_eq!(cpu.exec_state(), ExecState::WRITE(instruction::Port::Down));

        cpu.execute();
        cpu.write_cycle();
        assert_eq!(ports.get_read_port(instruction::Port::Down).read(), None);
        assert_eq!(cpu.exec_state(), ExecState::EXEC);

        // READ -> EXEC state
        cpu.execute();
        cpu.write_cycle();
        assert_eq!(cpu.exec_state(), ExecState::READ(instruction::Port::Down));
        inports.write_port(instruction::Port::Down, 20);

        cpu.execute();
        cpu.write_cycle();
        assert_eq!(cpu.exec_state(), ExecState::EXEC);
        assert_eq!(cpu.state.acc, 20);
    }
}
