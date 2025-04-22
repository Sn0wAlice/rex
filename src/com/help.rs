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
    version    Show the version of rex
    update     Update rex to the latest version

Web shit
    webcrawl   Crawl a website for links and resources
    gallery-dl Download all media from a web page

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

pub fn show_version() {
    let help_text = r#"

           __    v0.0.3
          / _) /
   .-^^^-/ /
  /       /
<_.|_|-|_|
"#;

    println!("{}", help_text);
}

pub fn update() {
    // check if we are on macos or on linux
    // and check run utils/install_macos.sh or utils/install_linux.sh

    if cfg!(target_os = "macos") {
        println!("> Updating rex on macos");
        println!("> Please wait...");
        std::process::Command::new("bash")
            .arg("utils/install_macos.sh")
            .status()
            .expect("Failed to update rex");
    } else if cfg!(target_os = "linux") {
        println!("> Updating rex on linux");
        println!("> Please wait...");
        std::process::Command::new("bash")
            .arg("utils/install_linux.sh")
            .status()
            .expect("Failed to update rex");
    } else {
        println!("> Unsupported OS for update");
    }
    
}