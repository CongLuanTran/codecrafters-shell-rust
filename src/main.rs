#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, process};

fn echo(input: &str) {
    println!("{}", input.trim());
}

fn exit(code: i32) {
    process::exit(code);
}

fn type_of(cmd: &str) {
    let builtin = ["echo", "type", "exit"];
    if builtin.contains(&cmd) {
        println!("{} is a shell builtin", cmd);
    } else {
        let path_var = env::var("PATH").unwrap_or_default();
        let path_dirs = env::split_paths(&path_var);
        for dir in path_dirs {
            let full_path = dir.join(cmd);
            if full_path.is_file() {
                println!("{} is {}", cmd, full_path.display());
                return;
            }
        }
        println!("{}: not found", cmd);
    }
}

fn main() {
    // Declare a variable to hold user input
    let mut input = String::new();

    loop {
        // Print the prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // Empty the input
        input.clear();

        // Wait for user input
        io::stdin().read_line(&mut input).unwrap();
        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts.as_slice() {
            [] => {}
            [cmd] => println!("{}: command not found", cmd),
            [cmd, args @ ..] => match *cmd {
                "echo" => echo(&args.join(" ")),
                "exit" => exit(args[0].parse::<i32>().unwrap()),
                "type" => type_of(args[0]),
                _ => println!("{}: command not found", cmd),
            },
        };
    }
}
