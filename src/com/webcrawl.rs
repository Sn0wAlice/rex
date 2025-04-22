use crate::helper::args::Args;
use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;
use uuid;
use std::fs;
use std::io::Write;

use indicatif::{ProgressBar, ProgressState, ProgressStyle};

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
        let starttime = std::time::Instant::now();
        start(baseurl.to_string(), args).await;
        let elapsed = starttime.elapsed();
        println!("Crawling completed in {:.2?}", elapsed);
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
    println!("  --thread=<nb>              (default: 10) Number of threads to use for crawling");
    println!("  --help             Show this help message");
}


async fn start(baseurl: String, args: Args) {
    println!("Starting crawl on: {}", baseurl);

    let allowed_exts: Vec<String> = match args.tabled.get("extract") {
        Some(exts) => exts.split(',').map(|s| s.trim().to_lowercase()).collect(),
        None => {
            println!("No --extract argument provided.");
            return;
        }
    };
    println!("Will extract files with extensions: {:?}", allowed_exts);

    let selectors = vec![
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

    let art = vec![
        "href", "src", "data-src", "data-href", "data-link",
        "data-file", "data-url", "data-video", "data-audio", "data-embed",
    ];

    let mut found: Vec<String> = Vec::new();
    let mut total_found = 0;
    
    let mut already_crawled: Vec<String> = Vec::new();
    let mut need_to_crawl = vec![baseurl.clone()];

    let mut max_threads = 10;
    if let Some(thread_count) = args.tabled.get("thread") {
        if let Ok(count) = thread_count.parse::<usize>() {
            max_threads = count;
        }
    }
    println!("Using {} threads for crawling.", max_threads);

    let pb = ProgressBar::new(need_to_crawl.len() as u64);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta}) ({msg})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));


    while !need_to_crawl.is_empty() {
        let mut tasks = FuturesUnordered::new();

        // Spawn concurrent tasks up to a limit (e.g., 10)
        for _ in 0..max_threads {
            if let Some(url) = need_to_crawl.pop() {
                if already_crawled.contains(&url) {
                    continue;
                }
                already_crawled.push(url.clone());
                let task = one_request(
                    url,
                    found.clone(),
                    selectors.clone(),
                    art.clone(),
                    allowed_exts.clone(),
                    baseurl.clone(),
                );
                tasks.push(task);

                pb.set_position(already_crawled.len() as u64);
                // update the progress bar maximum to the number of URLs to crawl
                pb.set_length((need_to_crawl.len() + already_crawled.len()) as u64);
            }
        }

        while let Some(res) = tasks.next().await {
            for new_url in res.new_urls {
                if !already_crawled.contains(&new_url) && !need_to_crawl.contains(&new_url) {
                    need_to_crawl.push(new_url);
                }
            }

            for found_url in res.found {
                if !found.contains(&found_url) {
                    found.push(found_url);
                }
            }
            total_found = found.len();
            pb.set_message(format!("{}", total_found));
        }
    }

    found.sort();
    found.dedup();

    let filename = format!("{}.txt", uuid::Uuid::new_v4());
    let mut file = fs::File::create(&filename).expect("Unable to create file");
    for url in &found {
        writeln!(file, "{}", url).expect("Unable to write data");
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
                                        //println!("Found matching file: {}", full_url);
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
                                //println!("Found link: {}", full_url);
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
