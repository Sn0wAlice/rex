use anyhow::{Result, Context};
use std::fs::{self, File};
use std::io::{BufWriter, Write};

fn split_domain(domain: &str) -> (&str, &str) {
    if let Some(pos) = domain.rfind('.') {
        (&domain[..pos], &domain[pos + 1..])
    } else {
        (domain, "")
    }
}

fn omission(s: &str) -> Vec<String> {
    (0..s.len()).map(|i| {
        let mut c = s.to_string();
        c.remove(i);
        c
    }).collect()
}

fn permutation(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut typos = Vec::new();
    for i in 0..(chars.len() - 1) {
        let mut swapped = chars.clone();
        swapped.swap(i, i + 1);
        typos.push(swapped.iter().collect());
    }
    typos
}

fn substitution(s: &str) -> Vec<String> {
    let neighbors = |c: char| -> Vec<char> {
        match c {
            'a' => vec!['q', 's', 'z'],
            'e' => vec!['w', 'r', 'd'],
            'o' => vec!['i', 'p', 'l'],
            'm' => vec!['n', 'j', 'k'],
            _ => vec![],
        }
    };
    let mut typos = Vec::new();
    for (i, c) in s.chars().enumerate() {
        for sub in neighbors(c) {
            let mut chars: Vec<char> = s.chars().collect();
            chars[i] = sub;
            typos.push(chars.iter().collect());
        }
    }
    typos
}

fn repetition(s: &str) -> Vec<String> {
    let mut typos = Vec::new();
    for (i, c) in s.chars().enumerate() {
        let mut chars: Vec<char> = s.chars().collect();
        chars.insert(i, c);
        typos.push(chars.iter().collect());
    }
    typos
}

fn missing_dot(domain: &str) -> Vec<String> {
    if domain.starts_with("www.") {
        vec![domain.replacen("www.", "www", 1)]
    } else {
        vec![]
    }
}

fn homoglyphs(s: &str) -> Vec<String> {
    let map = [('o', '0'), ('i', '1'), ('e', '3'), ('l', '1'), ('s', '5')];
    let mut typos = Vec::new();
    for (from, to) in map {
        if s.contains(from) {
            typos.push(s.replace(from, &to.to_string()));
        }
    }
    typos
}

fn insertion(s: &str) -> Vec<String> {
    (0..=s.len()).map(|i| {
        let mut c = s.to_string();
        c.insert(i, 'x');
        c
    }).collect()
}

fn tld_swap(base: &str) -> Vec<String> {
    let tlds = ["com", "net", "org", "co", "info", "xyz"];
    tlds.iter().map(|tld| format!("{}.{}", base, tld)).collect()
}

fn hyphenation(s: &str) -> Vec<String> {
    let mut typos = Vec::new();
    for i in 1..s.len() {
        let mut c = s.to_string();
        c.insert(i, '-');
        typos.push(c);
    }
    typos
}

fn subdomain_attack(domain: &str) -> Vec<String> {
    vec![
        format!("{}.attacker.com", domain),
        format!("login.{}", domain),
    ]
}

fn combo_squatting(s: &str) -> Vec<String> {
    let combos = ["secure", "login", "support", "mail"];
    combos.iter().map(|c| format!("{}-{}", c, s)).collect()
}

fn bitsquatting(s: &str) -> Vec<String> {
    vec![s.replace("m", "n"), s.replace("b", "d")]
}

fn unicode_abuse(s: &str) -> Vec<String> {
    vec![s.replace("o", "\u{043E}")] // Cyrillic 'о'
}

fn generate_with_methods(domain: &str, methods: &[&str]) -> Vec<String> {
    let (base, tld) = split_domain(domain);
    let base_only = base.replace('.', "");

    let mut all = Vec::new();
    for method in methods {
        let generated = match *method {
            "omission" => omission(&base_only),
            "permutation" => permutation(&base_only),
            "substitution" => substitution(&base_only),
            "repetition" => repetition(&base_only),
            "missingdot" => missing_dot(domain),
            "homoglyph" => homoglyphs(&base_only),
            "insertion" => insertion(&base_only),
            "tldswap" => tld_swap(&base_only),
            "hyphenation" => hyphenation(&base_only),
            "subdomain" => subdomain_attack(domain),
            "combo" => combo_squatting(&base_only),
            "bitsquatting" => bitsquatting(&base_only),
            "unicode" => unicode_abuse(&base_only),
            _ => {
                eprintln!("Unknown method: {}", method);
                vec![]
            }
        };
        all.extend(generated);
    }

    let all = all
        .into_iter()
        .map(|d| if d.contains('.') { d } else { format!("{}.{}", d, tld) })
        .collect::<Vec<_>>();

    let mut unique = all;
    unique.sort();
    unique.dedup();
    unique
}

fn write_to_file(path: &str, typos: &[String]) -> Result<()> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    let file = File::create(path)
        .with_context(|| format!("Failed to create {}", path))?;
    let mut writer = BufWriter::new(file);
    for typo in typos {
        writeln!(writer, "{}", typo)?;
    }
    println!("Output file: {}", path);
    Ok(())
}

pub fn generate_list(domain: &str, output: Option<&str>, method_arg: Option<&str>) -> Result<()> {
    let all_methods = vec![
        "omission", "permutation", "substitution", "repetition",
        "missingdot", "homoglyph", "insertion", "tldswap",
        "hyphenation", "subdomain", "combo", "bitsquatting", "unicode",
    ];

    let methods: Vec<&str> = match method_arg {
        Some(m) => m.split(',').collect(),
        None => all_methods,
    };

    println!("Generating typos for domain: {}", domain);
    println!("Using methods: {}", methods.join(", "));

    let typos = generate_with_methods(domain, &methods);

    let default_output = format!("./output/typo/{}.txt", domain);
    let output_path = output.unwrap_or(&default_output);

    write_to_file(output_path, &typos)
        .with_context(|| format!("Failed to write output to {}", output_path))?;

    Ok(())
}
