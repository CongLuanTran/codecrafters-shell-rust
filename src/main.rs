#[allow(unused_imports)]
use std::io::{self, Write};

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
        println!("{}: command not found", input.trim());
    }
}
