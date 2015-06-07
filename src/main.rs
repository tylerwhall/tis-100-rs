pub mod instruction;
pub mod parse;

fn main() {
    let program = parse::parse_program("TOP: NOP\nNOP\nJMP TOP").unwrap();
    println!("Program:\n {:?}", program);
}
