use std::collections::HashMap;
use std::env;
use std::fs::{self};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 4 {
        eprintln!("Usage: emoji [url|https://unicode.org/Public/emoji/16.0/emoji-test.txt] {{-F(Force fallback)}} {{-v(Verbose)}}");
        std::process::exit(1);
    }

    let url = &args[1];
    let force_fallback = args.contains(&"-F".to_string());
    let verbose = args.contains(&"-v".to_string());

    let response = match reqwest::get(url).await {
        Ok(resp) => match resp.text().await {
            Ok(text) => text,
            Err(e) => {
                eprintln!("Failed to read response: {}", e);
                return;
            }
        },
        Err(e) => {
            eprintln!("Request error: {}", e);
            return;
        }
    };

    let mut group_name = String::new();
    let mut items: HashMap<String, Vec<String>> = HashMap::new();

    for line in response.lines() {
        if let Some(subgroup) = line.strip_prefix("# subgroup: ") {
            group_name = subgroup.trim().to_string();
        } else if line.contains("; fully-qualified") && !line.contains("skin tone") {
            if let Some((codepoints, _)) = line.split_once(";") {
                let item = codepoints.trim().replace(' ', ",");
                items.entry(group_name.clone()).or_default().push(item);
            }
        }
    }

    let current_dir = env::current_dir().expect("Could not get current dir");
    println!("{:?}", current_dir);
    let relative_path = PathBuf::from("../../../platform_packages_inputmethods_LatinIME/java/res/values-v19/emoji-categories.xml");
    let target_path = current_dir.join(&relative_path);
    println!("{:?}", target_path);

    let mut input_path = target_path.clone();
    if !input_path.exists() || force_fallback {
        input_path = current_dir.join("emoji-script/fallback.xml");
    }

    let content = fs::read_to_string(&input_path).expect("Could not read input file");

    let mut updated_content = content.clone();

    for (key, group_items) in &items {
        let header = format!("<!-- {} -->", key);
        if let Some(start) = updated_content.find(&header) {
            let after_header = &updated_content[start..];
            let end1 = after_header.find("</array>").map(|i| i + start);
            let end2 = after_header.find("<!--").map(|i| i + start + 1);

            let min_end = match (end1, end2) {
                (Some(e1), Some(e2)) => e1.min(e2),
                (Some(e1), None) => e1,
                (None, Some(e2)) => e2,
                (None, None) => continue,
            };

            let replace_section = updated_content[start..min_end].trim();
            let mut built = format!("{}\n", header);
            for item in group_items {
                built.push_str(&format!("        <item>{}</item>\n", item));
            }

            updated_content = updated_content.replacen(replace_section, built.trim_end(), 1);
        }
    }

    println!("Updating emoji-categories.xml in LatinIME");
    if verbose {
        println!("{}", updated_content);
    }

    if target_path.exists() && force_fallback {
        fs::remove_file(&target_path).ok();
    }

    fs::write(&target_path, &updated_content).expect("Failed to write updated emoji-categories.xml");

    println!("Updating fallback.xml");
    fs::write(current_dir.join("fallback.xml"), updated_content).expect("Failed to write fallback.xml");

    println!("Done!");
}