use std::{
    collections::HashSet,
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use crate::{ast::Redirection, completer::ShellCompleter};

#[derive(Clone)]
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

    pub fn builtin_redirection(redirs: &[Redirection]) -> (Option<File>, Option<File>) {
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

    pub fn exit(args: Vec<String>) {
        if args.is_empty() {
            std::process::exit(0)
        }
        std::process::exit(args[0].parse::<i32>().unwrap_or(0));
    }

    pub fn echo(args: Vec<String>, mut output: Box<dyn Write>) {
        writeln!(output, "{}", args.join(" ")).unwrap_or_else(|_| {
            eprintln!("Failed to write to output");
        });
    }

    pub fn pwd(mut output: Box<dyn Write>) {
        if let Ok(current_dir) = std::env::current_dir() {
            if let Some(path) = current_dir.to_str() {
                writeln!(output, "{}", path).unwrap_or_else(|_| {
                    eprintln!("Failed to write to output");
                });
            }
        }
    }

    pub fn cd(args: Vec<String>, mut error: Box<dyn Write>) {
        if args.is_empty() {
            std::env::set_current_dir(std::env::home_dir().unwrap()).unwrap();
            return;
        }

        let target_dir = args[0].replace("~", std::env::home_dir().unwrap().to_str().unwrap_or(""));

        if std::env::set_current_dir(&target_dir).is_err() {
            writeln!(error, "cd: {}: No such file or directory", target_dir).unwrap_or_else(|_| {
                eprintln!("Failed to write to error output");
            });
        }
    }

    pub fn type_of(
        &self,
        args: Vec<String>,
        mut output: Box<dyn Write>,
        mut error: Box<dyn Write>,
    ) {
        for cmd in args {
            match cmd.as_str() {
                b if self.builtins.contains(&b) => writeln!(output, "{} is a shell builtin", b)
                    .unwrap_or_else(|_| {
                        eprintln!("Failed to write to output");
                    }),
                o => match self.find_executable(o) {
                    Some(path) => {
                        writeln!(output, "{} is {}", o, path.display()).unwrap_or_else(|_| {
                            eprintln!("Failed to write to output");
                        })
                    }
                    None => writeln!(error, "{}: not found", o).unwrap_or_else(|_| {
                        eprintln!("Failed to write to error output");
                    }),
                },
            };
        }
    }

    fn find_executable(&self, cmd: &str) -> Option<PathBuf> {
        if let Some(path) = self.path.as_ref() {
            for dir in env::split_paths(path) {
                if let Ok(full_path) = fs::read_dir(&dir) {
                    for entry in full_path {
                        let Ok(entry) = entry else { continue };
                        let path = entry.path();
                        if entry.file_name() == cmd
                            && path.is_file()
                            && is_executable(path.as_path())
                        {
                            return Some(entry.path());
                        }
                    }
                }
            }
        }
        None
    }

    pub fn is_builtin(&self, cmd: &str) -> bool {
        self.builtins.contains(cmd)
    }

    fn collect_path_executables(&self) -> HashSet<String> {
        let mut executables = HashSet::new();
        if let Some(path) = &self.path {
            for dir in env::split_paths(&path) {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() && is_executable(path.as_path()) {
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
