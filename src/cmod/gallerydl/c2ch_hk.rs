use regex::Regex;
use scraper::{Html, Selector};
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};
use reqwest::Client;

pub async fn download(url: String) {
    let thread_re = Regex::new(r"^https?://2ch\.hk/([a-zA-Z0-9]+)/res/(\d+)\.html$").unwrap();

    let caps = match thread_re.captures(&url) {
        Some(caps) => caps,
        None => {
            eprintln!("Invalid 2ch.hk thread URL.");
            return;
        }
    };

    let board = &caps[1];
    let thread_id = &caps[2];
    let save_dir = format!("./rex/gallery-dl/2ch.hk/{}/{}", board, thread_id);

    if let Err(e) = fs::create_dir_all(&save_dir).await {
        eprintln!("Failed to create directory: {}", e);
        return;
    }

    let client = Client::new();
    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to fetch thread: {}", e);
            return;
        }
    };

    let body = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Failed to read response body: {}", e);
            return;
        }
    };

    let document = Html::parse_document(&body);
    let selector = Selector::parse("img").unwrap();

    let mut handles = vec![];

    let elements: Vec<String> = document
        .select(&selector)
        .filter_map(|e| e.value().attr("data-src").map(|href| href.to_string()))
        .collect();

    for element in elements.clone() {
        let element = element.clone();
        
        if let Some(href) = element.split('?').next().map(|s| s.to_string()) {
            let media_url = format!("https://2ch.hk{}", href);
            let filename = match href.split('/').last() {
                Some(name) => name.to_string(),
                None => continue,
            };

            let filepath: PathBuf = [save_dir.as_str(), &filename].iter().collect();

            // Skip if file already exists
            if filepath.exists() {
                println!("Already exists: {}", filename);
                continue;
            }

            let client = client.clone();
            let filepath_clone = filepath.clone();
            let media_url_clone = media_url.clone();

            // Spawn concurrent download tasks
            let handle = tokio::spawn(async move {
                match client.get(&media_url_clone).send().await {
                    Ok(media_resp) => {
                        match fs::File::create(&filepath_clone).await {
                            Ok(mut file) => {
                                match media_resp.bytes().await {
                                    Ok(bytes) => {
                                        if let Err(e) = file.write_all(&bytes).await {
                                            eprintln!("Failed to write {}: {}", filename, e);
                                        } else {
                                            println!("Downloaded: {}", filename);
                                        }
                                    }
                                    Err(e) => eprintln!("Failed to read media body: {}", e),
                                }
                            }
                            Err(e) => eprintln!("Failed to create file {}: {}", filename, e),
                        }
                    }
                    Err(e) => eprintln!("Failed to download {}: {}", media_url_clone, e),
                }
            });

            handles.push(handle);
        }
    }

    // Wait for all tasks
    for handle in handles {
        let _ = handle.await;
    }
}
