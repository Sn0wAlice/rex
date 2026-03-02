use anyhow::{Result, Context};
use reqwest::Client;
use serde::Deserialize;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct CertificateEntry {
    pub common_name: Option<String>,
    pub entry_timestamp: String,
    pub id: u64,
    pub issuer_ca_id: u32,
    pub issuer_name: String,
    pub name_value: String,
    pub not_after: String,
    pub not_before: String,
    pub result_count: Option<u32>,
    pub serial_number: String,
}

pub async fn dump(domain: &str) -> Result<()> {
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .expect("static progress template")
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let pb = ProgressBar::new(1);
    pb.set_style(spinner_style);
    pb.set_prefix(format!("[{}/?]", 1));
    pb.set_message(format!("Loading certificates for {}", domain));

    let certs = fetch_certificates(domain).await
        .context("Failed to fetch certificates")?;

    pb.finish_with_message("done");
    println!("Found {} certificates:", certs.len());
    for cert in certs {
        println!(
            "- [{}] {}",
            cert.entry_timestamp,
            cert.name_value.replace("\n", ", ")
        );
    }
    Ok(())
}

async fn fetch_certificates(domain: &str) -> Result<Vec<CertificateEntry>> {
    let url = format!("https://crt.sh/?q={}&output=json", domain);
    let client = Client::new();
    let resp = client.get(&url).send().await
        .context("HTTP request to crt.sh failed")?;

    if resp.status().is_success() {
        let v: Value = resp.json().await
            .context("Failed to parse JSON response body")?;
        let mut certs: Vec<CertificateEntry> = serde_json::from_value(v)
            .unwrap_or_else(|_| {
                eprintln!("Failed to parse certificate entries");
                vec![]
            });
        certs.sort_by(|a, b| a.name_value.cmp(&b.name_value));
        certs.dedup_by(|a, b| a.name_value == b.name_value);
        Ok(certs)
    } else {
        eprintln!("Failed to fetch certificates: {}", resp.status());
        Ok(vec![])
    }
}
