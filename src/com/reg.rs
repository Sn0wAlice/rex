use anyhow::{Result, Context};
use std::process::{Command, Stdio};
use std::collections::{HashMap, HashSet};
use regex::Regex;
use std::io::{BufRead, BufReader};
use serde::Deserialize;
use std::fs;

//
//   _____         _               _
//  |   __|_ _ ___| |_ ___ _____ _| |
//  |__   | | |_ -|  _| -_|     | . |
//  |_____|_  |___|_| |___|_|_|_|___|
//        |___|

#[derive(Debug, Deserialize)]
struct JournalEntry {
    #[serde(rename = "PRIORITY")]
    priority: Option<String>,
    #[serde(rename = "SYSLOG_IDENTIFIER")]
    syslog_identifier: Option<String>,
    #[serde(rename = "_UID")]
    uid: Option<String>,
    #[serde(rename = "_PID")]
    pid: Option<String>,
    #[serde(rename = "MESSAGE")]
    message: Option<String>,
    #[serde(rename = "_SOURCE_REALTIME_TIMESTAMP")]
    #[allow(dead_code)]
    timestamp: Option<String>,
    #[serde(rename = "_SYSTEMD_UNIT")]
    systemd_unit: Option<String>,
}

fn spawn_journalctl(args: &[&str]) -> Result<std::process::Child> {
    Command::new("journalctl")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to start journalctl -- is it installed?")
}

pub fn sysd_extract(number: usize) -> Result<()> {
    let n_str = number.to_string();
    let mut child = spawn_journalctl(&["-o", "json", "-n", &n_str])?;
    let stdout = child.stdout.take()
        .context("Failed to capture journalctl stdout")?;
    let reader = BufReader::new(stdout);

    for line in reader.lines().flatten() {
        match serde_json::from_str::<JournalEntry>(&line) {
            Ok(entry) => {
                println!(
                    "[{}] {} (PID: {}) — {}",
                    entry.priority.unwrap_or_else(|| "?".to_string()),
                    entry.syslog_identifier.unwrap_or_else(|| "unknown".to_string()),
                    entry.pid.unwrap_or_else(|| "?".to_string()),
                    entry.message.unwrap_or_else(|| "empty".to_string())
                );
            }
            Err(e) => {
                eprintln!("Error parsing JSON: {}", e);
            }
        }
    }
    Ok(())
}

pub fn sysd_extract_group(number: usize) -> Result<()> {
    let n_str = number.to_string();
    let mut child = spawn_journalctl(&["-o", "json", "-n", &n_str])?;
    let stdout = child.stdout.take()
        .context("Failed to capture journalctl stdout")?;
    let reader = BufReader::new(stdout);

    let mut grouped_logs: HashMap<String, Vec<String>> = HashMap::new();

    for line in reader.lines().flatten() {
        if let Ok(entry) = serde_json::from_str::<JournalEntry>(&line) {
            let identifier = entry
                .syslog_identifier
                .unwrap_or_else(|| "unknown".to_string());

            let message = format!(
                "[{}] PID {}: {}",
                entry.priority.unwrap_or_else(|| "?".to_string()),
                entry.pid.unwrap_or_else(|| "?".to_string()),
                entry.message.unwrap_or_else(|| "empty message".to_string())
            );

            grouped_logs
                .entry(identifier)
                .or_default()
                .push(message);
        }
    }

    for (identifier, messages) in grouped_logs {
        println!("=== {} ===", identifier);
        for msg in messages {
            println!("{}", msg);
        }
        println!();
    }
    Ok(())
}

#[derive(Debug)]
enum EventType {
    FailedLogin,
    ServiceRestart,
    PrivilegeEscalation,
    Unknown,
}

fn classify_event(entry: &JournalEntry) -> EventType {
    if let Some(msg) = &entry.message {
        let msg_lower = msg.to_lowercase();
        if msg_lower.contains("failed password") || msg_lower.contains("authentication failure") {
            EventType::FailedLogin
        } else if msg_lower.contains("started") || msg_lower.contains("restarted") {
            EventType::ServiceRestart
        } else if msg_lower.contains("sudo") && msg_lower.contains("session opened") {
            EventType::PrivilegeEscalation
        } else {
            EventType::Unknown
        }
    } else {
        EventType::Unknown
    }
}

