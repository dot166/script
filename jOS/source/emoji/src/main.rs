use reqwest::blocking::get;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs::{self};
use std::path::PathBuf;
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct KaomojiRoot {
    name: String,
    categories: Vec<KaomojiCategory>,
}

#[derive(Debug, Deserialize)]
struct KaomojiCategory {
    name: String,
    emoticons: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        show_usage(&args);
    }

    let verbose;
    if args.len() == 3 {
        if args[2] != "-v" {
            show_usage(&args);
        }
        verbose = &args[2] == "-v";
    } else {
        if !args[1].starts_with("http") {
            show_usage(&args);
        }
        verbose = false;
    }

    let emoji_url = &args[1];
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
        ("Smileys & Emotion - boring", "emoji_eight_smiley_people_boring"),
        ("KaomojisJoy", "emoji_kaomojis_joy"),
        ("KaomojisLove", "emoji_kaomojis_love"),
        ("KaomojisEmbarrassment", "emoji_kaomojis_embarrassment"),
        ("KaomojisSympathy", "emoji_kaomojis_sympathy"),
        ("KaomojisDissatisfaction", "emoji_kaomojis_dissatisfaction"),
        ("KaomojisAnger", "emoji_kaomojis_anger"),
        ("KaomojisSadness", "emoji_kaomojis_sadness"),
        ("KaomojisPain", "emoji_kaomojis_pain"),
        ("KaomojisFear", "emoji_kaomojis_fear"),
        ("KaomojisIndifference", "emoji_kaomojis_indifference"),
        ("KaomojisConfusion", "emoji_kaomojis_confusion"),
        ("KaomojisDoubt", "emoji_kaomojis_doubt"),
        ("KaomojisSurprise", "emoji_kaomojis_surprise"),
        ("KaomojisGreeting", "emoji_kaomojis_greeting"),
        ("KaomojisHugging", "emoji_kaomojis_hugging"),
        ("KaomojisWinking", "emoji_kaomojis_winking"),
        ("KaomojisApologizing", "emoji_kaomojis_apologizing"),
        ("KaomojisNosebleeding", "emoji_kaomojis_nosebleeding"),
        ("KaomojisHiding", "emoji_kaomojis_hiding"),
        ("KaomojisWriting", "emoji_kaomojis_writing"),
        ("KaomojisRunning", "emoji_kaomojis_running"),
        ("KaomojisSleeping", "emoji_kaomojis_sleeping"),
        ("KaomojisCat", "emoji_kaomojis_cat"),
        ("KaomojisBear", "emoji_kaomojis_bear"),
        ("KaomojisDog", "emoji_kaomojis_dog"),
        ("KaomojisRabbit", "emoji_kaomojis_rabbit"),
        ("KaomojisPig", "emoji_kaomojis_pig"),
        ("KaomojisBird", "emoji_kaomojis_bird"),
        ("KaomojisSpider", "emoji_kaomojis_spider"),
        ("KaomojisFriends", "emoji_kaomojis_friends"),
        ("KaomojisEnemies", "emoji_kaomojis_enemies"),
        ("KaomojisMagic", "emoji_kaomojis_magic"),
        ("KaomojisFood", "emoji_kaomojis_food"),
        ("KaomojisMusic", "emoji_kaomojis_music"),
        ("KaomojisGames", "emoji_kaomojis_games"),
        ("KaomojisFaces", "emoji_kaomojis_faces"),
        ("KaomojisSpecial", "emoji_kaomojis_special")
    ]);

    let exe = env::current_exe().unwrap();
    let current_dir = exe.parent().expect("Could not get current dir");
    if verbose {
        println!("{:?}", current_dir);
    }
    let relative_path = PathBuf::from("../../platform_packages_inputmethods_LatinIME/java/res/values-v19/emoji-categories.xml");
    let target_path = current_dir.join(&relative_path);
    let template_path = current_dir.join("source/emoji/template.xml");
    let template_content = fs::read_to_string(&template_path)?;
    // inject emoticons into arrays
    for line in fs::read_to_string(current_dir.join("source/emoji/emoticons"))?.lines() {
        emoji_by_group.entry("Emoticons".parse().unwrap()).or_default().push(line.parse().unwrap());
    }
    
    let kaomoji_json_path = current_dir.join("source/emoji/kaomojis.json");
    let kaomoji_data = fs::read_to_string(kaomoji_json_path)?;
    let kaomojis = parse_kaomojis_json(&kaomoji_data)?;

    let kaomojis = dedup_preserve_order(kaomojis);

    emoji_by_group.extend(kaomojis);

    let mut updated = template_content.clone();
    for (group, items) in emoji_by_group {
        if let Some(array_name) = group_to_array.get(group.as_str()) {
            updated = update_android_array(&updated, array_name, &items, verbose);
        } else {
            eprintln!("Skipping group '{}': no array name mapping.", group);
        }
    }

    fs::write(target_path, &updated)?;

    println!("Successfully updated emoji xml files");
    Ok(())
}

fn dedup_preserve_order(mut map: HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>> { 
    for items in map.values_mut() {
        let mut seen = HashSet::new();
        items.retain(|item| seen.insert(item.clone()));
    }
    map
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

fn parse_kaomojis_json(data: &str) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
    let roots: Vec<KaomojiRoot> = serde_json::from_str(data)?;

    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    for root in roots {
        for category in root.categories {
            for emoticon in category.emoticons {
                result.entry("Kaomojis".to_owned() + &category.name).or_default().push(emoticon);
            }
        }
    }

    Ok(result)
}

fn show_usage(args: &Vec<String>) {
    eprintln!("Usage: {} [url|https://unicode.org/Public/emoji/16.0/emoji-test.txt] {{-v(Verbose)}}", args[0]);
    std::process::exit(1);
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace("\\", "\\\\")
     .replace('\'', "\\'")
}

pub fn update_android_array(content: &str, array_name: &str, items: &[String], verbose: bool) -> String {
    let updated = content.to_string();

    let array_re = Regex::new(&format!(
        r#"(?s)<array[^>]*\bname\s*=\s*"{0}"[^>]*>.*?</array>"#,
        regex::escape(array_name)
    )).unwrap();

    let items_str = items
        .iter()
        .map(|item| {
            let escaped = escape_xml(item);
            format!("        <item>{}</item>", escaped)
        })
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
        println!("changed lines in {} = {}", array_name, replacement.lines().count());
    }

    array_re.replace(&updated, replacement).to_string()
}
