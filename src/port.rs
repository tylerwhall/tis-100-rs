use std::cell::Cell;
use instruction;

pub trait Port {
    fn read(&mut self) -> Option<i32>;
    fn write(&mut self, val: i32) -> bool;
}

pub trait ReadPort {
    fn read(&self) -> Option<i32>;
}

pub struct CpuWritePorts {
    up:     Cell<Option<i32>>,
    down:   Cell<Option<i32>>,
    left:   Cell<Option<i32>>,
    right:  Cell<Option<i32>>,
    last:   Cell<instruction::Port>,
}

impl CpuWritePorts {
    pub fn new() -> Self {
        CpuWritePorts {
            up:     Default::default(),
            down:   Default::default(),
            left:   Default::default(),
            right:  Default::default(),
            last:   Cell::new(instruction::Port::Up),
        }
    }

    fn get_port(&self, p: instruction::Port) -> &Cell<Option<i32>> {
        match p {
            instruction::Port::Up =>    &self.up,
            instruction::Port::Down =>  &self.down,
            instruction::Port::Left =>  &self.left,
            instruction::Port::Right => &self.right,
            _ => panic!("Invalid port")
        }
    }

    fn set_all(&self, val: Option<i32>) {
        self.up.set(val);
        self.down.set(val);
        self.left.set(val);
        self.right.set(val);
    }

    /// Read from the CPU's output
    ///
    /// Returns None if the port is empty
    fn read(&self, port: instruction::Port) -> Option<i32> {
        let ret = self.get_port(port).get();
        if let Some(_) = ret {
            /* If the read is successful, clear all pending writes from the CPU.
             * This works for the write ANY case, but also works for a write to a
             * specific port because only one pending write is allowed at once */
            self.set_all(None);
            self.last.set(port);
        }
        ret
    }

    /// Returns true if no write is pending
    pub fn write_finished(&self) -> bool {
        match self.up.get()
            .or(self.down.get())
            .or(self.left.get())
            .or(self.right.get()) {
            Some(_) => false,
            None => true,
        }
    }

    pub fn get_last(&self) -> instruction::Port {
        self.last.get()
    }

    pub fn get_read_port(&self, p: instruction::Port) -> CpuWritePortsReader {
        CpuWritePortsReader::new(self, p)
    }

    /// Store from the CPU into the port
    ///
    /// Returns false if the port is full
    /// Accepts a direction or any (not last)
    pub fn write_port(&self, p: instruction::Port, val: i32) {
        assert!(self.write_finished());
        match p {
            instruction::Port::Any => self.set_all(Some(val)),
            _ => self.get_port(p).set(Some(val))
        }
    }
}

pub struct CpuWritePortsReader<'a> {
    ports:  &'a CpuWritePorts,
    active: instruction::Port,
}

impl<'a> CpuWritePortsReader<'a> {
    /// Creates a new object implementing ReadPort for a specific port of CpuWritePorts
    fn new(ports: &'a CpuWritePorts, port: instruction::Port) -> Self {
        CpuWritePortsReader {
            ports:  ports,
            active: port,
        }
    }
}

impl<'a> ReadPort for CpuWritePortsReader<'a> {
    /// Read from the CPU's output
    ///
    /// Returns None if the port is empty
    fn read(&self) -> Option<i32> {
        self.ports.read(self.active)
    }
}

pub struct CpuWritePortsReaders<'a> {
    pub up:     CpuWritePortsReader<'a>,
    pub down:   CpuWritePortsReader<'a>,
    pub left:   CpuWritePortsReader<'a>,
    pub right:  CpuWritePortsReader<'a>,
}

impl<'a> From<&'a CpuWritePorts> for CpuWritePortsReaders<'a> {
    fn from(ports: &'a CpuWritePorts) -> Self {
        CpuWritePortsReaders {
            up:     ports.get_read_port(instruction::Port::Up),
            down:   ports.get_read_port(instruction::Port::Down),
            left:   ports.get_read_port(instruction::Port::Left),
            right:  ports.get_read_port(instruction::Port::Right),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CpuWritePorts, Port, ReadPort};
    use instruction;

    #[test]
    fn test_cpu_write_ports() {
         let ports = CpuWritePorts::new();

         let port = ports.get_read_port(instruction::Port::Up);
         ports.write_port(instruction::Port::Up, 1);
         assert_eq!(ports.up.get(), Some(1));
         assert_eq!(port.read(), Some(1));
         assert_eq!(port.read(), None);
    }
}