pub fn sysd_scan(number: usize) -> Result<()> {
    let n_str = number.to_string();
    let mut child = spawn_journalctl(&["-o", "json", "-n", &n_str])?;
    let stdout = child.stdout.take()
        .context("Failed to capture journalctl stdout")?;
    let reader = BufReader::new(stdout);

    let mut events_by_type: HashMap<String, Vec<String>> = HashMap::new();

    for line in reader.lines().flatten() {
        if let Ok(entry) = serde_json::from_str::<JournalEntry>(&line) {
            let event_type = classify_event(&entry);

            let label = match event_type {
                EventType::FailedLogin => "Failed Login",
                EventType::ServiceRestart => "Service Restart",
                EventType::PrivilegeEscalation => "Privilege Escalation",
                EventType::Unknown => "Unknown",
            };

            let line_output = format!(
                "[{}] {} (PID: {}): {}",
                entry.priority.unwrap_or_else(|| "?".to_string()),
                entry.syslog_identifier.clone().unwrap_or_else(|| "unknown".to_string()),
                entry.pid.clone().unwrap_or_else(|| "?".to_string()),
                entry.message.clone().unwrap_or_else(|| "empty".to_string())
            );

            events_by_type.entry(label.to_string()).or_default().push(line_output);
        }
    }

    for (etype, logs) in events_by_type {
        println!("=== {} ===", etype);
        for log in logs {
            println!("{}", log);
        }
        println!();
    }
    Ok(())
}

pub fn sshfail() -> Result<()> {
    let failed_attempts = extract_failed_ssh_attempts()?;
    println!("| IP Address | Failed Attempts |");
    println!("|------------|-----------------|");
    for (ip, count) in failed_attempts {
        println!("| {} | {} |", ip, count);
    }
    Ok(())
}

fn extract_failed_ssh_attempts() -> Result<HashMap<String, u32>> {
    let mut child = spawn_journalctl(&["-o", "json", "-u", "ssh", "-n", "1000"])?;
    let stdout = child.stdout.take()
        .context("Failed to capture journalctl stdout")?;
    let reader = BufReader::new(stdout);

    let ip_regex = Regex::new(r"from ([0-9]{1,3}(\.[0-9]{1,3}){3})")
        .expect("static regex pattern");
    let mut ip_counter: HashMap<String, u32> = HashMap::new();

    for line in reader.lines().flatten() {
        if let Ok(entry) = serde_json::from_str::<JournalEntry>(&line) {
            if let Some(msg) = &entry.message {
                println!("=== {} ===", msg);
                if msg.to_lowercase().contains("failed") {
                    if let Some(cap) = ip_regex.captures(msg) {
                        if let Some(ip) = cap.get(1) {
                            let count = ip_counter.entry(ip.as_str().to_string()).or_insert(0);
                            *count += 1;
                        }
                    }
                }
            }
        }
    }
    println!("\n");

    Ok(ip_counter)
}

#[derive(Debug)]
enum SuspiciousEvent {
    BruteforceLogin { ip: String, attempts: u32 },
    FrequentRestart { unit: String, count: u32 },
    LogTampering { msg: String },
    SuspiciousUID0 { uid: String, msg: String },
    UnknownService { unit: String },
}

