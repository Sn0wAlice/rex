use anyhow::Result;
use std::io::{self, Write};

struct HashPattern {
    name: &'static str,
    len: usize,
    hex_only: bool,
    description: &'static str,
}

const PATTERNS: &[HashPattern] = &[
    // 32 hex
    HashPattern { name: "MD5",          len: 32,  hex_only: true,  description: "MD5 Message Digest" },
    HashPattern { name: "NTLM",        len: 32,  hex_only: true,  description: "Windows NTLM Hash" },
    HashPattern { name: "MD4",          len: 32,  hex_only: true,  description: "MD4 Message Digest" },
    // 40 hex
    HashPattern { name: "SHA-1",        len: 40,  hex_only: true,  description: "Secure Hash Algorithm 1" },
    HashPattern { name: "RIPEMD-160",   len: 40,  hex_only: true,  description: "RACE Integrity Primitives" },
    // 56 hex
    HashPattern { name: "SHA-224",      len: 56,  hex_only: true,  description: "SHA-2 (224-bit)" },
    // 64 hex
    HashPattern { name: "SHA-256",      len: 64,  hex_only: true,  description: "SHA-2 (256-bit)" },
    HashPattern { name: "SHA3-256",     len: 64,  hex_only: true,  description: "SHA-3 (256-bit)" },
    HashPattern { name: "BLAKE2s-256",  len: 64,  hex_only: true,  description: "BLAKE2s (256-bit)" },
    // 96 hex
    HashPattern { name: "SHA-384",      len: 96,  hex_only: true,  description: "SHA-2 (384-bit)" },
    HashPattern { name: "SHA3-384",     len: 96,  hex_only: true,  description: "SHA-3 (384-bit)" },
    // 128 hex
    HashPattern { name: "SHA-512",      len: 128, hex_only: true,  description: "SHA-2 (512-bit)" },
    HashPattern { name: "SHA3-512",     len: 128, hex_only: true,  description: "SHA-3 (512-bit)" },
    HashPattern { name: "BLAKE2b-512",  len: 128, hex_only: true,  description: "BLAKE2b (512-bit)" },
    HashPattern { name: "Whirlpool",    len: 128, hex_only: true,  description: "Whirlpool Hash" },
];

fn is_hex(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_base64(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}

fn detect_bcrypt(input: &str) -> bool {
    input.starts_with("$2a$") || input.starts_with("$2b$") || input.starts_with("$2y$")
}

fn detect_special_formats(input: &str) -> Vec<(&'static str, &'static str)> {
    let mut results = Vec::new();

    // bcrypt
    if detect_bcrypt(input) && input.len() == 60 {
        results.push(("bcrypt", "Blowfish-based password hash"));
    }

    // Unix crypt formats
    if input.starts_with("$1$") {
        results.push(("MD5 Crypt", "Unix MD5 password hash"));
    }
    if input.starts_with("$5$") {
        results.push(("SHA-256 Crypt", "Unix SHA-256 password hash"));
    }
    if input.starts_with("$6$") {
        results.push(("SHA-512 Crypt", "Unix SHA-512 password hash"));
    }
    if input.starts_with("$apr1$") {
        results.push(("Apache APR1", "Apache MD5 password hash"));
    }
    if input.starts_with("$argon2") {
        results.push(("Argon2", "Argon2 password hash"));
    }
    if input.starts_with("$scrypt$") || input.starts_with("$7$") {
        results.push(("scrypt", "scrypt password hash"));
    }
    if input.starts_with("$pbkdf2") {
        results.push(("PBKDF2", "PBKDF2 password hash"));
    }

    // MySQL
    if input.starts_with("*") && input.len() == 41 && is_hex(&input[1..]) {
        results.push(("MySQL 4.1+", "MySQL SHA1 password hash"));
    }

    // LM hash
    if input.len() == 32 && is_hex(input) && input == input.to_uppercase() {
        results.push(("LM Hash", "LAN Manager Hash (legacy Windows)"));
    }

    // CRC32
    if input.len() == 8 && is_hex(input) {
        results.push(("CRC-32", "Cyclic Redundancy Check (32-bit)"));
    }

    results
}

fn identify_hash(input: &str) -> Vec<(String, String)> {
    let mut candidates: Vec<(String, String)> = Vec::new();

    // Check special formats first (bcrypt, unix crypt, etc.)
    for (name, desc) in detect_special_formats(input) {
        candidates.push((name.to_string(), desc.to_string()));
    }

    // Check hex-based hashes by length
    let trimmed = input.trim();
    let lower = trimmed.to_lowercase();

    if is_hex(&lower) {
        for pattern in PATTERNS {
            if pattern.hex_only && lower.len() == pattern.len {
                candidates.push((pattern.name.to_string(), pattern.description.to_string()));
            }
        }
    }

    // Base64-encoded hashes (common for some tools)
    if candidates.is_empty() && is_base64(trimmed) {
        let decoded_len = (trimmed.trim_end_matches('=').len() * 3) / 4;
        match decoded_len {
            16 => candidates.push(("MD5 (base64)".to_string(), "MD5 encoded in base64".to_string())),
            20 => candidates.push(("SHA-1 (base64)".to_string(), "SHA-1 encoded in base64".to_string())),
            32 => candidates.push(("SHA-256 (base64)".to_string(), "SHA-256 encoded in base64".to_string())),
            48 => candidates.push(("SHA-384 (base64)".to_string(), "SHA-384 encoded in base64".to_string())),
            64 => candidates.push(("SHA-512 (base64)".to_string(), "SHA-512 encoded in base64".to_string())),
            _ => {}
        }
    }

    candidates
}

fn print_results(input: &str, candidates: &[(String, String)]) {
    let bar = "─".repeat(60);
    println!("\n┌{}┐", bar);
    println!("│{:^60}│", "Hash Analysis Result");
    println!("├{}┤", bar);
    println!("│ Input: {:<51}│", if input.len() > 51 {
        format!("{}...", &input[..48])
    } else {
        input.to_string()
    });
    println!("│ Length: {:<50}│", input.len());
    println!("├{}┤", bar);

    if candidates.is_empty() {
        println!("│{:^60}│", "No matching hash type found");
    } else {
        println!("│{:^60}│", format!("{} possible type(s) detected", candidates.len()));
        println!("├{}┤", bar);
        for (i, (name, desc)) in candidates.iter().enumerate() {
            let confidence = if candidates.len() == 1 { "HIGH" } else if i == 0 { "MEDIUM" } else { "LOW" };
            println!("│  [{:^6}] {:<48}│", confidence, name);
            println!("│           {:<48}│", desc);
            if i < candidates.len() - 1 {
                println!("│{:60}│", "");
            }
        }
    }
    println!("└{}┘", bar);
}

pub fn detect(hash: Option<&str>) -> Result<()> {
    let input = match hash {
        Some(h) => h.to_string(),
        None => {
            // Interactive mode
            let bar = "─".repeat(60);
            println!("┌{}┐", bar);
            println!("│{:^60}│", "Rex Hash Identifier");
            println!("├{}┤", bar);
            println!("│{:^60}│", "Paste a hash below to identify its type");
            println!("│{:^60}│", "Supports MD5, SHA, bcrypt, NTLM, and more");
            println!("└{}┘", bar);
            println!();
            print!("  > ");
            io::stdout().flush()?;
            let mut buf = String::new();
            io::stdin().read_line(&mut buf)?;
            buf.trim().to_string()
        }
    };

    if input.is_empty() {
        eprintln!("No hash provided.");
        return Ok(());
    }

    let candidates = identify_hash(&input);
    print_results(&input, &candidates);

    Ok(())
}
