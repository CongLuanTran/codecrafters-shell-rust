#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

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
        let args: Vec<&str> = input.split_whitespace().collect();
        if args[0] == "exit" {
            process::exit(args[1].parse::<i32>().unwrap());
        } else {
            println!("{}: command not found", input.trim());
        }
    }
}
