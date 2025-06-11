use lopdf::{Document, Object, Stream};
use std::fs::{create_dir_all, File};
use std::io::Write;

pub async fn main(args: Vec<String>) {
    if args.len() < 4 {
        eprintln!("Usage: {} pdf <command> <args>", args[0]);
        std::process::exit(1);
    }

    match args[2].as_str() {
        "extract" => {
            extract_pdf(args[3].clone());
        }
        _ => {
            eprintln!("Unknown PDF command: {}", args[1]);
        }
    }
}

fn extract_metadata(doc: &Document, output_dir: &str) {
    let info_ref = doc.trailer.get(b"Info")
        .and_then(|obj| obj.as_reference())
        .ok();

    let info_ref = match info_ref {
        Some(ref_obj) => ref_obj,
        None => {
            eprintln!("No metadata found in the PDF.");
            return;
        }
    };

    let info_obj = doc.get_object(info_ref).unwrap();

    if let Object::Dictionary(info_dict) = info_obj {
        let mut metadata = String::new();
        for (k, v) in info_dict.iter() {
            metadata.push_str(&format!("{}: {:?}\n", String::from_utf8_lossy(k), v));
        }
        std::fs::write(format!("{}/metadata.txt", output_dir), metadata).unwrap();
    }
}

fn extract_pdf(path: String) {
    let output_dir = "output";
    create_dir_all(format!("{}/images", output_dir)).unwrap();
    create_dir_all(format!("{}/js", output_dir)).unwrap();
    create_dir_all(format!("{}/raw_objects", output_dir)).unwrap();

    let doc = Document::load(path).expect("Failed to load PDF document");

    // Extraction des métadonnées
    extract_metadata(&doc, output_dir);

    let mut image_id = 0;
    let mut js_id = 0;

    // Parcours des objets
    for (id, object) in &doc.objects {
        // Dump brut
        let raw_path = format!("{}/raw_objects/obj_{}.txt", output_dir, id.0);
        let mut raw_file = File::create(&raw_path).unwrap();
        writeln!(raw_file, "{:#?}", object).unwrap();

        // Extraction des flux
        if let Ok(stream) = object.as_stream() {

            // JS
            if let Ok(type_obj) = stream.dict.get(b"Type") {
                if let Ok(name) = type_obj.as_name() {
                    let type_name = String::from_utf8_lossy(name);
                    if type_name == "JavaScript" {
                        let js_content = String::from_utf8_lossy(&stream.content);
                        let js_path = format!("{}/js/script_{}.js", output_dir, js_id);
                        std::fs::write(js_path, js_content.as_bytes()).unwrap();
                        js_id += 1;
                    } else if type_name == "XObject" {
                        // XObject, potentially an image
                        println!("Found XObject stream with ID: {}", id.0);
                    } else {
                        println!("Found stream with Type: {}", type_name);
                    }
                }
            }

            // Images
            if let Ok(subtype) = stream.dict.get(b"Subtype") {
                if let Ok(name) = subtype.as_name() {
                    let subtype_name = String::from_utf8_lossy(name);
                    if subtype_name == "Image" {
                        println!("Found image stream with ID: {}", id.0);
                        std::fs::write(format!("{}/images/image_{}.raw", output_dir, image_id), format!("{:?}", stream.dict)).unwrap();
                        let image_path = format!("{}/images/image_{}.unknowext.png", output_dir, image_id);
                        let mut image_file = File::create(&image_path).unwrap();
                        image_file.write_all(&stream.content).unwrap();
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
}