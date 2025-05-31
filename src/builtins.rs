use std::{collections::HashSet, env, fs, path::PathBuf};

pub struct Shell {
    builtins: HashSet<&'static str>,
    path: Option<String>,
}

impl Shell {
    pub fn new() -> Self {
        let builtins: HashSet<&'static str> = ["exit", "echo", "pwd", "cd", "type"]
            .iter()
            .cloned()
            .collect();

        Shell {
            builtins,
            path: env::var("PATH").ok(),
        }
    }

    fn exit(args: Vec<String>) {
        if args.is_empty() {
            std::process::exit(0)
        }
        std::process::exit(args[0].parse::<i32>().unwrap_or(0));
    }

    fn echo(args: Vec<String>) {
        if args.is_empty() {
            println!();
        } else {
            println!("{}", args.join(" "));
        }
    }

    fn pwd() {
        if let Ok(current_dir) = std::env::current_dir() {
            if let Some(path) = current_dir.to_str() {
                println!("{}", path);
            } else {
                eprintln!("Error: Unable to convert current directory to string");
            }
        } else {
            eprintln!("Error: Unable to get current directory");
        }
    }

    fn cd(args: Vec<String>) {
        let target_dir = if args.is_empty() || args[0] == "~" {
            if let Some(home) = env::var_os("HOME") {
                &home.into_string().unwrap_or_else(|_| String::from("/"))
            } else {
                eprintln!("Error: HOME environment variable not set");
                return;
            }
        } else {
            &args[0]
        };
        if let Err(_) = std::env::set_current_dir(target_dir) {
            eprintln!("cd: {}: No such file or directory", target_dir);
        }
    }

    fn type_of(&self, args: Vec<String>) {
        if args.is_empty() {
            eprintln!("Error: No command specified");
            return;
        }

        for cmd in args {
            match cmd.as_str() {
                b if self.builtins.contains(&b) => println!("{} is a shell builtin", b),
                o => match self.find_executable(o) {
                    Some(path) => println!("{} is {}", o, path.display()),
                    None => println!("{}: not found", o),
                },
            }
        }
    }

    fn find_executable(&self, cmd: &str) -> Option<PathBuf> {
        if let Some(path) = self.path.as_ref() {
            for dir in env::split_paths(path) {
                if let Ok(full_path) = fs::read_dir(&dir) {
                    for entry in full_path {
                        if let Ok(entry) = entry {
                            if entry.file_name() == cmd && entry.path().is_file() {
                                return Some(entry.path());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn run_builtin(&self, command: &str, args: Vec<String>) {
        match command {
            "exit" => Self::exit(args),
            "echo" => Self::echo(args),
            "pwd" => Self::pwd(),
            "cd" => Self::cd(args),
            "type" => self.type_of(args),
            _ => eprintln!("{}: command not found", command),
        }
    }

    fn run_executable(&self, command: &str, args: Vec<String>) {
        if let Some(path) = self.find_executable(command) {
            let mut cmd = std::process::Command::new(path.file_name().unwrap());
            cmd.args(args);
            if let Ok(output) = cmd.output() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            };
        } else {
            eprintln!("{}: commnand not found", command);
        }
    }

    pub fn run_command(&self, command: &str, args: Vec<String>) {
        if self.builtins.contains(command) {
            self.run_builtin(command, args);
        } else {
            self.run_executable(command, args);
        }
    }
}
