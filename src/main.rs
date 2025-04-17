extern crate rex;

use rex::helper::args;

fn main() {
    println!("Hello, world!");

    let a = args::parse_args();

    println!("Raw: {:?}", a);
}
