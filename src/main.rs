#[allow(unused_imports)]
use std::io::{self, Write};

use completer::MyHelper;
use parser::parse_pipeline;
use rustyline::{CompletionType, Config, Editor};

mod ast;
mod builtins;
mod completer;
mod parser;

fn main() {
    let shell = builtins::Shell::new();
    let completer = shell.initialize_completer();

    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();
    let h = MyHelper { completer };
    let mut rl = Editor::with_config(config).unwrap();
    rl.set_helper(Some(h));

    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(input) => {
                if let Ok(pipeline) = parse_pipeline(&input) {
                    // Successfully parsed the pipeline
                    for segment in pipeline.segments {
                        shell.run_command(segment);
                    }
                } else {
                    eprintln!("Error: Failed to parse pipeline");
                }
            }
            Err(rustyline::error::ReadlineError::Eof)
            | Err(rustyline::error::ReadlineError::Interrupted) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}
