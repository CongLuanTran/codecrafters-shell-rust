#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env::{self},
    path::PathBuf,
    process::{self, Command},
};

use is_executable::IsExecutable;

struct Shell {
    input: String,
    builtin: Vec<&'static str>,
    path_dirs: Vec<PathBuf>,
}

impl Shell {
    fn new() -> Self {
        let input = String::new();
        let builtin = vec!["echo", "type", "exit"];
        let path_var = env::var("PATH").unwrap_or_default();
        let path_dirs = env::split_paths(&path_var).collect();
        Shell {
            input,
            builtin,
            path_dirs,
        }
    }

    fn run(&mut self) {
        loop {
            // Print the prompt
            print!("$ ");
            io::stdout().flush().unwrap();

            // Empty the input
            self.input.clear();
            // Wait for user input
            io::stdin().read_line(&mut self.input).unwrap();
            let parts: Vec<&str> = self.input.split_whitespace().collect();
            match parts.as_slice() {
                [] => {}
                [cmd, args @ ..] => match *cmd {
                    "echo" => Shell::echo(args),
                    "exit" => Shell::exit(args),
                    "type" => self.type_of(args),
                    _ => self.execute(cmd, args),
                },
            };
        }
    }

    fn find_executable(&self, cmd: &str) -> Option<PathBuf> {
        for dir in &self.path_dirs {
            let full_path = dir.join(cmd);
            if full_path.is_file() && full_path.is_executable() {
                return Some(full_path);
            }
        }
        None
    }

    fn echo(args: &[&str]) {
        println!("{}", args.join(" ").trim());
    }

    fn exit(args: &[&str]) {
        if args.is_empty() {
            eprintln!("exit: missing exit code");
            return;
        }

        match args[0].parse() {
            Ok(code) => process::exit(code),
            Err(_) => eprintln!("exit: numeric argument required"),
        }
    }

    fn type_of(&self, args: &[&str]) {
        let cmd = args[0];
        if self.builtin.contains(&cmd) {
            println!("{} is a shell builtin", cmd);
        } else {
            match self.find_executable(cmd) {
                Some(path) => println!("{} is {}", cmd, path.display()),
                None => println!("{}: not found", cmd),
            }
        }
    }

    fn execute(&self, cmd: &str, args: &[&str]) {
        match self.find_executable(cmd) {
            None => println!("{}: command not found", cmd),
            Some(cmd) => {
                Command::new(cmd)
                    .args(args)
                    .status()
                    .expect("failed to execute process");
            }
        }
    }
}

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
