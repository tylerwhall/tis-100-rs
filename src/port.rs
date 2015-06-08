trait Port {
    fn read(&mut self) -> Option<i32>;
    fn write(&mut self, val: i32) -> bool;
}

/// The producer, or the output side of a CPU.
#[derive(Default)]
struct CpuPort {
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

#[cfg(test)]
mod tests {
    use super::{CpuPort, Port};

    #[test]
    fn test_cpu_port() {
        let mut port: CpuPort = Default::default();

        assert_eq!(port.read(), None);
        assert_eq!(port.write(1), true);
        assert_eq!(port.write(1), false);
        assert_eq!(port.read(), Some(1));
        assert_eq!(port.read(), None);
    }
}
