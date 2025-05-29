#[allow(unused_imports)]
use std::io::{self, Write};

mod ast;
mod builtins;
mod parser;

fn main() {
    // Uncomment this block to pass the first stage
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}
