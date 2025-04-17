pub fn show_help() {
let help_text = r#"

           __    Hello i'am Rex 
          / _) /
   .-^^^-/ /
  /       /
<_.|_|-|_|


rex - A simple command line tool for, euuuu, 
       ... for what exactly?
       ... cybersecurity?
       ... pentesting?

Never mind, just use it for whatever you want with it. 
       (except for illegal stuff, of course)
       (or do, i don't care)
       (but don't tell me about it, please)

General Commands: 
    help       Show this help message

Web shit
    webcrawl   Crawl a website for links and resources

File System Commands:
    fswatch    Watch a folder for changes in real-time


Notes: 
type 'rex <command> --help' for more information about a command
"#;

    println!("{}", help_text);
}

pub fn unknown_command() {
    println!("> Unknown command");
    println!("> Type 'rex help' for a list of commands");
}