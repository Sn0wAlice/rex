extern crate rex;

use rex::helper::args;
use rex::com;

fn main() {

    let a = args::parse_args();

    match a.command.as_str() {
        "help" => { com::help::show_help() }
        _ => { com::help::unknown_command() }
    }
}
