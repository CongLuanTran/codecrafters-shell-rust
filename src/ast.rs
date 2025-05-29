#![allow(unused)]
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
