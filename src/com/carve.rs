use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};

pub async fn main(args: Vec<String>) {
    if args.len() < 3 {
        eprintln!("Usage: rex carve <path_to_device_or_img> [--all]");
        return;
    }

    carve_entry(&args);
}

pub fn carve_entry(args: &[String]) {
    let mut target = "";
    let mut all_flag = false;
    let mut only_deleted = false;

    for arg in args.iter().skip(2) {
        if arg == "--all" {
            all_flag = true;
        } else if arg == "--only-deleted" {
            only_deleted = true;
        } else {
            target = arg;
        }
    }

    if target.is_empty() && !all_flag {
        eprintln!("Usage: rex carve <path_to_device_or_img> [--all] [--only-deleted]");
        return;
    }

    println!("[*] Carving from: {}", target);
    let session_id = uuid::Uuid::new_v4();
    let output_dir = format!("recovered/{}", session_id);
    if let Err(e) = carve_disk(target, all_flag, only_deleted, &output_dir) {
        eprintln!("Error during carving: {}", e);
        return;
    }
    if !only_deleted {
        let _ = extract_live_files(target, &output_dir);
    }
}

fn carve_disk(path: &str, all: bool, only_deleted: bool, output_dir: &str) -> io::Result<()> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut buffer = vec![0u8; 8192];
    let mut chunk = vec![0u8; 1024 * 1024];
    let mut offset = 0u64;
    let mut file_count = 0;

    fs::create_dir_all(output_dir)?;
    println!("[*] Recovery session started in {}", output_dir);

    loop {
        reader.seek(SeekFrom::Start(offset))?;
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        let mut found = false;
        for slide in (0..n.saturating_sub(32)).step_by(64) {
            let window = &buffer[slide..];
            if let Some(ext) = detect_signature(window) {
                let found_offset = offset + slide as u64;

                if only_deleted && found_offset < 1024 * 512 {
                    continue;
                }

                let output_path = format!("{}/file_{}_{}.{}", output_dir, file_count, found_offset, ext);
                println!("Found {} at offset {} -> {}", ext, found_offset, output_path);

                reader.seek(SeekFrom::Start(found_offset))?;
                let mut out = File::create(&output_path)?;
                let size = reader.read(&mut chunk)?;
                out.write_all(&chunk[..size])?;
                file_count += 1;

                offset = found_offset + 1024 * 1024; // Skip 1MB to avoid overlapping detections
                found = true;
                break;
            }
        }

        if !found {
            offset += 512;
        }

        if !all && file_count >= 10 {
            break;
        }
    }

    Ok(())
}

fn detect_signature(buf: &[u8]) -> Option<&'static str> {
    if let Some(kind) = infer::get(buf) {
        Some(kind.extension())
    } else {
        let printable = buf.iter().filter(|&&b| b == 0x09 || b == 0x0A || b == 0x0D || (b >= 0x20 && b <= 0x7E)).count();
        if printable * 100 / buf.len() > 90 {
            Some("txt")
        } else {
            None
        }
    }
}

fn extract_live_files(image_path: &str, output_dir: &str) -> io::Result<()> {
    use std::process::Command;
    use std::path::Path;

    let mount_point = "/tmp/rex_mount";
    fs::create_dir_all(mount_point)?;

    // Attempt to mount the image using hdiutil (macOS) or mount (Linux)
    #[cfg(target_os = "macos")]
    let status = Command::new("hdiutil")
        .args(["attach", image_path, "-mountpoint", mount_point, "-nobrowse"])
        .status()?;

    #[cfg(not(target_os = "macos"))]
    let status = Command::new("mount")
        .args(["-o", "loop", image_path, mount_point])
        .status()?;

    if !status.success() {
        eprintln!("Warning: Could not mount image for visible file extraction.");
        return Ok(());
    }

    // Copy all files preserving directory structure
    fn copy_recursively(src: &Path, dst: &Path) -> io::Result<()> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let rel_path = path.strip_prefix(src).unwrap();
            let dest_path = dst.join(rel_path);
            if path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                copy_recursively(&path, &dest_path)?;
            } else {
                fs::create_dir_all(dest_path.parent().unwrap())?;
                fs::copy(&path, &dest_path)?;
            }
        }
        Ok(())
    }

    copy_recursively(Path::new(mount_point), Path::new(output_dir))?;

    // Unmount
    #[cfg(target_os = "macos")]
    let _ = Command::new("hdiutil")
        .args(["detach", mount_point])
        .status();

    #[cfg(not(target_os = "macos"))]
    let _ = Command::new("umount")
        .arg(mount_point)
        .status();

    Ok(())
}
