use std::{
    collections::HashSet,
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use crate::{
    ast::{apply_redirection, CommandSegment, Redirection},
    completer::ShellCompleter,
};

macro_rules! write_or_stdout {
    ($file_opt:expr, $($arg:tt)*) => {
        if let Some(file) = $file_opt.as_mut() {
            writeln!(file, $($arg)*).unwrap();
        } else {
            println!($($arg)*);
        }
    };
}

macro_rules! write_or_stderr  {
    ($file_opt:expr, $($arg:tt)*) => {
        if let Some(file) = $file_opt.as_mut() {
            writeln!(file, $($arg)*).unwrap();
        } else {
            eprintln!($($arg)*);
        }
    };
}

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

    fn builtin_redirection(redirs: &[Redirection]) -> (Option<File>, Option<File>) {
        let mut output = None;
        let mut error = None;

        for redir in redirs {
            match redir {
                Redirection::Stdout(path) => {
                    if let Ok(file) = File::create(path) {
                        output = Some(file);
                    }
                }
                Redirection::StdoutAppend(path) => {
                    if let Ok(file) = OpenOptions::new().append(true).create(true).open(path) {
                        output = Some(file);
                    }
                }
                Redirection::Stderr(path) => {
                    if let Ok(file) = File::create(path) {
                        error = Some(file);
                    }
                }
                Redirection::StderrAppend(path) => {
                    if let Ok(file) = OpenOptions::new().append(true).create(true).open(path) {
                        error = Some(file);
                    }
                }
                Redirection::Stdin(_) => {}
            }
        }

        (output, error)
    }

    fn exit(args: Vec<String>) {
        if args.is_empty() {
            std::process::exit(0)
        }
        std::process::exit(args[0].parse::<i32>().unwrap_or(0));
    }

    fn echo(args: Vec<String>, mut output: Option<File>) {
        write_or_stdout!(output, "{}", args.join(" "));
    }

    fn pwd(mut output: Option<File>) {
        if let Ok(current_dir) = std::env::current_dir() {
            if let Some(path) = current_dir.to_str() {
                write_or_stdout!(output, "{}", path);
            }
        }
    }

    fn cd(args: Vec<String>, mut error: Option<File>) {
        if args.is_empty() {
            std::env::set_current_dir(std::env::home_dir().unwrap()).unwrap();
            return;
        }

        let target_dir = args[0].replace("~", std::env::home_dir().unwrap().to_str().unwrap_or(""));

        if let Err(_) = std::env::set_current_dir(&target_dir) {
            write_or_stderr!(error, "cd: {}: No such file or directory", target_dir);
        }
    }

    fn type_of(&self, args: Vec<String>, mut output: Option<File>, mut error: Option<File>) {
        for cmd in args {
            match cmd.as_str() {
                b if self.builtins.contains(&b) => {
                    write_or_stdout!(output, "{} is a shell builtin", b)
                }
                o => match self.find_executable(o) {
                    Some(path) => write_or_stdout!(output, "{} is {}", o, path.display()),
                    None => write_or_stderr!(error, "{}: not found", o),
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

    fn run_builtin(&self, command: CommandSegment) {
        let (output, mut error) = Self::builtin_redirection(&command.redirections);
        match command.cmd.as_str() {
            "exit" => Self::exit(command.args),
            "echo" => Self::echo(command.args, output),
            "pwd" => Self::pwd(output),
            "cd" => Self::cd(command.args, error),
            "type" => self.type_of(command.args, output, error),
            _ => write_or_stderr!(error, "{}: command not found", command.cmd),
        }
    }

    fn run_executable(&self, command: CommandSegment) {
        let (_, mut error) = Self::builtin_redirection(&command.redirections);
        if let Some(path) = self.find_executable(&command.cmd) {
            let excutable = path.file_name().unwrap();
            let mut cmd = std::process::Command::new(excutable);
            cmd.args(command.args);
            if let Err(e) = apply_redirection(&mut cmd, &command.redirections) {
                eprintln!("Redirection error: {}", e);
            }
            cmd.status().expect("command cannot be executed");
        } else {
            write_or_stderr!(error, "{}: commnand not found", command.cmd);
        }
    }

    pub fn run_command(&self, command: CommandSegment) {
        if self.builtins.contains(command.cmd.as_str()) {
            self.run_builtin(command);
        } else {
            self.run_executable(command);
        }
    }

    fn collect_path_executables(&self) -> HashSet<String> {
        let mut executables = HashSet::new();
        if let Some(path) = &self.path {
            for dir in env::split_paths(&path) {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() && is_executable(&path.as_path()) {
                            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                                executables.insert(name.to_string());
                            }
                        }
                    }
                }
            }
        };
        executables
    }

    pub fn initialize_completer(&self) -> ShellCompleter {
        let mut commands: HashSet<String> = self.builtins.iter().map(|s| s.to_string()).collect();
        let executables = self.collect_path_executables();
        commands.extend(executables);
        ShellCompleter::new(commands)
    }
}

fn is_executable(path: &Path) -> bool {
    if let Ok(metadata) = path.metadata() {
        let perm = metadata.permissions();
        perm.mode() & 0o111 != 0
    } else {
        false
    }
}
