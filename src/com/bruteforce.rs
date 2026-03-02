use anyhow::{Result, bail};
use digest::Digest;
use std::io::{self, Write};
use std::time::Instant;

const CHARSETS: &[(&str, &str)] = &[
    ("lower",        "abcdefghijklmnopqrstuvwxyz"),
    ("upper",        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
    ("digits",       "0123456789"),
    ("alpha",        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"),
    ("alphanumeric", "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"),
    ("all",          "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+"),
];

fn prompt(label: &str) -> Result<String> {
    print!("  {} > ", label);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn hash_candidate(algo: &str, candidate: &str) -> String {
    match algo {
        "md5" => {
            let mut h = md5::Md5::new();
            h.update(candidate.as_bytes());
            format!("{:x}", h.finalize())
        }
        "sha1" => {
            let mut h = sha1::Sha1::new();
            h.update(candidate.as_bytes());
            format!("{:x}", h.finalize())
        }
        "sha256" => {
            let mut h = sha2::Sha256::new();
            h.update(candidate.as_bytes());
            format!("{:x}", h.finalize())
        }
        "sha512" => {
            let mut h = sha2::Sha512::new();
            h.update(candidate.as_bytes());
            format!("{:x}", h.finalize())
        }
        _ => unreachable!(),
    }
}

fn total_combinations(charset_len: usize, extra_len: usize) -> u64 {
    (charset_len as u64).pow(extra_len as u32)
}

fn bruteforce_recursive(
    algo: &str,
    target: &str,
    prefix: &str,
    charset: &[u8],
    extra_len: usize,
    buffer: &mut Vec<u8>,
    count: &mut u64,
    total: u64,
    start: &Instant,
) -> Option<String> {
    if buffer.len() == extra_len {
        let suffix = std::str::from_utf8(buffer).unwrap();
        let candidate = format!("{}{}", prefix, suffix);
        *count += 1;

        if *count % 500_000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = *count as f64 / elapsed;
            let pct = (*count as f64 / total as f64) * 100.0;
            eprint!(
                "\r  [{:>6.2}%] {:>10} / {} | {:.0} hash/s    ",
                pct, count, total, rate
            );
        }

        let h = hash_candidate(algo, &candidate);
        if h == target {
            return Some(candidate);
        }
        return None;
    }

    for &c in charset {
        buffer.push(c);
        if let Some(found) = bruteforce_recursive(algo, target, prefix, charset, extra_len, buffer, count, total, start) {
            return Some(found);
        }
        buffer.pop();
    }

    None
}

pub fn run(
    algo: Option<&str>,
    target_hash: Option<&str>,
    prefix: Option<&str>,
    extra: Option<usize>,
    charset_name: Option<&str>,
) -> Result<()> {
    let bar = "─".repeat(60);

    // Interactive or argument mode
    let algo = match algo {
        Some(a) => a.to_string(),
        None => {
            println!("┌{}┐", bar);
            println!("│{:^60}│", "Rex Hash Bruteforcer");
            println!("├{}┤", bar);
            println!("│{:^60}│", "Supported: md5, sha1, sha256, sha512");
            println!("└{}┘", bar);
            println!();
            prompt("Hash type (md5/sha1/sha256/sha512)")?
        }
    };

    if !["md5", "sha1", "sha256", "sha512"].contains(&algo.as_str()) {
        bail!("Unsupported hash type: {}. Use md5, sha1, sha256, or sha512.", algo);
    }

    let target = match target_hash {
        Some(t) => t.to_string(),
        None => prompt("Target hash")?,
    };
    let target = target.to_lowercase();

    if target.is_empty() {
        bail!("No target hash provided.");
    }

    let prefix = match prefix {
        Some(p) => p.to_string(),
        None => prompt("Known prefix (leave empty if none)")?,
    };

    let extra_len: usize = match extra {
        Some(e) => e,
        None => {
            let val = prompt("Extra characters to bruteforce")?;
            val.parse().map_err(|_| anyhow::anyhow!("Invalid number: {}", val))?
        }
    };

    if extra_len == 0 {
        // Just hash the prefix and check
        let h = hash_candidate(&algo, &prefix);
        if h == target {
            println!("\n  Password found: {}", prefix);
        } else {
            println!("\n  Hash of '{}' = {}", prefix, h);
            println!("  Does not match target.");
        }
        return Ok(());
    }

    if extra_len > 6 {
        bail!("Max 6 extra characters supported (search space too large beyond that).");
    }

    let charset_key = charset_name.unwrap_or("alphanumeric");
    let charset = CHARSETS
        .iter()
        .find(|(name, _)| *name == charset_key)
        .map(|(_, chars)| chars)
        .unwrap_or(&CHARSETS[4].1); // default: alphanumeric

    let total = total_combinations(charset.len(), extra_len);

    println!();
    println!("  Algorithm : {}", algo);
    println!("  Prefix    : '{}'", prefix);
    println!("  Extra     : {} chars from '{}' ({} chars)", extra_len, charset_key, charset.len());
    println!("  Total     : {} combinations", total);
    println!();

    if total > 100_000_000 {
        println!("  Warning: this will take a while...");
    }

    let start = Instant::now();
    let mut count: u64 = 0;
    let mut buffer: Vec<u8> = Vec::with_capacity(extra_len);

    let result = bruteforce_recursive(
        &algo,
        &target,
        &prefix,
        charset.as_bytes(),
        extra_len,
        &mut buffer,
        &mut count,
        total,
        &start,
    );

    let elapsed = start.elapsed();
    eprintln!("\r{:80}", ""); // clear progress line

    println!("┌{}┐", bar);
    match result {
        Some(password) => {
            println!("│{:^60}│", "PASSWORD FOUND");
            println!("├{}┤", bar);
            println!("│  Password : {:<46}│", password);
        }
        None => {
            println!("│{:^60}│", "NOT FOUND");
            println!("├{}┤", bar);
            println!("│{:^60}│", "Password not in search space");
        }
    }
    println!("├{}┤", bar);
    println!("│  Tried    : {:<46}│", format!("{} hashes", count));
    println!("│  Time     : {:<46}│", format!("{:.2}s", elapsed.as_secs_f64()));
    println!("│  Speed    : {:<46}│", format!("{:.0} hash/s", count as f64 / elapsed.as_secs_f64().max(0.001)));
    println!("└{}┘", bar);

    Ok(())
}
