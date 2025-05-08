#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

fn main() {
    // Declare a variable to hold user input
    let mut input = String::new();
    let builtin = ["exit", "echo", "type"];

    loop {
        // Print the prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // Empty the input
        input.clear();

        // Wait for user input
        io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.split_whitespace().collect();
        match args[0] {
            "exit" => process::exit(args[1].parse::<i32>().unwrap()),
            "echo" => println!("{}", input.trim().strip_prefix("echo ").unwrap()),
            "type" => {
                if builtin.contains(&args[1]) {
                    println!("{} is a shell builtin", args[1]);
                } else {
                    println!("{}: not found", args[1]);
                }
            }
            _ => println!("{}: command not found", input.trim()),
        }
    }
}
