use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
use std::process::Command;

pub async fn main(args: Vec<String>) {
    if args.len() < 3 {
        if let Err(e) = get_block_devices() {
            eprintln!("Error: {}", e);
        }
        return;
    }

    match args[2].as_str() {
        _ => {
            eprintln!("Unknown DISKINFO command: {}", args[2]);
            eprintln!("Use 'help' to see available commands.");
            std::process::exit(1);
        }
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.2} {}", size, UNITS[unit])
}

fn read_to_string<P: AsRef<Path>>(path: P) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn get_block_devices() -> io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        let mounts = fs::read_to_string("/proc/mounts")?;
        let mount_map: HashMap<_, _> = mounts
            .lines()
            .filter_map(|line| {
                let parts: Vec<_> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect();

        let uuid_map = build_uuid_map("/dev/disk/by-uuid");

        for entry in fs::read_dir("/sys/block")? {
            let entry = entry?;
            let disk_name = entry.file_name().into_string().unwrap_or_default();
            let disk_path = format!("/dev/{}", disk_name);

            // Ignore loopback or ram devices
            if disk_name.starts_with("loop") || disk_name.starts_with("ram") {
                continue;
            }

            let size_path = entry.path().join("size");
            let model_path = entry.path().join("device/model");

            let sectors = read_to_string(&size_path)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let size_bytes = sectors * 512;

            println!("\n[Disk] {}", disk_path);
            println!("  Size   : {}", format_size(size_bytes));
            if let Some(model) = read_to_string(model_path) {
                println!("  Model  : {}", model);
            }

            let dev_dir = entry.path().join("");
            let partitions = fs::read_dir(dev_dir)?;

            for part in partitions {
                let part = part?;
                let part_name = part.file_name().into_string().unwrap_or_default();
                if part_name == disk_name {
                    continue;
                }

                let part_path = format!("/dev/{}", part_name);
                let mount = mount_map.get(&part_path).map(|s| s.as_str()).unwrap_or("-");
                let uuid = uuid_map.get(&part_name).map(|s| s.as_str()).unwrap_or("-");
                println!(
                    "    └─ {} - mount: {:<15} - UUID: {}",
                    part_path, mount, uuid
                );
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("diskutil")
            .args(&["list"])
            .output()
            .expect("failed to run diskutil");

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    }

    Ok(())
}

fn build_uuid_map<P: AsRef<Path>>(uuid_dir: P) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(entries) = fs::read_dir(uuid_dir) {
        for entry in entries.flatten() {
            if let Ok(target) = fs::read_link(entry.path()) {
                if let Some(dev_name) = target
                    .file_name()
                    .and_then(|s| s.to_os_string().into_string().ok())
                {
                    if let Some(uuid) = entry.file_name().into_string().ok() {
                        map.insert(dev_name, uuid);
                    }
                }
            }
        }
    }
    map
}
