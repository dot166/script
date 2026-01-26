use lib_aosp::scripts;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Expected 1 argument");
        std::process::exit(1);
    }

    let repo = args[1].trim_end_matches('/').to_string();
    let local_repo = repo.replace('/', "_");

    let (aosp_tag, _, branch) = scripts::read_common_sh();

    let upstream = format!("https://android.googlesource.com/{}", repo);

    // Clone the repository
    Command::new("git")
        .args(["clone", &upstream, "-b", &aosp_tag])
        .status()
        .expect("Failed to clone repository");

    // Move the cloned repository
    let repo_name = Path::new(&repo).file_name().unwrap().to_str().unwrap();
    fs::rename(repo_name, &local_repo).expect("Failed to rename repository directory");

    // Change into the new repository directory
    env::set_current_dir(&local_repo).expect("Failed to change directory");

    // Checkout a new branch
    Command::new("git")
        .args(["checkout", "-b", &branch])
        .status()
        .expect("Failed to checkout new branch");

    // Add upstream remote
    Command::new("git")
        .args(["remote", "add", "upstream", &upstream])
        .status()
        .expect("Failed to add upstream remote");

    // Fetch upstream tags
    Command::new("git")
        .args(["fetch", "upstream", "--tags"])
        .status()
        .expect("Failed to fetch upstream tags");

    // Remove origin remote
    Command::new("git")
        .args(["remote", "rm", "origin"])
        .status()
        .expect("Failed to remove origin remote");

    // Create a new GitHub repository
    let repo_name = format!("dot166/{}", local_repo);
    Command::new("gh")
        .args([
            "repo",
            "create",
            "--public",
            "--push",
            "--source",
            ".",
            &repo_name,
            "-h",
            "https://dot166.github.io/jOS/",
            "--disable-issues",
            "--disable-wiki",
        ])
        .status()
        .expect("Failed to create GitHub repository");

    // Set default repository
    Command::new("gh")
        .args(["repo", "set-default", &repo_name])
        .status()
        .expect("Failed to set default repository");

    // Edit repository settings
    Command::new("gh")
        .args([
            "repo",
            "edit",
            "--enable-projects=false",
            "--enable-merge-commit=false",
        ])
        .status()
        .expect("Failed to edit repository settings");

    // View repository on web
    Command::new("gh")
        .args(["repo", "view", "--web"])
        .status()
        .expect("Failed to view repository on web");
}
