use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Cell;
use instruction;

pub trait Port {
    fn read(&mut self) -> Option<i32>;
    fn write(&mut self, val: i32) -> bool;
}

pub trait ReadPort {
    fn read(&self) -> Option<i32>;
}

#[derive(Default)]
pub struct CpuWritePorts {
    up:     Cell<Option<i32>>,
    down:   Cell<Option<i32>>,
    left:   Cell<Option<i32>>,
    right:  Cell<Option<i32>>,
}

impl CpuWritePorts {
    pub fn new() -> Self {
        Default::default()
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

    pub fn get_read_port(&self, p: instruction::Port) -> &ReadPort {
        self.get_port(p)
    }

    /// Store from the CPU into the port
    ///
    /// Returns false if the port is full
    pub fn write_port(&self, p: instruction::Port, val: i32) -> bool {
        let port = self.get_port(p);
        if port.get() == None {
            port.set(Some(val));
            true
        } else {
            false
        }
    }
}

impl<'a> ReadPort for Cell<Option<i32>> {
    /// Read from the CPU's output
    ///
    /// Returns None if the port is empty
    fn read(&self) -> Option<i32> {
        // May return None if val is already None
        let tmp = self.get();
        self.set(None);
        tmp
    }
}

/// The producer, or the output side of a CPU.
#[derive(Default)]
pub struct CpuPort {
    val: Option<i32>
}

impl Port for CpuPort {
    /// Read from the CPU's output
    ///
    /// Returns None if the port is empty
    fn read(&mut self) -> Option<i32> {
        // May return None if val is already None
        let tmp = self.val;
        self.val = None;
        tmp
    }

    /// Store from the CPU into the port
    ///
    /// Returns false if the port is full
    fn write(&mut self, val: i32) -> bool {
        if self.val == None {
            self.val = Some(val);
            true
        } else {
            false
        }
    }
}

impl CpuPort {
    pub fn new() -> Self {
        Default::default()
    }
}

pub struct GenericPort(Rc<RefCell<Box<Port>>>);

impl GenericPort {
    pub fn create<T: Port + 'static>(x: T) -> GenericPort {
        GenericPort(Rc::new(RefCell::new(Box::new(x) as Box<Port>)))
    }

    pub fn read(&self) -> Option<i32> {
        self.0.borrow_mut().read()
    }

    pub fn write(&self, val: i32) -> bool {
        self.0.borrow_mut().write(val)
    }

    pub fn clone(&self) -> GenericPort {
        GenericPort(self.0.clone())
    }
}


#[cfg(test)]
mod tests {
    use super::{CpuPort, CpuWritePorts, Port};
    use instruction;

    #[test]
    fn test_cpu_port() {
        let mut port: CpuPort = Default::default();

        assert_eq!(port.read(), None);
        assert_eq!(port.write(1), true);
        assert_eq!(port.write(1), false);
        assert_eq!(port.read(), Some(1));
        assert_eq!(port.read(), None);
    }

    #[test]
    fn test_cpu_write_ports() {
         let ports: CpuWritePorts = Default::default();

         let port = ports.get_read_port(instruction::Port::Up);
         ports.up.set(Some(1));
         assert_eq!(ports.up.get(), Some(1));
         assert_eq!(port.read(), Some(1));
    }
}
