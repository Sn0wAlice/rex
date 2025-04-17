use crate::helper::args::Args;

use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;
use uuid;
use std::fs;
use std::io::Write;

struct Pscrreturn {
    pub found: Vec<String>,
    pub new_urls: Vec<String>,
}

pub async fn main(args:Args) {
    if args.tabled.contains_key("help") {
        show_help().await;
        return;
    }

    // check if we have a valid path with "--base-url=<http(s)_url>"
    if args.tabled.contains_key("base-url") {
        let baseurl = args.tabled.get("base-url").unwrap();
        start(baseurl.to_string(), args).await;
    } else {
        println!("No path specified. Use --base-url=<http(s)_url> to start crawling.");
        return;
    }
}

async fn show_help() {
    println!(">>>> webcrawl help <<<<\n");
    println!("Usage: webcrawl --base-url=<http(s)_url>");
    println!("Options:");
    println!("  --base-url=<http(s)_url>   (required) The base URL to crawl");
    println!("  --extract=<exts>           (required) extensions to extract, comma-separated");
    println!("  --help             Show this help message");
}



async fn start(baseurl: String, args: Args) {
    println!("Starting crawl on: {}", baseurl);

    let allowed_exts: Vec<String> = match args.tabled.get("extract") {
        Some(exts) => exts.split(',').map(|s| s.trim().to_lowercase()).collect::<Vec<_>>(),
        None => {
            println!("No --extract argument provided.");
            return;
        }
    };
    println!("Will extract files with extensions: {:?}", allowed_exts);

    let selectors: Vec<Selector> = vec![
        Selector::parse("a").unwrap(),
        Selector::parse("link").unwrap(),
        Selector::parse("script").unwrap(),
        Selector::parse("img").unwrap(),
        Selector::parse("source").unwrap(),
        Selector::parse("video").unwrap(),
        Selector::parse("audio").unwrap(),
        Selector::parse("iframe").unwrap(),
        Selector::parse("embed").unwrap(),
    ];
            
    let art: Vec<&str> = vec![
        "href",
        "src",
        "data-src",
        "data-href",
        "data-link",
        "data-file",
        "data-url",
        "data-video",
        "data-audio",
        "data-embed",
    ];

    let mut found:Vec<String> = Vec::new();

    let mut need_to_crawl = vec![baseurl.clone()];
    let mut already_crawled: Vec<String> = vec![];

    while !need_to_crawl.is_empty() {
        let url = need_to_crawl.pop().unwrap();
        let res = one_request(url.clone(), found.clone(), selectors.clone(), art.clone(), allowed_exts.clone(), baseurl.clone()).await;

        // Add new URLs to the need_to_crawl list
        for new_url in res.new_urls {
            if !already_crawled.contains(&new_url) && !need_to_crawl.contains(&new_url) {
                need_to_crawl.push(new_url);
            }
        }

        for found_url in res.found {
            if !already_crawled.contains(&found_url) && !found.contains(&found_url) {
                found.push(found_url);
            }
        }

        // Move the current URL to already_crawled
        already_crawled.push(url);
    }

    println!("Crawling completed.");

    // in case: remove duplicates from found
    found.sort();
    found.dedup();

    // save to [random_uuid].txt
    let filename = format!("{}.txt", uuid::Uuid::new_v4());
    let mut file = fs::File::create(&filename).expect("Unable to create file");
    for url in &found {
        writeln!(file, "{}", *url).expect("Unable to write data");
    }
    println!("Found {} files. Saved to {}", found.len(), filename);
    println!("Crawled {} URLs.", already_crawled.len());
}

async fn one_request(baseurl:String, allreadyfound:Vec<String>, selectors: Vec<Selector>,  art: Vec<&str>, allowed_exts:Vec<String>, burl:String) -> Pscrreturn {

    let mut new_urls = Vec::new();
    let mut found: Vec<String> = allreadyfound.clone();

    let client = Client::new();
    if let Ok(resp) = client.get(&baseurl).send().await {
        if let Ok(body) = resp.text().await {
            let doc = Html::parse_document(&body);
            let base = Url::parse(&baseurl).expect("Invalid base URL");

            for selector in selectors {
                for element in doc.select(&selector) {
                    // Check for attributes in the element
                    for attr in art.iter() {
                        if let Some(attr_value) = element.value().attr(attr) {
                            if let Ok(full_url) = base.join(attr_value) {
                                let path = full_url.path().to_lowercase();
                                if allowed_exts.iter().any(|ext| path.ends_with(&format!(".{}", ext))) {
                                    // check if the file is already found
                                    if found.contains(&full_url.to_string()) || allreadyfound.contains(&full_url.to_string()) {
                                        continue;
                                    } else {
                                        found.push(full_url.to_string());
                                        println!("Found matching file: {}", full_url);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // identify other links on the same website to crawl. push all known links to new_urls
            let link_selector = Selector::parse("a").unwrap();
            for link in doc.select(&link_selector) {
                if let Some(href) = link.value().attr("href") {
                    if let Ok(mut full_url) = base.join(href) {
                        // full_url remove #x
                        full_url.set_fragment(None);
                        if(full_url.host_str() == Some(base.host_str().unwrap()) && full_url.scheme() == base.scheme()) || full_url.host_str() == Some(burl.as_str()) {
                            new_urls.push(full_url.to_string());
                            if !new_urls.contains(&full_url.to_string()) {
                                new_urls.push(full_url.to_string());
                                println!("Found link: {}", full_url);
                            }
                        }
                    }
                }
            }
            
        }
    } else {
        println!("Failed to fetch {}", baseurl);
    }

    return Pscrreturn {
        found: found,
        new_urls: new_urls,
    };
}
