use crate::helper::{domainMail, domainTyposquat};

pub async fn main(args: Vec<String>) {
    if args.len() < 3 {
        eprintln!("Usage: {} domain <command>", args[0]);
        std::process::exit(1);
    }

    match args[2].as_str() {
        "mail" => {
            if args.len() < 4 {
                eprintln!("Usage: {} domain mail <args>", args[0]);
                std::process::exit(1);
            }
            mail(args[3..].to_vec()).await;
        },
        "typosquat" => {
            if args.len() < 4 {
                eprintln!("Usage: {} domain typosquat <args>", args[0]);
                std::process::exit(1);
            }
            domainTyposquat::generate_list(args[3..].to_vec());
        },
        _ => {
            eprintln!("Unknown NET command: {}", args[1]);
        }
    }
}

async fn mail(args: Vec<String>) {
    // checkw what is the first argument
    if args.is_empty() {
        eprintln!("Usage: mail <subcommand> [args]");
        return;
    }
    match args[0].as_str() {
        "scan" => {
            // Scan for the mail config if arg1 is a domain
            if args.len() < 2 {
                eprintln!("Usage: mail scan <domain>");
                return;
            }
            domainMail::mail_scan(args[1].as_str()).await;
        }
        _ => {
            eprintln!("Unknown mail command: {}", args[0]);
        }
    }
}