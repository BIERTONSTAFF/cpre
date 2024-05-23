mod parser;

use parser::Parser;
use std::{
    env,
    fs::File,
    io::{Read, Write},
    process::Command,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        println!(
            "PreC Preprocessor {}\n\nUSAGE: {} [OPTIONS] FILE\n\nOPTIONS:\n-h, --help      : print help\n-o, --output    : output file",
            env!("CARGO_PKG_VERSION"),
            args[0]
        );
        return;
    }

    if !args[3].ends_with(".preC") {
        eprintln!("FILE must be PreC source file");
        return;
    }

    let mut buffer = String::new();

    match File::open(&args[3]).and_then(|mut file| file.read_to_string(&mut buffer)) {
        Ok(_) => {
            let mut parser = Parser::new(buffer);

            parser.parse();

            match File::create("pre.c").and_then(|mut file| file.write(parser.src.as_bytes())) {
                Ok(_) => {
                    let compile = Command::new("clang")
                        .args(["-o", args[2].as_str(), "pre.c"])
                        .output()
                        .expect("Failed to run clang");

                    if compile.status.success() {
                        let _ = std::fs::remove_file("pre.c");
                    }
                }
                Err(_) => eprintln!("Failed to create output file"),
            }
        }
        Err(_) => eprintln!("Unable to read source file"),
    }
}