fn detect_suspicious_events() -> Result<Vec<SuspiciousEvent>> {
    let mut child = spawn_journalctl(&["-o", "json", "-n", "10000"])?;
    let stdout = child.stdout.take()
        .context("Failed to capture journalctl stdout")?;
    let reader = BufReader::new(stdout);

    let mut bruteforce_ips: HashMap<String, u32> = HashMap::new();
    let mut restart_counts: HashMap<String, u32> = HashMap::new();
    let mut suspicious_events: Vec<SuspiciousEvent> = Vec::new();
    let mut known_services: HashSet<String> = [
        "sshd.service", "cron.service", "sudo.service", "systemd-logind.service",
        "rsyslog.service", "network.service"
    ]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let ip_regex = Regex::new(r"from ([0-9]{1,3}(\.[0-9]{1,3}){3})")
        .expect("static regex pattern");

    for line in reader.lines().flatten() {
        if let Ok(entry) = serde_json::from_str::<JournalEntry>(&line) {
            let msg = entry.message.clone().unwrap_or_default().to_lowercase();
            let unit = entry.systemd_unit.clone().unwrap_or_default();

            // 1. SSH bruteforce detection
            if msg.contains("failed password") {
                if let Some(cap) = ip_regex.captures(&msg) {
                    if let Some(ip) = cap.get(1) {
                        let count = bruteforce_ips.entry(ip.as_str().to_string()).or_insert(0);
                        *count += 1;
                    }
                }
            }

            // 2. Frequent restart
            if msg.contains("started") || msg.contains("restarted") {
                if !unit.is_empty() {
                    let count = restart_counts.entry(unit.clone()).or_insert(0);
                    *count += 1;
                }
            }

            // 3. Log tampering
            if msg.contains("journal") && (msg.contains("stopped") || msg.contains("deleted") || msg.contains("rotated")) {
                suspicious_events.push(SuspiciousEvent::LogTampering {
                    msg: entry.message.clone().unwrap_or_default(),
                });
            }

            // 4. UID 0 but not root
            if entry.uid.as_deref() == Some("0") {
                if let Some(id) = &entry.syslog_identifier {
                    if id != "root" && !msg.contains("session closed") {
                        suspicious_events.push(SuspiciousEvent::SuspiciousUID0 {
                            uid: id.clone(),
                            msg: entry.message.unwrap_or_default(),
                        });
                    }
                }
            }

            // 5. Unknown service
            if !unit.is_empty() && !known_services.contains(&unit) {
                known_services.insert(unit.clone());
                suspicious_events.push(SuspiciousEvent::UnknownService {
                    unit: unit.clone(),
                });
            }
        }
    }

    for (ip, count) in bruteforce_ips {
        if count > 5 {
            suspicious_events.push(SuspiciousEvent::BruteforceLogin { ip, attempts: count });
        }
    }

    for (unit, count) in restart_counts {
        if count > 5 {
            suspicious_events.push(SuspiciousEvent::FrequentRestart { unit, count });
        }
    }

    Ok(suspicious_events)
}

pub fn sysd_deepscan(save: bool) -> Result<()> {
    let events = detect_suspicious_events()?;

    println!("=== Suspicious System Events Detected ===");
    let mut output = String::new();
    for event in events {
        match event {
            SuspiciousEvent::SuspiciousUID0 { uid, msg } => {
                println!("[UID 0 Activity] {} ran: {}", uid, msg);
                output.push_str(&format!("[UID 0 Activity] {} ran: {}\n", uid, msg));
            }
            SuspiciousEvent::BruteforceLogin { ip, attempts } => {
                println!("[Bruteforce] {} => {} attempts", ip, attempts);
                output.push_str(&format!("[Bruteforce] {} => {} attempts\n", ip, attempts));
            }
            SuspiciousEvent::FrequentRestart { unit, count } => {
                println!("[Frequent Restart] {} restarted {} times", unit, count);
                output.push_str(&format!("[Frequent Restart] {} restarted {} times\n", unit, count));
            }
            SuspiciousEvent::LogTampering { msg } => {
                println!("[Log Tampering] {}", msg);
                output.push_str(&format!("[Log Tampering] {}\n", msg));
            }
            SuspiciousEvent::UnknownService { unit } => {
                println!("[Unknown Service] {}", unit);
                output.push_str(&format!("[Unknown Service] {}\n", unit));
            }
        }
    }
    if output.is_empty() {
        println!("No suspicious events detected.");
    } else if save {
        let path = format!("suspicious_events_{}.log", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        fs::write(&path, output)
            .with_context(|| format!("Unable to write file {}", path))?;
        println!("Suspicious events saved to {}", path);
    }
    Ok(())
}
