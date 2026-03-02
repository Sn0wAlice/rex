use anyhow::{Result, Context};
use std::fs::{self, File};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::Command;

const MOUNT_POINT: &str = "/tmp/rex_mount";

pub fn run(target: &str, all: bool, only_deleted: bool, output_base: &str) -> Result<()> {
    println!("[*] Carving from: {}", target);
    let session_id = uuid::Uuid::new_v4();
    let output_dir = format!("{}/{}", output_base, session_id);

    carve_disk(target, all, only_deleted, &output_dir)
        .context("Error during carving")?;

    if !only_deleted {
        extract_live_files(target, &output_dir)?;
    }
    Ok(())
}

fn carve_disk(path: &str, all: bool, only_deleted: bool, output_dir: &str) -> Result<()> {
    let mut reader = BufReader::new(
        File::open(path).with_context(|| format!("Failed to open {}", path))?
    );
    let mut buffer = vec![0u8; 8192];
    let mut chunk = vec![0u8; 1024 * 1024];
    let mut offset = 0u64;
    let mut file_count = 0;

    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory {}", output_dir))?;
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
                let mut out = File::create(&output_path)
                    .with_context(|| format!("Failed to create {}", output_path))?;
                let size = reader.read(&mut chunk)?;
                out.write_all(&chunk[..size])?;
                file_count += 1;

                offset = found_offset + 1024 * 1024;
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

fn extract_live_files(image_path: &str, output_dir: &str) -> Result<()> {
    fs::create_dir_all(MOUNT_POINT)
        .with_context(|| format!("Failed to create mount point {}", MOUNT_POINT))?;

    #[cfg(target_os = "macos")]
    let status = Command::new("hdiutil")
        .args(["attach", image_path, "-mountpoint", MOUNT_POINT, "-nobrowse"])
        .status()?;

    #[cfg(not(target_os = "macos"))]
    let status = Command::new("mount")
        .args(["-o", "loop", image_path, MOUNT_POINT])
        .status()?;

    if !status.success() {
        eprintln!("Warning: Could not mount image for visible file extraction.");
        return Ok(());
    }

    copy_recursively(Path::new(MOUNT_POINT), Path::new(output_dir))?;

    #[cfg(target_os = "macos")]
    let _ = Command::new("hdiutil")
        .args(["detach", MOUNT_POINT])
        .status();

    #[cfg(not(target_os = "macos"))]
    let _ = Command::new("umount")
        .arg(MOUNT_POINT)
        .status();

    Ok(())
}

fn copy_recursively(src: &Path, dst: &Path) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let rel_path = path.strip_prefix(src)
            .context("Unexpected path prefix mismatch")?;
        let dest_path = dst.join(rel_path);
        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_recursively(&path, &dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}
