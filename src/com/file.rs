use anyhow::{Result, Context};
use lopdf::{Document, Object};
use std::fs::{create_dir_all, File};
use std::io::Write;

pub fn extract_pdf(path: &str, output_dir: &str) -> Result<()> {
    create_dir_all(format!("{}/images", output_dir))
        .context("Failed to create images output directory")?;
    create_dir_all(format!("{}/js", output_dir))
        .context("Failed to create js output directory")?;
    create_dir_all(format!("{}/raw_objects", output_dir))
        .context("Failed to create raw_objects output directory")?;

    let doc = Document::load(path)
        .with_context(|| format!("Failed to load PDF document: {}", path))?;

    extract_metadata(&doc, output_dir)?;

    let mut image_id = 0;
    let mut js_id = 0;

    for (id, object) in &doc.objects {
        let raw_path = format!("{}/raw_objects/obj_{}.txt", output_dir, id.0);
        let mut raw_file = File::create(&raw_path)
            .with_context(|| format!("Failed to create {}", raw_path))?;
        writeln!(raw_file, "{:#?}", object)
            .with_context(|| format!("Failed to write to {}", raw_path))?;

        if let Ok(stream) = object.as_stream() {
            // JS extraction
            if let Ok(type_obj) = stream.dict.get(b"Type") {
                if let Ok(name) = type_obj.as_name() {
                    let type_name = String::from_utf8_lossy(name);
                    if type_name == "JavaScript" {
                        let js_content = String::from_utf8_lossy(&stream.content);
                        let js_path = format!("{}/js/script_{}.js", output_dir, js_id);
                        std::fs::write(&js_path, js_content.as_bytes())
                            .with_context(|| format!("Failed to write JS file {}", js_path))?;
                        js_id += 1;
                    } else if type_name == "XObject" {
                        println!("Found XObject stream with ID: {}", id.0);
                    } else {
                        println!("Found stream with Type: {}", type_name);
                    }
                }
            }

            // Image extraction
            if let Ok(subtype) = stream.dict.get(b"Subtype") {
                if let Ok(name) = subtype.as_name() {
                    let subtype_name = String::from_utf8_lossy(name);
                    if subtype_name == "Image" {
                        println!("Found image stream with ID: {}", id.0);
                        let raw_img_path = format!("{}/images/image_{}.raw", output_dir, image_id);
                        std::fs::write(&raw_img_path, format!("{:?}", stream.dict))
                            .with_context(|| format!("Failed to write {}", raw_img_path))?;
                        let image_path = format!("{}/images/image_{}.unknowext.png", output_dir, image_id);
                        let mut image_file = File::create(&image_path)
                            .with_context(|| format!("Failed to create {}", image_path))?;
                        image_file.write_all(&stream.content)
                            .with_context(|| format!("Failed to write {}", image_path))?;
                        image_id += 1;
                    } else if subtype_name == "Form" {
                        println!("Found Form stream with ID: {}", id.0);
                    } else {
                        println!("Found stream with Subtype: {}", subtype_name);
                    }
                }
            }
        }
    }
    println!("Finished extracting PDF, output saved to '{}'", output_dir);
    Ok(())
}

fn extract_metadata(doc: &Document, output_dir: &str) -> Result<()> {
    let info_ref = doc.trailer.get(b"Info")
        .and_then(|obj| obj.as_reference())
        .ok();

    let info_ref = match info_ref {
        Some(ref_obj) => ref_obj,
        None => {
            eprintln!("No metadata found in the PDF.");
            return Ok(());
        }
    };

    let info_obj = doc.get_object(info_ref)
        .context("Failed to get PDF info object")?;

    if let Object::Dictionary(info_dict) = info_obj {
        let mut metadata = String::new();
        for (k, v) in info_dict.iter() {
            metadata.push_str(&format!("{}: {:?}\n", String::from_utf8_lossy(k), v));
        }
        std::fs::write(format!("{}/metadata.txt", output_dir), metadata)
            .context("Failed to write metadata.txt")?;
    }
    Ok(())
}
