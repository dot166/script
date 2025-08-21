use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, thread, time};

pub fn get_script_dir() -> Option<PathBuf> {
    let top_file = "build-android/.gitignore";

    // Check the current directory and navigate upwards if necessary
    let current_dir = env::current_dir().unwrap();
    let mut pwd = current_dir.clone();

    while pwd != Path::new("/") {
        let top_path = pwd.join(top_file);
        if top_path.exists() {
            return Some(pwd);
        }
        pwd = pwd.parent().unwrap_or(Path::new("/")).to_path_buf();
    }

    None
}

fn main() {
    unsafe { env::set_var("IS_CI", "true"); }
    env::set_current_dir(&get_script_dir().unwrap()).expect("Failed to change directory");
    let scripts = ["build-android", "emoji", "fork-aosp", "manage", "update-checkout"];
    for script in scripts.iter() {
        println!("Building {}", script);
        env::set_current_dir(&Path::new(script)).unwrap();
        let status = Command::new("cargo").arg("update").status().unwrap();
        if !status.success() {
            panic!("Failed to update dependencies of script: {}", script);
        }
        let status = Command::new("cargo").arg("build").arg("--release").status().unwrap();
        if !status.success() {
            panic!("Failed to build script: {}", script);
        }
        fs::copy("target/release/".to_owned() + script, get_script_dir().unwrap().join("../".to_owned() + script)).unwrap();
        env::set_current_dir(&Path::new("..")).unwrap();
    }
    let status = Command::new("git")
        .arg("commit")
        .arg("../../.")
        .arg("-m")
        .arg(format!("rebuilt scripts {}", chrono::offset::Utc::now().date_naive().format("%Y%m%d")))
        .status();

    if let Err(e) = status {
        panic!("Error committing changes: {}", e);
    }

    let status = Command::new("git")
        .arg("push")
        .status();

    if let Err(e) = status {
        panic!("Error pushing changes: {}", e);
    }
    thread::sleep(time::Duration::from_secs(5));
    fs::create_dir("tmp").unwrap();
    env::set_current_dir("tmp").unwrap();
    let status = Command::new("../../manage").arg("init").status().unwrap();
    if !status.success() {
        panic!("Failed to run init script");
    }
    env::set_current_dir(&Path::new("..")).unwrap();
    fs::remove_dir_all("tmp").unwrap();
}
