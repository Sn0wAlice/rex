use regex::Regex;
use trust_dns_resolver::TokioAsyncResolver;

pub async fn mail_scan(domain: &str) {
    println!("📬 Scanning mail configuration for domain: {}\n", domain);

    let resolver = TokioAsyncResolver::tokio_from_system_conf()
        .expect("Failed to create DNS resolver");

    let mut issues = vec![];

    // === SPF ===
    let spf_record = resolver.txt_lookup(domain).await.ok()
        .and_then(|txts| {
            txts.iter()
                .flat_map(|r| r.txt_data().iter())
                .map(|b| String::from_utf8_lossy(b).into_owned())
                .find(|txt| txt.starts_with("v=spf1"))
        });

    println!("🔹 SPF:");
    match spf_record {
        Some(spf) => {
            println!("SPF record found:\n   {}", spf);
            if !spf.contains("~all") && !spf.contains("-all") {
                println!("Warning: SPF record does not end with ~all or -all (soft/hard fail).");
                issues.push("SPF policy may not be strict enough.");
            }
            // broque the SPF record and list all the part of it, like an include, ip4, ip6, etc.
            let parts: Vec<&str> = spf.split_whitespace().collect();
            println!("\nSPF record parts:");
            for part in parts {
                if part.starts_with("include:") {
                    println!("  - Include: {}", &part[8..]);
                } else if part.starts_with("ip4:") {
                    println!("  - IPv4: {}", &part[4..]);
                } else if part.starts_with("ip6:") {
                    println!("  - IPv6: {}", &part[4..]);
                } else if part.starts_with("a") || part.starts_with("mx") {
                    println!("  - Mechanism: {}", part);
                } else {
                    if part.starts_with("v=spf1") {
                        continue; // Skip the version part
                    } else if part.ends_with("all") {
                        println!("  - Policy: {}", part);
                    } else {
                        println!("  - Other: {}", part);
                    }
                }
            }
        }
        None => {
            println!("No SPF record found.");
            issues.push("Missing SPF record.");
        }
    }
    println!();

    // === DKIM ===
    println!("🔹 DKIM:");
    let dkim_selector = "default"; // souvent "default", mais peut varier
    let dkim_domain = format!("{}._domainkey.{}", dkim_selector, domain);

    let dkim_record = resolver.txt_lookup(dkim_domain.clone()).await.ok()
        .and_then(|txts| {
            txts.iter()
                .flat_map(|r| r.txt_data().iter())
                .map(|b| String::from_utf8_lossy(b).into_owned())
                .find(|txt| txt.contains("v=DKIM1"))
        });

    match dkim_record {
        Some(dkim) => {
            println!("DKIM record found for selector '{}':\n   {}", dkim_selector, dkim);
        }
        None => {
            println!("No DKIM record found at '{}'", dkim_domain);
            issues.push("Missing DKIM record or wrong selector (default used here).");
        }
    }
    println!();

    // === DMARC ===
    println!("🔹 DMARC:");
    let dmarc_domain = format!("_dmarc.{}", domain);
    let dmarc_record = resolver.txt_lookup(dmarc_domain.clone()).await.ok()
        .and_then(|txts| {
            txts.iter()
                .flat_map(|r| r.txt_data().iter())
                .map(|b| String::from_utf8_lossy(b).into_owned())
                .find(|txt| txt.contains("v=DMARC1"))
        });

    match dmarc_record {
        Some(dmarc) => {
            println!("DMARC record found:\n   {}", dmarc);
            let re = Regex::new(r"p=([a-zA-Z]+)").unwrap();
            match re.captures(&dmarc) {
                Some(cap) if &cap[1] == "none" => {
                    println!("DMARC policy is 'none' — monitoring only, no enforcement.");
                    issues.push("DMARC policy is 'none'; consider 'quarantine' or 'reject'.");
                }
                Some(cap) => {
                    println!("DMARC policy is '{}'", &cap[1]);
                }
                None => {
                    println!("DMARC record found, but no policy (p=) detected.");
                    issues.push("DMARC record missing 'p=' policy.");
                }
            }
        }
        None => {
            println!("No DMARC record found.");
            issues.push("Missing DMARC record.");
        }
    }
    println!();

    // === Summary ===
    println!("📋 Summary Report for '{}':", domain);
    if issues.is_empty() {
        println!("All mail DNS records are correctly configured!");
    } else {
        for issue in issues {
            println!("{}", issue);
        }
    }
}