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
            let parts = self.input.split_once(" ");
            let (cmd, args) = match parts {
                None => (self.input.as_str(), None),
                Some((cmd, args)) => {
                    let args = args.trim();
                    let args = parse_args(args);
                    (cmd, Some(args))
                }
            };

            let cmd = cmd.trim();
            let args = &args.unwrap_or_default();
            match cmd {
                "echo" => Shell::echo(args),
                "exit" => Shell::exit(args),
                "type" => self.type_of(args),
                "pwd" => Shell::pwd(),
                "cd" => Shell::cd(args),
                _ => self.execute(cmd, args),
            }
        }
    }

    fn find_executable(&self, cmd: &String) -> Option<PathBuf> {
        for dir in &self.path_dirs {
            let full_path = dir.join(cmd);
            if full_path.is_file() && full_path.is_executable() {
                return Some(full_path);
            }
        }
        None
    }

    fn echo(args: &[String]) {
        println!("{}", args.join(" ").trim());
    }

    fn exit(args: &[String]) {
        if args.is_empty() {
            eprintln!("exit: missing exit code");
            return;
        }

        match args[0].parse() {
            Ok(code) => process::exit(code),
            Err(_) => eprintln!("exit: numeric argument required"),
        }
    }

    fn type_of(&self, args: &[String]) {
        let cmd = &args[0];
        if self.builtin.iter().any(|&builtin| builtin == cmd) {
            println!("{} is a shell builtin", cmd);
        } else {
            match self.find_executable(cmd) {
                Some(path) => println!("{} is {}", cmd, path.display()),
                None => println!("{}: not found", cmd),
            }
        }
    }

    fn execute(&self, cmd: &str, args: &[String]) {
        match self.find_executable(&cmd.to_string()) {
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

    fn cd(args: &[String]) {
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

        let path = PathBuf::from(expand_tilde(&args[0]));
        if env::set_current_dir(&path).is_err() {
            eprintln!("cd: {}: No such file or directory", path.display());
        }
    }
}

fn expand_tilde(path: &String) -> String {
    if path.starts_with('~') {
        let home = env::var("HOME").unwrap_or_default();
        format!("{}{}", home, &path.strip_prefix('~').unwrap_or_default())
    } else {
        path.to_string()
    }
}

fn parse_args(args: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut chars = args.chars().peekable();
    let mut is_in_quote = false;
    while let Some(&ch) = chars.peek() {
        match ch {
            '\'' => {
                chars.next();
                is_in_quote = !is_in_quote;
            }
            ' ' | '\t' => {
                chars.next();
                if is_in_quote {
                    token.push(ch);
                } else if !token.is_empty() {
                    tokens.push(token.clone());
                    token.clear();
                }
            }
            _ => {
                token.push(ch);
                chars.next();
            }
        }
    }

    if !token.is_empty() {
        tokens.push(token);
    }

    tokens
}

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
