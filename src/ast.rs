#![allow(unused)]
use std::fs::OpenOptions;

use std::{fs::File, path, process::Command};
#[derive(Debug)]
pub struct CommandLine {
    pub pipelines: Vec<Pipeline>,
}

#[derive(Debug)]
pub struct Pipeline {
    pub segments: Vec<CommandSegment>,
}

#[derive(Debug)]
pub struct CommandSegment {
    pub cmd: String,
    pub args: Vec<String>,
    pub redirections: Vec<Redirection>,
}

#[derive(Debug)]
pub enum Redirection {
    Stdout(String),       // >
    StdoutAppend(String), // >>
    Stderr(String),       // 2>
    StderrAppend(String), // 2>>
    Stdin(String),        // <
}

pub fn apply_redirection(cmd: &mut Command, redirs: &[Redirection]) -> std::io::Result<()> {
    for redir in redirs {
        match redir {
            Redirection::Stdout(path) => {
                let file = File::create(path)?;
                cmd.stdout(file);
            }
            Redirection::StdoutAppend(path) => {
                let file = OpenOptions::new().append(true).create(true).open(path)?;
                cmd.stdout(file);
            }
            Redirection::Stderr(path) => {
                let file = File::create(path)?;
                cmd.stderr(file);
            }
            Redirection::StderrAppend(path) => {
                let file = OpenOptions::new().append(true).create(true).open(path)?;
                cmd.stdout(file);
            }
            Redirection::Stdin(path) => {
                let file = File::open(path)?;
                cmd.stdin(file);
            }
        }
    }
    Ok(())
}
