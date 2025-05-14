use std::{env, fs};
use std::process::{exit, Command};
use lib_aosp::scripts;

fn main() {
    let (mut aosp_tag, mut aosp_tag_old, mut branch) = scripts::read_common_sh();
    let (graphene_tag, graphene_tag_old, lineage_latest_branch) = scripts::read_config_file();
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 { panic!("expected action as argument");}
    let action= args[1].clone();
    let mut tag_name = "";

    if action == "update" || action == "default" || action == "init" || action == "bupdate" {
        if env::var("IS_CI").unwrap() == "true" && (action != "update" && action != "init") {
            println!("cannot use {} in ci, this is done to prevent the ci from destroying the source tree", action);
            exit(0);
        }
        if args.len() != 2 {panic!("expected no arguments for $action");}
    } else if action == "release" || action == "delete" {
        if env::var("IS_CI").unwrap() == "true" {
            println!("cannot use {} in ci, this is done to prevent the ci from destroying the source tree", action);
            exit(0);
        }
        tag_name = &args[2];
        if args.len() != 3 {panic!("expected tag name as argument for $action");}
    } else {
        panic!("unrecognized action");
    }
    let aosp_forks=[
        "platform_frameworks_opt_calendar",
        "platform_frameworks_opt_colorpicker",
        "platform_frameworks_opt_timezonepicker",
    ];

    let grapheneos_forks=[
        "platform_bootable_recovery",
        "platform_build",
        "platform_build_release",
        "platform_frameworks_base",
        "jOS_manifest",
        "platform_packages_apps_DeskClock",
        "platform_packages_apps_Dialer",
        "platform_packages_apps_ExactCalculator",
        "platform_packages_apps_Launcher3",
        "platform_packages_apps_Settings",
        "platform_packages_apps_SetupWizard2",
        "platform_packages_apps_ThemePicker",
        "platform_packages_apps_Updater",
        "platform_packages_inputmethods_LatinIME",
        "platform_packages_services_telecomm",
    ];

    let lineageos_forks=[
        "platform_packages_apps_Recorder",
        "platform_packages_apps_Etar",
    ];

    let independent=[
        "jOS_j-lib",
        "jOS-System",
        "jOS-Updates",
    ];

    println!("\n>>> Handling script");

        if action == "init" {
            let status = Command::new("git")
                .arg("clone")
                .arg("https://github.com/dot166/script")
                .status();

            if let Err(e) = status {
                eprintln!("Error cloning script: {}", e);
                exit(1);
            }
        }

        if let Err(e) = env::set_current_dir("script") {
            eprintln!("Failed to change directory to script: {}", e);
            exit(1);
        }

        if action != "bupdate" {
            let status = Command::new("git")
                .arg("checkout")
                .arg(&branch)
                .status();

            if let Err(e) = status {
                eprintln!("Error checking out branch {}: {}", &branch, e);
                exit(1);
            }
        }

        let status = Command::new("git")
            .arg("pull")
            .status();

        if let Err(e) = status {
            eprintln!("Error pulling changes for script: {}", e);
            exit(1);
        }

        match action.as_str() {
            "delete" => {
                let _ = Command::new("git")
                    .arg("tag")
                    .arg("-d")
                    .arg(tag_name)
                    .status();

                let _ = Command::new("git")
                    .arg("push")
                    .arg("origin")
                    .arg("--delete")
                    .arg(tag_name)
                    .status();
            },
            "release" => {
                let status = Command::new("git")
                    .arg("tag")
                    .arg("-s")
                    .arg(tag_name)
                    .arg("-m")
                    .arg(tag_name)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error creating release tag {}: {}", tag_name, e);
                    exit(1);
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("origin")
                    .arg(tag_name)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error pushing release tag {}: {}", tag_name, e);
                    exit(1);
                }
            },
            "update" => {
                let status = Command::new("git")
                    .arg("fetch")
                    .arg("upstream")
                    .arg("--tags")
                    .arg("--force")
                    .status();

                if let Err(e) = status {
                    eprintln!("Error fetching upstream tags: {}", e);
                    exit(1);
                }

                let status = Command::new("git")
                    .arg("rebase")
                    .arg("--onto")
                    .arg(&graphene_tag)
                    .arg(&graphene_tag_old)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error rebasing script: {}", e);
                    exit(1);
                }

                (aosp_tag, aosp_tag_old, branch) = scripts::read_common_sh();

                let status = Command::new("git")
                    .arg("push")
                    .arg("-f")
                    .status();

                if let Err(e) = status {
                    eprintln!("Error pushing changes for script: {}", e);
                    exit(1);
                }
            },
            "default" => {
                let status = Command::new("gh")
                    .arg("repo")
                    .arg("edit")
                    .arg("dot166/script")
                    .arg("--default-branch")
                    .arg(&branch)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error editing default branch for script: {}", e);
                    exit(1);
                }
            },
            _ => {}
        }

        if action == "init" {
            let remote_url = "https://github.com/grapheneos/script";

            let status = Command::new("git")
                .arg("remote")
                .arg("add")
                .arg("upstream")
                .arg(remote_url)
                .status();

            if let Err(e) = status {
                eprintln!("Error adding upstream for script: {}", e);
                exit(1);
            }

            let status = Command::new("git")
                .arg("fetch")
                .arg("upstream")
                .arg("--tags")
                .status();

            if let Err(e) = status {
                eprintln!("Error fetching upstream tags for script: {}", e);
                exit(1);
            }
        }

    for repo in grapheneos_forks {
        println!("\n>>> Handling {}", repo);

        if action == "init" {
            let status = Command::new("git")
                .arg("clone")
                .arg(format!("https://github.com/dot166/{}", repo))
                .status();

            if let Err(e) = status {
                eprintln!("Error cloning {}: {}", repo, e);
                exit(1);
            }
        }

        if let Err(e) = env::set_current_dir(&repo) {
            eprintln!("Failed to change directory to {}: {}", repo, e);
            exit(1);
        }

        if action != "bupdate" {
            let status = Command::new("git")
                .arg("checkout")
                .arg(&branch)
                .status();

            if let Err(e) = status {
                eprintln!("Error checking out branch {}: {}", &branch, e);
                exit(1);
            }
        } else {
            let status = Command::new("git")
                .arg("checkout")
                .arg("origin")
                .status();

            if let Err(e) = status {
                eprintln!("Error checking out origin for {}: {}", repo, e);
                exit(1);
            }

            let status = Command::new("git")
                .arg("switch")
                .arg("-c")
                .arg(&branch)
                .status();

            if let Err(e) = status {
                eprintln!("Error switching to branch {}: {}", &branch, e);
                exit(1);
            }

            let status = Command::new("git")
                .arg("push")
                .arg("--set-upstream")
                .arg("origin")
                .arg(&branch)
                .status();

            if let Err(e) = status {
                eprintln!("Error pushing {} to upstream: {}", &branch, e);
                exit(1);
            }
        }

        let status = Command::new("git")
            .arg("pull")
            .status();

        if let Err(e) = status {
            eprintln!("Error pulling changes for {}: {}", repo, e);
            exit(1);
        }

        match action.as_str() {
            "delete" => {
                let _ = Command::new("git")
                    .arg("tag")
                    .arg("-d")
                    .arg(tag_name)
                    .status();

                let _ = Command::new("git")
                    .arg("push")
                    .arg("origin")
                    .arg("--delete")
                    .arg(tag_name)
                    .status();
            },
            "release" => {
                if repo == "jOS_manifest" {
                    let status = Command::new("git")
                        .arg("checkout")
                        .arg("-B")
                        .arg("tmp")
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error checking out tmp branch for {}: {}", repo, e);
                        exit(1);
                    }

                    let status = Command::new("sed")
                        .arg("-i")
                        .arg(format!("s%refs/heads/{}%refs/tags/%{}% default.xml", &branch, tag_name))
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error updating default.xml for {}: {}", repo, e);
                        exit(1);
                    }

                    let status = Command::new("git")
                        .arg("commit")
                        .arg("default.xml")
                        .arg("-m")
                        .arg(tag_name)
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error committing default.xml for {}: {}", repo, e);
                        exit(1);
                    }

                    let status = Command::new("git")
                        .arg("push")
                        .arg("-fu")
                        .arg("origin")
                        .arg("tmp")
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error pushing tmp branch for {}: {}", repo, e);
                        exit(1);
                    }
                }

                let status = Command::new("git")
                    .arg("tag")
                    .arg("-s")
                    .arg(tag_name)
                    .arg("-m")
                    .arg(tag_name)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error creating release tag {}: {}", tag_name, e);
                    exit(1);
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("origin")
                    .arg(tag_name)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error pushing release tag {}: {}", tag_name, e);
                    exit(1);
                }
            },
            "update" => {
                let status = Command::new("git")
                    .arg("fetch")
                    .arg("upstream")
                    .arg("--tags")
                    .arg("--force")
                    .status();

                if let Err(e) = status {
                    eprintln!("Error fetching upstream tags: {}", e);
                    exit(1);
                }

                if repo == "jOS_manifest" {
                    let status = Command::new("git")
                        .arg("rebase")
                        .arg("--onto")
                        .arg(format!("{}~1", &graphene_tag))
                        .arg(format!("{}~1", &graphene_tag_old))
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error rebasing {}: {}", repo, e);
                        exit(1);
                    }

                    let status = Command::new("sed")
                        .arg("-i")
                        .arg(format!("s%refs/tags/{}%refs/tags/%{}% default.xml", graphene_tag_old, graphene_tag))
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error updating default.xml for {}: {}", repo, e);
                        exit(1);
                    }

                    let status = Command::new("git")
                        .arg("diff")
                        .arg("--quiet")
                        .status();

                    let changes = if let Ok(status) = status {
                        if status.success() {
                            0
                        } else {
                            1
                        }
                    } else {
                        1
                    };

                    println!("CHANGES={}", changes);

                    if changes == 1 {
                        let status = Command::new("git")
                            .arg("commit")
                            .arg("default.xml")
                            .arg("-m")
                            .arg(format!("GrapheneOS {}", &graphene_tag))
                            .status();

                        if let Err(e) = status {
                            eprintln!("Error committing default.xml for {}: {}", repo, e);
                            exit(1);
                        }
                    }
                } else {
                    let status = Command::new("git")
                        .arg("rebase")
                        .arg("--onto")
                        .arg(&graphene_tag)
                        .arg(&graphene_tag_old)
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error rebasing {}: {}", repo, e);
                        exit(1);
                    }
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("-f")
                    .status();

                if let Err(e) = status {
                    eprintln!("Error pushing changes for {}: {}", repo, e);
                    exit(1);
                }
            },
            "default" => {
                let status = Command::new("gh")
                    .arg("repo")
                    .arg("edit")
                    .arg(format!("dot166/{}", repo))
                    .arg("--default-branch")
                    .arg(&branch)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error editing default branch for {}: {}", repo, e);
                    exit(1);
                }
            },
            _ => {}
        }

        if action == "init" {
            let remote_url = if repo == "jOS_manifest" {
                "https://github.com/grapheneos/platform_manifest"
            } else {
                &format!("https://github.com/grapheneos/{}", repo)
            };

            let status = Command::new("git")
                .arg("remote")
                .arg("add")
                .arg("upstream")
                .arg(remote_url)
                .status();

            if let Err(e) = status {
                eprintln!("Error adding upstream for {}: {}", repo, e);
                exit(1);
            }

            let status = Command::new("git")
                .arg("fetch")
                .arg("upstream")
                .arg("--tags")
                .status();

            if let Err(e) = status {
                eprintln!("Error fetching upstream tags for {}: {}", repo, e);
                exit(1);
            }
        }
    }

    for repo in aosp_forks {
        println!("\n>>> Handling {}", repo);

        match action.as_str() {
            "init" => {
                let status = Command::new("git")
                    .arg("clone")
                    .arg(format!("https://github.com/dot166/{}", repo))
                    .status();

                if let Err(e) = status {
                    eprintln!("Error cloning {}: {}", repo, e);
                    exit(1);
                }
            },
            _ => {}
        }

        if let Err(e) = env::set_current_dir(&repo) {
            eprintln!("Failed to change directory to {}: {}", repo, e);
            exit(1);
        }

        match action.as_str() {
            "bupdate" => {
                let status = Command::new("git")
                    .arg("checkout")
                    .arg("origin")
                    .status();

                if let Err(e) = status {
                    eprintln!("Error checking out origin for {}: {}", repo, e);
                    exit(1);
                }

                let status = Command::new("git")
                    .arg("switch")
                    .arg("-c")
                    .arg(&branch)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error switching to branch {}: {}", &branch, e);
                    exit(1);
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("--set-upstream")
                    .arg("origin")
                    .arg(&branch)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error pushing {} to upstream: {}", &branch, e);
                    exit(1);
                }
            },
            _ => {
                let status = Command::new("git")
                    .arg("checkout")
                    .arg(&branch)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error checking out branch {}: {}", &branch, e);
                    exit(1);
                }
            }
        }

        let status = Command::new("git")
            .arg("pull")
            .status();

        if let Err(e) = status {
            eprintln!("Error pulling changes for {}: {}", repo, e);
            exit(1);
        }

        match action.as_str() {
            "delete" => {
                let _ = Command::new("git")
                    .arg("tag")
                    .arg("-d")
                    .arg(tag_name)
                    .status();

                let _ = Command::new("git")
                    .arg("push")
                    .arg("origin")
                    .arg("--delete")
                    .arg(tag_name)
                    .status();
            },
            "release" => {
                let status = Command::new("git")
                    .arg("tag")
                    .arg("-s")
                    .arg(tag_name)
                    .arg("-m")
                    .arg(tag_name)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error creating release tag {}: {}", tag_name, e);
                    exit(1);
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("origin")
                    .arg(tag_name)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error pushing release tag {}: {}", tag_name, e);
                    exit(1);
                }
            },
            "update" => {
                let status = Command::new("git")
                    .arg("fetch")
                    .arg("upstream")
                    .arg("--tags")
                    .arg("--force")
                    .status();

                if let Err(e) = status {
                    eprintln!("Error fetching upstream tags: {}", e);
                    exit(1);
                }

                let status = Command::new("git")
                    .arg("rebase")
                    .arg("--onto")
                    .arg(&aosp_tag)
                    .arg(&aosp_tag_old)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error rebasing: {}", e);
                    exit(1);
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("-f")
                    .status();

                if let Err(e) = status {
                    eprintln!("Error pushing changes: {}", e);
                    exit(1);
                }
            },
            "default" => {
                let status = Command::new("gh")
                    .arg("repo")
                    .arg("edit")
                    .arg(format!("dot166/{}", repo))
                    .arg("--default-branch")
                    .arg(&branch)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error editing default branch for {}: {}", repo, e);
                    exit(1);
                }
            },
            _ => {}
        }

        if action == "init" {
            let status = Command::new("git")
                .arg("remote")
                .arg("add")
                .arg("upstream")
                .arg(format!("https://android.googlesource.com/{}", repo.replace('_', "/")))
                .status();

            if let Err(e) = status {
                eprintln!("Error adding upstream for {}: {}", repo, e);
                exit(1);
            }

            let status = Command::new("git")
                .arg("fetch")
                .arg("upstream")
                .arg("--tags")
                .status();

            if let Err(e) = status {
                eprintln!("Error fetching upstream tags for {}: {}", repo, e);
                exit(1);
            }
        }

        if let Err(e) = env::set_current_dir("..") {
            eprintln!("Failed to change back to parent directory: {}", e);
            exit(1);
        }
    }

    for repo in lineageos_forks {
        println!("\n>>> Handling {}", repo);

        if action == "init" {
            let status = Command::new("git")
                .arg("clone")
                .arg(format!("https://github.com/dot166/{}", repo))
                .status();

            if let Err(e) = status {
                eprintln!("Error cloning {}: {}", repo, e);
                exit(1);
            }
        }

        if let Err(e) = env::set_current_dir(&repo) {
            eprintln!("Failed to change directory to {}: {}", repo, e);
            exit(1);
        }

        if action != "bupdate" {
            let status = Command::new("git")
                .arg("checkout")
                .arg(&branch)
                .status();

            if let Err(e) = status {
                eprintln!("Error checking out branch {}: {}", &branch, e);
                exit(1);
            }
        } else {
            let status = Command::new("git")
                .arg("checkout")
                .arg("origin")
                .status();

            if let Err(e) = status {
                eprintln!("Error checking out origin for {}: {}", repo, e);
                exit(1);
            }

            let status = Command::new("git")
                .arg("switch")
                .arg("-c")
                .arg(&branch)
                .status();

            if let Err(e) = status {
                eprintln!("Error switching to branch {}: {}", &branch, e);
                exit(1);
            }

            let status = Command::new("git")
                .arg("push")
                .arg("--set-upstream")
                .arg("origin")
                .arg(&branch)
                .status();

            if let Err(e) = status {
                eprintln!("Error pushing {} to upstream: {}", &branch, e);
                exit(1);
            }
        }

        let status = Command::new("git")
            .arg("pull")
            .status();

        if let Err(e) = status {
            eprintln!("Error pulling changes for {}: {}", repo, e);
            exit(1);
        }

        match action.as_str() {
            "delete" => {
                let _ = Command::new("git")
                    .arg("tag")
                    .arg("-d")
                    .arg(tag_name)
                    .status();

                let _ = Command::new("git")
                    .arg("push")
                    .arg("origin")
                    .arg("--delete")
                    .arg(tag_name)
                    .status();
            },
            "release" => {
                let status = Command::new("git")
                    .arg("tag")
                    .arg("-s")
                    .arg(tag_name)
                    .arg("-m")
                    .arg(tag_name)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error creating release tag {}: {}", tag_name, e);
                    exit(1);
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("origin")
                    .arg(tag_name)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error pushing release tag {}: {}", tag_name, e);
                    exit(1);
                }
            },
            "update" => {
                let status = Command::new("git")
                    .arg("fetch")
                    .arg("upstream")
                    .status();

                if let Err(e) = status {
                    eprintln!("Error fetching upstream: {}", e);
                    exit(1);
                }

                let rebase_status = Command::new("git")
                    .arg("rebase")
                    .arg(format!("--onto upstream/{}", lineage_latest_branch))
                    .arg(fs::read_to_string("upstream-cm-commit").expect("Failed to read lineage commit"))
                    .status();

                if let Err(e) = rebase_status {
                    eprintln!("Error during rebase for {}: {}", repo, e);
                    exit(1);
                }

                fs::remove_file("upstream-cm-commit").expect("Failed to remove upstream-lineage-commit file");

                let rev_parse_status = Command::new("git")
                    .arg("rev-parse")
                    .arg(format!("--verify upstream/{}", lineage_latest_branch))
                    .output();

                if let Ok(output) = rev_parse_status {
                    let lineage_commit = String::from_utf8_lossy(&output.stdout);
                    fs::write("upstream-cm-commit", &*lineage_commit).expect("Failed to write lineage commit to file");
                } else {
                    eprintln!("Error getting commit hash for {}: {}", repo, rev_parse_status.unwrap_err());
                    exit(1);
                }

                let status = Command::new("git")
                    .arg("diff")
                    .arg("--quiet")
                    .status();

                let changes = if let Ok(status) = status {
                    if status.success() {
                        0
                    } else {
                        1
                    }
                } else {
                    1
                };

                println!("CHANGES={}", changes);

                if changes == 1 {
                    let status = Command::new("git")
                        .arg("add")
                        .arg(".")
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error staging changes: {}", e);
                        exit(1);
                    }

                    let status = Command::new("git")
                        .arg("commit")
                        .arg("-m")
                        .arg("update to a newer lineage commit")
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error committing changes: {}", e);
                        exit(1);
                    }
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("-f")
                    .status();

                if let Err(e) = status {
                    eprintln!("Error pushing changes: {}", e);
                    exit(1);
                }
            },
            "default" => {
                let status = Command::new("gh")
                    .arg("repo")
                    .arg("edit")
                    .arg(format!("dot166/{}", repo))
                    .arg("--default-branch")
                    .arg(&branch)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error editing default branch for {}: {}", repo, e);
                    exit(1);
                }
            },
            _ => {}
        }

        if action == "init" {
            let status = Command::new("git")
                .arg("remote")
                .arg("add")
                .arg("upstream")
                .arg(format!("https://github.com/LineageOS/{}", repo.replace("platform", "android")))
                .status();

            if let Err(e) = status {
                eprintln!("Error adding upstream for {}: {}", repo, e);
                exit(1);
            }

            let status = Command::new("git")
                .arg("fetch")
                .arg("upstream")
                .arg("--tags")
                .status();

            if let Err(e) = status {
                eprintln!("Error fetching upstream tags for {}: {}", repo, e);
                exit(1);
            }
        }

        if let Err(e) = env::set_current_dir("..") {
            eprintln!("Failed to change back to parent directory: {}", e);
            exit(1);
        }
    }

    for repo in independent {
        println!("\n>>> Handling {}", repo);

        if action == "init" {
            let status = Command::new("git")
                .arg("clone")
                .arg(format!("https://github.com/dot166/{}", repo))
                .status();

            if let Err(e) = status {
                eprintln!("Error cloning {}: {}", repo, e);
                exit(1);
            }
        }

        if let Err(e) = env::set_current_dir(&repo) {
            eprintln!("Failed to change directory to {}: {}", repo, e);
            exit(1);
        }

        if repo != "jOS-Updates" && repo != "jOS_j-lib" {
            if action != "bupdate" {
                let status = Command::new("git")
                    .arg("checkout")
                    .arg(&branch)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error checking out branch {}: {}", branch, e);
                    exit(1);
                }
            } else {
                let status = Command::new("git")
                    .arg("checkout")
                    .arg("origin")
                    .status();

                if let Err(e) = status {
                    eprintln!("Error checking out origin for {}: {}", repo, e);
                    exit(1);
                }

                let status = Command::new("git")
                    .arg("switch")
                    .arg("-c")
                    .arg(&branch)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error switching to branch {}: {}", &branch, e);
                    exit(1);
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("--set-upstream")
                    .arg("origin")
                    .arg(&branch)
                    .status();

                if let Err(e) = status {
                    eprintln!("Error pushing {} to upstream: {}", branch, e);
                    exit(1);
                }
            }
        } else {
            let status = Command::new("git")
                .arg("checkout")
                .arg("main")
                .status();

            if let Err(e) = status {
                eprintln!("Error checking out main for {}: {}", repo, e);
                exit(1);
            }
        }

        let status = Command::new("git")
            .arg("pull")
            .status();

        if let Err(e) = status {
            eprintln!("Error pulling changes for {}: {}", repo, e);
            exit(1);
        }

        match action.as_str() {
            "delete" => {
                if repo != "jOS_j-lib" {
                    let _ = Command::new("git")
                        .arg("tag")
                        .arg("-d")
                        .arg(tag_name)
                        .status();

                    let _ = Command::new("git")
                        .arg("push")
                        .arg("origin")
                        .arg("--delete")
                        .arg(tag_name)
                        .status();
                }
            },
            "release" => {
                if repo == "jOS-Updates" {
                    let release_dir = format!("../../jOS/releases/{}/release-oriole-{}/", tag_name, tag_name);
                    let status = Command::new("cp")
                        .arg("-T")
                        .arg(format!("{}oriole-stable", release_dir))
                        .arg("oriole-stable")
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error copying stable release for {}: {}", repo, e);
                        exit(1);
                    }

                    let status = Command::new("cp")
                        .arg("-T")
                        .arg(format!("{}oriole-beta", release_dir))
                        .arg("oriole-beta")
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error copying beta release for {}: {}", repo, e);
                        exit(1);
                    }

                    let status = Command::new("cp")
                        .arg("-T")
                        .arg(format!("{}oriole-alpha", release_dir))
                        .arg("oriole-alpha")
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error copying alpha release for {}: {}", repo, e);
                        exit(1);
                    }

                    let status = Command::new("git")
                        .arg("add")
                        .arg(".")
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error adding files for {}: {}", repo, e);
                        exit(1);
                    }

                    let status = Command::new("git")
                        .arg("commit")
                        .arg("-m")
                        .arg("add new version information")
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error committing files for {}: {}", repo, e);
                        exit(1);
                    }

                    let status = Command::new("git")
                        .arg("push")
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error pushing changes for {}: {}", repo, e);
                        exit(1);
                    }

                    let status = Command::new("gh")
                        .arg("release")
                        .arg("create")
                        .arg(tag_name)
                        .arg("--latest=true")
                        .arg("--notes")
                        .arg("i keep forgetting to write changelogs")
                        .arg(format!("{}oriole-ota_update-{}.zip", release_dir, tag_name))
                        .arg(format!("{}oriole-install-{}.zip", release_dir, tag_name))
                        .status();

                    if let Err(e) = status {
                        eprintln!("Error creating release for {}: {}", repo, e);
                        exit(1);
                    }
                } else {
                    if repo != "jOS_j-lib" {
                        let status = Command::new("git")
                            .arg("tag")
                            .arg("-s")
                            .arg(tag_name)
                            .arg("-m")
                            .arg(tag_name)
                            .status();

                        if let Err(e) = status {
                            eprintln!("Error tagging {}: {}", repo, e);
                            exit(1);
                        }

                        let status = Command::new("git")
                            .arg("push")
                            .arg("origin")
                            .arg(tag_name)
                            .status();

                        if let Err(e) = status {
                            eprintln!("Error pushing tag {}: {}", repo, e);
                            exit(1);
                        }
                    }
                }
            },
            _ => {}
        }

        if let Err(e) = env::set_current_dir("..") {
            eprintln!("Failed to change back to parent directory: {}", e);
            exit(1);
        }
    }

    if action == "bupdate" {
        let status = Command::new("script/jOS/manage")
            .arg("update")
            .status();

        if let Err(e) = status {
            eprintln!("Error running update: {}", e);
            exit(1);
        }

        let status = Command::new("script/jOS/manage")
            .arg("default")
            .status();

        if let Err(e) = status {
            eprintln!("Error running default: {}", e);
            exit(1);
        }
    }
}
