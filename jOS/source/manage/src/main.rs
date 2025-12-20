use lib_aosp::scripts;
use std::process::Command;
use std::{env, fs};

fn main() {
    let (_, _, mut branch) = scripts::read_common_sh();
    let (graphene_tag, graphene_tag_old, lineage_latest_branch) = scripts::read_config_file();
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 { panic!("expected action as argument");}
    let action= args[1].clone();
    let mut tag_name = "";

    if action == "update" || action == "default" || action == "init" || action == "bupdate" || action == "null" {
        if env::var("IS_CI").unwrap_or("false".parse().unwrap()) == "true" && action != "init" {
            panic!("cannot use {} in ci, this is done to prevent the ci from destroying the source tree", action);
        }
        if args.len() != 2 {panic!("expected no arguments for $action");}
    } else if action == "release" || action == "delete" {
        if env::var("IS_CI").unwrap_or("false".parse().unwrap()) == "true" {
            panic!("cannot use {} in ci, this is done to prevent the ci from destroying the source tree", action);
        }
        tag_name = &args[2];
        if args.len() != 3 {panic!("expected tag name as argument for $action");}
    } else {
        panic!("unrecognized action");
    }

    let grapheneos_forks=[
        "platform_build",
        "platform_build_release",
        "platform_frameworks_base",
        "platform_frameworks_libs_systemui",
        "platform_manifest",
        "platform_packages_apps_DeskClock",
        "platform_packages_apps_Dialer",
        "platform_packages_apps_ExactCalculator",
        "platform_packages_apps_Launcher3",
        "platform_packages_apps_Settings",
        "platform_packages_apps_SetupWizard2",
        "platform_packages_apps_Updater",
        "platform_packages_inputmethods_LatinIME",
        "platform_packages_services_telecomm",
    ];

    let lineageos_forks=[
        "platform_packages_apps_Recorder",
        "platform_packages_apps_Etar",
    ];

    let independent=[
        "jOS-Updates",
    ];

    println!("\n>>> Handling script");

        if action == "init" {
            let status = Command::new("git")
                .arg("clone")
                .arg("https://github.com/dot166/script")
                .status();

            if status.is_err() {
                panic!("Error cloning script: {}", status.unwrap_err());
            }
        }

        let status = env::set_current_dir("script");
        if status.is_err() {
            panic!("Failed to change directory to script: {}", status.unwrap_err());
        }

        if action != "bupdate" {
            let status = Command::new("git")
                .arg("checkout")
                .arg(&branch)
                .status();

            if status.is_err() {
                panic!("Error checking out branch {}: {}", &branch, status.unwrap_err());
            }
        }

        let status = Command::new("git")
            .arg("pull")
            .status();

        if status.is_err() {
            panic!("Error pulling changes for script: {}", status.unwrap_err());
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

                if status.is_err() {
                    panic!("Error creating release tag {}: {}", tag_name, status.unwrap_err());
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("origin")
                    .arg(tag_name)
                    .status();

                if status.is_err() {
                    panic!("Error pushing release tag {}: {}", tag_name, status.unwrap_err());
                }
            },
            "update" => {
                let status = Command::new("git")
                    .arg("fetch")
                    .arg("upstream")
                    .arg("--tags")
                    .arg("--force")
                    .status();

                if status.is_err() {
                    panic!("Error fetching upstream tags: {}", status.unwrap_err());
                }

                let status = Command::new("git")
                    .arg("rebase")
                    .arg("--onto")
                    .arg(&graphene_tag)
                    .arg(&graphene_tag_old)
                    .status();

                if status.is_err() {
                    panic!("Error rebasing script: {}", status.unwrap_err());
                }

                (_, _, branch) = scripts::read_common_sh();

                let status = Command::new("git")
                    .arg("push")
                    .arg("-f")
                    .status();

                if status.is_err() {
                    panic!("Error pushing changes for script: {}", status.unwrap_err());
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

                if status.is_err() {
                    panic!("Error editing default branch for script: {}", status.unwrap_err());
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

            if status.is_err() {
                panic!("Error adding upstream for script: {}", status.unwrap_err());
            }

            let status = Command::new("git")
                .arg("fetch")
                .arg("upstream")
                .arg("--tags")
                .status();

            if status.is_err() {
                panic!("Error fetching upstream tags for script: {}", status.unwrap_err());
            }
        }

        let status = env::set_current_dir("..");
        if status.is_err() {
            panic!("Failed to change back to parent directory: {}", status.unwrap_err());
        }

    for repo in grapheneos_forks {
        println!("\n>>> Handling {}", repo);

        if action == "init" {
            let status = Command::new("git")
                .arg("clone")
                .arg(format!("https://github.com/dot166/{}", repo))
                .status();

            if status.is_err() {
                panic!("Error cloning {}: {}", repo, status.unwrap_err());
            }
        }

        let status = env::set_current_dir(&repo);
        if status.is_err() {
            panic!("Failed to change directory to {}: {}", repo, status.unwrap_err());
        }

        if action != "bupdate" {
            let status = Command::new("git")
                .arg("checkout")
                .arg(&branch)
                .status();

            if status.is_err() {
                panic!("Error checking out branch {}: {}", &branch, status.unwrap_err());
            }
        } else {
            let status = Command::new("git")
                .arg("checkout")
                .arg("origin")
                .status();

            if status.is_err() {
                panic!("Error checking out origin for {}: {}", repo, status.unwrap_err());
            }

            let status = Command::new("git")
                .arg("pull")
                .status();

            if status.is_err() {
                panic!("Error pulling changes for {}: {}", repo, status.unwrap_err());
            }

            let status = Command::new("git")
                .arg("switch")
                .arg("-c")
                .arg(&branch)
                .status();

            if status.is_err() {
                panic!("Error switching to branch {}: {}", &branch, status.unwrap_err());
            }

            let status = Command::new("git")
                .arg("push")
                .arg("--set-upstream")
                .arg("origin")
                .arg(&branch)
                .status();

            if status.is_err() {
                panic!("Error pushing {} to upstream: {}", &branch, status.unwrap_err());
            }
        }

        let status = Command::new("git")
            .arg("pull")
            .status();

        if status.is_err() {
            panic!("Error pulling changes for {}: {}", repo, status.unwrap_err());
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

                    if status.is_err() {
                        panic!("Error checking out tmp branch for {}: {}", repo, status.unwrap_err());
                    }

                    let data = fs::read_to_string(env::current_dir().unwrap().join("default.xml")).unwrap();
                    let new = data.replace(&branch, tag_name);
                    let status = fs::write(env::current_dir().unwrap().join("default.xml"), &new);

                    if status.is_err() {
                        panic!("Error updating default.xml for {}: {}", repo, status.unwrap_err());
                    }

                    let status = Command::new("git")
                        .arg("commit")
                        .arg("default.xml")
                        .arg("-m")
                        .arg(tag_name)
                        .status();

                    if status.is_err() {
                        panic!("Error committing default.xml for {}: {}", repo, status.unwrap_err());
                    }

                    let status = Command::new("git")
                        .arg("push")
                        .arg("-fu")
                        .arg("origin")
                        .arg("tmp")
                        .status();

                    if status.is_err() {
                        panic!("Error pushing tmp branch for {}: {}", repo, status.unwrap_err());
                    }
                }

                let status = Command::new("git")
                    .arg("tag")
                    .arg("-s")
                    .arg(tag_name)
                    .arg("-m")
                    .arg(tag_name)
                    .status();

                if status.is_err() {
                    panic!("Error creating release tag {}: {}", tag_name, status.unwrap_err());
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("origin")
                    .arg(tag_name)
                    .status();

                if status.is_err() {
                    panic!("Error pushing release tag {}: {}", tag_name, status.unwrap_err());
                }
            },
            "update" => {
                let status = Command::new("git")
                    .arg("fetch")
                    .arg("upstream")
                    .arg("--tags")
                    .arg("--force")
                    .status();

                if status.is_err() {
                    panic!("Error fetching upstream tags: {}", status.unwrap_err());
                }

                if repo == "platform_manifest" {
                    let status = Command::new("git")
                        .arg("rebase")
                        .arg("--onto")
                        .arg(format!("{}~1", &graphene_tag))
                        .arg(format!("{}~1", &graphene_tag_old))
                        .status();

                    if status.is_err() {
                        panic!("Error rebasing {}: {}", repo, status.unwrap_err());
                    }

                    let data = fs::read_to_string(env::current_dir().unwrap().join("default.xml")).unwrap();
                    let new = data.replace(&graphene_tag_old, &graphene_tag);
                    let status = fs::write(env::current_dir().unwrap().join("default.xml"), &new);

                    if status.is_err() {
                        panic!("Error updating default.xml for {}: {}", "manifest", status.unwrap_err());
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

                        if status.is_err() {
                            panic!("Error committing default.xml for {}: {}", repo, status.unwrap_err());
                        }
                    }
                } else {
                    let status = Command::new("git")
                        .arg("rebase")
                        .arg("--onto")
                        .arg(&graphene_tag)
                        .arg(&graphene_tag_old)
                        .status();

                    if status.is_err() {
                        panic!("Error rebasing {}: {}", repo, status.unwrap_err());
                    }
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("-f")
                    .status();

                if status.is_err() {
                    panic!("Error pushing changes for {}: {}", repo, status.unwrap_err());
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

                if status.is_err() {
                    panic!("Error editing default branch for {}: {}", repo, status.unwrap_err());
                }
            },
            _ => {}
        }

        if action == "init" {
            let remote_url = &format!("https://github.com/grapheneos/{}", repo);

            let status = Command::new("git")
                .arg("remote")
                .arg("add")
                .arg("upstream")
                .arg(remote_url)
                .status();

            if status.is_err() {
                panic!("Error adding upstream for {}: {}", repo, status.unwrap_err());
            }

            let status = Command::new("git")
                .arg("fetch")
                .arg("upstream")
                .arg("--tags")
                .status();

            if status.is_err() {
                panic!("Error fetching upstream tags for {}: {}", repo, status.unwrap_err());
            }
        }

        let status = env::set_current_dir("..");
        if status.is_err() {
            panic!("Failed to change back to parent directory: {}", status.unwrap_err());
        }
    }

    for repo in lineageos_forks {
        println!("\n>>> Handling {}", repo);

        if action == "init" {
            let status = Command::new("git")
                .arg("clone")
                .arg(format!("https://github.com/dot166/{}", repo))
                .status();

            if status.is_err() {
                panic!("Error cloning {}: {}", repo, status.unwrap_err());
            }
        }

        let status = env::set_current_dir(&repo);
        if status.is_err() {
            panic!("Failed to change directory to {}: {}", repo, status.unwrap_err());
        }

        if action != "bupdate" {
            let status = Command::new("git")
                .arg("checkout")
                .arg(&branch)
                .status();

            if status.is_err() {
                panic!("Error checking out branch {}: {}", &branch, status.unwrap_err());
            }
        } else {
            let status = Command::new("git")
                .arg("checkout")
                .arg("origin")
                .status();

            if status.is_err() {
                panic!("Error checking out origin for {}: {}", repo, status.unwrap_err());
            }

            let status = Command::new("git")
                .arg("pull")
                .status();

            if status.is_err() {
                panic!("Error pulling changes for {}: {}", repo, status.unwrap_err());
            }

            let status = Command::new("git")
                .arg("switch")
                .arg("-c")
                .arg(&branch)
                .status();

            if status.is_err() {
                panic!("Error switching to branch {}: {}", &branch, status.unwrap_err());
            }

            let status = Command::new("git")
                .arg("push")
                .arg("--set-upstream")
                .arg("origin")
                .arg(&branch)
                .status();

            if status.is_err() {
                panic!("Error pushing {} to upstream: {}", &branch, status.unwrap_err());
            }
        }

        let status = Command::new("git")
            .arg("pull")
            .status();

        if status.is_err() {
            panic!("Error pulling changes for {}: {}", repo, status.unwrap_err());
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

                if status.is_err() {
                    panic!("Error creating release tag {}: {}", tag_name, status.unwrap_err());
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("origin")
                    .arg(tag_name)
                    .status();

                if status.is_err() {
                    panic!("Error pushing release tag {}: {}", tag_name, status.unwrap_err());
                }
            },
            "update" => {
                let status = Command::new("git")
                    .arg("fetch")
                    .arg("upstream")
                    .status();

                if status.is_err() {
                    panic!("Error fetching upstream: {}", status.unwrap_err());
                }

                let rebase_status = Command::new("git")
                    .arg("rebase")
                    .arg(format!("upstream/{}", lineage_latest_branch))
                    .status();

                if rebase_status.is_err() {
                    panic!("Error during rebase for {}: {}", repo, rebase_status.unwrap_err());
                }

                //let rebase_status = Command::new("git")
                //    .arg("rebase")
                //    .arg("--onto")
                //    .arg(format!("upstream/{}", lineage_latest_branch))
                //    .arg(fs::read_to_string("upstream-cm-commit").expect("Failed to read lineage commit").replace("\n", ""))
                //    .status();

                //if rebase_status.is_err() {
                //    panic!("Error during rebase for {}: {}", repo, rebase_status.unwrap_err());
                //}

                //fs::remove_file("upstream-cm-commit").expect("Failed to remove upstream-lineage-commit file");

                //let rev_parse_status = Command::new("git")
                //    .arg("rev-parse")
                //    .arg("--verify")
                //    .arg(format!("upstream/{}", lineage_latest_branch))
                //    .output();

                //if rev_parse_status.is_ok() {
                //    fs::write("upstream-cm-commit", rev_parse_status.unwrap().stdout).expect("Failed to write lineage commit to file");
                //} else {
                //    panic!("Error getting commit hash for {}: {}", repo, rev_parse_status.unwrap_err());
                //}

                //let status = Command::new("git")
                //    .arg("diff")
                //    .arg("--quiet")
                //    .status();

                //let changes = if let Ok(status) = status {
                //    if status.success() {
                //        0
                //    } else {
                //        1
                //    }
                //} else {
                //    1
                //};

                //println!("CHANGES={}", changes);

                //if changes == 1 {
                //    let status = Command::new("git")
                //        .arg("add")
                //        .arg(".")
                //        .status();

                //    if status.is_err() {
                //        panic!("Error staging changes: {}", status.unwrap_err());
                //    }

                //    let status = Command::new("git")
                //        .arg("commit")
                //        .arg("-m")
                //        .arg("update to a newer lineage commit")
                //        .status();

                //    if status.is_err() {
                //        panic!("Error committing changes: {}", status.unwrap_err());
                //    }
                //}

                let status = Command::new("git")
                    .arg("push")
                    .arg("-f")
                    .status();

                if status.is_err() {
                    panic!("Error pushing changes: {}", status.unwrap_err());
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

                if status.is_err() {
                    panic!("Error editing default branch for {}: {}", repo, status.unwrap_err());
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

            if status.is_err() {
                panic!("Error adding upstream for {}: {}", repo, status.unwrap_err());
            }

            let status = Command::new("git")
                .arg("fetch")
                .arg("upstream")
                .arg("--tags")
                .status();

            if status.is_err() {
                panic!("Error fetching upstream tags for {}: {}", repo, status.unwrap_err());
            }
        }

        let status = env::set_current_dir("..");
        if status.is_err() {
            panic!("Failed to change back to parent directory: {}", status.unwrap_err());
        }
    }

    for repo in independent {
        println!("\n>>> Handling {}", repo);

        if action == "init" {
            let status = Command::new("git")
                .arg("clone")
                .arg(format!("https://github.com/dot166/{}", repo))
                .status();

            if status.is_err() {
                panic!("Error cloning {}: {}", repo, status.unwrap_err());
            }
        }

        let status = env::set_current_dir(&repo);
        if status.is_err() {
            panic!("Failed to change directory to {}: {}", repo, status.unwrap_err());
        }

        if repo != "jOS-Updates" {
            if action != "bupdate" {
                let status = Command::new("git")
                    .arg("checkout")
                    .arg(&branch)
                    .status();

                if status.is_err() {
                    panic!("Error checking out branch {}: {}", branch, status.unwrap_err());
                }
            } else {
                let status = Command::new("git")
                    .arg("checkout")
                    .arg("origin")
                    .status();

                if status.is_err() {
                    panic!("Error checking out origin for {}: {}", repo, status.unwrap_err());
                }

                let status = Command::new("git")
                    .arg("pull")
                    .status();

                if status.is_err() {
                    panic!("Error pulling changes for {}: {}", repo, status.unwrap_err());
                }

                let status = Command::new("git")
                    .arg("switch")
                    .arg("-c")
                    .arg(&branch)
                    .status();

                if status.is_err() {
                    panic!("Error switching to branch {}: {}", &branch, status.unwrap_err());
                }

                let status = Command::new("git")
                    .arg("push")
                    .arg("--set-upstream")
                    .arg("origin")
                    .arg(&branch)
                    .status();

                if status.is_err() {
                    panic!("Error pushing {} to upstream: {}", branch, status.unwrap_err());
                }
            }
        } else {
            let status = Command::new("git")
                .arg("checkout")
                .arg("main")
                .status();

            if status.is_err() {
                panic!("Error checking out main for {}: {}", repo, status.unwrap_err());
            }
        }

        let status = Command::new("git")
            .arg("pull")
            .status();

        if status.is_err() {
            panic!("Error pulling changes for {}: {}", repo, status.unwrap_err());
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
                if repo == "jOS-Updates" {
                    let release_dir = format!("../../grapheneos/releases/{}/release-oriole-{}/", tag_name, tag_name);
                    let status = Command::new("cp")
                        .arg("-T")
                        .arg(format!("{}oriole-stable", release_dir))
                        .arg("oriole-stable")
                        .status();

                    if status.is_err() {
                        panic!("Error copying stable release for {}: {}", repo, status.unwrap_err());
                    }

                    let status = Command::new("cp")
                        .arg("-T")
                        .arg(format!("{}oriole-beta", release_dir))
                        .arg("oriole-beta")
                        .status();

                    if status.is_err() {
                        panic!("Error copying beta release for {}: {}", repo, status.unwrap_err());
                    }

                    let status = Command::new("cp")
                        .arg("-T")
                        .arg(format!("{}oriole-alpha", release_dir))
                        .arg("oriole-alpha")
                        .status();

                    if status.is_err() {
                        panic!("Error copying alpha release for {}: {}", repo, status.unwrap_err());
                    }

                    let status = Command::new("git")
                        .arg("add")
                        .arg(".")
                        .status();

                    if status.is_err() {
                        panic!("Error adding files for {}: {}", repo, status.unwrap_err());
                    }

                    let status = Command::new("git")
                        .arg("commit")
                        .arg("-m")
                        .arg("add new version information")
                        .status();

                    if status.is_err() {
                        panic!("Error committing files for {}: {}", repo, status.unwrap_err());
                    }

                    let status = Command::new("git")
                        .arg("push")
                        .status();

                    if status.is_err() {
                        panic!("Error pushing changes for {}: {}", repo, status.unwrap_err());
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

                    if status.is_err() {
                        panic!("Error creating release for {}: {}", repo, status.unwrap_err());
                    }
                } else {
                    let status = Command::new("git")
                        .arg("tag")
                        .arg("-s")
                        .arg(tag_name)
                        .arg("-m")
                        .arg(tag_name)
                        .status();

                    if status.is_err() {
                        panic!("Error tagging {}: {}", repo, status.unwrap_err());
                    }

                    let status = Command::new("git")
                        .arg("push")
                        .arg("origin")
                        .arg(tag_name)
                        .status();

                    if status.is_err() {
                        panic!("Error pushing tag {}: {}", repo, status.unwrap_err());
                    }
                }
            },
            "default" => {
                if repo != "jOS-Updates" {
                    let status = Command::new("gh")
                        .arg("repo")
                        .arg("edit")
                        .arg(format!("dot166/{}", repo))
                        .arg("--default-branch")
                        .arg(&branch)
                        .status();

                    if status.is_err() {
                        panic!("Error editing default branch for {}: {}", repo, status.unwrap_err());
                    }
                }
            },
            _ => {}
        }

        let status = env::set_current_dir("..");
        if status.is_err() {
            panic!("Failed to change back to parent directory: {}", status.unwrap_err());
        }
    }

    if action == "bupdate" {
        let status = Command::new("script/jOS/manage")
            .arg("update")
            .status();

        if status.is_err() {
            panic!("Error running update: {}", status.unwrap_err());
        }

        let status = Command::new("script/jOS/manage")
            .arg("default")
            .status();

        if status.is_err() {
            panic!("Error running default: {}", status.unwrap_err());
        }
    }
}
