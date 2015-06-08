extern crate tis_100;
use tis_100::parse;

#[allow(dead_code)]
fn main() {
    let program = parse::parse("TOP:\n NOP\nNOP\nJMP TOP\n").unwrap();
    println!("Program:\n {:?}", program);
}
