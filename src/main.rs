#[allow(unused_imports)]
use std::io::{self, Write};

use parser::parse_pipeline;

mod ast;
mod builtins;
mod parser;

fn main() {
    let shell = builtins::Shell::new();

    loop {
        // Uncomment this block to pass the first stage
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if let Ok(pipeline) = parse_pipeline(&input) {
            // Successfully parsed the pipeline
            for segment in pipeline.segments {
                shell.run_command(&segment.cmd, segment.args);
            }
        } else {
            eprintln!("Error: Failed to parse pipeline");
        }
    }
}
