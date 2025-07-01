use std::env;
use std::fs;
use lib_aosp::build::repo_sync;
use lib_aosp::utils::*;

fn main() -> std::io::Result<()> {
    require_top();
    env::set_current_dir(get_top().unwrap()).unwrap();

    let out_dir = env::current_dir().unwrap().join("out");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).unwrap();
    }

    repo_sync()
}
