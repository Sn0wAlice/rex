use crate::helper::args::Args;

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

pub async fn main(args:Args) {
    if args.tabled.contains_key("help") {
        show_help().await;
        return;
    }

    // check if we have a valid path with "--path=<path>"
    if args.tabled.contains_key("path") {
        let path = args.tabled.get("path").unwrap();
        watch_folder(path.to_string()).unwrap();
    } else {
        println!("No path specified. Use --path=<path> to specify a path.");
        return;
    }
}

fn watch_folder(folder_to_watch:String) -> notify::Result<()> {
    let (tx, rx) = channel();

    let p = string_to_pathbuf(&folder_to_watch);
    println!("🧐 Watching path: {}", p.display());

    // Create the watcher
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Watch recursively (though TrayFiles probably doesn't nest)
    watcher.watch(&p, RecursiveMode::NonRecursive)?;

    // Wait for events
    loop {
        match rx.recv_timeout(Duration::from_secs(10)) {
            Ok(Ok(event)) => handle_event(event),
            Ok(Err(e)) => eprintln!("watch error: {:?}", e),
            Err(_) => (), // Timeout — nothing new
        }
    }
}

fn handle_event(event: Event) {
    match event.kind {
        notify::EventKind::Access(_) => println!("File accessed: {:?}", event.paths),
        notify::EventKind::Create(_) => println!("File created: {:?}", event.paths),
        notify::EventKind::Modify(_) => println!("File modified: {:?}", event.paths),
        notify::EventKind::Remove(_) => println!("File removed: {:?}", event.paths),
        notify::EventKind::Other => println!("File event: {:?}", event.paths),
        _ => println!("Other event: {:?}", event),
    }
}

fn string_to_pathbuf(path: &str) -> PathBuf {
    let mut pathbuf = PathBuf::new();
    pathbuf.push(path);
    pathbuf
}


async fn show_help() {
    println!(">>>> fswatch help <<<<\n");
    println!("Usage: fswatch --path=<path>");
    println!("Options:");
    println!("  --path=<path>   (required) The path to watch");
    println!("  --help          Show this help message");
}