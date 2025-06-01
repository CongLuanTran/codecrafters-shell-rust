#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::{Command, Stdio};

use ast::apply_redirection;
use builtins::Shell;
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
        .auto_add_history(true)
        .build();
    let h = MyHelper { completer };
    let mut rl = Editor::with_config(config).unwrap();
    rl.set_helper(Some(h));

    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(input) => {
                if let Ok(pipeline) = parse_pipeline(&input) {
                    let mut processes = Vec::new();
                    let mut builtin_threads = Vec::new();
                    let mut prev_reader = None;
                    let mut segments = pipeline.segments.into_iter().peekable();

                    while let Some(segment) = segments.next() {
                        let is_last = segments.peek().is_none();

                        // Create a pipe if not the last segment
                        let (reader, writer) = if !is_last {
                            let (r, w) = os_pipe::pipe().unwrap();
                            (Some(r), Some(w))
                        } else {
                            (None, None)
                        };

                        if shell.is_builtin(segment.cmd.as_str()) {
                            // Builtin: run in a thread, write output to pipe or stdout
                            let mut output = writer;
                            let args = segment.args.clone();
                            let (outfile, errfile) =
                                Shell::builtin_redirection(&segment.redirections);
                            let shell_clone = shell.clone();
                            let history: Vec<String> =
                                rl.history().iter().map(|s| s.to_string()).collect();

                            let handle = std::thread::spawn(move || {
                                let out: Box<dyn Write> = if let Some(file) = outfile {
                                    Box::new(file)
                                } else if let Some(w) = output.take() {
                                    Box::new(w)
                                } else {
                                    Box::new(std::io::stdout())
                                };
                                let mut error: Box<dyn Write> = if let Some(file) = errfile {
                                    Box::new(file)
                                } else {
                                    Box::new(std::io::stderr())
                                };
                                match segment.cmd.as_str() {
                                    "exit" => Shell::exit(args),
                                    "echo" => Shell::echo(args, out),
                                    "pwd" => Shell::pwd(out),
                                    "cd" => Shell::cd(args, error),
                                    "type" => shell_clone.type_of(args, out, error),
                                    "history" => Shell::history(args, &history, out),
                                    _ => writeln!(error, "{}: command not found", segment.cmd)
                                        .unwrap(),
                                }
                            });
                            builtin_threads.push(handle);
                            prev_reader = reader;
                        } else {
                            // External command
                            let mut cmd = Command::new(&segment.cmd);
                            cmd.args(&segment.args);

                            if let Some(prev) = prev_reader.take() {
                                cmd.stdin(Stdio::from(prev));
                            }
                            if let Some(w) = writer {
                                cmd.stdout(Stdio::from(w));
                            }
                            if apply_redirection(&mut cmd, &segment.redirections).is_err() {
                                eprintln!("Redirection error in command: {}", segment.cmd);
                                return;
                            }

                            match cmd.spawn() {
                                Ok(child) => processes.push(child),
                                Err(_) => {
                                    eprintln!("{}: command not found", segment.cmd);
                                }
                            }
                            prev_reader = reader;
                        }
                    }

                    for mut child in processes {
                        let _ = child.wait();
                    }
                    for handle in builtin_threads {
                        let _ = handle.join();
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
