pub mod instruction;
pub mod parse;

fn main() {
    let program = parse::parse("TOP:\n NOP\nNOP\nJMP TOP\n").unwrap();
    println!("Program:\n {:?}", program);
}
