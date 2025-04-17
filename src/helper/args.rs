use std::collections::HashMap;

#[derive(Debug)]
pub struct Args {
    pub raw: String,
    pub command: String,
    pub array: Vec<String>,
    pub tabled: HashMap<String, String>
}


pub fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().into_iter().skip(1).collect();

    let t = args.iter()
        .filter(|arg| arg.starts_with("--"))
        .map(|arg| {
            let mut split = arg.splitn(2, '=');
            let key = split.next().unwrap().trim_start_matches("--").to_string();
            let value = split.next().unwrap_or("").to_string();
            (key, value)
        })
        .collect::<HashMap<String, String>>();


    return Args {
        raw: args.join(" "),
        command: args.get(0).unwrap_or(&"".to_string()).to_string(),
        array: args,
        tabled: t
    };
}