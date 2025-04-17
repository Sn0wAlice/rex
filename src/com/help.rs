pub fn show_help() {
let help_text = r#"

           __    Hello i'am Rex 
          / _) /
   .-^^^-/ /
  /       /
<__.|_|-|_|


rex - A simple command line tool for, euuuu, 
       ... for what exactly?
       ... cybersecurity?
       ... pentesting?

Never mind, just use it for whatever you want with it. (except for illegal stuff, of course)
       (or do, i don't care)
       (but don't tell me about it, please)

Commands: 
    help       Show this help message

"#;

    println!("{}", help_text);
}

pub fn unknown_command() {
    println!("> Unknown command");
    println!("> Type 'rex help' for a list of commands");
}