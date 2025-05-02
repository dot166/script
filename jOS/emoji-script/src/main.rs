use regex::Regex;
use reqwest::blocking::get;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::{self};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} [url|https://unicode.org/Public/emoji/16.0/emoji-test.txt] {{-v(Verbose)}}", args[0]);
        std::process::exit(1);
    }

    let emoji_url = &args[1];
    let verbose = args.contains(&"-v".to_string());
    let emoji_data = get(emoji_url)?.text()?;
    let mut emoji_by_group = parse_emoji_test_grouped(&emoji_data);
    let group_to_array: HashMap<&str, &str> = HashMap::from([
        ("Smileys & Emotion", "emoji_eight_smiley_people"),
        ("Animals & Nature", "emoji_eight_animals_nature"),
        ("Food & Drink", "emoji_eight_food_drink"),
        ("Travel & Places", "emoji_eight_travel_places"),
        ("Activities", "emoji_eight_activity"),
        ("Objects", "emoji_eight_objects"),
        ("Symbols", "emoji_eight_symbols"),
        ("Flags", "emoji_eight_flags"),
        ("Emoticons", "emoji_emoticons"),
        ("Smileys & Emotion - boring", "emoji_eight_smiley_people_boring")
    ]);

    // Step 2: Load fallback.xml
    let exe = env::current_exe().unwrap();
    let current_dir = exe.parent().expect("Could not get current dir");
    if verbose {
        println!("{:?}", current_dir);
    }
    let relative_path = PathBuf::from("../../platform_packages_inputmethods_LatinIME/java/res/values-v19/emoji-categories.xml");
    let target_path = current_dir.join(&relative_path);
    let fallback_path = current_dir.join("emoji-script/fallback.xml");
    let fallback_content = fs::read_to_string(&fallback_path)?;
    // inject emoticons into arrays
    for line in fs::read_to_string(current_dir.join("emoji-script/emoticons"))?.lines() {
        emoji_by_group.entry("Emoticons".parse().unwrap()).or_default().push(line.parse().unwrap());
    }

    // Step 3: Update fallback.xml content
    let mut updated = fallback_content.clone();
    for (group, items) in emoji_by_group {
        if let Some(array_name) = group_to_array.get(group.as_str()) {
            if verbose {
                println!("{}", array_name);
            }
            updated = update_emoji_array(&updated, array_name, &items, verbose);
        } else {
            eprintln!("Skipping group '{}': no array name mapping.", group);
        }
    }

    if verbose {
        println!("{}", &updated);
    }

    // Step 4: Write updated content
    fs::write(target_path, &updated)?;
    fs::write(fallback_path, &updated)?;

    println!("Successfully updated emoji xml files");
    Ok(())
}

/// Parses emoji-test.txt into group → Vec<emoji>
fn parse_emoji_test_grouped(data: &str) -> HashMap<String, Vec<String>> {
    let mut emoji_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut current_group = String::new();

    for line in data.lines() {
        if line.starts_with("# group: ") && !line.contains("Component") { // remove the component line to prevent clogging stdout with errors (because components are not in the pixel keyboard (gboard) emoji panel)
            if line["# group: ".len()..].trim().to_string() == "People & Body" {
                current_group = "Smileys & Emotion".parse().unwrap(); // merge people and body into the smileys and emotion category because AOSP things
            } else {
                current_group = line["# group: ".len()..].trim().to_string();
            }
        } else if line.contains("; fully-qualified") && !line.contains("skin tone") {
            if let Some((codepoints, _)) = line.split_once(';') {
                let code_str = codepoints
                    .trim()
                    .split_whitespace()
                    .map(|cp| cp.to_uppercase())
                    .collect::<Vec<_>>()
                    .join(",");
                emoji_map.entry(current_group.clone()).or_default().push(code_str.clone());
                if current_group == "Smileys & Emotion" {
                    emoji_map.entry(current_group.clone() + " - boring").or_default().push(code_str);
                }
            }
        }
    }

    emoji_map
}

/// Replaces <array name="..."> with new <item> lines
fn update_emoji_array(content: &str, array_name: &str, items: &[String], verbose: bool) -> String {
    let updated = content.to_string();

    let array_re = Regex::new(&format!(
        r#"(?s)<array[^>]*\bname\s*=\s*"{0}"[^>]*>.*?</array>"#,
        regex::escape(array_name)
    )).unwrap();

    let items_str = items
        .iter()
        .map(|item| format!("        <item>{}</item>", item))
        .collect::<Vec<_>>()
        .join("\n");

    let replacement = format!(
        r#"<array
        name="{}"
        format="string"
    >
{}
    </array>"#,
        array_name, items_str
    );
    if verbose {
        println!("{}", replacement);
    }

    array_re.replace(&updated, replacement).to_string()
}