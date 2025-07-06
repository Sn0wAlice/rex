extern crate rex;

use crate::rex::com::{ssl, help, file, net, reg, domain, web};

#[tokio::main]
async fn main() {
    // check what is the args passed to the program
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <command>", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "ssl" => {
            ssl::main(args).await;
        }
        "file" => {
            file::main(args).await;
        }
        "net" => {
            net::main(args).await;
        }
        "reg" => {
            reg::main(args).await;
        }
        "domain" => {
            domain::main(args).await;
        }
        "web" => {
            web::main(args).await;
        }

        "help" => {
            help::main()
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Use 'help' to see available commands.");
            std::process::exit(1);
        }
    }
}
