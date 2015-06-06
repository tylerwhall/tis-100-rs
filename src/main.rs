pub mod instruction;
pub mod parse;

use instruction::{Instruction, Port, Operand};

fn main() {
    let y = Instruction::MOV {src : Operand::Port(Port::Left), dst : Operand::ACC };
    //let y = Instruction::NOP;
    println!("Hello, world! {:?}", y);
    println!("{:?}", parse::parse_line("foo: NOP"));
    println!("{:?}", parse::parse_line("foo:: NOP"));
    println!("{:?}", parse::parse_line(""));
    println!("{:?}", parse::parse_line("SUB b c"));
    println!("{:?}", parse::parse_line("a b c d"));
}
