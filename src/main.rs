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
        let builtin = vec!["echo", "type", "exit", "pwd", "cd"];
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
                    "pwd" => Shell::pwd(),
                    "cd" => Shell::cd(args),
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
            Some(_) => {
                Command::new(cmd)
                    .args(args)
                    .status()
                    .expect("failed to execute process");
            }
        }
    }

    fn pwd() {
        match env::current_dir() {
            Ok(path) => println!("{}", path.display()),
            Err(e) => eprintln!("pwd: {}", e),
        }
    }

    fn cd(args: &[&str]) {
        if args.is_empty() {
            let home = env::var("HOME");
            if let Ok(home) = home {
                if env::set_current_dir(&home).is_err() {
                    eprintln!("cd: {}: No such file or directory", home);
                }
            } else {
                eprintln!("cd: HOME not set");
            }
        }

        if args.len() > 1 {
            eprintln!("cd: too many arguments");
            return;
        }

        let path = PathBuf::from(expand_tilde(args[0]));
        if env::set_current_dir(&path).is_err() {
            eprintln!("cd: {}: No such file or directory", path.display());
        }
    }
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with('~') {
        let home = env::var("HOME").unwrap_or_default();
        format!("{}{}", home, &path.strip_prefix('~').unwrap_or_default())
    } else {
        path.to_string()
    }
}

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
