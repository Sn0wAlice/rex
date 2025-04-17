use crate::helper::args::Args;

pub async fn main(args:Args) {
    if args.tabled.contains_key("help") {
        show_help().await;
        return;
    }

    if args.array.len() < 2 {
        println!("No path specified. Use gallery <url> to specify a url.");
        return;
    }

    if !args.array[1].clone().is_empty() {
        let url = args.array[1].clone();
        println!("🧐 Downloading media from URL: {}", url);
        download(url).await;
    } else {
        println!("No path specified. check gallery-dl help to see how to use it.");
        return;
    }
}

async fn show_help() {
    println!(">>>> gallery-dl help <<<<\n");
    println!("Usage: gallery-dl <url>");
    println!("Options:");
    println!("  <path>         (required) the web url to download");
    println!("  --help          Show this help message");
}



async fn download(url: String) {
    match url.clone() {
        u if u.starts_with("https://2ch.hk/") => {crate::cmod::gallerydl::c2ch_hk::download(url).await;}

        _ => {println!("Website is not supported");}
    }
}