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

pub async fn main(args: Vec<String>) {
    if args.len() < 4 {
        eprintln!("Usage: {} ssl <command> <args>", args[0]);
        std::process::exit(1);
    }

    match args[2].as_str() {
        "dump" => {
            let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
                .unwrap()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
            let pb = ProgressBar::new(1);
            pb.set_style(spinner_style.clone());
            pb.set_prefix(format!("[{}/?]",  1));
            pb.set_message(format!("Loading certificates for {}", args[3]));
            match fetch_certificates(&args[3]).await {
                Ok(certs) => {
                    pb.finish_with_message("done");
                    println!("Found {} certificates:", certs.len());
                    for cert in certs {
                        println!(
                            "- [{}] {}",
                            cert.entry_timestamp,
                            cert.name_value.replace("\n", ", ")
                        );
                    }
                }
                Err(e) => eprintln!("Error fetching certs: {}", e),
            }
        }
        _ => {
            eprintln!("Unknown SSL command: {}", args[1]);
        }
    }
}

pub async fn fetch_certificates(domain: &str) -> Result<Vec<CertificateEntry>, reqwest::Error> {
    let url = format!(
        "https://crt.sh/?q={}&output=json",
        domain
    );

    let client = Client::new();
    let resp = client.get(&url).send().await?;
    if resp.status().is_success() {
        let v:Value = resp.json().await?;

        let mut certs:Vec<CertificateEntry> = serde_json::from_value(v).unwrap_or_else(|_| {
            eprintln!("Failed to parse JSON response");
            vec![]
        });

        // Supprimer les doublons de noms
        certs.sort_by(|a, b| a.name_value.cmp(&b.name_value));
        certs.dedup_by(|a, b| a.name_value == b.name_value);

        Ok(certs)
    } else {
        eprintln!("Failed to fetch certificates: {}", resp.status());
        Ok(vec![])
    }
}



