extern crate rex;

use rex::helper::args;
use rex::com;

#[tokio::main]
async fn main() {

    let a = args::parse_args();

    match a.command.as_str() {
        "help" => { com::help::show_help() }
        "version" => { com::help::show_version() }
        "update" => { com::help::update() }

        "fswatch" => { com::fswatch::main(a).await }

        "webcrawl" => { com::webcrawl::main(a).await }
        "gallery-dl" => { com::gallery_dl::main(a).await }
    
        _ => { com::help::unknown_command() }
    }
}
