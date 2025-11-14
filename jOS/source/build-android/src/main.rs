use std::env;
use lib_aosp::build;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: aosp-build [device] [build type]");
        panic!("Expected two command-line arguments");
    }
    let build_type = build::get_build_type((&args[2]).parse().unwrap());
    if args[1] == "emulator" {
        args[1] = "sdk_phone64_x86_64".parse().unwrap();
    }
    let device = build::get_device((&args[1]).parse().unwrap());

    build::build_aosp(device, build_type);
}