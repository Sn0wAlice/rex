pub async fn main(args: Vec<String>) {
    if args.len() < 3 {
        eprintln!("Usage: {} web <command>", args[0]);
        std::process::exit(1);
    }

    match args[2].as_str() {
        "react" => {
            if args.len() < 4 {
                eprintln!("Usage: {} web react <args>", args[0]);
                std::process::exit(1);
            }

        }
        _ => {
            eprintln!("Unknown NET command: {}", args[1]);
        }
    }
}

