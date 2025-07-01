use std::env;
use std::process::{Command, Stdio};
use lib_aosp::build;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        args[2] = "user".parse().unwrap();
    }
    if args.len() != 3 {
        println!("Usage: aosp-build [device] [build type]");
        println!("build type is optional (default is user)");
        panic!("Expected two or less command-line arguments");
    }
    let build_type = build::get_build_type((&args[2]).parse().unwrap());
    if args[1] == "emulator" {
        args[1] = "sdk_phone64_x86_64".parse().unwrap();
    }
    let device = build::get_device((&args[1]).parse().unwrap());

    build::build_aosp(device, build_type);
    
    if args[1] == "sdk_phone64_x86_64" {
        run_emulator()
    }
}

fn run_emulator() {
    Command::new("bash")
        .arg("-c")
        .arg("emulator")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
}